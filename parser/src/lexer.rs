//! Creating a token stream from a string
use crate::{number_literal::NumberLiteral, prelude::*};
use std::{iter::Peekable, mem::take, str::Chars};
use tracing::{debug, trace, trace_span};

impl<'a> Lexer<'a> {
    ///creating a new Lexer
    pub fn new(s: &'a str) -> Self {
        debug!("Creating a lexer with: {:?}", s);
        Self {
            input: s.chars().peekable(),
            temp_ident: String::new(),
            temp_number: String::new(),
            done: false,
        }
    }
    #[deprecated]
    ///The main function of the Lexer, will create tokens and send them away
    pub fn tokenize(self) -> Vec<Token> {
        let mut res = Vec::new();
        let span = trace_span!("lexer::tokenize");
        let _enter = span.enter();
        debug!("tokenizing: {:?}", self.input);
        let mut temp_ident = String::new();
        let mut temp_number = String::new();
        for c in self.input {
            trace!("char = {c:?}");
            let t = match c {
                '0'..='9' | '.' => {
                    if !temp_ident.is_empty() {
                        res.push(Token::Identifier(take(&mut temp_ident)))
                    }
                    trace!("temp_number::push char={c:?}");
                    temp_number.push(c);
                    continue;
                }
                '\\' => Token::Backslash,
                '{' => Token::LeftCurlyBracket,
                '}' => Token::RightCurlyBracket,
                '[' => Token::LeftBracket,
                ']' => Token::RightBracket,
                '-' => Token::Minus,
                '\'' => Token::Apostrophe,
                '_' => Token::Underscore,
                '^' => Token::Caret,
                '|' => Token::VerticalPipe,
                '*' => Token::Asterisk,
                '+' => Token::Plus,
                '/' => Token::Slash,
                ',' => Token::Comma,
                '&' => Token::Ampersand,
                '=' => Token::Equals,
                '(' => Token::LeftParenthesis,
                ')' => Token::RightParenthesis,
                ' ' => {
                    if !temp_number.is_empty() {
                        let num = Token::NumberLiteral(temp_number.into());
                        temp_number = String::new();
                        res.push(num);
                    }
                    if !temp_ident.is_empty() {
                        res.push(Token::Identifier(take(&mut temp_ident)))
                    }
                    continue;
                }
                _ => {
                    if !temp_number.is_empty() {
                        let num = Token::NumberLiteral(temp_number.into());
                        temp_number = String::new();
                        res.push(num)
                    }

                    trace!("temp_ident::push char={c:?}");
                    temp_ident.push(c);
                    continue;
                }
            };
            if !temp_number.is_empty() {
                let num = Token::NumberLiteral(temp_number.into());
                temp_number = String::new();
                res.push(num)
            }
            if !temp_ident.is_empty() {
                res.push(Token::Identifier(take(&mut temp_ident)));
            }

            res.push(t);
        }
        if !temp_number.is_empty() {
            let num = Token::NumberLiteral(
                temp_number
                    .parse()
                    .expect("THIS NEEDS FIXING IT FAILED TO PARSE NUMBER"),
            );

            res.push(num);
        }
        if !temp_ident.is_empty() {
            res.push(Token::Identifier(take(&mut temp_ident)));
        }
        res.push(Token::EndOfContent);
        res
    }
}
///The lexer creating tokens from a string
pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    temp_ident: String,
    temp_number: String,
    done: bool,
}
const KNOWN_CHARS: [char; 29] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '\\', '{', '}', '[', ']',
    '-', '\'', '_', '^', '|', '*', '+', '/', ',', '&', '=', '(', ')', ' ',
];
impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let span = trace_span!("lexer::tokenize");
        let _enter = span.enter();
        debug!("tokenizing: {:?}", self.input);
        while let Some(c) = self.input.next() {
            trace!("char = {c:?}");
            let t = match c {
                '0'..='9' | '.' => {
                    trace!("temp_number::push char={c:?}");
                    self.temp_number.push(c);
                    if self
                        .input
                        .peek()
                        .is_some_and(|v| !matches!(v, '0'..='9' | '.'))
                    {
                        return Some(Token::NumberLiteral(
                            NumberLiteral::checked_new_unchanged_str(take(
                                &mut self.temp_number,
                            )),
                        ));
                    }
                    continue;
                }
                '\\' => Token::Backslash,
                '{' => Token::LeftCurlyBracket,
                '}' => Token::RightCurlyBracket,
                '[' => Token::LeftBracket,
                ']' => Token::RightBracket,
                '-' => Token::Minus,
                '\'' => Token::Apostrophe,
                '_' => Token::Underscore,
                '^' => Token::Caret,
                '|' => Token::VerticalPipe,
                '*' => Token::Asterisk,
                '+' => Token::Plus,
                '/' => Token::Slash,
                ',' => Token::Comma,
                '&' => Token::Ampersand,
                '=' => Token::Equals,
                '(' => Token::LeftParenthesis,
                ')' => Token::RightParenthesis,
                ' ' => {
                    assert!(self.temp_ident.is_empty());
                    assert!(self.temp_number.is_empty());
                    continue;
                }
                c => {
                    self.temp_ident.push(c);
                    if self
                        .input
                        .peek()
                        .is_some_and(|v| KNOWN_CHARS.contains(v))
                    {
                        return Some(Token::Identifier(take(
                            &mut self.temp_ident,
                        )));
                    }
                    continue;
                }
            };
            return Some(t);
        }
        trace!("no more chars");
        if !self.temp_number.is_empty() {
            let num =
                Token::NumberLiteral(NumberLiteral::checked_new_unchanged_str(
                    take(&mut self.temp_number),
                ));

            return Some(num);
        }
        if !self.temp_ident.is_empty() {
            return Some(Token::Identifier(take(&mut self.temp_ident)));
        }
        if !self.done {
            self.done = true;
            return Some(Token::EndOfContent);
        }
        trace!("returning none");
        None
    }
}
#[cfg(test)]
mod tests {

