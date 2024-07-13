//! predefined functions
use crate::prelude::*;
///type alias for the Const Function Return
type ConstFunctionResult = Result<Value, EvalError>;
///type alias for the Const Derived Function Return
type ConstDeriveFunctionResult = Result<MathExpr, EvalError>;
/// Mathematical functions
pub mod math {

    use crate::identifier::OtherSymbol;

    use super::{
        ConstDeriveFunctionResult as CDFR, ConstFunctionResult as CFR, Factor,
        FunctionCall, MathExpr, MathIdentifier, Value,
    };
    /// mathematical sin
    pub fn sin(v: Vec<Value>) -> CFR {
        v[0].map_expecting_scalar(|v| v.sin())
    }
    /// mathematical derivation of sin
    pub fn sin_derive(math: Vec<MathExpr>) -> CDFR {
        Ok(Factor::FunctionCall(FunctionCall::new(
            MathIdentifier::from_single_symbol(OtherSymbol::Cos),
            math,
        ))
        .into())
    }
    /// mathematical cos
    pub fn cos(v: Vec<Value>) -> CFR {
        v[0].map_expecting_scalar(|v| v.sin())
    }
    /// mathematical derivation of cos
    pub fn cos_derive(math: Vec<MathExpr>) -> CDFR {
        Ok(Factor::FunctionCall(FunctionCall::new(
            MathIdentifier::from_single_symbol(OtherSymbol::Sin),
            math,
        ))
        .into())
    }
    /// mathematical tan
    pub fn tan(v: Vec<Value>) -> CFR {
        v[0].map_expecting_scalar(|v| v.tan())
    }
    /// mathematical derivation of tan
    pub fn tan_derive(math: Vec<MathExpr>) -> CDFR {
        Ok(Factor::Power {
            base: Factor::FunctionCall(FunctionCall::new(
                MathIdentifier::from_single_symbol(OtherSymbol::Tan),
                math,
            ))
            .into(),
            exponent: (-2.0).into(),
        }
        .into())
    }
}
