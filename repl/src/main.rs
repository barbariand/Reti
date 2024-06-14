use std::{ops::ControlFlow, sync::Arc};

use clap::{command, Parser as ClapParser};
use colored::Colorize;
use directories::ProjectDirs;
use parser::{
    ast::{simplify::Simplify, Factor, MathExpr, MathIdentifier, Term},
    prelude::*,
};
use rustyline::{
    error::ReadlineError, history::FileHistory, DefaultEditor, Editor,
};
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
    simple_ast_mode:bool,
    approximator: Approximator,
    rl: Editor<(), FileHistory>,
}
impl Repl {
    fn new(ast_start: bool) -> Repl {
        let context = MathContext::standard_math();
        Repl {
            simple_ast_mode:false,
            ast_mode: ast_start,
            approximator: Approximator::new(context),
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
                let ast = self.parse(&line).await.map_err(|e| {
                    error!("{}", e);
                    ControlFlow::Continue(())
                })?;

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

                let s = self.eval(ast).map_err(|e| {
                    error!("could not evaluate {:?}", e);
                    ControlFlow::Continue(())
                })?;
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
            println!();
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
        if lowercase=="simple"{
            self.simple_ast_mode=!self.simple_ast_mode;
            match self.simple_ast_mode{
                true => info!("Simple ast mode enabled"),
                false => info!("Simple ast mode disabled"),
            }
            return Err(ControlFlow::Continue(()));
        }
        Ok(())
    }
    async fn parse(&mut self, line: &str) -> Result<Ast, AstError> {
        parse(line, self.approximator.context()).await
    }
    fn eval(&mut self, ast: Ast) -> Result<String, EvalError> {
        if self.ast_mode {
            println!("{:#?}", ast); //TODO fix some display for the tree
        };

        match ast {
            Ast::Expression(expr) => {
                let simple=expr.simple(self.approximator.context())?;
                if self.simple_ast_mode{
                    println!("{:#?}",simple)
                }
                let result = self.approximator.eval_expr(simple);
                Ok(value_res_to_string(result))
            }
            Ast::Equality(lhs, rhs) => Ok(self.ast_equality(lhs, rhs)),
        }
    }
    fn ast_equality(&mut self, lhs: MathExpr, rhs: MathExpr) -> String {
        if let MathExpr::Term(Term::Multiply(
            parser::ast::MulType::Implicit,
            var,
            Factor::Parenthesis(possible_args),
        )) = lhs
        {
            if let (
                Term::Factor(Factor::Variable(var)),
                MathExpr::Term(Term::Factor(Factor::Variable(args))),
            ) = (&*var, &*possible_args)
            {
                let variable_name = args.clone();
                // FIXME: this is not perfect but i think we might need to live
                // with it otherwise we just don't know what it
                // is
                self.approximator.context_mut().functions.insert(
                    var.clone(),
                    MathFunction::new_foreign(rhs,vec![variable_name]),
                );
                "added function:".to_owned()
            } else {
                todo!("Could not understand equals.");
            }
        } else if let MathExpr::Term(Term::Factor(Factor::Variable(ident))) =
            lhs
        {
           
                    self.approximator
                        .context_mut()
                        .variables
                        .insert(ident, rhs);
                    "added variable".to_owned()
           
        } else {
            todo!("Could not understand equals.");
        }
    }
}

fn value_res_to_string(result: Result<Value, EvalError>) -> String {
    match result {
        Ok(v) => format!("> {}", v),
        Err(e) => {
            error!("Could not evaluate {:?}", e);
            format!("Could not evaluate {:?}", e)
        }
    }
}

