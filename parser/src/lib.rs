#![allow(dead_code)]

use std::fmt::Display;

use ast::Ast;
use context::MathContext;

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

pub async fn parse(text: &str) -> Result<Ast, Error> {
    let span = trace_span!("parsing");
    let _enter = span.enter();
    debug!(text);
    let channel = 32;
    trace!("reading channel {channel:?}");
    let (lexer_in, lexer_out): (Sender<Token>, Receiver<Token>) =
        mpsc::channel(channel);
    debug!("successfully connected to lexer");
    let (normalizer_in, normalizer_out): (Sender<Token>, Receiver<Token>) =
        mpsc::channel(channel);
    debug!("successfully connected to normalizer");

    let context = MathContext::new();
    trace!("created MathContext");
    let lexer = Lexer::new(lexer_in);
    trace!("created Lexer");
    let mut normalizer = Normalizer::new(lexer_out, normalizer_in);
    trace!("created Normalizer");
    let mut parser = Parser::new(normalizer_out, context);
    trace!("created Parser");
    let cloned_text = text.to_owned();
    trace!("cloned text");
    trace!("spawning new tokenize async task");
    let lexer_handle =
        tokio::spawn(async move { lexer.tokenize(&cloned_text).await });
    trace!("spawning new normalize async task");
    let normalizer_handle =
        tokio::spawn(async move { normalizer.normalize().await });
    trace!("spawning new parser async task");
    let parser_handle = tokio::spawn(async move { parser.parse().await });
    match lexer_handle.await {
        Err(e) => {
            error!("lexer task failed");
            normalizer_handle.abort();
            parser_handle.abort();
            Err(Error::Lexer(e))
        }
        Ok(()) => {
            debug!("lexer task finished");
            match normalizer_handle.await {
                Err(e) => {
                    error!("normalizer task failed");
                    parser_handle.abort();
                    Err(Error::Normalizer(e))
                }
                Ok(()) => {
                    debug!("normalizer task finished");
                    match parser_handle.await {
                        Err(e) => {
                            error!("parser task failed");
                            Err(Error::Parser(e))
                        }
                        Ok(ast) => {
                            debug!("parser task finished");
                            error!("ast generation failed");
                            ast.map_err(Error::ParseError)
                        }
                    }
                }
            }
        }
    }
}
#[derive(Debug)]
pub enum Error {
    Lexer(JoinError),
    Normalizer(JoinError),
    Parser(JoinError),
    ParseError(ParseError),
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Lexer(e) => write!(f, "lexer thread failed beacuse: {}", e),
            Error::Normalizer(e) => {
                write!(f, "normalizer thread failed beacuse: {}", e)
            }
            Error::Parser(e) => {
                write!(f, "parser thread failed beacuse: {}", e)
            }
            Error::ParseError(e) => {
                write!(f, "Failed to parse string beacuse: {}", e)
            }
        }
    }
}
