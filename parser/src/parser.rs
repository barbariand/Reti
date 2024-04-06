use tokio::sync::mpsc::Receiver;

use crate::{
    ast::{Ast, Factor, FunctionCall, MathExpr, MathIdentifier, Term},
    context::MathContext,
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
    TrailingToken(Token),
}

pub struct Parser {
    reader: TokenReader,
    context: MathContext,
}

impl Parser {
    pub fn new(tokens: Receiver<Token>, context: MathContext) -> Self {
        Parser {
            reader: TokenReader::new(tokens),
            context,
        }
    }

    pub async fn parse(&mut self) -> Result<Ast, ParseError> {
        // Parse expression
        let root_expr = self.expr().await?;

        // Check if we have more to read, if so we have trailing tokens
        // which means we failed to parse the expression fully.
        let trailing = self.reader.read().await;
        if trailing != Token::EndOfContent {
            return Err(ParseError::TrailingToken(trailing));
        }

        Ok(Ast { root_expr })
    }

    pub(crate) async fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        let found = self.reader.read().await;
        if found == expected {
            return Ok(());
        }
        Err(ParseError::UnexpectedToken { expected, found })
    }

    async fn read_identifier(&mut self) -> Result<String, ParseError> {
        let token = self.reader.read().await;
        match token {
            Token::Identifier(val) => Ok(val),
            found => {
                return Err(ParseError::UnexpectedToken {
                    expected: Token::Identifier("".to_string()),
                    found,
                })
            }
        }
    }

    /// Parse a mathematical expression that consists of multiple terms added and
    /// subtracted.
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

    /// Parse a term that consists of multiple factors multiplied and divided. Will
    /// handle implicit multiplication and continues to read until the end of the
    /// term.
    #[async_recursion]
    async fn term(&mut self) -> Result<Term, ParseError> {
        let mut term = Term::Factor(self.factor().await?);

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
                // Implicit multiplication
                Token::Identifier(_)
                | Token::NumberLiteral(_)
                | Token::Backslash
                | Token::LeftParenthesis => {
                    let rhs = self.factor().await?;
                    term = Term::Multiply(Box::new(term), rhs);
                }
                _ => break,
            }
        }

        Ok(term)
    }

    /// Parse a factor, and if the factor has an exponent attached to it, parse the
    /// exponent too.
    #[async_recursion]
    async fn factor(&mut self) -> Result<Factor, ParseError> {
        // Split identifiers into single characters
        match self.reader.peek().await {
            Token::Identifier(text) => {
                if text.len() > 1 {
                    let mut tokens = Vec::new();
                    for c in text.chars() {
                        tokens.push(Token::Identifier(c.to_string()));
                    }
                    self.reader.replace(0..=0, tokens).await;
                }
            }
            _ => {}
        }

        // First read a factor, but then see if we have exponents after it.
        // Exponents need to be baked into the factor since exponents should
        // be evaluated before multiplications.
        //
        let factor = match self.reader.read().await {
            Token::NumberLiteral(val) => Factor::Constant(val.parsed),
            Token::LeftParenthesis => {
                let expr = self.expr().await?;
                self.expect(Token::RightParenthesis).await?;
                Factor::Expression(Box::new(expr))
            }
            Token::Backslash => {
                let command = self.read_identifier().await?;
                self.factor_command(&*command).await?
            }
            Token::VerticalPipe => {
                let expr = self.expr().await?;
                self.expect(Token::VerticalPipe).await?;
                Factor::Abs(Box::new(expr))
            }
            // TODO handle multiple variables in one string, for example
            // "xy". But this should maybe be done by the normalizer
            Token::Identifier(ident) => Factor::Variable(MathIdentifier {
                tokens: vec![Token::Identifier(ident)],
            }),
            Token::Minus => Factor::Constant(-1.0),
            token => todo!("token = {:?}", token),
        };

        let next = self.reader.peek().await;
        if next == Token::Caret {
            // This factor is an exponential
            self.reader.skip().await;
            return self.factor_exponent(factor).await;
        }

        Ok(factor)
    }

    /// Parse a factor that is a LaTeX command.
    ///
    /// The `command` parameter is the LaTeX command.
    async fn factor_command(&mut self, command: &str) -> Result<Factor, ParseError> {
        Ok(match &*command {
            "sqrt" => {
                let next = self.reader.peek().await;
                let mut degree = None;
                if next == Token::LeftBracket {
                    self.reader.skip().await;
                    degree = Some(Box::new(self.expr().await?));
                    self.expect(Token::RightBracket).await?;
                }
                self.expect(Token::LeftCurlyBracket).await?;
                let radicand = Box::new(self.expr().await?);
                self.expect(Token::RightCurlyBracket).await?;
                Factor::Root { degree, radicand }
            }
            "frac" => {
                self.expect(Token::LeftCurlyBracket).await?;
                let numerator = Box::new(self.expr().await?);
                self.expect(Token::RightCurlyBracket).await?;
                self.expect(Token::LeftCurlyBracket).await?;
                let denomminator = Box::new(self.expr().await?);
                self.expect(Token::RightCurlyBracket).await?;
                Factor::Fraction(numerator, denomminator)
            }
            _ => {
                // assume greek alphabet
                let math_identifier = MathIdentifier {
                    tokens: vec![Token::Backslash, Token::Identifier(command.to_string())],
                };
                // This might be a function.
                if self.context.is_function(&math_identifier)
                    && self.reader.peek().await == Token::LeftParenthesis
                {
                    // TODO indexes, for example \log_2(x)

                    self.factor_function_call(math_identifier).await?
                } else {
                    Factor::Variable(math_identifier)
                }
            }
        })
    }

    /// Parse the exponent part of a factor.
    ///
    /// The `factor` parameter is the base, and the tokens to be parsed by this
    /// function is the exponent.
    async fn factor_exponent(&mut self, factor: Factor) -> Result<Factor, ParseError> {
        let next = self.reader.peek().await;
        let exponent = match next {
            Token::LeftCurlyBracket => {
                self.reader.skip().await;
                let expr = self.expr().await?;
                self.expect(Token::RightCurlyBracket).await?;
                expr
            }
            Token::Backslash => {
                let factor = self.factor().await?;
                MathExpr::Term(Term::Factor(factor))
            }
            Token::Identifier(ident) => {
                let ident = ident.clone();
                self.reader.skip().await;
                if ident.len() != 1 {
                    panic!(
                        "The normalizer did not correctly handle exponent, got ident = {}",
                        ident
                    );
                }
                MathExpr::Term(Term::Factor(Factor::Variable(MathIdentifier {
                    tokens: vec![Token::Identifier(ident)],
                })))
            }
            Token::NumberLiteral(num) => {
                if num.raw.len() != 1 {
                    panic!(
                        "The normalizer did not correctly handle exponent, got num = {:?}",
                        num
                    );
                }
                let parsed = num.parsed;
                self.reader.skip().await;
                MathExpr::Term(Term::Factor(Factor::Constant(parsed)))
            }
            token => return Err(ParseError::InvalidToken(token.clone())),
        };

        return Ok(Factor::Exponent {
            base: Box::new(factor),
            exponent: Box::new(exponent),
        });
    }

    async fn factor_function_call(
        &mut self,
        function_name: MathIdentifier,
    ) -> Result<Factor, ParseError> {
        // TODO handle function arguments without parenthesis, for example \ln 2
        let mut arguments = Vec::new();
        self.expect(Token::LeftParenthesis).await?;
        loop {
            let next = self.reader.peek().await;
            if next == Token::RightParenthesis {
                break;
            }
            let expr = self.expr().await?;
            arguments.push(expr);
        }
        self.expect(Token::RightParenthesis).await?;

        Ok(Factor::FunctionCall(FunctionCall {
            function_name,
            arguments,
        }))
    }
}

