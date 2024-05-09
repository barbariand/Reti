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
    let a=1;
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
#[cfg(all(target_arch="wasm32",target_os="unknown"))]
#[component]
fn Output() -> impl IntoView {
    let opts = katex::Opts::builder().display_mode(true).build().unwrap();
    let html_in_display_mode =
        katex::render_with_opts("E = mc^2", &opts).unwrap();
    view! {<div inner_html=html_in_display_mode />}
}

#[cfg(not(target_arch="wasm32"))]
#[component]
fn Output() -> impl IntoView {

    view! {<div inner_html="Katex not supported" />}
}
