use slicedisplay::SliceDisplay;
use snafu::Snafu;
use tokio::task::JoinError;

use crate::prelude::Token;

#[derive(Debug, Snafu)]
pub enum ParseError {
    #[snafu(display(
        "Got unexpected Token:\"{found}\", expected one of Tokens:\"{}\"", expected.display()
    ))]
    UnexpectedToken { expected: Vec<Token>, found: Token },
    #[snafu(display("Got invalid token:\"{token}\""))]
    Invalid { token: Token },
    #[snafu(display("Trailing invalid token\"{token}\""))]
    Trailing { token: Token },
    #[snafu(display("Got invalid \\begin{{{token}}}"))]
    InvalidFactor { token: Token },
    #[snafu(display("Trailing invalid token\"{beginning}\""))]
    InvalidBegin { beginning: String },
    #[snafu(display("Expected it to have the same amount of columns, but previous had:{prev} instead got:{current}"))]
    MismatchedMatrixColumnSize { prev: usize, current: usize },
}
#[derive(Debug, Snafu)]
pub enum AstError {
    #[snafu(transparent)]
    Join { source: JoinError },
    #[snafu(whatever)]
    Panic { message: String },
    #[snafu(transparent)]
    ParseError { source: ParseError },
}
