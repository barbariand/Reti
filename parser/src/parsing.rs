//!Parsing the TokenStream to an AST
use tracing::{trace, trace_span};

use crate::{
    identifier::{MathLetter, MathString, ModifierType},
    prelude::*,
};
///Parser for parsing the stream when it is done by the normalizer
pub struct Parser<I>
where
    I: IntoIterator<Item = Token>,
{
    ///token stream from the normalizer
    reader: TokenReader<I>,
    ///the context in witch it operates in
    context: MathContext,
}

impl<I> Parser<I>
where
    I: IntoIterator<Item = Token>,
{
    ///creating a new
    pub fn new(tokens: I, context: MathContext) -> Self {
        trace!("created Parser");

        Parser {
            reader: TokenReader::new(tokens),
            context,
        }
    }
    ///Starting the parser
    pub fn parse(mut self) -> Result<Ast, ParseError> {
        let span = trace_span!("parse");
        let _enter = span.enter();

        // Parse expression
        let root_expr = self.expr()?;
        trace!("root_expr = {root_expr:?}");

        // Check if we have more to read, if not, that means we have a full
        // expression we can return.
        let next = self.reader.read();
        match next {
            None => return Ok(Ast::Expression(root_expr)),
            Some(Token::Equals) => {
                // An equality. Try parse a right hand side.
                let rhs = self.expr()?;
                let next = self.reader.read();
                trace!("trailing = {next:?}");
                if let Some(next) = next {
                    return Err(ParseError::Trailing { token: next });
                }

                return Ok(Ast::Equality(root_expr, rhs));
            }
            n => {
                // It seems we have expected trailing tokens.
                // This means we failed to parse the expression fully.
                Err(ParseError::Trailing {
                    token: n.expect("it is some"),
                })
            }
        }
    }
    ///expect the next token to be of a type
    pub(crate) fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        let found = self.reader.read().ok_or_else(|| {
            ParseError::UnexpectedEndOfContent {
                expected: vec![expected.clone()],
            }
        })?;
        if found == expected {
            return Ok(());
        }
        Err(ParseError::UnexpectedToken {
            expected: vec![expected],
            found: found,
        })
    }
    ///reads the token until it is not a char and then adds them up
    fn read_identifier(&mut self) -> Result<String, ParseError> {
        let token = self.reader.read();
        match token {
            Some(Token::Identifier(val)) => Ok(val),
            Some(found) => Err(ParseError::UnexpectedToken {
                expected: vec![Token::Identifier("".to_owned())],
                found,
            }),
            None => Err(ParseError::UnexpectedEndOfContent {
                expected: vec![Token::Identifier("".to_owned())],
            }),
        }
    }

    /// Parse a mathematical expression that consists of multiple terms added
    /// and subtracted.
    fn expr(&mut self) -> Result<MathExpr, ParseError> {
        let mut expr = MathExpr::Term(self.term()?);

        loop {
            let next = self.reader.peek();
            match next {
                Some(Token::Plus) => {
                    self.reader.skip();
                    let rhs = self.term()?;
                    expr = MathExpr::Add(Box::new(expr), rhs);
                }
                Some(Token::Minus) => {
                    self.reader.skip();
                    let rhs = self.term()?;
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

    fn term(&mut self) -> Result<Term, ParseError> {
        let mut term = Term::Factor(self.factor()?);

        loop {
            let next = self.reader.peek_range(0..=1);
            trace!("term: {next:?}");
            match next[..] {
                [Some(Token::Asterisk), _] => {
                    self.reader.skip();
                    let rhs = self.factor()?;
                    term =
                        Term::Multiply(MulType::Asterisk, Box::new(term), rhs);
                }
                [Some(Token::Backslash), Some(Token::Identifier(ident))]
                    if ident == "cdot"
                        || ident == "cdotp"
                        || ident == "times" =>
                {
                    let mul_type = match ident.as_str() {
                        "cdot" | "cdotp" => MulType::Cdot,
                        "times" => MulType::Times,
                        _ => unreachable!("invalid multype"),
                    };
                    self.reader.skip();
                    self.reader.skip();
                    let rhs = self.factor()?;
                    term = Term::Multiply(mul_type, Box::new(term), rhs);
                }
                [Some(Token::Slash), _] => {
                    self.reader.skip();
                    let rhs = self.factor()?;
                    term = Term::Divide(Box::new(term), rhs);
                }
                [Some(Token::Backslash), Some(Token::Backslash)] => {
                    break;
                }
                [Some(Token::Backslash), Some(Token::Identifier(ident))]
                    if ident == "end" =>
                {
                    break;
                }
                // Implicit multiplication
                [Some(Token::Identifier(_))
                | Some(Token::NumberLiteral(_))
                | Some(Token::Backslash)
                | Some(Token::LeftParenthesis), _] => {
                    let rhs = self.factor()?;
                    term =
                        Term::Multiply(MulType::Implicit, Box::new(term), rhs);
                }
                [None, None] => {
                    break;
                }
                [Some(_), None] => {
                    break;
                }
                [None, Some(_)] => {
                    unreachable!("iterator is walking backwards")
                }
                [Some(_), Some(_)] => break,
                _ => break,
            }
        }

        Ok(term)
    }

    /// If the next character is an identifier, ensure that it only has length
    /// one by splitting it.
    fn split_next_identifier(&mut self) {
        if let Some(Token::Identifier(text)) = self.reader.peek() {
            if text.len() > 1 {
                let mut tokens = Vec::new();
                for c in text.chars() {
                    tokens.push(Token::Identifier(c.to_string()));
                }
                self.reader.replace(0..=0, tokens);
            }
        }
    }

    /// Parse a factor, and if the factor has an exponent attached to it, parse
    /// the exponent too.
    fn factor(&mut self) -> Result<Factor, ParseError> {
        // Split identifiers into single characters
        self.split_next_identifier();
        // First read a factor, but then see if we have exponents after it.
        // Exponents need to be baked into the factor since exponents should
        // be evaluated before multiplications.
        //
        let factor = match self.reader.read() {
            Some(Token::NumberLiteral(val)) => Factor::Constant(val),
            Some(Token::LeftParenthesis) => {
                // In most cases, this is one value, for example (1+1).
                // But parse many values since it could be a vector (1,2,3)
                // if commas are encountered.
                let mut values = Vec::with_capacity(1);
                loop {
                    let expr = self.expr()?;
                    values.push(expr);
                    let next = self.reader.peek();
                    if next != Some(&Token::Comma) {
                        break;
                    }
                    self.reader.skip();
                }
                self.expect(Token::RightParenthesis)?;
                let len = values.len();
                if len == 1 {
                    Factor::Parenthesis(Box::new(values.remove(0)))
                } else {
                    Factor::Matrix(Matrix::new(values, 1, len))
                }
            }
            Some(Token::Backslash) => {
                let command = self.read_identifier()?;
                self.factor_command(&command)?
            }
            Some(Token::VerticalPipe) => {
                let expr = self.expr()?;
                self.expect(Token::VerticalPipe)?;
                Factor::Abs(Box::new(expr))
            }
            Some(Token::Identifier(ident)) => {
                if ident.chars().count() != 1 {
                    panic!("Identifier was not splitted correctly.")
                }
                let math_identifier = MathIdentifier::from_single_ident(&ident);
                let math_identifier =
                    self.math_identifier_tail(math_identifier)?;
                self.factor_identifier(math_identifier)?
            }
            Some(Token::Minus) => Factor::Constant((-1.0).into()),
            Some(token) => return Err(ParseError::InvalidFactor { token }),
            None => {
                return Err(ParseError::UnexpectedEndOfContent {
                    expected: vec![Token::Minus, Token::Plus],
                })
            }
        };

        let next = self.reader.peek();
        if next == Some(&Token::Caret) {
            // This factor is an exponential
            self.reader.skip();
            return self.factor_exponent(factor);
        }

        Ok(factor)
    }

    /// Parse a factor that is a LaTeX command.
    ///
    /// The `command` parameter is the LaTeX command.
    fn factor_command(&mut self, command: &str) -> Result<Factor, ParseError> {
        Ok(match command {
            "sqrt" => {
                let next = self.reader.peek();
                let mut degree = None;
                if next == Some(&Token::LeftBracket) {
                    self.reader.skip();
                    degree = Some(Box::new(self.expr()?));
                    self.expect(Token::RightBracket)?;
                }
                self.expect(Token::LeftCurlyBracket)?;
                let radicand = Box::new(self.expr()?);
                self.expect(Token::RightCurlyBracket)?;
                Factor::Root { degree, radicand }
            }
            "frac" => {
                self.expect(Token::LeftCurlyBracket)?;
                let numerator = Box::new(self.expr()?);
                self.expect(Token::RightCurlyBracket)?;
                self.expect(Token::LeftCurlyBracket)?;
                let denominator = Box::new(self.expr()?);
                self.expect(Token::RightCurlyBracket)?;
                Factor::Fraction(numerator, denominator)
            }
            "begin" => {
                self.expect(Token::LeftCurlyBracket)?;
                let s = self.read_identifier()?;
                match s.as_str() {
                    "bmatrix" | "pmatrix" | "Bmatrix" => {
                        self.expect(Token::RightCurlyBracket)?;
                        Factor::Matrix(self.matrix(s)?)
                    }
                    "vmatrix" | "Vmatrix" => {
                        self.expect(Token::RightCurlyBracket)?;
                        let matrix = self.matrix(s)?;
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
                let ident = self.parse_math_identifier_command(command)?;
                self.factor_identifier(ident)?
            }
        })
    }

    /// Parse a [MathIdentifier] that starts with a command.
    fn parse_math_identifier_command(
        &mut self,
        command: &str,
    ) -> Result<MathIdentifier, ParseError> {
        let letter = MathLetter::from_latex(command);
        let modifier = ModifierType::from_latex(command);
        let math_identifier = if let Some(letter) = letter {
            let math_str = MathString::from_letters(vec![letter]);
            MathIdentifier::Name(math_str)
        } else if let Some(modifier) = modifier {
            let inner = self.parse_inner_math_identifier()?;
            MathIdentifier::Modifier(modifier, Box::new(inner))
        } else {
            return Err(ParseError::InvalidIdentifierCommmand {
                command: command.to_string(),
            });
        };
        // Parse index if there is one and then return.
        self.math_identifier_tail(math_identifier)
    }

    /// Parse an inner identifier which may be one character long, or if
    /// surrounded by curly brackets may hold an entire inner identifier.
    fn parse_inner_math_identifier(&mut self) -> Result<MathExpr, ParseError> {
        if self.reader.peek() == Some(&Token::LeftCurlyBracket) {
            self.reader.skip();
            let inner = self.expr()?;
            self.expect(Token::RightCurlyBracket)?;
            Ok(inner)
        } else {
            todo!("handle single character inner. For example \\overline x or x_1.");
        }
    }

    /// Parse the index of an identifier, if there is an index. Otherwise return
    /// the identifier as is.
    fn math_identifier_tail(
        &mut self,
        ident: MathIdentifier,
    ) -> Result<MathIdentifier, ParseError> {
        // Check for index
        Ok(if self.reader.peek() == Some(&Token::Underscore) {
            self.reader.skip();
            let index = self.parse_inner_math_identifier()?;
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
    fn factor_identifier(
        &mut self,
        identifier: MathIdentifier,
    ) -> Result<Factor, ParseError> {
        // This might be a function.
        if self.context.is_defined_function(&identifier) {
            Ok(self.factor_function_call(identifier)?)
        } else {
            Ok(Factor::Variable(identifier))
        }
    }

    /// Parse the exponent part of a factor.
    ///
    /// The `factor` parameter is the base, and the tokens to be parsed by this
    /// function is the exponent.
    fn factor_exponent(
        &mut self,
        factor: Factor,
    ) -> Result<Factor, ParseError> {
        let next = self.reader.peek();
        let exponent = match next {
            Some(Token::LeftCurlyBracket) => {
                self.reader.skip();
                let expr = self.expr()?;
                self.expect(Token::RightCurlyBracket)?;
                expr
            }
            Some(Token::Backslash) => {
                let factor = self.factor()?;
                MathExpr::Term(Term::Factor(factor))
            }
            Some(Token::Identifier(ident)) => {
                let ident = ident.clone();
                self.reader.skip();
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
            Some(Token::NumberLiteral(num)) => {
                if num.0.len() != 1 {
                    panic!(
                        "The normalizer did not correctly handle exponent, got num = {:?}",
                        num
                    );
                }
                // Get the lifetime of the &NumberLiteral to end before
                // the reader gets used twice
                let res =
                    MathExpr::Term(Term::Factor(Factor::Constant(num.clone())));
                self.reader.skip();
                res
            }
            Some(token) => {
                return Err(ParseError::Invalid {
                    token: token.clone(),
                })
            }
            None => {
                return Err(ParseError::UnexpectedEndOfContent {
                    expected: vec![
                        Token::LeftCurlyBracket,
                        Token::NumberLiteral("0".into()),
                    ],
                })
            }
        };

        Ok(Factor::Power {
            base: Box::new(factor),
            exponent: Box::new(exponent),
        })
    }
    ///parsing a suspected function
    fn factor_function_call(
        &mut self,
        function_name: MathIdentifier,
    ) -> Result<Factor, ParseError> {
        let mut arguments = Vec::new();
        // Read arguments in parenthesis, eg. f(1, 2)
        if self.reader.peek() == Some(&Token::LeftParenthesis) {
            self.reader.skip();
            loop {
                let next = self.reader.peek();
                match next {
                    Some(Token::RightParenthesis) => break,
                    Some(Token::Comma) => self.reader.skip(),
                    _ => {
                        let expr = self.expr()?;
                        arguments.push(expr);
                    }
                }
            }
            self.expect(Token::RightParenthesis)?;
        } else {
            // Read one explicit argument, for example \ln 2
            let arg = self.term()?;
            arguments.push(MathExpr::Term(arg));
        }

        Ok(Factor::FunctionCall(FunctionCall {
            function_name,
            arguments,
        }))
    }
    ///Parsing a suspected matrix
    fn matrix(
        &mut self,
        matrix_type: String,
    ) -> Result<Matrix<MathExpr>, ParseError> {
        let mut rows = Vec::new();
        let mut current_row = Vec::new();
        let mut column_count = Option::None;

        loop {
            let cell = self.expr()?;
            current_row.push(cell);

            let next = self.reader.peek();
            match next {
                Some(Token::Ampersand) => {
                    self.reader.skip();
                    continue;
                }
                Some(Token::Backslash) => {
                    if self.reader.peekn(1) == Some(&Token::Backslash) {
                        // Two backslashes means end of row.
                        self.reader.skip();
                        self.reader.skip();
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
                        self.reader.skip();
                        self.expect(Token::Identifier("end".to_owned()))?;
                        self.expect(Token::LeftCurlyBracket)?;
                        self.expect(Token::Identifier(matrix_type))?;
                        self.expect(Token::RightCurlyBracket)?;
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
                Some(e) => {
                    return Err(ParseError::UnexpectedToken {
                        expected: vec![Token::Ampersand, Token::Backslash],
                        found: e.clone(),
                    });
                }
                None => {
                    return Err(ParseError::UnexpectedEndOfContent {
                        expected: vec![Token::Ampersand, Token::Backslash],
                    })
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
    use tracing_test::traced_test;
    fn parse_test(text: &str, expected_ast: Ast) {
        let found_ast = parse(text, &MathContext::standard_math());
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

    #[traced_test]
    #[test]
    fn constant() {
        parse_test("1", Ast::Expression(1f64.into()));
    }

    #[traced_test]
    #[test]
    fn addition() {
        parse_test(
            "1+2+3",
            Ast::Expression(MathExpr::Add(
                Box::new(MathExpr::Add(Box::new(1f64.into()), 2f64.into())),
                3f64.into(),
            )),
        );
    }

    #[traced_test]
    #[test]
    fn addition_multiplication_order_of_operations() {
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
        );
    }

    #[traced_test]
    #[test]
    fn multiplication_asterisk() {
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
        );
    }

    #[traced_test]
    #[test]
    fn multiplication_cdot() {
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
        );
    }

    #[traced_test]
    #[test]
    fn multiplication_times() {
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
        );
    }

    #[traced_test]
    #[test]
    fn sqrt() {
        parse_test(
            "\\sqrt{9}",
            Ast::Expression(
                Factor::Root {
                    degree: None,
                    radicand: 9f64.into(),
                }
                .into(),
            ),
        );
    }

    #[traced_test]
    #[test]
    fn cube_root() {
        parse_test(
            "\\sqrt[3]{27}",
            Ast::Expression(
                Factor::Root {
                    degree: Some(3f64.into()),
                    radicand: 27f64.into(),
                }
                .into(),
            ),
        );
    }

    #[traced_test]
    #[test]
    fn exponent() {
        parse_test(
            "2^{3}",
            Ast::Expression(
                Factor::Power {
                    base: Box::new(Factor::Constant(2.0.into())),
                    exponent: Box::new(MathExpr::Term(Term::Factor(
                        Factor::Constant(3.0.into()),
                    ))),
                }
                .into(),
            ),
        );
    }

    #[traced_test]
    #[test]
    fn exponent_command() {
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
        );
    }

    #[traced_test]
    #[test]
    fn exponent_split_token() {
        parse_test(
            "2^025", // this is 2^0 * 25
            Ast::Expression(MathExpr::Term(Term::Multiply(
                MulType::Implicit,
                //2^0
                Box::new(Term::Factor(Factor::Power {
                    base: Box::new(Factor::Constant("2.0".into())),
                    exponent: Box::new(MathExpr::Term(Term::Factor(
                        Factor::Constant("0.0".into()),
                    ))),
                })),
                // 25
                Factor::Constant("25.0".into()),
            ))),
        );
    }

    #[traced_test]
    #[test]
    fn parenthesis_and_exponent() {
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
        );
    }

    #[traced_test]
    #[test]
    fn implicit_multiplication_and_exponent_order_of_operations() {
        parse_test(
            "2x^{2} + 5xy",
            Ast::Expression(MathExpr::Add(
                // 2x^{2}
                Box::new(MathExpr::Term(Term::Multiply(
                    MulType::Implicit,
                    // 2
                    Box::new(Term::Factor(Factor::Constant("2.0".into()))),
                    // x^{2}
                    Factor::Power {
                        base: Box::new(Factor::Variable(
                            MathIdentifier::from_single_ident("x"),
                        )),
                        exponent: Box::new(MathExpr::Term(Term::Factor(
                            Factor::Constant("2.0".into()),
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
        );
    }

    #[traced_test]
    #[test]
    fn implicit_multiplication_single_identifier_token() {
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
        );
    }

    #[traced_test]
    #[test]
    fn pi() {
        parse_test(
            "\\pi",
            Ast::Expression(MathExpr::Term(Term::Factor(Factor::Variable(
                MathIdentifier::from_single_greek(GreekLetter::LowercasePi),
            )))),
        );
    }

    #[traced_test]
    #[test]
    fn implicit_multiplication_vs_function_call() {
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
        );
    }

    #[traced_test]
    #[test]
    fn division_order_of_operations() {
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
                        Box::new(Term::Factor(Factor::Constant("5.0".into()))),
                        // 2
                        Factor::Constant("2.0".into()),
                    )),
                    // x
                    Factor::Variable(MathIdentifier::from_single_ident("x")),
                ))),
                // 3
                Term::Factor(Factor::Constant("3.0".into())),
            )),
        );
    }

    #[traced_test]
    #[test]
    fn fraction() {
        parse_test(
            "\\frac{1}{2}",
            Ast::Expression(Factor::Fraction(1f64.into(), 2f64.into()).into()),
        );
    }

    #[traced_test]
    #[test]
    fn abs() {
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
        );
    }

    #[traced_test]
    #[test]
    fn equality() {
        parse_test(
            "x=2",
            Ast::Equality(
                Factor::Variable(MathIdentifier::from_single_ident("x")).into(),
                2f64.into(),
            ),
        );
    }

    #[traced_test]
    #[test]
    fn pmatrix_column_vector() {
        let mut matrix = Matrix::zero(3, 1);
        matrix.set(0, 0, 1f64.into());
        matrix.set(1, 0, 2f64.into());
        matrix.set(2, 0, 3f64.into());
        parse_test(
            r#"\begin{pmatrix} 1 \\ 2 \\ 3 \end{pmatrix}"#,
            Ast::Expression(Factor::Matrix(matrix).into()),
        );
    }

    #[traced_test]
    #[test]
    fn pmatrix_row_vector() {
        let mut matrix = Matrix::zero(1, 3);
        matrix.set(0, 0, 1f64.into());
        matrix.set(0, 1, 2f64.into());
        matrix.set(0, 2, 3f64.into());
        parse_test(
            r#"\begin{pmatrix} 1 & 2 & 3 \end{pmatrix}"#,
            Ast::Expression(Factor::Matrix(matrix).into()),
        );
    }

    #[traced_test]
    #[test]
    fn parenthesis_row_vector() {
        let mut matrix = Matrix::zero(1, 3);
        matrix.set(0, 0, 1f64.into());
        matrix.set(0, 1, 2f64.into());
        matrix.set(0, 2, 3f64.into());
        parse_test(
            r#"(1,2,3)"#,
            Ast::Expression(Factor::Matrix(matrix).into()),
        );
    }

    #[traced_test]
    #[test]
    fn bmatrix_2x3() {
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
        );
    }

    #[traced_test]
    #[test]
    fn math_identifier_overline() {
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
        );
    }

    #[traced_test]
    #[test]
    fn math_identifier_index_letter() {
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
        );
    }

    #[traced_test]
    #[test]
    fn math_identifier_index_brackets_digit() {
        parse_test(
            "x_{1}",
            Ast::Expression(
                Factor::Variable(MathIdentifier::Index {
                    name: Box::new(MathIdentifier::from_single_ident("x")),
                    index: Box::new(Factor::Constant("1.0".into()).into()),
                })
                .into(),
            ),
        );
    }

    #[traced_test]
    #[test]
    fn math_identifier_index_n_plus_one() {
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
        );
    }
}
