use crate::{CHANNELS, CURRENT_CHANNEL};

use super::message::Message;
use dioxus::prelude::*;
use fermi::use_atom_state;
use protocol::Message;

#[derive(PartialEq, Props)]
pub struct ChatProps {
    pub messages: Vec<Message>,
}

#[allow(non_snake_case)]
pub fn Chat(cx: Scope<ChatProps>) -> Element {
    let channels = use_atom_state(cx, CHANNELS);
    let current_channel = use_atom_state(cx, CURRENT_CHANNEL);
    let channel = current_channel.clone();
    let channel = channel.as_ref().unwrap();
    let messages = channels
        .get()
        .get(channel)
        .unwrap()
        .messages
        .iter()
        .map(|message| {
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
