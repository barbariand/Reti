#![allow(non_snake_case)]
use crate::logging::init_logger;
use leptos::*;
use tracing::warn;

/// |-------------|
/// | input   |res|
/// |         |   |
/// |         |   |
/// |-------------|
/// |  katex      |
/// |-------------|
#[component]
pub fn App() -> impl IntoView {
    init_logger();
    let a = 1;
    view! {
        <div class={"container"}>
            <div class={"ui"}>
                <Editor/>
                <Results/>
            </div>
            <Output/>
        </div>
    }
}

#[component]
fn Editor() -> impl IntoView {
    warn!("This is called from Editor component!");
    view! {
        <div class={"editor"}>
            <div class={"linenumbers"}/>
            <div class={"buffer"}/>
        </div>
    }
}
#[component]
fn Results() -> impl IntoView {
    view! {
        <div>Results</div>
    }
}
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[component]
fn Output() -> impl IntoView {
    let (latex, set_latex) = create_signal("E = mc^2".to_string());

    let html = move || {
        let opts = katex::Opts::builder().display_mode(true).build().unwrap();
        katex::render_with_opts(&latex.get(), &opts).ok()
    };

    view! {
        <div>
            <input
            on:input=move |ev| {
                set_latex(event_target_value(&ev));
            }
            prop:value=latex
            />
            {move || if let Some(html) = html() {
                view! { <div inner_html=html /> }
            } else {
                view! { <div style="text-align: center; color: red;">{ latex }</div> }
            }}
        </div>
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[component]
fn Output() -> impl IntoView {
    view! {<div inner_html="Katex not supported" />}
}
