use self::mathexpr::MathExpr;

mod mathexpr;

enum AST {
    MathExpr(MathExpr),
    Ignore(String), // other things than math expressions?
}
