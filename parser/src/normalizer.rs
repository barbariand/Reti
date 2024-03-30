use std::mem::replace;

use tokio::sync:: mpsc::{Receiver, Sender};

use crate::{token::Token, token_reader::TokenReader};

struct Normalizer{
    input:TokenReader,
    output:Sender<Token>,
}
impl Normalizer{
    fn new(input:Receiver<Token>,output:Sender<Token>,)->Self{
        Self{
            input:TokenReader::new(input),
            output,
        }
    }
    async fn normalize(&mut self){
        let mut previous=self.input.read().await;
        if previous==Token::EOF{
                self.send_or_crash(previous).await;
                return;
            }
        loop {
            let next=self.input.read().await;
            if next==Token::EOF{
                self.send_or_crash(previous).await;
                self.send_or_crash(Token::EOF).await;
                return;
            }
            match previous{
                Token::Backslash=>{},
                Token::EOF => unreachable!(),
                _=>todo!()
            }
            self.send_or_crash(replace(&mut previous,next)).await;
        }
    }
    async fn send_or_crash(&self, token: Token) {
        self.output.send(token).await.expect("Broken Pipe")
    }
}
