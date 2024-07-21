//!All valid token types in the TokenStream
use std::{fmt::Display, hash::Hash};

use crate::number_literal::NumberLiteral;
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
    pub const fn take_ident(&self) -> Option<&String> {
        match self {
            Self::Identifier(v) => Some(v),
            _ => None,
        }
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
