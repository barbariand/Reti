use tokio::sync::mpsc::Receiver;

use crate::{lexer::Token, token_reader::TokenReader};

pub struct Parser {
    reader: TokenReader,
}

impl Parser {
    pub fn new(tokens: Receiver<Token>) -> Self {
        Parser { reader: TokenReader::new(tokens) }
    }
}