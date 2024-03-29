use std::ops::Neg;

use super::{MathExpr, MathExprKey};

// Represents a matrix (2D vector of expressions)
#[derive(Debug, Clone)]
pub struct Matrix {
    rows: Vec<MathExprKey>,
    n: usize,
    m: usize,
}
