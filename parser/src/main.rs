use parser::{
    approximator::Approximator,
    ast::{Factor, MathExpr, Term},
    context::MathContext,
    lexer::Lexer,
    normalizer::Normalizer,
    parser::Parser,
    token::Token,
};
use tokio::{
    join,
    sync::mpsc::{self, Receiver, Sender},
};

#[tokio::main]
pub async fn main() {
    Prompt::new().start().await;
}

struct Prompt {
    ast_mode: bool,
}

impl Prompt {
    pub fn new() -> Self {
        Prompt { ast_mode: false }
    }

    async fn start(&mut self) {
        println!("Welcome to the Reti prompt.");
        println!("Type 1+1 and press Enter to get started.");
        println!();

        let context = MathContext::standard_math();
        let mut approximator = Approximator::new(context);

        let stdin = std::io::stdin();
        let mut buf = String::new();
        loop {
            buf.clear();
            stdin
                .read_line(&mut buf)
                .expect("Failed to read from stdin.");

            let trimmed = buf.trim();
            if trimmed.len() == 0 {
                println!();
                continue;
            }

            let lowercase = trimmed.to_ascii_lowercase();
            if lowercase == "ast" {
                self.ast_mode = !self.ast_mode;
                match self.ast_mode {
                    true => println!("Enabled AST mode."),
                    false => println!("Disabled AST mode."),
                }
                continue;
            }

            self.run(trimmed, &mut approximator).await;
        }
    }

    async fn run(&self, text: &str, approximator: &mut Approximator) {
        let (lexer_in, lexer_out): (Sender<Token>, Receiver<Token>) = mpsc::channel(32);
        let (normalizer_in, normalizer_out): (Sender<Token>, Receiver<Token>) = mpsc::channel(32);

        let context = approximator.context();
        let lexer = Lexer::new(lexer_in);
        let mut normalizer = Normalizer::new(lexer_out, normalizer_in);
        let mut parser = Parser::new(normalizer_out, context);

        let future1 = lexer.tokenize(text);
        let future2 = normalizer.normalize();
        let future3 = parser.parse();

        let (_, _, ast) = join!(future1, future2, future3);
        let ast = match ast {
            Ok(ast) => ast,
            Err(err) => {
                println!("Failed to parse:");
                println!("{:?}", err); // TODO impl Display
                println!();
                return;
            }
        };

        if self.ast_mode {
            println!("{:#?}", ast);
        }

        match ast {
            parser::ast::Ast::Expression(expr) => {
                let result = approximator.eval_expr(&expr);
                println!("> {}", result);
            }
            parser::ast::Ast::Equality(lhs, rhs) => match lhs {
                MathExpr::Term(Term::Factor(Factor::Variable(ident))) => {
                    let value = approximator.eval_expr(&rhs);
                    let context = approximator.context_mut();
                    context.variables.insert(ident, value);
                    println!("Variable changed!");
                }
                _ => {
                    println!("I don't understand. Please type an expression, like 1+1, or x=2 for assignment.");
                }
            },
        }

        println!();
    }
}
