mod ast;
mod lexer;
mod parser;
mod repl;
mod token;

use std::io;

fn main() {
    println!("Hello! This is the Monkey programming language!");
    println!("Feel free to type in commands");
    println!("");
    repl::start(io::stdin(), io::stdout());
}
