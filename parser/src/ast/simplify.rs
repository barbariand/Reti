//! the implementations of simplification

use crate::prelude::*;
use tracing::trace;

use super::{
    factorize::{FactorVec, Factorize},
    helper::{NumberCompare, Simple, SimpleCompare},
};

impl Simplify for Simple {
    fn simple(self, _: &MathContext) -> Result<Simple, EvalError> {
        Ok(self)
    }
}
///helper trait to make it easier to change stuff
pub trait Simplify {
    ///Simplifies as best as it can
    fn simple(self, cont: &MathContext) -> Result<Simple, EvalError>;
}

impl Ast {
    ///simplifies the ast as best possible
    pub fn simplify(self, cont: &MathContext) -> Result<Ast, EvalError> {
        Ok(match self {
            Ast::Expression(e) => e.simple(cont)?.expression(),
            Ast::Equality(a, b) => {
                (a.simple(cont)?, b.simple(cont)?).ast_equals()
            }
        })
    }
}

impl Simplify for MathExpr {
    ///Tries to simplify this
    fn simple(self, cont: &MathContext) -> Result<Simple, EvalError> {
        Ok(match self {
            MathExpr::Term(t) => return t.simple(cont),
            MathExpr::Add(lhs, rhs) => {
                let simple = (lhs.simple(cont)?, rhs.simple(cont)?);
                match simple.to_math_expr() {
                    (
                        MathExpr::Term(Term::Factor(Factor::Constant(a))),
                        MathExpr::Term(Term::Factor(Factor::Constant(b))),
                    ) => Simple::add(*a, *b),
                    (MathExpr::Term(Term::Factor(Factor::Constant(a))), _) => {
                        println!("first");
                        if a.is_zero() {
                            simple.1
                        } else {
                            simple.add_wrapped()
                        }
                    }
                    (_, MathExpr::Term(Term::Factor(Factor::Constant(b)))) => {
                        println!("second");
                        if b.is_zero() {
                            simple.0
                        } else {
                            simple.add_wrapped()
                        }
                    }
                    _ => simple.add_wrapped(),
                }
            }
            MathExpr::Subtract(lhs, rhs) => {
                let simple = (lhs.simple(cont)?, rhs.simple(cont)?);
                if simple.equivalent(cont) {
                    return Ok(Simple::constant(0.0));
                }
                match simple.to_math_expr() {
                    (
                        MathExpr::Term(Term::Factor(Factor::Constant(a))),
                        MathExpr::Term(Term::Factor(Factor::Constant(b))),
                    ) => Simple::sub(*a, *b),
                    (MathExpr::Term(Term::Factor(Factor::Constant(a))), _) => {
                        if a < &f64::EPSILON {
                            simple.0
                        } else {
                            simple.sub_wrapped()
                        }
                    }
                    (_, MathExpr::Term(Term::Factor(Factor::Constant(b)))) => {
                        if b < &f64::EPSILON {
                            simple.1
                        } else {
                            simple.sub_wrapped()
                        }
                    }
                    _ => simple.sub_wrapped(),
                }
            }
        })
    }
}

// Since Simple only works for MathExprs we can't implement it here.
impl FactorVec {
    /// Simplify each factor in this vector.
    ///
    /// Note that this method does not simplify the vec itself, but only maps
    /// each factor to simple.
    pub fn simplify_factors(
        self,
        cont: &MathContext,
    ) -> Result<FactorVec, EvalError> {
        Ok(FactorVec {
            vec: self
                .vec
                .into_iter()
                .map(|factor| {
                    factor
                        .simple(cont)
                        .map(|simple| simple.get_factor_or_wrap())
                })
                .collect::<Result<Vec<Factor>, EvalError>>()?,
        })
    }

    /// Simplify this vector of factors by merging constant terms, removing
    /// ones, etc.
    fn simple(self) -> FactorVec {
        // TODO this method only works for real numbers (or things that
        // commute). Matricies for example do not commute, so we cannot
        // move them around like this method does.
        if self.vec.is_empty() {
            panic!("Cannot simplify empty factors vector.");
        }
        trace!("simple, before: {:?}", self.vec);
        let mut result = Vec::with_capacity(self.vec.len());
        let mut constant_term = 1.0;
        for factor in self.vec {
            if let Factor::Constant(c) = factor {
                if c.is_one() {
                    // Skip terms that are 1 since multiplying by 1 is
                    // redundant.
                    continue;
                } else if c.is_zero() {
                    // If we have a zero anywhere, then all of the factors will
                    // be zero.
                    result = vec![Factor::Constant(0.0)];
                    break;
                }
                // Collect all constant terms into one term.
                constant_term *= c;
                continue;
            }
            // Push the rest of the factors.
            result.push(factor);
        }
        if !constant_term.is_one() || result.is_empty() {
            result.insert(0, Factor::Constant(constant_term));
        }
        trace!("simple, after: {:?}", result);
        FactorVec { vec: result }
    }
}

