use clap::{command, Parser as ClapParser};
use colored::Colorize;

use rustyline::{error::ReadlineError, DefaultEditor};

use tracing::level_filters::LevelFilter;
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
use parser::parse;
impl Prompt {
    async fn start(&mut self) {
        let mut rl = DefaultEditor::new().expect("Could not make terminal");
        if rl.load_history("~/history.txt").is_err() {
            println!("No previous history.");
        }
        println!("{}", "Welcome to the Reti prompt.".green());
        println!("{}", "Type 1+1 and press Enter to get started.".green());
        println!();

        loop {
            let readline = rl.readline(&format!("{}", ">> ".white()));
            match readline {
                Ok(line) => {
                    rl.add_history_entry(line.as_str())
                        .expect("could not add history");

                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        println!();
                        continue;
                    }

                    let lowercase = trimmed.to_ascii_lowercase();
                    if lowercase == "ast" {
                        self.ast_mode = !self.ast_mode;
                        match self.ast_mode {
                            true => println!("{}", "Enabled AST mode.".green()),
                            false => println!("{}", "Disabled AST mode.".green()),
                        }
                        continue;
                    }
                    match parse(&line).await {
                        Ok(v) => {
                            if self.ast_mode {
                                println!("{:#?}", v); //TODO fix some display for the tree
                            };
                            println!("> {}", v.eval())
                        }
                        Err(e) => println!("Got an error during parsing: {}", e.to_string().red()),
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
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
    }
}
