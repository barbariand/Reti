pub use crate::{ast::Ast, context::MathContext};

pub(crate) type TokenResiver = Receiver<Token>;
pub(crate) use tokio::sync::mpsc;
pub(crate) type TokenSender = Sender<Token>;
#[allow(unused_imports)]
pub(crate) use crate::{
    approximator::Approximator,
    ast::{Factor, FunctionCall, MathExpr, MathIdentifier, Term},
    lexer::Lexer,
    normalizer::Normalizer,
    parsing::{ParseError, Parser},
    token::Token,
    token_reader::TokenReader,
};

use futures::FutureExt;
use std::{fmt::Display, panic::AssertUnwindSafe};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinError,
};
use tracing::{debug, error, trace, trace_span};

pub async fn parse(text: &str, context: &MathContext) -> Result<Ast, AstErrors> {
    let span = trace_span!("parsing");
    let _enter = span.enter();
    debug!(text);
    let channel_buffer_size = 32;
    let (lexer_in, lexer_out): (TokenSender, TokenResiver) = mpsc::channel(channel_buffer_size);
    debug!(
        "successfully created channel for lexer with {} long buffer",
        channel_buffer_size
    );
    let (normalizer_in, normalizer_out): (TokenSender, TokenResiver) =
        mpsc::channel(channel_buffer_size);
    debug!(
        "successfully created channel for normalizer with {} long buffer",
        channel_buffer_size
    );

    let lexer = Lexer::new(lexer_in);
    let normalizer = Normalizer::new(lexer_out, normalizer_in);
    let parser = Parser::new(normalizer_out, context.clone());
    let cloned_text = text.to_owned();
    trace!("cloned text");

    let lexer_future = async move { lexer.tokenize(&cloned_text).await };
    let normalizer_future = async move { normalizer.normalize().await };
    let parser_future = async move { parser.parse().await };
    trace!("spawning new tokenize async task");
    let lexer_handle = spawn_logging_task(lexer_future);
    trace!("spawning new normalizer async task");
    let normalizer_handle = spawn_logging_task(normalizer_future);
    trace!("spawning new parser async task");
    let parser_handle = spawn_logging_task(parser_future);

    let (lexer_result, normalizer_result, parser_result) =
        tokio::join!(lexer_handle, normalizer_handle, parser_handle);

    match lexer_result {
        Err(e) => {
            error!("lexer task failed");
            Err(AstErrors::Lexer(e))
        }
        Ok(Err(err)) => {
            error!("lexer task panicked");
            Err(AstErrors::LexerPanic(err.to_owned()))
        }
        _ => Ok(()),
    }?;

    match normalizer_result {
        Err(e) => {
            error!("normalizer task failed");
            Err(AstErrors::Normalizer(e))
        }
        Ok(Err(err)) => {
            error!("normalizer task failed");
            Err(AstErrors::NormalizerPanic(err.to_owned()))
        }
        _ => Ok(()),
    }?;
    match parser_result {
        Err(e) => {
            error!("parser task failed");
            Err(AstErrors::Parser(e))
        }
        Ok(Err(err)) => {
            error!("parser task failed");
            Err(AstErrors::ParserPanic(err.to_owned()))
        }
        Ok(Ok(ast)) => ast.map_err(AstErrors::ParseError),
    }
}

#[derive(Debug)]
pub enum AstErrors {
    Lexer(JoinError),
    LexerPanic(String),
    Normalizer(JoinError),
    NormalizerPanic(String),
    Parser(JoinError),
    ParserPanic(String),
    ParseError(ParseError),
}
impl Display for AstErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstErrors::Lexer(e) => write!(f, "lexer thread failed because: {}", e),
            AstErrors::Normalizer(e) => write!(f, "normalizer thread failed because: {}", e),
            AstErrors::Parser(e) => write!(f, "parser thread failed because: {}", e),
            AstErrors::ParseError(e) => write!(f, "Failed to parse string because: {}", e),
            AstErrors::LexerPanic(s) => write!(f, "Lexer Panicked: {}", s),
            AstErrors::NormalizerPanic(s) => write!(f, "Normalizer Panicked: {}", s),
            AstErrors::ParserPanic(s) => write!(f, "Parser Panicked: {}", s),
        }
    }
}
fn spawn_logging_task<F, T>(future: F) -> tokio::task::JoinHandle<Result<T, &'static str>>
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    tokio::spawn(async move {
        match AssertUnwindSafe(future).catch_unwind().await {
            Ok(result) => Ok(result),
            Err(err) => {
                let panic_message = if let Some(s) = err.downcast_ref::<&str>() {
                    *s
                } else {
                    "panic occurred in spawned task"
                };

                // Log the panic as a tracing event
                //error!(target: "panic", "Panic in spawned task: {}", panic_message);

                // Return the error for further handling if necessary
                Err(panic_message)
            }
        }
    })
}
#[cfg(test)]
mod tests {
    use crate::prelude::*;