#[cfg(test)]
mod tests {
    use tokio::{
        join,
        sync::mpsc::{self, Receiver, Sender},
    };

    use crate::{
        ast::{Ast, Factor, FunctionCall, MathExpr, MathIdentifier, Term},
        context::MathContext,
        lexer::Lexer,
        normalizer::Normalizer,
        token::Token,
    };

    use super::Parser;

    async fn parse_test(text: &str, expected_ast: Ast) {
        let (lexer_in, lexer_out): (Sender<Token>, Receiver<Token>) = mpsc::channel(32);
        let (normalizer_in, normalizer_out): (Sender<Token>, Receiver<Token>) = mpsc::channel(32);

        let mut context = MathContext::new();
        context.functions.insert(
            MathIdentifier {
                tokens: vec![Token::Backslash, Token::Identifier("ln".to_string())],
            },
            |val| val[0].eval().ln(),
        );

        let lexer = Lexer::new(lexer_in);
        let mut normalizer = Normalizer::new(lexer_out, normalizer_in);
        let mut parser = Parser::new(normalizer_out, context);

        let future1 = lexer.tokenize(text);
        let future2 = normalizer.normalize();
        let future3 = parser.parse();

        let (_, _, ast) = join!(future1, future2, future3);
        let found_ast = ast.unwrap();

        // Compare and print with debug and formattig otherwise.
        if expected_ast != found_ast {
            panic!("Expected: {:#?}\nFound: {:#?}", expected_ast, found_ast);
        }
    }

