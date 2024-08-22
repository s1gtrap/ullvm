#![allow(non_snake_case)]

use std::collections::HashSet;

use dioxus::prelude::*;
use js_sys::Reflect::prevent_extensions;
use petgraph::data::DataMapMut;
use tracing::Level;
use wasm_bindgen::prelude::*;

mod code;
mod cursor;
mod graphviz;
mod ir;
mod iter_prev;
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
pub fn U8(i: u8) -> Element {
    rsx! {
        p { "{i}" }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct U8Iter(std::ops::Range<u8>);

impl Iterator for U8Iter {
    type Item = U8Props;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|i| U8Props { i })
    }
}

#[component]
pub fn GraphDisplay(i: u8) -> Element {
    rsx! {
        p { "{i}" }
    }
}

#[derive(Clone, Debug)]
pub struct Graph(petgraph::graph::UnGraph<Option<usize>, ()>);

impl Graph {
    fn new() -> Self {
        use petgraph::graph::UnGraph;
        let g = UnGraph::<Option<usize>, ()>::from_edges(&[(1, 2), (2, 3), (3, 4), (1, 4)]);
        Graph(g)
    }
}

fn graph_eq<N, E, Ty, Ix>(
    a: &petgraph::Graph<N, E, Ty, Ix>,
    b: &petgraph::Graph<N, E, Ty, Ix>,
) -> bool
where
    N: PartialEq,
    E: PartialEq,
    Ty: petgraph::EdgeType,
    Ix: petgraph::graph::IndexType + PartialEq,
{
    let a_ns = a.raw_nodes().iter().map(|n| &n.weight);
    let b_ns = b.raw_nodes().iter().map(|n| &n.weight);
    let a_es = a
        .raw_edges()
        .iter()
        .map(|e| (e.source(), e.target(), &e.weight));
    let b_es = b
        .raw_edges()
        .iter()
        .map(|e| (e.source(), e.target(), &e.weight));
    a_ns.eq(b_ns) && a_es.eq(b_es)
}

impl PartialEq<Graph> for Graph {
    fn eq(&self, other: &Graph) -> bool {
        tracing::info!(
            "{:?} ?= {:?}",
            petgraph::dot::Dot::new(&self.0),
            petgraph::dot::Dot::new(&other.0)
        );
        graph_eq(&self.0, &other.0)
    }
}

