use dioxus::prelude::*;

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

#[component]
pub fn ExamplePicker(onpick: EventHandler<String>) -> Element {
    rsx! {
        select {
            id: "example-picker",
            class: "h-12 w-full bg-slate-100",
            onchange: move |e: Event<FormData>| {
                let pick = e.data.value().parse::<usize>().unwrap() - 1;
                let input = EXAMPLES[pick].1;
                tracing::info!("input changed to: \"{}\"", input);
                onpick.call(input.to_string());
            },
            option { value: "0", disabled: true, "-- example --" }
            for (i , (n , _)) in EXAMPLES.iter().enumerate() {
                option { key: "{i}", value: "{i + 1}", "{n}" }
            }
        }
    }
}
