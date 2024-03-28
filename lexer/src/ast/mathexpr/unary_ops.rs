use super::MathExpr;

// Represents unary operations
#[derive(Debug, Clone)]
pub struct UnaryOp {
    op: UnaryOperator,
    expr: MathExpr,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Negate,
    Differential,
}
