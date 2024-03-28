use super::MathExprKey;

// Represents binary operations
#[derive(Debug, Clone)]
pub struct BinOp {
    op: BinOperator,
    left: MathExprKey,
    right: MathExprKey,
}

#[derive(Debug, Clone)]
pub enum BinOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}
