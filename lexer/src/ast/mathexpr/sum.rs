use super::{customfunc::CustomFunction, MathExpr};

// Represents a summation
#[derive(Debug, Clone)]
pub struct Sum {
    expression: CustomFunction, // Expression to be summed
    variable: String,           // Variable of summation
    lower_bound: MathExpr,      // Lower bound
    upper_bound: MathExpr,      // Upper bound
}
