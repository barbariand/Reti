pub enum Token{
    CommandPrefix,
    Fraction,
    ExpressionBegin,
    ExpressionEnd,
    BracketBegin,
    BracketEnd,
    Literal(f64),
    Constants,
    UnrecognicedCommand(String),
    Modulo,
    Div,
    Add,
    Subtract,
    Multiply,
    Log,
    Derrivate,
    Negative,
    Cos,
    Sin,
    Tan,
    ACos,
    ASin,
    ATan,
    Floor,
    Ceil,
    Floor,
    Ln,
    Pow,
    Integral,
    Sqrt,
    Sum,
    Binom,
    Underscore,
    Caret,
    VerticalPipe,
}
struct Tokenizer{
    
}