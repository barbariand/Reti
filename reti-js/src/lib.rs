#![allow(non_snake_case)]
mod api;

pub use api::JsAPI;
use console_error_panic_hook;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref RT: Runtime = Builder::new_current_thread().build().unwrap();
}
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // print pretty errors in wasm https://github.com/rustwasm/console_error_panic_hook
    // This is not needed for tracing_wasm to work, but it is a common tool for
    // getting proper error line numbers for panics.
    console_error_panic_hook::set_once();

    // Add this line:
    tracing_wasm::set_as_global_default();

    Ok(())
}
use tokio::runtime::{Builder, Runtime};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
