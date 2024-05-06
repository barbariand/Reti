//! AST for representing Latex
use crate::prelude::*;

///The root of the AST that is non recursive
#[derive(PartialEq, Debug)]
pub enum Ast {
    ///A simple expression with no equals
    Expression(MathExpr),
    /// One equals for assigning
    Equality(MathExpr, MathExpr),
}

/// The recursive part of the AST containing subtraction and addition to make
/// the math rules enforced by the type system
#[derive(PartialEq, Debug, Clone)]
pub enum MathExpr {
    /// A [Term] containing the rest of the syntax that go before in evaluation
    Term(Term),
    /// Adding a MathExpr to a Term
    ///  ## Examples
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
    Add(Box<MathExpr>, Term),
    /// Subtracting a MathExpr from a Term
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
impl From<Term> for MathExpr {
    fn from(value: Term) -> Self {
        MathExpr::Term(value)
    }
}

impl From<Factor> for MathExpr {
    fn from(value: Factor) -> Self {
        MathExpr::Term(Term::Factor(value))
    }
}
impl From<f64> for MathExpr {
    fn from(value: f64) -> Self {
        MathExpr::Term(Term::from(value))
    }
}
impl From<f64> for Box<MathExpr> {
    fn from(value: f64) -> Self {
        Box::new(value.into())
    }
}
impl From<FunctionCall> for MathExpr {
    fn from(value: FunctionCall) -> Self {
        MathExpr::Term(Term::from(value))
    }
}
impl From<MathIdentifier> for MathExpr {
    fn from(value: MathIdentifier) -> Self {
        MathExpr::Term(Term::from(value))
    }
}
///For multiplication and division
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
    ///             Box::new(Term::Factor(
    ///                 Factor::Constant(2.0)
    ///             )),
    ///             Factor::Constant(2.0)
    ///         ).into()
    ///     )
    /// );

    /// ```
    Multiply(Box<Term>, Factor),
    ///Division of Term and Factor
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

impl From<Factor> for Term {
    fn from(value: Factor) -> Self {
        Self::Factor(value)
    }
}
impl From<f64> for Term {
    fn from(value: f64) -> Self {
        Term::Factor(Factor::Constant(value))
    }
}
impl From<MathIdentifier> for Term {
    fn from(value: MathIdentifier) -> Self {
        Term::Factor(Factor::Variable(value))
    }
}
impl From<FunctionCall> for Term {
    fn from(value: FunctionCall) -> Self {
        Term::Factor(Factor::FunctionCall(value))
    }
}
///The factor containing
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
    ///     parse("1", &context),
    ///     Ast::Expression(
    ///         Factor::Constant(1.0).into()
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
    ///         Factor::Parenthesis(
    ///             Box::new(
    ///                 Factor::Constant(1.0).into()
    ///             )
    ///         ).into()
    ///     )
    /// );

    /// ```
    Parenthesis(Box<MathExpr>),
    /// A Variable that is hopefully defined
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
    /// A Function that is hopefully defined
    /// ## Examples
    /// ```
    /// # use parser::ast::*;
    /// # use parser::token::Token;
    /// # use parser::prelude::MathContext;
    /// # use parser::prelude::MathFunction;
    /// # use parser::prelude::_private::parse_sync_doc_test as parse;
    /// # use std::sync::Arc;
    /// # use parser::value::Value;
    /// # let mut context=MathContext::standard_math();
    /// # context.functions.insert(MathIdentifier::new(vec![Token::Identifier("f".to_owned())]), MathFunction::new(Arc::new(|_,_|Ok(Value::Scalar(2.0)))));
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
    /// To the power of
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
    /// A Fraction
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
    /// Absolute
    /// ## Examples
    /// ```
    /// # use parser::ast::*;
    /// # use parser::token::Token;
    /// # use parser::prelude::MathContext;
    /// # use parser::prelude::_private::parse_sync_doc_test as parse;
    /// # let mut context=MathContext::standard_math();
    /// // parsing |1|
    /// assert_eq!(
    ///     parse("|1|", &context),
    ///     Ast::Expression(Factor::Abs(Box::new(Factor::Constant(1.0).into())).into())
    /// );
    /// ```
    Abs(Box<MathExpr>),
    /// A Function that is hopefully defined
    /// ## Examples
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
    ///             vec![Factor::Constant(1.0).into(), Factor::Constant(1.0).into()],
    ///             1,
    ///             2
    ///         ))
    ///         .into()
    ///     )
    /// );
    /// ```
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
    ///             vec![Factor::Constant(1.0).into(), Factor::Constant(1.0).into()],
    ///             1,
    ///             2
    ///         ))
    ///         .into()
    ///     )
    /// );
    /// ```
    /// ```
    /// # use parser::ast::*;
    /// # use parser::matrix::Matrix;
    /// # use parser::token::Token;
    /// # use parser::prelude::MathContext;
    /// # use parser::prelude::_private::parse_sync_doc_test as parse;
    /// # let mut context=MathContext::standard_math();
    /// // parsing \begin{bmatrix}1\\1\end{bmatrix}
    /// assert_eq!(
    ///     parse("\\begin{Vmatrix}1&1\\end{Vmatrix}", &context),
    ///     Ast::Expression(
    ///         Factor::Abs(Box::new(
    ///             Factor::Matrix(Matrix::new(
    ///                 vec![Factor::Constant(1.0).into(), Factor::Constant(1.0).into()],
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
impl From<f64> for Factor {
    fn from(value: f64) -> Self {
        Factor::Constant(value)
    }
}
impl From<MathIdentifier> for Factor {
    fn from(value: MathIdentifier) -> Self {
        Factor::Variable(value)
    }
}
impl From<FunctionCall> for Factor {
    fn from(value: FunctionCall) -> Self {
        Factor::FunctionCall(value)
    }
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

impl MathIdentifier {
    ///Creates a new MathIdentifier fom a vec to identify a variable and
    /// function
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens }
    }
    ///Creates a new MathIdentifier from a single Token to idetify a variable
    /// and a function
    pub fn new_from_one(token: Token) -> Self {
        Self {
            tokens: vec![token],
        }
    }
}
///A parsed function call where it has found a function with that name
#[derive(PartialEq, Debug, Clone)]
pub struct FunctionCall {
    ///The name for the function called
    pub function_name: MathIdentifier,
    ///The input to the function
    pub arguments: Vec<MathExpr>,
}

impl FunctionCall {
    ///Creating a new Functioncall
    pub fn new(function_name: MathIdentifier, arguments: Vec<MathExpr>) -> Self {
        Self {
            function_name,
            arguments,
        }
    }
}
