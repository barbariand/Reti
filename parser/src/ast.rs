use self::mathexpr::MathExpr;

pub mod mathexpr;

pub enum AST {
    MathExpr(MathExpr),
    Ignore(String), // other things than math expressions?
}
