#![allow(non_snake_case)]

use std::collections::HashSet;

use dioxus::prelude::*;

#[component]
pub fn Lva(
    old: Vec<(HashSet<crate::ir::Name>, HashSet<crate::ir::Name>, String)>,
    new: Vec<(HashSet<crate::ir::Name>, HashSet<crate::ir::Name>, String)>,
) -> Element {
    let mut names: Vec<_> = new
        .iter()
        .cloned()
        .flat_map(|(r#in, out, _)| r#in.iter().chain(out.iter()).cloned().collect::<Vec<_>>())
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
                    highlight.write().clone_from(&h.map(|i| names[i].clone()));
                },
                option { "None" }
                for (i , name) in names.iter().enumerate() {
                    option { value: "{i}", "{name:?}" }
                }
            }
            div { class: "font-mono whitespace-pre bg-white box-border",
                for ((oin , oout , _) , (r#in , out , s)) in old.iter().zip(new) {
                    if highlight.read().is_some()
                        && (r#in.contains(&highlight.read().clone().unwrap())
                            || out.contains(&highlight.read().clone().unwrap()))
                    {

                        div { class: "flex text-red-500",
                            span { class: "flex-none text-right w-1/4 text-green-500",
                                "{r#in:?}"
                            }
                            span { class: "flex-none text-right w-1/4 text-red-500",
                                "{out:?}"
                            }
                            span { class: "flex-none w-1/2", "{s}" }
                        }
                    } else {
                        div { class: "flex",
                            if *oin == r#in {

                                span { class: "flex-none text-right w-1/4 text-green-900",
                                    "{r#in:?}"
                                }
                            } else {
                                span { class: "flex-none text-right w-1/4 text-green-500",
                                    "{r#in:?}"
                                }
                            }
                            if *oout == out {
                                span { class: "flex-none text-right w-1/4 text-red-900",
                                    "{out:?}"
                                }
                            } else {
                                span { class: "flex-none text-right w-1/4 text-red-500",
                                    "{out:?}"
                                }
                            }
                            span { class: "flex-none w-1/2", "{s}" }
                        }
                    }
                }
            }
        }
    }
}
