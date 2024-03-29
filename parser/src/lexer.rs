use std::mem::take;
use tokio::sync::mpsc::Sender;

#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    Ident(String),
    NumberLiteral(f64),
    CommandPrefix,
    ExpressionBegin, // {
    ExpressionEnd,   // }
    BracketBegin,    // [
    BracketEnd,      // ]
    ParenthesisBegin,
    ParenthesisEnd,
    Negative,
    Apostrofy,
    Underscore,
    Caret,
    Mul,
    Add,
    Div,
    VerticalPipe,
    EOF,
}

impl Token {
    pub fn is_ident(&self, text: &str) -> bool {
        match self {
            Self::Ident(val) => val == text,
            _ => false,
        }
    }
    pub fn take_ident(&self) -> Option<&String> {
        match self {
            Self::Ident(v) => Some(v),
            _ => None,
        }
    }
}

struct Lexer {
    chanel: Sender<Token>,
}

impl Lexer {
    async fn send_or_crach(&self, token: Token) {
        self.chanel.send(token).await.expect("Broken Pipe")
    }
    async fn tokenize(&self, s: &str) {
        let mut temp_ident = String::new();
        let mut temp_number = String::new();
        let mut latest_was_ident = true;
        for c in s.chars() {
            let t = match c {
                '0'..='9' | '.' => {
                    if !temp_ident.is_empty() {
                        self.send_or_crach(Token::Ident(take(&mut temp_ident)))
                            .await;
                    }
                    temp_number.push(c);
                    continue;
                }
                '\\' => Token::CommandPrefix,
                '{' => Token::ExpressionBegin,
                '}' => Token::ExpressionEnd,
                '[' => Token::BracketBegin,
                ']' => Token::BracketEnd,
                '-' => Token::Negative,
                '\'' => Token::Apostrofy,
                '_' => Token::Underscore,
                '^' => Token::Caret,
                '|' => Token::VerticalPipe,
                '*' => Token::Mul,
                '+' => Token::Add,
                '/' => Token::Div,
                ' ' => {
                    if !temp_number.is_empty() {
                        let num = Token::NumberLiteral(
                            temp_number
                                .parse()
                                .expect("THIS NEEDS FIXING IT FAILED TO PARSE NUMBER"),
                        );
                        temp_number = String::new();
                        self.send_or_crach(num).await;
                    }
                    if !temp_ident.is_empty() {
                        self.send_or_crach(Token::Ident(take(&mut temp_ident)))
                            .await;
                    }
                    continue;
                }
                _ => {
                    if !temp_number.is_empty() {
                        let num = Token::NumberLiteral(
                            temp_number
                                .parse()
                                .expect("THIS NEEDS FIXING IT FAILED TO PARSE NUMBER"),
                        );
                        temp_number = String::new();
                        self.send_or_crach(num).await;
                    }
                    temp_ident.push(c);
                    continue;
                }
            };
            if !temp_number.is_empty() {
                let num = Token::NumberLiteral(
                    temp_number
                        .parse()
                        .expect("THIS NEEDS FIXING IT FAILED TO PARSE NUMBER"),
                );
                temp_number = String::new();
                self.send_or_crach(num).await;
            }
            if !temp_ident.is_empty() {
                self.send_or_crach(Token::Ident(take(&mut temp_ident)))
                    .await;
            }

            self.send_or_crach(t).await;
        }
        self.send_or_crach(Token::EOF).await;
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc::{self, Receiver, Sender};

    use crate::lexer::Lexer;

    use super::Token;

    async fn tokenize(text: &str) -> Vec<Token> {
        let (tx, mut rx): (Sender<Token>, Receiver<Token>) = mpsc::channel(32); // idk what that 32 means tbh
        let lexer = Lexer { chanel: tx };

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
    async fn test1() {
        assert_eq!(
            tokenize("\\sqrt{1+2x}").await,
            vec![
                Token::CommandPrefix,
                Token::Ident("sqrt".to_string()),
                Token::ExpressionBegin,
                Token::NumberLiteral(1.0),
                Token::Add,
                Token::NumberLiteral(2.0),
                Token::Ident("x".to_string()),
                Token::ExpressionEnd,
            ]
        );
    }
    #[tokio::test]
    async fn test1_fail() {
        assert_ne!(
            tokenize("\\sqrt{1+2x}").await,
            vec![
                Token::Ident("sqrt".to_string()),
                Token::ExpressionBegin,
                Token::NumberLiteral(1.0),
                Token::Add,
                Token::NumberLiteral(2.0),
                Token::Ident("x".to_string()),
                Token::ExpressionEnd,
            ]
        );
    }

    #[tokio::test]
    async fn benchsqrt() {
        assert_eq!(
            tokenize("\\sqrt{1+2x}").await,
            vec![
                Token::CommandPrefix,
                Token::Ident("sqrt".to_string()),
                Token::ExpressionBegin,
                Token::NumberLiteral(1.0),
                Token::Add,
                Token::NumberLiteral(2.0),
                Token::Ident("x".to_string()),
                Token::ExpressionEnd,
            ]
        );
    }
}
