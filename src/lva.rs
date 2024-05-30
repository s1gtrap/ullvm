#![allow(non_snake_case)]

use std::collections::HashSet;

use dioxus::prelude::*;

#[component]
pub fn Lva(lva: Vec<(HashSet<crate::ir::Name>, HashSet<crate::ir::Name>, String)>) -> Element {
    let mut names: Vec<_> = lva
        .iter()
        .cloned()
        .map(|(r#in, out, _)| r#in.iter().chain(out.iter()).cloned().collect::<Vec<_>>())
        .flatten()
        .collect();

    names.sort();
    names.dedup();

    let mut highlight = use_signal(|| None::<crate::ir::Name>);

    rsx! {
        div { class: "ml-1",
            {"Highlight: "},
            select {
                onchange: move |e: Event<FormData>| {
                    tracing::info!("{e:?}");
                    let h: Option<usize> = e.data.value().parse().ok();
                    *highlight.write() = h.map(|i| names[i].clone()).clone();
                },
                option { "None" }
                for (i , name) in names.iter().enumerate() {
                    option { value: "{i}", "{name:?}" }
                }
            }
            div { class: "font-mono whitespace-pre bg-white box-border",
                for (r#in , out , s) in &lva {
                    if highlight.read().is_some()
                        && (r#in.contains(&highlight.read().clone().unwrap())
                            || out.contains(&highlight.read().clone().unwrap()))
                    {

                        if !r#in.contains(&highlight.read().clone().unwrap()) {
                            div { class: "bg-clip-text text-transparent bg-gradient-to-b from-black to-red-500 via-red-500",
                                "{s}"
                            }
                        } else if !out.contains(&highlight.read().clone().unwrap()) {

                            div { class: "bg-clip-text text-transparent bg-gradient-to-b from-red-500 to-black via-red-500",
                                "{s}"
                            }
                        } else {

                            div { class: "text-red-500", "{s}" }
                        }
                    } else {
                        div { "{s}" }
                    }
                }
            }
        }
    }
}
