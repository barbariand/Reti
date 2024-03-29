use std::mem::take;
use tokio::sync::mpsc::Sender;
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
struct Tokenizer {
    chanel: Sender<Token>,
}

impl Tokenizer {
    async fn tokenize(&self, s: &str) {
        let mut temp = String::new();
        for c in s.chars() {
            if let Some(t) = token(c, &mut temp) {
                if !temp.is_empty() {
                    self.chanel.send(Token::Ident(take(&mut temp))).await;
                }
                self.chanel.send(t).await;
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
        _ => {
            temp.push(c);
            return None;
        }
    })
}
