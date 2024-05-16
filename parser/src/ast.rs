//! AST for representing Latex
use crate::prelude::*;

pub mod derivative;
pub mod equality;
pub mod factorize;
pub mod helper;
pub mod into;
pub mod simplify;
pub mod to_latex;
///The root of the AST that is non recursive
#[derive(PartialEq, Debug)]
pub enum Ast {
    /// A mathematical expression that can be evaluated.
    Expression(MathExpr),
    /// An equation consisting of an equality between a left-hand side and a
    /// right-hand side.
    Equality(MathExpr, MathExpr),
}
/// A mathematical expression that consists of one or more terms added
/// or subtracted.
///
/// See Wikipedia article [Expression (mathematics)](https://en.wikipedia.org/wiki/Expression_(mathematics)).
#[derive(PartialEq, Debug, Clone)]
pub enum MathExpr {
    /// A [Term] containing the rest of the syntax that go before in evaluation
    Term(Term),
    /// Addition between a MathExpr and a Term.
    ///  ## Examples
    ///  ```
    /// # use parser::ast::*;
    /// # use parser::prelude::*;
    /// # use parser::prelude::_private::parse_sync_doc_test as parse;
    /// # let context=MathContext::standard_math();
    /// assert_eq!(
    ///     parse("2+2", &context),
    ///     Ast::Expression(
    ///         MathExpr::Add(
    ///             Box::new(
    ///                 Factor::Constant(2.0).into()
    ///             ),
    ///             Factor::Constant(2.0).into()
    ///         )
    ///     )
    /// );
    /// ```
    Add(Box<MathExpr>, Term),
    /// Subtraction between a MathExpr and a Term.
    /// ## Examples
    ///  ```
    /// # use parser::ast::*;
    /// # use parser::prelude::MathContext;
    /// # use parser::prelude::_private::parse_sync_doc_test as parse;
    /// # let context=MathContext::standard_math();
    /// assert_eq!(
    ///     parse("2-2", &context),
    ///     Ast::Expression(
    ///         MathExpr::Subtract(
    ///             Box::new(
    ///                 Factor::Constant(2.0).into()
    ///             ),
    ///             Factor::Constant(2.0).into()
    ///         )
    ///     )
    /// );

    /// ```
    Subtract(Box<MathExpr>, Term),
}

/// The type of multiplication.
///
/// For scalar multiplication, the type of multiplication makes no difference,
/// but in some cases, for example when multiplying vectors, the symbol used
/// for multiplication makes a difference.
#[derive(PartialEq, Debug, Clone)]
pub enum MulType {
    /// 2 * x
    ///
    /// Not defined when used on matrices.
    Asterisk,

    /// 2 \cdot x
    ///
    /// Dot product when used on matrices (vectors).
    Cdot,

    /// 2 \times x
    ///
    /// Cross product when used on matrices (vectors).
    Times,

    /// 2x
    ///
    /// Matrix multiplication when used on matrices.
    Implicit,
}

/// A term consists of a number or variable, or the product or quotient of
/// multiple numbers and variables.
///
/// In regard to the order of operations, the individual terms are always
/// evaluated before being added or subtracted.
///
/// See Wikipedia article [Term (mathematics)](https://simple.wikipedia.org/wiki/Term_(mathematics)).
///
/// ## Examples
/// For example, in
/// > 1 + 2x + 8yzx
///
/// *1*, *2x*, and *8yzx* are three separate terms.
#[derive(PartialEq, Debug, Clone)]
pub enum Term {
    /// A [Factor] containing the rest of the syntax that go before in
    /// evaluation
    Factor(Factor),
    ///Multiplication of Term and Factor
    /// ## Examples
    ///  ```
    /// # use parser::ast::*;
    /// # use parser::prelude::MathContext;
    /// # use parser::prelude::_private::parse_sync_doc_test as parse;
    /// # let context=MathContext::standard_math();
    /// assert_eq!(
    ///     parse("2*2", &context),
    ///     Ast::Expression(
    ///         Term::Multiply(
    ///             MulType::Asterisk,
    ///             Box::new(Term::Factor(
    ///                 Factor::Constant(2.0)
    ///             )),
    ///             Factor::Constant(2.0)
    ///         ).into()
    ///     )
    /// );

