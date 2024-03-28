use super::MathExpr;

// Represents a logarithm, potentially with a specific base
#[derive(Debug, Clone)]
pub struct Logarithm {
    base: Option<MathExpr>, // None for natural log, Some for base specified
    argument: MathExpr,
}
