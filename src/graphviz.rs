use wasm_bindgen::prelude::*;

pub async fn svg(
    dot: &petgraph::dot::Dot<'_, &'_ petgraph::graph::DiGraph<&'_ crate::ir::Name, ()>>,
) -> String {
    let cfg_dot = format!("{:?}", dot);

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

    let cfg: JsValue = dot
        .call1(&graphviz, &JsValue::from_str(&cfg_dot))
        .unwrap()
        .dyn_into()
        .unwrap();

    let svg = cfg.dyn_ref::<js_sys::JsString>().unwrap().to_string();
    svg.into()
}

pub async fn svg2(
    dot: &petgraph::dot::Dot<'_, &'_ petgraph::graph::UnGraph<Option<usize>, ()>>,
) -> String {
    let cfg_dot = format!("{:?}", dot);

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

    let cfg: JsValue = dot
        .call1(&graphviz, &JsValue::from_str(&cfg_dot))
        .unwrap()
        .dyn_into()
        .unwrap();

    let svg = cfg.dyn_ref::<js_sys::JsString>().unwrap().to_string();
    svg.into()
}
