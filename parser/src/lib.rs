#![allow(dead_code)]

mod approximator;
mod ast;

mod context;
mod lexer;
pub mod matrix;
mod normalizer;
mod parsing;
pub mod prelude;
mod token;
mod token_reader;
pub mod value;
pub use prelude::parse;
pub mod value;
