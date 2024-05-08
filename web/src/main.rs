#![allow(non_snake_case)]
use leptos::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! {<App/>});
}

/// |-------------|
/// | input   |res|
/// |         |   |
/// |         |   |
/// |-------------|
/// |  katex      |
/// |-------------|
#[component]
fn App() -> impl IntoView {
    view! {
        <div class={"container"}>
            <div class={"ui"}>
                <Canvas/>
                <Results/>
            </div>
            <Output/>
        </div>
    }
}

#[component]
fn Canvas() -> impl IntoView {
    view! {
        <div>Canvas</div>
    }
}
#[component]
fn Results() -> impl IntoView {
    view! {
        <div>Results</div>
    }
}
#[component]
fn Output() -> impl IntoView {
    let opts = katex::Opts::builder().display_mode(true).build().unwrap();
    let html_in_display_mode =
        katex::render_with_opts("E = mc^2", &opts).unwrap();
    view! {<div inner_html=html_in_display_mode />}
}
