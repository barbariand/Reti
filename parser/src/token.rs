
#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    Ident(String),
    NumberLiteral(f64),
    CommandPrefix,
    ExpressionBegin,  // {
    ExpressionEnd,    // }
    BracketBegin,     // [
    BracketEnd,       // ]
    ParenthesisBegin, // (
    ParenthesisEnd,   // )
    Negative,         // -
    Apostrofy,        // '
    Underscore,       // _
    Caret,            // ^
    Mul,              // *
    Add,              // +
    Div,              // \
    VerticalPipe,     // ｜ ik its tipping
    EOF,              // \EOF
}

impl Token {
    pub fn is_ident(&self, text: &str) -> bool {
        match self {
            Self::Ident(val) => val == text,
            _ => false,
        }
    }
    pub fn take_ident(&self) -> Option<&String> {
        match self {
            Self::Ident(v) => Some(v),
            _ => None,
        }
    }

    /// Determine if the token marks the end of an expression.
    pub fn is_end(&self) -> bool {
        match self {
            Token::ExpressionEnd => true,
            Token::BracketEnd => true,
            Token::ParenthesisEnd => true,
            Token::EOF => true,
            _ => false,
        }
    }
}
