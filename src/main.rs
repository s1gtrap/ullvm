#![allow(non_snake_case)]

use dioxus::prelude::*;
use tracing::Level;
use wasm_bindgen::prelude::*;

mod code;
mod ir;
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

const EXAMPLES: &[(&str, &str)] = &test!["min.ll", "ret.ll", "fib.ll", "brainfuck.ll"];

#[component]
fn App() -> Element {
    let mut input = use_signal(|| "".to_owned());
    let mut output_json = use_signal(|| "".to_owned());
    let mut output_debug = use_signal(|| "".to_owned());
    let mut output_abstract = use_signal(|| "".to_owned());
    let mut output_cfg = use_signal(|| "".to_owned());
    rsx! {
        main { class: "w-full bg-indigo",
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
                                class: "w-full h-full font-mono whitespace-pre",
                                onchange: move |e: Event<FormData>| {
                                    *input.write() = e.value().to_string();
                                },
                                "{input}"
                            }
                        }
                        div { class: "flex-none",
                            button {
                                class: "h-12 w-full bg-slate-100",
                                onclick: move |_| {
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
                                    tracing::info!("pre-cfg\n");
                                    let cfg = ir::cfg(&m.functions[0]);
                                    tracing::info!("post-cfg");
                                    tracing::info!(
                                        "{:?}", petgraph::dot::Dot::with_config(& cfg, &
                                        [petgraph::dot::Config::EdgeNoLabel])
                                    );
                                    *output_cfg.write() = format!(
                                        "{:?}",
                                        petgraph::dot::Dot::with_config(&cfg, &[petgraph::dot::Config::EdgeNoLabel]),
                                    );
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
                                "JSON",
                                rsx! {
                                    code::Code { code : output_json }
                                },
                            ),
                            (
                                "Debug",
                                rsx! {
                                    code::Code { code : output_debug }
                                },
                            ),
                            (
                                "Abstract",
                                rsx! {
                                    code::Code { code : output_abstract }
                                },
                            ),
                            (
                                "CFG",
                                rsx! {
                                    code::Code { code : output_cfg }
                                },
                            ),
                        ]
                    }
                }
            }
        }
    }
}
