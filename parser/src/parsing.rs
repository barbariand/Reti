//!Parsing the TokenStream to an AST
use tracing::{trace, trace_span};

use crate::{
    identifier::{MathLetter, MathString, ModifierType},
    prelude::*,
};
use async_recursion::async_recursion;
///Parser for parsing the stream when it is done by the normalizer
pub struct Parser {
    ///token stream from the normalizer
    reader: TokenReader,
    ///the context in witch it operates in
    context: MathContext,
}

impl Parser {
    ///creating a new
    pub fn new(tokens: TokenReceiver, context: MathContext) -> Self {
        trace!("created Parser");

        Parser {
            reader: TokenReader::new(tokens),
            context,
        }
    }
    ///Starting the parser
    pub async fn parse(mut self) -> Result<Ast, ParseError> {
        let span = trace_span!("parse");
        let _enter = span.enter();

        // Parse expression
        let root_expr = self.expr().await?;
        trace!("root_expr = {root_expr:?}");

        // Check if we have more to read, if not, that means we have a full
        // expression we can return.
        let next = self.reader.read().await;
        if next == Token::EndOfContent {
            return Ok(Ast::Expression(root_expr));
        }
        if next == Token::Equals {
            // An equality. Try parse a right hand side.
            let rhs = self.expr().await?;
            let next = self.reader.read().await;
            trace!("trailing = {next}");
            if next != Token::EndOfContent {
                return Err(ParseError::Trailing { token: next });
            }
            return Ok(Ast::Equality(root_expr, rhs));
        }
        // It seems we have expected trailing tokens.
        // This means we failed to parse the expression fully.
        Err(ParseError::Trailing { token: next })
    }
    ///expect the next token to be of a type
    pub(crate) async fn expect(
        &mut self,
        expected: Token,
    ) -> Result<(), ParseError> {
        let found = self.reader.read().await;
        if found == expected {
            return Ok(());
        }
        Err(ParseError::UnexpectedToken {
            expected: vec![expected],
            found,
        })
    }
    ///reads the token until it is not a char and then adds them up
    async fn read_identifier(&mut self) -> Result<String, ParseError> {
        let token = self.reader.read().await;
        match token {
            Token::Identifier(val) => Ok(val),
            found => Err(ParseError::UnexpectedToken {
                expected: vec![Token::Identifier("".to_string())],
                found,
            }),
        }
    }

    /// Parse a mathematical expression that consists of multiple terms added
    /// and subtracted.
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

    /// Parse a term that consists of multiple factors multiplied and divided.
    /// Will handle implicit multiplication and continues to read until the
    /// end of the term.
    #[async_recursion]
    async fn term(&mut self) -> Result<Term, ParseError> {
        let mut term = Term::Factor(self.factor().await?);

        loop {
            let next = self.reader.peek_range(0..=1).await;
            match next[..] {
                [Token::Asterisk, _] => {
                    self.reader.skip().await;
                    let rhs = self.factor().await?;
                    term =
                        Term::Multiply(MulType::Asterisk, Box::new(term), rhs);
                }
                [Token::Backslash, Token::Identifier(ident)]
                    if ident == "cdot"
                        || ident == "cdotp"
                        || ident == "times" =>
                {
                    let mul_type = match ident.as_str() {
                        "cdot" | "cdotp" => MulType::Cdot,
                        "times" => MulType::Times,
                        _ => unreachable!("invalid multype"),
                    };
                    self.reader.skip().await;
                    self.reader.skip().await;
                    let rhs = self.factor().await?;
                    term = Term::Multiply(mul_type, Box::new(term), rhs);
                }
                [Token::Slash, _] => {
                    self.reader.skip().await;
                    let rhs = self.factor().await?;
                    term = Term::Divide(Box::new(term), rhs);
                }
                [Token::Backslash, Token::Backslash] => {
                    break;
                }
                [Token::Backslash, Token::Identifier(ident)]
                    if ident == "end" =>
                {
                    break;
                }
                // Implicit multiplication
                [Token::Identifier(_)
                | Token::NumberLiteral(_)
                | Token::Backslash
                | Token::LeftParenthesis, _] => {
                    let rhs = self.factor().await?;
                    term =
                        Term::Multiply(MulType::Implicit, Box::new(term), rhs);
                }
                _ => break,
            }
        }

        Ok(term)
    }

