
#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    Identifier(String),
    NumberLiteral(f64),
    Backslash,          // \
    LeftCurlyBracket,   // {
    RightCurlyBracket,  // }
    LeftBracket,        // [
    RightBracket,       // ]
    LeftParen,          // (
    RightParen,         // )
    Plus,               // +
    Minus,              // -
    Asterisk,           // *
    Slash,              // /
    Apostrophe,         // '
    Underscore,         // _
    Caret,              // ^
    VerticalPipe,       // | and |
    EOF,                // A special token that represents the end of content.
}

impl Token {
    pub fn is_ident(&self, text: &str) -> bool {
        match self {
            Self::Identifier(val) => val == text,
            _ => false,
        }
    }
    pub fn take_ident(&self) -> Option<&String> {
        match self {
            Self::Identifier(v) => Some(v),
            _ => None,
        }
    }

    /// Determine if the token marks the end of an expression.
    pub fn is_end(&self) -> bool {
        match self {
            Token::RightCurlyBracket => true,
            Token::RightBracket => true,
            Token::RightParen => true,
            Token::EOF => true,
            _ => false,
        }
    }
}