    use crate::{number_literal::NumberLiteral, prelude::*};
    use pretty_assertions::assert_eq;
    fn tokenize(text: &str) -> Vec<Token> {
        let lexer = Lexer::new(text);
        lexer.collect()
    }

    #[test]
    fn test_simple_sqrt() {
        assert_eq!(
            tokenize("\\sqrt{1+2x}"),
            vec![
                Token::Backslash,
                Token::Identifier("sqrt".to_string()),
                Token::LeftCurlyBracket,
                Token::NumberLiteral(1.into()),
                Token::Plus,
                Token::NumberLiteral(2.into()),
                Token::Identifier("x".to_string()),
                Token::RightCurlyBracket,
                Token::EndOfContent
            ]
        );
    }
    #[test]
    fn test_all_simple_operations() {
        assert_eq!(
            tokenize("-+*/"),
            vec![
                Token::Minus,
                Token::Plus,
                Token::Asterisk,
                Token::Slash,
                Token::EndOfContent
            ]
        );
    }

    #[test]
    fn test_single_character_tokens() {
        assert_eq!(
            tokenize("()[]{}^'|"),
            vec![
                Token::LeftParenthesis,
                Token::RightParenthesis,
                Token::LeftBracket,
                Token::RightBracket,
                Token::LeftCurlyBracket,
                Token::RightCurlyBracket,
                Token::Caret,
                Token::Apostrophe,
                Token::VerticalPipe,
                Token::EndOfContent
            ]
        );
    }
    #[test]
    fn test_number_literals() {
        assert_eq!(
            tokenize("3.14 42"),
            vec![
                Token::NumberLiteral("3.14".to_owned().into()),
                Token::NumberLiteral(42.into()),
                Token::EndOfContent
            ]
        );
    }
    #[test]
    fn test_identifiers_and_commands() {
        assert_eq!(
            tokenize("\\pi R"),
            vec![
                Token::Backslash,
                Token::Identifier("pi".to_string()),
                Token::Identifier("R".to_string()),
                Token::EndOfContent
            ]
        );
    }
    #[test]
    fn test_complex_expressions() {
        assert_eq!(
            tokenize("{3.14*R^2}"),
            vec![
                Token::LeftCurlyBracket,
                Token::NumberLiteral("3.14".to_owned().into()),
                Token::Asterisk,
                Token::Identifier("R".to_string()),
                Token::Caret,
                Token::NumberLiteral(2.into()),
                Token::RightCurlyBracket,
                Token::EndOfContent
            ]
        );
    }
    #[test]
    fn test_number_followed_by_identifier() {
        assert_eq!(
            tokenize("42x + 3.14y"),
            vec![
                Token::NumberLiteral(42.into()),
                Token::Identifier("x".to_string()),
                Token::Plus,
                Token::NumberLiteral("3.14".to_owned().into()),
                Token::Identifier("y".to_string()),
                Token::EndOfContent
            ]
        );
    }
    #[test]
    fn test_number_followed_by_command() {
        assert_eq!(
            tokenize("3.14\\piR"),
            vec![
                Token::NumberLiteral("3.14".to_owned().into()),
                Token::Backslash,
                Token::Identifier("piR".to_string()),
                Token::EndOfContent
            ]
        );
    }
    #[test]
    fn test_mixed_number_and_text_sequences() {
        assert_eq!(
            tokenize("2a + 4b - 5\\sqrt{c}"),
            vec![
                Token::NumberLiteral(2.into()),
                Token::Identifier("a".to_string()),
                Token::Plus,
                Token::NumberLiteral(4.into()),
                Token::Identifier("b".to_string()),
                Token::Minus,
                Token::NumberLiteral(5.into()),
                Token::Backslash,
                Token::Identifier("sqrt".to_string()),
                Token::LeftCurlyBracket,
                Token::Identifier("c".to_string()),
                Token::RightCurlyBracket,
                Token::EndOfContent
            ]
        );
    }
    #[test]
    fn test_space_priority() {
        assert_eq!(
            tokenize("2^025"),
            vec![
                Token::NumberLiteral(2.into()),
                Token::Caret,
                Token::NumberLiteral(NumberLiteral::checked_new_unchanged_str(
                    "025".to_owned()
                )),
                Token::EndOfContent
            ]
        )
    }
}
