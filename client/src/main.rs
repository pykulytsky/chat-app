use std::collections::HashMap;

use dioxus::prelude::*;

use components::{Chat, Header, Login, Sidebar};
use futures::{SinkExt, StreamExt, executor::block_on};
use tokio::{net::TcpStream, select};
use tokio_util::codec::{BytesCodec, Framed};

use bytes::Bytes;
use protocol::{Frame, Message, User};
use fermi::prelude::*;

use tokio::sync::Notify;
use std::sync::Arc;

mod components;

fn main() {
    dioxus_desktop::launch(app);
}

type MessagesStorage = HashMap<String, Vec<Message>>;

pub static CURRENT_USER: Atom<Option<User>> = |_| None;
pub static CURRENT_CHANNEL: Atom<Option<String>> = |_| None;
pub static MESSAGES: Atom<Vec<Message>> = |_| Vec::new();

fn app(cx: Scope) -> Element {
    use_init_atom_root(cx);
    let user = use_atom_state(cx, CURRENT_USER);
    let channel = use_atom_state(cx, CURRENT_CHANNEL);

    let notify = Arc::new(Notify::new());
    let block = notify.clone();

    let messages = use_state(cx, Vec::<Message>::new);
    let values = messages.clone();
    let message = use_state(cx, String::new);

    let server_tx = use_coroutine(&cx, |mut rx: UnboundedReceiver<Frame>| async move {
        block.notified().await;
        let stream = TcpStream::connect("127.0.0.1:9999").await.unwrap();
        let chat = Framed::new(stream, BytesCodec::new());
        let (mut sink, mut stream) = chat.split();

        let login_frame = block_on(rx.next()).unwrap();

        if let Frame::Authorize(user) = login_frame {
            let connect_message = Frame::Connect(user.clone(), Some("test".to_string()));
            let bytes: Bytes = connect_message.try_into().unwrap();
            let _ = sink.send(bytes).await;
        } else {
            println!("wrong");
        }

        loop {
            select! {
                Some(msg) = rx.next() => {
                    let bytes: Bytes = msg.try_into().unwrap();
                    sink.send(bytes).await.unwrap();
                }
                result = stream.next() => match result {
                    Some(Ok(msg)) => {
                        let message: Frame = msg.freeze().try_into().unwrap();
                        match message {
                            Frame::Message(message) => {
                                let values = values.clone();
                                values.with_mut(|v| {
                                    v.push(message);
                                });
                            },
                            Frame::Bulk(messages) => {
                                let values = values.clone();
                                values.with_mut(|v| {
                                    v.extend_from_slice(&messages);
                                });
                            },
                            Frame::Error(_) => {
                                break;
                            }
                            _ => {
                            }
                        }
                    },
                    Some(Err(_)) => {
                    }
                    None => {
                    }
                },
            }
        }
    });

    let tx1 = server_tx.clone();
    let login_tx = server_tx.clone();

    let chat = if channel.current().is_some() {
        cx.render(rsx!{
            Header {
                score: 1
            }
            Chat {
                messages: messages.clone().to_vec()
            }

        div {
            class: "border-t-2 border-gray-200 px-4 pt-4 mb-2 sm:mb-0",
            div {
                class: "relative flex",
                input {
                    placeholder: "Write your message!", 
                    class: "w-full focus:outline-none focus:placeholder-gray-400 text-gray-600 placeholder-gray-600 px-6 bg-gray-200 rounded-md py-3",
                    "type": "text",
                    value: "{message}",
                    oninput: move |evt| message.set(evt.value.clone())
                }
                div {
                    class: "absolute right-0 items-center inset-y-0 sm:flex",
                    button {
                        class: "z-40 inline-flex items-center justify-center rounded-lg px-4 py-3 transition duration-500 ease-in-out text-white bg-blue-500 hover:bg-blue-400 focus:outline-none",
                        onclick: move |_| {

                            let message = Frame::Message(Message::new(
                                user.as_ref().unwrap().clone(),
                                None,
                                message.clone().to_string()
                            ));
                            tx1.send(message)
                        },
                        span {
                            class: "font-bold",
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                view_box: "0 0 20 20",
                                fill: "currentColor",
                                class: "h-6 w-6 ml-2 transform rotate-90",
                                path {
                                    d: "M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z"
                                }
                            }
                        }
                    }
                }
            }
        }
        })
    } else {
        None
    };

    if user.current().is_none() {
        cx.render(rsx! {
            style { include_str!("../css/tailwind_compiled.css") }
            Login {
                onsubmit: move |(username, password)| {
                    let logged_in = User {
                        username,
                        password,
                        color: None
                    };
                    user.modify(|_| Some(logged_in.clone()));
                    login_tx.send(Frame::Authorize(logged_in));
                    notify.notify_one();
                }
            }
        })
    } else {
        cx.render(rsx! (
            style { include_str!("../css/tailwind_compiled.css") }
            
            Sidebar {
                onselect: move |v| { }
            }
            div {
                class: "ml-64 flex-1 p:2 sm:p-6 justify-between flex flex-col h-screen",

                chat
            }
        ))
    }
}
