use std::sync::Mutex;

use parser::{ast::{simplify::Simplify, Ast}, error::{AstError, EvalError}, parse, prelude::{Evaluation, Evaluator}};
use lazy_static::lazy_static;
lazy_static!{
    pub static ref API: JsAPI = JsAPI::standard_math();
} 

pub struct JsAPI(Mutex<Evaluator>);
impl JsAPI{
    pub fn standard_math()->Self{
        JsAPI(Mutex::new(Evaluator::standard_math()))
    }
    pub fn eval_ast(&self,ast:Ast)->Result<Evaluation,EvalError>{
        let mut lock=self.0.lock().expect("Failed to get lock");
        let simple=ast.simple(lock.context())?;
        lock.eval_ast(simple)
    }
    pub async fn parse(&self,text:String)->Result<Ast, AstError>{
        let guard=self.0.lock().expect("Failed to get lock");
        parse(&text, guard.context()).await
    }
}