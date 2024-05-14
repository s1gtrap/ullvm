#![allow(non_snake_case)]

use dioxus::prelude::*;

#[component]
pub fn Code(code: String) -> Element {
    rsx! {
        div { class: "font-mono whitespace-pre ml-1 bg-white box-border", "{code}" }
    }
}
