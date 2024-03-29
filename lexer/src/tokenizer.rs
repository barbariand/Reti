use std::{
    io::{BufRead, Read},
    mem::{replace, take},
};

pub enum Token {
    CommandPrefix,
    ExpressionBegin,
    ExpressionEnd,
    BracketBegin,
    BracketEnd,
    NumberLiteral(f64),
    Constants,
    StringLiteral(String),
    Negative,
    Apostrofy,
    Underscore,
    Caret,
    VerticalPipe,
}
struct Tokenizer {}

impl Tokenizer {
    fn tokenize(s: &str) -> Vec<Token> {
        let mut temp = String::new();
        let mut tokens = Vec::new();
        for c in s.chars() {
            if let Some(t) = token(c, &mut temp) {
                 if !temp.is_empty() {
                    tokens.push(Token::StringLiteral(take(&mut temp)))
                }
                tokens.push(t);
            }
        }
        tokens
    }
}
fn token(c: char, temp: &mut String) -> Option<Token> {
    Some(match c {
        '\\' => Token::CommandPrefix,
        '{' => Token::ExpressionBegin,
        '}' => Token::ExpressionEnd,
        '[' => Token::BracketBegin,
        ']' => Token::BracketEnd,
        _ => {
            temp.push(c);
            return None;
        }
    })
}
