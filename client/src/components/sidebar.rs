use crate::CHANNELS;

use super::Contact;
use dioxus::prelude::*;
use fermi::use_atom_state;
use protocol::Channel;

#[allow(non_snake_case)]
#[inline_props]
pub fn Sidebar<'a>(cx: Scope<'a>, onselect: EventHandler<'a, String>) -> Element<'a> {
    let channels = use_atom_state(cx, CHANNELS);

    let channels_list = channels.iter().map(|(_, ch)| {
        let len = ch.messages.len();
        let mut last_message = if len > 5 {
            ch.messages[len - 1].body.to_string()
        } else {
            "".to_owned()
        };
        if last_message.len() > 25 {
            last_message = last_message[0..25].to_string() + "...";
        }
        rsx! {
            Contact {
                name: ch.name.clone(),
                last_message: last_message,
                onselect: |_| { }
            }
        }
    });
    cx.render(rsx! {
        aside {
            class: "fixed top-0 left-0 z-40 w-64 h-screen transition-transform -translate-x-full sm:translate-x-0",
            div {
                class: "h-full py-4 overflow-y-auto bg-gray-50 dark:bg-gray-800",
                div {
                    h2 {
                        class: "my-2 mb-2 ml-2 text-lg text-gray-600",
                        "Chats"
                    },
                    channels_list
                }
            }
        }
    })
}
