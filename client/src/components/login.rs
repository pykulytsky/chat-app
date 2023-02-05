use dioxus::prelude::*;

#[allow(non_snake_case)]
#[inline_props]
pub fn Login<'a>(cx: Scope<'a>, onsubmit: EventHandler<'a, (String, String)>) -> Element<'a> {
    let username = use_state(cx, String::new);
    let avatar = use_state(cx, String::new);
    cx.render(rsx! {
        div {
            class: "inset-0 fixed pin flex items-center",
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
                            span {
                                class: "mr-2",
                                "Exit"
                            }
                        }
                    }
                    h1 {
                        class: "text-center text-2xl text-green-dark",
                        "Login"
                        div {
                            class: "pt-6 pb-2 my-2",
                            div {
                                class: "mb-4",
                                label {
                                    class: "block text-sm font-bold mb-2",
                                    "for": "username",
                                    "Username"
                                }
                                input {
                                    class: "shadow appearance-none border rounded w-full py-2 px-3 text-grey-darker",
                                    id: "username",
                                    "type": "text",
                                    placeholder: "Enter your username",
                                    value: "{username}",
                                    oninput: move |evt| username.set(evt.value.clone()),
                                }
                            }
                            div {
                                class: "mb-6",
                                label {
                                    class: "block text-sm font-bold mb-2",
                                    "for": "avatar",
                                    "Avatar"
                                }
                                input {
                                    class: "shadow appearance-none border rounded w-full py-2 px-3 text-grey-darker mb-3",
                                    id: "avatar",
                                    "type": "text",
                                    placeholder: "Enter link to your avatar",
                                    value: "{avatar}",
                                    oninput: move |evt| avatar.set(evt.value.clone()),
                                }
                            }
                            div {
                                class: "mb-6 flex justify-center",
                                button {
                                    class: "bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded",
                                    onclick: move |_| onsubmit.call((username.clone().to_string(), avatar.clone().to_string())),
                                    "Login"
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}
