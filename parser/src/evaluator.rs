/// A simple single-threaded evaluator for an AST.
use crate::ast::{Ast, Factor, MathExpr, Term};

impl Ast {
    pub fn eval(&self) -> f64 {
        self.root_expr.eval()
    }
}

impl MathExpr {
    pub fn eval(&self) -> f64 {
        match self {
            MathExpr::Term(term) => term.eval(),
            MathExpr::Add(a, b) => a.eval() + b.eval(),
            MathExpr::Subtract(a, b) => a.eval() - b.eval(),
        }
    }
}

impl Term {
    pub fn eval(&self) -> f64 {
        match self {
            Term::Factor(factor) => factor.eval(),
            Term::Multiply(a, b) => a.eval() * b.eval(),
            Term::Divide(a, b) => a.eval() / b.eval(),
        }
    }
}

impl Factor {
    pub fn eval(&self) -> f64 {
        match self {
            Factor::Constant(c) => *c,
            Factor::Expression(expr) => expr.eval(),
            Factor::Variable(x) => todo!("I don't know the value of the variable {:?}", x),
            Factor::FunctionCall(call) => todo!("call = {:?}", call),
            Factor::Exponent { base, exponent } => base.eval().powf(exponent.eval()),
            Factor::Root { degree, radicand } => match degree.as_ref().map(|expr| expr.eval()) {
                Some(2.0) | None => radicand.eval().sqrt(),
                Some(0.0) => 1.0,
                Some(degree) => radicand.eval().powf(1.0 / degree),
            },
            Factor::Abs(val) => val.eval().abs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{Ast, MathExpr, Term};

    fn eval_test(expected: f64, ast: Ast) {
        let found = ast.eval();

        if (found - expected).abs() > f64::EPSILON {
            panic!("Found {} expected {}", found, expected);
        }
    }

    #[test]
    fn eval_1_plus_1() {
        eval_test(
            2.0,
            Ast {
                root_expr: MathExpr::Add(Box::new(1.0.into()), 1.0.into()),
            },
        );
    }

    #[test]
    fn eval_multiplication() {
        eval_test(
            17.0,
            // 2+3*5
            Ast {
                root_expr: MathExpr::Add(
                    Box::new(2.0.into()),
                    Term::Multiply(Box::new(3.0.into()), 5.0.into()),
                ),
            },
        );
    }
}
