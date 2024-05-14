//! # Context
//! this module is for helping with keeping track of variables and functions
//! for that it uses MathContext where you can add any function or variable  
use std::{collections::HashMap, sync::Arc};
///The inner type alias for the function to execute to find the value
type InnerMathFunction = Arc<
    dyn Fn(Vec<Value>, &MathContext) -> Result<Value, EvalError> + Send + Sync,
>;
///The inner type alias for the derivative function
type InnerDeriveFunction =
    Arc<dyn Fn(Vec<MathExpr>) -> Result<MathExpr, EvalError> + Send + Sync>;
use crate::prelude::*;
/// A MathFunction that can be run
#[derive(Clone)]
pub struct MathFunction {
    ///The function to run
    approximate: InnerMathFunction,
    ///the amount of acceptable arguments
    arguments: usize,
    ///the function to get the derivative
    derivative: Option<InnerDeriveFunction>,
}

impl MathFunction {
    ///Helper new
    pub fn new(
        func: InnerMathFunction,
        arguments: usize,
        derivative: Option<InnerDeriveFunction>,
    ) -> Self {
        Self {
            approximate: func,
            arguments,
            derivative,
        }
    }
    ///Helper new for fn pointers
    pub fn from_fn_pointer(
        func: fn(Vec<Value>, &MathContext) -> Result<Value, EvalError>,
        arguments: usize,
        derivative: Option<InnerDeriveFunction>,
    ) -> Self {
        Self {
            approximate: Arc::new(func),
            arguments,
            derivative,
        }
    }
    ///Helper new for fn pointers with f64s
    pub fn from_fn_pointer_expecting_scalars<
        F: Fn(Vec<f64>) -> f64 + Send + Sync + 'static,
    >(
        func: F,
        arguments: usize,
        derivative: Option<InnerDeriveFunction>,
    ) -> Self {
        Self {
            approximate: Arc::new(move |v: Vec<Value>, _: &MathContext| {
                let v_new: Result<Vec<f64>, EvalError> =
                    v.into_iter().map(|val| val.scalar()).collect();
                Ok(Value::Scalar(func(v_new?)))
            }),
            arguments,
            derivative,
        }
    }
    ///Helper function for creating a new when expecting single
    pub fn from_fn_pointer_expecting_single_scalar<
        F: Fn(f64) -> f64 + Send + Sync + 'static,
    >(
        func: F,
        arguments: usize,
        derivative: Option<InnerDeriveFunction>,
    ) -> Self {
        Self {
            approximate: Arc::new(move |v: Vec<Value>, _: &MathContext| {
                let s = v[0].scalar()?;
                Ok(Value::Scalar(func(s)))
            }),
            arguments,
            derivative,
        }
    }
    /// run the function with given arguments
    pub fn eval(
        &self,
        vec: Vec<Value>,
        cont: &MathContext,
    ) -> Result<Value, EvalError> {
        if vec.len() != self.arguments {
            return Err(EvalError::ArgumentLengthMismatch {
                expected: vec![self.arguments],
                found: vec.len(),
            });
        }
        (self.approximate)(vec, cont)
    }
}
///The MathContext, holding all the functions and variables
#[derive(Clone)]
pub struct MathContext {
    ///The variables
    pub variables: HashMap<MathIdentifier, Value>,
    /// The functions defined in this math context
    pub functions: HashMap<MathIdentifier, MathFunction>,
}

impl Default for MathContext {
    fn default() -> Self {
        Self::new()
    }
}

impl MathContext {
    ///Creates a new empty MathContext
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    ///merging the functions available from a MathContext
    /// in case of collision it not mutate itself preferring to keep those
    /// values
    pub fn merge(&mut self, other: &MathContext) {
        other.variables.iter().for_each(|(key, value)| {
            self.variables.entry(key.clone()).or_insert(value.clone());
        });

        other.functions.iter().for_each(|(key, value)| {
            self.functions.entry(key.clone()).or_insert(value.clone());
        });
    }

