use super::MathExpr;

// Represents a fraction
#[derive(Debug, Clone)]
pub struct Fraction {
    numerator: MathExpr,
    denominator: MathExpr,
}
