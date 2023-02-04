use super::Contact;
use dioxus::prelude::*;

#[inline_props]
pub fn Sidebar<'a>(cx: Scope<'a>, onselect: EventHandler<'a, String>) -> Element<'a> {
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
                    Contact {
                        name: "test".to_string(),
                        onselect: |_| { }
                    }
                    Contact {
                        name: "john".to_string(),
                        onselect: |_| { }
                    }
                }
            }
        }
    })
}
