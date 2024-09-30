//! # Reti parser common exports
//! Reti parser common exports for interacting with the latex parser
//! * [parse] function
//! * outputted [Ast]
//!
//! [parse]: self::parse

pub use crate::{
    ast::Ast,
    context::MathContext,
    error::{
        AstError, DeriveError, EvalError, IncompatibleMatrixSizes, ParseError,
    },
    evaluator::Evaluation,
    evaluator::Evaluator,
    functions::MathFunction,
    value::Value,
};

pub(crate) use crate::{
    ast::{Factor, FunctionCall, MathExpr, MulType, Term},
    functions::IntoMathFunction,
    identifier::MathIdentifier,
    lexer::Lexer,
    matrix::Matrix,
    normalizer::Normalizer,
    parsing::Parser,
    token::Token,
    token_reader::TokenReader,
};

use tracing::{debug, error, trace_span};
///The parse function central to the parsing functionality, and outputs an AST
/// that can be evaluated using
pub fn parse(text: &str, context: &MathContext) -> Result<Ast, AstError> {
    let span = trace_span!("parsing");
    let _enter = span.enter();
    debug!(text);
    let lexer = Lexer::new(text.chars());
    let normalizer = Normalizer::new(lexer);
    let parser = Parser::new(normalizer, context.clone());

    match parser.parse() {
        Err(e) => {
            error!("parser task failed");
            Err(e.into())
        }
        Ok(ast) => return Ok(ast),
    }
}
///starting a task that has a certain output and returning the JoinHandle
/// making sure to catch panics as results
#[cfg(test)]
mod tests {
    use crate::{
        identifier::{GreekLetter, OtherSymbol},
        prelude::*,
    };
    use pretty_assertions::assert_eq;
    use tracing_test::traced_test;
    fn parse_test(text: &str, expected_ast: Ast) {
        let found_ast = parse(text, &MathContext::standard_math()).unwrap();
        // Compare and print with debug and formatting otherwise.
        assert_eq!(found_ast, expected_ast)
    }

    #[traced_test]
    #[test]
    fn constant() {
        parse_test("1", Ast::Expression(1f64.into()));
    }

    #[traced_test]
    #[test]
    fn addition() {
        parse_test(
            "1+2+3",
            Ast::Expression(MathExpr::Add(
                Box::new(MathExpr::Add(Box::new(1f64.into()), 2f64.into())),
                3f64.into(),
            )),
        );
    }

    #[traced_test]
    #[test]
    fn addition_multiplication_order_of_operations() {
        parse_test(
            "1+2+3+(4+5)*6",
            Ast::Expression(MathExpr::Add(
                Box::new(MathExpr::Add(
                    Box::new(MathExpr::Add(Box::new(1f64.into()), 2f64.into())),
                    3f64.into(),
                )),
                Term::Multiply(
                    MulType::Asterisk,
                    Box::new(Term::Factor(Factor::Parenthesis(Box::new(
                        MathExpr::Add(Box::new(4f64.into()), 5f64.into()),
                    )))),
                    6f64.into(),
                ),
            )),
        );
    }
    #[traced_test]
    #[test]
    fn sqrt() {
        parse_test(
            "\\sqrt{9}",
            Ast::Expression(
                Factor::Root {
                    degree: None,
                    radicand: 9f64.into(),
                }
                .into(),
            ),
        );
    }

    #[traced_test]
    #[test]
    fn cube_root() {
        parse_test(
            "\\sqrt[3]{27}",
            Ast::Expression(
                Factor::Root {
                    degree: Some(3f64.into()),
                    radicand: 27f64.into(),
                }
                .into(),
            ),
        );
    }

    #[traced_test]
    #[test]
    fn exponent() {
        parse_test(
            "2^{3}",
            Ast::Expression(
                Factor::Power {
                    base: Box::new(Factor::Constant(2.0.into())),
                    exponent: Box::new(MathExpr::Term(Term::Factor(
                        Factor::Constant(3.0.into()),
                    ))),
                }
                .into(),
            ),
        );
    }

    #[traced_test]
    #[test]
    fn exponent_command() {
        parse_test(
            "2^\\pi",
            Ast::Expression(
                Factor::Power {
                    base: Box::new(2f64.into()),
                    exponent: Box::new(
                        Factor::Variable(MathIdentifier::from_single_greek(
                            GreekLetter::LowercasePi,
                        ))
                        .into(),
                    ),
                }
                .into(),
            ),
        );
    }

    #[traced_test]
    #[test]
    fn exponent_split_token() {
        parse_test(
            "2^025", // this is 2^0 * 25
            Ast::Expression(MathExpr::Term(Term::Multiply(
                MulType::Implicit,
                //2^0
                Box::new(Term::Factor(Factor::Power {
                    base: Box::new(Factor::Constant(2.0.into())),
                    exponent: Box::new(MathExpr::Term(Term::Factor(
                        Factor::Constant(0.0.into()),
                    ))),
                })),
                // 25
                Factor::Constant(25.0.into()),
            ))),
        );
    }

