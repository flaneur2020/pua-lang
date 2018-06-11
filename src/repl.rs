use lexer::Lexer;
use std::io::{Stdin, Stdout, Write};
use token::TokenType;

pub fn start(stdin: Stdin, stdout: Stdout) {
    loop {
        let mut out = stdout.lock();

        out.write(b">> ").unwrap();
        out.flush().unwrap();

        let mut line = String::new();

        stdin.read_line(&mut line).expect("Failed to read line");

        let mut l = Lexer::new(&line.trim());

        loop {
            let tok = l.next_token();

            match tok.token_type {
                TokenType::Eof => {
                    break;
                }
                _ => {
                    out.write(format!("{:?}\n", tok).as_bytes()).unwrap();
                }
            }
        }

        out.write(b"\n").unwrap();
        out.flush().unwrap();
    }
}
