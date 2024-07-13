//! Reti is an in-development calculator that can evaluate LaTeX expressions.
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::missing_const_for_fn
)]

pub mod approximator;
pub mod ast;
pub mod consts;
pub mod context;
pub mod error;
pub mod functions;
pub mod identifier;
pub mod lexer;
pub mod matrix;
pub mod normalizer;
pub mod parsing;
pub mod prelude;
pub mod token;
pub mod token_reader;
pub mod value;
pub use prelude::parse;
