use super::{MathExpr, MathExprKey};

// Represents a differential operation, e.g., d/dx
#[derive(Debug, Clone)]
pub struct Differential {
    variable: String,        // Variable of differentiation
    expression: MathExprKey, // Expression to be differentiated
}
