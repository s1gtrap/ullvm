#![allow(non_snake_case)]

use std::collections::{HashMap, HashSet};

use dioxus::prelude::*;
use tracing::Level;
use wasm_bindgen::prelude::*;

mod code;
mod interf;
mod ir;
mod lva;
mod tabs;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "Module")]
    type Module;

    #[wasm_bindgen(static_method_of = Module)]
    fn ccall(id: JsValue, rty: JsValue, targs: JsValue, args: JsValue) -> JsValue;

    #[wasm_bindgen(static_method_of = Module)]
    fn UTF8ToString(data: JsValue) -> JsValue;
}

fn main() {
    console_error_panic_hook::set_once();

    dioxus_logger::init(Level::INFO).expect("logger failed to init");

    launch(App);
}

macro_rules! test {
    ($($data:expr),* $(,)?)  => {
        [ $( ($data, include_str!(concat!("../examples/ll/", $data))) ),* ]
    };
}

const EXAMPLES: &[(&str, &str)] = &test![
    "min.ll",
    "ret.ll",
    "for0.ll",
    "for1.ll",
    "fib.ll",
    "brainfuck.ll",
];

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Graphviz)]
    async fn load(g: &JsValue) -> wasm_bindgen::JsValue;
}

#[component]
fn App() -> Element {
    let mut input = use_signal(|| "".to_owned());
    let mut output_json = use_signal(|| "".to_owned());
    let mut output_debug = use_signal(|| "".to_owned());
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
    let mut output_iter: Signal<Vec<ir::Iter>> = use_signal(|| vec![]);
    let map_lva = |(i, a): (usize, (String, ir::Lva, ir::Lva))| {
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
        (
            a.0.clone(),
            rsx! {
                div {
                    lva::Lva { old: a.1, new: a.2 }
                    div { class: "flex columns-4",
                        button { class: "w-full h-12", disabled: "true", "<<" }
                        button { class: "w-full h-12", disabled: "true", "<" }
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
    let mut output_intf =
        use_signal(|| HashMap::<String, String>::from([("".to_string(), "".to_string())]));
    rsx! {
        main { class: "w-full bg-slate-100",
            div { class: "flex",
                div { class: "w-1/2 lg:w-1/3",
                    div { class: "flex flex-col h-screen",
                        div { class: "flex-none",
                            select {
                                id: "example-picker",
                                class: "h-12 w-full bg-slate-100",
                                onchange: move |e: Event<FormData>| {
                                    let pick = e.data.value().parse::<usize>().unwrap() - 1;
                                    *input.write() = EXAMPLES[pick].1.to_string();
                                    let window = web_sys::window().unwrap();
                                    let document = window.document().unwrap();
                                    let element = document.get_element_by_id("example-picker").unwrap();
                                    let select: &web_sys::HtmlSelectElement = element.dyn_ref().unwrap();
                                    select.set_value("0");
                                },
                                option { value: "0", disabled: true, "-- example --" }
                                for (i , (n , _)) in EXAMPLES.iter().enumerate() {
                                    option { key: "{i}", value: "{i + 1}", "{n}" }
                                }
                            }
                        }
                        div { class: "flex-1",
                            textarea {
                                class: "w-full h-full font-mono whitespace-pre mr-1 box-border",
                                onchange: move |e: Event<FormData>| {
                                    *input.write() = e.value().to_string();
                                },
                                "{input}"
                            }
                        }
                        div { class: "flex-none",
                            button {
                                class: "h-12 w-full bg-slate-100",
                                onclick: move |_| async move {
                                    Module::ccall(
                                        JsValue::from_str("parse"),
                                        JsValue::NULL,
                                        js_sys::Array::of1(&JsValue::from_str("string")).into(),
                                        js_sys::Array::of1(&JsValue::from_str(&input())).into(),
                                    );
                                    let ptr = Module::ccall(
                                        JsValue::from_str("json"),
                                        JsValue::from_str("number"),
                                        js_sys::Array::new().into(),
                                        js_sys::Array::new().into(),
                                    );
                                    let str = Module::UTF8ToString(ptr).as_string().unwrap();
                                    let obj = js_sys::JSON::parse(&str).unwrap();
                                    let out: js_sys::JsString = js_sys::JSON::stringify_with_replacer_and_space(
                                            &obj,
                                            &JsValue::NULL,
                                            &JsValue::from_f64(1.0),
                                        )
                                        .unwrap();
                                    tracing::info!("{}", out);
                                    let s: String = out.into();
                                    *output_json.write() = s.clone();
                                    let m: llvm_ir::Module = serde_json::from_str(&s).unwrap();
                                    tracing::info!("llvm-ir: {:?}", m);
                                    *output_debug.write() = format!("{:#?}", m);
                                    let m: ir::Module = serde_json::from_str(&s).unwrap();
                                    tracing::info!("abstract: {:?}", m);
                                    *output_abstract.write() = format!("{:#?}", m);
                                    let window = web_sys::window().unwrap();
                                    let hpccWasm = js_sys::Reflect::get(&window, &JsValue::from_str("@hpcc-js/wasm"))
                                        .unwrap();
                                    tracing::info!("{hpccWasm:?}");
                                    let graphviz = js_sys::Reflect::get(&hpccWasm, &JsValue::from_str("Graphviz"))
                                        .unwrap();
                                    tracing::info!("{graphviz:?}");
                                    let load = js_sys::Reflect::get(&graphviz, &JsValue::from_str("load")).unwrap();
                                    let load: &js_sys::Function = load.dyn_ref().unwrap();
                                    let promise: js_sys::Promise = load
                                        .call0(&graphviz)
                                        .unwrap()
                                        .dyn_into()
                                        .unwrap();
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
                                                petgraph::dot::Dot::with_config(
                                                    &cfg,
                                                    &[petgraph::dot::Config::EdgeNoLabel],
                                                ),
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
                                                    .map(|(r#in, out, insn)| {
                                                        (HashSet::new(), HashSet::new(), format!("{insn}"))
                                                    })
                                                    .collect(),
                                                insns
                                                    .iter()
                                                    .map(|(r#in, out, insn)| {
                                                        (HashSet::new(), HashSet::new(), format!("{insn}"))
                                                    })
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
                                            iter
                                        })
                                        .collect();
                                    *output_intf.write() = m
                                        .functions
                                        .iter()
                                        .map(|f| {
                                            let iter = ir::Iter::new(&f);
                                            let opt: Option<ir::Lva> = iter.last();
                                            (
                                                f.name.clone(),
                                                opt
                                                    .map(|lva| {
                                                        let g = interf::interf(lva);
                                                        let dot = petgraph::dot::Dot::with_attr_getters(
                                                            &g,
                                                            &[petgraph::dot::Config::EdgeNoLabel],
                                                            &|_, _| "color=red".to_string(),
                                                            &|_, _| "".to_string(),
                                                        );
                                                        format!("{dot:?}")
                                                    })
                                                    .unwrap_or("".to_string()),
                                            )
                                        })
                                        .collect();
                                },
                                "Parse"
                            }
                        }
                    }
                }
                div { class: "w-1/2 lg:w-2/3",
                    tabs::Tabs {
                        tabs: vec![
                            (
                                "JSON".to_string(),
                                rsx! {
                                    code::Code { code : output_json }
                                },
                            ),
                            (
                                "Debug".to_string(),
                                rsx! {
                                    code::Code { code : output_debug }
                                },
                            ),
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
                            (
                                "Interference".to_string(),
                                rsx! {
                                    tabs::Tabs { tabs : output_intf.read().clone().into_iter().map(| (n, g) |
                                    { (n.clone(), rsx! { div { div { dangerous_inner_html : "{g}", }
                                    code::Code { code : "{g}" } } }) }).collect::< Vec < _ >> (), }
                                },
                            ),
                        ]
                    }
                }
            }
        }
    }
}
