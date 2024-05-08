//! Simple single-threaded Approximator for AST
use super::prelude::*;

/// The errors that can happen when evaluating a AST
#[derive(Debug)]
pub enum EvalError {
    /// A matrix was found when it was expected to be a scalar
    ExpectedScalar,
    /// General error for when it can not be used
    IncompatibleTypes(&'static str),
    ///When the Matrix required another size for doing the operation
    IncompatibleMatrixSizes(IncompatibleMatrixSizes),
    /// Could nto fund the value expected
    NotDefined,
    ///Invalid amount of arguments
    ArgumentLenghtMissmatch { expected: Vec<usize>, found: usize },
}

/// The error for when it required another size of the matrix
#[derive(Debug)]
pub enum IncompatibleMatrixSizes {
    /// Wrong row value for the matrix operation
    Row {
        /// The expected value for the matrix
        expected: usize,
        /// The value found
        found: usize,
    },
    /// Wrong column value for the matrix operation
    Column {
        /// The expected value for the matrix
        expected: usize,
        /// The value found
        found: usize,
    },
}

impl From<IncompatibleMatrixSizes> for EvalError {
    fn from(value: IncompatibleMatrixSizes) -> Self {
        Self::IncompatibleMatrixSizes(value)
    }
}
/// A simple single-threaded evaluator for an AST.
pub struct Approximator {
    /// the MathContext holding all the defined functions
    context: MathContext,
}

impl Approximator {
    /// Makes a new Approximator
    pub fn new(context: MathContext) -> Self {
        Self { context }
    }
    ///returns a reference to the [MathContext] used for evaluating functions
    pub fn context(&self) -> &MathContext {
        &self.context
    }
    ///returns a mutable reference to the [MathContext] used for evaluating
    /// functions
    pub fn context_mut(&mut self) -> &mut MathContext {
        &mut self.context
    }
    ///Evaluates a MathExpr
    ///
    /// # Errors
    /// [EvalError]
    /// This can error if it can not be completed or it is wrong
    pub fn eval_expr(&self, expr: &MathExpr) -> Result<Value, EvalError> {
        match expr {
            MathExpr::Term(term) => self.eval_term(term),
            MathExpr::Add(a, b) => {
                self.eval_expr(a.as_ref())? + self.eval_term(b)?
            }
            MathExpr::Subtract(a, b) => {
                self.eval_expr(a.as_ref())? - self.eval_term(b)?
            }
        }
    }
    ///Evaluates a Term
    ///
    /// # Errors
    /// [EvalError]
    /// This can error if it can not complete
    fn eval_term(&self, term: &Term) -> Result<Value, EvalError> {
        match term {
            Term::Factor(factor) => self.eval_factor(factor),
            Term::Multiply(a, b) => {
                self.eval_term(a.as_ref())? * self.eval_factor(b)?
            }
            Term::Divide(a, b) => {
                self.eval_term(a.as_ref())? / self.eval_factor(b)?
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
    fn eval_factor(&self, factor: &Factor) -> Result<Value, EvalError> {
        Ok(match factor {
            Factor::Constant(c) => Value::Scalar(*c),
            Factor::Parenthesis(expr) => self.eval_expr(expr.as_ref())?,
            Factor::Variable(x) => self
                .context
                .variables
                .get(x)
                .ok_or(EvalError::NotDefined)?
                .clone(),
            Factor::FunctionCall(call) => {
                match self.context.functions.get(&call.function_name) {
                    Some(func) => {
                        let args: Result<Vec<Value>, EvalError> = call
                            .arguments
                            .iter()
                            .map(|expr| self.eval_expr(expr))
                            .collect();
                        func.eval(args?, self.context.clone())?
                    }
                    None => panic!(
                        "Parser incorrectly identified function {:?}",
                        call
                    ),
                }
            }
            Factor::Power { base, exponent } => {
                let base_val = self.eval_factor(base.as_ref())?.scalar()?;
                let exp_val = self.eval_expr(exponent.as_ref())?.scalar()?;
                Value::Scalar(base_val.powf(exp_val))
            }
            Factor::Root { degree, radicand } => Value::Scalar(
                match degree.as_ref().map(|expr| self.eval_expr(expr.as_ref()))
                {
                    None => self.eval_expr(radicand.as_ref())?.scalar()?.sqrt(),
                    Some(degree) => {
                        let radicand_val =
                            self.eval_expr(radicand.as_ref())?.scalar()?;
                        let degree_val = degree?.scalar()?;
                        radicand_val.powf(1.0 / degree_val)
                    }
                },
            ),
            Factor::Fraction(a, b) => {
                let a_val = self.eval_expr(a.as_ref())?;
                let b_val = self.eval_expr(b.as_ref())?;
                (a_val / b_val)?
            }
            Factor::Abs(val) => {
                Value::Scalar(self.eval_expr(val.as_ref())?.scalar()?.abs())
            }
            Factor::Matrix(matrix) => {
                Value::Matrix(matrix.map(|expr| self.eval_expr(expr))?)
            }
        })
    }
}

#[cfg(test)]
mod tests {

    use tokio::{
        join,
        sync::mpsc::{self},
    };

    use crate::prelude::*;

    fn eval_test_from_ast(expected: f64, ast: Ast) {
        let context = MathContext::new();
        let approximator = Approximator::new(context);

        let expr = match ast {
            Ast::Expression(expr) => expr,
            Ast::Equality(_, _) => panic!("Cannot evaluate statement."),
        };

        let value = match approximator.eval_expr(&expr) {
            Ok(val) => val,
            Err(err) => panic!("{err:?}"),
        };

        let found = match value {
            Value::Scalar(val) => val,
            Value::Matrix(m) => panic!("Unexpected matrix {m:?}"),
        };

        if (found - expected).abs() > f64::EPSILON {
            panic!("Found {} expected {}", found, expected);
        }
    }

    async fn eval_test_from_str(expected: f64, text: &str) {
        let (tx, rx): (TokenSender, TokenReceiver) = mpsc::channel(32);

        let context = MathContext::new();
        let lexer = Lexer::new(tx);

        let parser = Parser::new(rx, context);

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
