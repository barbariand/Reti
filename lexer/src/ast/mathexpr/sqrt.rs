use super::{MathExpr, MathExprKey};

// Represents a square root or nth root
#[derive(Debug, Clone)]
pub struct Root {
    degree: Option<MathExprKey>, // None for square roots, Some for nth roots
    radicand: MathExprKey,
}
