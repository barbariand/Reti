use slicedisplay::SliceDisplay;
use snafu::Snafu;
use tokio::task::JoinError;

use crate::prelude::{MulType, Token};
///The errors that can happen while parsing latex
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
///The errors that can hapen when generating the AST
#[derive(Debug, Snafu)]
pub enum AstError {
    #[snafu(transparent)]
    Join { source: JoinError },
    #[snafu(whatever)]
    Panic { message: String },
    #[snafu(transparent)]
    ParseError { source: ParseError },
}
/// The errors that can happen when evaluating a AST
#[derive(Debug, Snafu)]
pub enum EvalError {
    #[snafu(display(
        "A matrix was found when it was expected to be a scalar"
    ))]
    ExpectedScalar,
    #[snafu(whatever, display("The types are not compatible: {message}"))]
    IncompatibleTypes { message: String },
    #[snafu(transparent)]
    IncompatibleMatrixSizes { source: IncompatibleMatrixSizes },
    #[snafu(display("Value is undefined"))]
    NotDefined,
    /// Unclear multiplication type when multiplying matricies.
    #[snafu(display(
        "Unclear multiplication type {type:?} when multiplying matricies"
    ))]
    AmbiguousMulType { r#type: MulType },
}
/// The error for when it required another size of the matrix
#[derive(Debug, Snafu)]
pub enum IncompatibleMatrixSizes {
    #[snafu(display("Expected row {expected:?} found {found:?}"))]
    Row {
        /// The expected value for the matrix
        expected: usize,
        /// The value found
        found: usize,
    },
    #[snafu(display("Expected column {expected:?} found {found:?}"))]
    Column {
        /// The expected value for the matrix
        expected: usize,
        /// The value found
        found: usize,
    },
}
