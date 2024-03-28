
pub use self::{
    bin_ops::BinOp, customfunc::CustomFunction, diffirential::Differential, fraction::Fraction,
    unary_functions::UnaryFunctions, integral::Integral, log::Logarithm, matrix::Matrix, number::Number,
    sqrt::Root, sum::Sum, unary_ops::UnaryOp, variable::Variable,modulo::Modulo
};

mod bin_ops;
mod customfunc;
mod diffirential;
mod fraction;
mod unary_functions;
mod integral;
mod log;
mod matrix;
mod number;
mod sqrt;
mod state;
mod sum;
mod unary_ops;
mod variable;
mod modulo;
#[derive(Debug, Clone)]
pub enum MathExpr {
    Number(Number),                  // Represents a numeric literal
    Variable(Variable),              // Represents a variable (e.g., "x")
    BinOp(Box<BinOp>),               // Represents a binary operation (add, subtract, multiply, divide)
    UnaryOp(Box<UnaryOp>),           // Represents a unary operation (e.g., negation)
    FunctionCall(Box<UnaryFunctions>), // Represents a function call (e.g., sin, cos, exp)
    Matrix(Matrix),                  // Represents a matrix (e.g. [[2,2],[2,2]])
    Integral(Box<Integral>),         // Represents a integral
    Sum(Box<Sum>),
    Differential(Box<Differential>),
    Fraction(Box<Fraction>),
    Root(Box<Root>),
    Logarithm(Box<Logarithm>),
    CustomFunction(Box<CustomFunction>),
    Mod(Box<Modulo>)
}
