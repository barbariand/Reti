use super::{MathExpr, MathExprKey};

// Represents a square root or nth root
#[derive(Debug, Clone)]
pub struct Root {
    pub degree: Option<MathExprKey>, // None for square roots, Some for nth roots
    pub radicand: MathExprKey,
}