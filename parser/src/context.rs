use std::collections::HashMap;

use crate::ast::{MathExpr, MathIdentifier};

pub type MathFunction = fn(Vec<MathExpr>) -> f64;

pub struct MathContext {
    pub variables: HashMap<MathIdentifier, f64>,
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
            self.variables.insert((*variable).clone(), *value);
        }
        for (name, value) in other.functions.iter() {
            self.functions.insert(name.clone(), *value);
        }
    }

    pub fn is_function(&self, ident: &MathIdentifier) -> bool {
        let res = self.functions.contains_key(ident);
        println!("is_function({:?}) = {}", ident, res);
        res
    }
}
