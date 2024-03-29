use std::collections::HashSet;

use super::{MathExpr, MathExprKey};

struct State {
    current_work: Vec<MathExpr>,
    finished_work:HashMap<usize,Value>
}
