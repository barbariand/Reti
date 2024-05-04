//! Reti is an in-development calculator that can evaluate LaTeX expressions.
#![allow(dead_code)]
#![warn(missing_docs, clippy::missing_docs_in_private_items)]

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
