//!Helper methods for reading a stream of tokens
use tracing::{debug, trace, trace_span};

use crate::prelude::*;
use std::{collections::VecDeque, ops::RangeInclusive};
/// A struct for easier management of tokens
#[derive(Debug)]
pub struct TokenReader<I>
where
    I: IntoIterator<Item = Token>,
{
    ///The actual token stream
    tokens: I::IntoIter,
    ///The cached tokens
    next: VecDeque<Token>,
}

impl<I> TokenReader<I>
where
    I: IntoIterator<Item = Token>,
{
    /// Creates a new TokenReader
    pub fn new(tokens: I) -> Self {
        TokenReader {
            tokens: tokens.into_iter(),
            next: VecDeque::new(),
        }
    }

    /// Read the next token from the stream, and disregard the "next" queue.

    fn read_internal(&mut self) -> Option<Token> {
        let span = trace_span!("reading_tokens");
        let _enter = span.enter();
        let token = self.tokens.next()?;
        debug!("sending: {}", token);
        Some(token)
    }

    /// Look at the next token without consuming it.
    ///
    /// Equivalent to `peekn(0)`.
    ///
    /// If end of content is reached, `` will be returned for
    /// subsequent reads.
    pub fn peek(&mut self) -> Option<&Token> {
        self.peekn(0)
    }

    /// Look at the token a few steps away from the cursor.
    ///
    /// If end of content is reached, `` will be returned for
    /// subsequent reads.
    ///
    /// ## Panics
    /// If this method is called out of order, for example `peekn(1)`,
    /// `peekn(3)`, this method will panic since that is usually a sign of a
    /// bug.
    pub fn peekn(&mut self, n: usize) -> Option<&Token> {
        trace!("peeking:{}, length is:{}", n, self.next.len());
        if self.next.len() == n {
            let token = self.read_internal()?;
            self.next.push_back(token);
        }
        if self.next.len() < n {
            return None;
        }

        // Will never panic since we ensured the queue has enough elements.
        self.next.get(n)
    }

    /// Peek a range of tokens at once.
    pub fn peek_range(
        &mut self,
        range: RangeInclusive<usize>,
    ) -> Vec<Option<&Token>> {
        trace!("peeking:{:?}", range);
        // Ensure we have peeked the tokens.
        for n in range.clone() {
            self.peekn(n);
        }

        let mut vec = Vec::new();
        for n in range {
            vec.push(self.next.get(n));
        }
        vec
    }

    /// Read and consume the next token from the token stream.
    ///
    /// If end of file is reached, `Token::EOF` will be returned for subsequent
    /// reads.
    pub fn read(&mut self) -> Option<Token> {
        // If we already had it peeked, just consume and return that.
        self.next.pop_front().or_else(|| self.read_internal())
    }

    /// Consume the next token.
    pub fn skip(&mut self) {
        // Read but ignore value.
        _ = self.read();
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
    pub fn replace(
        &mut self,
        range: RangeInclusive<usize>,
        replacement: Vec<Token>,
    ) {
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
    use pretty_assertions::assert_eq;
    use tracing_test::traced_test;
    #[traced_test]
    #[test]
    fn read_test() {
        let tokens = vec![
            Token::LeftCurlyBracket,
            Token::Identifier("5".to_string()),
            Token::RightCurlyBracket,
        ];

        let mut reader = TokenReader::new(tokens.clone());

        for token in tokens {
            assert_eq!(token, reader.read().unwrap());
        }
    }

    #[traced_test]
    #[test]
    fn peek_test() {
        let tokens = vec![
            Token::Backslash,
            Token::LeftCurlyBracket,
            Token::RightCurlyBracket,
            Token::LeftBracket,
            Token::RightBracket,
        ];

        let mut reader = TokenReader::new(tokens);

        assert_eq!(Token::Backslash, reader.peek().unwrap());
        assert_eq!(Token::Backslash, reader.read().unwrap());
        assert_eq!(Token::LeftCurlyBracket, reader.peek().unwrap());
        assert_eq!(Token::LeftCurlyBracket, reader.read().unwrap());
        assert_eq!(Token::RightCurlyBracket, reader.read().unwrap());
        assert_eq!(Token::LeftBracket, reader.read().unwrap());
        assert_eq!(Token::RightBracket, reader.read().unwrap());
    }

    #[traced_test]
    #[test]
    fn peekn_test() {
        let tokens = vec![
            Token::Backslash,
            Token::LeftCurlyBracket,
            Token::RightCurlyBracket,
            Token::LeftBracket,
            Token::RightBracket,
        ];

        let mut reader = TokenReader::new(tokens.clone());

        assert_eq!(Token::Backslash, reader.peek().unwrap());
        assert_eq!(Token::LeftCurlyBracket, reader.peekn(1).unwrap());
        assert_eq!(Token::Backslash, reader.read().unwrap());
        assert_eq!(Token::LeftCurlyBracket, reader.peekn(0).unwrap());
        assert_eq!(Token::LeftCurlyBracket, reader.read().unwrap());
        assert_eq!(Token::RightCurlyBracket, reader.peekn(0).unwrap());
        assert_eq!(Token::RightCurlyBracket, reader.peekn(0).unwrap());
        assert_eq!(Token::RightCurlyBracket, reader.read().unwrap());
        assert_eq!(Token::LeftBracket, reader.read().unwrap());
        assert_eq!(Token::RightBracket, reader.read().unwrap());
    }
    #[traced_test]
    #[test]
    fn peek_read_end_of_content() {
        let v = vec![Token::Plus];
        let mut reader = TokenReader::new(v);

        assert_eq!(Token::Plus, reader.read().unwrap());
        assert_eq!(None, reader.read());
        for _ in 0..5 {
            assert_eq!(None, reader.peek());
        }
        for _ in 0..10 {
            assert_eq!(None, reader.read());
        }
        for i in 0..10 {
            assert_eq!(None, reader.peekn(i));
        }
    }

    #[should_panic]
    #[traced_test]
    #[test]
    fn jump_peek_panic() {
        let tokens = vec![
            Token::Backslash,
            Token::LeftCurlyBracket,
            Token::RightCurlyBracket,
            Token::LeftBracket,
            Token::RightBracket,
        ];

        let mut reader = TokenReader::new(tokens);

        assert_eq!(Token::Backslash, reader.peekn(0).unwrap());
        assert_eq!(Token::RightCurlyBracket, reader.peekn(2).unwrap());
    }

    #[traced_test]
    #[test]
    fn replace_test() {
        let tokens = vec![
            Token::LeftBracket,
            Token::Backslash,
            Token::Identifier("test".to_string()),
            Token::RightCurlyBracket,
            Token::LeftBracket,
            Token::RightBracket,
        ];

        let mut reader = TokenReader::new(tokens.clone());

        assert_eq!(Token::LeftBracket, reader.read().unwrap());
        assert_eq!(Token::Backslash, reader.peekn(0).unwrap());
        assert_eq!(
            Token::Identifier("test".to_string()),
            reader.peekn(1).unwrap()
        );
        reader.replace(0..=1, vec![Token::Plus, Token::Minus]);
        assert_eq!(Token::Plus, reader.read().unwrap());
        assert_eq!(Token::Minus, reader.read().unwrap());

        assert_eq!(Token::RightCurlyBracket, reader.read().unwrap());
        assert_eq!(Token::LeftBracket, reader.read().unwrap());
        assert_eq!(Token::RightBracket, reader.read().unwrap());
        assert_eq!(None, reader.read());
    }

    #[traced_test]
    #[test]
    fn replace_one() {
        let tokens = vec![Token::LeftBracket, Token::Plus, Token::RightBracket];

        let mut reader = TokenReader::new(tokens.clone());

        assert_eq!(Token::LeftBracket, reader.read().unwrap());
        assert_eq!(Token::Plus, reader.peekn(0).unwrap());
        reader.replace(0..=0, vec![Token::Minus]);
        assert_eq!(Token::Minus, reader.read().unwrap());
        assert_eq!(Token::RightBracket, reader.read().unwrap());
    }

    #[should_panic]
    #[traced_test]
    #[test]
    fn replace_without_peeking_panics() {
        let tokens = vec![
            Token::LeftBracket,
            Token::Backslash,
            Token::Identifier("left".to_string()),
            Token::RightCurlyBracket,
            Token::LeftBracket,
            Token::RightBracket,
        ];

        let mut reader = TokenReader::new(tokens.clone());

        assert_eq!(Token::LeftBracket, reader.read().unwrap());
        reader.replace(0..=1, vec![Token::Asterisk]);
    }
}
