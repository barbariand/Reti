use std::collections::VecDeque;

use tokio::sync::mpsc::{Receiver, Sender};

use crate::{token::Token, token_reader::TokenReader};

struct Normalizer {
    input: TokenReader,
    output: Sender<Token>,
    queue: VecDeque<Token>,
}
impl Normalizer {
    fn new(input: Receiver<Token>, output: Sender<Token>) -> Self {
        Self {
            input: TokenReader::new(input),
            output,
            queue: VecDeque::new(),
        }
    }
    async fn normalize(&mut self) {
        if self.read_into_queue().await {
            self.empty_queue().await;
            return;
        }
        if self.read_into_queue().await {
            self.empty_queue().await;
            return;
        }
        self.normalize_tokens(0, 1).await;
        loop {
            let is_end = self.read_into_queue().await;

            self.normalize_tokens(1, 2).await;
            if is_end {
                self.empty_queue().await;
                return;
            } else {
                self.send_last_queued_items().await;
            }
        }
    }
    async fn normalize_tokens(&mut self, tok: usize, tok2: usize) {
        match (&self.queue[tok], &self.queue[tok2]) {
            (Token::Backslash, Token::Identifier(v)) => match v.as_str() {
                "cdot" | "cdotp" | "times" => {
                    self.replace_2_last_and_get_new(Token::Asterisk).await
                }
                "left" | "right" => self.replace_2_last_with_new().await,
                _ => {}
            },
            _ => {}
        }
    }
    async fn replace_2_last_with_new(&mut self) {
        let _ = self.queue.pop_back().expect("Queue has changed size");
        let _ = self.queue.pop_back().expect("Queue has changed size");
        self.read_into_queue().await;
        self.read_into_queue().await;
    }
    async fn replace_2_last_and_get_new(&mut self, tok: Token) {
        let _ = self.queue.pop_back().expect("Queue has changed size");
        let _ = self.queue.pop_back().expect("Queue has changed size");
        self.queue.push_back(tok);
        self.read_into_queue().await;
    }
    async fn read_into_queue(&mut self) -> bool {
        let tok = self.input.read().await;
        let ret = tok.is_eof();
        self.queue.push_back(tok);
        ret
    }
    async fn empty_queue(&mut self) {
        while let Some(tok) = self.queue.pop_front() {
            self.send_or_crash(tok).await;
        }
    }
    async fn send_last_queued_items(&mut self) {
        let tok = self.queue.pop_front().expect("Queue is empty");
        self.send_or_crash(tok).await;
    }
    async fn send_or_crash(&self, token: Token) {
        self.output.send(token).await.expect("Broken Pipe")
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