    /// If the next character is an identifier, ensure that it only has length
    /// one by splitting it.
    async fn split_next_identifier(&mut self) {
        if let Token::Identifier(text) = self.reader.peek().await {
            if text.len() > 1 {
                let mut tokens = Vec::new();
                for c in text.chars() {
                    tokens.push(Token::Identifier(c.to_string()));
                }
                self.reader.replace(0..=0, tokens).await;
            }
        }
    }

    /// Parse a factor, and if the factor has an exponent attached to it, parse
    /// the exponent too.
    #[async_recursion]
    async fn factor(&mut self) -> Result<Factor, ParseError> {
        // Split identifiers into single characters
        self.split_next_identifier().await;
        // First read a factor, but then see if we have exponents after it.
        // Exponents need to be baked into the factor since exponents should
        // be evaluated before multiplications.
        //
        let factor = match self.reader.read().await {
            Token::NumberLiteral(val) => Factor::Constant(val.parsed),
            Token::LeftParenthesis => {
                // In most cases, this is one value, for example (1+1).
                // But parse many values since it could be a vector (1,2,3)
                // if commas are encountered.
                let mut values = Vec::with_capacity(1);
                loop {
                    let expr = self.expr().await?;
                    values.push(expr);
                    let next = self.reader.peek().await;
                    if next != Token::Comma {
                        break;
                    }
                    self.reader.skip().await;
                }
                self.expect(Token::RightParenthesis).await?;
                let len = values.len();
                if len == 1 {
                    Factor::Parenthesis(Box::new(values.remove(0)))
                } else {
                    Factor::Matrix(Matrix::new(values, 1, len))
                }
            }
            Token::Backslash => {
                let command = self.read_identifier().await?;
                self.factor_command(&command).await?
            }
            Token::VerticalPipe => {
                let expr = self.expr().await?;
                self.expect(Token::VerticalPipe).await?;
                Factor::Abs(Box::new(expr))
            }
            Token::Identifier(ident) => {
                if ident.chars().count() != 1 {
                    panic!("Identifier was not splitted correctly.")
                }
                let math_identifier = MathIdentifier::from_single_ident(&ident);
                let math_identifier =
                    self.math_identifier_tail(math_identifier).await?;
                self.factor_identifier(math_identifier).await?
            }
            Token::Minus => Factor::Constant(-1.0),
            token => return Err(ParseError::InvalidFactor { token }),
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
    async fn factor_command(
        &mut self,
        command: &str,
    ) -> Result<Factor, ParseError> {
        Ok(match command {
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
                let denominator = Box::new(self.expr().await?);
                self.expect(Token::RightCurlyBracket).await?;
                Factor::Fraction(numerator, denominator)
            }
            "begin" => {
                self.expect(Token::LeftCurlyBracket).await?;
                let s = self.read_identifier().await?;
                match s.as_str() {
                    "bmatrix" | "pmatrix" | "Bmatrix" => {
                        self.expect(Token::RightCurlyBracket).await?;
                        Factor::Matrix(self.matrix(s).await?)
                    }
                    "vmatrix" | "Vmatrix" => {
                        self.expect(Token::RightCurlyBracket).await?;
                        let matrix = self.matrix(s).await?;
                        Factor::Abs(Box::new(MathExpr::Term(Term::Factor(
                            Factor::Matrix(matrix),
                        ))))
                    }
                    _ => {
                        return Err(ParseError::InvalidBegin {
                            beginning: s.to_owned(),
                        });
                    }
                }
            }
            _ => {
                let ident = self.parse_math_identifier_command(command).await?;
                self.factor_identifier(ident).await?
            }
        })
    }

    /// Parse a [MathIdentifier] that starts with a command.
    async fn parse_math_identifier_command(
        &mut self,
        command: &str,
    ) -> Result<MathIdentifier, ParseError> {
        let letter = MathLetter::from_latex(command);
        let modifier = ModifierType::from_latex(command);
        let math_identifier = if let Some(letter) = letter {
            let math_str = MathString::from_letters(vec![letter]);
            MathIdentifier::Name(math_str)
        } else if let Some(modifier) = modifier {
            let inner = self.parse_inner_math_identifier().await?;
            MathIdentifier::Modifier(modifier, Box::new(inner))
        } else {
            return Err(ParseError::InvalidIdentifierCommmand {
                command: command.to_string(),
            });
        };
        // Parse index if there is one and then return.
        self.math_identifier_tail(math_identifier).await
    }

    /// Parse an inner identifier which may be one character long, or if
    /// surrounded by curly brackets may hold an entire inner identifier.
    async fn parse_inner_math_identifier(
        &mut self,
    ) -> Result<MathExpr, ParseError> {
        if self.reader.peek().await == Token::LeftCurlyBracket {
            self.reader.skip().await;
            let inner = self.expr().await?;
            self.expect(Token::RightCurlyBracket).await?;
            Ok(inner)
        } else {
            todo!("handle single character inner. For example \\overline x or x_1.");
        }
    }

    /// Parse the index of an identifier, if there is an index. Otherwise return
    /// the identifier as is.
    async fn math_identifier_tail(
        &mut self,
        ident: MathIdentifier,
    ) -> Result<MathIdentifier, ParseError> {
        // Check for index
        Ok(if self.reader.peek().await == Token::Underscore {
            self.reader.skip().await;
            let index = self.parse_inner_math_identifier().await?;
            MathIdentifier::Index {
                name: Box::new(ident),
                index: Box::new(index),
            }
        } else {
            // No index, just return as-is.
            ident
        })
    }

    /// Parse a factor when an identifier was just read. This may either be a
    /// function or a variable.
    async fn factor_identifier(
        &mut self,
        identifier: MathIdentifier,
    ) -> Result<Factor, ParseError> {
        // This might be a function.
        if self.context.is_defined_function(&identifier) {
            Ok(self.factor_function_call(identifier).await?)
        } else {
            Ok(Factor::Variable(identifier))
        }
    }

    /// Parse the exponent part of a factor.
    ///
    /// The `factor` parameter is the base, and the tokens to be parsed by this
    /// function is the exponent.
    async fn factor_exponent(
        &mut self,
        factor: Factor,
    ) -> Result<Factor, ParseError> {
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
                MathExpr::Term(Term::Factor(Factor::Variable(
                    MathIdentifier::Name(MathString::from_letters(vec![
                        MathLetter::Ascii(ident.bytes().next().unwrap()),
                    ])),
                )))
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
            token => {
                return Err(ParseError::Invalid {
                    token: token.clone(),
                })
            }
        };

        Ok(Factor::Power {
            base: Box::new(factor),
            exponent: Box::new(exponent),
        })
    }
    ///parsing a suspected function
    async fn factor_function_call(
        &mut self,
        function_name: MathIdentifier,
    ) -> Result<Factor, ParseError> {
        let mut arguments = Vec::new();
        // Read arguments in parenthesis, eg. f(1, 2)
        if self.reader.peek().await == Token::LeftParenthesis {
            self.reader.skip().await;
            loop {
                let next = self.reader.peek().await;
                match next {
                    Token::RightParenthesis => break,
                    Token::Comma => self.reader.skip().await,
                    _ => {
                        let expr = self.expr().await?;
                        arguments.push(expr);
                    }
                }
            }
            self.expect(Token::RightParenthesis).await?;
        } else {
            // Read one explicit argument, for example \ln 2
            let arg = self.term().await?;
            arguments.push(MathExpr::Term(arg));
        }

        Ok(Factor::FunctionCall(FunctionCall {
            function_name,
            arguments,
        }))
    }
    ///Parsing a suspected matrix
    async fn matrix(
        &mut self,
        matrix_type: String,
    ) -> Result<Matrix<MathExpr>, ParseError> {
        let mut rows = Vec::new();
        let mut current_row = Vec::new();
        let mut column_count = Option::None;

        loop {
            let cell = self.expr().await?;
            current_row.push(cell);

            let next = self.reader.peek().await;
            match next {
                Token::Ampersand => {
                    self.reader.skip().await;
                    continue;
                }
                Token::Backslash => {
                    if self.reader.peekn(1).await == Token::Backslash {
                        // Two backslashes means end of row.
                        self.reader.skip().await;
                        self.reader.skip().await;
                        match column_count {
                            Some(column_count) => {
                                if column_count != current_row.len() {
                                    return Err(ParseError::MismatchedMatrixColumnSize {
                                        prev: column_count,
                                        current: current_row.len(),
                                    });
                                }
                            }
                            None => column_count = Some(current_row.len()),
                        }
                        rows.push(current_row);
                        current_row = Vec::new();
                    } else {
                        // \end{matrix_type}
                        self.reader.skip().await;
                        self.expect(Token::Identifier("end".to_owned()))
                            .await?;
                        self.expect(Token::LeftCurlyBracket).await?;
                        self.expect(Token::Identifier(matrix_type)).await?;
                        self.expect(Token::RightCurlyBracket).await?;
                        // TODO de-duplicate code
                        match column_count {
                            Some(column_count) => {
                                if column_count != current_row.len() {
                                    return Err(ParseError::MismatchedMatrixColumnSize {
                                        prev: column_count,
                                        current: current_row.len(),
                                    });
                                }
                            }
                            None => column_count = Some(current_row.len()),
                        }
                        rows.push(current_row);
                        break;
                    }
                }
                e => {
                    return Err(ParseError::UnexpectedToken {
                        expected: vec![Token::Ampersand, Token::Backslash],
                        found: e.clone(),
                    });
                }
            }
        }

        let row_count = rows.len();
        let column_count = column_count.unwrap_or(0);
        let mut values = Vec::with_capacity(row_count * column_count);
        // Ensure this is always the same ordering as Matrix::index expects
        for row in rows.into_iter() {
            for value in row.into_iter() {
                values.push(value);
            }
        }

        if row_count == 0 || column_count == 0 {
            return Err(ParseError::EmptyMatrix);
        }

        Ok(Matrix::new(values, row_count, column_count))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        identifier::{GreekLetter, ModifierType, OtherSymbol},
        prelude::*,
    };
    use pretty_assertions::assert_eq;
    async fn parse_test(text: &str, expected_ast: Ast) {
        let found_ast = parse(text, &MathContext::standard_math()).await;
        match found_ast {
            Ok(found_ast) => {
                // Compare and print with debug and formatting otherwise.
                assert_eq!(found_ast, expected_ast);
            }
            Err(err) => {
                panic!("Failed to parse AST:\n{}\n\nDebug: {:?}", err, err);
            }
        }
    }

