use tokio::sync::mpsc::Receiver;

use crate::{
    ast::{
        mathexpr::{MathExpr, MathExprKey, Root},
        AST,
    },
    token::Token,
    token_reader::TokenReader,
};
use async_recursion::async_recursion;
use std::boxed::Box;
#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken { expected: Token, found: Token },
    ExpectedButNothingFound(Token),
    UnexpectedEnd,
    ExpectedEndOfFile,
    InvalidToken(Token),
}
#[derive(Hash, PartialEq, Eq)]
struct Test(Box<String>);
// TODO make this compile
// pub(crate) trait Parsable {
//     #[allow(async_fn_in_trait)]
//     async fn parse(reader: &mut Parser) -> Result<Self, ParseError>
//     where
//         Self: Sized;
// }
pub struct Parser {
    pub(crate) reader: TokenReader,
}

impl Parser {
    pub fn new(tokens: Receiver<Token>) -> Self {
        Parser {
            reader: TokenReader::new(tokens),
        }
    }

    pub async fn parse(&mut self) -> Result<AST, ParseError> {
        let root_expr = self.expr().await?;
        // TODO detect trailing tokens, like what if we read an expression but then we found more tokens?
        Ok(AST::MathExpr(root_expr))
    }

    pub(crate) async fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        let found = self.reader.read().await;
        if found == expected {
            return Ok(());
        }
        return Err(ParseError::UnexpectedToken { expected, found });
    }

    pub(crate) fn get_key(&mut self, _expr: MathExpr) -> MathExprKey {
        todo!();
    }

    #[async_recursion]
    pub(crate) async fn expr(&mut self) -> Result<MathExpr, ParseError> {
        // TODO figure out how to handle trailing implicit multiplication

        let token = self.reader.read().await;
        Ok(match token {
            Token::Backslash => {
                let cmd = self.reader.read().await;
                let a = cmd
                    .take_ident()
                    .ok_or(ParseError::UnexpectedToken {
                        expected: Token::Identifier("".to_string()),
                        found: cmd.clone(),
                    })?
                    .as_str();
                match a {
                    "sqrt" => MathExpr::Root(Root::parse(self).await?),

                    _ => return Err(ParseError::UnexpectedEnd),
                }
            }
            _ => todo!(),
        })
    }
}
