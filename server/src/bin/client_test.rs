use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::{BytesCodec, Framed};

use clap::Parser;
use protocol::{Frame, Message, User};
use server::cli::Cli;
use std::error::Error;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    // Open a TCP stream to the socket address.
    //
    // Note that this is the Tokio TcpStream, which is fully async.
    let stream = TcpStream::connect("127.0.0.1:9999").await?;
    let chat = Framed::new(stream, BytesCodec::new());
    let (mut sink, mut stream) = chat.split();

    let args = Cli::parse();

    let user = User {
        username: args.user,
        color: args.color,
        password: "1234".to_owned(),
    };

    tokio::spawn(async move {
        loop {
            let resp = stream.next().await.unwrap().unwrap();
            let message: Frame = resp.freeze().try_into().unwrap();
            match message {
                Frame::Message(message) => {
                    println!("{}\x07", &message);
                }
                Frame::Bulk(messages) => {
                    for message in messages.iter() {
                        println!("{}\x07", &message);
                    }
                }
                Frame::Error(err) => {
                    println!("err: {err}");
                    break;
                }
                _ => {
                    println!("idk: {message:?}");
                }
            }
        }
    });

    let connect_message = Frame::Connect(user.clone(), args.room);

    let bytes: Bytes = connect_message.try_into().unwrap();

    let _ = sink.send(bytes).await;

    loop {
        let mut inp = String::new();
        std::io::stdin().read_line(&mut inp).unwrap();
        let inp = inp.trim().to_owned();
        let message = Frame::Message(Message::new(user.clone(), None, inp));

        let bytes: Bytes = message.try_into().unwrap();
        let _ = sink.send(bytes).await;
    }
}