    #[tokio::test]
    async fn constant() {
        parse_test("1", Ast::Expression(1f64.into())).await;
    }

    #[tokio::test]
    async fn addition() {
        parse_test(
            "1+2+3",
            Ast::Expression(MathExpr::Add(
                Box::new(MathExpr::Add(Box::new(1f64.into()), 2f64.into())),
                3f64.into(),
            )),
        )
        .await;
    }

    #[tokio::test]
    async fn addition_multiplication_order_of_operations() {
        parse_test(
            "1+2+3+(4+5)*6",
            Ast::Expression(MathExpr::Add(
                Box::new(MathExpr::Add(
                    Box::new(MathExpr::Add(Box::new(1f64.into()), 2f64.into())),
                    3f64.into(),
                )),
                Term::Multiply(
                    MulType::Asterisk,
                    Box::new(Term::Factor(Factor::Parenthesis(Box::new(
                        MathExpr::Add(Box::new(4f64.into()), 5f64.into()),
                    )))),
                    6f64.into(),
                ),
            )),
        )
        .await;
    }

    #[tokio::test]
    async fn multiplication_asterisk() {
        parse_test(
            "2*3",
            Ast::Expression(
                Term::Multiply(
                    MulType::Asterisk,
                    Box::new(2f64.into()),
                    3f64.into(),
                )
                .into(),
            ),
        )
        .await;
    }

