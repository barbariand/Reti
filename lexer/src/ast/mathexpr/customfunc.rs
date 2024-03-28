use super::MathExpr;

// Represents a custom-defined function or operator
#[derive(Debug, Clone)]
pub struct CustomFunction {
    name: String,
    arguments: Vec<MathExpr>,
}