    /// ```
    Multiply(MulType, Box<Term>, Factor),
    /// Division between a Term and Factor.
    /// ## Examples
    ///  ```
    /// # use parser::ast::*;
    /// # use parser::prelude::MathContext;
    /// # use parser::prelude::_private::parse_sync_doc_test as parse;
    /// # let context=MathContext::standard_math();
    /// assert_eq!(
    ///     parse("2/2", &context),
    ///     Ast::Expression(
    ///         Term::Divide(
    ///             Box::new(Term::Factor(
    ///                 Factor::Constant(2.0)
    ///             )),
    ///             Factor::Constant(2.0)
    ///         ).into()
    ///     )
    /// );

    /// ```
    Divide(Box<Term>, Factor),
}
/// A factor that consists of a single value.
///
/// Factors are in some sense the bottom of the Abstract Syntax Tree, and
/// factors will always be evaluated first before being multiplied or added
/// together.
///
/// Factors also represent most of the mathematical syntax, like roots and
/// functions. This is because they operate on the same level in terms of
/// order of operations.
#[derive(PartialEq, Debug, Clone)]
pub enum Factor {
    /// Normal numbers
    /// ## Examples
    ///  ```
    /// # use parser::ast::*;
    /// # use parser::prelude::MathContext;
    /// # use parser::prelude::_private::parse_sync_doc_test as parse;
    /// # let context=MathContext::standard_math();
    /// assert_eq!(
    ///     parse("2", &context),
    ///     Ast::Expression(
    ///         Factor::Constant(2.0).into()
    ///     )
    /// );
    /// assert_eq!(
    ///     parse("1.1", &context),
    ///     Ast::Expression(
    ///         Factor::Constant(1.1).into()
    ///     )
    /// );
    /// ```
    Constant(f64),
    /// Parenthesis with a MathExpr
    /// ## Examples
    /// ```
    /// # use parser::ast::*;
    /// # use parser::token::Token;
    /// # use parser::prelude::MathContext;
    /// # use parser::prelude::_private::parse_sync_doc_test as parse;
    /// # let mut context=MathContext::standard_math();
    /// // parsing `(1)`
    /// assert_eq!(
    ///     parse("(1)", &context),
    ///     Ast::Expression(
    ///         Factor::Parenthesis(Box::new(Factor::Constant(1.0).into())).into()
    ///     )
    /// );

    /// ```
    Parenthesis(Box<MathExpr>),

    /// A variable whose value is not known at parse time.
    ///
    /// Note that the term variable here refers to the fact that the value is
    /// unknown at parse time. While evaluating, the value of the variable may
    /// be constant, for example \pi or user-defined variables/constants.
    /// The variable could also vary, such as when defining functions. For
    /// example, if f(x)=2x, then the value of x will vary between calls to
    /// f.
    ///
    /// Variables are identified using the [MathIdentifier] struct.
    /// ## Examples
    /// ```
    /// # use parser::ast::*;
    /// # use parser::token::Token;
    /// # use parser::prelude::MathContext;
    /// # use parser::value::Value;
    /// # use parser::prelude::_private::parse_sync_doc_test as parse;
    /// # let mut context=MathContext::standard_math();
    /// # context.variables.insert(MathIdentifier::new(vec![Token::Identifier("f".to_owned())]), Value::Scalar(2.0));
    /// // parsing x
    ///
    /// assert_eq!(
    ///     parse("x", &context),
    ///     Ast::Expression(
    ///         Factor::Variable(
    ///             MathIdentifier::new(
    ///                 vec![
    ///                     Token::Identifier("x".to_owned())
    ///                 ]
    ///             )
    ///         ).into()
    ///     )
    /// );
    /// ```
    Variable(MathIdentifier),
    /// An expression that represents a function that is being invoked.
    /// ## Examples
    /// ```
    /// # use parser::ast::*;
    /// # use parser::token::Token;
    /// # use parser::prelude::*;
    /// # use parser::context::IntoMathFunction;
    /// # use parser::prelude::_private::parse_sync_doc_test as parse;
    /// # use std::sync::Arc;
    /// # use parser::value::Value;
    /// # let mut context=MathContext::standard_math();
    /// # context.add_function(
    /// # vec![Token::Identifier("f".to_owned())],
    /// # (|_x:f64|{2.0},None)
    /// # );
    /// // parsing f(x)
    /// // where f needs to be defined for it to be interpreted as a function call
    ///
    /// assert_eq!(parse("f(x)",&context),
    /// Ast::Expression(
    ///     Factor::FunctionCall(
    ///         FunctionCall::new(
    ///             MathIdentifier::new(
    ///                 vec![Token::Identifier("f".to_owned())]
    ///             ),
    ///             vec![
    ///                 Factor::Variable(
    ///                     MathIdentifier::new(
    ///                         vec![Token::Identifier("x".to_owned())]
    ///                     )
    ///                 ).into()
    ///             ],
    ///         )
    ///     ).into()
    /// ));
    /// ```
    FunctionCall(FunctionCall),

