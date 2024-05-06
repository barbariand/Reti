use tracing::{debug, trace_span};

use crate::prelude::*;
use std::{collections::VecDeque, ops::RangeInclusive};
/// A struct for easeier management of tokens
#[derive(Debug)]
pub struct TokenReader {
    tokens: TokenReceiver,
    next: VecDeque<Token>,
    eof: bool,
}

impl TokenReader {
    /// Creates a new TokenReader
    pub fn new(tokens: TokenReceiver) -> Self {
        TokenReader {
            tokens,
            next: VecDeque::new(),
            eof: false,
        }
    }

    /// Read the next token from the stream, and disregard the "next" queue.

    async fn read_internal(&mut self) -> Token {
        let span = trace_span!("reading_tokens");
        let _enter = span.enter();
        if self.eof {
            debug!("sending: {}", Token::EndOfContent);
            return Token::EndOfContent;
        }
        let token = self.tokens.recv().await.expect("Broken pipe");
        // Handle end of file
        if token == Token::EndOfContent {
            self.eof = true;
            debug!("sending: {}", Token::EndOfContent);
            return Token::EndOfContent;
        }
        debug!("sending: {}", token);
        token
    }

    /// Look at the next token without consuming it.
    ///
    /// Equivalent to `peekn(0)`.
    ///
    /// If end of content is reached, `Token::EndOfContent` will be returned for
    /// subsequent reads.
    pub async fn peek(&mut self) -> &Token {
        self.peekn(0).await
    }

    /// Look at the token a few steps away from the cursor.
    ///
    /// If end of content is reached, `Token::EndOfContent` will be returned for
    /// subsequent reads.
    ///
    /// ## Panics
    /// If this method is called out of order, for example `peekn(1)`,
    /// `peekn(3)`, this method will panic since that is usually a sign of a
    /// bug.
    pub async fn peekn(&mut self, n: usize) -> &Token {
        if self.next.len() == n {
            let token = self.read_internal().await;
            self.next.push_back(token);
        }
        if self.next.len() < n {
            panic!(
                "Jump peek detected. This is usually a bug. \
                Previous peek: {:?}, this peek: {}",
                self.next.len().checked_sub(1),
                n
            );
        }

        // Will never panic since we ensured the queue has enough elements.
        &self.next[n]
    }

    /// Peek a range of tokens at once.
    pub async fn peek_range(&mut self, range: RangeInclusive<usize>) -> Vec<&Token> {
        // Ensure we have peeked the tokens.
        for n in range.clone() {
            self.peekn(n).await;
        }

        let mut vec = Vec::new();
        for n in range {
            vec.push(&self.next[n]);
        }
        vec
    }

    /// Read and consume the next token from the token stream.
    ///
    /// If end of file is reached, `Token::EOF` will be returned for subsequent
    /// reads.
    pub async fn read(&mut self) -> Token {
        // If we already had it peeked, just consume and return that.
        if let Some(token) = self.next.pop_front() {
            return token;
        }
        // Read from channel
        self.read_internal().await
    }

    /// Consume the next token.
    pub async fn skip(&mut self) {
        // Read but ignore value.
        _ = self.read().await;
    }

