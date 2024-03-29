use super::{MathExpr, MathExprKey};

#[derive(Debug, Clone)]
pub struct Modulo {
    val: MathExprKey,
    modulo: MathExprKey,
}
