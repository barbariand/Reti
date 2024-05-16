//! helper functions

use std::ops::Deref;

use crate::prelude::*;

use super::{equality::PrivateMathEquality, simplify::Simplify};

impl MathExpr {
    ///makes a new MathExpr where the term part is wrapped if needed
    fn add_wrapped(a: MathExpr, b: MathExpr) -> Self {
        Self::Add(a.boxed(), b.get_term_or_wrap())
    }
    ///makes a new MathExpr where the term part is wrapped if needed
    fn subtract_wrapped(a: MathExpr, b: MathExpr) -> Self {
        Self::Subtract(a.boxed(), b.get_term_or_wrap())
    }
    ///gets the term or wraps it in parenthesis
    pub fn get_term_or_wrap(&self) -> Term {
        match self {
            MathExpr::Term(t) => t.clone(),
            _ => Factor::Parenthesis(self.clone().boxed()).into(),
        }
    }
    ///gets the factor or wraps it in parenthesis
    pub fn get_factor_or_wrap(&self) -> Factor {
        match self {
            MathExpr::Term(Term::Factor(f)) => f.clone(),
            _ => Factor::Parenthesis(self.clone().boxed()),
        }
    }
    ///Gets a factor if it is a factor otherwise None
    pub fn factor(&self) -> Option<&Factor> {
        match self {
            MathExpr::Term(Term::Factor(f)) => Some(f),
            _ => None,
        }
    }
    /// gets a term if it is a term otherwise None
    pub fn term(&self) -> Option<&Term> {
        match self {
            MathExpr::Term(t) => Some(t),
            _ => None,
        }
    }

    ///Boxes self
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

impl Term {
    ///Boxes self
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
    ///does multiplication but wraps if need be
    fn mul_wrapped(mul: MulType, a: MathExpr, b: MathExpr) -> Self {
        Self::Multiply(
            mul,
            a.get_term_or_wrap().boxed(),
            b.get_factor_or_wrap(),
        )
    }
    ///does division but wraps if need be
    fn div_wrapped(a: MathExpr, b: MathExpr) -> Self {
        Self::Divide(a.get_term_or_wrap().boxed(), b.get_factor_or_wrap())
    }
    ///Gets a factor if it is a factor otherwise None
    pub fn factor(&self) -> Option<&Factor> {
        match self {
            Term::Factor(f) => Some(f),
            _ => None,
        }
    }
}

impl Factor {
    ///Boxes self
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

impl MathIdentifier {
    ///Creates a new MathIdentifier fom a vec to identify a variable and
    /// function
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens }
    }
    ///Creates a new MathIdentifier from a single Token to identify a variable
    /// and a function
    pub fn new_from_one(token: Token) -> Self {
        Self {
            tokens: vec![token],
        }
    }
    ///# Warning
    /// does no conversion or latex translation
    pub fn from_single_ident(s: &str) -> Self {
        Self {
            tokens: vec![Token::Identifier(s.to_owned())],
        }
    }
}

impl FunctionCall {
    ///a helper method
    pub fn new(
        function_name: MathIdentifier,
        arguments: Vec<MathExpr>,
    ) -> Self {
        Self {
            function_name,
            arguments,
        }
    }
}
///Helper trait for comparing f64
pub(crate) trait NumberCompare {
    ///if it is zero
    fn is_zero(&self) -> bool;
    ///if it is one
    fn is_one(&self) -> bool;
    ///if it equals the other
    #[allow(dead_code)]
    fn equals(&self, other: &Self) -> bool;
}
impl NumberCompare for f64 {
    fn is_zero(&self) -> bool {
        self.abs() < f64::EPSILON
    }

    fn is_one(&self) -> bool {
        (self - 1.0).abs() < f64::EPSILON
    }

