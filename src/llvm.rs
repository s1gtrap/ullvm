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

pub fn parse(input: &str) -> crate::ir::Module {
    Module::ccall(
        JsValue::from_str("parse"),
        JsValue::NULL,
        js_sys::Array::of1(&JsValue::from_str("string")).into(),
        js_sys::Array::of1(&JsValue::from_str(input)).into(),
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

    serde_json::from_str(&String::from(out)).unwrap()
}
