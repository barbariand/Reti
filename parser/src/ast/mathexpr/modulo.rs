use super::MathExprKey;

#[derive(Debug, Clone)]
pub struct Modulo {
    val: MathExprKey,
    modulo: MathExprKey,
}
