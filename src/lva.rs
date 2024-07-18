#![allow(non_snake_case)]

use std::collections::HashSet;

use dioxus::prelude::*;

#[component]
pub fn Lva(new: Vec<(HashSet<crate::ir::Name>, HashSet<crate::ir::Name>, String)>) -> Element {
    rsx! {
        div { class: "ml-1",
            div { class: "font-mono whitespace-pre bg-white box-border",
                for (r#in , out , s) in &new {
                    div { class: "flex",

                        span { class: "flex-none text-right w-1/4 text-green-900", "{r#in:?}" }
                        span { class: "flex-none text-right w-1/4 text-red-900", "{out:?}" }
                        span { class: "flex-none w-1/2", "{s}" }
                    }
                }
            }
        }
    }
}
