mod approximator;
mod ast;
mod context;
mod lexer;

use std::{collections::HashMap, sync::Arc};

use prelude::*;
mod normalizer;
mod parsing;
mod token;
mod token_reader;
use clap::{command, Parser as ClapParser};
use colored::Colorize;
pub mod prelude;

use rustyline::{error::ReadlineError, DefaultEditor};
use tracing::{debug, error, info, level_filters::LevelFilter, trace_span};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::context::MathFunction;
#[tokio::main]
pub async fn main() {
    let mut map = HashMap::new();
    map.insert("hello", "hello");
    map.insert("hello", "world");
    println!("hello {:?}", map.get("hello"));
    let mut prompt = Prompt::parse();
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(prompt.tracing_level.into())
                .from_env_lossy(),
        )
        .init();
    prompt.start().await;
}
#[derive(ClapParser, Debug)]
#[command(version, about, long_about = None)]
struct Prompt {
    #[arg(short, long, default_value_t = false)]
    ast_mode: bool,
    #[arg(default_value_t=LevelFilter::WARN)]
    tracing_level: LevelFilter,
}

impl Prompt {
    async fn start(&mut self) {
        let mut rl = DefaultEditor::new().expect("Could not make terminal");
        if rl.load_history("~/history.txt").is_err() {
            info!("No previous history.");
        }
        println!("{}", "Welcome to the Reti prompt.".green());
        println!("{}", "Type 1+1 and press Enter to get started.\n".green());
        let context = MathContext::new();
        let mut approximator = Approximator::new(context);
        loop {
            let readline = rl.readline(&format!("{}", ">> ".white()));
            match readline {
                Ok(line) => {
                    let span = trace_span!("preparing statement");
                    let _enter = span.enter();
                    debug!(line);
                    debug!("adding expression to history");
                    rl.add_history_entry(line.as_str())
                        .expect("could not add history");

                    debug!("trimming expression");
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        println!();
                        continue;
                    }

                    let lowercase = trimmed.to_ascii_lowercase();
                    if lowercase == "ast" {
                        self.ast_mode = !self.ast_mode;
                        match self.ast_mode {
                            true => info!("AST mode enabled"),
                            false => info!("AST mode disabled"),
                        }
                        continue;
                    }

                    drop(_enter);
                    match parse(&line, approximator.context()).await {
                        Ok(ast) => {
                            if self.ast_mode {
                                println!("{:#?}", ast); //TODO fix some display for the tree
                            };

                            match ast {
                                Ast::Expression(expr) => {
                                    let result = approximator.eval_expr(&expr);
                                    println!("> {}", result);
                                }
                                Ast::Equality(lhs, rhs) => {
                                    if let MathExpr::Term(Term::Multiply(
                                        var,
                                        Factor::Parenthesis(possible_args),
                                    )) = lhs
                                    {
                                        if let (
                                            Term::Factor(Factor::Variable(var)),
                                            MathExpr::Term(Term::Factor(Factor::Variable(args))),
                                        ) = (&*var, &*possible_args)
                                        {
                                            let coppied = args.clone();
                                            // TODO: this is not perfect but i think we might need to live with it otherwise we just dont know what it is
                                            approximator.context_mut().functions.insert(
                                                var.clone(),
                                                MathFunction::new(Arc::new(
                                                    move |func: Vec<f64>, outer_context| {
                                                        let mut context = outer_context.clone();
                                                        context
                                                            .variables
                                                            .insert(coppied.clone(), func[0]);
                                                        let aprox = Approximator::new(context);

                                                        aprox.eval_expr(&rhs)
                                                    },
                                                )),
                                            );
                                        } else {
                                            todo!("assign variables to MathContext.");
                                        }
                                    } else if let MathExpr::Term(Term::Factor(Factor::Variable(
                                        ident,
                                    ))) = lhs
                                    {
                                        let res = approximator.eval_expr(&rhs);
                                        approximator.context_mut().variables.insert(ident, res);
                                    } else {
                                        todo!("assign variables to MathContext.");
                                    }
                                }
                            }
                        }
                        Err(e) => error!("{}", format!("{:?}", e).red()),
                    };
                }
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    error!("{:?}", err);
                    break;
                }
            }
        }
    }
}
