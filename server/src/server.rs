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

pub type Tx = mpsc::UnboundedSender<Frame>;
type Rx = mpsc::UnboundedReceiver<Frame>;

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

    pub fn with_peers(name: String, cover: Option<String>, peers: HashMap<SocketAddr, Tx>) -> Self {
        Shared {
            peers,
            name,
            cover,
            messages: vec![],
        }
    }

    /// Send a `LineCodec` encoded message to every peer, except
    /// for the sender.
    async fn broadcast(&mut self, sender: SocketAddr, frame: &Frame) {
        for peer in self.peers.iter_mut() {
            if *peer.0 != sender {
                let _ = peer.1.send(frame.clone());
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
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env().add_directive("server=info".parse()?))
            .with_span_events(FmtSpan::FULL)
            .init();

        tracing::info!("server running on {}", addr);

        let addr = addr.to_socket_addrs().unwrap().next().unwrap();
        let listener = TcpListener::bind(addr).await?;

        let mut channels = HashMap::new();
        channels.insert("default".to_string(), Shared::new(
                "default".to_string(),
                Some("https://cdn-icons-png.flaticon.com/512/134/134932.png".to_string())
        ));
        channels.insert("another".to_string(), Shared::new(
                "another".to_string(),
                Some("https://cdn-icons-png.flaticon.com/512/134/134932.png".to_string())
        ));
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
            _ => {
                tracing::error!("Failed to get username from {}. Client disconnected.", addr);
                return Ok(());
            }
        };

        let mut peer = Peer::new(state.clone(), chat).await?;

        loop {
            tokio::select! {
                // A message was received from a peer. Send it to the current user.
                Some(frame) = peer.rx.recv() => {
                    let bytes: Bytes = frame.try_into().unwrap();
                    peer.stream.send(bytes).await?;
                }
                result = peer.stream.next() => match result {
                    // A message was received from the current user, we should
                    // broadcast this message to the other users.
                    Some(Ok(msg)) => {
                        let frame: Frame = msg.freeze().try_into().unwrap();
                        match frame {
                            Frame::Message(msg) => {
                                let frame = Frame::Message(msg.clone());
                                let bytes: Bytes = frame.clone().try_into().unwrap();
                                peer.stream.send(bytes).await?;

                                state
                                    .lock()
                                    .await
                                    .get_mut(&msg.channel)
                                    .unwrap()
                                    .broadcast(addr, &frame)
                                    .await;
                                state
                                    .lock()
                                    .await
                                    .get_mut(&msg.channel)
                                    .unwrap()
                                    .messages
                                    .push(msg);
                            },
                            Frame::Channel(channel) => {
                                let name = &channel.name.to_owned();
                                
                                let peers: HashMap<SocketAddr, Tx>  = state
                                    .lock()
                                    .await
                                    .get("default")
                                    .unwrap()
                                    .peers
                                    .iter()
                                    .map(|(addr, tx)| (addr.clone(), tx.clone()))
                                    .collect();
                                state
                                    .lock()
                                    .await
                                    .insert(
                                        name.to_string(),
                                        Shared::with_peers(
                                            name.to_string(), 
                                            channel.cover.to_owned(),
                                            peers
                                        )
                                    );

                                
                                let channels: Vec<Channel> = self.channels.lock().await.iter().map(|(_, v)| {
                                    Channel {
                                        name: v.name.to_owned(),
                                        cover: v.cover.to_owned(),
                                        messages: v.messages.to_owned(),
                                    }
                                }).collect();
                                let frame = Frame::Bulk(vec![], channels);
                                let bytes: Bytes = frame.clone().try_into().unwrap();
                                peer.stream.send(bytes).await.unwrap();
                                state
                                    .lock()
                                    .await
                                    .get_mut("default")
                                    .unwrap()
                                    .broadcast(addr, &frame)
                                    .await;
                            },
                            _ => {

                            }
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

        Ok(())
    }
}
