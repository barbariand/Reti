use super::number::Number;

#[derive(Debug, Clone)]
pub struct Variable {
    name: String,
    value: Number,
}
