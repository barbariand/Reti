use std::mem::replace;

use tokio::sync::mpsc::{Receiver, Sender};

use crate::{token::Token, token_reader::TokenReader};

struct Normalizer {
    input: TokenReader,
    output: Sender<Token>,
}
impl Normalizer {
    fn new(input: Receiver<Token>, output: Sender<Token>) -> Self {
        Self {
            input: TokenReader::new(input),
            output,
        }
    }
    async fn normalize(&mut self) {
        let mut previous = self.input.read().await;
        if previous == Token::EndOfContent {
            self.send_or_crash(previous).await;
            return;
        }
        loop {
            let next = self.input.read().await;
            if next == Token::EndOfContent {
                self.send_or_crash(previous).await;
                self.send_or_crash(Token::EndOfContent).await;
                return;
            }
            match previous {
                Token::Backslash => {}
                Token::EndOfContent => unreachable!(),
                _ => todo!(),
            }
            self.send_or_crash(replace(&mut previous, next)).await;
        }
    }
    async fn send_or_crash(&self, token: Token) {
        self.output.send(token).await.expect("Broken Pipe")
    }
}

#[cfg(test)]
mod tests {
    use super::Normalizer;
    use crate::token::Token;
    use tokio::sync::mpsc::{self, Receiver, Sender};

    async fn normalize(tokens: Vec<Token>) -> Vec<Token> {
        let (tx1, rx1): (Sender<Token>, Receiver<Token>) = mpsc::channel(32);
        let (tx2, mut rx2): (Sender<Token>, Receiver<Token>) = mpsc::channel(32);
        let mut normalizer = Normalizer::new(rx1, tx2);

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
    async fn remove_left_right() {
        assert_eq!(
            normalize(vec![
                Token::Backslash,
                Token::Identifier("left".to_string()),
                Token::LeftParenthesis,
                Token::NumberLiteral("1".to_owned().into()),
                Token::Plus,
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
                Token::Plus,
                Token::NumberLiteral("1".to_owned().into()),
                Token::RightParenthesis,
                Token::EndOfContent,
            ]
        );
    }
}
