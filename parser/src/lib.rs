//! Reti is an in-development calculator that can evaluate LaTeX expressions.
#![allow(dead_code)]
#![warn(missing_docs, clippy::missing_docs_in_private_items)]

pub mod approximator;
pub mod ast;

pub mod context;
pub mod lexer;

pub mod error;
pub mod matrix;
pub mod normalizer;
pub mod parsing;
pub mod prelude;
pub mod token;
pub mod token_reader;
pub mod value;
pub use prelude::parse;
