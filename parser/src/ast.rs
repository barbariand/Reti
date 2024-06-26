use crate::prelude::Token;

#[derive(PartialEq, Debug)]
pub enum Ast {
    Expression(MathExpr),
    Equality(MathExpr, MathExpr),
}

#[derive(PartialEq, Debug)]
pub enum MathExpr {
    Term(Term),
    Add(Box<MathExpr>, Term),
    Subtract(Box<MathExpr>, Term),
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

#[derive(PartialEq, Debug)]
pub enum Term {
    Factor(Factor),
    Multiply(Box<Term>, Factor),
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

#[derive(PartialEq, Debug)]
pub enum Factor {
    Constant(f64),
    Parenthesis(Box<MathExpr>),
    Variable(MathIdentifier),
    FunctionCall(FunctionCall),
    Power {
        base: Box<Factor>,
        exponent: Box<MathExpr>,
    },
    Root {
        degree: Option<Box<MathExpr>>,
        radicand: Box<MathExpr>,
    },
    Fraction(Box<MathExpr>, Box<MathExpr>),
    Abs(Box<MathExpr>),
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
    pub tokens: Vec<Token>,
}

#[derive(PartialEq, Debug)]
pub struct FunctionCall {
    pub function_name: MathIdentifier,
    pub arguments: Vec<MathExpr>,
}
