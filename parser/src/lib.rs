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

pub mod ast;
mod context;
mod evaluator;
mod lexer;
mod normalizer;
mod parser;
pub mod token;
mod token_reader;

pub async fn parse(text: &str) -> Result<Ast, Error> {
    let (lexer_in, lexer_out): (Sender<Token>, Receiver<Token>) = mpsc::channel(32);
    let (normalizer_in, normalizer_out): (Sender<Token>, Receiver<Token>) = mpsc::channel(32);

    let context = MathContext::new();
    let lexer = Lexer::new(lexer_in);
    let mut normalizer = Normalizer::new(lexer_out, normalizer_in);
    let mut parser = Parser::new(normalizer_out, context);
    let cloned_text = text.to_owned();
    let lexer_handle = tokio::spawn(async move { lexer.tokenize(&cloned_text).await });
    let normalizer_handle = tokio::spawn(async move { normalizer.normalize().await });
    let parser_handle = tokio::spawn(async move { parser.parse().await });
    match lexer_handle.await {
        Err(e) => {
            normalizer_handle.abort();
            parser_handle.abort();
            Err(Error::Lexer(e))
        }
        Ok(()) => match normalizer_handle.await {
            Err(e) => {
                parser_handle.abort();
                Err(Error::Normalizer(e))
            }
            Ok(()) => match parser_handle.await {
                Err(e) => Err(Error::Parser(e)),
                Ok(ast) => ast.map_err(Error::ParseError),
            },
        },
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
            Error::Normalizer(e) => write!(f, "normalizer thread failed beacuse: {}", e),
            Error::Parser(e) => write!(f, "parser thread failed beacuse: {}", e),
            Error::ParseError(e) => write!(f, "Failed to parse string beacuse: {}", e),
        }
    }
}
