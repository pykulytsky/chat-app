use bytes::Bytes;
use std::{
    collections::HashMap,
    io,
    net::{SocketAddr, ToSocketAddrs},
    sync::Arc,
};

use futures::{SinkExt, StreamExt};
use tokio::sync::{mpsc, Mutex};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{OwnedSemaphorePermit, Semaphore, TryAcquireError},
};
use tokio_util::codec::{BytesCodec, Framed};

use protocol::{ConnectionError, Frame, Message, Channel};

pub type Tx = mpsc::UnboundedSender<Message>;
type Rx = mpsc::UnboundedReceiver<Message>;

const MAX_CONNECTIONS: usize = 64;

#[derive(Debug)]
pub struct Shared {
    pub peers: HashMap<SocketAddr, Tx>,
    pub messages: Vec<Message>,
    pub name: String,
    pub cover: Option<String>
}

impl Shared {
    /// Creates a new shared state for peer.
    /// ```
    /// let shared = chat::server::Shared::new();
    ///
    /// assert_eq!(shared.peers.len(), 0);
    /// ```
    pub fn new(name: String, cover: Option<String>) -> Self {
        Shared {
            peers: HashMap::new(),
            name,
            cover,
            messages: vec![],
        }
    }

    /// Send a `LineCodec` encoded message to every peer, except
    /// for the sender.
    async fn broadcast(&mut self, sender: SocketAddr, message: &Message) {
        for peer in self.peers.iter_mut() {
            if *peer.0 != sender {
                let _ = peer.1.send(message.clone());
            }
        }
    }
}

struct Peer {
    rx: Rx,
    // stream: Framed<TcpStream, ChatCodec>,
    stream: Framed<TcpStream, BytesCodec>,
}

impl Peer {
    /// Create a new instance of `Peer`.
    pub async fn new(
        state: Arc<Mutex<HashMap<String, Shared>>>,
        stream: Framed<TcpStream, BytesCodec>,
    ) -> io::Result<Peer> {
        // Get the client socket address
        let addr = stream.get_ref().peer_addr()?;

        // Create a channel for this peer
        let (tx, rx) = mpsc::unbounded_channel();

        // Add an entry for this `Peer` in the shared state map.
        state
            .lock()
            .await
            .iter_mut()
            .for_each(|(_, v)| {
                v
                .peers
                .insert(addr, tx.clone());
            });

        Ok(Peer { stream, rx })
    }
}

#[derive(Debug)]
pub struct Server {
    pub addr: SocketAddr,
    pub listener: TcpListener,
    pub channels: Arc<Mutex<HashMap<String, Shared>>>,
    pub messages: Arc<Mutex<Vec<Message>>>,
    pub max_connetions: Arc<Semaphore>,
}

