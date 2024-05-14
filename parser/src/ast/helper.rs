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
///Helper trait for comparing f64
pub(crate) trait NumberCompare{
    ///if it is zero
    fn is_zero(&self)->bool;
    ///if it is one
    fn is_one(&self)->bool;
    ///if it equals the other
    #[allow(dead_code)]
    fn equals(&self,other:Self)->bool;
}
impl NumberCompare for f64{
    fn is_zero(&self)->bool{
        self.abs()<f64::EPSILON
    }

    fn is_one(&self)->bool{
        (self-1.0).abs()<f64::EPSILON
    }

    fn equals(&self,other:Self)->bool{
        (self-other)<f64::EPSILON
    }
}
