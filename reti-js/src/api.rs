use std::sync::Mutex;

use parser::{ast::simplify::Simplify, prelude::*};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::RT;
#[wasm_bindgen(js_name = "RetiJS")]
pub struct JsAPI(Mutex<Evaluator>);
#[wasm_bindgen(js_class = RetiJS)]
impl JsAPI {
    #[wasm_bindgen(constructor)]
    pub fn standard_math() -> JsAPI {
        JsAPI(Mutex::new(Evaluator::standard_math()))
    }
    fn eval_ast(&self, ast: Ast) -> Result<Evaluation, EvalError> {
        let mut lock = self.0.lock().expect("Failed to get lock");
        let simple = ast.simple(lock.context())?;
        lock.eval_ast(simple)
    }

    pub fn parse(&self, text: String) -> Result<String, String> {
        let guard = self.0.lock().expect("Failed to get lock");
        let res = RT
            .block_on(parse(&text, guard.context()))
            .map_err(|v| format!("{v}"))?;
        drop(guard);
        self.eval_ast(res)
            .map(|v| v.to_string())
            .map_err(|v| v.to_string())
    }
}
