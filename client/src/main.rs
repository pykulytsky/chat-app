use std::collections::HashMap;

use dioxus::prelude::*;

use components::{Chat, Login, Sidebar};
use futures::{SinkExt, StreamExt, executor::block_on};
use tokio::{net::TcpStream, select};
use tokio_util::codec::{BytesCodec, Framed};

use bytes::Bytes;
use protocol::{Frame, Message, User, Channel};
use fermi::prelude::*;

use tokio::sync::Notify;
use std::sync::Arc;

mod components;

fn main() {
    dioxus_desktop::launch(app);
}

pub static CURRENT_USER: Atom<Option<User>> = |_| None;
pub static CURRENT_CHANNEL: Atom<Option<String>> = |_| None;
pub static MESSAGES: Atom<Vec<Message>> = |_| Vec::new();
pub static CHANNELS: Atom<HashMap<String, Channel>> = |_| HashMap::new();

fn app(cx: Scope) -> Element {
    use_init_atom_root(cx);
    let user = use_atom_state(cx, CURRENT_USER);
    let channel = use_atom_state(cx, CURRENT_CHANNEL);
    let channels = use_atom_state(cx, CHANNELS);
    let channels_state = use_state(cx, HashMap::<String, Channel>::new);
    let channels_state_clone = channels_state.clone();

    let notify = Arc::new(Notify::new());
    let block = notify.clone();

    let chnls = channels.clone();
    let chnls1 = channels.clone();
    let message = use_state(cx, String::new);

    let server_tx = use_coroutine(&cx, |mut rx: UnboundedReceiver<Frame>| async move {
        block.notified().await;
        let stream = TcpStream::connect("127.0.0.1:9999").await.unwrap();
        let chat = Framed::new(stream, BytesCodec::new());
        let (mut sink, mut stream) = chat.split();

        let login_frame = block_on(rx.next()).unwrap();

        if let Frame::Authorize(_) = login_frame {
            let bytes: Bytes = login_frame.try_into().unwrap();
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
                                let channels = channels_state_clone.clone();
                                // chnls1.with_mut(|chnls| {
                                //     chnls.get_mut(&message.channel).unwrap().messages.push(message);
                                // });
                                channels.with_mut(|chnls| {
                                    chnls.get_mut(&message.channel).unwrap().messages.push(message);
                                });
                                chnls1.set(HashMap::from_iter(channels.current().clone().iter().map(|ch| (ch.0.to_owned(), ch.1.clone()))));
                            },
                            Frame::Bulk(_, chnls) => {
                                let channels = channels_state_clone.clone();
                                chnls1.with_mut(|chnl| {
                                    chnl.extend(chnls.iter().map(|ch| {
                                        (ch.name.to_owned(), ch.clone())
                                    }));
                                });
                                channels.with_mut(|chnl| {
                                    chnl.extend(chnls.iter().map(|ch| {
                                        (ch.name.to_owned(), ch.clone())
                                    }));
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
    let sidebar_tx = server_tx.clone();
    
    let chat = if channel.current().is_some() {
        cx.render(rsx!{
            // Header {
            //     name: channel.as_ref().unwrap().to_string(),
            //     cover: chnls.clone().current().get(channel.as_ref().unwrap()).unwrap().cover.clone().unwrap().to_string()
            // }
            Chat {
                messages: chnls.clone().current().get(channel.as_ref().unwrap()).unwrap().messages.clone()
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
                                channel.as_ref().unwrap().clone(),
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
                onsubmit: move |(username, avatar): (String, String)| {
                    let logged_in = User {
                        username,
                        color: None,
                        avatar: if avatar.len() > 0 {
                            Some(avatar)
                        } else {
                            Some("https://w7.pngwing.com/pngs/754/2/png-transparent-samsung-galaxy-a8-a8-user-login-telephone-avatar-pawn-blue-angle-sphere-thumbnail.png".to_string())
                        },
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
                onselect: move |_| { },
                onsubmit: move |(name, cover): (String, String)| {
                    let channel = Frame::Channel(Channel {
                        name,
                        cover: if cover.len() > 0 {Some(cover)} else {
                            Some("https://cdn-icons-png.flaticon.com/512/134/134932.png".to_string())
                        },
                        messages: vec![]
                    });
                    sidebar_tx.send(channel);
                }
            }
            div {
                class: "ml-64 flex-1 p:2 sm:p-6 justify-between flex flex-col h-screen",

                chat
            }
        ))
    }
}
