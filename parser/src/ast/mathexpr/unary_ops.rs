use super::MathExprKey;

// Represents unary operations
#[derive(Debug, Clone)]
pub struct UnaryOp {
    op: UnaryOperator,
    expr: MathExprKey,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Negate,
    Derrivate,
}
