#![allow(dead_code)]
//! Simple single-threaded Approximator for AST

use crate::ast::{helper::Simple, simplify::Simplify};

use super::prelude::*;

/// A simple single-threaded evaluator for an AST.
pub(crate) struct Approximator<'a> {
    /// the MathContext holding all the defined functions
    context: &'a MathContext,
}

impl<'a> Approximator<'a> {
    /// Makes a new Approximator
    pub const fn new(context: &'a MathContext) -> Self {
        Self { context }
    }
    ///Evaluates a MathExpr
    ///
    /// # Errors
    /// [EvalError]
    /// This can error if it can not be completed or it is wrong
    pub fn approximate_expr(
        &self,
        expr: Simple<MathExpr>,
    ) -> Result<Value, EvalError> {
        match expr.inner() {
            MathExpr::Term(term) => {
                self.approximate_term(term.simple(self.context)?)
            }
            MathExpr::Add(a, b) => {
                self.approximate_expr(a.simple(self.context)?)?
                    + self.approximate_term(b.simple(self.context)?)?
            }
            MathExpr::Subtract(a, b) => {
                self.approximate_expr(a.simple(self.context)?)?
                    - self.approximate_term(b.simple(self.context)?)?
            }
        }
    }
    ///Evaluates a Term
    ///
    /// # Errors
    /// [EvalError]
    /// This can error if it can not complete
    fn approximate_term(&self, term: Simple<Term>) -> Result<Value, EvalError> {
        match term.inner() {
            Term::Factor(factor) => {
                self.approximate_factor(factor.simple(self.context)?)
            }
            Term::Multiply(mul_type, a, b) => {
                self.approximate_term(a.simple(self.context)?)?.mul(
                    &mul_type,
                    &self.approximate_factor(b.simple(self.context)?)?,
                )
            }
            Term::Divide(a, b) => {
                self.approximate_term(a.simple(self.context)?)?
                    / self.approximate_factor(b.simple(self.context)?)?
            }
        }
    }
    ///Evaluates a Factor
    ///
    /// # Errors
    /// [EvalError]
    /// This can error if it can not complete
    ///
    /// # Panics
    ///  this implementation currently panics when it can not under
    fn approximate_factor(
        &self,
        factor: Simple<Factor>,
    ) -> Result<Value, EvalError> {
        Ok(match factor.inner() {
            Factor::Constant(c) => Value::Scalar(c),
            Factor::Parenthesis(expr) => {
                self.approximate_expr(expr.simple(self.context)?)?
            }

            Factor::Variable(x) => self.approximate_expr(
                self.context
                    .variables
                    .get(&x)
                    .map(|v| v.clone().simple(self.context))
                    .ok_or(EvalError::NotDefined)??,
            )?,
            Factor::FunctionCall(func_call) => {
                match self.context.functions.get(&func_call.function_name) {
                    Some(func) => match func {
                        MathFunction::Native(n) => {
                            let args: Result<Vec<Value>, EvalError> = func_call
                                .arguments
                                .iter()
                                .map(|expr| {
                                    self.approximate_expr(
                                        expr.clone().simple(self.context)?,
                                    )
                                })
                                .collect();
                            n.run(args?)?
                        }
                        MathFunction::Foreign(_) => {
                            unreachable!("this should never be reatched, as the function should be inlined already")
                        }
                    },
                    None => unreachable!(
                        "Parser incorrectly identified function {:?}",
                        func_call
                    ),
                }
            }
            Factor::Power { base, exponent } => {
                let base_val = self
                    .approximate_factor(base.simple(self.context)?)?
                    .scalar()?;
                let exp_val = self
                    .approximate_expr(exponent.simple(self.context)?)?
                    .scalar()?;
                Value::Scalar(base_val.powf(exp_val))
            }
            Factor::Root { degree, radicand } => Value::Scalar(
                match degree.as_ref().map(|expr| {
                    self.approximate_expr(expr.clone().simple(self.context)?)
                }) {
                    None => self
                        .approximate_expr(radicand.simple(self.context)?)?
                        .scalar()?
                        .sqrt(),
                    Some(degree) => {
                        let radicand_val = self
                            .approximate_expr(radicand.simple(self.context)?)?
                            .scalar()?;
                        let degree_val = degree?.scalar()?;
                        radicand_val.powf(1.0 / degree_val)
                    }
                },
            ),
            Factor::Fraction(a, b) => {
                let a_val = self.approximate_expr(a.simple(self.context)?)?;
                let b_val = self.approximate_expr(b.simple(self.context)?)?;
                (a_val / b_val)?
            }
            Factor::Abs(val) => Value::Scalar(
                self.approximate_expr(val.simple(self.context)?)?
                    .scalar()?
                    .abs(),
            ),
            Factor::Matrix(matrix) => Value::Matrix(matrix.map(|expr| {
                self.approximate_expr(expr.clone().simple(self.context)?)
            })?),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Approximator;
    use crate::{
        ast::{helper::NumberCompare, simplify::Simplify},
        prelude::*,
    };
    use tokio::{
        join,
        sync::mpsc::{self},
    };

    fn eval_test_from_ast(expected: f64, ast: Ast) {
        let context = MathContext::new();
        let approximator = Approximator::new(&context);

        let expr = match ast {
            Ast::Expression(expr) => expr,
            Ast::Equality(_, _) => unimplemented!("Cannot evaluate statement."),
        };

        let value = approximator
            .approximate_expr(
                expr.simple(approximator.context)
                    .expect("Test should not fail"),
            )
            .expect("Test should not fail");

        let found = match value {
            Value::Scalar(val) => val,
            Value::Matrix(m) => unimplemented!("Unexpected matrix {m:?}"),
        };

        assert!(found.equals(&expected));
    }

    async fn eval_test_from_str(expected: f64, text: &str) {
        let (tx, rx): (TokenSender, TokenReceiver) = mpsc::channel(32);

        let context = MathContext::new();
        let lexer = Lexer::new(tx);

        let parser = Parser::new(rx, context);

        let future1 = lexer.tokenize(text);
        let future2 = parser.parse();

        let ((), ast) = join!(future1, future2);
        let ast = ast.expect("Failed to create ast in test");

        eval_test_from_ast(expected, ast);
    }

    #[test]
    fn eval_1_plus_1() {
        eval_test_from_ast(
            2.0,
            Ast::Expression(MathExpr::Add(Box::new(1.0.into()), 1.0.into())),
        );
    }

    #[test]
    fn eval_multiplication() {
        eval_test_from_ast(
            17.0,
            // 2+3*5
            Ast::Expression(MathExpr::Add(
                Box::new(2.0.into()),
                Term::Multiply(
                    MulType::Asterisk,
                    Box::new(3.0.into()),
                    5.0.into(),
                ),
            )),
        );
    }

    #[tokio::test]
    async fn parenthesis_and_exponent() {
        eval_test_from_str(54.0, "2(3)^3").await;
    }

    #[tokio::test]
    async fn fraction_sqrt_cube_root() {
        eval_test_from_str(
            3.0,
            "\\frac{2( 1+1)^{3} +5}{\\sqrt{\\frac{49}{3}\\sqrt[3]{27}}}",
        )
        .await;
    }
    #[tokio::test]
    async fn markdown_example() {
        eval_test_from_str(0.5, "\\frac{2\\sqrt{9}+5}{3(3+4)+1}").await;
    }
}
