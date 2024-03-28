use super::MathExpr;

// Represents a matrix (2D vector of expressions)
#[derive(Debug, Clone)]
pub struct Matrix {
    rows: Vec<Vec<MathExpr>>,
}
