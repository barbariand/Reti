use crate::token::Token;

pub mod mathexpr;

#[derive(PartialEq, Debug)]
pub struct AST {
    pub root_expr: MathExpr,
}

#[derive(PartialEq, Debug)]
pub enum MathExpr {
    Term(Term),
    Add(Box<MathExpr>, Term),
    Subtract(Box<MathExpr>, Term),
}

#[derive(PartialEq, Debug)]
pub enum Term {
    Factor(Factor),
    Multiply(Box<Term>, Factor),
    Divide(Box<Term>, Factor),
}

#[derive(PartialEq, Debug)]
pub enum Factor {
    Constant(f64),
    Expression(Box<MathExpr>),
    Variable(MathIdentifier),
    FunctionCall(FunctionCall),
    Exponent {
        base: Box<Factor>,
        exponent: Box<MathExpr>,
    },
    Root {
        degree: Option<Box<MathExpr>>,
        radicand: Box<MathExpr>,
    },
    Abs(Box<MathExpr>),
}

/// A mathematical identifier, for example variable or function names.
///
/// Examples of valid math identifiers: "x", "x_1", "F_g", "\overline{v}".
#[derive(PartialEq, Debug)]
pub struct MathIdentifier {
    pub tokens: Vec<Token>,
}

#[derive(PartialEq, Debug)]
pub struct FunctionCall {
    pub function_name: MathIdentifier,
    pub arguments: Vec<MathExpr>,
}
