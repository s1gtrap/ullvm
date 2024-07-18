use wasm_bindgen::prelude::*;

pub fn dark_mode() -> Result<bool, JsValue> {
    let window = web_sys::window().unwrap();
    Ok(window
        .match_media("(prefers-color-scheme: dark)")?
        .map(|q| q.matches())
        .unwrap_or(false))
}
