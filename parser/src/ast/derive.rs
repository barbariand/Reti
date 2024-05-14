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
                rhs.derivative(dependent)?.expect_term()?.clone(),
            ),
            MathExpr::Subtract(lhs, rhs) => MathExpr::Subtract(
                lhs.derivative(dependent)?.boxed(),
                rhs.derivative(dependent)?.expect_term()?.clone(),
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
                            .expect_term()?
                            .clone()
                            .boxed(),
                        lhs.clone(),
                    )
                    .into(),
                ),
                Term::Multiply(
                    mul.clone(),
                    rhs.clone(),
                    lhs.derivative(dependent)?.get_factor()?.clone(),
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
                        Term::Factor(exponent.get_factor()?.clone()).boxed(),
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
                    base.derivative(dependent)?.get_factor()?.clone(),
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
                    exponent.derivative(dependent)?.get_factor()?.clone(),
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
    async fn ast_test_derive(
        text: &str,
        dependent: &MathIdentifier,
        expected_ast: Ast,
    ) {
        let found_ast = parse(text, &MathContext::standard_math())
            .await
            .expect("failed to parse AST")
            .derivative(dependent)
            .expect("Failed")
            .simplify();

        // Compare and print with debug and formatting otherwise.
        if expected_ast != found_ast {
            panic!("Expected: {:#?}\nFound: {:#?}", expected_ast, found_ast);
        }
    }

    #[tokio::test]
    async fn x_squared_derivative() {
        ast_test_derive(
            "x^2",
            &MathIdentifier::from_single_ident("x"),
            Ast::Expression(Term::Multiply(MulType::Implicit, Factor::Constant(2.0).into(), Factor::Variable(MathIdentifier::from_single_ident("x"))).into()),
        )
        .await;
    }
}
