use std::{num::ParseFloatError, str::FromStr};

#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    Identifier(String),
    NumberLiteral(NumberLiteral),
    Backslash,         // \
    LeftCurlyBracket,  // {
    RightCurlyBracket, // }
    LeftBracket,       // [
    RightBracket,      // ]
    LeftParenthesis,   // (
    RightParenthesis,  // )
    Plus,              // +
    Minus,             // -
    Asterisk,          // *
    Slash,             // /
    Apostrophe,        // '
    Underscore,        // _
    Caret,             // ^
    VerticalPipe,      // | and |
    EndOfContent,      // A special token that represents the end of content.
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
    pub fn is_eof(&self) -> bool {
        self == Token::EndOfContent
    }
    /// Determine if the token marks the end of an expression.
    pub fn is_end(&self) -> bool {
        matches!(
            self,
            Token::RightCurlyBracket
                | Token::RightBracket
                | Token::RightParenthesis
                | Token::EndOfContent
        )
    }
}
impl PartialEq<Token> for &Token {
    fn eq(&self, other: &Token) -> bool {
        **self == *other
    }
}
impl PartialEq<&Token> for Token {
    fn eq(&self, other: &&Token) -> bool {
        *self == **other
    }
}
#[derive(Debug, Clone)]
pub struct NumberLiteral {
    raw: String,
    pub parsed: f64,
}
impl PartialEq for NumberLiteral {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}
impl Eq for NumberLiteral {}

impl FromStr for NumberLiteral {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let i: f64 = s.parse()?;
        Ok(Self {
            raw: s.to_owned(),
            parsed: i,
        })
    }
}
impl From<&str> for NumberLiteral {
    fn from(value: &str) -> Self {
        Self {
            parsed: value
                .parse()
                .expect("THIS NEEDS FIXING IT FAILED TO PARSE NUMBER"),
            raw: value.to_owned(),
        }
    }
}
impl From<String> for NumberLiteral {
    fn from(value: String) -> Self {
        Self {
            parsed: value
                .parse()
                .expect("THIS NEEDS FIXING IT FAILED TO PARSE NUMBER"),
            raw: value,
        }
    }
}
impl From<usize> for NumberLiteral {
    fn from(value: usize) -> Self {
        value.to_string().into()
    }
}
