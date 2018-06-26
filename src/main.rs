mod ast;
mod token;
mod lexer;
mod parser;
mod evaluator;
mod repl;

use std::io;

fn main() {
    println!("Hello! This is the Monkey programming language!");
    println!("Feel free to type in commands");
    println!("");
    repl::start(io::stdin(), io::stdout());
}
