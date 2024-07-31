use parser::{ast::simplify::Simplify, prelude::*};
use std::sync::Mutex;
use tracing::info;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::RT;
#[derive(serde::Serialize, serde::Deserialize, tsify_next::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum RetiJsError {
    EvalError(EvalError),
    AstError(AstError),
}
impl From<AstError> for RetiJsError {
    fn from(value: AstError) -> Self {
        RetiJsError::AstError(value)
    }
}
impl From<EvalError> for RetiJsError {
    fn from(value: EvalError) -> Self {
        RetiJsError::EvalError(value)
    }
}

#[wasm_bindgen(js_name = "RetiJS")]
pub struct JsAPI(Mutex<Evaluator>);
#[wasm_bindgen(js_class = RetiJS)]
impl JsAPI {
    #[wasm_bindgen(constructor)]
    pub fn standard_math() -> JsAPI {
        JsAPI(Mutex::new(Evaluator::standard_math()))
    }
    pub fn parse(&mut self, text: String) -> Result<Evaluation, RetiJsError> {
        info!("starting parse");
        let lock = self.0.lock().expect("Failed to get lock");
        info!("got mutex for parse");
        let res = RT.block_on(parse(&text, lock.context()))?;
        info!("parsed to ast");
        drop(lock);
        Ok(self.eval_ast(res)?)
    }
    fn eval_ast(&self, ast: Ast) -> Result<Evaluation, EvalError> {
        info!("starting evaluation");
        let mut lock = self.0.lock().expect("Failed to get lock");
        info!("got mutex for evaluation");
        let simple = ast.simple(lock.context())?;
        info!("got simple for evaluation");
        lock.eval_ast(simple)
    }
}