#[component]
pub fn DrawGraph(i: Graph) -> Element {
    use petgraph::dot::Dot;
    tracing::info!("draw_graph");

    let mut svg = use_signal(|| None::<String>);
    let g = i.0.clone();
    use_future(move || async move {
        tracing::info!("future");
    });
    //use_effect(move || {
    tracing::info!("render");
    let g = g.clone();
    spawn(async move {
        tracing::info!("pre-render");
        let dot = Dot::with_attr_getters(
            &g,
            &[petgraph::dot::Config::EdgeNoLabel],
            &|_, _| String::new(),
            &|_, n| match n.1 {
                Some(_) => {
                    use petgraph::visit::NodeRef;
                    let n = n.weight().unwrap_or(0);
                    let r = n % 3;
                    let g = (n / 3) % 3;
                    let b = (n / 9) % 3;
                    let lumi = 0.2126 * (r as f32 * 100.0)
                        + 0.7152 * (g as f32 * 100.0)
                        + 0.0722 * (b as f32 * 100.0); // per ITU-R BT.709
                    tracing::info!("lumi={lumi}");
                    format!(
                        "style=filled,color=\"#{:0>2x}{:0>2x}{:0>2x}\"{}",
                        r * 0x7f,
                        g * 0x7f,
                        b * 0x7f,
                        if lumi < 40.0 { ",fontcolor=white" } else { "" }
                    )
                }
                None => String::new(),
            },
        );
        tracing::info!("rendered: {dot:?}");
        let svgdot = graphviz::svg2(&dot).await;
        if *svg.read() != Some(svgdot.clone()) {
            *svg.write() = Some(svgdot.to_string());
        }
    });

    rsx! {
        p {
            dangerous_inner_html: svg().unwrap_or("".into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphIter(Graph);

impl GraphIter {
    pub fn new() -> Self {
        GraphIter(Graph::new())
    }
}

impl Iterator for GraphIter {
    type Item = DrawGraphProps;
    fn next(&mut self) -> Option<Self::Item> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        tracing::info!("NEXT");

        use petgraph::visit::IntoNodeReferences;
        let n = self.0 .0.node_count() as u32;
        let mut rng = rand::thread_rng();
        let i = rng.gen_range(0..n);
        let c = self
            .0
             .0
            .node_references()
            .filter(|(_, c)| c.is_some())
            .count();
        if let Some(n) = self.0 .0.node_weights_mut().filter(|n| n.is_none()).next() {
            *n = Some(c);
        }
        if rng.gen_range(0..100) > 1 {
            tracing::info!("some");
            Some(DrawGraphProps { i: self.0.clone() }) // TODO: this
        } else {
            tracing::info!("none");
            None
        }
        // use petgraph::visit::IntoNodeReferences;
        // use rand::Rng;
        // let n = self.0 .0.node_count() as u32;
        // let mut rng = rand::thread_rng();
        // let i = rng.gen_range(0..n);
        // let c = self
        //     .0
        //      .0
        //     .node_references()
        //     .filter(|(_, c)| c.is_some())
        //     .count();
        // *self.0 .0.node_weight_mut(i.into()).unwrap() = Some(c);
        // Some(DrawGraphProps { i: self.0.clone() }) // TODO: this
    }
}

#[component]
fn App() -> Element {
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
                    lva::Lva { new: a.2 }
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
    let mut editor = use_signal(|| None::<JsValue>);
    use_effect(move || {
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
        let callback = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
            tracing::info!("monaco is ready!");
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

            let arg2 = js_sys::JSON::parse(&format!(
                r#"{{
    "value": {:?},
    "language": "llvm",
    "minimap": {{ "enabled": false }},
    "automaticLayout": true
}}"#,
                include_str!("../examples/ll/for1.ll"),
            ))
            .unwrap();

            *editor.write() = Some(create.call2(&monaco_editor, &container, &arg2).unwrap());
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
    let prev_iter = use_signal(|| iter_prev::Iter::new(U8Iter(0..5)));
    let graph_iter = use_signal(|| iter_prev::Iter::new(GraphIter::new()));
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
                                    let input = EXAMPLES[pick].1;
                                    tracing::info!("input changed to: \"{}\"", input);
                                    if let Some(ref editor) = *editor.read() {
                                        let get_model: js_sys::Function = js_sys::Reflect::get(
                                                editor,
                                                &JsValue::from_str("getModel"),
                                            )
                                            .unwrap()
                                            .dyn_into()
                                            .unwrap();
                                        let model = get_model.call0(editor).unwrap();
                                        let set_value: js_sys::Function = js_sys::Reflect::get(
                                                &model,
                                                &JsValue::from_str("setValue"),
                                            )
                                            .unwrap()
                                            .dyn_into()
                                            .unwrap();
                                        set_value.call1(&model, &JsValue::from_str(input)).unwrap();
                                    }
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
                            div { id: "container", class: "w-full h-full" }
                        }
                        div { class: "flex-none",
                            button {
                                class: "h-12 w-full bg-slate-100",
                                onclick: move |_| async move {
                                    if let Some(ref editor) = *editor.read() {
                                        let get_model: js_sys::Function = js_sys::Reflect::get(
                                                editor,
                                                &JsValue::from_str("getModel"),
                                            )
                                            .unwrap()
                                            .dyn_into()
                                            .unwrap();
                                        let model = get_model.call0(editor).unwrap();
                                        let get_value: js_sys::Function = js_sys::Reflect::get(
                                                &model,
                                                &JsValue::from_str("getValue"),
                                            )
                                            .unwrap()
                                            .dyn_into()
                                            .unwrap();
                                        let input = get_value.call0(&model).unwrap();
                                        Module::ccall(
                                            JsValue::from_str("parse"),
                                            JsValue::NULL,
                                            js_sys::Array::of1(&JsValue::from_str("string")).into(),
                                            js_sys::Array::of1(&input).into(),
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
                                        let hpccWasm = js_sys::Reflect::get(
                                                &window,
                                                &JsValue::from_str("@hpcc-js/wasm"),
                                            )
                                            .unwrap();
                                        tracing::info!("{hpccWasm:?}");
                                        let graphviz = js_sys::Reflect::get(
                                                &hpccWasm,
                                                &JsValue::from_str("Graphviz"),
                                            )
                                            .unwrap();
                                        tracing::info!("{graphviz:?}");
                                        let load = js_sys::Reflect::get(&graphviz, &JsValue::from_str("load"))
                                            .unwrap();
                                        let load: &js_sys::Function = load.dyn_ref().unwrap();
                                        let promise: js_sys::Promise = load
                                            .call0(&graphviz)
                                            .unwrap()
                                            .dyn_into()
                                            .unwrap();
                                        tracing::info!("{promise:?}");
                                        let graphviz = wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
                                        let dot = js_sys::Reflect::get(&graphviz, &JsValue::from_str("dot"))
                                            .unwrap();
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
                                                iter_prev::Iter::new(iter)
                                            })
                                            .collect();
                                    }
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
                                "Cursor".to_string(),
                                rsx! {
                                    cursor::Cursor { init : U8Props { i : 69 }, iter : prev_iter, c : U8 }
                                },
                            ),
                            (
                                "Graph".to_string(),
                                rsx! {
                                    cursor::Cursor { init : DrawGraphProps { i : Graph::new() }, iter : graph_iter, c : DrawGraph }
                                },
                            ),
                        ]
                    }
                }
            }
        }
    }
}
