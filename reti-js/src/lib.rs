mod api;

use api::API;
use console_error_panic_hook;
use lazy_static::lazy_static;

lazy_static! {
    static ref RT: Runtime = Builder::new_current_thread().build().unwrap();
}
#[wasm_bindgen(start)]
pub fn init_wasm() {
    console_error_panic_hook::set_once();
}
use tokio::runtime::{Builder, Runtime};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn parse(test: String) -> Result<String, String> {
    let parsed =
        RT.block_on(async { API.parse(test).await});
    let parsed_ast = parsed.map_err(|e| format!("{e}"))?;
    
    API.eval_ast(parsed_ast).map(|v|v.to_string()).map_err(|err|err.to_string())
}
