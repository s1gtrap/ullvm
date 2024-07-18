#![allow(non_snake_case)]

use std::collections::HashSet;

use dioxus::prelude::*;
use tracing::Level;
use wasm_bindgen::prelude::*;

mod code;
mod editor;
mod example_picker;
mod ir;
mod iter_prev;
mod llvm;
mod lva;
mod tabs;

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
    let mut output_abstract = use_signal(|| "".to_owned());
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
    let mut output_iter: Signal<Vec<iter_prev::Iter<ir::Iter>>> = use_signal(|| vec![]);
    let map_lva = |(i, a): (
        usize,
        (
            String,
            Vec<(HashSet<ir::Name>, HashSet<ir::Name>, String)>,
            Vec<(HashSet<ir::Name>, HashSet<ir::Name>, String)>,
        ),
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

        tracing::info!("abstract: {:?}", m);
        *output_abstract.write() = format!("{:#?}", m);
        let window = web_sys::window().unwrap();
        let hpccWasm = js_sys::Reflect::get(&window, &JsValue::from_str("@hpcc-js/wasm")).unwrap();
        tracing::info!("{hpccWasm:?}");
        let graphviz = js_sys::Reflect::get(&hpccWasm, &JsValue::from_str("Graphviz")).unwrap();
        tracing::info!("{graphviz:?}");
        let load = js_sys::Reflect::get(&graphviz, &JsValue::from_str("load")).unwrap();
        let load: &js_sys::Function = load.dyn_ref().unwrap();
        let promise: js_sys::Promise = load.call0(&graphviz).unwrap().dyn_into().unwrap();
        tracing::info!("{promise:?}");
        let graphviz = wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
        let dot = js_sys::Reflect::get(&graphviz, &JsValue::from_str("dot")).unwrap();
        let dot: &js_sys::Function = dot.dyn_ref().unwrap();
        tracing::info!("{dot:?}");
        *output_cfg.write() = m
            .functions
            .iter()
            .map(|f| {
                let (_blocks, cfg) = ir::cfg(f);
                let cfg_dot = format!(
                    "{:?}",
                    petgraph::dot::Dot::with_config(&cfg, &[petgraph::dot::Config::EdgeNoLabel],),
                );
                let cfg: JsValue = dot
                    .call1(&graphviz, &JsValue::from_str(&cfg_dot))
                    .unwrap()
                    .dyn_into()
                    .unwrap();
                tracing::info!("{cfg:?}");
                let svg = cfg.dyn_ref::<js_sys::JsString>().unwrap().to_string();
                (f.name.clone(), cfg_dot, svg.into())
            })
            .collect();
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
        main { class: "w-full bg-slate-100",
            div { class: "flex",
                div { class: "w-1/2 lg:w-1/3",
                    div { class: "flex flex-col h-screen",
                        div { class: "flex-none",
                            example_picker::ExamplePicker { onpick: move |s| *content.write() = s }
                        }
                        div { class: "flex-1",
                            editor::Editor { content, onChange: move |s| *content.write() = s }
                        }
                        div { class: "flex-none",
                            button {
                                class: "h-12 w-full bg-slate-100",
                                onclick: onclickparse,
                                "Parse"
                            }
                        }
                    }
                }
                div { class: "w-1/2 lg:w-2/3",
                    tabs::Tabs {
                        tabs: vec![
                            (
                                "Abstract".to_string(),
                                rsx! {
                                    code::Code { code : output_abstract }
                                },
                            ),
                            (
                                "CFG".to_string(),
                                rsx! {
                                    tabs::Tabs { tabs : output_cfg.read().clone().into_iter().map(| s | { (s
                                    .0.clone(), rsx! { div { div { dangerous_inner_html : "{s.2}", }
                                    code::Code { code : "{s.1}" } } }) }).collect::< Vec < _ >> (), }
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
