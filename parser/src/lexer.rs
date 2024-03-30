use crate::token::Token;
use std::mem::take;
use tokio::sync::mpsc::Sender;

pub struct Lexer {
    channel: Sender<Token>,
}

impl Lexer {
    pub fn new(channel: Sender<Token>) -> Self {
        Self { channel }
    }
    async fn send_or_crash(&self, token: Token) {
        self.channel.send(token).await.expect("Broken Pipe")
    }
    pub async fn tokenize(&self, s: &str) {
        let mut temp_ident = String::new();
        let mut temp_number = String::new();
        let mut latest_was_ident = true;
        for c in s.chars() {
            let t = match c {
                '0'..='9' | '.' => {
                    if !temp_ident.is_empty() {
                        self.send_or_crash(Token::Identifier(take(&mut temp_ident)))
                            .await;
                    }
                    temp_number.push(c);
                    continue;
                }
                '\\' => Token::Backslash,
                '{' => Token::LeftCurlyBracket,
                '}' => Token::RightCurlyBracket,
                '[' => Token::LeftBracket,
                ']' => Token::RightBracket,
                '-' => Token::Minus,
                '\'' => Token::Apostrophe,
                '_' => Token::Underscore,
                '^' => Token::Caret,
                '|' => Token::VerticalPipe,
                '*' => Token::Asterisk,
                '+' => Token::Plus,
                '/' => Token::Slash,
                '(' => Token::LeftParen,
                ')' => Token::RightParen,
                ' ' => {
                    if !temp_number.is_empty() {
                        let num = Token::NumberLiteral(
                            temp_number.into(),
                        );
                        temp_number = String::new();
                        self.send_or_crash(num).await;
                    }
                    if !temp_ident.is_empty() {
                        self.send_or_crash(Token::Identifier(take(&mut temp_ident)))
                            .await;
                    }
                    continue;
                }
                _ => {
                    if !temp_number.is_empty() {
                        let num = Token::NumberLiteral(
                            temp_number
                                .into(),
                        );
                        temp_number = String::new();
                        self.send_or_crash(num).await;
                    }
                    temp_ident.push(c);
                    continue;
                }
            };
            if !temp_number.is_empty() {
                let num = Token::NumberLiteral(
                    temp_number.into(),
                );
                temp_number = String::new();
                self.send_or_crash(num).await;
            }
            if !temp_ident.is_empty() {
                self.send_or_crash(Token::Identifier(take(&mut temp_ident)))
                    .await;
            }

            self.send_or_crash(t).await;
        }
        if !temp_number.is_empty() {
            let num = Token::NumberLiteral(
                temp_number
                    .parse()
                    .expect("THIS NEEDS FIXING IT FAILED TO PARSE NUMBER"),
            );
            temp_number = String::new();
            self.send_or_crash(num).await;
        }
        if !temp_ident.is_empty() {
            self.send_or_crash(Token::Identifier(take(&mut temp_ident)))
                .await;
        }
        self.send_or_crash(Token::EOF).await;
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc::{self, Receiver, Sender};

    use crate::lexer::Lexer;

    use super::Token;

    async fn tokenize(text: &str) -> Vec<Token> {
        let (tx, mut rx): (Sender<Token>, Receiver<Token>) = mpsc::channel(32); // idk what that 32 means tbh
        let lexer = Lexer::new(tx);

        lexer.tokenize(text).await;

        let mut vec = Vec::new();
        while let Some(t) = rx.recv().await {
            if t == Token::EOF {
                break;
            }
            vec.push(t);
        }
        vec
    }

    #[tokio::test]
    async fn test_simple_sqrt() {
        assert_eq!(
            tokenize("\\sqrt{1+2x}").await,
            vec![
                Token::Backslash,
                Token::Identifier("sqrt".to_string()),
                Token::LeftCurlyBracket,
                Token::NumberLiteral(1.0),
                Token::Plus,
                Token::NumberLiteral(2.0),
                Token::Identifier("x".to_string()),
                Token::RightCurlyBracket,
            ]
        );
    }
    #[tokio::test]
    async fn test_all_simple_operations() {
        assert_eq!(
            tokenize("-+*/").await,
            vec![Token::Minus, Token::Plus, Token::Asterisk, Token::Slash]
        );
    }

    #[tokio::test]
    async fn test_single_character_tokens() {
        assert_eq!(
            tokenize("()[]{}^'|").await,
            vec![
                Token::LeftParen,
                Token::RightParen,
                Token::LeftBracket,
                Token::RightBracket,
                Token::LeftCurlyBracket,
                Token::RightCurlyBracket,
                Token::Caret,
                Token::Apostrophe,
                Token::VerticalPipe,
            ]
        );
    }
    #[tokio::test]
    async fn test_number_literals() {
        assert_eq!(
            tokenize("3.14 42").await,
            vec![Token::NumberLiteral(3.14), Token::NumberLiteral(42.0),]
        );
    }
    #[tokio::test]
    async fn test_identifiers_and_commands() {
        assert_eq!(
            tokenize("\\pi R").await,
            vec![
                Token::Backslash,
                Token::Identifier("pi".to_string()),
                Token::Identifier("R".to_string()),
            ]
        );
    }
    #[tokio::test]
    async fn test_complex_expressions() {
        assert_eq!(
            tokenize("{3.14*R^2}").await,
            vec![
                Token::LeftCurlyBracket,
                Token::NumberLiteral(3.14),
                Token::Asterisk,
                Token::Identifier("R".to_string()),
                Token::Caret,
                Token::NumberLiteral(2.0),
                Token::RightCurlyBracket,
            ]
        );
    }
    #[tokio::test]
    async fn test_number_followed_by_identifier() {
        assert_eq!(
            tokenize("42x + 3.14y").await,
            vec![
                Token::NumberLiteral(42.0),
                Token::Identifier("x".to_string()),
                Token::Plus,
                Token::NumberLiteral(3.14),
                Token::Identifier("y".to_string()),
            ]
        );
    }
    #[tokio::test]
    async fn test_number_followed_by_command() {
        assert_eq!(
            tokenize("3.14\\piR").await,
            vec![
                Token::NumberLiteral(3.14),
                Token::Backslash,
                Token::Identifier("piR".to_string()),
            ]
        );
    }
    #[tokio::test]
    async fn test_mixed_number_and_text_sequences() {
        assert_eq!(
            tokenize("2a + 4b - 5\\sqrt{c}").await,
            vec![
                Token::NumberLiteral(2.0),
                Token::Identifier("a".to_string()),
                Token::Plus,
                Token::NumberLiteral(4.0),
                Token::Identifier("b".to_string()),
                Token::Minus,
                Token::NumberLiteral(5.0),
                Token::Backslash,
                Token::Identifier("sqrt".to_string()),
                Token::LeftCurlyBracket,
                Token::Identifier("c".to_string()),
                Token::RightCurlyBracket,
            ]
        );
    }
}
