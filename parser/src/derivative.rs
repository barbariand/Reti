use crate::prelude::*;
///Helper trait for derivation
pub trait Derivative {
    ///get derivative
    fn derivative(
        &self,
        dependent: &MathIdentifier,
    ) -> Result<MathExpr, EvalError>;
}
