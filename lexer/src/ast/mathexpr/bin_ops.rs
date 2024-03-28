use super::MathExpr;

// Represents binary operations
#[derive(Debug, Clone)]
pub struct BinOp {
    op: BinOperator,
    left: MathExpr,
    right: MathExpr,
}

#[derive(Debug, Clone)]
pub enum BinOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}
