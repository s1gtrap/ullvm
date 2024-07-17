use dioxus::prelude::*;

#[component]
fn Empty(p: ()) -> Element {
    rsx! {

        p { "asd" }
    }
}

#[component]
fn Generic<P: Properties + std::cmp::PartialEq>(p: P, c: Component<P>) -> Element {
    rsx! {
        p { {c(p)} }
    }
}

#[component]
pub fn Cursor<
    P: Properties + std::cmp::PartialEq,
    I: Iterator<Item = P> + std::cmp::PartialEq + 'static,
>(
    init: P,
    iter: Signal<crate::iter_prev::Iter<I>>,
    c: Component<P>,
) -> Element {
    let mut body = use_signal(|| init.clone());
    let mut is_first = use_signal(|| true);
    let mut is_last = use_signal(|| false);
    let mut first = {
        let init = init.clone();
        move || {
            *is_first.write() = true;
            *is_last.write() = false;
            if let Some(_i) = iter.write().first() {
                *body.write() = init.clone();
            }
        }
    };
    let mut prev = move || {
        *is_last.write() = false;
        if let Some(i) = iter.write().prev() {
            *body.write() = i.clone();
        } else {
            *body.write() = init.clone();
            *is_first.write() = true;
        }
    };
    let mut next = move || {
        *is_first.write() = false;
        if let Some(i) = iter.write().next() {
            *body.write() = i.clone();
        } else {
            *is_last.write() = true;
        }
    };
    let mut last = move || {
        *is_first.write() = false;
        *is_last.write() = true;
        if let Some(i) = iter.write().by_ref().last() {
            *body.write() = i.clone();
        }
    };
    //Generic { p: EmptyProps { p: () }, c: Empty }
    rsx! {
        div { class: "flex flex-col h-full",
            div { class: "grow", {c(body.read().clone())} }
            div { class: "flex h-12 w-full",
                button {
                    class: "grow bg-white disabled:bg-slate-50 disabled:text-slate-500",
                    disabled: is_first,
                    onclick: move |_| first(),
                    "<<"
                }
                button {
                    class: "grow bg-white disabled:bg-slate-50 disabled:text-slate-500",
                    disabled: is_first,
                    onclick: move |_| prev(),
                    "<"
                }
                button {
                    class: "grow bg-white disabled:bg-slate-50 disabled:text-slate-500",
                    disabled: is_last,
                    onclick: move |_| next(),
                    ">"
                }
                button {
                    class: "grow bg-white disabled:bg-slate-50 disabled:text-slate-500",
                    disabled: is_last,
                    onclick: move |_| last(),
                    ">>"
                }
            }
        }
    }
}