    #[tokio::test]
    async fn multiplication_cdot() {
        parse_test(
            "2\\cdot3",
            Ast::Expression(
                Term::Multiply(
                    MulType::Cdot,
                    Box::new(2f64.into()),
                    3f64.into(),
                )
                .into(),
            ),
        )
        .await;
    }

    #[tokio::test]
    async fn multiplication_times() {
        parse_test(
            "2\\times3",
            Ast::Expression(
                Term::Multiply(
                    MulType::Times,
                    Box::new(2f64.into()),
                    3f64.into(),
                )
                .into(),
            ),
        )
        .await;
    }

    #[tokio::test]
    async fn sqrt() {
        parse_test(
            "\\sqrt{9}",
            Ast::Expression(
                Factor::Root {
                    degree: None,
                    radicand: 9f64.into(),
                }
                .into(),
            ),
        )
        .await;
    }

    #[tokio::test]
    async fn cube_root() {
        parse_test(
            "\\sqrt[3]{27}",
            Ast::Expression(
                Factor::Root {
                    degree: Some(3f64.into()),
                    radicand: 27f64.into(),
                }
                .into(),
            ),
        )
        .await;
    }

    #[tokio::test]
    async fn exponent() {
        parse_test(
            "2^{3}",
            Ast::Expression(
                Factor::Power {
                    base: Box::new(Factor::Constant(2.0)),
                    exponent: Box::new(MathExpr::Term(Term::Factor(
                        Factor::Constant(3.0),
                    ))),
                }
                .into(),
            ),
        )
        .await;
    }

