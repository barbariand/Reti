use tokio::sync::mpsc::Receiver;

use crate::{ast::{mathexpr::{MathExpr, MathExprKey, Root}, AST}, lexer::Token, token_reader::TokenReader};

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken {
        expected: Token,
        found: Token,
    },
    ExpectedButNothingFound(Token),
    UnexpectedEnd,
    ExpectedEndOfFile,
    InvalidToken(Token),
}
trait Parseable{
    fn parse()
}

pub struct Parser {
    reader: TokenReader,
}

impl Parser {
    pub fn new(tokens: Receiver<Token>) -> Self {
        Parser { reader: TokenReader::new(tokens) }
    }

    pub async fn parse(&mut self) -> Result<AST, ParseError> {
        let root_expr = self.expr()?;
        // TODO detect trailing tokens, like what if we read an expression but then we found more tokens?
        Ok(AST::MathExpr(root_expr))
    }

    async fn read(&mut self) -> Result<Token, ParseError> {
        Ok(self.reader.read().await.ok_or(ParseError::UnexpectedEnd)?)
    }

    async fn peek(&mut self) -> Result<Token, ParseError> {
        Ok(self.reader.peek().await.ok_or(ParseError::UnexpectedEnd)?)
    }

    async fn skip(&mut self) {
        self.reader.skip();
    }

    async fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        let found = self.reader.read().await.ok_or(ParseError::UnexpectedEnd)?;
        if found == expected {
            return Ok(())
        }
        return Err(ParseError::UnexpectedToken { expected, found })
    }

    async fn expr(&mut self) -> Result<MathExpr, ParseError> {
        let token = self.read().await?;
        match token {
            Token::CommandPrefix => {
                let cmd = self.read().await?;
                match cmd.take_ident().ok_or(ParseError::UnexpectedToken { expected: Token::Ident("".to_string()), found: cmd })?.as_str() {
                    "sqrt"=>{
                        let mut degree: Option<MathExprKey> = None;
                            if self.peek().await? == Token::BracketBegin {
                                self.skip(); // skip [
                                    let degree_expr = self.expr().await?;
                            }
                            return Ok(MathExpr::Root(Root {
                                degree,
                                radicand: todo!(),
                            }))
                        }
                        _=>return Err(ParseError::)
                }
                if cmd.is_ident("sqrt") {
                    
                }
            }
            _ => todo!(),
        }
        todo!();
    }
}