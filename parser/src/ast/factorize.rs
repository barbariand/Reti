//!Factorization
use crate::prelude::*;
use std::fmt::Debug;
use tracing::trace;

use super::{
    helper::{Simple, SimpleCompareTerm},
    simplify::Simplify,
};

/// A vector of factors. They are usually intended to be multiplied together.
#[derive(Debug, Default)]
pub struct FactorVec {
    /// The vector that holds the factors.
    pub vec: Vec<Factor>,
}

impl FactorVec {
    /// Convert this array of factors into an AST in the form of a [Term].
    ///
    /// Returns `None` if the vector is empty.
    pub fn to_term_ast(
        self,
        cont: &MathContext,
    ) -> Option<Result<Simple<Term>, EvalError>> {
        if self.vec.is_empty() {
            None
        } else if self.vec.len() == 2 {
            let mut vec = self.vec;
            let left = Into::<Term>::into(vec.remove(0)).simple(cont);
            let right = vec.remove(0).simple(cont);
            let simples: (Simple<Term>, Simple<Factor>) = (
                match left {
                    Ok(v) => v,
                    e => return Some(e),
                },
                match right {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                },
            );
            Some(Ok(simples.mul_wrapped(MulType::Implicit)))
        } else {
            let mut iter = self.vec.into_iter();
            let mut term = Term::Factor(iter.next().expect("Not empty."));
            for next in iter {
                term = Term::Multiply(MulType::Implicit, term.boxed(), next);
            }
            Some(Ok(Simple(term)))
        }
    }
}

/// The result of factorization that holds the factors for the numerator and
/// denominator.
#[derive(Debug, Default)]
pub struct FactorizationResult {
    /// The factors in the numerator (top part of a division).
    pub factors_num: FactorVec,
    /// The factors in the denominator (the bottom part of the division).
    pub factors_den: FactorVec,
}

/// A trait added to things that can be factorized into vectors of factors.
///
/// Note that the current implementation makes no effort to factorize, and will
/// simply return all factors in a vector instead of being represented in an
/// AST. So "x^2-4" will not be factorized using the difference of squares.
/// Integers will not be factorized into prime numbers but will be left as-is.
pub trait Factorize {
    /// Get the factors as a Vec.
    /// See [Factorize].
    fn factorize(self) -> FactorizationResult;
}

impl<T: FactorizeCollecting + Debug + Clone> Factorize for T {
    fn factorize(self) -> FactorizationResult {
        trace!("Factorizing {:#?}", self);
        let mut result = FactorizationResult::default();
        self.clone().factorize_collecting(&mut result); // TODO this is debug, remove clone.
        assert!(
            !result.factors_num.vec.is_empty(),
            "Empty numerator when factorizing {:?}",
            self
        );

        trace!("Resulted in {:#?}", result);
        result
    }
}

/// Factorize in a way that collects factors into a mutable
/// [FactorizationResult].
///
/// This can be seen as an internal method used to implement factorization.
trait FactorizeCollecting {
    /// Factorize and put factors in the provided [FactorizationResult].
    fn factorize_collecting(self, result: &mut FactorizationResult);
}

impl FactorizeCollecting for Factor {
    fn factorize_collecting(self, result: &mut FactorizationResult) {
        // If a parenthesis with only one term we can continue factorizing the
        // term.
        if let Factor::Parenthesis(ref p) = self {
            if let MathExpr::Term(t) = *p.clone() {
                return t.factorize_collecting(result);
            }
        }
        // Otherwise we have one factor, self, to add to the numerators.
        result.factors_num.vec.push(self);
    }
}

impl FactorizeCollecting for Term {
    fn factorize_collecting(self, result: &mut FactorizationResult) {
        match self {
            Term::Factor(f) => f.factorize_collecting(result),
            Term::Multiply(_m, lhs, rhs) => {
                lhs.factorize_collecting(result);
                rhs.factorize_collecting(result);
            }
            Term::Divide(num, den) => {
                num.factorize_collecting(result);

                // For the denominator we cannot collect because the
                // factorize_collecting method does not know that we "are" in
                // the denominator. We therefore factorize the denominator
                // separately and add the factors but in swapped vectors.
                // This is because:
                //  a     c     a     d
                // --- / --- = --- * ---
                //  b     d     b     c
                let mut den_res = den.factorize();
                result.factors_num.vec.append(&mut den_res.factors_den.vec);
                result.factors_den.vec.append(&mut den_res.factors_num.vec);
            }
        }
    }
}
