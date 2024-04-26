use super::MathExprKey;

// Represents an integral
#[derive(Debug, Clone)]
pub struct Integral {
    integrand: MathExprKey,           // Expression to be integrated
    variable: String,                 // Variable of integration
    lower_bound: Option<MathExprKey>, // Optional lower bound
    upper_bound: Option<MathExprKey>, // Optional upper bound
}
