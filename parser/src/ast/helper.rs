//! helper functions

use std::ops::{Deref, DerefMut};

use crate::{number_literal::NumberLiteral, prelude::*};

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
///combines the dependants into one vec
trait SimpleDependantDrain {
    ///drain dependants to one vec
    fn concat_dependant(&self) -> Vec<u64>;
}
impl<U: Simplify, V: Simplify> SimpleDependantDrain for (Simple<U>, Simple<V>) {
    fn concat_dependant(&self) -> Vec<u64> {
        [&self.0.dependents[..], &self.1.dependents[..]].concat()
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
        Simple::new_unchecked(
            Ast::Equality(self.0, self.1.value),
            self.1.dependents,
        )
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
        (&self.0.value, &self.1.value)
    }

    fn add(self) -> Simple<MathExpr> {
        let dependants = self.concat_dependant();
        Simple::new_unchecked(
            MathExpr::Add(self.0.value.boxed(), self.1.value),
            dependants,
        )
    }

    fn sub_wrapped(self) -> Simple<MathExpr> {
        let dependants = self.concat_dependant();
        Simple::new_unchecked(
            MathExpr::Subtract(self.0.boxed_inner(), self.1.inner()),
            dependants,
        )
    }
}
impl SimpleCompareEquivalent for (Simple<MathExpr>, Simple<Term>) {
    fn equivalent(&self, cont: &MathContext) -> bool {
        self.0
            .value
            .equivalent(&MathExpr::Term(self.1.value.clone()), cont)
    }
}
///This type if for comparing Simple<Term> to Simple<Factor> and returning
/// Simples, this makes sure that only correct simples can be constructed
pub trait SimpleCompareTerm {
    ///gets the math_exprs to compare
    fn to_expr(&self) -> (&Term, &Factor);
    ///multiplies the compared items and produces a Simple
    fn mul(self, m: MulType) -> Simple<Term>;
    ///divides them and produces a Simple
    fn div(self) -> Simple<Term>;
}
impl SimpleCompareTerm for (Simple<Term>, Simple<Factor>) {
    fn to_expr(&self) -> (&Term, &Factor) {
        (&self.0.value, &self.1.value)
    }

    fn mul(self, m: MulType) -> Simple<Term> {
        let dependants = self.concat_dependant();
        Simple::new_unchecked(
            Term::Multiply(m, self.0.value.boxed(), self.1.value),
            dependants,
        )
    }

