use std::{collections::HashMap, sync::Arc};

use crate::prelude::*;

#[derive(Clone)]
pub struct MathFunction {
    pub approximate: Arc<dyn Fn(Vec<Value>, MathContext) -> Result<Value, EvalError> + Send + Sync>,
}

impl MathFunction {
    pub fn new(
        func: Arc<dyn Fn(Vec<Value>, MathContext) -> Result<Value, EvalError> + Send + Sync>,
    ) -> Self {
        Self { approximate: func }
    }
    pub fn from_fn_pointer(func: fn(Vec<Value>, MathContext) -> Result<Value, EvalError>) -> Self {
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

    pub fn merge(&mut self, other: &MathContext) {
        for (variable, value) in other.variables.iter() {
            self.variables.insert(variable.clone(), value.clone());
        }
        for (name, value) in other.functions.iter() {
            self.functions.insert(name.clone(), value.clone());
        }
    }

    pub fn is_function(&self, ident: &MathIdentifier) -> bool {
        // println!("is_function({:?}) = {}", ident, res); TODO tracing
        self.functions.contains_key(ident)
    }

    fn add_var(&mut self, identifier: Vec<Token>, value: Value) {
        self.variables
            .insert(MathIdentifier { tokens: identifier }, value);
    }

    fn add_function(&mut self, identifier: Vec<Token>, func: MathFunction) {
        self.functions
            .insert(MathIdentifier { tokens: identifier }, func);
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
        //  and value sets to validate the amount of arguments, the types of arguments
        //  (scalar or matrix).

        // Trigonometric functions
        context.add_function(
            vec![Token::Backslash, Token::Identifier("sin".to_string())],
            MathFunction::from_fn_pointer(|args, _| args[0].map_scalar(|v| v.sin())),
        );
        context.add_function(
            vec![Token::Backslash, Token::Identifier("cos".to_string())],
            MathFunction::from_fn_pointer(|args, _| args[0].map_scalar(|v| v.cos())),
        );
        context.add_function(
            vec![Token::Backslash, Token::Identifier("tan".to_string())],
            MathFunction::from_fn_pointer(|args, _| args[0].map_scalar(|v| v.tan())),
        );

        // Logarithm
        context.add_function(
            vec![Token::Backslash, Token::Identifier("ln".to_string())],
            MathFunction::from_fn_pointer(|args, _| args[0].map_scalar(|v| v.ln())),
        );

        context
    }
}
