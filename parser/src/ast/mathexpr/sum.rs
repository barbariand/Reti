use super::{customfunc::CustomFunction, MathExprKey};

// Represents a summation
#[derive(Debug, Clone)]
pub struct Sum {
    expression: CustomFunction, // Expression to be summed
    variable: String,           // Variable of summation
    lower_bound: MathExprKey,   // Lower bound
    upper_bound: MathExprKey,   // Upper bound
}
