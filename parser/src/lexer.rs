use std::mem::take;
use tokio::sync::mpsc::Sender;
use crate::token::Token;

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
                '(' => Token::ParenthesisBegin,
                ')' => Token::ParenthesisEnd,
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
    async fn test_simple_sqrt() {
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
    async fn test_all_simple_operations() {
        assert_ne!(
            tokenize("-+*/").await,
            vec![
                Token::Negative,
                Token::Add,
                Token::Mul,
                Token::Div
            ]
        );
    }

    #[tokio::test]
    async fn test_single_character_tokens() {
        assert_eq!(
            tokenize("()[]{}^'|").await,
            vec![
                Token::ParenthesisBegin,
                Token::ParenthesisEnd,
                Token::BracketBegin,
                Token::BracketEnd,
                Token::ExpressionBegin,
                Token::ExpressionEnd,
                Token::Caret,
                Token::Apostrofy,
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
                Token::CommandPrefix,
                Token::Ident("pi".to_string()),
                Token::Ident("R".to_string()),
            ]
        );
        #[tokio::test]
        async fn test_complex_expressions() {
            assert_eq!(
                tokenize("{3.14*R^2}").await,
                vec![
                    Token::ExpressionBegin,
                    Token::NumberLiteral(3.14),
                    Token::Mul,
                    Token::Ident("R".to_string()),
                    Token::Caret,
                    Token::NumberLiteral(2.0),
                    Token::ExpressionEnd,
                ]
            );
        }
    }
    #[tokio::test]
    async fn test_number_followed_by_identifier() {
        assert_eq!(
            tokenize("42x + 3.14y").await,
            vec![
                Token::NumberLiteral(42.0),
                Token::Ident("x".to_string()),
                Token::Add,
                Token::NumberLiteral(3.14),
                Token::Ident("y".to_string()),
            ]
        );
    }
    #[tokio::test]
    async fn test_number_followed_by_command() {
        assert_eq!(
            tokenize("3.14\\piR").await,
            vec![
                Token::NumberLiteral(3.14),
                Token::CommandPrefix,
                Token::Ident("piR".to_string()),
            ]
        );
    }
    #[tokio::test]
    async fn test_mixed_number_and_text_sequences() {
        assert_eq!(
            tokenize("2a + 4b - 5\\sqrt{c}").await,
            vec![
                Token::NumberLiteral(2.0),
                Token::Ident("a".to_string()),
                Token::Add,
                Token::NumberLiteral(4.0),
                Token::Ident("b".to_string()),
                Token::Negative,
                Token::NumberLiteral(5.0),
                Token::CommandPrefix,
                Token::Ident("sqrt".to_string()),
                Token::ExpressionBegin,
                Token::Ident("c".to_string()),
                Token::ExpressionEnd,
            ]
        );
    }
}
