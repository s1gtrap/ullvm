#![allow(non_snake_case)]

use dioxus::prelude::*;

#[component]
pub fn Tabs(tabs: Vec<(String, Element)>) -> Element {
    let mut active = use_signal(|| 0);
    rsx! {
        div { class: "flex flex-col h-full",
            nav { class: "flex-none",
                ul { class: "flex",
                    for (i , (t , _)) in tabs.iter().enumerate() {
                        li { class: "flex-1",
                            button {
                                class: "bg-zinc-100 dark:bg-zinc-800 h-12 w-full",
                                onclick: move |_| {
                                    *active.write() = i;
                                },
                                "{t}"
                            }
                        }
                    }
                }
            }
            div { class: "bg-white dark:bg-zinc-900 flex-1 overflow-scroll", {&tabs[*active.read()].1} }
        }
    }
}
