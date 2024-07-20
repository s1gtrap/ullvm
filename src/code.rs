#![allow(non_snake_case)]

use dioxus::prelude::*;

#[component]
pub fn Code(code: String) -> Element {
    rsx! {
        div { class: "box-border font-mono ml-1 whitespace-pre", "{code}" }
    }
}
