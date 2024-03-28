use super::MathExpr;

// Represents function calls
#[derive(Debug, Clone)]
pub struct FunctionCall {
    function: MathFunction,
    argument: MathExpr,
}

#[derive(Debug, Clone)]
pub enum MathFunction {
    Sin,
    Cos,
    Exp,
}
