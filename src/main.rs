#![allow(non_snake_case)]

use std::collections::HashSet;

use dioxus::prelude::*;
use tracing::Level;
use wasm_bindgen::prelude::*;

mod code;
mod editor;
mod example_picker;
mod graphviz;
mod ir;
mod iter_prev;
mod llvm;
mod lva;
mod tabs;
mod util;

fn main() {
    console_error_panic_hook::set_once();

    dioxus_logger::init(Level::INFO).expect("logger failed to init");

    launch(App);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Graphviz)]
    async fn load(g: &JsValue) -> wasm_bindgen::JsValue;
}

#[component]
fn App() -> Element {
    let mut output_cfg = use_signal(|| vec![(String::new(), String::new(), String::new())]);
    let mut output_lva = use_signal(|| {
        vec![(
            String::new(),
            vec![(
                HashSet::<ir::Name>::new(),
                HashSet::<ir::Name>::new(),
                String::new(),
            )],
            vec![(
                HashSet::<ir::Name>::new(),
                HashSet::<ir::Name>::new(),
                String::new(),
            )],
        )]
    });
    let mut output_iter: Signal<Vec<iter_prev::Iter<ir::Iter>>> = use_signal(Vec::new);
    let map_lva = |(i, a): (
        usize,
        (String, Vec<ir::OwnedInstLive>, Vec<ir::OwnedInstLive>),
    )| {
        let mut lva_next = move || {
            if let Some(iter) = output_iter.write().get_mut(i) {
                if let Some(lives) = iter.next() {
                    tracing::info!("next: {:?}", lives);
                    let name = output_lva.read()[i].0.clone();
                    let old = output_lva.read()[i].2.clone();
                    output_lva.write()[i] = (name, old, lives);
                }
            }
        };
        let mut lva_finish = move || {
            if let Some(iter) = output_iter.write().get_mut(i) {
                if let Some(lives) = iter.last() {
                    tracing::info!("last: {:?}", lives);
                    let name = output_lva.read()[i].0.clone();
                    let old = output_lva.read()[i].2.clone();
                    output_lva.write()[i] = (name, old, lives);
                }
            }
        };
        let mut lva_prev = move || {
            if let Some(iter) = output_iter.write().get_mut(i) {
                if let Some(lives) = iter.prev() {
                    tracing::info!("prev: {:?}", lives);
                    let name = output_lva.read()[i].0.clone();
                    let old = output_lva.read()[i].2.clone();
                    output_lva.write()[i] = (name, old, lives);
                } else {
                    let name = output_lva.read()[i].0.clone();
                    let old = output_lva.read()[i].2.clone();
                    output_lva.write()[i] = (
                        name,
                        old.clone(),
                        old.into_iter()
                            .map(|(_, _, s)| (HashSet::new(), HashSet::new(), s))
                            .collect(),
                    );
                }
            }
        };
        let mut lva_reset = move || {
            if let Some(iter) = output_iter.write().get_mut(i) {
                let _ = iter.first();
                let name = output_lva.read()[i].0.clone();
                let old = output_lva.read()[i].2.clone();
                output_lva.write()[i] = (
                    name,
                    old.clone(),
                    old.into_iter()
                        .map(|(_, _, s)| (HashSet::new(), HashSet::new(), s))
                        .collect(),
                );
            }
        };
        (
            a.0.clone(),
            rsx! {
                div {
                    lva::Lva { old: a.1, new: a.2 }
                    div { class: "flex columns-4",
                        button {
                            class: "w-full h-12",
                            onclick: move |_| lva_reset(),
                            "<<"
                        }
                        button {
                            class: "w-full h-12",
                            onclick: move |_| lva_prev(),
                            "<"
                        }
                        button {
                            class: "w-full h-12",
                            onclick: move |_| lva_next(),
                            ">"
                        }
                        button {
                            class: "w-full h-12",
                            onclick: move |_| lva_finish(),
                            ">>"
                        }
                    }
                }
            },
        )
    };

    let mut content = use_signal(|| include_str!("../examples/ll/for1.ll").to_string());

    let onclickparse = move |_| async move {
        let input = content.read().clone();
        let m: ir::Module = llvm::parse(&input);

        *output_cfg.write() = futures::future::join_all(m.functions.iter().map(|f| async {
            let (_blocks, cfg) = ir::cfg(f);
            let dot = petgraph::dot::Dot::with_config(
                &cfg,
                &[
                    petgraph::dot::Config::EdgeNoLabel,
                    petgraph::dot::Config::_GraphAttr("bgcolor", "transparent"),
                ],
            );
            let svg = graphviz::svg(&dot).await;
            (f.name.clone(), format!("{dot:?}"), svg)
        }))
        .await;

        *output_lva.write() = m
            .functions
            .iter()
            .map(|f| {
                let insns = ir::lva(f);
                (
                    f.name.to_string(),
                    insns
                        .iter()
                        .map(|(_in, _out, insn)| (HashSet::new(), HashSet::new(), insn.to_string()))
                        .collect(),
                    insns
                        .iter()
                        .map(|(_in, _out, insn)| (HashSet::new(), HashSet::new(), insn.to_string()))
                        .collect(),
                )
            })
            .collect();

        *output_iter.write() = m
            .functions
            .iter()
            .map(|f| {
                let f: ir::Function = f.clone();
                let iter = ir::Iter::new(&f);
                iter_prev::Iter::new(iter)
            })
            .collect();
    };

    rsx! {
        main { class: "bg-zinc-100 dark:bg-zinc-900 dark:text-zinc-300 w-full",
            div { class: "flex flex-col md:flex-row h-screen",
                div { class: "h-1/2 w-full md:h-full md:w-1/2 lg:w-1/3",
                    div { class: "flex flex-col h-full",
                        div { class: "flex-none",
                            example_picker::ExamplePicker { onpick: move |s| *content.write() = s }
                        }
                        div { class: "flex-1",
                            editor::Editor { content, onChange: move |s| *content.write() = s }
                        }
                        div { class: "flex-none",
                            button {
                                class: "bg-zinc-100 dark:bg-zinc-800 h-12 w-full",
                                onclick: onclickparse,
                                "Parse"
                            }
                        }
                    }
                }
                div { class: "bg-green-500 h-1/2 md:h-full md:w-1/2 lg:w-2/3",
                    tabs::Tabs {
                        tabs: vec![
                            (
                                "CFG".to_string(),
                                rsx! {
                                    tabs::Tabs { tabs : output_cfg.read().clone().into_iter().map(| s | { (s
                                    .0.clone(), rsx! { div { div { class : "dark:invert",
                                    dangerous_inner_html : "{s.2}", } code::Code { code : "{s.1}" } } }) })
                                    .collect::< Vec < _ >> (), }
                                },
                            ),
                            (
                                "LVA".to_string(),
                                rsx! {
                                    tabs::Tabs { tabs : output_lva.read().clone().into_iter().enumerate()
                                    .map(map_lva).collect::< Vec < _ >> (), }
                                },
                            ),
                        ]
                    }
                }
            }
        }
    }
}
