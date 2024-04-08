#![allow(dead_code)]

use std::{fmt::Display, panic::AssertUnwindSafe};

use ast::Ast;
use context::MathContext;

use futures::{
    future::{join, join3},
    join, FutureExt,
};
use lexer::Lexer;
use normalizer::Normalizer;
use parser::{ParseError, Parser};
use token::Token;
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::JoinError,
};
use tracing::error;

pub mod ast;
mod context;
mod evaluator;
mod lexer;
mod normalizer;
mod parser;
pub mod token;
mod token_reader;

pub async fn parse(text: &str) -> Result<Ast, AstErrors> {
    let (lexer_in, lexer_out): (Sender<Token>, Receiver<Token>) = mpsc::channel(32);
    let (normalizer_in, normalizer_out): (Sender<Token>, Receiver<Token>) = mpsc::channel(32);

    let context = MathContext::new();
    let lexer = Lexer::new(lexer_in);
    let normalizer = Normalizer::new(lexer_out, normalizer_in);
    let parser = Parser::new(normalizer_out, context);
    let cloned_text = text.to_owned();
    let lexer_future = async move { lexer.tokenize(&cloned_text).await };
    let normalizer_future = async move { normalizer.normalize().await };
    let parser_future = async move { parser.parse().await };
    let lexer_handle = spawn_logging_task(lexer_future);
    let normalizer_handle = spawn_logging_task(normalizer_future);
    let parser_handle = spawn_logging_task(parser_future);

    let (lexer_result, normalizer_result, parser_result) =
        join3(lexer_handle, normalizer_handle, parser_handle).await;

    match lexer_result {
        Err(e) => Err(AstErrors::Lexer(e)),
        Ok(Err(err)) => Err(AstErrors::LexerPaniced(err.to_owned())),
        _ => Ok(()),
    }?;
    match normalizer_result {
        Err(e) => Err(AstErrors::Lexer(e)),
        Ok(Err(err)) => Err(AstErrors::NormalizerPaniced(err.to_owned())),
        _ => Ok(()),
    }?;
    match parser_result {
        Err(e) => Err(AstErrors::Lexer(e)),
        Ok(Err(err)) => Err(AstErrors::ParserPaniced(err.to_owned())),
        Ok(Ok(ast)) => ast.map_err(AstErrors::ParseError),
    }
}
#[derive(Debug)]
pub enum AstErrors {
    Lexer(JoinError),
    LexerPaniced(String),
    Normalizer(JoinError),
    NormalizerPaniced(String),
    Parser(JoinError),
    ParserPaniced(String),
    ParseError(ParseError),
}
impl Display for AstErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstErrors::Lexer(e) => write!(f, "lexer thread failed beacuse: {}", e),
            AstErrors::Normalizer(e) => write!(f, "normalizer thread failed beacuse: {}", e),
            AstErrors::Parser(e) => write!(f, "parser thread failed beacuse: {}", e),
            AstErrors::ParseError(e) => write!(f, "Failed to parse string beacuse: {}", e),
            AstErrors::LexerPaniced(s) => write!(f, "Lexer Paniced: {}", s),
            AstErrors::NormalizerPaniced(s) => write!(f, "Normalizer Paniced: {}", s),
            AstErrors::ParserPaniced(s) => write!(f, "Parser Paniced: {}", s),
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
                error!(target: "panic", "Panic in spawned task: {}", panic_message);

                // Return the error for further handling if necessary
                Err(panic_message)
            }
        }
    })
}
