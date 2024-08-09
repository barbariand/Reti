//! Creating a token stream from a string
use crate::{number_literal::NumberLiteral, prelude::*};
use std::{fmt::Debug, iter::Peekable, mem::take, str::Chars};
use tracing::{debug, trace, trace_span};
///The lexer creating tokens from a string
pub struct Lexer<I>
where
    I: IntoIterator<Item = char>,
{
    input: Peekable<I::IntoIter>,
    temp_ident: String,
    temp_number: String,
}
impl<I> Lexer<I>
where
    I: IntoIterator<Item = char>,
{
    ///creating a new Lexer
    pub fn new(s: I) -> Lexer<I>
    where
        I: Debug,
    {
        debug!("Creating a lexer with: {:?}", s);
        Lexer {
            input: s.into_iter().peekable(),
            temp_ident: String::new(),
            temp_number: String::new(),
        }
    }
}

const KNOWN_CHARS: [char; 29] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '\\', '{', '}', '[', ']',
    '-', '\'', '_', '^', '|', '*', '+', '/', ',', '&', '=', '(', ')', ' ',
];
impl<I> Iterator for Lexer<I>
where
    I: IntoIterator<Item = char>,
{
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        let span = trace_span!("lexer::tokenize");
        let _enter = span.enter();
        debug!("tokenizing: {:?}", self.input.peek());
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
        trace!("returning none");
        None
    }
}
#[cfg(test)]
mod tests {

    use crate::{number_literal::NumberLiteral, prelude::*};
    use pretty_assertions::assert_eq;
    use tracing_test::traced_test;
    fn tokenize(text: &str) -> Vec<Token> {
        let lexer = Lexer::new(text.chars());
        lexer.collect()
    }

    #[traced_test]
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
            ]
        );
    }
    #[traced_test]
    #[test]
    fn test_all_simple_operations() {
        assert_eq!(
            tokenize("-+*/"),
            vec![Token::Minus, Token::Plus, Token::Asterisk, Token::Slash,]
        );
    }

    #[traced_test]
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
            ]
        );
    }
    #[traced_test]
    #[test]
    fn test_number_literals() {
        assert_eq!(
            tokenize("3.14 42"),
            vec![
                Token::NumberLiteral("3.14".to_owned().into()),
                Token::NumberLiteral(42.into()),
            ]
        );
    }
    #[traced_test]
    #[test]
    fn test_identifiers_and_commands() {
        assert_eq!(
            tokenize("\\pi R"),
            vec![
                Token::Backslash,
                Token::Identifier("pi".to_string()),
                Token::Identifier("R".to_string()),
            ]
        );
    }
    #[traced_test]
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
            ]
        );
    }
    #[traced_test]
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
            ]
        );
    }
    #[traced_test]
    #[test]
    fn test_number_followed_by_command() {
        assert_eq!(
            tokenize("3.14\\piR"),
            vec![
                Token::NumberLiteral("3.14".to_owned().into()),
                Token::Backslash,
                Token::Identifier("piR".to_string()),
            ]
        );
    }
    #[traced_test]
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
            ]
        );
    }
    #[traced_test]
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
            ]
        )
    }
}
