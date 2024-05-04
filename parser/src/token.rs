use std::{fmt::Display, hash::Hash, num::ParseFloatError, str::FromStr};
/// All the axepted tokens
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum Token {
    /// Any non other 
    Identifier(String),
    NumberLiteral(NumberLiteral),
    /// String representation:`\`
    /// for command starts and seperating matrixes
    Backslash,
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
    Comma,             // ,
    Ampersand,         // &
    Equals,            // =
    EndOfContent,      // A special token that represents the end of content.
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Token::Identifier(c) => c.as_str(),
                Token::NumberLiteral(n) => {
                    return writeln!(f, "{}", n);
                }
                Token::Backslash => "\\",
                Token::LeftCurlyBracket => "{",
                Token::RightCurlyBracket => "}",
                Token::LeftBracket => "[",
                Token::RightBracket => "]",
                Token::LeftParenthesis => "(",
                Token::RightParenthesis => ")",
                Token::Plus => "+",
                Token::Minus => "-",
                Token::Asterisk => "*",
                Token::Slash => "/",
                Token::Apostrophe => "'",
                Token::Underscore => "_",
                Token::Caret => "^",
                Token::VerticalPipe => "|",
                Token::Comma => ",",
                Token::Ampersand => "&",
                Token::EndOfContent => "EOF",
                Token::Equals => "=",
            }
        )
    }
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
    pub raw: String,
    pub parsed: f64,
}
impl Display for NumberLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.parsed)
    }
}
impl NumberLiteral {
    pub fn reparse_from_raw(&mut self) {
        self.parsed = self.raw.parse().expect("INTERNAL PARSING BUG")
    }
}
impl PartialEq for NumberLiteral {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}
impl Eq for NumberLiteral {}
impl Hash for NumberLiteral {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state)
    }
}

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
            parsed: value.parse().expect("INTERNAL PARSING BUG"),
            raw: value.to_owned(),
        }
    }
}
impl From<String> for NumberLiteral {
    fn from(value: String) -> Self {
        Self {
            parsed: value.parse().expect("INTERNAL PARSING BUG"),
            raw: value,
        }
    }
}
impl From<usize> for NumberLiteral {
    fn from(value: usize) -> Self {
        value.to_string().into()
    }
}
