//!Tries to find out if they are the same
use crate::prelude::*;
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
        todo!()
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
        todo!()
    }
}
impl MathEquality for Factor{
    fn equals(&self,other:&MathExpr)->bool {
        todo!()
    }
}