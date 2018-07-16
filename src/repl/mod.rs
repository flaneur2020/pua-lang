use evaluator::Evaluator;
use lexer::Lexer;
use parser::Parser;
use rustyline::error::ReadlineError;
use rustyline::Editor;

pub fn start() {
    let mut rl = Editor::<()>::new();
    let mut evaluator = Evaluator::new();

    println!("Hello! This is the Monkey programming language!");
    println!("Feel free to type in commands");
    println!("");

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
