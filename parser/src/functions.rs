//! How we can represent a function

///The inner type alias for the function to execute to find the value
pub type InnerMathFunction =
    Arc<dyn Fn(Vec<Value>) -> Result<Value, EvalError> + Send + Sync>;

///The inner type alias for the derivative function
pub type InnerDeriveFunction =
    Arc<dyn Fn(Vec<MathExpr>) -> Result<MathExpr, EvalError> + Send + Sync>;
use std::{fmt::Debug, sync::Arc};

use crate::{
    ast::{helper::Simple, simplify::Simplify},
    prelude::*,
};

#[derive(Clone)]
/// A native function that is implemented in rust
pub struct NativeFunction {
    ///The function to run
    approximate: InnerMathFunction,
    ///the amount of acceptable arguments
    arguments: usize,
    ///the derivation
    derivative: InnerDeriveFunction,
}
impl Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{NativeFunction}}, args:{}", self.arguments)
    }
}
impl NativeFunction {
    ///New native function
    pub(crate) const fn new(
        approximate: InnerMathFunction,
        arguments: usize,
        derivative: InnerDeriveFunction,
    ) -> Self {
        Self {
            approximate,
            arguments,
            derivative,
        }
    }
    ///Running the function
    pub fn run(&self, args: Vec<Value>) -> Result<Value, EvalError> {
        if args.len() != self.arguments {
            return Err(EvalError::ArgumentLengthMismatch {
                expected: vec![1],
                found: args.len(),
            });
        }
        (self.approximate)(args)
    }
    ///takes the derivative of the function
    fn derivate(&self, v: Vec<MathExpr>) -> Result<MathExpr, EvalError> {
        if v.len() != self.arguments {
            return Err(EvalError::ArgumentLengthMismatch {
                expected: vec![1],
                found: v.len(),
            });
        }
        (self.derivative)(v)
    }
}
///A user defined function
#[derive(Clone, Debug)]
pub struct ForeignFunction {
    ///the expression that is the foreign function
    pub expr: MathExpr,
    ///the inputs
    pub input: Vec<MathIdentifier>,
}
/// A MathFunction that can be run
#[derive(Clone, Debug)]
pub enum MathFunction {
    ///A native function that is implemented in rust
    Native(NativeFunction),
    ///A user defined function
    Foreign(ForeignFunction),
}

impl MathFunction {
    ///Helper new for native functions
    fn new_native(
        func: InnerMathFunction,
        arguments: usize,
        derivative: Option<InnerDeriveFunction>,
    ) -> Self {
        Self::Native(NativeFunction::new(
            func,
            arguments,
            derivative.unwrap_or(Arc::new(|_| {
                Err(DeriveError::All {
                    message: "can not derive this function".to_owned(),
                }
                .into())
            })),
        ))
    }
    /// Helper new for foreign functions
    pub const fn new_foreign(
        expr: MathExpr,
        input: Vec<MathIdentifier>,
    ) -> Self {
        Self::Foreign(ForeignFunction { expr, input })
    }
    ///Helper new for fn pointers
    pub fn from_fn_pointer(
        func: fn(Vec<Value>) -> Result<Value, EvalError>,
        arguments: usize,
        derivative: Option<InnerDeriveFunction>,
    ) -> Self {
        Self::new_native(Arc::new(func), arguments, derivative)
    }
    ///Helper new for fn pointers with f64s
    pub fn from_fn_pointer_expecting_scalars<
        F: Fn(Vec<f64>) -> f64 + Send + Sync + 'static,
    >(
        func: F,
        arguments: usize,
        derivative: Option<InnerDeriveFunction>,
    ) -> Self {
        Self::new_native(
            Arc::new(move |v: Vec<Value>| {
                let v_new: Result<Vec<f64>, EvalError> =
                    v.into_iter().map(|val| val.scalar()).collect();
                Ok(Value::Scalar(func(v_new?)))
            }),
            arguments,
            derivative,
        )
    }
    ///Helper function for creating a new when expecting single
    pub fn from_fn_pointer_expecting_single_scalar<
        F: Fn(f64) -> f64 + Send + Sync + 'static,
        D: Fn(MathExpr) -> Result<MathExpr, EvalError> + Send + Sync + 'static,
    >(
        func: F,
        arguments: usize,
        deriv: D,
    ) -> Self {
        Self::new_native(
            Arc::new(move |v: Vec<Value>| {
                let s = v[0].scalar()?;
                Ok(Value::Scalar(func(s)))
            }),
            arguments,
            Some(single_var_derivation_function(deriv)),
        )
    }
    ///Helper function for creating a new when expecting a single value and
    /// also the derive is none
    pub fn from_fn_pointer_expecting_single_scalar_without_derive<
        F: Fn(f64) -> f64 + Send + Sync + 'static,
    >(
        func: F,
        arguments: usize,
    ) -> Self {
        Self::new_native(
            Arc::new(move |v: Vec<Value>| {
                let s = v[0].scalar()?;
                Ok(Value::Scalar(func(s)))
            }),
            arguments,
            None,
        )
    }
    ///take the derivative of the function
    pub fn derivate(
        &self,
        val: Vec<MathExpr>,
        cont: &MathContext,
        _dependant: MathIdentifier,
    ) -> Result<Simple<MathExpr>, EvalError> {
        match self {
            MathFunction::Native(n) => {
                n.derivate(val).map(|v| v.simple(cont))?
            }
            MathFunction::Foreign(_f) => todo!(),
        }
    }
}
///helper function to be able to ensure the function only takes 1 argument
fn single_var_derivation_function<
    D: Fn(MathExpr) -> Result<MathExpr, EvalError> + Send + Sync + 'static,
>(
    func: D,
) -> Arc<dyn Fn(Vec<MathExpr>) -> Result<MathExpr, EvalError> + Send + Sync> {
    Arc::new(move |v: Vec<MathExpr>| {
        let len = v.len();
        if len != 1 {
            return Err(EvalError::ArgumentLengthMismatch {
                expected: vec![1],
                found: len,
            });
        }
        (func)(v[0].clone())
    })
}

/// The trait for easier managing of functions by automatically implementing it
/// for common functions of f64 and other types
///
/// Note that if this is only implemented for Fn(f64)->f64 not Fn(&f64)->f64
/// because a limitation in rusts compiler as they are seen as conflicting
pub trait IntoMathFunction {
    ///To convert to math function
    fn into_math_function(self) -> MathFunction;
}
impl<F> IntoMathFunction for (F, usize, Option<InnerDeriveFunction>)
where
    F: Fn(Vec<Value>) -> Result<Value, EvalError> + Send + Sync + 'static,
{
    fn into_math_function(self) -> MathFunction {
        MathFunction::new_native(Arc::new(self.0), self.1, self.2)
    }
}
impl IntoMathFunction for MathFunction {
    fn into_math_function(self) -> MathFunction {
        self
    }
}
impl<F: Fn(f64) -> f64, D: Fn(MathExpr) -> Result<MathExpr, EvalError>>
    IntoMathFunction for (F, D)
where
    F: Send + Sync + 'static,
    D: Send + Sync + 'static,
{
    fn into_math_function(self) -> MathFunction {
        MathFunction::from_fn_pointer_expecting_single_scalar(self.0, 1, self.1)
    }
}
impl<F: Fn(f64) -> f64> IntoMathFunction for F
where
    F: Send + Sync + 'static,
{
    fn into_math_function(self) -> MathFunction {
        MathFunction::from_fn_pointer_expecting_single_scalar_without_derive(
            self, 1,
        )
    }
}