impl Term {
    /// Simplify the inner parts of the Term, but not the term itself.
    pub fn simple_inner(self, cont: &MathContext) -> Result<Simple, EvalError> {
        match self {
            Term::Factor(f) => f.simple(cont),
            Term::Multiply(m, lhs, rhs) => {
                Ok((lhs.simple_inner(cont)?, rhs.simple(cont)?).mul_wrapped(m))
            }
            Term::Divide(num, den) => {
                Ok((num.simple_inner(cont)?, den.simple(cont)?).div_wrapped())
            }
        }
    }
}

impl Simplify for Term {
    fn simple(self, cont: &MathContext) -> Result<Simple, EvalError> {
        let factors = self.simple_inner(cont)?.get_term_or_wrap().factorize();

        let factors_num = factors.factors_num.simplify_factors(cont)?.simple();

        let numerator = factors_num
            .to_term_ast()
            .expect("simplify_factors does not return empty factors");

        let mut factors_den = factors.factors_den;
        if factors_den.vec.len() == 1 {
            if let Factor::Constant(c) = factors_den.vec[0] {
                if c.is_one() {
                    // The denominator is 1, we don't need to express this as a
                    // fraction. Remove the denominator (make vec size 0).
                    factors_den.vec.remove(0);
                } else if c.is_zero() {
                    return Err(EvalError::DivideByZero);
                }
            }
        }
        if factors_den.vec.is_empty() {
            // No denominator means we don't need to express this as a
            // fraction.
            return Ok(Simple::new_unchecked(MathExpr::Term(numerator)));
        }

        let factors_den = factors_den.simplify_factors(cont)?.simple();

        if let Term::Factor(Factor::Constant(c)) = numerator {
            if c.is_zero() {
                // The numerator is 0, everything is zero.
                return Factor::Constant(0.0).simple(cont);
            }
        }

        let denominator = factors_den
            .to_term_ast()
            .expect("simplify_factors does not return empty factors");

        let denominator_factor = match denominator {
            Term::Factor(f) => f,
            _ => Factor::Parenthesis(MathExpr::Term(denominator).boxed()),
        };

        Ok(Simple::new_unchecked(MathExpr::Term(Term::Divide(
            numerator.boxed(),
            denominator_factor,
        ))))
    }
}

