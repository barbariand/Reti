//! convert the ast to latex

use crate::prelude::*;

impl MathExpr {
    pub fn to_latex(&self) -> String {
        match self {
            MathExpr::Term(term) => term.to_latex(),
            MathExpr::Add(a, b) => format!("{}+{}", a.to_latex(), b.to_latex()),
            MathExpr::Subtract(a, b) => {
                format!("{}-{}", a.to_latex(), b.to_latex())
            }
        }
    }
}

impl Term {
    pub fn to_latex(&self) -> String {
        match self {
            Term::Factor(factor) => factor.to_latex(),
            Term::Multiply(mul_type, a, b) => {
                let mul_token = match mul_type {
                    MulType::Asterisk => "*",
                    MulType::Cdot => "\\cdot",
                    MulType::Times => "\\times",
                    MulType::Implicit => "",
                };
                format!("{}{}{}", a.to_latex(), mul_token, b.to_latex())
            }
            Term::Divide(a, b) => {
                format!("\\frac{{{}}}{{{}}}", a.to_latex(), b.to_latex())
            }
        }
    }
}

impl Factor {
    pub fn to_latex(&self) -> String {
        match self {
            Factor::Constant(c) => format!("{}", c),
            Factor::Parenthesis(expr) => {
                format!("\\left({}\\right)", expr.to_latex())
            }
            Factor::Variable(var) => var.to_latex(),
            Factor::FunctionCall(call) => {
                let args = call
                    .arguments
                    .iter()
                    .map(|arg| arg.to_latex())
                    .collect::<String>();
                format!("{}({})", call.function_name.to_latex(), args)
            }
            Factor::Power { base, exponent } => {
                format!("{}^{{{}}}", base.to_latex(), exponent.to_latex())
            }
            Factor::Root { degree, radicand } => {
                if let Some(degree) = degree {
                    format!(
                        "\\sqrt[{}]{{{}}}",
                        degree.to_latex(),
                        radicand.to_latex()
                    )
                } else {
                    format!("\\sqrt{{{}}}", radicand.to_latex())
                }
            }
            Factor::Fraction(a, b) => {
                format!("\\frac{{{}}}{{{}}}", a.to_latex(), b.to_latex())
            }
            Factor::Abs(val) => format!("|{}|", val.to_latex()),
            Factor::Matrix(m) => {
                let mut str = String::new();
                str.push_str("\\begin{bmatrix}");

                let content = (0..m.row_count())
                    .map(|row| {
                        (0..m.column_count())
                            .map(|column| m.get(row, column).to_latex())
                            .collect::<Vec<_>>()
                            .join(" & ")
                    })
                    .collect::<Vec<_>>()
                    .join(" \\\\");

                str.push_str(&content);
                str.push_str("\\end{bmatrix}");
                str
            }
        }
    }
}

impl MathIdentifier {
    pub fn to_latex(&self) -> String {
        self.tokens
            .iter()
            .map(|token| format!("{}", token))
            .collect()
    }
}
