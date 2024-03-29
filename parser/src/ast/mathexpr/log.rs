use super::MathExprKey;

// Represents a logarithm, potentially with a specific base
#[derive(Debug, Clone)]
pub struct Logarithm {
    base: Option<MathExprKey>, // None for natural log, Some for base specified
    argument: MathExprKey,
}
