//!Tries to find out if they are the same
use crate::prelude::*;

use super::{
    helper::{NumberCompare, Simple},
    simplify::Simplify,
};
///the implementation part
#[allow(private_bounds)]
pub trait MathEquality<T = Self>: PrivateMathEquality<T>
where
    T: Simplify<T>+Into<MathExpr>,
{
    ///the user part of the trait
    fn equivalent(&self, other: &Self, cont: &MathContext) -> bool {
        self.private_equals(other, cont)
    }
}
impl MathEquality<MathExpr> for Simple<MathExpr> {}
impl MathEquality<Term> for Simple<Term> {}
impl MathEquality<Factor> for Simple<Factor> {}
///tries to see if they are mathematically the same
trait PrivateMathEquality<T = Self>: Simplify<T> + Clone+ Into<MathExpr>
where
    T: Simplify<T> + Into<MathExpr>,
{
    ///The implementation part
    fn private_equals(&self, other: &Self, cont: &MathContext) -> bool {
        self.clone().into().simple(cont).is_ok_and(|s| {
            other
                .clone()
                .into()
                .simple(cont)
                .is_ok_and(|other| s.equals(&other, cont))
        })
    }
    ///This is garantied to be in the simplest form as long as the Simple
    /// Implementation is correct and that if they are not the same type so both
    /// are not Term OR MathExpr OR Factor
    fn equals(&self, other: &Self, cont: &MathContext) -> bool;
}

impl PrivateMathEquality<MathExpr> for Simple<MathExpr> {
    fn equals(&self, other: &Self, cont: &MathContext) -> bool {
        match (self.inner(), other.inner()) {
            (
                MathExpr::Term(Term::Factor(a)),
                MathExpr::Term(Term::Factor(b)),
            ) => a.equals(b, cont),
            (MathExpr::Term(a), MathExpr::Term(b)) => a.equals(b, cont),
            (a, b) => a.equals(b, cont),
        }
    }
}
impl PrivateMathEquality<Term> for Simple<Term> {
    fn equals(&self, other: &Self, cont: &MathContext) -> bool {
        match (self.inner(), other.inner()) {
            (Term::Factor(a), Term::Factor(b)) => a.equals(b, cont),
            (a, b) => a.equals(b, cont),
        }
    }
}
impl PrivateMathEquality<Factor> for Simple<Factor> {
    fn equals(&self, other: &Self, cont: &MathContext) -> bool {
        self.inner().equals(&other.0, cont)
    }
}

