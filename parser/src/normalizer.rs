use std::{collections::VecDeque, future::Future, mem::replace, ops::ControlFlow};

use tokio::sync::mpsc::{Receiver, Sender};

use crate::{token::Token, token_reader::TokenReader};

struct Normalizer {
    queue: QueueSender,
}
impl Normalizer {
    fn new(input: Receiver<Token>, output: Sender<Token>) -> Self {
        Self {
            queue: QueueSender {
                input: TokenReader::new(input),
                queue: None,
                output,
            },
        }
    }
    async fn normalize(&mut self) {
        if self.queue.start_read().await.is_break() {
            return;
        }
        loop {
            if self.normalize_tokens().await.is_break() {
                return;
            }
            if self.queue.read_into_queue().await.is_break() {
                return;
            }
        }
    }
    async fn normalize_tokens(&mut self) -> ControlFlow<()> {
        match self.queue.get_toks() {
            (Token::Backslash, Token::Identifier(v)) => match v.as_str() {
                "cdot" | "cdotp" | "times" => {
                    return self.queue.replace_queued_with(Token::Asterisk).await
                }
                "left" | "right" => return self.queue.replace_queue().await,
                _ => {}
            },
            (Token::Caret, Token::NumberLiteral(n)) => {
                if n.raw.len() == 0 {
                    panic!("string is wierd");
                }
                if n.raw.len() != 1 {
                    let mut s = n.raw.clone();
                    let new2 = Token::NumberLiteral(s.split_off(1).into());
                    let new1 = Token::NumberLiteral(s.into());
                    self.queue.replace_second_with(new1);
                    self.queue.continue_queue_with(new2).await;
                }
            }
            _ => {}
        }
        ControlFlow::Continue(())
    }
}

struct QueueSender {
    input: TokenReader,
    queue: Option<[Token; 2]>,
    output: Sender<Token>,
}
impl QueueSender {
    async fn start_read(&mut self) -> ControlFlow<()> {
        if self.queue.is_some() {
            panic!("Started reading while still reading")
        }
        let first = self.input.read().await;
        if first.is_eof() {
            self.send_or_crash(first).await;
            return ControlFlow::Break(());
        }
        let second = self.input.read().await;
        if second.is_eof() {
            self.send_or_crash(first).await;
            self.send_or_crash(second).await;
            return ControlFlow::Break(());
        }

        self.queue = Some([first, second]);
        ControlFlow::Continue(())
    }
    pub async fn replace_queued_with(&mut self, tok: Token) -> ControlFlow<()> {
        self.move_one_step(tok);
        let next = self.read().await;
        if next.is_eof() {
            self.empty_queue().await;
            self.send_or_crash(Token::EndOfContent).await;
            return ControlFlow::Break(());
        }
        self.move_one_step(next);
        ControlFlow::Continue(())
    }
    pub async fn replace_queue(&mut self) -> ControlFlow<()> {
        self.queue = None;
        self.start_read().await
    }
    fn get_queue_or_crash(&mut self) -> &mut [Token; 2] {
        self.queue.as_mut().expect("the queue was not constructed")
    }
    fn get_toks(&mut self) -> (&Token, &Token) {
        let queue = self.get_queue_or_crash();

        (&queue[0], &queue[1])
    }
    pub fn move_one_step(&mut self, token: Token) -> Token {
        let queue = self.get_queue_or_crash();
        let temp = replace(&mut queue[1], token);
        replace(&mut queue[0], temp)
    }
    pub async fn read_into_queue(&mut self) -> ControlFlow<()> {
        let tok = self.input.read().await;
        let retur = tok.is_eof();
        let ret = self.move_one_step(tok);
        self.send_or_crash(ret).await;
        match retur {
            true => {
                self.empty_queue().await;
                ControlFlow::Break(())
            }
            false => ControlFlow::Continue(()),
        }
    }
    pub async fn continue_queue_with(&mut self, next: Token) {
        let temp = self.move_one_step(next);
        self.send_or_crash(temp).await;
    }
    async fn read(&mut self) -> Token {
        self.input.read().await
    }
    async fn send_or_crash(&self, token: Token) {
        self.output.send(token).await.expect("Broken Pipe")
    }
    async fn empty_queue(&mut self) {
        let queue = self.queue.take().expect("the queue was not constructed");
        for t in queue.into_iter() {
            self.send_or_crash(t).await;
        }
    }
    pub fn replace_second_with(&mut self, token: Token) {
        self.get_queue_or_crash()[1] = token;
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
