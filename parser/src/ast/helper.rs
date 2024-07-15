//! helper functions

use std::ops::Deref;

use crate::prelude::*;

use super::{equality::MathEquality, simplify::Simplify};

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
    pub const fn factor(&self) -> Option<&Factor> {
        match self {
            MathExpr::Term(Term::Factor(f)) => Some(f),
            _ => None,
        }
    }
    /// gets a term if it is a term otherwise None
    pub const fn term(&self) -> Option<&Term> {
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
    pub const fn factor(&self) -> Option<&Factor> {
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
    pub const fn new(tokens: Vec<Token>) -> Self {
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
    pub const fn new(
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
        (self - other).abs() < f64::EPSILON
    }
}

pub trait SimpleCompareEquivalent{
    ///checks if the contained are  the same
    fn equivalent(&self, cont: &MathContext) -> bool;
}
///This type if for comparing Simples and returning Simples, this makes sure
/// that only correct simples can be constructed
pub trait SimpleCompareMathExpr {
    ///Gives as Ast::equals
    fn ast_equals(self) -> Ast;
    ///gets the math_exprs to compare
    fn to_expr(&self) -> (&MathExpr, &Term);
    ///adds the compared items and produces a Simple
    fn add(self) -> Simple<MathExpr>;
    ///subtracts the compared items and produces a Simple
    fn sub_wrapped(self) -> Simple<MathExpr>;
}
impl SimpleCompareMathExpr for (Simple<MathExpr>,Simple<Term>){
    fn to_expr(&self) -> (&MathExpr, &Term) {
        (&self.0.0,&self.1.0)
    }

    fn add(self) -> Simple<MathExpr> {
        Simple(MathExpr::Add(self.0.boxed_inner(), self.1.expr()))
    }

    fn sub_wrapped(self) -> Simple<MathExpr> {
        Simple(MathExpr::Subtract(self.0.boxed_inner(), self.1.expr()))
    }
    
    fn ast_equals(self) -> Ast {
        todo!()
    }
}
impl SimpleCompareEquivalent for (Simple<MathExpr>,Simple<Term>){
    fn equivalent(&self, cont: &MathContext) -> bool {
        todo!("IDK MAN")
    }
}
pub trait SimpleCompareFactor{
    ///gets the math_exprs to compare
    fn to_expr(&self) -> (&Term, &Factor);
    ///multiplies the compared items and produces a Simple
    fn mul_wrapped(self, m: MulType) -> Simple<Term>;
    ///divides them and produces a Simple
    fn div_wrapped(self) -> Simple<Term>;
}
impl SimpleCompareFactor for (Simple<Term>,Simple<Factor>){
    fn to_expr(&self) -> (&Term, &Factor) {
        (&self.0.0,&self.1.0)
    }

    fn mul_wrapped(self, m: MulType) -> Simple<Term> {
        Simple(Term::Multiply(m, self.0.boxed_inner(), self.1.expr()))
    }

    fn div_wrapped(self) -> Simple<Term> {
        Simple(Term::Divide( self.0.boxed_inner(), self.1.expr()))
    }
}
impl SimpleCompareEquivalent for (Simple<Term>,Simple<Factor>){
    fn equivalent(&self, cont: &MathContext) -> bool {
        todo!("IDK MAN")
    }
}
pub trait SimpleCompareTerm{
    ///gets the math_exprs to compare
    fn to_expr(&self) -> (&Factor, &MathExpr);
    ///pows them and produces a Simple
    fn pow_wrapped(self) -> Simple<Factor>;
}
impl SimpleCompareTerm for (Simple<Factor>,Simple<MathExpr>){
    fn to_expr(&self) -> (&Factor, &MathExpr) {
        (&self.0,&self.1)
    }

    fn pow_wrapped(self) -> Simple<Factor> {
        Simple(Factor::Power { base: self.0.boxed_inner(), exponent: self.1.boxed_inner() })
    }
}
impl SimpleCompareEquivalent for (Simple<Factor>,Simple<MathExpr>){
    fn equivalent(&self, cont: &MathContext) -> bool {
        todo!("IDK MAN")
    }
}
///Simple is a wrapper struct only allowed to be constructed when the contained
/// MathExpr is in the simplest form
#[derive(Clone, Debug, PartialEq)]
pub struct Simple<T:Simplify+Into<MathExpr>>(pub(crate) T);
impl<T:Simplify+Into<MathExpr>> Simple<T> {
    pub fn boxed_inner(self)->Box<T>{
        Box::new(self.0)
    }
    ///Constructs a Simple from a MathExpr
    pub fn new(
        math_expr: T,
        cont: &MathContext,
    ) -> Result<Simple<T>, EvalError> {
        math_expr.simple(cont)
    }
    ///Constructs a Ast::Expression from the contained MathExpr
    pub fn expression(self) -> Ast {
        Ast::Expression(self.0.into())
    }
    ///gets a ref to inner item
    pub const fn inner(&self) -> &T {
        &self.0
    }
    ///gets a ref to inner item
    pub fn expr(self) -> T {
        self.0
    }
    
}
impl From<Simple<Term>> for Simple<MathExpr>{
    fn from(value: Simple<Term>) -> Self {
        Simple(value.0.into())
    }
}
impl From<Simple<Factor>> for Simple<MathExpr>{
    fn from(value: Simple<Factor>) -> Self {
        Simple(value.0.into())
    }
}
impl From<Simple<Factor>> for Simple<Term>{
    fn from(value: Simple<Factor>) -> Self {
        Simple(value.0.into())
    }
}
impl Simple<Factor>{
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
    pub fn divide(numerator: f64, denominator: f64) -> Self {
        Simple(Factor::Constant(numerator / denominator).into())
    }
    ///makes a Factor::Constant() containing the given constant
    pub const fn constant(constant: f64) -> Self {
        Simple(Factor::Constant(constant))
    }
    ///Puts a Variable into a Simple
    pub const fn variable(m: MathIdentifier) -> Self {
        Simple(Factor::Variable(m))
    }
    ///Puts a functionCall into a Simple
    pub const fn function(f: FunctionCall) -> Self {
        Simple(Factor::FunctionCall(f))
    }
    ///simplifies a matrix
    pub fn matrix(m:Matrix<MathExpr>,cont:&MathContext)->Result<Self, EvalError>{
        Ok(Simple(Factor::Matrix(m.map_owned(|v|Ok(v.simple(cont)?.expr()))?).into()))
    }
}


impl AsRef<MathExpr> for Simple<MathExpr> {
    fn as_ref(&self) -> &MathExpr {
        &self.0
    }
}

impl Deref for Simple<MathExpr> {
    type Target = MathExpr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Deref for Simple<Term> {
    type Target = Term;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Deref for Simple<Factor> {
    type Target = Factor;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
