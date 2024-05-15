//!Tries to find out if they are the same
use crate::prelude::*;

use super::{
    helper::{NumberCompare, Simple},
    simplify::Simplify,
};
///the implementation part
pub trait MathEquality: PrivateMathEquality {
    ///the user part of the trait
    fn eq(&self, other: impl Into<MathExpr>) -> bool {
        let other_new: MathExpr = other.into();
        self.private_equals(&other_new)
    }
}
impl MathEquality for Simple {}
///tries to see if they are mathematically the same
pub trait PrivateMathEquality: Simplify + Clone {
    ///The implementation part
    fn private_equals(&self, other: &MathExpr) -> bool {
        self.clone().simple().equals(&other.clone().simple())
    }
    ///This is garantied to be in the simplest form as long as the Simple
    /// Implementation is correct and that if they are not the same type so both
    /// are not Term OR MathExpr OR Factor
    fn equals(&self, other: &Self) -> bool;
}
impl PrivateMathEquality for Simple {
    fn equals(&self, other: &Self) -> bool {
        match (self.math_expr(), other.math_expr()) {
            (
                MathExpr::Term(Term::Factor(a)),
                MathExpr::Term(Term::Factor(b)),
            ) => a.equals(b),
            (MathExpr::Term(a), MathExpr::Term(b)) => a.equals(b),
            (a, b) => a.equals(b),
        }
    }
}

impl PrivateMathEquality for MathExpr {
    fn equals(&self, other: &MathExpr) -> bool {
        match (self,other){
            (MathExpr::Term(_), MathExpr::Term(_)) => unreachable!("The Simple structs implementations should ensure that this is working"),
            (MathExpr::Add(_, _), MathExpr::Add(_, _)) => todo!(),
            (MathExpr::Subtract(_, _), MathExpr::Subtract(_, _)) => todo!(),
            _=>false
        }
    }
}
impl PrivateMathEquality for Term {
    fn equals(&self, other: &Term) -> bool {
        match (self,other){
            (Term::Factor(_), Term::Factor(_)) => unreachable!("The Simple structs implementations should ensure that this is working"),
            (Term::Multiply(_, _, _), Term::Multiply(_, _, _)) => todo!(),
            (Term::Divide(_, _), Term::Divide(_, _)) => todo!(),
            _=>false
        }
    }
}
impl PrivateMathEquality for Factor {
    fn equals(&self, other: &Factor) -> bool {
        match (self, other) {
            (Factor::Constant(c_1), Factor::Constant(c_2)) => c_1.equals(c_2),
            (Factor::Parenthesis(p_1), Factor::Parenthesis(p_2)) => {
                p_1.equals(p_2)
            }
            (Factor::Variable(_), Factor::Variable(_)) => todo!(),
            (Factor::FunctionCall(_), Factor::FunctionCall(_)) => todo!(),
            (
                Factor::Power {
                    base: _,
                    exponent: _,
                },
                Factor::Power {
                    base: _,
                    exponent: _,
                },
            ) => todo!(),
            (
                Factor::Root {
                    degree: _,
                    radicand: _,
                },
                Factor::Root {
                    degree: _,
                    radicand: _,
                },
            ) => todo!(),
            (Factor::Fraction(_, _), Factor::Fraction(_, _)) => todo!(),
            (Factor::Abs(_), Factor::Abs(_)) => todo!(),
            (Factor::Matrix(_), Factor::Matrix(_)) => todo!(),
            _ => false,
        }
    }
}
