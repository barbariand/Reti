use super::MathExpr;

// Represents a square root or nth root
#[derive(Debug, Clone)]
pub struct Root {
    degree: Option<MathExpr>, // None for square roots, Some for nth roots
    radicand: MathExpr,
}
