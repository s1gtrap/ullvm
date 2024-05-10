#![allow(non_snake_case)]

use dioxus::prelude::*;
use tracing::Level;
use wasm_bindgen::prelude::*;

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
    let mut output = use_signal(|| "".to_owned());
    rsx! {
        main {
            div {
                select {
                    id: "example-picker",
                    onchange: move |e: Event<FormData>| {
                        // set input to item picked
                        let pick = e.data.value().parse::<usize>().unwrap() - 1;
                        *input.write() = EXAMPLES[pick].1.to_string();

                        // reset value after pick
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
                textarea {
                    onchange: move |e: Event<FormData>| {
                        *input.write() = e.value().to_owned();
                    },
                    "{input}"
                }
                button {
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
                        let out = js_sys::JSON::stringify_with_replacer_and_space(
                                &obj,
                                &JsValue::NULL,
                                &JsValue::from_f64(1.0),
                            )
                            .unwrap();
                        *output.write() = out.into();
                    },
                    "Parse"
                }
            }
            div { class: "output", "{output}" }
        }
    }
}
