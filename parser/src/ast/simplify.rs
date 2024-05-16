//! the implementations of simplification

use crate::prelude::*;

use super::{
    equality::PrivateMathEquality,
    helper::{NumberCompare, Simple, SimpleCompare},
};

impl Simplify for Simple {
    fn simple(self) -> Simple {
        self
    }
}
///helper trait to make it easier to change stuff
pub trait Simplify {
    ///Simplifies as best as it can
    fn simple(self) -> Simple;
}

impl Ast {
    ///simplifies the ast as best possible
    pub fn simplify(self) -> Self {
        match self {
            Ast::Expression(e) => e.simple().expression(),
            Ast::Equality(a, b) => (a.simple(), b.simple()).ast_equals(),
        }
    }
}

impl Simplify for MathExpr {
    ///Tries to simplify this
    fn simple(self) -> Simple {
        match self {
            MathExpr::Term(t) => t.simple(),
            MathExpr::Add(lhs, rhs) => {
                let simple = (lhs.simple(), rhs.simple());
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
                let simple = (lhs.simple(), rhs.simple());
                if simple.symmetrical() {
                    return Simple::constant(0.0);
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
        }
    }
}

impl Simplify for Term {
    fn simple(self) -> Simple {
        match self {
            Term::Factor(f) => f.simple(),
            Term::Multiply(m, lhs, rhs) => {
                let simple = (lhs.simple(), rhs.simple());
                match simple.to_math_expr() {
                    (
                        MathExpr::Term(Term::Factor(Factor::Constant(lhs))),
                        MathExpr::Term(Term::Factor(Factor::Constant(rhs))),
                    ) => Simple::mul(*lhs, *rhs),
                    (
                        _,
                        MathExpr::Term(Term::Factor(Factor::Constant(rhs))),
                    ) => {
                        if rhs.is_one() {
                            simple.0
                        } else if rhs.is_zero() {
                            Simple::constant(0.0)
                        } else {
                            simple.mul_wrapped(m.clone())
                        }
                    }
                    (
                        MathExpr::Term(Term::Factor(Factor::Parenthesis(p))),
                        rhs,
                    ) => {
                        todo!()
                    }
                    (
                        lhs,
                        MathExpr::Term(Term::Factor(Factor::Parenthesis(p))),
                    ) => {
                        todo!()
                    }
                    (
                        MathExpr::Term(Term::Factor(Factor::Constant(lhs))),
                        _,
                    ) => {
                        if lhs.is_one() {
                            simple.1
                        } else if lhs.is_zero() {
                            Simple::constant(0.0)
                        } else {
                            simple.mul_wrapped(m.clone())
                        }
                    }
                    _ => simple.mul_wrapped(m.clone()),
                }
            }
            Term::Divide(a, b) => {
                // TODO simplify fraction, aka factor a and b and cancel common
                // factors, remove fraction of b==1, etc.

                let simple = (a.simple(), b.simple());
                simplify_fraction_or_div(simple)
            }
        }
    }
}

impl Simplify for Factor {
    fn simple(self) -> Simple {
        match self {
            Factor::Constant(c) => Simple::constant(c),
            Factor::Parenthesis(p) => p.simple(),
            Factor::Variable(m) => Simple::variable(m),
            Factor::FunctionCall(f) => Simple::function(f),
            Factor::Power { base, exponent } => {
                let simple = (base.simple(), exponent.simple());
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
            } => todo!(),
            Factor::Fraction(a, b) => {
                // TODO simplify fraction, aka factor a and b and cancel common
                // factors, remove fraction of b==1, etc.

                let simple = (a.simple(), b.simple());
                simplify_fraction_or_div(simple)
            }
            Factor::Abs(_) => todo!(),
            Factor::Matrix(_) => todo!(),
        }
    }
}
fn simplify_fraction_or_div(simple: (Simple, Simple)) -> Simple {
    if simple.symmetrical() {
        Simple::constant(1.0)
    } else {
        simple.div_wrapped()
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
            .simplify();
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
}
