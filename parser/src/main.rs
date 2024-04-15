use clap::{command, Parser as ClapParser};
use colored::Colorize;
use rustyline::{error::ReadlineError, DefaultEditor};
use tracing::{debug, error, info, level_filters::LevelFilter, trace_span};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
#[tokio::main]
pub async fn main() {
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
mod approximator;
use approximator::Approximator;
mod context;
use context::MathContext;
use parser::parse;
impl Prompt {
    async fn start(&mut self) {
        let mut rl = DefaultEditor::new().expect("Could not make terminal");
        if rl.load_history("~/history.txt").is_err() {
            info!("No previous history.");
        }
        println!("{}", "Welcome to the Reti prompt.".green());
        println!("{}", "Type 1+1 and press Enter to get started.\n".green());
        let context = MathContext::new();
        let approximator = Approximator::new(context);
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
                    match parse(&line).await {
                        Ok(ast) => {
                            if self.ast_mode {
                                println!("{:#?}", ast); //TODO fix some display for the tree
                            };

                            println!("> {}", approximator.eval_expr(expr));
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
