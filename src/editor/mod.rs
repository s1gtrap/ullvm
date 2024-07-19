#![allow(non_snake_case)]

use dioxus::prelude::*;
use wasm_bindgen::prelude::*;

use crate::util;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "Module")]
    type Module;

    #[wasm_bindgen(static_method_of = Module)]
    fn ccall(id: JsValue, rty: JsValue, targs: JsValue, args: JsValue) -> JsValue;

    #[wasm_bindgen(static_method_of = Module)]
    fn UTF8ToString(data: JsValue) -> JsValue;
}

#[component]
pub fn Editor(content: String, onChange: EventHandler<String>) -> Element {
    let mut editor = use_signal(|| None::<JsValue>);

    if let Some(ref editor) = *editor.read() {
        let get_model: js_sys::Function =
            js_sys::Reflect::get(editor, &JsValue::from_str("getModel"))
                .unwrap()
                .dyn_into()
                .unwrap();
        let model = get_model.call0(editor).unwrap();

        let get_value: js_sys::Function =
            js_sys::Reflect::get(&model, &JsValue::from_str("getValue"))
                .unwrap()
                .dyn_into()
                .unwrap();
        let con: JsValue = get_value.call0(&model).unwrap();
        let con = con.as_string().unwrap();

        if con != content {
            let set_value: js_sys::Function =
                js_sys::Reflect::get(&model, &JsValue::from_str("setValue"))
                    .unwrap()
                    .dyn_into()
                    .unwrap();
            set_value
                .call1(&model, &JsValue::from_str(&content))
                .unwrap();
        }
    }

    use_effect(move || {
        tracing::info!("effect");
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let require = js_sys::Reflect::get(&window, &JsValue::from_str("require")).unwrap();
        let config: js_sys::Function = js_sys::Reflect::get(&require, &JsValue::from_str("config"))
            .unwrap()
            .dyn_into()
            .unwrap();
        let arg1 = js_sys::JSON::parse(r#"{ "paths": { "vs": "https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.49.0/min/vs" } }"#).unwrap();
        config.call1(&require, &arg1).unwrap();
        tracing::info!("Configured monaco loader");
        let require: js_sys::Function = require.dyn_into().unwrap();
        let arg1 = js_sys::JSON::parse(r#"["vs/editor/editor.main"]"#).unwrap();
        let content = content.clone();
        let callback = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
            //tracing::info!("monaco is ready!");
            let monaco = js_sys::Reflect::get(&window, &JsValue::from_str("monaco")).unwrap();
            let monaco_editor =
                js_sys::Reflect::get(&monaco, &JsValue::from_str("editor")).unwrap();
            let create: js_sys::Function =
                js_sys::Reflect::get(&monaco_editor, &JsValue::from_str("create"))
                    .unwrap()
                    .dyn_into()
                    .unwrap();
            let container = document.get_element_by_id("container").unwrap();

            // register llvm language
            let languages = js_sys::Reflect::get(&monaco, &JsValue::from_str("languages")).unwrap();
            let register: js_sys::Function =
                js_sys::Reflect::get(&languages, &JsValue::from_str("register"))
                    .unwrap()
                    .dyn_into()
                    .unwrap();
            let arg1 = js_sys::JSON::parse(r#"{ "id": "llvm" }"#).unwrap();
            register.call1(&languages, &arg1).unwrap();
            let set_monarch_tokens_provider: js_sys::Function =
                js_sys::Reflect::get(&languages, &JsValue::from_str("setMonarchTokensProvider"))
                    .unwrap()
                    .dyn_into()
                    .unwrap();
            let root = js_sys::Array::of4(
                &js_sys::Array::of2(
                    &js_sys::RegExp::new("(%|@)[a-zA-Z0-9\\.]+", ""),
                    &JsValue::from_str("variable"),
                ),
                &js_sys::Array::of2(
                    &js_sys::RegExp::new("@?[a-zA-Z][\\w$]*", ""),
                    &js_sys::JSON::parse(
                        r#"{
				"cases": {
					"@keywords": "keyword"
				}
			}"#,
                    )
                    .unwrap(),
                ),
                &js_sys::Array::of2(
                    &js_sys::RegExp::new(r#"".*?""#, ""),
                    &JsValue::from_str("string"),
                ),
                &js_sys::Array::of2(
                    &js_sys::RegExp::new(r#";.*$"#, ""),
                    &JsValue::from_str("comment"),
                ),
            );
            let arg2 = js_sys::JSON::parse(
                r#"{
	"keywords": ["define", "declare", "attributes"],
	"tokenizer": {}
}"#,
            )
            .unwrap();
            let tokenizer = js_sys::Reflect::get(&arg2, &JsValue::from_str("tokenizer")).unwrap();
            js_sys::Reflect::set(&tokenizer, &JsValue::from_str("root"), &root).unwrap();
            set_monarch_tokens_provider
                .call2(&languages, &JsValue::from_str("llvm"), &arg2)
                .unwrap();

            let theme = if util::dark_mode().unwrap() {
                "vs-dark"
            } else {
                "vs"
            };
            let arg2 = js_sys::JSON::parse(&format!(
                r#"{{
    "value": {:?},
    "language": "llvm",
    "minimap": {{ "enabled": false }},
    "automaticLayout": true,
    "theme": {:?}
}}"#,
                content, theme,
            ))
            .unwrap();

            tracing::info!("set editor");
            let e = create.call2(&monaco_editor, &container, &arg2).unwrap();

            let get_model: js_sys::Function =
                js_sys::Reflect::get(&e, &JsValue::from_str("getModel"))
                    .unwrap()
                    .dyn_into()
                    .unwrap();

            tracing::info!("get_model: {:?}", get_model);

            let model = get_model.call0(&e).unwrap();
            tracing::info!("model: {:?}", model);
            let on_did_change_content: js_sys::Function =
                js_sys::Reflect::get(&model, &JsValue::from_str("onDidChangeContent"))
                    .unwrap()
                    .dyn_into()
                    .unwrap();

            tracing::info!("onDidChangeContent: {:?}", on_did_change_content);

            let callback = wasm_bindgen::closure::Closure::<dyn FnMut(JsValue)>::new(move |v| {
                tracing::info!("change: {:?}", v);

                let get_value: js_sys::Function =
                    js_sys::Reflect::get(&model, &JsValue::from_str("getValue"))
                        .unwrap()
                        .dyn_into()
                        .unwrap();
                let content: JsValue = get_value.call0(&model).unwrap();

                onChange(content.as_string().unwrap());
            });

            let model = get_model.call0(&e).unwrap();

            on_did_change_content
                .call1(&model, callback.as_ref().unchecked_ref())
                .unwrap();

            callback.forget();

            *editor.write() = Some(e);
        });
        require
            .call2(
                &wasm_bindgen::JsValue::NULL,
                &arg1,
                callback.as_ref().unchecked_ref(),
            )
            .unwrap();
        callback.forget();
    });

    use_effect(move || {
        if editor.read().is_some() {
            // register an event listener for whether to use dark mode if editor is set

            let window = web_sys::window().unwrap();
            let callback = wasm_bindgen::closure::Closure::<dyn FnMut(JsValue)>::new(move |_| {
                // NOTE: setTheme is only defined for window.monaco.editor: https://stackoverflow.com/a/52357826/5479994
                let window = web_sys::window().unwrap();
                let monaco = js_sys::Reflect::get(&window, &JsValue::from_str("monaco")).unwrap();
                let editor = js_sys::Reflect::get(&monaco, &JsValue::from_str("editor")).unwrap();

                let set_theme: js_sys::Function =
                    js_sys::Reflect::get(&editor, &JsValue::from_str("setTheme"))
                        .unwrap()
                        .dyn_into()
                        .unwrap();

                let theme = if util::dark_mode().unwrap() {
                    "vs-dark"
                } else {
                    "vs"
                };

                set_theme
                    .call1(&wasm_bindgen::JsValue::NULL, &JsValue::from_str(theme))
                    .unwrap();
            });

            window
                .match_media("(prefers-color-scheme: dark)")
                .unwrap()
                .unwrap()
                .add_event_listener_with_callback("change", callback.as_ref().unchecked_ref())
                .unwrap();

            callback.forget();
        }
    });

    rsx! {
        div { id: "container", class: "w-full h-full" }
    }
}
