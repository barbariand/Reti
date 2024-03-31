use tokio::sync::mpsc::Receiver;

use crate::{
    ast::{Ast, Factor, MathExpr, MathIdentifier, Term},
    token::Token,
    token_reader::TokenReader,
};
use async_recursion::async_recursion;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken { expected: Token, found: Token },
    ExpectedButNothingFound(Token),
    UnexpectedEnd,
    ExpectedEndOfFile,
    InvalidToken(Token),
}

pub struct Parser {
    pub(crate) reader: TokenReader,
}

impl Parser {
    pub fn new(tokens: Receiver<Token>) -> Self {
        Parser {
            reader: TokenReader::new(tokens),
        }
    }

    pub async fn parse(&mut self) -> Result<Ast, ParseError> {
        let root_expr = self.expr().await?;
        // TODO detect trailing tokens, like what if we read an expression but then we found more tokens?
        Ok(Ast { root_expr })
    }

    pub(crate) async fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        let found = self.reader.read().await;
        if found == expected {
            return Ok(());
        }
        Err(ParseError::UnexpectedToken { expected, found })
    }

    #[async_recursion]
    async fn expr(&mut self) -> Result<MathExpr, ParseError> {
        let mut expr = MathExpr::Term(self.term().await?);

        loop {
            let next = self.reader.peek().await;
            match next {
                Token::Plus => {
                    self.reader.skip().await;
                    let rhs = self.term().await?;
                    expr = MathExpr::Add(Box::new(expr), rhs);
                }
                Token::Minus => {
                    self.reader.skip().await;
                    let rhs = self.term().await?;
                    expr = MathExpr::Subtract(Box::new(expr), rhs);
                }
                _ => break,
            };
        }

        Ok(expr)
    }

    #[async_recursion]
    async fn term(&mut self) -> Result<Term, ParseError> {
        let mut term = Term::Factor(self.factor().await?);

        // TODO handle these cases:
        //  - a \cdot b
        //  - a \times b
        //  - a(b)
        //  - 2x
        loop {
            let next = self.reader.peek().await;
            match next {
                Token::Asterisk => {
                    self.reader.skip().await;
                    let rhs = self.factor().await?;
                    term = Term::Multiply(Box::new(term), rhs);
                }
                Token::Slash => {
                    self.reader.skip().await;
                    let rhs = self.factor().await?;
                    term = Term::Divide(Box::new(term), rhs);
                }
                Token::Backslash => {
                    //TODO add preprossesor step to remove the \cdot osv
                    self.factor().await?;
                }
                _ => break,
            }
        }

        Ok(term)
    }

    #[async_recursion]
    async fn factor(&mut self) -> Result<Factor, ParseError> {
        // First read a factor, but then see if we have exponents after it.
        // Exponents need to be baked into the factor since exponents should
        // be evaluated before multiplications.
        //
        let factor = match self.reader.read().await {
            Token::NumberLiteral(val) => Factor::Constant(val.parsed),
            Token::LeftParenthesis => {
                // TODO handle "\left("
                let expr = self.expr().await?;
                // TODO handle "\right)"
                self.expect(Token::RightParenthesis).await?;
                Factor::Expression(Box::new(expr))
            }
            Token::Backslash => {
                let token = self.reader.read().await;
                let command = match token {
                    Token::Identifier(val) => val,
                    found => {
                        return Err(ParseError::UnexpectedToken {
                            expected: Token::Identifier("".to_string()),
                            found,
                        })
                    }
                };
                // assume greek alphabet
                // TODO function calls
                Factor::Variable(MathIdentifier {
                    tokens: vec![Token::Backslash, Token::Identifier(command)],
                })
            }
            token => todo!("token = {:?}", token),
        };

        let next = self.reader.peek().await;
        if next == Token::Caret {
            // This factor is an exponential
            self.reader.skip().await;
            let exponent = if self.reader.peek().await == Token::LeftCurlyBracket {
                self.reader.skip().await;
                let expr = self.expr().await?;
                self.expect(Token::RightCurlyBracket).await?;
                expr
            } else {
                // TODO this will be a problem since we will need to split the next token.......
                todo!("Please use explicit exponetials for now, so instead of a^b please do a^{{b}}. Thanks!");
            };

            return Ok(Factor::Exponent {
                base: Box::new(factor),
                exponent: Box::new(exponent),
            });
        }

        Ok(factor)
    }
}

#[cfg(test)]
mod tests {
    use tokio::{
        join,
        sync::mpsc::{self, Receiver, Sender},
    };

    use crate::{
        ast::{Ast, Factor, MathExpr, MathIdentifier, Term},
        lexer::Lexer,
        token::Token,
    };

    use super::Parser;

    async fn parse_test(text: &str, expected_ast: Ast) {
        let (tx, rx): (Sender<Token>, Receiver<Token>) = mpsc::channel(32); // idk what that 32 means tbh

        let lexer = Lexer::new(tx);
        let mut parser = Parser::new(rx);

        let future1 = lexer.tokenize(text);
        let future2 = parser.parse();

        let (_, ast) = join!(future1, future2);
        let found_ast = ast.unwrap();

        // Compare and print with debug and formattig otherwise.
        if expected_ast != found_ast {
            panic!("Expected: {:#?}\nFound: {:#?}", expected_ast, found_ast);
        }
    }

    #[tokio::test]
    async fn addition() {
        parse_test(
            "1+2+3",
            Ast {
                root_expr: MathExpr::Add(
                    Box::new(MathExpr::Add(
                        Box::new(MathExpr::Term(Term::Factor(Factor::Constant(1.0)))),
                        Term::Factor(Factor::Constant(2.0)),
                    )),
                    Term::Factor(Factor::Constant(3.0)),
                ),
            },
        )
        .await;
    }

