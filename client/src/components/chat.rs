use super::message::Message;
use dioxus::prelude::*;
use protocol::Message;

#[derive(PartialEq, Props)]
pub struct ChatProps {
    pub messages: Vec<Message>,
}

pub fn Chat(cx: Scope<ChatProps>) -> Element {
    let messages = cx.props.messages.iter().map(|message| {
        rsx!(Message {
            left: true,
            message: message.clone()
        })
    });
    cx.render(rsx! {
        div {
            id: "messages",
            class:"flex flex-col space-y-4 p-3 overflow-y-auto scrollbar-thumb-blue scrollbar-thumb-rounded scrollbar-track-blue-lighter scrollbar-w-2 scrolling-touch",
            messages
        }
    })
}
