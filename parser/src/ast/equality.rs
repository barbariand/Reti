//!Tries to find out if they are the same
use crate::prelude::*;

use super::simplify::Simplify;
///tries to see if they are mathematically the same
pub trait MathEquality{
    ///the user part of the trait
    fn eq(&self,other:impl Into<MathExpr>)->bool{
        let other_new:MathExpr=other.into();
        self.equals(&other_new)
    }
    ///The implementation part
    fn equals(&self,other:&MathExpr)->bool;
}
impl MathEquality for Ast{
    fn equals(&self,other:&MathExpr)->bool {
        match self{
            Ast::Expression(e) => e.equals(other),
            Ast::Equality(_, _) => false,
        }
    }
}
impl MathEquality for MathExpr{
    fn equals(&self,other:&MathExpr)->bool {
        match (self.simplify(),other.simplify()){
            (MathExpr::Term(_), MathExpr::Term(_)) => todo!(),
            (MathExpr::Add(_, _), MathExpr::Add(_, _)) => todo!(),
            (MathExpr::Subtract(_, _), MathExpr::Subtract(_, _)) => todo!(),
            _=>false
        }
    }
}
impl MathEquality for Term{
    fn equals(&self,other:&MathExpr)->bool {
        match (self.simplify(),other.simplify()){
            (MathExpr::Term(a), MathExpr::Term(b)) => todo!(),
            (a,b)=>a.equals(&b)
        }
    }
}
impl MathEquality for Factor{
    fn equals(&self,other:&MathExpr)->bool {
        match (self.simplify(),other.simplify()){
            (MathExpr::Term(Term::Factor(a)),MathExpr::Term(Term::Factor(b)))=>todo!(),
            (a,b)=>a.equals(&b)
        }
    }
}