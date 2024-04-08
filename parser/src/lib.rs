#![allow(dead_code)]

use std::{fmt::Display, panic::AssertUnwindSafe};

use ast::Ast;
use context::MathContext;

use futures::{future::join3, FutureExt};
use lexer::Lexer;
use normalizer::Normalizer;
use parser::{ParseError, Parser};
use token::Token;
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::JoinError,
};
use tracing::{debug, error, trace, trace_span};

pub mod ast;
mod context;
mod evaluator;
mod lexer;
mod normalizer;
mod parser;
pub mod token;
mod token_reader;

pub async fn parse(text: &str) -> Result<Ast, AstErrors> {
    let span = trace_span!("parsing");
    let _enter = span.enter();
    debug!(text);
    let channel = 32;
    trace!("reading channel {channel:?}");
    let (lexer_in, lexer_out): (Sender<Token>, Receiver<Token>) = mpsc::channel(channel);
    debug!("successfully connected to lexer");
    let (normalizer_in, normalizer_out): (Sender<Token>, Receiver<Token>) = mpsc::channel(channel);
    debug!("successfully connected to normalizer");

    let context = MathContext::new();
    trace!("created MathContext");
    let lexer = Lexer::new(lexer_in);
    trace!("created Lexer");
    let normalizer = Normalizer::new(lexer_out, normalizer_in);
    trace!("created Normalizer");
    let parser = Parser::new(normalizer_out, context);
    trace!("created Parser");
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
        join3(lexer_handle, normalizer_handle, parser_handle).await;

    match lexer_result {
        Err(e) => {
            error!("lexer task failed");
            Err(AstErrors::Lexer(e))
        }
        Ok(Err(err)) => {
            error!("lexer task paniced");
            Err(AstErrors::LexerPaniced(err.to_owned()))
        }
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
                //error!(target: "panic", "Panic in spawned task: {}", panic_message);

                // Return the error for further handling if necessary
                Err(panic_message)
            }
        }
    })
}
