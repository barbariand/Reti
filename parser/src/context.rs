//! # Context
//! this module is for helping with keeping track of variables and functions
//! for that it uses MathContext where you can add any function or variable  
use std::collections::HashMap;

use crate::{identifier::GreekLetter, prelude::*};

///The MathContext, holding all the functions and variables
#[derive(Clone, Debug)]
pub struct MathContext {
    ///The variables
    pub variables: HashMap<MathIdentifier, MathExpr>,
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

    /// Adding a variable that is a single greek letter.
    fn add_greek_var(&mut self, letter: GreekLetter, value: MathExpr) {
        self.variables
            .insert(MathIdentifier::from_single_greek(letter), value);
    }

    fn add_ascii_var(&mut self, s: &str, value: MathExpr) {
        self.variables
            .insert(MathIdentifier::from_single_ident(s), value);
    }

    ///Adding a function when it is IntoMathFunction
    pub fn add_function(&mut self, s: &str, func: impl IntoMathFunction) {
        self.functions.insert(
            MathIdentifier::from_single_ident(s),
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
        context.add_greek_var(
            GreekLetter::LowercasePi,
            Factor::Constant(std::f64::consts::PI).into(),
        );
        context
            .add_ascii_var("e", Factor::Constant(std::f64::consts::E).into());

        // TODO add proper functions system so we can define the definition
        //  and value sets to validate the amount of arguments, the types of
        // arguments  (scalar or matrix).

        // Trigonometric functions
        context.add_function("sin", f64::sin);
        context.add_function("cos", f64::cos);
        context.add_function("tan", f64::tan);

        // Logarithm
        context.add_function("ln", f64::ln);

        context
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
            |v: f64| v,
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
            |v: f64| v,
        );
        let mut c1 = MathContext::new();
        c1.add_function(
            vec![Token::Backslash, Token::Identifier("nothing".to_owned())],
            (|_| whatever!("testing"), 1, None),
        );
        c1.merge(&c2);
        let f = &c1.functions[&MathIdentifier::new(vec![
            Token::Backslash,
            Token::Identifier("nothing".to_owned()),
        ])];
        assert!(match f {
            MathFunction::Native(n) => n.run(vec![Value::Scalar(1.1)]).is_err(),
            MathFunction::Foreign(_) => false,
        });
    }
}
