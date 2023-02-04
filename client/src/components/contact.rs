use crate::CURRENT_CHANNEL;
use dioxus::prelude::*;
use fermi::use_atom_state;

#[inline_props]
pub fn Contact<'a>(cx: Scope<'a>, name: String, onselect: EventHandler<'a, String>) -> Element<'a> {
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
                src: "https://cdn.pixabay.com/photo/2018/09/12/12/14/man-3672010__340.jpg",
                alt: ""
            }
            div {
                class: "w-full pb-2",
                div {
                    class: "flex justify-between",
                    span {
                        class: "block ml-2 font-semibold",
                        "Jhon Done"
                    }
                    span {
                        class: "block ml-2 text-sm",
                        "25 min"
                    }
                }
                span {
                    class: "block ml-2 text-sm",
                    "bye"
                }
            }

        }
        }
    })
}
