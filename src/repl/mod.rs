use std::io::{Stdin, Stdout, Write};
use token::Token;
use lexer::Lexer;
use parser::Parser;
use evaluator::{Evaluator};

pub fn start(stdin: Stdin, stdout: Stdout) {
    let mut evaluator = Evaluator::new();

    loop {
        let mut out = stdout.lock();

        out.write(b">> ").unwrap();
        out.flush().unwrap();

        let mut line = String::new();

        stdin.read_line(&mut line).expect("Failed to read line");

        let mut parser = Parser::new(Lexer::new(&line));
        let program = parser.parse();
        let errors = parser.get_errors();

        if errors.len() > 0 {
            for err in errors {
                out.write(format!("{}", err).as_bytes());
            }
            out.flush();
            continue;
        }

        let evaluated = evaluator.eval(program);

        out.write(format!("{}", evaluated).as_bytes());

        out.write(b"\n").unwrap();
        out.flush().unwrap();
    }
}
