//! helper functions
use crate::prelude::*;

impl MathExpr {
    ///makes a new MathExpr where the term part is wrapped if needed
    pub fn add_wrapped(a:MathExpr,b:MathExpr)->Self{
        Self::Add(a.boxed(), b.get_term_or_wrap())
    }
    ///makes a new MathExpr where the term part is wrapped if needed
    pub fn subtract_wrapped(a:MathExpr,b:MathExpr)->Self{
        Self::Subtract(a.boxed(), b.get_term_or_wrap())
    }
    ///Returns a term or a eval error
    pub fn expect_term(&self) -> Result<&Term, EvalError> {
        match self {
            MathExpr::Term(t) => Ok(t),
            _ => Err(EvalError::ExpectedTerm {
                found: self.clone(),
            }),
        }
    }
    ///gets the term or wraps it in parenthesis
    pub fn get_term_or_wrap(&self) -> Term {
        match self {
            MathExpr::Term(t) => t.clone(),
            _ => Factor::Parenthesis(self.clone().boxed()).into(),
        }
    }
    ///Returns a factor or a eval error
    pub fn get_factor(&self) -> Result<&Factor, EvalError> {
        match self {
            MathExpr::Term(t) => t.get_factor(),
            _ => Err(EvalError::ExpectedFactor {
                found: self.clone(),
            }),
        }
    }
    ///gets the factor or wraps it in parenthesis
    pub fn get_factor_or_wrap(&self) -> Factor {
        match self {
            MathExpr::Term(Term::Factor(f)) => f.clone(),
            _ => Factor::Parenthesis(self.clone().boxed()),
        }
    }
    
    ///Boxes self
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

impl Term {
    ///Returns a factor or a eval error
    fn get_factor(&self) -> Result<&Factor, EvalError> {
        match self {
            Term::Factor(f) => Ok(f),
            _ => Err(EvalError::ExpectedFactor {
                found: self.clone().into(),
            }),
        }
    }
    ///Boxes self
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

impl Factor {
    ///Boxes self
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

impl MathIdentifier {
    ///Creates a new MathIdentifier fom a vec to identify a variable and
    /// function
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens }
    }
    ///Creates a new MathIdentifier from a single Token to identify a variable
    /// and a function
    pub fn new_from_one(token: Token) -> Self {
        Self {
            tokens: vec![token],
        }
    }
    ///# Warning
    /// does no conversion or latex translation
    pub fn from_single_ident(s: &str) -> Self {
        Self {
            tokens: vec![Token::Identifier(s.to_owned())],
        }
    }
}

impl FunctionCall {
    ///a helper method
    pub fn new(
        function_name: MathIdentifier,
        arguments: Vec<MathExpr>,
    ) -> Self {
        Self {
            function_name,
            arguments,
        }
    }
}
