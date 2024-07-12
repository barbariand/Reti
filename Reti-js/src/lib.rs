use console_error_panic_hook;
use lazy_static::lazy_static;
use parser::{
    approximator::Approximator,
    ast::{simplify::Simplify, Ast},
    context::MathContext,
    parse as parse_internal,
};

lazy_static! {
    static ref RT: Runtime = Builder::new_current_thread().build().unwrap();
}
#[wasm_bindgen(start)]
pub fn init_wasm() {
    console_error_panic_hook::set_once()
}
use tokio::runtime::{Builder, Runtime};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
#[wasm_bindgen]
pub fn parse(s: String) -> Result<JsValue, String> {
    let aprox = Approximator::new(MathContext::default());
    let parsed =
        RT.block_on(async { parse_internal(&s, aprox.context()).await });
    let parsed = parsed.map_err(|e| format!("{e}"))?;
    match parsed {
        Ast::Expression(expr) => expr
            .simple(aprox.context())
            .and_then(|v| aprox.eval_expr(v))
            .map_err(|e| format!("{e}"))
            .map(|v| {
                serde_wasm_bindgen::to_value(&v)
                    .expect("could not make jsvalue")
            }),
        Ast::Equality(_, _) => Err("hello i cant do equality".to_owned()),
    }
}