    #[tokio::test]
    async fn exponent_command() {
        parse_test(
            "2^\\pi",
            Ast::Expression(
                Factor::Power {
                    base: Box::new(2f64.into()),
                    exponent: Box::new(
                        Factor::Variable(MathIdentifier::from_single_greek(
                            GreekLetter::LowercasePi,
                        ))
                        .into(),
                    ),
                }
                .into(),
            ),
        )
        .await;
    }

    #[tokio::test]
    async fn exponent_split_token() {
        parse_test(
            "2^025", // this is 2^0 * 25
            Ast::Expression(MathExpr::Term(Term::Multiply(
                MulType::Implicit,
                //2^0
                Box::new(Term::Factor(Factor::Power {
                    base: Box::new(Factor::Constant(2.0)),
                    exponent: Box::new(MathExpr::Term(Term::Factor(
                        Factor::Constant(0.0),
                    ))),
                })),
                // 25
                Factor::Constant(25.0),
            ))),
        )
        .await;
    }

    #[tokio::test]
    async fn parenthesis_and_exponent() {
        parse_test(
            "2(3)^3",
            Ast::Expression(MathExpr::Term(Term::Multiply(
                MulType::Implicit,
                // 2
                Box::new(2f64.into()),
                // (3)^3
                Factor::Power {
                    base: Box::new(Factor::Parenthesis(Box::new(3f64.into()))),
                    exponent: Box::new(3f64.into()),
                },
            ))),
        )
        .await;
    }

