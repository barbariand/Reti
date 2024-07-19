mod api;

pub use api::JsAPI;
use console_error_panic_hook;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref RT: Runtime = Builder::new_current_thread()
        .build()
        .expect("failed to create runtime in wasm");
}
#[wasm_bindgen(start)]
pub fn init_wasm() {
    console_error_panic_hook::set_once();
}
use tokio::runtime::{Builder, Runtime};
use wasm_bindgen::prelude::wasm_bindgen;
