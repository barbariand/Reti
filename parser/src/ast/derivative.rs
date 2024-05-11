//! the derive implementations
use crate::prelude::*;
impl Ast {
    ///doing derivation for the AST
    pub fn derivative(
        &self,
        dependent: &MathIdentifier,
    ) -> Result<Ast, EvalError> {
        Ok(match self {
            Ast::Expression(m) => Ast::Expression(m.derivative(dependent)?),
            Ast::Equality(lhs, rhs) => Ast::Equality(
                lhs.derivative(dependent)?,
                rhs.derivative(dependent)?,
            ),
        })
    }
}

impl MathExpr {
    ///doing derivation for the math expression
    pub fn derivative(
        &self,
        dependent: &MathIdentifier,
    ) -> Result<MathExpr, EvalError> {
        Ok(match self {
            MathExpr::Term(t) => t.derivative(dependent)?,
            MathExpr::Add(lhs, rhs) => MathExpr::Add(
                lhs.derivative(dependent)?.boxed(),
                rhs.derivative(dependent)?.get_term_or_wrap().clone(),
            ),
            MathExpr::Subtract(lhs, rhs) => MathExpr::Subtract(
                lhs.derivative(dependent)?.boxed(),
                rhs.derivative(dependent)?.get_term_or_wrap().clone(),
            ),
        })
    }
}
impl Term {
    ///doing derivation for the Term
    pub fn derivative(
        &self,
        dependent: &MathIdentifier,
    ) -> Result<MathExpr, EvalError> {
        Ok(match self {
            Term::Factor(f) => f.derivative(dependent)?,
            Term::Multiply(mul, rhs, lhs) => MathExpr::Add(
                Box::new(
                    Term::Multiply(
                        mul.clone(),
                        rhs.derivative(dependent)?
                            .get_term_or_wrap()
                            .clone()
                            .boxed(),
                        lhs.clone(),
                    )
                    .into(),
                ),
                Term::Multiply(
                    mul.clone(),
                    rhs.clone(),
                    lhs.derivative(dependent)?.get_factor_or_wrap().clone(),
                ),
            ),
            Term::Divide(_, _) => todo!("division"),
        })
    }
}
impl Factor {
    ///doing derivation for the Factor
    pub fn derivative(
        &self,
        dependent: &MathIdentifier,
    ) -> Result<MathExpr, EvalError> {
        Ok(match self {
            Factor::Constant(_) => Factor::Constant(0.0).into(),
            Factor::Parenthesis(e) => e.derivative(dependent)?,
            Factor::Variable(v) => match v == dependent {
                true => Factor::Constant(1.0).into(),
                false => Factor::Constant(0.0).into(),
            },
            Factor::FunctionCall(_) => todo!("function call"),
            Factor::Power { base, exponent } => MathExpr::Add(
                Term::Multiply(
                    MulType::Implicit,
                    Term::Multiply(
                        MulType::Implicit,
                        Term::Factor(exponent.get_factor_or_wrap().clone()).boxed(),
                        Factor::Power {
                            base: base.clone(),
                            exponent: MathExpr::Subtract(
                                exponent.clone(),
                                Factor::Constant(1.0).into(),
                            )
                            .boxed(),
                        },
                    )
                    .into(),
                    base.derivative(dependent)?.get_factor_or_wrap().clone(),
                )
                .into(),
                Term::Multiply(
                    MulType::Implicit,
                    Term::Multiply(
                        MulType::Implicit,
                        Factor::FunctionCall(FunctionCall::new(
                            MathIdentifier::new(vec![
                                Token::Backslash,
                                Token::Identifier("ln".to_owned()),
                            ]),
                            vec![*exponent.clone()],
                        ))
                        .into(),
                        Factor::Power {
                            base: base.clone(),
                            exponent: exponent.clone(),
                        },
                    )
                    .into(),
                    exponent.derivative(dependent)?.get_factor_or_wrap().clone(),
                ),
            ),

            Factor::Root {
                degree: _,
                radicand: _,
            } => todo!("root"),
            Factor::Fraction(_, _) => todo!("fraction"),
            Factor::Abs(_math) => todo!("ABS"),
            Factor::Matrix(_) => todo!("matrix"),
        })
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use pretty_assertions::assert_eq;
    async fn ast_test_derive(
        text: &str,
        dependent: &MathIdentifier,
        expected_to_ast:&str,
    ) {
        let context=MathContext::standard_math();
        let found_ast = parse(text,&context )
            .await
            .expect("failed to parse AST")
            .derivative(dependent)
            .expect("Failed_ ")
            .simplify();
        let expected_ast=parse(expected_to_ast, &context).await.expect("could not parse the expected ast");
        // Compare and print with debug and formatting otherwise.
        assert_eq!(found_ast,expected_ast,"found/expected")
    }

    #[tokio::test]
    async fn x_squared_derivative() {
        ast_test_derive(
            "x^2",
            &MathIdentifier::from_single_ident("x"),
            "2x",
        )
        .await;
    }
    #[tokio::test]
    async fn polynomial_1() {
        ast_test_derive(
            "3x^2+2x+1",
            &MathIdentifier::from_single_ident("x"),
            "3(2x)+2",
        )
        .await;
    }
}