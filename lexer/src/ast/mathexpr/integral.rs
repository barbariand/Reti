use super::MathExpr;

// Represents an integral
#[derive(Debug, Clone)]
pub struct Integral {
    integrand: MathExpr,           // Expression to be integrated
    variable: String,              // Variable of integration
    lower_bound: Option<MathExpr>, // Optional lower bound
    upper_bound: Option<MathExpr>, // Optional upper bound
}
