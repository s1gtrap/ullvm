#![allow(non_snake_case)]

use dioxus::prelude::*;

#[component]
pub fn Editor(content: String, onChange: EventHandler<String>) -> Element {
    rsx! {
        textarea {
            id: "container",
            class: "w-full h-full",
            oninput: move |e| {
                onChange.call(e.value().clone());
            },
            "{content}"
        }
    }
}
