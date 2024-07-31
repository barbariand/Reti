//! Reti is an in-development calculator that can evaluate LaTeX expressions.
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::missing_const_for_fn
)]
#![allow(non_snake_case)]
//enabeling the doc_test feature when running doc tests so that we can have a
// usable parse function in the thingy
#![cfg_attr(doc, feature = "doc_test")]

pub mod approximator;
pub mod ast;
pub mod consts;
pub mod context;
pub mod error;
pub mod evaluator;
pub mod functions;
pub mod identifier;
pub mod lexer;
pub mod matrix;
pub mod normalizer;
pub mod number_literal;
pub mod parsing;
pub mod prelude;
pub mod token;
pub mod token_reader;
pub mod value;
pub use prelude::parse;