    #[tokio::test]
    async fn constant() {
        parse_test(
            "1",
            Ast {
                root_expr: 1f64.into(),
            },
        )
        .await;
    }

    #[tokio::test]
    async fn addition() {
        parse_test(
            "1+2+3",
            Ast {
                root_expr: MathExpr::Add(
                    Box::new(MathExpr::Add(Box::new(1f64.into()), 2f64.into())),
                    3f64.into(),
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
                        Box::new(MathExpr::Add(Box::new(1f64.into()), 2f64.into())),
                        3f64.into(),
                    )),
                    Term::Multiply(
                        Box::new(Term::Factor(Factor::Expression(Box::new(MathExpr::Add(
                            Box::new(4f64.into()),
                            5f64.into(),
                        ))))),
                        6f64.into(),
                    ),
                ),
            },
        )
        .await;
    }
    #[tokio::test]
    async fn sqrt() {
        parse_test(
            "\\sqrt{9}",
            Ast {
                root_expr: MathExpr::Term(Term::Factor(Factor::Root {
                    degree: None,
                    radicand: 9f64.into(),
                })),
            },
        )
        .await;
    }

    #[tokio::test]
    async fn cube_root() {
        parse_test(
            "\\sqrt[3]{27}",
            Ast {
                root_expr: MathExpr::Term(Term::Factor(Factor::Root {
                    degree: Some(3f64.into()),
                    radicand: 27f64.into(),
                })),
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
    async fn parenthesis_and_exponent() {
        parse_test(
            "2(3)^3",
            Ast {
                root_expr: MathExpr::Term(Term::Multiply(
                    // 2
                    Box::new(2f64.into()),
                    // (3)^3
                    Factor::Exponent {
                        base: Box::new(Factor::Expression(Box::new(3f64.into()))),
                        exponent: Box::new(3f64.into()),
                    },
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
                        // 5x
                        Box::new(Term::Multiply(
                            // 5
                            Box::new(5f64.into()),
                            // x
                            Factor::Variable(MathIdentifier {
                                tokens: vec![Token::Identifier("x".to_string())],
                            }),
                        )),
                        // y
                        Factor::Variable(MathIdentifier {
                            tokens: vec![Token::Identifier("y".to_string())],
                        }),
                    ),
                ),
            },
        )
        .await;
    }

    #[tokio::test]
    async fn implicit_multiplication_single_identifier_token() {
        parse_test(
            "2xy^2",
            Ast {
                root_expr: MathExpr::Term(Term::Multiply(
                    // 2x
                    Box::new(Term::Multiply(
                        // 2
                        Box::new(2f64.into()),
                        // x
                        Factor::Variable(MathIdentifier {
                            tokens: vec![Token::Identifier("x".to_string())],
                        }),
                    )),
                    // y^2
                    Factor::Exponent {
                        base: Box::new(Factor::Variable(MathIdentifier {
                            tokens: vec![Token::Identifier("y".to_string())],
                        })),
                        exponent: 2f64.into(),
                    },
                )),
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
    async fn implicit_multiplication_vs_function_call() {
        parse_test(
            "\\pi(x)\\ln(x)", // this is pi * x * ln(x)
            Ast {
                root_expr: MathExpr::Term(Term::Multiply(
                    // \pi(x)
                    Box::new(Term::Multiply(
                        Box::new(
                            Factor::Variable(MathIdentifier {
                                tokens: vec![Token::Backslash, Token::Identifier("pi".to_string())],
                            })
                            .into(),
                        ),
                        Factor::Expression(Box::new(
                            Factor::Variable(MathIdentifier {
                                tokens: vec![Token::Identifier("x".to_string())],
                            })
                            .into(),
                        )),
                    )),
                    Factor::FunctionCall(FunctionCall {
                        function_name: MathIdentifier {
                            tokens: vec![Token::Backslash, Token::Identifier("ln".to_string())],
                        },
                        arguments: vec![Factor::Variable(MathIdentifier {
                            tokens: vec![Token::Identifier("x".to_string())],
                        })
                        .into()],
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
            // This is a bit mathematically ambiguous, but it means
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
                            Factor::Constant(2.0),
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

    #[tokio::test]
    async fn fraction() {
        parse_test(
            "\\frac{1}{2}",
            Ast {
                root_expr: Factor::Fraction(1f64.into(), 2f64.into()).into(),
            },
        )
        .await;
    }

    #[tokio::test]
    async fn abs() {
        parse_test(
            "|-3|",
            Ast {
                root_expr: Factor::Abs(Box::new(MathExpr::Term(Term::Multiply(
                    Box::new((-1f64).into()),
                    3f64.into(),
                ))))
                .into(),
            },
        )
        .await;
    }
}
