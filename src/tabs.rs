#![allow(non_snake_case)]

use dioxus::prelude::*;

#[component]
pub fn Tabs(tabs: Vec<(&'static str, Element)>) -> Element {
    let mut active = use_signal(|| 0);
    rsx! {
        div { class: "flex flex-col h-screen",
            nav { class: "flex-none",
                ul { class: "flex",
                    for (i , (t , _)) in tabs.iter().enumerate() {
                        li { class: "flex-1",
                            button {
                                class: "h-12 bg-slate-100 w-full",
                                onclick: move |_| {
                                    *active.write() = i;
                                },
                                "{t}"
                            }
                        }
                    }
                }
            }
            div { class: "flex-1 overflow-scroll", {&tabs[*active.read()].1} }
        }
    }
}
