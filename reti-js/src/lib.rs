#![allow(non_snake_case)]
mod api;

use std::sync::atomic::{AtomicBool, Ordering};

pub use api::JsAPI;
use console_error_panic_hook;
use lazy_static::lazy_static;
lazy_static! {
    pub static ref RT: Runtime = Builder::new_current_thread().build().unwrap();
}
pub static STARTED: AtomicBool = AtomicBool::new(false);

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
#[wasm_bindgen(start)]
pub fn start() {
    use Ordering::*;
    if STARTED.load(SeqCst) {
        return;
    }
    // print pretty errors in wasm https://github.com/rustwasm/console_error_panic_hook
    // This is not needed for tracing_wasm to work, but it is a common tool for
    // getting proper error line numbers for panics.
    console_error_panic_hook::set_once();
    log("Hello start");
    // Add this line:
    tracing_wasm::set_as_global_default();
    STARTED.store(true, SeqCst)
}
use tokio::runtime::{Builder, Runtime};
use wasm_bindgen::prelude::wasm_bindgen;
