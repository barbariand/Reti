use super::{MathExpr, MathExprKey};

// Represents a fraction
#[derive(Debug, Clone)]
pub struct Fraction {
    numerator: MathExprKey,
    denominator: MathExprKey,
}
