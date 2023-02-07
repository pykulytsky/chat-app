use crate::CHANNELS;

use super::Contact;
use crate::components::channel_form::ChannelForm;
use dioxus::prelude::*;
use fermi::use_atom_state;

#[allow(non_snake_case)]
#[inline_props]
pub fn Sidebar<'a>(
    cx: Scope<'a>,
    onselect: EventHandler<'a, String>,
    onsubmit: EventHandler<'a, (String, String)>,
) -> Element<'a> {
    let channels = use_atom_state(cx, CHANNELS);

    let channel_form = use_state(cx, || false);

    let channels_list = channels.iter().map(|(_, ch)| {
        let len = ch.messages.len();
        let (mut last_message, dt) = if len > 0 {
            (ch.messages[len - 1].body.to_string(), ch.messages[len-1].created.format("%H:%M").to_string())
        } else {
            ("".to_string(), "".to_string())
        };
        if last_message.len() > 25 {
            last_message = last_message[0..25].to_string() + "...";
        }
        rsx! {
            Contact {
                name: ch.name.clone(),
                cover: ch.cover.clone().unwrap_or("https://cdn-icons-png.flaticon.com/512/134/134932.png".to_string()),
                last_message: last_message,
                dt: dt,
                onselect: |_| { }
            }
        }
    });
    let form = if **channel_form {
        cx.render(rsx! {
            ChannelForm {
                onsubmit: move |(name, cover)| {
                    channel_form.set(false);
                    onsubmit.call((name, cover));
                },
                oncancel: move |_| {
                    channel_form.set(false);
                },
            }
        })
    } else {
        None
    };
    cx.render(rsx! {
        aside {
            class: "fixed top-0 left-0 z-40 w-64 h-screen transition-transform -translate-x-full sm:translate-x-0",
            div {
                class: "h-full py-4 overflow-y-auto bg-gray-50 dark:bg-gray-800",
                div {
                    div {
                        class: "flex justify-between my-2 mb-2 ml-2 ",
                        h2 {
                            class: "text-lg text-gray-600",
                            "Chats"
                        },
                        button {
                            "type": "button",
                            class: "text-white bg-blue-600 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-2.5 text-center inline-flex items-center mr-2 dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800",
                            span {
                                class: "text-lg",
                                onclick: move |_| channel_form.set(true),
                                "+"
                            }
                        }
                    }
                    channels_list
                    form
                }
            }
        }
    })
}
