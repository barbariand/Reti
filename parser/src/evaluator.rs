//!A robust single threaded evaluator for [Ast]
use std::fmt::Display;

use crate::{
    ast::{helper::Simple, simplify::Simplify, to_latex::ToLaTeX},
    prelude::*,
};

///Evaluations for ast, delegates it to aprox and other
pub struct Evaluator(MathContext);
impl Default for Evaluator {
    fn default() -> Self {
        Self(MathContext::standard_math())
    }
}

impl Evaluator {
    ///creates a new Evaluator with a empty [MathContext]
    pub fn new_empty() -> Self {
        Self(MathContext::new())
    }
    /// from math_context
    pub fn with_context(context: MathContext) -> Self {
        Self(context)
    }
    ///Get the context to it
    pub const fn context(&self) -> &MathContext {
        &self.0
    }
    ///Creates a new with [MathContext::standard_math]
    pub fn standard_math() -> Self {
        Self(MathContext::standard_math())
    }
    ///Evaluates the ast and inserts variables as needed
    pub fn eval_ast(
        &mut self,
        ast: Simple<Ast>,
    ) -> Result<Evaluation, EvalError> {
        Ok(match ast.value {
            Ast::Expression(expr) => {
                Evaluation::LaTeX(expr.simple(self.context())?.to_latex())
            }
            Ast::Equality(lhs, rhs) => self.eval_equality(lhs, rhs)?,
        })
    }
    /// Evaluate [Ast::Equality]
    fn eval_equality(
        &mut self,
        lhs: MathExpr,
        rhs: MathExpr,
    ) -> Result<Evaluation, EvalError> {
        if let MathExpr::Term(Term::Multiply(MulType::Implicit, var, factor)) =
            lhs
        {
            if let Term::Factor(Factor::Variable(function_name)) = &*var {
                match factor {
                    Factor::Parenthesis(f) => {
                        if let MathExpr::Term(Term::Factor(Factor::Variable(
                            arg,
                        ))) = *f
                        {
                            let variable_name = arg.clone();
                            let rhs_simple = rhs.simple(self.context())?;
                            let res = Evaluation::AddedFunction(
                                rhs_simple.to_latex(),
                            );
                            self.0.add_function(
                                function_name.clone(),
                                MathFunction::new_foreign(
                                    rhs_simple,
                                    vec![variable_name],
                                ),
                            );
                            Ok(res)
                        } else {
                            todo!("you cant have anything but a variable in a function definition")
                        }
                    }
                    Factor::Matrix(matrix) => {
                        if matrix.is_vector() {
                            let vec = matrix.get_all_vector_elements();
                            let args: Option<Vec<MathIdentifier>> = vec
                                .iter()
                                .cloned()
                                .map(|v| match v {
                                    MathExpr::Term(Term::Factor(
                                        Factor::Variable(var),
                                    )) => Some(var),
                                    _ => None,
                                })
                                .collect();
                            let rhs_simple = rhs.simple(self.context())?;
                            let res = Evaluation::AddedFunction(
                                rhs_simple.to_latex(),
                            );
                            self.0.add_function(
                                function_name.clone(),
                                MathFunction::new_foreign(
                                    rhs_simple,
                                    args.expect(
                                        "The values uses was not identifiers only",
                                    ),
                                ),
                            );
                            Ok(res)
                        } else {
                            todo!("Could not understand equals. is it a 2d matrix as input?")
                        }
                    }
                    e => {
                        todo!(
                            "Could not understand equals: got factor:{:#?}",
                            e
                        )
                    }
                }
            } else {
                todo!(
                    "Could not understand equals. got:{:#?}",
                    MathExpr::Term(Term::Multiply(
                        MulType::Implicit,
                        var,
                        factor,
                    ))
                );
            }
        } else if let MathExpr::Term(Term::Factor(Factor::Variable(ident))) =
            lhs
        {
            let rhs_simple = rhs.simple(self.context())?;
            let res = Evaluation::AddedVariable(rhs_simple.to_latex());
            self.0.variables.insert(ident, rhs_simple);
            return Ok(res);
        } else {
            todo!("Could not understand equals. got:{:#?}", lhs);
        }
    }
}
///The response for the Approximator
#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub enum Evaluation {
    ///Added a function to the context
    AddedFunction(String),
    ///Added a variable to the context
    AddedVariable(String),
    ///Got a value from it
    LaTeX(String),
}
impl Display for Evaluation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Evaluation::AddedFunction(s) => write!(f, "Added Function: {}", s),
            Evaluation::AddedVariable(s) => write!(f, "Added variable: {}", s),
            Evaluation::LaTeX(s) => write!(f, "{}", s),
        }
    }
}
impl From<&str> for Evaluation {
    fn from(value: &str) -> Self {
        Evaluation::LaTeX(value.to_owned())
    }
}
impl From<f64> for Evaluation {
    fn from(value: f64) -> Self {
        Evaluation::LaTeX(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::Evaluator;
    use crate::{ast::simplify::Simplify, prelude::*};

    fn eval_test_from_ast(expected: impl Into<Evaluation>, ast: Ast) {
        let mut evaluator = Evaluator::new_empty();

        let value = match evaluator
            .eval_ast(ast.simple(evaluator.context()).unwrap())
        {
            Ok(val) => val,
            Err(err) => panic!("{err:?}"),
        };

        assert_eq!(expected.into(), value);
    }

    async fn eval_test_from_str(expected: impl Into<Evaluation>, text: &str) {
        let ast = parse(text, &MathContext::new()).await.unwrap();

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
}
