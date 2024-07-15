//!The errors that can happen when evaluating a latex string
use crate::prelude::{MulType, Token};
use slicedisplay::SliceDisplay;
use snafu::Snafu;
use tokio::task::JoinError;
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
    /// When an unexpected command is encountered when parsing a
    /// MathIdentifier.
    #[snafu(display("Unexpected command {{{command}}}, I expected either a greek letter like \\alpha or a modifier like \\overline{{x}}"))]
    InvalidIdentifierCommmand {
        /// The command that was unexpected.
        command: String,
    },
    /// When an unexpected token is encountered when parsing a MathIdentifier.
    #[snafu(display("Unexpected token {{{token}}}, I expected either a greek letter like \\alpha or a modifier like \\overline{{x}}"))]
    InvalidIdentifierToken {
        /// The token that was unexpected.
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
    #[snafu(display("A matrix cannot be empty."))]
    ///The matrix was an empty matrix
    EmptyMatrix,
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
    #[snafu(display(
        "Invalid amount of arguments, expected lengths of {}, but found the length {}",expected.display(),found
    ))]
    ArgumentLengthMismatch {
        ///The possible amounts of arguments it can have because of
        /// overloading
        expected: Vec<usize>,
        ///the found amount of arguments
        found: usize,
    },
    #[snafu(transparent)]
    ///Cant derive this expression
    DeriveError {
        ///The actual derive error
        source: DeriveError,
    },
    /// Division by zero.
    #[snafu(display("Cannot divide by zero"))]
    DivideByZero,
}
/// The error for when it required another size of the matrix
#[derive(Debug, Snafu)]
pub enum IncompatibleMatrixSizes {
    // TODO I don't like how we say that something is "expected" here. We
    // can't say something is expected, we just know that they are
    // incompatible. /Alvin
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
    ///Invalid Vector size for cross product
    #[snafu(display("Cross product can only be used on vectors with 3 components, got {found_size:?}"))]
    CrossProduct {
        /// The found size of the vector.
        found_size: usize,
    },
    ///Was not a Vector instead had more then 1 row and column
    #[snafu(display(
        "Expected a vector but got a {rows:?}x{columns:?} matrix."
    ))]
    Vector {
        ///The rows of the matrix
        rows: usize,
        ///The columns of the matrix
        columns: usize,
    },
    ///The vectors are not the same dimensions
    #[snafu(display(
        "Vectors must be of the same size, but got {a:?} and {b:?}"
    ))]
    SameSizeVectors {
        ///first vector dimensions
        a: usize,
        ///Second Vector dimensions
        b: usize,
    },
}
#[derive(Debug, Snafu)]
///All the ways we cant derive
pub enum DeriveError {
    ///So it don't complain
    #[snafu(whatever, display("The types are not compatible: {message}"))]
    All {
        ///the message
        message: String,
    },
}
