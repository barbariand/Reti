use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{debug, trace, trace_span};

use crate::{token::Token, token_reader::TokenReader};

pub struct Normalizer {
    reader: TokenReader,
    output: Sender<Token>,
}
impl Normalizer {
    pub fn new(input: Receiver<Token>, output: Sender<Token>) -> Self {
        Self {
            reader: TokenReader::new(input),
            output,
        }
    }

    pub async fn normalize(mut self) {
        let span = trace_span!("normalizer::normalize");
        let _enter = span.enter();
        loop {
            self.normalize_tokens().await;
            debug!("successfully normalized tokens");

            let token = self.reader.read().await;
            trace!("reader::read {token}");
            let eof = token == Token::EndOfContent;
            trace!("output::send {token}");
            self.output.send(token).await.expect("Broken pipe");
            if eof {
                trace!("'end of content' has been hit");
                break;
            }
        }
    }

    async fn normalize_tokens(&mut self) {
        let span = trace_span!("normalize_tokens");
        let _enter = span.enter();
        trace!("normalize_tokens");
        match self.reader.peek_range(0..=1).await[..] {
            [Token::Backslash, Token::Identifier(v)] => {
                trace!("ident = {v}");
                match v.as_str() {
                    "cdot" | "cdotp" | "times" => {
                        self.reader.replace(0..=1, vec![Token::Asterisk]).await;
                    }
                    "left" | "middle" | "right" => {
                        self.reader.replace(0..=1, vec![]).await;
                        // TODO Remove dot after, for example "\left."
                        // we have no token for lone dots though
                    }
                    "displaystyle" | "textstyle" => {
                        self.reader.replace(0..=1, vec![]).await;
                    }
                    _ => {}
                }
            }
            [Token::Caret, Token::NumberLiteral(n)] => {
                trace!("number literal = {n}");
                if n.raw.is_empty() {
                    panic!("string is weird");
                }
                if n.raw.len() != 1 {
                    let mut s = n.raw.clone();
                    let rest = Token::NumberLiteral(s.split_off(1).into());
                    trace!("rest = {:?}", rest);
                    let single = Token::NumberLiteral(s.into());
                    trace!("single = {:?}", single);
                    self.reader.replace(1..=1, vec![single, rest]).await;
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;

    use super::Normalizer;
    use crate::token::Token;
    use tokio::sync::mpsc::{self, Receiver, Sender};

    async fn normalize(tokens: Vec<Token>) -> Vec<Token> {
        let (tx1, rx1): (Sender<Token>, Receiver<Token>) = mpsc::channel(32);
        let (tx2, mut rx2): (Sender<Token>, Receiver<Token>) = mpsc::channel(32);
        let normalizer = Normalizer::new(rx1, tx2);

        let mut result = Vec::with_capacity(tokens.len());

        for token in tokens {
            tx1.send(token).await.unwrap();
        }

        normalizer.normalize().await;

        while let Some(t) = rx2.recv().await {
            if t == Token::EndOfContent {
                result.push(Token::EndOfContent);
                break;
            }
            result.push(t);
        }

        result
    }
    #[tokio::test]
    async fn direct_eof() {
        black_box(normalize(vec![Token::EndOfContent]).await);
    }
    #[tokio::test]
    async fn second_is_eof() {
        black_box(normalize(vec![Token::Backslash, Token::EndOfContent]).await);
    }
    #[tokio::test]
    async fn all_tokens_returned() {
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
            ])
            .await,
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

    #[tokio::test]
    async fn exponent_split() {
        assert_eq!(
            normalize(vec![
                Token::NumberLiteral("2".to_owned().into()),
                Token::Caret,
                Token::NumberLiteral("025".to_owned().into()),
                Token::EndOfContent,
            ])
            .await,
            vec![
                Token::NumberLiteral("2".to_owned().into()),
                Token::Caret,
                Token::NumberLiteral("0".to_owned().into()),
                Token::NumberLiteral("25".to_owned().into()),
                Token::EndOfContent,
            ]
        );
    }

    #[tokio::test]
    async fn replace_cdot_with_asterisk() {
        assert_eq!(
            normalize(vec![
                Token::NumberLiteral("1".to_owned().into()),
                Token::Backslash,
                Token::Identifier("cdot".to_string()),
                Token::NumberLiteral("1".to_owned().into()),
                Token::EndOfContent,
            ])
            .await,
            vec![
                Token::NumberLiteral("1".to_owned().into()),
                Token::Asterisk,
                Token::NumberLiteral("1".to_owned().into()),
                Token::EndOfContent,
            ]
        );
    }

    #[tokio::test]
    async fn replace_cdotp_with_asterisk() {
        assert_eq!(
            normalize(vec![
                Token::NumberLiteral("1".to_owned().into()),
                Token::Backslash,
                Token::Identifier("cdotp".to_string()),
                Token::NumberLiteral("1".to_owned().into()),
                Token::EndOfContent,
            ])
            .await,
            vec![
                Token::NumberLiteral("1".to_owned().into()),
                Token::Asterisk,
                Token::NumberLiteral("1".to_owned().into()),
                Token::EndOfContent,
            ]
        );
    }

    #[tokio::test]
    async fn replace_times_with_asterisk() {
        assert_eq!(
            normalize(vec![
                Token::NumberLiteral("1".to_owned().into()),
                Token::Backslash,
                Token::Identifier("times".to_string()),
                Token::NumberLiteral("1".to_owned().into()),
                Token::EndOfContent,
            ])
            .await,
            vec![
                Token::NumberLiteral("1".to_owned().into()),
                Token::Asterisk,
                Token::NumberLiteral("1".to_owned().into()),
                Token::EndOfContent,
            ]
        );
    }

    #[tokio::test]
    async fn remove_left_middle_right() {
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
            ])
            .await,
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
}
