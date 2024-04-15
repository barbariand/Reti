/// A simple single-threaded evaluator for an AST.
use crate::prelude::*;

pub struct Approximator {
    context: MathContext,
}

impl Approximator {
    pub fn new(context: MathContext) -> Self {
        Self { context }
    }

    pub fn context(&self) -> &MathContext {
        &self.context
    }
    pub fn context_mut(&mut self) -> &mut MathContext {
        &mut self.context
    }

    pub fn eval_expr(&self, expr: &MathExpr) -> f64 {
        match expr {
            MathExpr::Term(term) => self.eval_term(term),
            MathExpr::Add(a, b) => self.eval_expr(a.as_ref()) + self.eval_term(b),
            MathExpr::Subtract(a, b) => self.eval_expr(a.as_ref()) - self.eval_term(b),
        }
    }

    fn eval_term(&self, term: &Term) -> f64 {
        match term {
            Term::Factor(factor) => self.eval_factor(factor),
            Term::Multiply(a, b) => self.eval_term(a.as_ref()) * self.eval_factor(b),
            Term::Divide(a, b) => self.eval_term(a.as_ref()) / self.eval_factor(b),
        }
    }

    fn eval_factor(&self, factor: &Factor) -> f64 {
        match factor {
            Factor::Constant(c) => *c,
            Factor::Parenthesis(expr) => self.eval_expr(expr.as_ref()),
            Factor::Variable(x) => {
                match self.context.variables.get(x) {
                    Some(val) => *val,
                    None => panic!(), // TODO return error here instead of panic
                }
            }
            Factor::FunctionCall(call) => match self.context.functions.get(&call.function_name) {
                Some(func) => {
                    let args = call
                        .arguments
                        .iter()
                        .map(|expr| self.eval_expr(expr))
                        .collect();
                    (func.approximate)(args)
                }
                None => panic!("Parser incorrectly identified function {:?}", call),
            },
            Factor::Power { base, exponent } => self
                .eval_factor(base.as_ref())
                .powf(self.eval_expr(exponent.as_ref())),
            Factor::Root { degree, radicand } => {
                match degree.as_ref().map(|expr| self.eval_expr(expr.as_ref())) {
                    None => self.eval_expr(radicand.as_ref()).sqrt(),
                    Some(degree) => self.eval_expr(radicand.as_ref()).powf(1.0 / degree),
                }
            }
            Factor::Fraction(a, b) => self.eval_expr(a.as_ref()) / self.eval_expr(b.as_ref()),
            Factor::Abs(val) => self.eval_expr(val.as_ref()).abs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{string::ParseError, sync::Arc};

    use tokio::{
        join,
        sync::mpsc::{self, Receiver, Sender},
    };

    use crate::prelude::*;

    fn eval_test_from_ast(expected: f64, ast: Ast) {
        let context = MathContext::new();
        let approximator = Approximator::new(context);

        let found = match ast {
            Ast::Expression(expr) => approximator.eval_expr(&expr),
            Ast::Equality(_, _) => panic!("Cannot evaluate statement."),
        };

        if (found - expected).abs() > f64::EPSILON {
            panic!("Found {} expected {}", found, expected);
        }
    }

    async fn eval_test_from_str(expected: f64, text: &str) {
        let (tx, rx): (Sender<Token>, Receiver<Token>) = mpsc::channel(32); // idk what that 32 means tbh

        let context = MathContext::new();
        let lexer = Lexer::new(tx);

        let mut parser = Parser::new(rx, context);

        let future1 = lexer.tokenize(text);
        let future2 = parser.parse();

        let ((), ast) = join!(future1, future2);
        let ast = ast.unwrap();

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
                Term::Multiply(Box::new(3.0.into()), 5.0.into()),
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
}
