#![allow(dead_code)]

mod approximator;
mod ast;

mod context;
mod lexer;
mod normalizer;
mod parsing;
pub mod prelude;
mod token;
mod token_reader;
pub use prelude::parse;
