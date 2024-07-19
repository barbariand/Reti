use std::ops::ControlFlow;

use clap::{command, Parser as ClapParser};
use colored::Colorize;
use directories::ProjectDirs;
use parser::{
    ast::{simplify::Simplify, Factor, MathExpr, Term},
    identifier::MathIdentifier,
    prelude::*,
};
use rustyline::{
    error::ReadlineError, history::FileHistory, DefaultEditor, Editor,
};
use tokio::time::Instant;
use tracing::{debug, error, info, trace_span};
use tracing_subscriber::filter::LevelFilter;

use parser::functions::MathFunction;
#[tokio::main]
pub async fn main() {
    let project_dirs = ProjectDirs::from("", "", "Reti");
    let prompt = Prompt::parse();
    let _guard = utils::logging::init_logger(
        project_dirs,
        prompt.tracing_level,
        "reti-repl",
    );
    prompt.into_repl().start().await;
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
    fn into_repl(self) -> Repl {
        Repl::new(self.ast_mode)
    }
}
struct Repl {
    ast_mode: bool,
    simple_ast_mode: bool,
    evaluator: Evaluator,
    rl: Editor<(), FileHistory>,
    time_it: bool,
}
impl Repl {
    fn new(ast_start: bool) -> Repl {
        Repl {
            simple_ast_mode: false,
            time_it: false,
            ast_mode: ast_start,
            evaluator: Evaluator::standard_math(),
            rl: DefaultEditor::new().expect("could not use as a terminal"), /* TODO manage this
                                                                             * so we just accept
                                                                             * stdin instead */
        }
    }
    async fn start(&mut self) {
        if self.rl.load_history("~/history.txt").is_err() {
            info!("No previous history.");
        }
        println!("{}", "Welcome to the Reti prompt.".green());
        println!("{}", "Type 1+1 and press Enter to get started.\n".green());
        loop {
            match self.prompt().await {
                Ok(()) => continue,
                Err(ControlFlow::Continue(())) => continue,
                Err(ControlFlow::Break(())) => break,
            }
        }
    }
    async fn prompt(&mut self) -> Result<(), ControlFlow<()>> {
        let readline = self.rl.readline(">> ");
        match readline {
            Ok(line) => {
                self.read(&line).await?;
                let start_parse = Instant::now();
                let ast = self.parse(&line).await.map_err(|e| {
                    error!("{}", e);
                    ControlFlow::Continue(())
                })?;
                let time_parse = start_parse.elapsed();

                /*
                use parser::ast::to_latex::ToLaTeX;
                let deriv =
                    ast.derivative(&MathIdentifier::from_single_ident("x"));

                match deriv {
                    Ok(val) => {
                        println!("before simplify: {}", val.to_latex());
                        println!("\n{}", val.simplify().to_latex());
                    }
                    Err(err) => {
                        println!("{:?}", err);
                    }
                }
                if true {
                    return Ok(());
                };
                */
                let start_eval = Instant::now();
                let s = self.eval(ast).map_err(|e| {
                    error!("could not evaluate ast {:?}", e);
                    ControlFlow::Continue(())
                })?;
                let time_eval = start_eval.elapsed();
                if self.time_it {
                    println!("Parsing took:{}ns", time_parse.as_nanos());
                    println!("Evaluating took:{}ns", time_eval.as_nanos());
                }
                println!("{}", s);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                return Err(ControlFlow::Break(()));
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                return Err(ControlFlow::Break(()));
            }
            Err(err) => {
                error!("{:?}", err);
                return Err(ControlFlow::Break(()));
            }
        }
        Ok(())
    }
    async fn read(&mut self, line: &String) -> Result<(), ControlFlow<()>> {
        let span = trace_span!("preparing statement");
        let _enter = span.enter();
        debug!(line);
        debug!("adding expression to history");
        self.rl
            .add_history_entry(line.as_str())
            .expect("could not add history");

        debug!("trimming expression");
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Err(ControlFlow::Continue(()));
        }

        let lowercase = trimmed.to_ascii_lowercase();
        if lowercase == "ast" {
            self.ast_mode = !self.ast_mode;
            match self.ast_mode {
                true => info!("AST mode enabled"),
                false => info!("AST mode disabled"),
            }
            return Err(ControlFlow::Continue(()));
        }
        if lowercase == "simple" {
            self.simple_ast_mode = !self.simple_ast_mode;
            match self.simple_ast_mode {
                true => info!("Simple ast mode enabled"),
                false => info!("Simple ast mode disabled"),
            }
            return Err(ControlFlow::Continue(()));
        }
        if lowercase == "time" {
            self.time_it = !self.time_it;
            match self.time_it {
                true => info!("Timing mode enabled"),
                false => info!("Timing mode disabled"),
            }
            return Err(ControlFlow::Continue(()));
        }
        Ok(())
    }
    async fn parse(&mut self, line: &str) -> Result<Ast, AstError> {
        parse(line, self.evaluator.context()).await
    }
    fn eval(&mut self, ast: Ast) -> Result<String, EvalError> {
        if self.ast_mode {
            println!("{:#?}", ast); //TODO fix some display for the tree
        };
        let simple_ast = ast.simple(self.evaluator.context())?;
        if self.simple_ast_mode {
            println!("{:#?}", simple_ast)
        }
        self.evaluator
            .eval_ast(simple_ast)
            .map(|v| format!("> {}", v))
    }
}