    async fn parse_test(text: &str, expected_ast: Ast) {
        let found_ast = parse(text, &MathContext::standard_math()).await.unwrap();
        // Compare and print with debug and formatting otherwise.
        if expected_ast != found_ast {
            panic!("Expected: {:#?}\nFound: {:#?}", expected_ast, found_ast);
        }
    }

    #[tokio::test]
    async fn constant() {
        parse_test("1", Ast::Expression(1f64.into())).await;
    }

    #[tokio::test]
    async fn addition() {
        parse_test(
            "1+2+3",
            Ast::Expression(MathExpr::Add(
                Box::new(MathExpr::Add(Box::new(1f64.into()), 2f64.into())),
                3f64.into(),
            )),
        )
        .await;
    }

    #[tokio::test]
    async fn addition_multiplication_order_of_operations() {
        parse_test(
            "1+2+3+(4+5)*6",
            Ast::Expression(MathExpr::Add(
                Box::new(MathExpr::Add(
                    Box::new(MathExpr::Add(Box::new(1f64.into()), 2f64.into())),
                    3f64.into(),
                )),
                Term::Multiply(
                    Box::new(Term::Factor(Factor::Parenthesis(Box::new(MathExpr::Add(
                        Box::new(4f64.into()),
                        5f64.into(),
                    ))))),
                    6f64.into(),
                ),
            )),
        )
        .await;
    }
    #[tokio::test]
    async fn sqrt() {
        parse_test(
            "\\sqrt{9}",
            Ast::Expression(
                Factor::Root {
                    degree: None,
                    radicand: 9f64.into(),
                }
                .into(),
            ),
        )
        .await;
    }

    #[tokio::test]
    async fn cube_root() {
        parse_test(
            "\\sqrt[3]{27}",
            Ast::Expression(
                Factor::Root {
                    degree: Some(3f64.into()),
                    radicand: 27f64.into(),
                }
                .into(),
            ),
        )
        .await;
    }

    #[tokio::test]
    async fn exponent() {
        parse_test(
            "2^{3}",
            Ast::Expression(
                Factor::Power {
                    base: Box::new(Factor::Constant(2.0)),
                    exponent: Box::new(MathExpr::Term(Term::Factor(Factor::Constant(3.0)))),
                }
                .into(),
            ),
        )
        .await;
    }

    #[tokio::test]
    async fn exponent_command() {
        parse_test(
            "2^\\pi",
            Ast::Expression(
                Factor::Power {
                    base: Box::new(2f64.into()),
                    exponent: Box::new(
                        Factor::Variable(MathIdentifier {
                            tokens: vec![Token::Backslash, Token::Identifier("pi".to_string())],
                        })
                        .into(),
                    ),
                }
                .into(),
            ),
        )
        .await;
    }

    #[tokio::test]
    async fn exponent_split_token() {
        parse_test(
            "2^025", // this is 2^0 * 25
            Ast::Expression(MathExpr::Term(Term::Multiply(
                //2^0
                Box::new(Term::Factor(Factor::Power {
                    base: Box::new(Factor::Constant(2.0)),
                    exponent: Box::new(MathExpr::Term(Term::Factor(Factor::Constant(0.0)))),
                })),
                // 25
                Factor::Constant(25.0),
            ))),
        )
        .await;
    }

    #[tokio::test]
    async fn parenthesis_and_exponent() {
        parse_test(
            "2(3)^3",
            Ast::Expression(MathExpr::Term(Term::Multiply(
                // 2
                Box::new(2f64.into()),
                // (3)^3
                Factor::Power {
                    base: Box::new(Factor::Parenthesis(Box::new(3f64.into()))),
                    exponent: Box::new(3f64.into()),
                },
            ))),
        )
        .await;
    }