    #[traced_test]
    #[test]
    fn parenthesis_and_exponent() {
        parse_test(
            "2(3)^3",
            Ast::Expression(MathExpr::Term(Term::Multiply(
                MulType::Implicit,
                // 2
                Box::new(2f64.into()),
                // (3)^3
                Factor::Power {
                    base: Box::new(Factor::Parenthesis(Box::new(3f64.into()))),
                    exponent: Box::new(3f64.into()),
                },
            ))),
        );
    }

    #[traced_test]
    #[test]
    fn implicit_multiplication_and_exponent_order_of_operations() {
        parse_test(
            "2x^{2} + 5xy",
            Ast::Expression(MathExpr::Add(
                // 2x^{2}
                Box::new(MathExpr::Term(Term::Multiply(
                    MulType::Implicit,
                    // 2
                    Box::new(Term::Factor(Factor::Constant(2.0.into()))),
                    // x^{2}
                    Factor::Power {
                        base: Box::new(Factor::Variable(
                            MathIdentifier::from_single_ident("x"),
                        )),
                        exponent: Box::new(MathExpr::Term(Term::Factor(
                            Factor::Constant(2.0.into()),
                        ))),
                    },
                ))),
                // 5xy
                Term::Multiply(
                    MulType::Implicit,
                    // 5x
                    Box::new(Term::Multiply(
                        MulType::Implicit,
                        // 5
                        Box::new(5f64.into()),
                        // x
                        Factor::Variable(MathIdentifier::from_single_ident(
                            "x",
                        )),
                    )),
                    // y
                    Factor::Variable(MathIdentifier::from_single_ident("y")),
                ),
            )),
        );
    }

    #[traced_test]
    #[test]
    fn implicit_multiplication_single_identifier_token() {
        parse_test(
            "2xy^2",
            Ast::Expression(MathExpr::Term(Term::Multiply(
                MulType::Implicit,
                // 2x
                Box::new(Term::Multiply(
                    MulType::Implicit,
                    // 2
                    Box::new(2f64.into()),
                    // x
                    Factor::Variable(MathIdentifier::from_single_ident("x")),
                )),
                // y^2
                Factor::Power {
                    base: Box::new(Factor::Variable(
                        MathIdentifier::from_single_ident("y"),
                    )),
                    exponent: 2f64.into(),
                },
            ))),
        );
    }

    #[traced_test]
    #[test]
    fn pi() {
        parse_test(
            "\\pi",
            Ast::Expression(MathExpr::Term(Term::Factor(Factor::Variable(
                MathIdentifier::from_single_greek(GreekLetter::LowercasePi),
            )))),
        );
    }

    #[traced_test]
    #[test]
    fn implicit_multiplication_vs_function_call() {
        parse_test(
            "\\pi(x)\\ln(x)", // this is pi * x * ln(x)
            Ast::Expression(MathExpr::Term(Term::Multiply(
                MulType::Implicit,
                // \pi(x)
                Box::new(Term::Multiply(
                    MulType::Implicit,
                    Box::new(
                        Factor::Variable(MathIdentifier::from_single_greek(
                            GreekLetter::LowercasePi,
                        ))
                        .into(),
                    ),
                    Factor::Parenthesis(Box::new(
                        Factor::Variable(MathIdentifier::from_single_ident(
                            "x",
                        ))
                        .into(),
                    )),
                )),
                Factor::FunctionCall(FunctionCall {
                    function_name: MathIdentifier::from_single_symbol(
                        OtherSymbol::Ln,
                    ),
                    arguments: vec![Factor::Variable(
                        MathIdentifier::from_single_ident("x"),
                    )
                    .into()],
                }),
            ))),
        );
    }

    #[traced_test]
    #[test]
    fn division_order_of_operations() {
        parse_test(
            "5/2x + 3",
            // This is a bit mathematically ambiguous, but it means
            // 5/2 * x + 3 because multiplication and division are
            // on the same level, so it is evaluated left to right.
            Ast::Expression(MathExpr::Add(
                // 5/2x
                Box::new(MathExpr::Term(Term::Multiply(
                    MulType::Implicit,
                    // 5/2
                    Box::new(Term::Divide(
                        // 5
                        Box::new(Term::Factor(Factor::Constant(5.0.into()))),
                        // 2
                        Factor::Constant(2.0.into()),
                    )),
                    // x
                    Factor::Variable(MathIdentifier::from_single_ident("x")),
                ))),
                // 3
                Term::Factor(Factor::Constant(3.0.into())),
            )),
        );
    }

    #[traced_test]
    #[test]
    fn fraction() {
        parse_test(
            "\\frac{1}{2}",
            Ast::Expression(Factor::Fraction(1f64.into(), 2f64.into()).into()),
        );
    }

    #[traced_test]
    #[test]
    fn abs() {
        parse_test(
            "|-3|",
            Ast::Expression(
                Factor::Abs(Box::new(MathExpr::Term(Term::Multiply(
                    MulType::Implicit,
                    Box::new((-1f64).into()),
                    3f64.into(),
                ))))
                .into(),
            ),
        );
    }

    #[traced_test]
    #[test]
    fn equality() {
        parse_test(
            "x=2",
            Ast::Equality(
                Factor::Variable(MathIdentifier::from_single_ident("x")).into(),
                2f64.into(),
            ),
        );
    }
}
