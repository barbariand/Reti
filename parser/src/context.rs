use std::collections::HashMap;

use crate::{approximator::EvalError, ast::MathIdentifier, token::Token, value::Value};

#[derive(Debug, Clone)]
pub struct MathFunction {
    pub approximate: fn(Vec<Value>) -> Result<Value, EvalError>,
}

pub struct MathContext {
    pub variables: HashMap<MathIdentifier, Value>,
    pub functions: HashMap<MathIdentifier, MathFunction>,
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
        let res = self.functions.contains_key(ident);
        // println!("is_function({:?}) = {}", ident, res); TODO tracing
        res
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
            MathFunction {
                approximate: |args| Ok(Value::Scalar(args[0].scalar()?.sin())),
            },
        );
        context.add_function(
            vec![Token::Backslash, Token::Identifier("cos".to_string())],
            MathFunction {
                approximate: |args| Ok(Value::Scalar(args[0].scalar()?.cos())),
            },
        );
        context.add_function(
            vec![Token::Backslash, Token::Identifier("tan".to_string())],
            MathFunction {
                approximate: |args| Ok(Value::Scalar(args[0].scalar()?.tan())),
            },
        );

        // Logarithm
        context.add_function(
            vec![Token::Backslash, Token::Identifier("ln".to_string())],
            MathFunction {
                approximate: |args| Ok(Value::Scalar(args[0].scalar()?.ln())),
            },
        );

        context
    }
}
