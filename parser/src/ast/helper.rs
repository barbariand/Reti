//! helper functions

use std::ops::{Deref, DerefMut};

use crate::prelude::*;

use super::{equality::MathEquality, simplify::Simplify, to_latex::ToLaTeX};

impl MathExpr {
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
///This type if for comparing Simple<MathExpr>s and returning Simples, this
/// makes sure that only correct simples can be constructed///Trait for
/// comparing Simple<MathExpr>s
pub trait SimpleMathExprs {
    ///Gives as Ast::equals
    fn ast_equals(self) -> Simple<Ast>;
}
impl SimpleMathExprs for (MathExpr, Simple<MathExpr>) {
    fn ast_equals(self) -> Simple<Ast> {
        Simple(Ast::Equality(self.0, self.1 .0))
    }
}
///Trait for finding if they are equivalent
pub trait SimpleCompareEquivalent {
    ///checks if the contained are  the same
    fn equivalent(&self, cont: &MathContext) -> bool;
}
///This type if for comparing Simple<MathExpr> to Simple<Term> and returning
/// Simples, this makes sure that only correct simples can be constructed
pub trait SimpleCompareMathExpr {
    ///gets the math_exprs to compare
    fn to_expr(&self) -> (&MathExpr, &Term);
    ///adds the compared items and produces a Simple
    fn add(self) -> Simple<MathExpr>;
    ///subtracts the compared items and produces a Simple
    fn sub_wrapped(self) -> Simple<MathExpr>;
}
impl SimpleCompareMathExpr for (Simple<MathExpr>, Simple<Term>) {
    fn to_expr(&self) -> (&MathExpr, &Term) {
        (&self.0 .0, &self.1 .0)
    }

    fn add(self) -> Simple<MathExpr> {
        Simple(MathExpr::Add(self.0.boxed_inner(), self.1.inner()))
    }

    fn sub_wrapped(self) -> Simple<MathExpr> {
        Simple(MathExpr::Subtract(self.0.boxed_inner(), self.1.inner()))
    }
}
impl SimpleCompareEquivalent for (Simple<MathExpr>, Simple<Term>) {
    fn equivalent(&self, cont: &MathContext) -> bool {
        self.0
            .equivalent(&Simple(MathExpr::Term(self.1 .0.clone())), cont)
    }
}
///This type if for comparing Simple<Term> to Simple<Factor> and returning
/// Simples, this makes sure that only correct simples can be constructed
pub trait SimpleCompareTerm {
    ///gets the math_exprs to compare
    fn to_expr(&self) -> (&Term, &Factor);
    ///multiplies the compared items and produces a Simple
    fn mul_wrapped(self, m: MulType) -> Simple<Term>;
    ///divides them and produces a Simple
    fn div_wrapped(self) -> Simple<Term>;
}
impl SimpleCompareTerm for (Simple<Term>, Simple<Factor>) {
    fn to_expr(&self) -> (&Term, &Factor) {
        (&self.0 .0, &self.1 .0)
    }

    fn mul_wrapped(self, m: MulType) -> Simple<Term> {
        Simple(Term::Multiply(m, self.0.boxed_inner(), self.1.inner()))
    }

    fn div_wrapped(self) -> Simple<Term> {
        Simple(Term::Divide(self.0.boxed_inner(), self.1.inner()))
    }
}
impl SimpleCompareEquivalent for (Simple<Term>, Simple<Factor>) {
    fn equivalent(&self, cont: &MathContext) -> bool {
        self.0
            .equivalent(&Simple(Term::Factor(self.1 .0.clone())), cont)
    }
}
///This type if for comparing Simple<Factor> to Simple<MathExpr> and returning
/// Simples, this makes sure that only correct simples can be constructed
pub trait SimpleCompareFactor {
    ///gets the math_exprs to compare
    fn to_expr(&self) -> (&Factor, &MathExpr);
    ///pows them and produces a Simple
    fn pow_wrapped(self) -> Simple<Factor>;
}
impl SimpleCompareFactor for (Simple<Factor>, Simple<MathExpr>) {
    fn to_expr(&self) -> (&Factor, &MathExpr) {
        (&self.0, &self.1)
    }

