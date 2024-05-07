use std::{collections::HashMap, sync::Arc};

use crate::prelude::*;

#[derive(Clone)]
pub struct MathFunction {
    pub approximate: Arc<
        dyn Fn(Vec<Value>, MathContext) -> Result<Value, EvalError>
            + Send
            + Sync,
    >,
}

impl MathFunction {
    pub fn new(
        func: Arc<
            dyn Fn(Vec<Value>, MathContext) -> Result<Value, EvalError>
                + Send
                + Sync,
        >,
    ) -> Self {
        Self { approximate: func }
    }
    pub fn from_fn_pointer(
        func: fn(Vec<Value>, MathContext) -> Result<Value, EvalError>,
    ) -> Self {
        Self {
            approximate: Arc::new(func),
        }
    }
}
#[derive(Clone)]
pub struct MathContext {
    pub variables: HashMap<MathIdentifier, Value>,
    pub functions: HashMap<MathIdentifier, MathFunction>,
}

impl Default for MathContext {
    fn default() -> Self {
        Self::new()
    }
}

impl MathContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }
    //this will be non overriding
    pub fn merge(&mut self, other: &MathContext) {
        other.variables.iter().for_each(|(key, value)| {
            self.variables.entry(key.clone()).or_insert(value.clone());
        });

        other.functions.iter().for_each(|(key, value)| {
            self.functions.entry(key.clone()).or_insert(value.clone());
        });
    }

    pub fn is_defined_function(&self, ident: &MathIdentifier) -> bool {
        // println!("is_function({:?}) = {}", ident, res); TODO tracing
        self.functions.contains_key(ident)
    }

    fn add_var(&mut self, identifier: Vec<Token>, value: Value) {
        self.variables
            .insert(MathIdentifier { tokens: identifier }, value);
    }

    fn add_function(
        &mut self,
        identifier: Vec<Token>,
        func: impl IntoMathFunction,
    ) {
        self.functions
            .insert(MathIdentifier { tokens: identifier }, func.into());
    }

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
            MathFunction::from_fn_pointer(|args, _| {
                args[0].map_expecting_scalar(|v| v.sin())
            }),
        );
        context.add_function(
            vec![Token::Backslash, Token::Identifier("cos".to_string())],
            MathFunction::from_fn_pointer(|args, _| {
                args[0].map_expecting_scalar(|v| v.cos())
            }),
        );
        context.add_function(
            vec![Token::Backslash, Token::Identifier("tan".to_string())],
            MathFunction::from_fn_pointer(|args, _| {
                args[0].map_expecting_scalar(|v| v.tan())
            }),
        );

        // Logarithm
        context.add_function(
            vec![Token::Backslash, Token::Identifier("ln".to_string())],
            MathFunction::from_fn_pointer(|args, _| {
                args[0].map_expecting_scalar(|v| v.ln())
            }),
        );

        context
    }
}
trait IntoMathFunction {
    fn into(self) -> MathFunction;
}
impl IntoMathFunction for MathFunction {
    fn into(self) -> MathFunction {
        self
    }
}
impl<F> IntoMathFunction for F
where
    F: Fn(Vec<Value>, MathContext) -> Result<Value, EvalError>
        + Send
        + Sync
        + 'static,
{
    fn into(self) -> MathFunction {
        MathFunction::new(Arc::new(self))
    }
}

mod test {
    use snafu::whatever;

    #[allow(unused_imports)]
    use crate::prelude::*;

    #[test]
    pub fn merging_functions() {
        let mut c = MathContext::new();
        c.add_function(
            vec![Token::Backslash, Token::Identifier("nothing".to_owned())],
            |v: Vec<Value>, _: MathContext| Ok(v[0].clone()),
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
            |v: Vec<Value>, _: MathContext| Ok(v[0].clone()),
        );
        let mut c1 = MathContext::new();
        c1.add_function(
            vec![Token::Backslash, Token::Identifier("nothing".to_owned())],
            |_, _| whatever!("testing"),
        );
        c1.merge(&c2);
        assert!((c1.functions[&MathIdentifier::new(vec![
            Token::Backslash,
            Token::Identifier("nothing".to_owned())
        ])]
            .approximate)(vec![Value::Scalar(1.1)], c1.clone())
        .is_err());
    }
}
