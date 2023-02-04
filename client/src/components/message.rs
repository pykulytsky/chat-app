use crate::CURRENT_USER;
use dioxus::prelude::*;
use fermi::prelude::*;

#[derive(PartialEq, Props)]
pub struct MessageProps {
    pub left: bool,
    pub message: protocol::Message,
}

pub fn Message(cx: Scope<MessageProps>) -> Element {
    let current_user = use_atom_state(cx, CURRENT_USER);
    let left =
        current_user.current().as_ref().clone().unwrap().username != cx.props.message.from.username;
    cx.render(rsx! {
        div {
            class: "chat-message transition-all ease-in-out delay-150",
            div {
                class: if left {"flex items-end"} else {"flex items-end justify-end"},
                div {
                    class: if left {"flex flex-col space-y-2 text-xs max-w-xs mx-2 order-2 items-start"} else {
                        "flex flex-col space-y-2 text-xs max-w-xs mx-2 order-1 items-end"
                    },
                    div {
                        div {
                            class: if left {
                                "px-4 my-2 py-2 rounded-lg inline-block rounded-bl-none bg-gray-300 text-gray-600 "
                            } else {
                                "px-4 my-2 py-2 rounded-lg inline-block rounded-br-none bg-blue-600 text-white "
                            },
                            p {
                                class: "font-extrabold",
                                "{cx.props.message.from.username}"
                            }
                            p {
                                "{cx.props.message.body}"
                            }
                        }
                    }
                }
                img {
                    src: "https://images.unsplash.com/photo-1549078642-b2ba4bda0cdb?ixlib=rb-1.2.1&amp;ixid=eyJhcHBfaWQiOjEyMDd9&amp;auto=format&amp;fit=facearea&amp;facepad=3&amp;w=144&amp;h=144", alt:"My profile", class:"w-8 h-8 rounded-full order-1"
                }
            }
        }
    })
}