    fn equals(&self, other: &Self) -> bool {
        (self - other) < f64::EPSILON
    }
}
///This type if for comparing Simples and returning Simples, this makes sure
/// that only correct simples can be constructed
pub trait SimpleCompare {
    fn to_math_expr(&self)->(&MathExpr,&MathExpr);
    fn add_wrapped(self)->Simple;
    fn sub_wrapped(self)->Simple;
    fn mul_wrapped(self,m:MulType)->Simple;
    fn div_wrapped(self)->Simple;
    fn pow_wrapped(self)->Simple;
    fn ast_equals(self)->Ast;
    fn ast_expr(self)->Ast;
    fn symmetrical(&self)->bool;
}
impl SimpleCompare for (Simple,Simple) {
    fn to_math_expr(&self)->(&MathExpr,&MathExpr){
        (&*self.0,&*self.1)
    }
    fn add_wrapped(self)->Simple {
        Simple(MathExpr::add_wrapped(self.0.0, self.1.0))
    }

    fn sub_wrapped(self)->Simple {
        Simple(MathExpr::subtract_wrapped(self.0.0, self.1.0))
    }

    fn mul_wrapped(self,m:MulType)->Simple {
        Simple(MathExpr::Term(Term::mul_wrapped(m, self.0.0, self.1.0)))
    }

    fn div_wrapped(self)->Simple {
        Simple(MathExpr::Term(Term::div_wrapped(self.0.0, self.1.0)))
    }

    fn pow_wrapped(self)->Simple {
        Simple(MathExpr::Term(Term::Factor(Factor::Power{base: self.0.0.get_factor_or_wrap().boxed(),exponent: self.1.0.boxed()})))
    }

    fn ast_equals(self)->Ast {
        todo!()
    }

    fn ast_expr(self)->Ast {
        todo!()
    }
    
    fn symmetrical(&self)->bool {
        self.0.equals(&self.1)
    }
}

///Simple is a wrapper struct only allowed to be constructed when the contained
/// MathExpr is in the simplest form
#[derive(Clone)]
pub struct Simple(MathExpr);
impl Deref for Simple {
    type Target=MathExpr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Simple {
    ///Constructs a Simple from a MathExpr
    pub fn new(math_expr: MathExpr) -> Simple {
        math_expr.simple()
    }
    ///Constructs a Ast::Expression from the contained MathExpr
    pub fn expression(&self) -> Ast {
        Ast::Expression(self.0.clone())
    }
    ///gets a ref to inner item
    pub fn math_expr(&self) -> &MathExpr {
        &self.0
    }
    ///gets as a Term if it is one
    pub fn get_term(&self) -> Option<Term> {
        match &self.0 {
            MathExpr::Term(t) => Some(t.clone()),
            _ => None,
        }
    }
    ///gets as Factor if it is one
    pub fn get_factor(&self) -> Option<Factor> {
        match &self.0 {
            MathExpr::Term(Term::Factor(f)) => Some(f.clone()),
            _ => None,
        }
    }
    ///adds 2 f64s and makes a Simple Constant
    pub fn add(lhs: f64, rhs: f64) -> Self {
        Simple(Factor::Constant(lhs + rhs).into())
    }
    ///subtracts 2 f64s and makes a Simple Constant
    pub fn sub(lhs: f64, rhs: f64) -> Self {
        Simple(Factor::Constant(lhs - rhs).into())
    }
    ///multiplies 2 f64s and makes a Simple Constant
    pub fn mul(lhs: f64, rhs: f64) -> Self {
        Simple(Factor::Constant(lhs * rhs).into())
    }
    ///divide 2 f64s and makes a Simple Constant
    pub fn divide(lhs: f64, rhs: f64) -> Self {
        Simple(Factor::Constant(lhs / rhs).into())
    }
    ///makes a Factor::Constant() containing the given constant
    pub fn constant(constant: f64) -> Simple {
        Simple(MathExpr::Term(Term::Factor(Factor::Constant(constant))))
    }
    ///Puts a Variable into a Simple
    pub fn variable(m: MathIdentifier) -> Simple {
        Simple(MathExpr::Term(Term::Factor(Factor::Variable(m))))
    }
    ///Puts a functionCall into a Simple
    pub fn function(f: FunctionCall) -> Simple {
        Simple(MathExpr::Term(Term::Factor(Factor::FunctionCall(f))))
    }
    
}