    /// Replace a range of tokens that have been peeked with a vector of
    /// replacements.
    ///
    /// ## Examples
    /// ```ignore
    /// peeked tokens before:
    /// [1, Backslash, "cdot", Two]
    ///
    /// replace(1..=2, vec![Asterisk])
    ///
    /// peeked tokens after:
    /// [1, Asterisk, Two]
    /// ```
    ///
    /// ## Panics
    /// You must peek tokens before calling replace. In other words, you need to
    /// know what you are replacing before calling this function.
    pub async fn replace(&mut self, range: RangeInclusive<usize>, replacement: Vec<Token>) {
        if self.next.len() <= *range.end() {
            panic!(
                "Please call peekn before calling replace. You must know what you are replacing! range = {:?}",
                range
            );
        }
        let start = *range.start();
        for _ in range.clone() {
            // Always remove start index because it shifts elements down.
            self.next.remove(start).expect("length already checked");
        }
        for token in replacement.into_iter().rev() {
            self.next.insert(*range.start(), token);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[tokio::test]
    async fn read_test() {
        let (tx, rx): (TokenSender, TokenReceiver) = mpsc::channel(32);

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
        let (tx, rx): (TokenSender, TokenReceiver) = mpsc::channel(32);

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

    #[tokio::test]
    async fn peekn_test() {
        let (tx, rx): (TokenSender, TokenReceiver) = mpsc::channel(32);

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
        assert_eq!(Token::LeftCurlyBracket, reader.peekn(1).await);
        assert_eq!(Token::Backslash, reader.read().await);
        assert_eq!(Token::LeftCurlyBracket, reader.peekn(0).await);
        assert_eq!(Token::LeftCurlyBracket, reader.read().await);
        assert_eq!(Token::RightCurlyBracket, reader.peekn(0).await);
        assert_eq!(Token::RightCurlyBracket, reader.peekn(0).await);
        assert_eq!(Token::RightCurlyBracket, reader.read().await);
        assert_eq!(Token::LeftBracket, reader.read().await);
        assert_eq!(Token::RightBracket, reader.read().await);
    }

    #[tokio::test]
    async fn peek_read_end_of_content() {
        let (tx, rx): (TokenSender, TokenReceiver) = mpsc::channel(32);

        let mut reader = TokenReader::new(rx);
        tx.send(Token::Plus).await.unwrap();
        tx.send(Token::EndOfContent).await.unwrap();

        assert_eq!(Token::Plus, reader.read().await);
        assert_eq!(Token::EndOfContent, reader.read().await);
        for _ in 0..5 {
            assert_eq!(Token::EndOfContent, reader.peek().await);
        }
        for _ in 0..10 {
            assert_eq!(Token::EndOfContent, reader.read().await);
        }
        for i in 0..10 {
            assert_eq!(Token::EndOfContent, reader.peekn(i).await);
        }
    }

    #[should_panic]
    #[tokio::test]
    async fn jump_peek_panic() {
        let (tx, rx): (TokenSender, TokenReceiver) = mpsc::channel(32);

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

        assert_eq!(Token::Backslash, reader.peekn(0).await);
        assert_eq!(Token::RightCurlyBracket, reader.peekn(2).await);
    }

    #[tokio::test]
    async fn replace_test() {
        let (tx, rx): (TokenSender, TokenReceiver) = mpsc::channel(32);

        let tokens = vec![
            Token::LeftBracket,
            Token::Backslash,
            Token::Identifier("test".to_string()),
            Token::RightCurlyBracket,
            Token::LeftBracket,
            Token::RightBracket,
            Token::EndOfContent,
        ];

        let mut reader = TokenReader::new(rx);

        for token in &tokens {
            tx.send(token.clone()).await.unwrap();
        }

        assert_eq!(Token::LeftBracket, reader.read().await);
        assert_eq!(Token::Backslash, reader.peekn(0).await);
        assert_eq!(Token::Identifier("test".to_string()), reader.peekn(1).await);
        reader.replace(0..=1, vec![Token::Plus, Token::Minus]).await;
        assert_eq!(Token::Plus, reader.read().await);
        assert_eq!(Token::Minus, reader.read().await);

        assert_eq!(Token::RightCurlyBracket, reader.read().await);
        assert_eq!(Token::LeftBracket, reader.read().await);
        assert_eq!(Token::RightBracket, reader.read().await);
        assert_eq!(Token::EndOfContent, reader.read().await);
    }

    #[tokio::test]
    async fn replace_one() {
        let (tx, rx): (TokenSender, TokenReceiver) = mpsc::channel(32);

        let tokens = vec![Token::LeftBracket, Token::Plus, Token::RightBracket];

        let mut reader = TokenReader::new(rx);

        for token in &tokens {
            tx.send(token.clone()).await.unwrap();
        }

        assert_eq!(Token::LeftBracket, reader.read().await);
        assert_eq!(Token::Plus, reader.peekn(0).await);
        reader.replace(0..=0, vec![Token::Minus]).await;
        assert_eq!(Token::Minus, reader.read().await);
        assert_eq!(Token::RightBracket, reader.read().await);
    }

    #[should_panic]
    #[tokio::test]
    async fn replace_without_peeking_panics() {
        let (tx, rx): (TokenSender, TokenReceiver) = mpsc::channel(32);

        let tokens = vec![
            Token::LeftBracket,
            Token::Backslash,
            Token::Identifier("left".to_string()),
            Token::RightCurlyBracket,
            Token::LeftBracket,
            Token::RightBracket,
            Token::EndOfContent,
        ];

        let mut reader = TokenReader::new(rx);

        for token in &tokens {
            tx.send(token.clone()).await.unwrap();
        }

        assert_eq!(Token::LeftBracket, reader.read().await);
        reader.replace(0..=1, vec![Token::Asterisk]).await;
    }
}