    fn div(self) -> Simple<Term> {
        let dependants = self.concat_dependant();
        Simple::new_unchecked(
            Term::Divide(self.0.value.boxed(), self.1.value),
            dependants,
        )
    }
}
impl SimpleCompareEquivalent for (Simple<Term>, Simple<Factor>) {
    fn equivalent(&self, cont: &MathContext) -> bool {
        self.0
            .value
            .equivalent(&Term::Factor(self.1.value.clone()), cont)
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
        let dependants = self.concat_dependant();
        Simple::new_unchecked(
            Factor::Power {
                base: self.0.boxed_inner(),
                exponent: self.1.boxed_inner(),
            },
            dependants,
        )
    }
}
impl SimpleCompareEquivalent for (Simple<Factor>, Simple<MathExpr>) {
    fn equivalent(&self, cont: &MathContext) -> bool {
        self.1.value.equivalent(
            &MathExpr::Term(Term::Factor(self.0.value.clone())),
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
        let dependants = self.concat_dependant();
        Simple::new_unchecked(
            Factor::Root {
                degree: Some(self.0.value.boxed()),
                radicand: self.1.value.boxed(),
            },
            dependants,
        )
    }
}
///Simple is a wrapper struct only allowed to be constructed when the contained
/// MathExpr is in the simplest form
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Simple<T: Simplify> {
    ///The value that is simplified
    pub(crate) value: T,
    ///The dependants of the simplified values
    dependents: Vec<u64>,
}
impl<T: Simplify> Simple<T> {
    ///returns the inner T in a box
    pub fn boxed_inner(self) -> Box<T> {
        Box::new(self.inner())
    }
    /// Gets the dependants
    pub const fn dependant(&self) -> &Vec<u64> {
        &self.dependents
    }
    /// Gets the inner value with dependants
    pub fn destruct(self) -> (T, Vec<u64>) {
        (self.value, self.dependents)
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
    pub(super) const fn new_unchecked(value: T, dependants: Vec<u64>) -> Self {
        Self {
            value,
            dependents: dependants,
        }
    }
    ///gets a ref to inner item
    pub const fn ref_inner(&self) -> &T {
        &self.value
    }
    ///gets a ref to inner item
    pub fn inner(self) -> T {
        self.value
    }
}
impl<T: Simplify + Into<MathExpr>> Simple<T> {
    ///Constructs a Ast::Expression from the contained MathExpr
    pub fn expression(self) -> Simple<Ast> {
        Simple::new_unchecked(
            Ast::Expression(self.value.into()),
            self.dependents,
        )
    }
}
impl<T: Simplify> Deref for Simple<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<T: Simplify> DerefMut for Simple<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
impl From<Simple<Term>> for Simple<MathExpr> {
    fn from(simple: Simple<Term>) -> Self {
        Simple::new_unchecked(simple.value.into(), simple.dependents)
    }
}
impl From<Simple<Factor>> for Simple<MathExpr> {
    fn from(simple: Simple<Factor>) -> Self {
        Simple::new_unchecked(simple.value.into(), simple.dependents)
    }
}
impl From<Simple<Factor>> for Simple<Term> {
    fn from(simple: Simple<Factor>) -> Self {
        Simple::new_unchecked(simple.value.into(), simple.dependents)
    }
}
impl Simple<Factor> {
    ///adds 2 NumberLiterals and makes a Simple Constant
    pub fn add(lhs: &NumberLiteral, rhs: &NumberLiteral) -> Self {
        Simple::new_unchecked(Factor::Constant(lhs + rhs), Vec::new())
    }
    ///subtracts 2 NumberLiterals and makes a Simple Constant
    pub fn sub(lhs: &NumberLiteral, rhs: &NumberLiteral) -> Self {
        Simple::new_unchecked(Factor::Constant(lhs - rhs), Vec::new())
    }
    ///multiplies 2 NumberLiterals and makes a Simple Constant
    pub fn mul(lhs: &NumberLiteral, rhs: &NumberLiteral) -> Self {
        Simple::new_unchecked(Factor::Constant(lhs * rhs), Vec::new())
    }
    ///divide 2 NumberLiterals and makes a Simple Constant
    pub fn divide(
        numerator: &NumberLiteral,
        denominator: &NumberLiteral,
    ) -> Self {
        Simple::new_unchecked(
            Factor::Constant(numerator / denominator),
            Vec::new(),
        )
    }
    ///makes a Factor::Constant() containing the given constant
    pub fn constant(constant: impl Into<NumberLiteral>) -> Self {
        Simple::new_unchecked(Factor::Constant(constant.into()), Vec::new())
    }
    ///Puts a Variable into a Simple
    pub const fn variable(m: MathIdentifier) -> Self {
        Simple::new_unchecked(Factor::Variable(m), Vec::new())
    }
    ///Puts a functionCall into a Simple
    pub const fn native_function(f: FunctionCall) -> Self {
        Simple::new_unchecked(Factor::FunctionCall(f), Vec::new())
    }
    ///simplifies a matrix
    pub fn matrix(
        m: Matrix<MathExpr>,
        cont: &MathContext,
    ) -> Result<Self, EvalError> {
        let mut dependant = Vec::new();
        let matrix = m.map_owned(|v| {
            Ok({
                let (expr, dep) = v.simple(cont)?.destruct();
                dependant.extend(dep);
                expr
            })
        });
        Ok(Simple::new_unchecked(Factor::Matrix(matrix?), dependant))
    }
}
impl<T: ToLaTeX + Simplify> ToLaTeX for Simple<T> {
    fn to_latex(&self) -> String {
        self.value.to_latex()
    }
}
