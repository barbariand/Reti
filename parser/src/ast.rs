use crate::token::Token;

pub mod mathexpr;

#[derive(PartialEq, Debug)]
pub struct Ast {
    pub root_expr: MathExpr,
}

#[derive(PartialEq, Debug)]
pub enum MathExpr {
    Term(Term),
    Add(Box<MathExpr>, Term),
    Subtract(Box<MathExpr>, Term),
}

impl From<Term> for MathExpr {
    fn from(value: Term) -> Self {
        MathExpr::Term(value)
    }
}
trait MathExprOperation {
    fn as_addition(self) -> MathExpr;
    fn as_subtraction(self) -> MathExpr;
}
impl MathExprOperation for (Box<MathExpr>, Term) {
    fn as_addition(self) -> MathExpr {
        MathExpr::Add(self.0, self.1)
    }

    fn as_subtraction(self) -> MathExpr {
        MathExpr::Subtract(self.0, self.1)
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

trait TermOperation {
    fn as_multiplication(self) -> Term;
    fn as_division(self) -> Term;
}

impl TermOperation for (Box<Term>, Factor) {
    fn as_multiplication(self) -> Term {
        Term::Multiply(self.0, self.1)
    }

    fn as_division(self) -> Term {
        Term::Divide(self.0, self.1)
    }
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

impl From<f64> for Factor {
    fn from(value: f64) -> Self {
        Factor::Constant(value)
    }
}
impl From<Box<MathExpr>> for Factor {
    fn from(value: Box<MathExpr>) -> Self {
        Factor::Expression(value)
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
#[derive(PartialEq, Debug)]
pub struct MathIdentifier {
    pub tokens: Vec<Token>,
}

#[derive(PartialEq, Debug)]
pub struct FunctionCall {
    pub function_name: MathIdentifier,
    pub arguments: Vec<MathExpr>,
}