    #[tokio::test]
    async fn implicit_multiplication_and_exponent_order_of_operations() {
        parse_test(
            "2x^{2} + 5xy",
            Ast::Expression(MathExpr::Add(
                // 2x^{2}
                Box::new(MathExpr::Term(Term::Multiply(
                    MulType::Implicit,
                    // 2
                    Box::new(Term::Factor(Factor::Constant(2.0))),
                    // x^{2}
                    Factor::Power {
                        base: Box::new(Factor::Variable(
                            MathIdentifier::from_single_ident("x"),
                        )),
                        exponent: Box::new(MathExpr::Term(Term::Factor(
                            Factor::Constant(2.0),
                        ))),
                    },
                ))),
                // 5xy
                Term::Multiply(
                    MulType::Implicit,
                    // 5x
                    Box::new(Term::Multiply(
                        MulType::Implicit,
                        // 5
                        Box::new(5f64.into()),
                        // x
                        Factor::Variable(MathIdentifier::from_single_ident(
                            "x",
                        )),
                    )),
                    // y
                    Factor::Variable(MathIdentifier::from_single_ident("y")),
                ),
            )),
        )
        .await;
    }

    #[tokio::test]
    async fn implicit_multiplication_single_identifier_token() {
        parse_test(
            "2xy^2",
            Ast::Expression(MathExpr::Term(Term::Multiply(
                MulType::Implicit,
                // 2x
                Box::new(Term::Multiply(
                    MulType::Implicit,
                    // 2
                    Box::new(2f64.into()),
                    // x
                    Factor::Variable(MathIdentifier::from_single_ident("x")),
                )),
                // y^2
                Factor::Power {
                    base: Box::new(Factor::Variable(
                        MathIdentifier::from_single_ident("y"),
                    )),
                    exponent: 2f64.into(),
                },
            ))),
        )
        .await;
    }

    #[tokio::test]
    async fn pi() {
        parse_test(
            "\\pi",
            Ast::Expression(MathExpr::Term(Term::Factor(Factor::Variable(
                MathIdentifier::from_single_greek(GreekLetter::LowercasePi),
            )))),
        )
        .await;
    }

    #[tokio::test]
    async fn implicit_multiplication_vs_function_call() {
        parse_test(
            "\\pi(x)\\ln(x)", // this is pi * x * ln(x)
            Ast::Expression(MathExpr::Term(Term::Multiply(
                MulType::Implicit,
                // \pi(x)
                Box::new(Term::Multiply(
                    MulType::Implicit,
                    Box::new(
                        Factor::Variable(MathIdentifier::from_single_greek(
                            GreekLetter::LowercasePi,
                        ))
                        .into(),
                    ),
                    Factor::Parenthesis(Box::new(
                        Factor::Variable(MathIdentifier::from_single_ident(
                            "x",
                        ))
                        .into(),
                    )),
                )),
                Factor::FunctionCall(FunctionCall {
                    function_name: MathIdentifier::from_single_symbol(
                        OtherSymbol::Ln,
                    ),
                    arguments: vec![Factor::Variable(
                        MathIdentifier::from_single_ident("x"),
                    )
                    .into()],
                }),
            ))),
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
            Ast::Expression(MathExpr::Add(
                // 5/2x
                Box::new(MathExpr::Term(Term::Multiply(
                    MulType::Implicit,
                    // 5/2
                    Box::new(Term::Divide(
                        // 5
                        Box::new(Term::Factor(Factor::Constant(5.0))),
                        // 2
                        Factor::Constant(2.0),
                    )),
                    // x
                    Factor::Variable(MathIdentifier::from_single_ident("x")),
                ))),
                // 3
                Term::Factor(Factor::Constant(3.0)),
            )),
        )
        .await;
    }

    #[tokio::test]
    async fn fraction() {
        parse_test(
            "\\frac{1}{2}",
            Ast::Expression(Factor::Fraction(1f64.into(), 2f64.into()).into()),
        )
        .await;
    }

    #[tokio::test]
    async fn abs() {
        parse_test(
            "|-3|",
            Ast::Expression(
                Factor::Abs(Box::new(MathExpr::Term(Term::Multiply(
                    MulType::Implicit,
                    Box::new((-1f64).into()),
                    3f64.into(),
                ))))
                .into(),
            ),
        )
        .await;
    }

    #[tokio::test]
    async fn equality() {
        parse_test(
            "x=2",
            Ast::Equality(
                Factor::Variable(MathIdentifier::from_single_ident("x")).into(),
                2f64.into(),
            ),
        )
        .await;
    }

