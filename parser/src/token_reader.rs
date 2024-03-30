
use tokio::sync::mpsc::Receiver;

use crate::token::Token;

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

    /// Look at the next token without consuming it.
    ///
    /// If end of file is reached, `Token::EOF` will be returned for subsequent
    /// reads.
    pub async fn peek(& mut self) -> & Token {
        // If we already had the next token, provide it.
        if self.next.is_none() {
        self.next = Some(self.read().await);
        }
        
        self.next.as_ref().expect("Memory Corruption")
    }

    /// Read and consume the next token from the token stream.
    ///
    /// If end of file is reached, `Token::EOF` will be returned for subsequent
    /// reads.
    pub async fn read(&mut self) -> Token {
        if self.eof {
            return Token::EndOfContent;
        }
        // If we already had it peaked, just consume and return that.
        if let Some(token) = &self.next {
            let ugly_code = token.clone();
            self.next = None;
            return ugly_code;
        }
        // Read from channel
        let token = self.tokens.recv().await.expect("Broken pipe");
        // Handle end of file
        if token == Token::EndOfContent {
            self.eof = true;
            return Token::EndOfContent;
        }
        token
    }

    /// Consume the next token.
    pub async fn skip(&mut self) {
        // Read but ignore value.
        _ = self.read().await;
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc::{self, Receiver, Sender};

    use crate::{token::Token, token_reader::TokenReader};

    #[tokio::test]
    async fn read_test() {
        let (tx, rx): (Sender<Token>, Receiver<Token>) = mpsc::channel(32);

        let tokens = vec![
            Token::LeftCurlyBracket,
            Token::Identifier("5".to_string()),
            Token::RightCurlyBracket,
        ];

        let mut reader = TokenReader::new(rx);

        for token in &tokens {
            tx.send(token.clone()).await.unwrap();
        }

        for token in tokens {
            assert_eq!(token, reader.read().await);
        }
    }

    #[tokio::test]
    async fn peek_test() {
        let (tx, rx): (Sender<Token>, Receiver<Token>) = mpsc::channel(32);

        let tokens = vec![
            Token::Backslash,
            Token::LeftCurlyBracket,
            Token::RightCurlyBracket,
            Token::LeftBracket,
            Token::RightBracket,
        ];

        let mut reader = TokenReader::new(rx);

        for token in &tokens {
            tx.send(token.clone()).await.unwrap();
        }

        assert_eq!(Token::Backslash, reader.peek().await);
        assert_eq!(Token::Backslash, reader.read().await);
        assert_eq!(Token::LeftCurlyBracket, reader.peek().await);
        assert_eq!(Token::LeftCurlyBracket, reader.read().await);
        assert_eq!(Token::RightCurlyBracket, reader.read().await);
        assert_eq!(Token::LeftBracket, reader.read().await);
        assert_eq!(Token::RightBracket, reader.read().await);
    }
}
