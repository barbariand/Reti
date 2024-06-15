//! convert the ast to latex

use crate::prelude::*;

///Converting the AST to latex
pub trait ToLaTeX {
    ///The function to convert it back to latex
    fn to_latex(&self) -> String;
}

impl ToLaTeX for Ast {
    fn to_latex(&self) -> String {
        match self {
            Ast::Expression(e) => e.to_latex(),
            Ast::Equality(a, b) => format!("{}={}", a.to_latex(), b.to_latex()),
        }
    }
}
impl ToLaTeX for MathExpr {
    fn to_latex(&self) -> String {
        match self {
            MathExpr::Term(term) => term.to_latex(),
            MathExpr::Add(a, b) => format!("{}+{}", a.to_latex(), b.to_latex()),
            MathExpr::Subtract(a, b) => {
                format!("{}-{}", a.to_latex(), b.to_latex())
            }
        }
    }
}

impl ToLaTeX for Term {
    fn to_latex(&self) -> String {
        match self {
            Term::Factor(factor) => factor.to_latex(),
            Term::Multiply(mul_type, a, b) => {
                let mut mul_type = mul_type;
                if mul_type == &MulType::Implicit {
                    mul_type = &MulType::Cdot;
                    // TODO detect when implicit multiplication is possible,
                    // eg mul(1, 2) and mul(x2, 3y) cannot be implicit, because
                    // 12 and x23y is incorrect, but mul(2, x) can be implicit
                    // since it would be 2x.
                }
                let mul_token = match mul_type {
                    MulType::Asterisk => "*",
                    MulType::Cdot => "\\cdot ",
                    MulType::Times => "\\times ",
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

impl ToLaTeX for Factor {
    fn to_latex(&self) -> String {
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
                let mut output = String::new();
                output.push_str("\\begin{bmatrix}");

                let content = (0..m.row_count())
                    .map(|row| {
                        (0..m.column_count())
                            .map(|column| m.get(row, column).to_latex())
                            .collect::<Vec<_>>()
                            .join(" & ")
                    })
                    .collect::<Vec<_>>()
                    .join(" \\\\");

                output.push_str(&content);
                output.push_str("\\end{bmatrix}");
                output
            }
        }
    }
}

impl ToLaTeX for MathIdentifier {
    fn to_latex(&self) -> String {
        self.tokens.iter().fold(String::new(), |mut output, token| {
            output.push_str(&token.to_string());
            output
        })
    }
}
