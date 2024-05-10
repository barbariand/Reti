//!All valid token types in the TokenStream
use std::{fmt::Display, hash::Hash, num::ParseFloatError, str::FromStr};
/// All the accepted tokens
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum Token {
    /// Any other unknown chars not identified under
    Identifier(String),

    /// A number with extra info
    /// specifically matching [ `0` - `9` ] and `.`
    NumberLiteral(NumberLiteral),

    /// String representation:`\`
    Backslash,

    /// String representation:`{`
    LeftCurlyBracket,

    /// String representation:`}`
    RightCurlyBracket,

    /// String representation:`[`
    LeftBracket,

    /// String representation:`]`
    RightBracket,

    /// String representation:`(`
    LeftParenthesis,

    /// String representation:`)`
    RightParenthesis,

    /// String representation:`+`
    Plus,
    
    /// String representation:`-`
    Minus,
    
    /// String representation:`*`
    Asterisk,
    
    /// String representation:`/`
    Slash,
    
    /// String representation:`'`
    Apostrophe,
    
    /// String representation:`_`
    Underscore,
    
    /// String representation:`^`
    Caret,
    
    /// String representation:`|`
    VerticalPipe,

    /// String representation:`,`
    Comma,

    /// String representation:`&`
    Ampersand,

    /// String representation:`=`
    Equals,

    /// No string representation
    EndOfContent,
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
    ///if it is the ident variant and matching the string provided
    pub fn is_ident(&self, text: &str) -> bool {
        match self {
            Self::Identifier(val) => val == text,
            _ => false,
        }
    }
    ///Takes a ref to the inner string of the ident
    pub fn take_ident(&self) -> Option<&String> {
        match self {
            Self::Identifier(v) => Some(v),
            _ => None,
        }
    }
    ///is end of content
    pub fn is_eoc(&self) -> bool {
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
///The number representation
#[derive(Debug, Clone)]
pub struct NumberLiteral {
    ///the raw string without being parsed as a number
    pub raw: String,
    ///The parsed value
    pub parsed: f64,
}
impl Display for NumberLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.parsed)
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
