use super::MathExprKey;

// Represents a fraction
#[derive(Debug, Clone)]
pub struct Fraction {
    numerator: MathExprKey,
    denominator: MathExprKey,
}
