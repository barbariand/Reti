use std::collections::HashMap;

use crate::prelude::{MathIdentifier, Token};

#[derive(Debug, Clone)]
pub struct MathFunction {
    pub approximate: fn(Vec<f64>) -> f64,
}
#[derive(Clone)]
pub struct MathContext {
    pub variables: HashMap<MathIdentifier, f64>,
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
            self.variables.insert((*variable).clone(), *value);
        }
        for (name, value) in other.functions.iter() {
            self.functions.insert(name.clone(), value.clone());
        }
    }

    pub fn is_function(&self, ident: &MathIdentifier) -> bool {
        
        // println!("is_function({:?}) = {}", ident, res); TODO tracing
        self.functions.contains_key(ident)
    }

    fn add_var(&mut self, identifier: Vec<Token>, value: f64) {
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
            std::f64::consts::PI,
        );
        context.add_var(
            vec![Token::Identifier("e".to_string())],
            std::f64::consts::E,
        );

        // Trigonometric functions
        context.add_function(
            vec![Token::Backslash, Token::Identifier("sin".to_string())],
            MathFunction {
                approximate: |args| args[0].sin(),
            },
        );
        context.add_function(
            vec![Token::Backslash, Token::Identifier("cos".to_string())],
            MathFunction {
                approximate: |args| args[0].cos(),
            },
        );
        context.add_function(
            vec![Token::Backslash, Token::Identifier("tan".to_string())],
            MathFunction {
                approximate: |args| args[0].tan(),
            },
        );

        // Logarithm
        context.add_function(
            vec![Token::Backslash, Token::Identifier("ln".to_string())],
            MathFunction {
                approximate: |args| args[0].ln(),
            },
        );

        context
    }
}
