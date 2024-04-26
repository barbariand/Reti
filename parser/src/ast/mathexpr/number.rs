#[derive(Debug, Clone)]
pub enum Number {
    Float(f64),
    Int(i64),
    Constants(Constants),
}
#[derive(Debug, Clone)]
pub enum Constants {
    Infinity,
    PI,
    E,
    Tau,
}
