//!Factorization
use crate::prelude::*;

use super::helper::Simple;

///helper trait to make it factorize
pub trait Factorize {
    ///Simplifies as best as it can
    fn factorize(self) -> Simple;
}
impl Factorize for MathExpr {
    fn factorize(self) -> Simple {
        todo!()
    }
}