impl Server {
    pub async fn bind(
        addr: impl ToSocketAddrs + std::fmt::Display,
    ) -> Result<&'static mut Self, ConnectionError> {
        use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};
        // Configure a `tracing` subscriber that logs traces emitted by the chat
        // server.
        tracing_subscriber::fmt()
            // Filter what traces are displayed based on the RUST_LOG environment
            // variable.
            //
            // Traces emitted by the example code will always be displayed. You
            // can set `RUST_LOG=tokio=trace` to enable additional traces emitted by
            // Tokio itself.
            .with_env_filter(EnvFilter::from_default_env().add_directive("server=info".parse()?))
            // Log events when `tracing` spans are created, entered, exited, or
            // closed. When Tokio's internal tracing support is enabled (as
            // described above), this can be used to track the lifecycle of spawned
            // tasks on the Tokio runtime.
            .with_span_events(FmtSpan::FULL)
            // Set this subscriber as the default, to collect all traces emitted by
            // the program.
            .init();

        tracing::info!("server running on {}", addr);

        let addr = addr.to_socket_addrs().unwrap().next().unwrap();
        let listener = TcpListener::bind(addr).await?;

        let mut channels = HashMap::new();
        channels.insert("default".to_string(), Shared::new("default".to_string(), None));
        channels.insert("another".to_string(), Shared::new("another".to_string(), None));
        let channels = Arc::new(Mutex::new(channels));

        Ok(Box::leak(Box::new(Server {
            addr,
            listener,
            channels,
            messages: Arc::new(Mutex::new(vec![])),
            max_connetions: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
        })))
    }

    pub async fn run(&'static self) -> Result<(), ConnectionError> {
        loop {
            // Asynchronously wait for an inbound TcpStream.
            let (stream, addr) = self.listener.accept().await?;

            // Clone a handle to the `Shared` state for the new connection.
            let peers = Arc::clone(&self.channels);
            let connections = Arc::clone(&self.max_connetions);

            // Spawn our handler to be run asynchronously.
            let permit = connections.try_acquire_owned();
            tokio::spawn(async move {
                tracing::debug!("accepted connection");
                if let Err(e) = self.process(peers, stream, addr, permit).await {
                    tracing::info!("an error occurred; error = {:?}", e);
                }
            });
        }
    }

    async fn process(
        &self,
        state: Arc<Mutex<HashMap<String, Shared>>>,
        stream: TcpStream,
        addr: SocketAddr,
        acquired_permit: Result<OwnedSemaphorePermit, TryAcquireError>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut chat = Framed::new(stream, BytesCodec::new());

        if acquired_permit.is_err() {
            let bytes: Bytes = Frame::Error("Max connections reached".to_string())
                .try_into()
                .unwrap();
            chat.send(bytes).await.unwrap();
            tracing::info!("{}: max connections reached", &addr);
            return Ok(());
        }

        let user = match chat.next().await {
            Some(Ok(bytes)) => {
                let frame: Frame = bytes.freeze().try_into().unwrap();
                if let Frame::Authorize(user) = frame {
                    let channels: Vec<Channel> = self.channels.lock().await.iter().map(|(_, v)| {
                        Channel {
                            name: v.name.to_owned(),
                            cover: v.cover.to_owned(),
                            messages: v.messages.to_owned(),
                        }
                    }).collect();
                    let bytes: Bytes = Frame::Bulk(vec![], channels).try_into().unwrap();
                    chat.send(bytes).await.unwrap();

                    user
                } else {
                    tracing::error!("Failed to get username from {}. Client disconnected.", addr);
                    return Ok(());
                }
            }
            // We didn't get a line so we return early here.
            _ => {
                tracing::error!("Failed to get username from {}. Client disconnected.", addr);
                return Ok(());
            }
        };

        // Register our peer with state which internally sets up some channels.
        let mut peer = Peer::new(state.clone(), chat).await?;

        // A client has connected, let's let everyone know.
        // {
        //     let msg = Message::new(
        //         user.clone(),
        //         None,
        //         format!("{} has joined the chat", user.username),
        //     );
        //
        //     state
        //         .lock()
        //         .await
        //         .get_mut(&channel)
        //         .unwrap()
        //         .broadcast(addr, &msg)
        //         .await;
        // }

        // Process incoming messages until our stream is exhausted by a disconnect.
        loop {
            tokio::select! {
                // A message was received from a peer. Send it to the current user.
                Some(msg) = peer.rx.recv() => {
                    let bytes: Bytes = Frame::Message(msg).try_into().unwrap();
                    peer.stream.send(bytes).await?;
                }
                result = peer.stream.next() => match result {
                    // A message was received from the current user, we should
                    // broadcast this message to the other users.
                    Some(Ok(msg)) => {
                        let message: Frame = msg.freeze().try_into().unwrap();
                        if let Frame::Message(msg) = message {
                            let bytes: Bytes = Frame::Message(msg.clone()).try_into().unwrap();
                            peer.stream.send(bytes).await?;

                            state.lock().await.get_mut(&msg.channel).unwrap().broadcast(addr, &msg).await;
                            state.lock().await.get_mut(&msg.channel).unwrap().messages.push(msg);
                            // let mut messages = messages.lock().await;
                            // messages.push(msg);
                        }
                    }
                    // An error occurred.
                    Some(Err(e)) => {
                        tracing::error!(
                            "an error occurred while processing messages for {}; error = {:?}",
                            user.username,
                            e
                        );
                    }
                    // The stream has been exhausted.
                    None => break,
                },
            }
        }

        // If this section is reached it means that the client was disconnected!
        // Let's let everyone still connected know about it.
        // {
        //     state
        //         .lock()
        //         .await
        //         .get_mut(&channel)
        //         .unwrap()
        //         .peers
        //         .remove(&addr);
        //
        //     let msg = Frame::Disconnect(user);
        //     tracing::info!("{}", msg);
        //     state
        //         .lock()
        //         .await
        //         .get_mut(&channel)
        //         .unwrap()
        //         .broadcast(addr, &msg.try_into().unwrap())
        //         .await;
        // }

        Ok(())
    }
}
