use super::MathExprKey;

// Represents function calls
#[derive(Debug, Clone)]
pub struct UnaryFunctions {
    function: MathFunction,
    argument: MathExprKey,
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
    Pow10,
}