    #[tokio::test]
    async fn addition_multiplication_order_of_operations() {
        parse_test(
            "1+2+3+(4+5)*6",
            Ast {
                root_expr: MathExpr::Add(
                    Box::new(MathExpr::Add(
                        Box::new(MathExpr::Add(
                            Box::new(MathExpr::Term(Term::Factor(Factor::Constant(1.0)))),
                            Term::Factor(Factor::Constant(2.0)),
                        )),
                        Term::Factor(Factor::Constant(3.0)),
                    )),
                    Term::Multiply(
                        Box::new(Term::Factor(Factor::Expression(Box::new(MathExpr::Add(
                            Box::new(MathExpr::Term(Term::Factor(Factor::Constant(4.0)))),
                            Term::Factor(Factor::Constant(5.0)),
                        ))))),
                        Factor::Constant(6.0),
                    ),
                ),
            },
        )
        .await;
    }
    #[tokio::test]
    async fn exponent() {
        parse_test(
            "2^{3}",
            Ast {
                root_expr: MathExpr::Term(Term::Factor(Factor::Exponent {
                    base: Box::new(Factor::Constant(2.0)),
                    exponent: Box::new(MathExpr::Term(Term::Factor(Factor::Constant(3.0)))),
                })),
            },
        )
        .await;
    }

    #[tokio::test]
    async fn exponent_command() {
        parse_test(
            "2^\\pi",
            Ast {
                root_expr: MathExpr::Term(Term::Factor(Factor::Exponent {
                    base: Box::new(Factor::Constant(2.0)),
                    exponent: Box::new(MathExpr::Term(Term::Factor(Factor::Variable(
                        MathIdentifier {
                            tokens: vec![Token::Backslash, Token::Identifier("pi".to_string())],
                        },
                    )))),
                })),
            },
        )
        .await;
    }

    #[tokio::test]
    async fn exponent_paren() {
        parse_test(
            "2^(1)",
            Ast {
                root_expr: MathExpr::Term(Term::Factor(Factor::Exponent {
                    base: Box::new(Factor::Constant(2.0)),
                    exponent: Box::new(MathExpr::Term(Term::Factor(Factor::Constant(1.0)))),
                })),
            },
        )
        .await;
    }

    #[tokio::test]
    async fn exponent_split_token() {
        parse_test(
            "2^025", // this is 2^0 * 25
            Ast {
                root_expr: MathExpr::Term(Term::Multiply(
                    //2^0
                    Box::new(Term::Factor(Factor::Exponent {
                        base: Box::new(Factor::Constant(2.0)),
                        exponent: Box::new(MathExpr::Term(Term::Factor(Factor::Constant(0.0)))),
                    })),
                    // 25
                    Factor::Constant(25.0),
                )),
            },
        )
        .await;
    }

    #[tokio::test]
    async fn implicit_multiplication_and_exponent_order_of_operations() {
        parse_test(
            "2x^{2} + 5xy",
            Ast {
                root_expr: MathExpr::Add(
                    // 2x^{2}
                    Box::new(MathExpr::Term(Term::Multiply(
                        // 2
                        Box::new(Term::Factor(Factor::Constant(2.0))),
                        // x^{2}
                        Factor::Exponent {
                            base: Box::new(Factor::Variable(MathIdentifier {
                                tokens: vec![Token::Identifier("x".to_string())],
                            })),
                            exponent: Box::new(MathExpr::Term(Term::Factor(Factor::Constant(2.0)))),
                        },
                    ))),
                    // 5xy
                    Term::Multiply(
                        // 5
                        Box::new(Term::Factor(Factor::Constant(5.0))),
                        // xy
                        // TODO split into x and y
                        Factor::Variable(MathIdentifier {
                            tokens: vec![Token::Identifier("xy".to_string())],
                        }),
                    ),
                ),
            },
        )
        .await;
    }

    #[tokio::test]
    async fn pi() {
        parse_test(
            "\\pi",
            Ast {
                root_expr: MathExpr::Term(Term::Factor(Factor::Variable(MathIdentifier {
                    tokens: vec![Token::Backslash, Token::Identifier("pi".to_string())],
                }))),
            },
        )
        .await;
    }

    #[tokio::test]
    async fn implicit_multiplication_command() {
        parse_test(
            "2\\pi",
            Ast {
                root_expr: MathExpr::Term(Term::Multiply(
                    // 2
                    Box::new(Term::Factor(Factor::Constant(2.0))),
                    // \pi
                    Factor::Variable(MathIdentifier {
                        tokens: vec![Token::Backslash, Token::Identifier("pi".to_string())],
                    }),
                )),
            },
        )
        .await;
    }

    #[tokio::test]
    async fn division_order_of_operations() {
        parse_test(
            "5/2x + 3",
            // This is a bit mathematically abvigous, but it means
            // 5/2 * x + 3 because multiplication and division are
            // on the same level, so it is evaluated left to right.
            Ast {
                root_expr: MathExpr::Add(
                    // 5/2x
                    Box::new(MathExpr::Term(Term::Multiply(
                        // 5/2
                        Box::new(Term::Divide(
                            // 5
                            Box::new(Term::Factor(Factor::Constant(5.0))),
                            // 2
                            Factor::Constant(5.0),
                        )),
                        // x
                        Factor::Variable(MathIdentifier {
                            tokens: vec![Token::Identifier("x".to_string())],
                        }),
                    ))),
                    // 3
                    Term::Factor(Factor::Constant(3.0)),
                ),
            },
        )
        .await;
    }
}
