use parser::{ast::simplify::Simplify, prelude::*};
use std::sync::Mutex;
use tracing::{debug, info};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::RT;
#[derive(serde::Serialize, serde::Deserialize, tsify_next::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct RetiJsError {
    display: String,
    error: RetiJsErrorEnum,
}
#[derive(serde::Serialize, serde::Deserialize, tsify_next::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum RetiJsErrorEnum {
    EvalError(EvalError),
    AstError(AstError),
    Unknown,
}
impl From<AstError> for RetiJsError {
    fn from(value: AstError) -> Self {
        Self {
            display: format!("{value}"),
            error: RetiJsErrorEnum::AstError(value),
        }
    }
}
impl From<EvalError> for RetiJsError {
    fn from(value: EvalError) -> Self {
        Self {
            display: format!("{value}"),
            error: RetiJsErrorEnum::EvalError(value),
        }
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
    pub fn parse(
        &mut self,
        text: String,
    ) -> Result<RetiJsEValuation, RetiJsError> {
        debug!("starting parse");
        let lock = self.0.lock().expect("Failed to get lock");
        debug!("got mutex for parse");
        let func = parse(&text, lock.context());
        debug!("got function, now executing it");
        let res = RT.block_on(func)?;
        info!("parsed to ast");
        drop(lock);
        Ok(self.eval_ast(res)?)
    }
    fn eval_ast(&self, ast: Ast) -> Result<RetiJsEValuation, EvalError> {
        info!("starting evaluation");
        let mut lock = self.0.lock().expect("Failed to get lock");
        info!("got mutex for evaluation");
        let simple = ast.simple(lock.context())?;
        info!("got simple for evaluation");
        lock.eval_ast(simple).map(|v| v.into())
    }
}
#[derive(serde::Serialize, serde::Deserialize, tsify_next::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct RetiJsEValuation {
    tag: RetiJsEValuationTag,
    latex: String,
}
impl From<Evaluation> for RetiJsEValuation {
    fn from(value: Evaluation) -> Self {
        match value {
            Evaluation::AddedFunction(v) => Self {
                tag: RetiJsEValuationTag::AddedFunction,
                latex: v,
            },
            Evaluation::AddedVariable(v) => Self {
                tag: RetiJsEValuationTag::AddedVariable,
                latex: v,
            },
            Evaluation::LaTeX(v) => Self {
                tag: RetiJsEValuationTag::Evaluation,
                latex: v,
            },
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize, tsify_next::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum RetiJsEValuationTag {
    AddedFunction,
    AddedVariable,
    Evaluation,
}