    #[tokio::test]
    async fn implicit_multiplication_and_exponent_order_of_operations() {
        parse_test(
            "2x^{2} + 5xy",
            Ast::Expression(MathExpr::Add(
                // 2x^{2}
                Box::new(MathExpr::Term(Term::Multiply(
                    // 2
                    Box::new(Term::Factor(Factor::Constant(2.0))),
                    // x^{2}
                    Factor::Power {
                        base: Box::new(Factor::Variable(MathIdentifier {
                            tokens: vec![Token::Identifier("x".to_string())],
                        })),
                        exponent: Box::new(MathExpr::Term(Term::Factor(Factor::Constant(2.0)))),
                    },
                ))),
                // 5xy
                Term::Multiply(
                    // 5x
                    Box::new(Term::Multiply(
                        // 5
                        Box::new(5f64.into()),
                        // x
                        Factor::Variable(MathIdentifier {
                            tokens: vec![Token::Identifier("x".to_string())],
                        }),
                    )),
                    // y
                    Factor::Variable(MathIdentifier {
                        tokens: vec![Token::Identifier("y".to_string())],
                    }),
                ),
            )),
        )
        .await;
    }

    #[tokio::test]
    async fn implicit_multiplication_single_identifier_token() {
        parse_test(
            "2xy^2",
            Ast::Expression(MathExpr::Term(Term::Multiply(
                // 2x
                Box::new(Term::Multiply(
                    // 2
                    Box::new(2f64.into()),
                    // x
                    Factor::Variable(MathIdentifier {
                        tokens: vec![Token::Identifier("x".to_string())],
                    }),
                )),
                // y^2
                Factor::Power {
                    base: Box::new(Factor::Variable(MathIdentifier {
                        tokens: vec![Token::Identifier("y".to_string())],
                    })),
                    exponent: 2f64.into(),
                },
            ))),
        )
        .await;
    }

    #[tokio::test]
    async fn pi() {
        parse_test(
            "\\pi",
            Ast::Expression(MathExpr::Term(Term::Factor(Factor::Variable(
                MathIdentifier {
                    tokens: vec![Token::Backslash, Token::Identifier("pi".to_string())],
                },
            )))),
        )
        .await;
    }

    #[tokio::test]
    async fn implicit_multiplication_vs_function_call() {
        parse_test(
            "\\pi(x)\\ln(x)", // this is pi * x * ln(x)
            Ast::Expression(MathExpr::Term(Term::Multiply(
                // \pi(x)
                Box::new(Term::Multiply(
                    Box::new(
                        Factor::Variable(MathIdentifier {
                            tokens: vec![Token::Backslash, Token::Identifier("pi".to_string())],
                        })
                        .into(),
                    ),
                    Factor::Parenthesis(Box::new(
                        Factor::Variable(MathIdentifier {
                            tokens: vec![Token::Identifier("x".to_string())],
                        })
                        .into(),
                    )),
                )),
                Factor::FunctionCall(FunctionCall {
                    function_name: MathIdentifier {
                        tokens: vec![Token::Backslash, Token::Identifier("ln".to_string())],
                    },
                    arguments: vec![Factor::Variable(MathIdentifier {
                        tokens: vec![Token::Identifier("x".to_string())],
                    })
                    .into()],
                }),
            ))),
        )
        .await;
    }

    #[tokio::test]
    async fn division_order_of_operations() {
        parse_test(
            "5/2x + 3",
            // This is a bit mathematically ambiguous, but it means
            // 5/2 * x + 3 because multiplication and division are
            // on the same level, so it is evaluated left to right.
            Ast::Expression(MathExpr::Add(
                // 5/2x
                Box::new(MathExpr::Term(Term::Multiply(
                    // 5/2
                    Box::new(Term::Divide(
                        // 5
                        Box::new(Term::Factor(Factor::Constant(5.0))),
                        // 2
                        Factor::Constant(2.0),
                    )),
                    // x
                    Factor::Variable(MathIdentifier {
                        tokens: vec![Token::Identifier("x".to_string())],
                    }),
                ))),
                // 3
                Term::Factor(Factor::Constant(3.0)),
            )),
        )
        .await;
    }

    #[tokio::test]
    async fn fraction() {
        parse_test(
            "\\frac{1}{2}",
            Ast::Expression(Factor::Fraction(1f64.into(), 2f64.into()).into()),
        )
        .await;
    }

    #[tokio::test]
    async fn abs() {
        parse_test(
            "|-3|",
            Ast::Expression(
                Factor::Abs(Box::new(MathExpr::Term(Term::Multiply(
                    Box::new((-1f64).into()),
                    3f64.into(),
                ))))
                .into(),
            ),
        )
        .await;
    }

    #[tokio::test]
    async fn equality() {
        parse_test(
            "x=2",
            Ast::Equality(
                Factor::Variable(MathIdentifier {
                    tokens: vec![Token::Identifier("x".to_string())],
                })
                .into(),
                2f64.into(),
            ),
        )
        .await;
    }
}
