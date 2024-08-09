//!Removing inconsistencies and style choices using the Normalizer
use std::collections::VecDeque;

use tracing::{debug, trace, trace_span};

use crate::prelude::*;
///The normalizer for making the tokens easier to handle by removing
/// stylization for example
pub struct Normalizer<T>
where
    T: IntoIterator<Item = Token>,
{
    ///The input from the lexer
    input: T::IntoIter,
    remainders: VecDeque<Token>,
}
impl<I: IntoIterator<Item = Token>> Normalizer<I> {
    ///Creates a normalizer
    pub fn new(t: I) -> Self {
        trace!("created Normalizer");

        Self {
            input: t.into_iter(),
            remainders: VecDeque::new(),
        }
    }
}
impl<T: IntoIterator<Item = Token>> Iterator for Normalizer<T> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let span = trace_span!("normalize_tokens");
        let _enter = span.enter();
        trace!("normalize_tokens");

        loop {
            let first =
                self.remainders.pop_front().or_else(|| self.input.next());
            let second =
                self.remainders.pop_front().or_else(|| self.input.next());
            debug!("tokens=({:?},{:?})", first, second);
            match (first, second) {
                (None, None) => return None,
                (None, Some(_)) => unreachable!("Some after None in iterator"),
                (Some(v), None) => return Some(v),
                (Some(first), Some(second)) => {
                    match [first, second] {
                        [Token::Backslash, Token::Identifier(v)] => {
                            trace!("ident = {v}");
                            match v.as_str() {
                                "left" | "middle" | "right" => {
                                    continue;
                                }
                                "displaystyle" | "textstyle" => {
                                    continue;
                                }
                                _ => {
                                    self.remainders.push_front(
                                        Token::Identifier(v.clone()),
                                    );
                                    return Some(Token::Backslash);
                                }
                            }
                        }
                        [Token::Caret, Token::NumberLiteral(n)] => {
                            trace!("number literal = {n}");
                            if n.0.is_empty() {
                                panic!("string is weird");
                            }
                            if n.0.len() != 1 {
                                let mut s = n.0.clone();
                                let rest =
                                    Token::NumberLiteral(s.split_off(1).into());
                                trace!("rest = {:?}", rest);
                                let single = Token::NumberLiteral(s.into());
                                trace!("single = {:?}", single);
                                self.remainders.push_front(rest);
                                self.remainders.push_front(single);
                                return Some(Token::Caret);
                            }
                        }
                        [first, second] => {
                            self.remainders.push_front(second);
                            return Some(first);
                        }
                    };
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use std::hint::black_box;

    use crate::{number_literal::NumberLiteral, prelude::*};
    use pretty_assertions::assert_eq;
    use tracing_test::traced_test;
    fn lex_and_normalize(s: &str) -> Vec<Token> {
        Normalizer::new(Lexer::new(s)).collect()
    }
    fn normalize(tokens: Vec<Token>) -> Vec<Token> {
        Normalizer::new(tokens).collect()
    }
    #[test]
    fn direct_eof() {
        black_box(normalize(vec![Token::EndOfContent]));
    }
    #[traced_test]
    #[test]
    fn second_is_eof() {
        black_box(normalize(vec![Token::Backslash, Token::EndOfContent]));
    }
    #[traced_test]
    #[test]
    fn all_tokens_returned() {
        assert_eq!(
            normalize(vec![
                Token::Backslash,
                Token::Identifier("sqrt".to_string()),
                Token::LeftCurlyBracket,
                Token::NumberLiteral(1.into()),
                Token::Plus,
                Token::NumberLiteral(2.into()),
                Token::Identifier("x".to_string()),
                Token::RightCurlyBracket,
                Token::EndOfContent,
            ]),
            vec![
                Token::Backslash,
                Token::Identifier("sqrt".to_string()),
                Token::LeftCurlyBracket,
                Token::NumberLiteral(1.into()),
                Token::Plus,
                Token::NumberLiteral(2.into()),
                Token::Identifier("x".to_string()),
                Token::RightCurlyBracket,
                Token::EndOfContent,
            ]
        );
    }
    #[traced_test]
    #[test]
    fn exponent_split() {
        assert_eq!(
            normalize(vec![
                Token::NumberLiteral(2.into()),
                Token::Caret,
                Token::NumberLiteral(NumberLiteral("025".to_owned())),
                Token::EndOfContent,
            ]),
            vec![
                Token::NumberLiteral(2.into()),
                Token::Caret,
                Token::NumberLiteral(0.into()),
                Token::NumberLiteral("25".to_owned().into()),
                Token::EndOfContent,
            ]
        );
    }

    #[test]
    fn remove_left_middle_right() {
        assert_eq!(
            normalize(vec![
                Token::Backslash,
                Token::Identifier("left".to_string()),
                Token::LeftParenthesis,
                Token::NumberLiteral("1".to_owned().into()),
                Token::Backslash,
                Token::Identifier("middle".to_string()),
                Token::Slash,
                Token::NumberLiteral("1".to_owned().into()),
                Token::Backslash,
                Token::Identifier("right".to_string()),
                Token::RightParenthesis,
                Token::EndOfContent,
            ]),
            vec![
                Token::LeftParenthesis,
                Token::NumberLiteral("1".to_owned().into()),
                Token::Slash,
                Token::NumberLiteral("1".to_owned().into()),
                Token::RightParenthesis,
                Token::EndOfContent,
            ]
        );
    }
    #[test]
    fn parenthasis_and_carret() {
        assert_eq!(
            lex_and_normalize("2x^{2} + 5xy"),
            vec![Token::NumberLiteral(2.into())]
        );
    }
}
