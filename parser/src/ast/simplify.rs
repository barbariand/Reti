//! the implementations of simplification
use crate::prelude::*;
impl Ast {
    ///simplify the ast
    pub fn simplify(&self) -> Ast {
        println!("simplifying ast");
        match self {
            Ast::Expression(e) => Ast::Expression(e.simplify()),
            Ast::Equality(lhs, rhs) => {
                Ast::Equality(lhs.simplify(), rhs.simplify())
            }
        }
    }
}

impl MathExpr {
    ///Tries to simplify this
    fn simplify(&self) -> MathExpr {
        println!("simplifying mathexpr");
        match self {
            MathExpr::Term(t) => t.simplify(),
            MathExpr::Add(lhs, rhs) => {
                let simple = (lhs.simplify(), rhs.simplify());
                match simple {
                    (
                        MathExpr::Term(Term::Factor(Factor::Constant(a))),
                        MathExpr::Term(Term::Factor(Factor::Constant(b))),
                    ) => Factor::Constant(a + b).into(),
                    (MathExpr::Term(Term::Factor(Factor::Constant(a))),_)=>{
                        if a<f64::EPSILON{
                            simple.1
                        }else{
                            Self::add_wrapped(simple.0, simple.1)
                        }
                    }
                    (_,MathExpr::Term(Term::Factor(Factor::Constant(b))))=>{
                        if b<f64::EPSILON{
                            simple.0
                        }else{
                            Self::add_wrapped(simple.0, simple.1)
                        }
                    }
                    _ => {
                        Self::add_wrapped(simple.0, simple.1)
                    }
                }
            }
            MathExpr::Subtract(lhs, rhs) => {
                let simple = (lhs.simplify(), rhs.simplify());
                match simple {
                    (
                        MathExpr::Term(Term::Factor(Factor::Constant(a))),
                        MathExpr::Term(Term::Factor(Factor::Constant(b))),
                    ) => Factor::Constant(a - b).into(),
                    (MathExpr::Term(Term::Factor(Factor::Constant(a))),_)=>{
                        if a<f64::EPSILON{
                            simple.1
                        }else{
                            Self::subtract_wrapped(simple.0, simple.1)
                        }
                    }
                    (_,MathExpr::Term(Term::Factor(Factor::Constant(b))))=>{
                        if b<f64::EPSILON{
                            simple.0
                        }else{
                            Self::subtract_wrapped(simple.0, simple.1)
                        }
                    }
                    _ => {
                        Self::subtract_wrapped(simple.0, simple.1)
                    }
                }
            }
        }
    }
}

impl Term {
    ///test
    fn simplify(&self) -> MathExpr {
        println!("simplifying term");
        match self {
            Term::Factor(f) => f.simplify(),
            Term::Multiply(m, lhs, rhs) => Term::Multiply(
                m.clone(),
                {
                    let simple = lhs.simplify();

                    if let MathExpr::Term(t) = simple {
                        if let Term::Factor(Factor::Constant(c)) = t {
                            if (c - 1.0).abs() < f64::EPSILON {
                                return rhs.simplify();
                            } else if c.abs() < f64::EPSILON {
                                return Factor::Constant(0.0).into();
                            }
                        }
                        t.boxed()
                    } else {
                        Box::new(
                            Factor::Parenthesis(simple.clone().boxed()).into(),
                        )
                    }
                },
                {
                    let simple = rhs.simplify();
                    if let MathExpr::Term(Term::Factor(f)) = simple {
                        if let Factor::Constant(c) = f {
                            if (c - 1.0).abs() < f64::EPSILON {
                                return lhs.simplify();
                            } else if c.abs() < f64::EPSILON {
                                return Factor::Constant(0.0).into();
                            }
                        }
                        f
                    } else {
                        Factor::Parenthesis(simple.clone().boxed())
                    }
                },
            )
            .into(),
            Term::Divide(_, _) => todo!("divide"),
        }
    }
}

impl Factor {
    #[allow(dead_code)]
    ///simplifying factors
    fn simplify(&self) -> MathExpr {
        match self {
            Factor::Constant(_) => self.clone().into(),
            Factor::Parenthesis(p) => p.simplify(),
            Factor::Variable(_) => self.clone().into(),
            Factor::FunctionCall(_) => self.clone().into(),
            Factor::Power { base, exponent } => {
                println!("simplifying power");
                let simple_base = base.simplify();
                let exponent_simple = exponent.simplify();

                if let MathExpr::Term(Term::Factor(Factor::Constant(c))) =
                    exponent_simple
                {
                    println!("constat power: {c}");
                    if (c - 1.0).abs() < f64::EPSILON {
                        //exponent is 1
                        return simple_base;
                    } else if c < f64::EPSILON {
                        return Factor::Constant(0.0).into();
                    }
                }

                Factor::Power {
                    base: {
                        if let MathExpr::Term(Term::Factor(Factor::Constant(
                            c,
                        ))) = simple_base
                        {
                            if (c - 1.0).abs() < f64::EPSILON {
                                return Factor::Constant(1.0).into();
                            } else {
                                Factor::Constant(c).into()
                            }
                        } else {
                            Factor::Parenthesis(simple_base.boxed()).into()
                        }
                    },
                    exponent: exponent_simple.boxed(),
                }
                .into()
            }
            Factor::Root {
                degree: _,
                radicand: _,
            } => todo!(),
            Factor::Fraction(_, _) => todo!(),
            Factor::Abs(_) => todo!(),
            Factor::Matrix(_) => todo!(),
        }
    }
}
#[cfg(test)]
mod test {
    use crate::prelude::*;
    async fn ast_test_simplify(text: &str, expected_ast: Ast) {
        let found_ast = parse(text, &MathContext::standard_math())
            .await
            .expect("failed to parse AST")
            .simplify();
        // Compare and print with debug and formatting otherwise.
        if expected_ast != found_ast {
            panic!("Expected: {:#?}\nFound: {:#?}", expected_ast, found_ast);
        }
    }
    #[tokio::test]
    async fn one_minus_one() {
        ast_test_simplify("1-1", Ast::Expression(Factor::Constant(0.0).into()))
            .await;
    }
    #[tokio::test]
    async fn one_plus_one() {
        ast_test_simplify("1+1", Ast::Expression(Factor::Constant(2.0).into()))
            .await;
    }
    #[tokio::test]
    async fn one_times_one() {
        ast_test_simplify("1*1", Ast::Expression(Factor::Constant(1.0).into()))
            .await;
    }
    #[tokio::test]
    async fn one_times_zero() {
        ast_test_simplify("1*0", Ast::Expression(Factor::Constant(0.0).into()))
            .await;
    }
    #[tokio::test]
    async fn one_times_parenthasis() {
        ast_test_simplify(
            "0*(1+1+1+1+1*2)",
            Ast::Expression(Factor::Constant(0.0).into()),
        )
        .await;
    }
    #[tokio::test]
    async fn two_minus_one() {
        ast_test_simplify("2-1", Ast::Expression(Factor::Constant(1.0).into()))
            .await;
    }
}
