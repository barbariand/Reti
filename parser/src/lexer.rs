use std::mem::take;
use tokio::sync::mpsc::Sender;

#[derive(PartialEq, Eq)]
pub enum Token {
    Ident(String),
    CommandPrefix,
    ExpressionBegin,
    ExpressionEnd,
    BracketBegin,
    BracketEnd,
    Negative,
    Apostrofy,
    Underscore,
    Caret,
    Mul,
    Add,
    Div,
    VerticalPipe,
}
struct Lexer {
    chanel: Sender<Token>,
}

impl Lexer {
    async fn tokenize(&self, s: &str) {
        let mut temp = String::new();
        for c in s.chars() {
            if let Some(t) = token(c, &mut temp) {
                if !temp.is_empty() {
                    self.chanel.send(Token::Ident(take(&mut temp))).await.expect("Broken pipe");
                }
                self.chanel.send(t).await.expect("Broken pipe");
            }
        }
    }
}
fn token(c: char, temp: &mut String) -> Option<Token> {
    Some(match c {
        '\\' => Token::CommandPrefix,
        '{' => Token::ExpressionBegin,
        '}' => Token::ExpressionEnd,
        '[' => Token::BracketBegin,
        ']' => Token::BracketEnd,
        '-'=> Token::Negative,
        '\'' =>Token::Apostrofy,
        '_'=>Token::Underscore,
        '^'=>Token::Caret,
        '|'=>Token::VerticalPipe,
        '*'=>Token::Mul,
        '+'=>Token::Add,
        '/'=>Token::Div,
        ' '=>return None,
        _ => {
            temp.push(c);
            return None;
        }
    })
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc::{self, Receiver, Sender};

    use crate::lexer::{token, Lexer};

    use super::Token;

    async fn tokenize(text: &str) -> Vec<Token> {
        let (tx, rx): (Sender<Token>, Receiver<Token>) = mpsc::channel(32); // idk what that 32 means tbh
        let lexer = Lexer { chanel: tx };

        lexer.tokenize(text).await;

        todo!("Tokenize and give me the tokens in a Vec so I can compare them easialy pls ty");
    }

    #[test]
    async fn test1() {
        assert_eq!(
            tokenize("\\sqrt{1+2x}").await,
            vec![
                Token::CommandPrefix,
                Token::Ident("sqrt".to_string()),
                Token::ExpressionBegin,
                Token::Ident("1".to_string()),
                Token::Add,
                Token::Ident("2x".to_string()),
                Token::ExpressionEnd,
            ]
        );
    }
}