impl PrivateMathEquality for MathExpr {
    fn equals(&self, other: &MathExpr, cont: &MathContext) -> bool {
        match (self,other){
            (MathExpr::Term(_), MathExpr::Term(_)) => unreachable!("The Simple structs implementations should ensure that this is working"),
            (MathExpr::Add(lhs_1, rhs_1), MathExpr::Add(lhs_2, rhs_2)) =>
            (lhs_1.equals(lhs_2,cont)&&rhs_1.equals(rhs_2,cont))||
            (lhs_1.term().map_or(false,|f|f.equals(rhs_2,cont))&&
            lhs_2.term().map_or(false, |f|f.equals(rhs_1,cont))),
            (MathExpr::Subtract(lhs_1, rhs_1), MathExpr::Subtract(lhs_2, rhs_2)) =>
            (lhs_1.equals(lhs_2,cont)&&rhs_1.equals(rhs_2,cont))||
            (lhs_1.term().map_or(false,|f|f.equals(rhs_2,cont))&&
            lhs_2.term().map_or(false, |f|f.equals(rhs_1,cont))),
            _=>false
        }
    }
}
impl PrivateMathEquality for Term {
    fn equals(&self, other: &Term, cont: &MathContext) -> bool {
        match (self,other){
            (Term::Factor(_), Term::Factor(_)) => unreachable!("The Simple structs implementations should ensure that this is working"),
            (Term::Multiply(_, lhs_1, rhs_1), Term::Multiply(_, lhs_2, rhs_2)) => {
            if let(Term::Factor(lhs_1),Term::Factor(lhs_2))=(lhs_1.as_ref(),lhs_2.as_ref()){
                #[allow(unused_variables)]//remove this when we do matrix equality needs to be checked her because MulType
                return match (lhs_1,rhs_1,lhs_2,rhs_2){
                    (Factor::Matrix(lhs1),Factor::Matrix(rhs1),Factor::Matrix(lhs2),Factor::Matrix(rhs2))=>todo!(),
                    (Factor::Matrix(lhs1),rhs1,Factor::Matrix(lhs2),_)=>todo!(),
                    (Factor::Matrix(lhs1),rhs1,lhs2,Factor::Matrix(rhs2))=>todo!(),
                    (_,Factor::Matrix(rhs1),Factor::Matrix(lhs2),_)=>todo!(),
                    (_,Factor::Matrix(rhs1),_,Factor::Matrix(rhs2))=>todo!(),
                    _=>(lhs_1.equals(lhs_2,cont)&&rhs_1.equals(rhs_2,cont))||
            (lhs_1.equals(rhs_2,cont)&&
            lhs_2.equals(rhs_1,cont))
                }
            }
            (lhs_1.equals(lhs_2,cont)&&rhs_1.equals(rhs_2,cont))||
            (lhs_1.factor().map_or(false,|f|f.equals(rhs_2,cont))&&
            lhs_2.factor().map_or(false, |f|f.equals(rhs_1,cont)))
        }
            (Term::Divide(_, _), Term::Divide(_, _)) => todo!(),
            _=>false
        }
    }
}
impl PrivateMathEquality for Factor {
    fn equals(&self, other: &Factor, cont: &MathContext) -> bool {
        match (self, other) {
            (Factor::Constant(c_1), Factor::Constant(c_2)) => c_1.equals(c_2),
            (Factor::Parenthesis(p_1), Factor::Parenthesis(p_2)) => {
                p_1.equals(p_2, cont)
            }
            (Factor::Variable(v_1), Factor::Variable(v_2)) => v_1 == v_2,
            (Factor::FunctionCall(f_1), Factor::FunctionCall(f_2)) => {
                f_1 == f_2
            }
            (
                Factor::Power {
                    base: b_1,
                    exponent: e_1,
                },
                Factor::Power {
                    base: b_2,
                    exponent: e_2,
                },
            ) => b_1.equals(b_2, cont) && e_1.equals(e_2, cont),
            (
                Factor::Root {
                    degree: d_1,
                    radicand: r_1,
                },
                Factor::Root {
                    degree: d_2,
                    radicand: r_2,
                },
            ) => {
                let res = match (d_1, d_2) {
                    (None, None) => true,
                    (None, Some(d_2)) => {
                        d_2.clone().simple(cont).is_ok_and(|v| {
                            Into::<MathExpr>::into(Simple::constant(2.0))
                                .equals(&v, cont)
                        })
                    }
                    (Some(d_1), None) => {
                        d_1.clone().simple(cont).is_ok_and(|v| {
                            Into::<MathExpr>::into(Simple::constant(2.0))
                                .equals(&v, cont)
                        })
                    }
                    (Some(d_1), Some(d_2)) => {
                        d_1.clone().simple(cont).is_ok_and(|v_1| {
                            d_2.clone()
                                .simple(cont)
                                .is_ok_and(|v_2| v_1.equals(&v_2, cont))
                        })
                    }
                };
                res && r_1.equals(r_2, cont)
            }
            (Factor::Fraction(t_1, n_1), Factor::Fraction(t_2, n_2)) => {
                t_1.equals(t_2, cont) && n_1.equals(n_2, cont)
            }
            (Factor::Abs(a_1), Factor::Abs(a_2)) => a_1.equals(a_2, cont),
            (Factor::Matrix(_), Factor::Matrix(_)) => todo!(),
            _ => false,
        }
    }
}
