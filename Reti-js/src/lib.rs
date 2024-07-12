use std::fmt::format;

use parser::{
    approximator::Approximator,
    ast::{simplify::Simplify, Ast},
    context::MathContext,
    error::AstError,
    parse,
};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
pub async fn test() -> Result<JsValue, String> {
    let aprox = Approximator::new(MathContext::default());
    let parsed = parse("1+1", aprox.context())
        .await
        .map_err(|e| format!("{e}"))?;
    match parsed {
        Ast::Expression(expr) => expr
            .simple(aprox.context())
            .and_then(|v| aprox.eval_expr(v))
            .map_err(|e| format!("{e}"))
            .map(|v| {
                serde_wasm_bindgen::to_value(&v)
                    .expect("could not make jsvalue")
            }),
        Ast::Equality(_, _) => todo!(),
    }
}