    ///if the function is contained
    pub fn is_defined_function(&self, ident: &MathIdentifier) -> bool {
        // println!("is_function({:?}) = {}", ident, res); TODO tracing
        self.functions.contains_key(ident)
    }

    ///Adding a variable
    fn add_var(&mut self, identifier: Vec<Token>, value: Value) {
        self.variables
            .insert(MathIdentifier { tokens: identifier }, value);
    }

    ///Adding a function when it is IntoMathFunction
    pub fn add_function(
        &mut self,
        identifier: Vec<Token>,
        func: impl IntoMathFunction,
    ) {
        self.functions.insert(
            MathIdentifier { tokens: identifier },
            func.into_math_function(),
        );
    }

    ///The standard math
    /// Variables:
    /// * pi
    /// * e
    ///
    /// Functions:
    /// * sin
    /// * cos
    /// * tan
    /// * ln - natural log
    pub fn standard_math() -> MathContext {
        let mut context = MathContext::new();

        // Constants
        context.add_var(
            vec![Token::Backslash, Token::Identifier("pi".to_string())],
            Value::Scalar(std::f64::consts::PI),
        );
        context.add_var(
            vec![Token::Identifier("e".to_string())],
            Value::Scalar(std::f64::consts::E),
        );

        // TODO add proper functions system so we can define the definition
        //  and value sets to validate the amount of arguments, the types of
        // arguments  (scalar or matrix).

        // Trigonometric functions
        context.add_function(
            vec![Token::Backslash, Token::Identifier("sin".to_string())],
            (f64::sin, None),
        );
        context.add_function(
            vec![Token::Backslash, Token::Identifier("cos".to_string())],
            (f64::cos, None),
        );
        context.add_function(
            vec![Token::Backslash, Token::Identifier("tan".to_string())],
            (f64::tan, None),
        );

        // Logarithm
        context.add_function(
            vec![Token::Backslash, Token::Identifier("ln".to_string())],
            (f64::ln, None),
        );

        context
    }
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
    F: Fn(Vec<Value>, &MathContext) -> Result<Value, EvalError>
        + Send
        + Sync
        + 'static,
{
    fn into_math_function(self) -> MathFunction {
        MathFunction::new(Arc::new(self.0), self.1, self.2)
    }
}
impl IntoMathFunction for MathFunction {
    fn into_math_function(self) -> MathFunction {
        self
    }
}
impl<F: Fn(f64) -> f64> IntoMathFunction for (F, Option<InnerDeriveFunction>)
where
    F: Send + Sync + 'static,
{
    fn into_math_function(self) -> MathFunction {
        MathFunction::from_fn_pointer_expecting_single_scalar(self.0, 1, self.1)
    }
}

#[cfg(test)]
mod test {
    use snafu::whatever;

    #[allow(unused_imports)]
    use crate::prelude::*;

    #[test]
    pub fn merging_functions() {
        let mut c = MathContext::new();
        c.add_function(
            vec![Token::Backslash, Token::Identifier("nothing".to_owned())],
            (|v: f64| v, None),
        );
        let mut c1 = MathContext::new();
        c1.merge(&c);
        assert!(c1.is_defined_function(&MathIdentifier::new(vec![
            Token::Backslash,
            Token::Identifier("nothing".to_owned())
        ])))
    }
    #[test]
    pub fn overloading_functions() {
        let mut c2 = MathContext::new();
        c2.add_function(
            vec![Token::Backslash, Token::Identifier("nothing".to_owned())],
            (|v: f64| v, None),
        );
        let mut c1 = MathContext::new();
        c1.add_function(
            vec![Token::Backslash, Token::Identifier("nothing".to_owned())],
            (|_, _: &MathContext| whatever!("testing"), 1, None),
        );
        c1.merge(&c2);
        assert!((c1.functions[&MathIdentifier::new(vec![
            Token::Backslash,
            Token::Identifier("nothing".to_owned())
        ])]
            .approximate)(vec![Value::Scalar(1.1)], &c1)
        .is_err());
    }
}
