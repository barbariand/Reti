//!The errors that can happen when evaluating a latex string
use slicedisplay::SliceDisplay;
use snafu::Snafu;
use tokio::task::JoinError;

use crate::prelude::{MulType, Token};
///The errors that can happen while parsing latex
#[derive(Debug, Snafu)]
pub enum ParseError {
    ///Unexpected token, expected one off
    #[snafu(display(
        "Got unexpected Token:\"{found}\", expected one of Tokens:\"{}\"", expected.display()
    ))]
    UnexpectedToken {
        /// expected one of the tokens here
        expected: Vec<Token>,
        ///but found this one instead
        found: Token,
    },
    ///Invalid token
    #[snafu(display("Got invalid token:\"{token}\""))]
    Invalid {
        ///this token is invalid
        token: Token,
    },
    ///Invalid Trailing token
    #[snafu(display("Trailing invalid token\"{token}\""))]
    Trailing {
        ///the Trailing token
        token: Token,
    },
    ///Invalid Factor
    #[snafu(display("Got invalid when parsing factor{{{token}}}"))]
    InvalidFactor {
        ///The token could not be understood when parsing a factor
        token: Token,
    },
    ///Invalid in begin statement
    #[snafu(display("Got invalid \\begin{{{beginning}}}"))]
    InvalidBegin {
        ///The string of what was to "begin"
        beginning: String,
    },
    ///Column size is wrong
    #[snafu(display("Expected it to have the same amount of columns, but previous had:{prev} instead got:{current}"))]
    MismatchedMatrixColumnSize {
        ///the expected
        prev: usize,
        ///what was found
        current: usize,
    },
}
///The errors that can happen when generating the AST
#[derive(Debug, Snafu)]
pub enum AstError {
    ///could not join the threads
    #[snafu(transparent)]
    Join {
        ///the source of what crashed
        source: JoinError,
    },
    ///Thread panicked
    #[snafu(whatever)]
    Panic {
        ///panic message
        message: String,
    },
    ///Got a parse error
    #[snafu(transparent)]
    ParseError {
        ///The ParseError
        source: ParseError,
    },
}
/// The errors that can happen when evaluating a AST
#[derive(Debug, Snafu)]
pub enum EvalError {
    ///Expected a scalar but found a matrix
    #[snafu(display(
        "A matrix was found when it was expected to be a scalar"
    ))]
    ExpectedScalar,
    ///Incompatible types
    #[snafu(whatever, display("The types are not compatible: {message}"))]
    IncompatibleTypes {
        ///The message describing it further
        message: String,
    },
    ///When the matrix is not correctly sized for the operation
    #[snafu(transparent)]
    IncompatibleMatrixSizes {
        ///Enum for describing what is wrong
        source: IncompatibleMatrixSizes,
    },
    ///The variable or function is not defined
    #[snafu(display("Value is undefined"))]
    NotDefined,
    /// Unclear multiplication type when multiplying matrices.
    #[snafu(display(
        "Unclear multiplication type {type:?} when multiplying matrices"
    ))]
    AmbiguousMulType {
        ///there is not a matrix operation defined for that type
        r#type: MulType,
    },
    ///Invalid amount of arguments
    ArgumentLengthMismatch {
        ///The possible amounts of arguments it can have because of
        /// overloading
        expected: Vec<usize>,
        ///the found amount of arguments
        found: usize,
    },
}
/// The error for when it required another size of the matrix
#[derive(Debug, Snafu)]
pub enum IncompatibleMatrixSizes {
    ///Rows don't match
    #[snafu(display("Expected row {expected:?} found {found:?}"))]
    Row {
        /// The expected value for the matrix
        expected: usize,
        /// The value found
        found: usize,
    },
    ///Columns don't match
    #[snafu(display("Expected column {expected:?} found {found:?}"))]
    Column {
        /// The expected value for the matrix
        expected: usize,
        /// The value found
        found: usize,
    },
}
