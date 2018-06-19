use ast::*;
use lexer::Lexer;
use token::Token;

#[derive(Debug, Clone)]
pub enum ParseErrorKind {
    UnexpectedToken,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    kind: ParseErrorKind,
    msg: String,
}

impl ParseError {
    fn new(kind: ParseErrorKind, msg: String) -> Self {
        ParseError { kind, msg }
    }
}

pub type ParseErrors = Vec<ParseError>;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
    next_token: Token,
    errors: ParseErrors,
}

impl<'a> Parser<'a> {
    fn new(lexer: Lexer<'a>) -> Self {
        let mut parser = Parser {
            lexer,
            current_token: Token::Eof,
            next_token: Token::Eof,
            errors: vec![],
        };

        parser.bump();
        parser.bump();

        parser
    }

    pub fn get_errors(&mut self) -> ParseErrors {
        self.errors.clone()
    }

    fn bump(&mut self) {
        self.current_token = self.next_token.clone();
        self.next_token = self.lexer.next_token();
    }

    fn current_token_is(&mut self, tok: Token) -> bool {
        self.current_token == tok
    }

    fn next_token_is(&mut self, tok: Token) -> bool {
        self.next_token == tok
    }

    fn expect_next_token(&mut self, tok: Token) -> bool {
        if self.next_token_is(tok.clone()) {
            self.bump();
            return true;
        } else {
            self.error_next_token(tok);
            return false;
        }
    }

    fn error_next_token(&mut self, tok: Token) {
        self.errors.push(ParseError::new(
            ParseErrorKind::UnexpectedToken,
            format!(
                "expected next token to be {:?}, got {:?} instead",
                tok, self.next_token
            ),
        ));
    }

    pub fn parse(&mut self) -> Program {
        let mut program: Program = vec![];

        while !self.current_token_is(Token::Eof) {
            match self.parse_stmt() {
                Some(stmt) => program.push(stmt),
                None => {
                    self.bump();
                }
            }
        }

        program
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        match self.current_token {
            Token::Let => self.parse_let_stmt(),
            Token::Return => self.parse_return_stmt(),
            _ => None,
        }
    }

    fn parse_let_stmt(&mut self) -> Option<Stmt> {
        let ident = match &self.next_token {
            Token::Ident(s) => s.clone(),
            _ => return None,
        };

        self.bump();

        if !self.expect_next_token(Token::Assign) {
            return None;
        }

        // TODO We're skipping the expressions until we encounter a semicolon.
        while !self.current_token_is(Token::Semicolon) {
            self.bump();
        }

        Some(Stmt::Let(
            Ident(ident),
            Expr::Ident(Ident(String::new())), // TODO
        ))
    }

    fn parse_return_stmt(&mut self) -> Option<Stmt> {
        let stmt = Stmt::Return(
            Expr::Ident(Ident(String::new())),
        );

        self.bump();

        // TODO We're skipping the expressions until we encounter a semicolon.
        while !self.current_token_is(Token::Semicolon) {
            self.bump();
        }

        Some(stmt)
    }
}

#[cfg(test)]
mod tests {
    use ast::*;
    use lexer::Lexer;
    use parser::Parser;

    fn check_parse_errors(parser: &mut Parser) {
        let errors = parser.get_errors();

        if errors.len() == 0 {
            return;
        }

        println!("\n");

        println!("parser has {} errors", errors.len());

        for err in errors {
            println!("parse error: {:?}", err);
        }

        println!("\n");

        panic!("failed");
    }

    #[test]
    fn test_let_stmt() {
        let input = r#"
let x = 5;
let y = 10;
let foobar = 838383;
        "#;

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse();

        check_parse_errors(&mut parser);

        let tests = vec![
            Stmt::Let(
                Ident(String::from("x")),
                Expr::Ident(Ident(String::new())), // TODO
            ),
            Stmt::Let(
                Ident(String::from("y")),
                Expr::Ident(Ident(String::new())), // TODO
            ),
            Stmt::Let(
                Ident(String::from("foobar")),
                Expr::Ident(Ident(String::new())), // TODO
            ),
        ];

        assert_eq!(tests.len(), program.len());

        for (i, expect) in tests.into_iter().enumerate() {
            assert_eq!(expect, program[i]);
        }
    }

    #[test]
    fn test_return_stmt() {
        let input = r#"
return 5;
return 10;
return 993322;
        "#;

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse();

        check_parse_errors(&mut parser);

        let tests = vec![
            Stmt::Return(
                Expr::Ident(Ident(String::from(""))), // TODO
            ),
            Stmt::Return(
                Expr::Ident(Ident(String::from(""))), // TODO
            ),
            Stmt::Return(
                Expr::Ident(Ident(String::from(""))), // TODO
            ),
        ];

        assert_eq!(tests.len(), program.len());

        for (i, expect) in tests.into_iter().enumerate() {
            assert_eq!(expect, program[i]);
        }
    }
}
