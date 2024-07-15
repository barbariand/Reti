//! into and from implementations
use crate::prelude::*;

use super::{helper::Simple, simplify::Simplify};
impl From<Term> for MathExpr {
    fn from(value: Term) -> Self {
        MathExpr::Term(value)
    }
}
impl<T: Simplify + Into<MathExpr>> From<Simple<T>> for MathExpr {
    fn from(value: Simple<T>) -> Self {
        value.0.into()
    }
}

impl From<Factor> for MathExpr {
    fn from(value: Factor) -> Self {
        MathExpr::Term(Term::Factor(value))
    }
}
impl From<f64> for MathExpr {
    fn from(value: f64) -> Self {
        MathExpr::Term(Term::from(value))
    }
}
impl From<f64> for Box<MathExpr> {
    fn from(value: f64) -> Self {
        Box::new(value.into())
    }
}
impl From<FunctionCall> for MathExpr {
    fn from(value: FunctionCall) -> Self {
        MathExpr::Term(Term::from(value))
    }
}
impl From<MathIdentifier> for MathExpr {
    fn from(value: MathIdentifier) -> Self {
        MathExpr::Term(Term::from(value))
    }
}
impl From<Factor> for Box<MathExpr> {
    fn from(value: Factor) -> Self {
        Box::new(MathExpr::Term(Term::Factor(value)))
    }
}
impl From<Term> for Box<MathExpr> {
    fn from(value: Term) -> Self {
        Box::new(MathExpr::Term(value))
    }
}

impl From<Factor> for Term {
    fn from(value: Factor) -> Self {
        Self::Factor(value)
    }
}
impl From<f64> for Term {
    fn from(value: f64) -> Self {
        Term::Factor(Factor::Constant(value))
    }
}
impl From<MathIdentifier> for Term {
    fn from(value: MathIdentifier) -> Self {
        Term::Factor(Factor::Variable(value))
    }
}
impl From<FunctionCall> for Term {
    fn from(value: FunctionCall) -> Self {
        Term::Factor(Factor::FunctionCall(value))
    }
}
impl From<Factor> for Box<Term> {
    fn from(value: Factor) -> Self {
        Box::new(Term::Factor(value))
    }
}
impl From<f64> for Factor {
    fn from(value: f64) -> Self {
        Factor::Constant(value)
    }
}
impl From<MathIdentifier> for Factor {
    fn from(value: MathIdentifier) -> Self {
        Factor::Variable(value)
    }
}
impl From<FunctionCall> for Factor {
    fn from(value: FunctionCall) -> Self {
        Factor::FunctionCall(value)
    }
}