impl Simplify for Factor {
    fn simple(self, cont: &MathContext) -> Result<Simple, EvalError> {
        Ok(match self {
            Factor::Constant(c) => Simple::constant(c),
            Factor::Parenthesis(p) => p.simple(cont)?,
            Factor::Variable(m) => {
                return cont
                    .variables
                    .get(&m)
                    .map(|v| v.clone().simple(cont))
                    .unwrap_or(Ok(Simple::variable(m)))
            }
            Factor::FunctionCall(func_call) => {
                let func = cont
                    .functions
                    .get(&func_call.function_name)
                    .ok_or(EvalError::NotDefined)?;
                match func {
                    MathFunction::Native(_) => {
                        Simple::function(func_call.clone())
                    }
                    MathFunction::Foreign(f) => {
                        match func_call.arguments.len() == f.input.len() {
                            false => Err(EvalError::ArgumentLengthMismatch {
                                expected: vec![f.input.len()],
                                found: func_call.arguments.len(),
                            })?,
                            true => {
                                debug_assert!(
                                    func_call.arguments.len() == f.input.len()
                                );
                                let mut new_cont:MathContext=
                                        f.input.iter().zip(func_call.arguments.iter())
                                        .try_fold(MathContext::new(),
                                        |mut context:MathContext,(ident,expr)|
                                        Ok::<MathContext,EvalError>({
                                            context.variables.insert(ident.clone(), expr.clone());
                                            context}))?;
                                new_cont.merge(cont);
                                return f.expr.clone().simple(&new_cont);
                            }
                        }
                    }
                }
            }
            Factor::Power { base, exponent } => {
                let simple = (base.simple(cont)?, exponent.simple(cont)?);
                match simple.to_math_expr() {
                    (
                        MathExpr::Term(Term::Factor(Factor::Constant(base))),
                        MathExpr::Term(Term::Factor(Factor::Constant(
                            exponent,
                        ))),
                    ) => Simple::constant(base.powf(*exponent)),
                    (
                        _,
                        MathExpr::Term(Term::Factor(Factor::Constant(
                            exponent,
                        ))),
                    ) => {
                        if exponent.is_one() {
                            simple.0
                        } else if exponent.is_zero() {
                            Simple::constant(1.0)
                        } else {
                            simple.pow_wrapped()
                        }
                    }
                    _ => simple.pow_wrapped(),
                }
            }
            Factor::Root {
                degree: _,
                radicand: _,
            } => Simple::new_unchecked(MathExpr::Term(Term::Factor(self))), /* TODO */
            Factor::Fraction(numerator, denominator) => {
                simplify_fraction_or_div(
                    numerator.simple(cont)?,
                    denominator.simple(cont)?,
                    cont,
                )
            }
            Factor::Abs(_) => {
                Simple::new_unchecked(MathExpr::Term(Term::Factor(self)))
            } // TODO
            Factor::Matrix(m) => Simple::matrix(m, cont)?,
        })
    }
}
///Managing simplification off division and fraction
///
/// Because the division and factorization both use Simple objects it can be
/// ensured that

fn simplify_fraction_or_div(
    numerator: Simple,
    denominator: Simple,
    cont: &MathContext,
) -> Simple {
    // TODO simplify fraction, aka factor a and b and cancel common
    // factors, remove fraction of b==1, etc.
    let simple = (numerator, denominator);
    if simple.equivalent(cont) {
        Simple::constant(1.0)
    } else {
        match simple.to_math_expr() {
            (
                MathExpr::Term(Term::Factor(Factor::Constant(num))),
                MathExpr::Term(Term::Factor(Factor::Constant(den))),
            ) => Simple::divide(*num, *den),
            (_, MathExpr::Term(Term::Factor(Factor::Constant(c)))) => {
                if c.is_one() {
                    simple.0
                } else if c.is_zero() {
                    Simple::constant(f64::NAN)
                } else {
                    simple.div_wrapped()
                }
            }
            _ => simple.div_wrapped(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use pretty_assertions::assert_eq;
    async fn ast_test_simplify(text: &str, expected_latex: &str) {
        let context = MathContext::standard_math();
        let found_ast = parse(text, &context)
            .await
            .expect("failed to parse AST")
            .simplify(&context)
            .unwrap();
        let expected_ast = parse(expected_latex, &context)
            .await
            .expect("failed to parse latex to ast");
        // Compare and print with debug and formatting otherwise.
        assert_eq!(found_ast, expected_ast, "found/expected")
    }
    #[tokio::test]
    async fn one_minus_one() {
        ast_test_simplify("1-1", "0").await;
    }
    #[tokio::test]
    async fn one_plus_one() {
        ast_test_simplify("1+1", "2").await;
    }
    #[tokio::test]
    async fn one_times_one() {
        ast_test_simplify("1*1", "1").await;
    }
    #[tokio::test]
    async fn one_times_zero() {
        ast_test_simplify("1*0", "0").await;
    }
    #[tokio::test]
    async fn zero_times_parenthesis() {
        ast_test_simplify("0*(1+1+1+1+1*2)", "0").await;
    }
    #[tokio::test]
    async fn two_minus_one() {
        ast_test_simplify("2-1", "1").await;
    }
    #[tokio::test]
    async fn two_x_minus_two_x() {
        ast_test_simplify("2x-2x", "0").await;
    }
    #[tokio::test]
    async fn test() {
        ast_test_simplify("2x^{2-1}1+\\ln(2)x^{2}0", "2x").await;
    }
    #[tokio::test]
    async fn multiply_remove_parenthesis() {
        ast_test_simplify("2(2x)", "4x").await;
    }
    #[tokio::test]
    async fn multiply_remove_parenthesis_2() {
        ast_test_simplify("3(2x)+2", "6x+2").await;
    }
}
