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
}
