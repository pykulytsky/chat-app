use dioxus::prelude::*;

#[allow(non_snake_case)]
#[inline_props]
pub fn ChannelForm<'a>(
    cx: Scope<'a>,
    onsubmit: EventHandler<'a, (String, String)>,
    oncancel: EventHandler<'a>,
) -> Element<'a> {
    let name = use_state(cx, String::new);
    let cover = use_state(cx, String::new);
    cx.render(rsx! {
        div {
            class: "inset-0 w-1/2 mx-auto fixed pin flex items-center",
            div {
                class: "fixed pin bg-black opacity-75 z-10"
            }
            div {
                class: "relative mx-6 md:mx-auto w-full md:w-1/2 lg:w-1/3 z-20 m-8",
                div {
                    class: "shadow-lg bg-white rounded-lg p-8",
                    div {
                        class: "flex justify-end mb-6",
                        button {
                            onclick: move |_| oncancel.call(()),
                            span {
                                class: "mr-2",
                                "Exit"
                            }
                        }
                    }
                    h1 {
                        class: "text-center text-2xl text-green-dark",
                        "Create new channel"
                        div {
                            class: "pt-6 pb-2 my-2",
                            div {
                                class: "mb-4",
                                label {
                                    class: "block text-sm font-bold mb-2",
                                    "for": "name",
                                    "Name"
                                }
                                input {
                                    class: "shadow appearance-none border rounded w-full py-2 px-3 text-grey-darker",
                                    id: "name",
                                    "type": "text",
                                    placeholder: "Enter name",
                                    value: "{name}",
                                    oninput: move |evt| name.set(evt.value.clone()),
                                }
                            }
                            div {
                                class: "mb-6",
                                label {
                                    class: "block text-sm font-bold mb-2",
                                    "for": "cover",
                                    "Cover"
                                }
                                input {
                                    class: "shadow appearance-none border rounded w-full py-2 px-3 text-grey-darker mb-3",
                                    id: "cover",
                                    "type": "text",
                                    placeholder: "Enter link to cover",
                                    value: "{cover}",
                                    oninput: move |evt| cover.set(evt.value.clone()),
                                }
                            }
                            div {
                                class: "mb-6 flex justify-center",
                                button {
                                    class: "bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded",
                                    onclick: move |_| onsubmit.call((name.clone().to_string(), cover.clone().to_string())),
                                    "Create"
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}