    /// An exponetiation that describes an expression that is being raised to
    /// the power of an exponent.
    /// ## Examples
    /// ```
    /// # use parser::ast::*;
    /// # use parser::token::Token;
    /// # use parser::prelude::MathContext;
    /// # use parser::prelude::_private::parse_sync_doc_test as parse;
    /// # let mut context=MathContext::standard_math();
    /// // parsing 3^2
    /// assert_eq!(
    ///     parse("3^2", &context),
    ///     Ast::Expression(
    ///         Factor::Power {
    ///             base: Box::new(Factor::Constant(3.0)),
    ///             exponent: Box::new(Factor::Constant(2.0).into())
    ///         }
    ///         .into()
    ///     )
    /// );
    /// ```
    Power {
        /// The base of the ^ so in our example about it would be 3.0 for
        /// Fraction::Power
        base: Box<Factor>,
        /// The exponent of the ^ so the 2 in our example above for
        /// Fraction::Power
        exponent: Box<MathExpr>,
    },
    /// The root of a MathExpr
    /// ## Examples
    /// ```
    /// # use parser::ast::*;
    /// # use parser::token::Token;
    /// # use parser::prelude::MathContext;
    /// # use parser::prelude::_private::parse_sync_doc_test as parse;
    /// # let mut context=MathContext::standard_math();
    /// // parsing \sqrt[3]{2}
    /// assert_eq!(
    ///     parse("\\sqrt[3]{2}", &context),
    ///     Ast::Expression(
    ///         Factor::Root {
    ///             degree: Some(Box::new(Factor::Constant(3.0).into())),
    ///             radicand: Box::new(Factor::Constant(2.0).into()),
    ///         }
    ///         .into()
    ///     )
    /// );
    /// ```
    Root {
        ///Optional degree of the root, otherwise understood as sqrt
        degree: Option<Box<MathExpr>>,
        /// The thing to take the Nth Root of
        radicand: Box<MathExpr>,
    },

