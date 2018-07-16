extern crate rustyline;

mod ast;
mod token;
mod lexer;
mod parser;
mod evaluator;
mod repl;

fn main() {
    repl::start();
}
