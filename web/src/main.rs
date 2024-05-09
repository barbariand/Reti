use leptos::{mount_to_body, view};

mod components;
mod logging;

use crate::components::App;

#[cfg(all(target_arch="wasm32",target_os="unknown"))]
fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! {<App/>});
}

#[cfg(not(all(target_arch="wasm32",target_os="unknown")))]
fn main(){
    compile_error!("Use wasm32 target because katex does not work otherwise")
}