extern crate pua_lang;
extern crate rustyline;

use pua_lang::evaluator::builtins::new_builtins;
use pua_lang::evaluator::env::Env;
use pua_lang::evaluator::Evaluator;
use pua_lang::lexer::Lexer;
use pua_lang::parser::Parser;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let mut rl = Editor::<()>::new();
    let env = Env::from(new_builtins());
    let mut evaluator = Evaluator::new(Rc::new(RefCell::new(env)));

    println!("Hello! This is the PUA programming language!");
    println!("Feel free to type in commands\n");

    loop {
        match rl.readline(">> ") {
            Ok(line) => {
                rl.add_history_entry(&line);

                let mut parser = Parser::new(Lexer::new(&line));
                let program = parser.parse();
                let errors = parser.get_errors();

                if errors.len() > 0 {
                    for err in errors {
                        println!("{}", err);
                    }
                    continue;
                }

                if let Some(evaluated) = evaluator.eval(program) {
                    println!("{}\n", evaluated);
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("\nBye :)");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }
    }
}
