use super::MathExpr;

// Represents a differential operation, e.g., d/dx
#[derive(Debug, Clone)]
pub struct Differential {
    variable: String,     // Variable of differentiation
    expression: MathExpr, // Expression to be differentiated
}
