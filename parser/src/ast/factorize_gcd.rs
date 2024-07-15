//!Factorization
use crate::prelude::*;

use super::helper::Simple;

///helper trait to make it factorize
pub trait FactorizeGcd {
    ///Extracts factors as best it can
    fn factorize_gcd(self) -> Simple<MathExpr>;
}
impl FactorizeGcd for MathExpr {
    fn factorize_gcd(self) -> Simple<MathExpr> {
        todo!()
    }
}
