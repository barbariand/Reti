use super::MathExpr;

// Represents function calls
#[derive(Debug, Clone)]
pub struct UnaryFunctions {
    function: MathFunction,
    argument: MathExpr,
}

#[derive(Debug, Clone)]
pub enum MathFunction {
    Sin,
    Cos,
    Tan,
    Abs,
    Floor,
    Ceil,
    ACos,
    ASin,
    ATan,
    Ln,
    Log10,
    PowE,
    Pow10
}
