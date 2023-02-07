use crate::CURRENT_CHANNEL;
use dioxus::prelude::*;
use fermi::use_atom_state;

#[allow(non_snake_case)]
#[inline_props]
pub fn Contact<'a>(
    cx: Scope<'a>,
    name: String,
    cover: String,
    last_message: String,
    dt: String,
    onselect: EventHandler<'a, String>,
) -> Element<'a> {
    let current_channel = use_atom_state(cx, CURRENT_CHANNEL);
    let class_name: &str = if current_channel.is_some()
        && current_channel.current().as_ref().clone().unwrap() == cx.props.name
    {
        "flex items-center bg-blue-500 px-3 py-2 text-sm transition duration-150 ease-in-out border-b border-gray-300 cursor-pointer hover:bg-gray-100 focus:outline-none text-white"
    } else {
        "flex items-center px-3 py-2 text-sm transition duration-150 ease-in-out border-b border-gray-300 cursor-pointer hover:bg-gray-100 focus:outline-none text-gray-600"
    };
    cx.render(rsx! {
        div {
        a {
            class: class_name,
            prevent_default: "onclick",
            onclick: move |_| {
                current_channel.modify(|_| Some(cx.props.name.clone()));
            },
            img {
                class: "object-cover w-10 h-10 rounded-full",
                src: "{cover}",
                alt: ""
            }
            div {
                class: "w-full pb-2",
                div {
                    class: "flex justify-between",
                    span {
                        class: "block ml-2 font-semibold",
                        "{name}"
                    }
                    span {
                        class: "block ml-2 text-sm",
                        "{dt}"
                    }
                }
                span {
                    class: "block ml-2 text-sm",
                    "{last_message}"
                }
            }

        }
        }
    })
}
