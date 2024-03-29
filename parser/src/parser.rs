use tokio::sync::mpsc::Receiver;

use crate::{ast::{mathexpr::{MathExpr, MathExprKey, Root}, AST}, lexer::Token, token_reader::TokenReader};
use std::boxed::Box;
use async_recursion::async_recursion;
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
#[derive(Hash,PartialEq, Eq)]
struct Test(Box<String>);

pub struct Parser {
    reader: TokenReader,
}

impl Parser {
    pub fn new(tokens: Receiver<Token>) -> Self {
        Parser { reader: TokenReader::new(tokens) }
    }

    pub async fn parse(&mut self) -> Result<AST, ParseError> {
        let root_expr = self.expr(None).await?;
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

    fn get_key(&mut self, expr: MathExpr) -> MathExprKey {
        todo!();
    }
    #[async_recursion]
    async fn expr(&mut self, until: Option<Token>) -> Result<MathExpr, ParseError> {
        // TODO figure out how to handle trailing implicit multiplication
        let token = self.read().await?;
        match token {
            Token::CommandPrefix => {
                let cmd = self.read().await?;
                match cmd.take_ident().ok_or(ParseError::UnexpectedToken { expected: Token::Ident("".to_string()), found: cmd.clone() })?.as_str() {
                    "sqrt" => {

                        // \sqrt{123}x
                        //       <mul>
                        //     /       \
                        // \sqrt{123}    x 
                        // let arr:Vec<MathExpr>=[123,sqrt(&1),x,mul(&2,&3)]
                        
                        // x=2
                        // [123,sqrt(&1),x,mul(&2,&3)]
                        // 123 vad värde? 123
                        // sqrt(&1) vad värde? vänta jag ska kolla med &1, hämtar 123 och tar5 sqrt mitt värde är 11
                        // x vad värde? 2
                        // mul(&2,&3) vad värde? vänta jag ska kolla med de andra? hämtar 123 och 2 mitt värde är 246
                        // jag är klar detta betyder att värde av allt är den sista som vi kollade
                        

                        // \sqrt{123x}x
                        //            <mul>
                        //          /       \
                        //      \sqrt        x 
                        //        |
                        //      <mul>
                        //        |
                        //     /     \
                        //    123    x
                        //[123,x,mul(&1,&2),]

                        //
                        // \sqrt{123}
                        // \sqrt[123]{123}
                        //
                        //self.state.push(MathExpr)->MathExperKey
                        let mut degree: Option<MathExprKey> = None;
                        if self.peek().await? == Token::BracketBegin {
                            // We found a square bracket containing the square root degree.

                            self.skip().await; // skip [
                            // Read expression for degree
                            let degree_expr = self.expr(Some(Token::BracketEnd)).await?;
                            self.expect(Token::BracketEnd).await?; // expect ]
                            degree = Some(self.get_key(degree_expr));
                        }

                        self.expect(Token::ExpressionBegin).await?;
                        let radicand_expr = self.expr(Some(Token::ExpressionEnd)).await?;
                        self.expect(Token::ExpressionEnd).await?;

                        let radicand = self.get_key(radicand_expr);

                        return Ok(MathExpr::Root(Root { degree, radicand, }));
                    }
                    _=>return Err(ParseError::UnexpectedEnd)
                }

            }
            _ => todo!(),
        }
        todo!();
    }
}