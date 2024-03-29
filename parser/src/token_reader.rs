use tokio::sync::mpsc::Receiver;

use crate::lexer::Token;

pub struct TokenReader {
    tokens: Receiver<Token>,
    next: Option<Token>,
    eof: bool,
}

impl TokenReader {
    pub fn new(tokens: Receiver<Token>) -> Self {
        TokenReader {
            tokens,
            next: None,
            eof: false,
        }
    }

    pub async fn peek(&mut self) -> Option<Token> {
        // If we already had the next token, provide it.
        if let Some(token) = &self.next {
            return Some(token.clone());
        }
        // Read next token
        let token = match self.read().await {
            Some(token) => token,
            None => {
                // eof
                return None;
            }
        };
        // Store in next field to only peek the value without consuming it.
        self.next = Some(token.clone());
        return Some(token);
    }

    pub async fn read(&mut self) -> Option<Token> {
        if self.eof {
            return None;
        }
        // If we already had it peaked, just consume and return that.
        if let Some(token) = &self.next {
            let ugly_code = token.clone();
            self.next = None;
            return Some(ugly_code);
        }
        // Read from channel
        let token = self.tokens.recv().await.expect("Broken pipe");
        // Handle end of file
        if token == Token::EOF {
            self.eof = true;
            return None;
        }
        return Some(token);
    }

    pub fn skip(&mut self) {
        // Read but ignore value.
        _ = self.read();
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc::{self, Receiver, Sender};

    use crate::{lexer::Token, token_reader::TokenReader};

    #[tokio::test]
    async fn read_test() {
        let (tx, rx): (Sender<Token>, Receiver<Token>) = mpsc::channel(32);

        let tokens = vec![
            Token::ExpressionBegin,
            Token::Ident("5".to_string()),
            Token::ExpressionEnd,
        ];

        let mut reader = TokenReader::new(rx);

        for token in &tokens {
            tx.send(token.clone()).await.unwrap();
        }

        for token in tokens {
            assert_eq!(token, reader.read().await.unwrap());
        }
    }

    #[tokio::test]
    async fn peek_test() {
        let (tx, rx): (Sender<Token>, Receiver<Token>) = mpsc::channel(32);

        let tokens = vec![
            Token::CommandPrefix,
            Token::ExpressionBegin,
            Token::ExpressionEnd,
            Token::BracketBegin,
            Token::BracketEnd,
        ];

        let mut reader = TokenReader::new(rx);

        for token in &tokens {
            tx.send(token.clone()).await.unwrap();
        }

        assert_eq!(Token::CommandPrefix, reader.peek().await.unwrap());
        assert_eq!(Token::CommandPrefix, reader.read().await.unwrap());
        assert_eq!(Token::ExpressionBegin, reader.peek().await.unwrap());
        assert_eq!(Token::ExpressionBegin, reader.read().await.unwrap());
        assert_eq!(Token::ExpressionEnd, reader.read().await.unwrap());
        assert_eq!(Token::BracketBegin, reader.read().await.unwrap());
        assert_eq!(Token::BracketEnd, reader.read().await.unwrap());
    }
}