    #[tokio::test]
    async fn pmatrix_column_vector() {
        let mut matrix = Matrix::zero(3, 1);
        matrix.set(0, 0, 1f64.into());
        matrix.set(1, 0, 2f64.into());
        matrix.set(2, 0, 3f64.into());
        parse_test(
            r#"\begin{pmatrix} 1 \\ 2 \\ 3 \end{pmatrix}"#,
            Ast::Expression(Factor::Matrix(matrix).into()),
        )
        .await;
    }

    #[tokio::test]
    async fn pmatrix_row_vector() {
        let mut matrix = Matrix::zero(1, 3);
        matrix.set(0, 0, 1f64.into());
        matrix.set(0, 1, 2f64.into());
        matrix.set(0, 2, 3f64.into());
        parse_test(
            r#"\begin{pmatrix} 1 & 2 & 3 \end{pmatrix}"#,
            Ast::Expression(Factor::Matrix(matrix).into()),
        )
        .await;
    }

    #[tokio::test]
    async fn parenthesis_row_vector() {
        let mut matrix = Matrix::zero(1, 3);
        matrix.set(0, 0, 1f64.into());
        matrix.set(0, 1, 2f64.into());
        matrix.set(0, 2, 3f64.into());
        parse_test(
            r#"(1,2,3)"#,
            Ast::Expression(Factor::Matrix(matrix).into()),
        )
        .await;
    }

    #[tokio::test]
    async fn bmatrix_2x3() {
        let mut matrix = Matrix::zero(2, 3);
        matrix.set(0, 0, 1f64.into());
        matrix.set(0, 1, 2f64.into());
        matrix.set(0, 2, 3f64.into());
        matrix.set(1, 0, 4f64.into());
        matrix.set(1, 1, 5f64.into());
        matrix.set(1, 2, 6f64.into());
        parse_test(
            r#"\begin{bmatrix} 1 & 2 & 3 \\ 4 & 5 & 6  \end{bmatrix}"#,
            Ast::Expression(Factor::Matrix(matrix).into()),
        )
        .await;
    }

    #[tokio::test]
    async fn math_identifier_overline() {
        parse_test(
            r#"\overline{x}"#,
            Ast::Expression(
                Factor::Variable(MathIdentifier::Modifier(
                    ModifierType::Overline,
                    Box::new(
                        Factor::Variable(MathIdentifier::from_single_ident(
                            "x",
                        ))
                        .into(),
                    ),
                ))
                .into(),
            ),
        )
        .await;
    }

    #[tokio::test]
    async fn math_identifier_index_letter() {
        parse_test(
            "x_{y}",
            Ast::Expression(
                Factor::Variable(MathIdentifier::Index {
                    name: Box::new(MathIdentifier::from_single_ident("x")),
                    index: Box::new(
                        Factor::Variable(MathIdentifier::from_single_ident(
                            "y",
                        ))
                        .into(),
                    ),
                })
                .into(),
            ),
        )
        .await;
    }

    #[tokio::test]
    async fn math_identifier_index_brackets_digit() {
        parse_test(
            "x_{1}",
            Ast::Expression(
                Factor::Variable(MathIdentifier::Index {
                    name: Box::new(MathIdentifier::from_single_ident("x")),
                    index: Box::new(Factor::Constant(1.0).into()),
                })
                .into(),
            ),
        )
        .await;
    }

    #[tokio::test]
    async fn math_identifier_index_n_plus_one() {
        parse_test(
            "x_{n+1}",
            Ast::Expression(
                Factor::Variable(MathIdentifier::Index {
                    name: Box::new(MathIdentifier::from_single_ident("x")),
                    index: Box::new(MathExpr::Add(
                        Factor::Variable(MathIdentifier::from_single_ident(
                            "n",
                        ))
                        .into(),
                        1_f64.into(),
                    )),
                })
                .into(),
            ),
        )
        .await;
    }
}
