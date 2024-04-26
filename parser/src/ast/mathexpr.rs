pub use self::{
    bin_ops::BinOp, customfunc::CustomFunction, diffirential::Differential, fraction::Fraction,
    integral::Integral, log::Logarithm, matrix::Matrix, modulo::Modulo, number::Number, sqrt::Root,
    sum::Sum, unary_functions::UnaryFunctions, unary_ops::UnaryOp, variable::Variable,
};

mod bin_ops;
mod customfunc;
mod diffirential;
mod fraction;
mod integral;
mod log;
mod matrix;
mod modulo;
mod number;
mod sqrt;

mod sum;
mod unary_functions;
mod unary_ops;
mod variable;
#[derive(Debug, Clone)]
pub enum MathExpr {
    Number(Number),               // Represents a numeric literal
    Variable(Variable),           // Represents a variable (e.g., "x")
    BinOp(BinOp),                 // Represents a binary operation (add, subtract, multiply, divide)
    UnaryOp(UnaryOp),             // Represents a unary operation (e.g., negation)
    FunctionCall(UnaryFunctions), // Represents a function call (e.g., sin, cos, exp)
    Matrix(Matrix),               // Represents a matrix (e.g. [[2,2],[2,2]])
    Integral(Integral),           // Represents a integral
    Sum(Sum),
    Differential(Differential),
    Fraction(Fraction),
    Root(Root),
    Logarithm(Logarithm),
    CustomFunction(CustomFunction),
    Mod(Modulo),
}
#[derive(Debug, Clone, Copy)]
pub struct MathExprKey(usize);