    /// A fraction.
    ///
    /// Note that fractions are treated as a factor, despite them being a
    /// division. This is because, in expressions like \frac{1}{2}x, the
    /// fraction acts like a factor.
    ///
    /// Also note that the term "fraction" is used to denote a quotient
    /// regardless of the contents on the numerator and denominator while
    /// the mathematical definition requires they be integers.
    ///
    /// ## Examples
    /// ```
    /// # use parser::ast::*;
    /// # use parser::token::Token;
    /// # use parser::prelude::MathContext;
    /// # use parser::prelude::_private::parse_sync_doc_test as parse;
    /// # let mut context=MathContext::standard_math();
    /// // parsing \frac{1}{2}
    /// assert_eq!(
    ///     parse("\\frac{1}{2}", &context),
    ///     Ast::Expression(
    ///         Factor::Fraction(
    ///             Box::new(Factor::Constant(1.0).into()),
    ///             Box::new(Factor::Constant(2.0).into()),
    ///         )
    ///         .into()
    ///     )
    /// );
    /// ```
    Fraction(Box<MathExpr>, Box<MathExpr>),
    /// Take the absolute value of an expression.
    /// ## Examples
    /// ```
    /// # use parser::ast::*;
    /// # use parser::token::Token;
    /// # use parser::prelude::MathContext;
    /// # use parser::prelude::_private::parse_sync_doc_test as parse;
    /// # let mut context=MathContext::standard_math();
    /// // parsing |3|
    /// assert_eq!(
    ///     parse("|3|", &context),
    ///     Ast::Expression(
    ///         Factor::Abs(Box::new(Factor::Constant(3.0).into())).into()
    ///     )
    /// );
    /// ```
    Abs(Box<MathExpr>),
    /// A Matrix
    ///
    /// ## Examples
    /// Vector matrixes:
    /// ```
    /// # use parser::ast::*;
    /// # use parser::matrix::Matrix;
    /// # use parser::token::Token;
    /// # use parser::prelude::MathContext;
    /// # use parser::prelude::_private::parse_sync_doc_test as parse;
    /// # let mut context=MathContext::standard_math();
    /// // parsing (1,1)
    /// assert_eq!(
    ///     parse("(1,1)", &context),
    ///     Ast::Expression(
    ///         Factor::Matrix(Matrix::new(
    ///             vec![
    ///                 Factor::Constant(1.0).into(),
    ///                 Factor::Constant(1.0).into()
    ///             ],
    ///             1,
    ///             2
    ///         ))
    ///         .into()
    ///     )
    /// );
    /// ```
    /// "Normal" matrix
    /// ```
    /// # use parser::ast::*;
    /// # use parser::matrix::Matrix;
    /// # use parser::token::Token;
    /// # use parser::prelude::MathContext;
    /// # use parser::prelude::_private::parse_sync_doc_test as parse;
    /// # let mut context=MathContext::standard_math();
    /// // parsing \begin{bmatrix}1\\1\end{bmatrix}
    /// assert_eq!(
    ///     parse("\\begin{bmatrix}1&1\\end{bmatrix}", &context),
    ///     Ast::Expression(
    ///         Factor::Matrix(Matrix::new(
    ///             vec![
    ///                 Factor::Constant(1.0).into(),
    ///                 Factor::Constant(1.0).into()
    ///             ],
    ///             1,
    ///             2
    ///         ))
    ///         .into()
    ///     )
    /// );
    /// ```
    /// Vmatrix means it is a determinant for witch absolute is wrapping the
    /// matrix
    /// ```
    /// # use parser::ast::*;
    /// # use parser::matrix::Matrix;
    /// # use parser::token::Token;
    /// # use parser::prelude::MathContext;
    /// # use parser::prelude::_private::parse_sync_doc_test as parse;
    /// # let mut context=MathContext::standard_math();
    /// // parsing \begin{Vmatrix}1\\1\end{Vmatrix}
    /// assert_eq!(
    ///     parse("\\begin{Vmatrix}1&1\\end{Vmatrix}", &context),
    ///     Ast::Expression(
    ///         Factor::Abs(Box::new(
    ///             Factor::Matrix(Matrix::new(
    ///                 vec![
    ///                     Factor::Constant(1.0).into(),
    ///                     Factor::Constant(1.0).into()
    ///                 ],
    ///                 1,
    ///                 2
    ///             ))
    ///             .into()
    ///         ))
    ///         .into()
    ///     )
    /// );
    /// ```
    Matrix(Matrix<MathExpr>),
}

/// A mathematical identifier, for example variable or function names.
///
/// Examples of valid math identifiers: "x", "x_1", "F_g", "\overline{v}".
#[derive(Eq, PartialEq, Debug, Hash, Clone)]
pub struct MathIdentifier {
    /// The tokens making up a identification
    /// can be multiple as _ is understood as a token
    pub tokens: Vec<Token>,
}

/// an identified function
#[derive(PartialEq, Debug, Clone)]
pub struct FunctionCall {
    ///The name for the function called
    pub function_name: MathIdentifier,
    ///The input to the function
    pub arguments: Vec<MathExpr>,
}
