use dioxus::prelude::*;
use fermi::use_atom_state;

use crate::{CHANNELS, CURRENT_CHANNEL};

#[allow(non_snake_case)]
#[inline_props]
pub fn Header(cx: Scope, name: String, cover: String) -> Element {
    println!("{name} {cover}");
    let current_channel = use_atom_state(cx, CURRENT_CHANNEL);
    let channels = use_atom_state(cx, CHANNELS);
    let channel = current_channel.clone();
    let current_channel = current_channel.as_ref().unwrap();
    // let cover = channels
    //     .get()
    //     .get(channel.as_ref().unwrap())
    //     .unwrap()
    //     .cover
    //     .clone()
    //     .unwrap_or("https://cdn-icons-png.flaticon.com/512/134/134932.png".to_string());
    cx.render(rsx! {
            div {
                class: "flex sm:items-center justify-between py-3 px-6 border-b-2 border-gray-200 sticky",
                div {
                    class: "relative flex items-center space-x-4",
                    div {
                        class: "relative",
                        img {
                            src: "{cover}",
                            alt: "",
                            class: "w-10 sm:w-16 h-10 sm:h-16 rounded-full"
                        }
                    }
                    div {
                        class: "flex flex-col leading-tight",
                        div {
                            class: "text-2xl mt-1 flex items-center ml-6",
                            span {
                                class: "text-gray-700 mr-3",
                                "{name}"
                            }
                        }
                    }
                }
                div {
                    class: "flex items-center space-x-2",
                    button {
                        class: "inline-flex items-center justify-center rounded-lg border h-10 w-10 transition duration-500 ease-in-out text-gray-500 hover:bg-gray-300 focus:outline-none",
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            fill:"none",
                            view_box:"0 0 24 24",
                            stroke:"currentColor",
                            class:"h-6 w-6",
                            path {
                                stroke_linecap:"round", stroke_linejoin:"round", stroke_width:"2", d:"M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                            }
                        }
                    }
                    button {
                        class: "inline-flex items-center justify-center rounded-lg border h-10 w-10 transition duration-500 ease-in-out text-gray-500 hover:bg-gray-300 focus:outline-none",
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            fill:"none",
                            view_box:"0 0 24 24",
                            stroke:"currentColor",
                            class:"h-6 w-6",
                            path {
                                stroke_linecap:"round", stroke_linejoin:"round", stroke_width:"2", d:"M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z"
                            }
                        }
                    }
                }
            }
    })
}
