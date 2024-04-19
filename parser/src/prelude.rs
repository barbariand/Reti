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