    fn pow_wrapped(self) -> Simple<Factor> {
        Simple(Factor::Power {
            base: self.0.boxed_inner(),
            exponent: self.1.boxed_inner(),
        })
    }
}
impl SimpleCompareEquivalent for (Simple<Factor>, Simple<MathExpr>) {
    fn equivalent(&self, cont: &MathContext) -> bool {
        self.1.equivalent(
            &Simple(MathExpr::Term(Term::Factor(self.0 .0.clone()))),
            cont,
        )
    }
}
///This type if for comparing Simple<MathExpr> to Simple<MathExpr> and
/// returning Simples, this makes sure that only correct simples can be
/// constructed
pub trait SimpleCompareMultipleMathExprs {
    ///gets the math_exprs to compare
    fn to_expr(&self) -> (&MathExpr, &MathExpr);
    /// takes the root as simple
    fn root(self) -> Simple<Factor>;
}
impl SimpleCompareMultipleMathExprs for (Simple<MathExpr>, Simple<MathExpr>) {
    fn to_expr(&self) -> (&MathExpr, &MathExpr) {
        (&self.0, &self.1)
    }

    fn root(self) -> Simple<Factor> {
        Simple(Factor::Root {
            degree: Some(self.0.inner().boxed()),
            radicand: self.1.inner().boxed(),
        })
    }
}
///Simple is a wrapper struct only allowed to be constructed when the contained
/// MathExpr is in the simplest form
#[derive(Clone, Debug, PartialEq)]
pub struct Simple<T: Simplify>(pub(crate) T);
impl<T: Simplify> Simple<T> {
    ///returns the inner T in a box
    pub fn boxed_inner(self) -> Box<T> {
        Box::new(self.0)
    }
    ///Constructs a Simple from a MathExpr
    pub fn new(
        math_expr: T,
        cont: &MathContext,
    ) -> Result<Simple<T>, EvalError> {
        math_expr.simple(cont)
    }
    /// Construct a Simple without actually checking
    /// that it's simplified.
    pub const fn new_unchecked(expr: T) -> Self {
        Simple(expr)
    }
    ///gets a ref to inner item
    pub const fn ref_inner(&self) -> &T {
        &self.0
    }
    ///gets a ref to inner item
    pub fn inner(self) -> T {
        self.0
    }
}
impl<T: Simplify + Into<MathExpr>> Simple<T> {
    ///Constructs a Ast::Expression from the contained MathExpr
    pub fn expression(self) -> Ast {
        Ast::Expression(self.0.into())
    }
}
impl<T: Simplify> Deref for Simple<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: Simplify> DerefMut for Simple<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl From<Simple<Term>> for Simple<MathExpr> {
    fn from(value: Simple<Term>) -> Self {
        Simple(value.0.into())
    }
}
impl From<Simple<Factor>> for Simple<MathExpr> {
    fn from(value: Simple<Factor>) -> Self {
        Simple(value.0.into())
    }
}
impl From<Simple<Factor>> for Simple<Term> {
    fn from(value: Simple<Factor>) -> Self {
        Simple(value.0.into())
    }
}
impl Simple<Factor> {
    ///adds 2 f64s and makes a Simple Constant
    pub fn add(lhs: f64, rhs: f64) -> Self {
        Simple(Factor::Constant(lhs + rhs))
    }
    ///subtracts 2 f64s and makes a Simple Constant
    pub fn sub(lhs: f64, rhs: f64) -> Self {
        Simple(Factor::Constant(lhs - rhs))
    }
    ///multiplies 2 f64s and makes a Simple Constant
    pub fn mul(lhs: f64, rhs: f64) -> Self {
        Simple(Factor::Constant(lhs * rhs))
    }
    ///divide 2 f64s and makes a Simple Constant
    pub fn divide(numerator: f64, denominator: f64) -> Self {
        Simple(Factor::Constant(numerator / denominator))
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
    pub fn matrix(
        m: Matrix<MathExpr>,
        cont: &MathContext,
    ) -> Result<Self, EvalError> {
        Ok(Simple(Factor::Matrix(
            m.map_owned(|v| Ok(v.simple(cont)?.inner()))?,
        )))
    }
}
impl<T: ToLaTeX + Simplify> ToLaTeX for Simple<T> {
    fn to_latex(&self) -> String {
        self.0.to_latex()
    }
}
