#![allow(non_snake_case)]

use dioxus::prelude::*;

#[component]
pub fn Code(code: String) -> Element {
    rsx! {
        div { class: "bg-white box-border dark:bg-zinc-900 font-mono ml-1 whitespace-pre", "{code}" }
    }
}
