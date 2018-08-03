use ast::*;
use lexer::Lexer;
use std::fmt;
use token::Token;

#[derive(Debug, Clone)]
pub enum ParseErrorKind {
    UnexpectedToken,
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseErrorKind::UnexpectedToken => write!(f, "Unexpected Token"),
        }
    }
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

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.msg)
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
    pub fn new(lexer: Lexer<'a>) -> Self {
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

    fn token_to_precedence(tok: &Token) -> Precedence {
        match tok {
            Token::Equal | Token::NotEqual => Precedence::Equals,
            Token::LessThan | Token::LessThanEqual => Precedence::LessGreater,
            Token::GreaterThan | Token::GreaterThanEqual => Precedence::LessGreater,
            Token::Plus | Token::Minus => Precedence::Sum,
            Token::Slash | Token::Asterisk => Precedence::Product,
            Token::Lbracket => Precedence::Index,
            Token::Lparen => Precedence::Call,
            _ => Precedence::Lowest,
        }
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

    fn next_token_is(&mut self, tok: &Token) -> bool {
        self.next_token == *tok
    }

    fn expect_next_token(&mut self, tok: Token) -> bool {
        if self.next_token_is(&tok) {
            self.bump();
            return true;
        } else {
            self.error_next_token(tok);
            return false;
        }
    }

    fn current_token_precedence(&mut self) -> Precedence {
        Self::token_to_precedence(&self.current_token)
    }

    fn next_token_precedence(&mut self) -> Precedence {
        Self::token_to_precedence(&self.next_token)
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

    fn error_no_prefix_parser(&mut self) {
        self.errors.push(ParseError::new(
            ParseErrorKind::UnexpectedToken,
            format!(
                "no prefix parse function for  \"{:?}\" found",
                self.current_token,
            ),
        ));
    }

    pub fn parse(&mut self) -> Program {
        let mut program: Program = vec![];

        while !self.current_token_is(Token::Eof) {
            match self.parse_stmt() {
                Some(stmt) => program.push(stmt),
                None => {}
            }
            self.bump();
        }

        program
    }

    fn parse_block_stmt(&mut self) -> BlockStmt {
        self.bump();

        let mut block = vec![];

        while !self.current_token_is(Token::Rbrace) && !self.current_token_is(Token::Eof) {
            match self.parse_stmt() {
                Some(stmt) => block.push(stmt),
                None => {}
            }
            self.bump();
        }

        block
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        match self.current_token {
            Token::Let => self.parse_let_stmt(),
            Token::Return => self.parse_return_stmt(),
            Token::Blank => Some(Stmt::Blank),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_let_stmt(&mut self) -> Option<Stmt> {
        match &self.next_token {
            Token::Ident(_) => self.bump(),
            _ => return None,
        };

        let name = match self.parse_ident() {
            Some(name) => name,
            None => return None,
        };

        if !self.expect_next_token(Token::Assign) {
            return None;
        }

        self.bump();

        let expr = match self.parse_expr(Precedence::Lowest) {
            Some(expr) => expr,
            None => return None,
        };

        if self.next_token_is(&Token::Semicolon) {
            self.bump();
        }

        Some(Stmt::Let(name, expr))
    }

    fn parse_return_stmt(&mut self) -> Option<Stmt> {
        self.bump();

        let expr = match self.parse_expr(Precedence::Lowest) {
            Some(expr) => expr,
            None => return None,
        };

        if self.next_token_is(&Token::Semicolon) {
            self.bump();
        }

        Some(Stmt::Return(expr))
    }

    fn parse_expr_stmt(&mut self) -> Option<Stmt> {
        match self.parse_expr(Precedence::Lowest) {
            Some(expr) => {
                if self.next_token_is(&Token::Semicolon) {
                    self.bump();
                }
                Some(Stmt::Expr(expr))
            }
            None => None,
        }
    }

    fn parse_expr(&mut self, precedence: Precedence) -> Option<Expr> {
        // prefix
        let mut left = match self.current_token {
            Token::Ident(_) => self.parse_ident_expr(),
            Token::Int(_) => self.parse_int_expr(),
            Token::String(_) => self.parse_string_expr(),
            Token::Bool(_) => self.parse_bool_expr(),
            Token::Lbracket => self.parse_array_expr(),
            Token::Lbrace => self.parse_hash_expr(),
            Token::Bang | Token::Minus | Token::Plus => self.parse_prefix_expr(),
            Token::Lparen => self.parse_grouped_expr(),
            Token::If => self.parse_if_expr(),
            Token::Func => self.parse_func_expr(),
            _ => {
                self.error_no_prefix_parser();
                return None;
            }
        };

        // infix
        while !self.next_token_is(&Token::Semicolon) && precedence < self.next_token_precedence() {
            match self.next_token {
                Token::Plus
                | Token::Minus
                | Token::Slash
                | Token::Asterisk
                | Token::Equal
                | Token::NotEqual
                | Token::LessThan
                | Token::LessThanEqual
                | Token::GreaterThan
                | Token::GreaterThanEqual => {
                    self.bump();
                    left = self.parse_infix_expr(left.unwrap());
                }
                Token::Lbracket => {
                    self.bump();
                    left = self.parse_index_expr(left.unwrap());
                }
                Token::Lparen => {
                    self.bump();
                    left = self.parse_call_expr(left.unwrap());
                }
                _ => return left,
            }
        }

        left
    }

    fn parse_ident(&mut self) -> Option<Ident> {
        match self.current_token {
            Token::Ident(ref mut ident) => Some(Ident(ident.clone())),
            _ => None,
        }
    }

    fn parse_ident_expr(&mut self) -> Option<Expr> {
        match self.parse_ident() {
            Some(ident) => Some(Expr::Ident(ident)),
            None => None,
        }
    }

    fn parse_int_expr(&mut self) -> Option<Expr> {
        match self.current_token {
            Token::Int(ref mut int) => Some(Expr::Literal(Literal::Int(int.clone()))),
            _ => None,
        }
    }

    fn parse_string_expr(&mut self) -> Option<Expr> {
        match self.current_token {
            Token::String(ref mut s) => Some(Expr::Literal(Literal::String(s.clone()))),
            _ => None,
        }
    }

    fn parse_bool_expr(&mut self) -> Option<Expr> {
        match self.current_token {
            Token::Bool(value) => Some(Expr::Literal(Literal::Bool(value == true))),
            _ => None,
        }
    }

    fn parse_array_expr(&mut self) -> Option<Expr> {
        match self.parse_expr_list(Token::Rbracket) {
            Some(list) => Some(Expr::Literal(Literal::Array(list))),
            None => None,
        }
    }

    fn parse_hash_expr(&mut self) -> Option<Expr> {
        let mut pairs = Vec::new();

        while !self.next_token_is(&Token::Rbrace) {
            self.bump();

            let key = match self.parse_expr(Precedence::Lowest) {
                Some(expr) => expr,
                None => return None,
            };

            if !self.expect_next_token(Token::Colon) {
                return None;
            }

            self.bump();

            let value = match self.parse_expr(Precedence::Lowest) {
                Some(expr) => expr,
                None => return None,
            };

            pairs.push((key, value));

            if !self.next_token_is(&Token::Rbrace) && !self.expect_next_token(Token::Comma) {
                return None;
            }
        }

        if !self.expect_next_token(Token::Rbrace) {
            return None;
        }

        Some(Expr::Literal(Literal::Hash(pairs)))
    }

    fn parse_expr_list(&mut self, end: Token) -> Option<Vec<Expr>> {
        let mut list = vec![];

        if self.next_token_is(&end) {
            self.bump();
            return Some(list);
        }

        self.bump();

        match self.parse_expr(Precedence::Lowest) {
            Some(expr) => list.push(expr),
            None => return None,
        }

        while self.next_token_is(&Token::Comma) {
            self.bump();
            self.bump();

            match self.parse_expr(Precedence::Lowest) {
                Some(expr) => list.push(expr),
                None => return None,
            }
        }

        if !self.expect_next_token(end) {
            return None;
        }

        Some(list)
    }

    fn parse_prefix_expr(&mut self) -> Option<Expr> {
        let prefix = match self.current_token {
            Token::Bang => Prefix::Not,
            Token::Minus => Prefix::Minus,
            Token::Plus => Prefix::Plus,
            _ => return None,
        };

        self.bump();

        match self.parse_expr(Precedence::Prefix) {
            Some(expr) => Some(Expr::Prefix(prefix, Box::new(expr))),
            None => None,
        }
    }

    fn parse_infix_expr(&mut self, left: Expr) -> Option<Expr> {
        let infix = match self.current_token {
            Token::Plus => Infix::Plus,
            Token::Minus => Infix::Minus,
            Token::Slash => Infix::Divide,
            Token::Asterisk => Infix::Multiply,
            Token::Equal => Infix::Equal,
            Token::NotEqual => Infix::NotEqual,
            Token::LessThan => Infix::LessThan,
            Token::LessThanEqual => Infix::LessThanEqual,
            Token::GreaterThan => Infix::GreaterThan,
            Token::GreaterThanEqual => Infix::GreaterThanEqual,
            _ => return None,
        };

        let precedence = self.current_token_precedence();

        self.bump();

        match self.parse_expr(precedence) {
            Some(expr) => Some(Expr::Infix(infix, Box::new(left), Box::new(expr))),
            None => None,
        }
    }

    fn parse_index_expr(&mut self, left: Expr) -> Option<Expr> {
        self.bump();

        let index = match self.parse_expr(Precedence::Lowest) {
            Some(expr) => expr,
            None => return None,
        };

        if !self.expect_next_token(Token::Rbracket) {
            return None;
        }

        Some(Expr::Index(Box::new(left), Box::new(index)))
    }

    fn parse_grouped_expr(&mut self) -> Option<Expr> {
        self.bump();

        let expr = self.parse_expr(Precedence::Lowest);

        if !self.expect_next_token(Token::Rparen) {
            None
        } else {
            expr
        }
    }

    fn parse_if_expr(&mut self) -> Option<Expr> {
        if !self.expect_next_token(Token::Lparen) {
            return None;
        }

        self.bump();

        let cond = match self.parse_expr(Precedence::Lowest) {
            Some(expr) => expr,
            None => return None,
        };

        if !self.expect_next_token(Token::Rparen) || !self.expect_next_token(Token::Lbrace) {
            return None;
        }

        let consequence = self.parse_block_stmt();
        let mut alternative = None;

        if self.next_token_is(&Token::Else) {
            self.bump();

            if !self.expect_next_token(Token::Lbrace) {
                return None;
            }

            alternative = Some(self.parse_block_stmt());
        }

        Some(Expr::If {
            cond: Box::new(cond),
            consequence,
            alternative,
        })
    }

    fn parse_func_expr(&mut self) -> Option<Expr> {
        if !self.expect_next_token(Token::Lparen) {
            return None;
        }

        let params = match self.parse_func_params() {
            Some(params) => params,
            None => return None,
        };

        if !self.expect_next_token(Token::Lbrace) {
            return None;
        }

        Some(Expr::Func {
            params,
            body: self.parse_block_stmt(),
        })
    }

    fn parse_func_params(&mut self) -> Option<Vec<Ident>> {
        let mut params = vec![];

        if self.next_token_is(&Token::Rparen) {
            self.bump();
            return Some(params);
        }

        self.bump();

        match self.parse_ident() {
            Some(ident) => params.push(ident),
            None => return None,
        };

        while self.next_token_is(&Token::Comma) {
            self.bump();
            self.bump();

            match self.parse_ident() {
                Some(ident) => params.push(ident),
                None => return None,
            };
        }

        if !self.expect_next_token(Token::Rparen) {
            return None;
        }

        Some(params)
    }

    fn parse_call_expr(&mut self, func: Expr) -> Option<Expr> {
        let args = match self.parse_expr_list(Token::Rparen) {
            Some(args) => args,
            None => return None,
        };

        Some(Expr::Call {
            func: Box::new(func),
            args,
        })
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
    fn test_blank() {
        let input = r#"
1000;

1000;


1000;

if (x) {

    x;

}
        "#;

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(
            vec![
                Stmt::Expr(Expr::Literal(Literal::Int(1000))),
                Stmt::Blank,
                Stmt::Expr(Expr::Literal(Literal::Int(1000))),
                Stmt::Blank,
                Stmt::Blank,
                Stmt::Expr(Expr::Literal(Literal::Int(1000))),
                Stmt::Blank,
                Stmt::Expr(Expr::If {
                    cond: Box::new(Expr::Ident(Ident(String::from("x")))),
                    consequence: vec![
                        Stmt::Blank,
                        Stmt::Expr(Expr::Ident(Ident(String::from("x")))),
                        Stmt::Blank,
                    ],
                    alternative: None,
                }),
            ],
            program,
        );
    }

    #[test]
    fn test_let_stmt() {
        let input = r#"
let x = 5;
let y = 10;
let foobar = 838383;
        "#;

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(
            vec![
                Stmt::Let(Ident(String::from("x")), Expr::Literal(Literal::Int(5))),
                Stmt::Let(Ident(String::from("y")), Expr::Literal(Literal::Int(10))),
                Stmt::Let(
                    Ident(String::from("foobar")),
                    Expr::Literal(Literal::Int(838383)),
                ),
            ],
            program,
        );
    }

    #[test]
    fn test_return_stmt() {
        let input = r#"
return 5;
return 10;
return 993322;
        "#;

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(
            vec![
                Stmt::Return(Expr::Literal(Literal::Int(5))),
                Stmt::Return(Expr::Literal(Literal::Int(10))),
                Stmt::Return(Expr::Literal(Literal::Int(993322))),
            ],
            program,
        );
    }

    #[test]
    fn test_ident_expr() {
        let input = "foobar;";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(
            vec![Stmt::Expr(Expr::Ident(Ident(String::from("foobar"))))],
            program,
        );
    }

    #[test]
    fn test_integer_literal_expr() {
        let input = "5;";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(vec![Stmt::Expr(Expr::Literal(Literal::Int(5)))], program,);
    }

    #[test]
    fn test_string_literal_expr() {
        let input = "\"hello world\";";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(
            vec![Stmt::Expr(Expr::Literal(Literal::String(String::from(
                "hello world",
            ))))],
            program,
        );
    }

    #[test]
    fn test_boolean_literal_expr() {
        let tests = vec![
            ("true;", Stmt::Expr(Expr::Literal(Literal::Bool(true)))),
            ("false;", Stmt::Expr(Expr::Literal(Literal::Bool(false)))),
        ];

        for (input, expect) in tests {
            let mut parser = Parser::new(Lexer::new(input));
            let program = parser.parse();

            check_parse_errors(&mut parser);
            assert_eq!(vec![expect], program);
        }
    }

    #[test]
    fn test_array_literal_expr() {
        let input = "[1, 2 * 2, 3 + 3]";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(
            vec![Stmt::Expr(Expr::Literal(Literal::Array(vec![
                Expr::Literal(Literal::Int(1)),
                Expr::Infix(
                    Infix::Multiply,
                    Box::new(Expr::Literal(Literal::Int(2))),
                    Box::new(Expr::Literal(Literal::Int(2))),
                ),
                Expr::Infix(
                    Infix::Plus,
                    Box::new(Expr::Literal(Literal::Int(3))),
                    Box::new(Expr::Literal(Literal::Int(3))),
                ),
            ])))],
            program,
        );
    }

    #[test]
    fn test_hash_literal_expr() {
        let tests = vec![
            ("{}", Stmt::Expr(Expr::Literal(Literal::Hash(vec![])))),
            (
                "{\"one\": 1, \"two\": 2, \"three\": 3}",
                Stmt::Expr(Expr::Literal(Literal::Hash(vec![
                    (
                        Expr::Literal(Literal::String(String::from("one"))),
                        Expr::Literal(Literal::Int(1)),
                    ),
                    (
                        Expr::Literal(Literal::String(String::from("two"))),
                        Expr::Literal(Literal::Int(2)),
                    ),
                    (
                        Expr::Literal(Literal::String(String::from("three"))),
                        Expr::Literal(Literal::Int(3)),
                    ),
                ]))),
            ),
            (
                "{\"one\": 0 + 1, \"two\": 10 - 8, \"three\": 15 / 5}",
                Stmt::Expr(Expr::Literal(Literal::Hash(vec![
                    (
                        Expr::Literal(Literal::String(String::from("one"))),
                        Expr::Infix(
                            Infix::Plus,
                            Box::new(Expr::Literal(Literal::Int(0))),
                            Box::new(Expr::Literal(Literal::Int(1))),
                        ),
                    ),
                    (
                        Expr::Literal(Literal::String(String::from("two"))),
                        Expr::Infix(
                            Infix::Minus,
                            Box::new(Expr::Literal(Literal::Int(10))),
                            Box::new(Expr::Literal(Literal::Int(8))),
                        ),
                    ),
                    (
                        Expr::Literal(Literal::String(String::from("three"))),
                        Expr::Infix(
                            Infix::Divide,
                            Box::new(Expr::Literal(Literal::Int(15))),
                            Box::new(Expr::Literal(Literal::Int(5))),
                        ),
                    ),
                ]))),
            ),
            (
                "{key: \"value\"}",
                Stmt::Expr(Expr::Literal(Literal::Hash(vec![(
                    Expr::Ident(Ident(String::from("key"))),
                    Expr::Literal(Literal::String(String::from("value"))),
                )]))),
            ),
        ];

        for (input, expect) in tests {
            let mut parser = Parser::new(Lexer::new(input));
            let program = parser.parse();

            check_parse_errors(&mut parser);
            assert_eq!(vec![expect], program);
        }
    }

    #[test]
    fn test_index_expr() {
        let input = "myArray[1 + 1]";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(
            vec![Stmt::Expr(Expr::Index(
                Box::new(Expr::Ident(Ident(String::from("myArray")))),
                Box::new(Expr::Infix(
                    Infix::Plus,
                    Box::new(Expr::Literal(Literal::Int(1))),
                    Box::new(Expr::Literal(Literal::Int(1))),
                )),
            ))],
            program
        );
    }

    #[test]
    fn test_prefix_expr() {
        let tests = vec![
            (
                "!5;",
                Stmt::Expr(Expr::Prefix(
                    Prefix::Not,
                    Box::new(Expr::Literal(Literal::Int(5))),
                )),
            ),
            (
                "-15;",
                Stmt::Expr(Expr::Prefix(
                    Prefix::Minus,
                    Box::new(Expr::Literal(Literal::Int(15))),
                )),
            ),
            (
                "+15;",
                Stmt::Expr(Expr::Prefix(
                    Prefix::Plus,
                    Box::new(Expr::Literal(Literal::Int(15))),
                )),
            ),
        ];

        for (input, expect) in tests {
            let mut parser = Parser::new(Lexer::new(input));
            let program = parser.parse();

            check_parse_errors(&mut parser);
            assert_eq!(vec![expect], program);
        }
    }

    #[test]
    fn test_infix_expr() {
        let tests = vec![
            (
                "5 + 5;",
                Stmt::Expr(Expr::Infix(
                    Infix::Plus,
                    Box::new(Expr::Literal(Literal::Int(5))),
                    Box::new(Expr::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 - 5;",
                Stmt::Expr(Expr::Infix(
                    Infix::Minus,
                    Box::new(Expr::Literal(Literal::Int(5))),
                    Box::new(Expr::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 * 5;",
                Stmt::Expr(Expr::Infix(
                    Infix::Multiply,
                    Box::new(Expr::Literal(Literal::Int(5))),
                    Box::new(Expr::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 / 5;",
                Stmt::Expr(Expr::Infix(
                    Infix::Divide,
                    Box::new(Expr::Literal(Literal::Int(5))),
                    Box::new(Expr::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 > 5;",
                Stmt::Expr(Expr::Infix(
                    Infix::GreaterThan,
                    Box::new(Expr::Literal(Literal::Int(5))),
                    Box::new(Expr::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 < 5;",
                Stmt::Expr(Expr::Infix(
                    Infix::LessThan,
                    Box::new(Expr::Literal(Literal::Int(5))),
                    Box::new(Expr::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 == 5;",
                Stmt::Expr(Expr::Infix(
                    Infix::Equal,
                    Box::new(Expr::Literal(Literal::Int(5))),
                    Box::new(Expr::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 != 5;",
                Stmt::Expr(Expr::Infix(
                    Infix::NotEqual,
                    Box::new(Expr::Literal(Literal::Int(5))),
                    Box::new(Expr::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 >= 5;",
                Stmt::Expr(Expr::Infix(
                    Infix::GreaterThanEqual,
                    Box::new(Expr::Literal(Literal::Int(5))),
                    Box::new(Expr::Literal(Literal::Int(5))),
                )),
            ),
            (
                "5 <= 5;",
                Stmt::Expr(Expr::Infix(
                    Infix::LessThanEqual,
                    Box::new(Expr::Literal(Literal::Int(5))),
                    Box::new(Expr::Literal(Literal::Int(5))),
                )),
            ),
        ];

        for (input, expect) in tests {
            let mut parser = Parser::new(Lexer::new(input));
            let program = parser.parse();

            check_parse_errors(&mut parser);
            assert_eq!(vec![expect], program);
        }
    }

    #[test]
    fn test_if_expr() {
        let input = "if (x < y) { x }";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(
            vec![Stmt::Expr(Expr::If {
                cond: Box::new(Expr::Infix(
                    Infix::LessThan,
                    Box::new(Expr::Ident(Ident(String::from("x")))),
                    Box::new(Expr::Ident(Ident(String::from("y")))),
                )),
                consequence: vec![Stmt::Expr(Expr::Ident(Ident(String::from("x"))))],
                alternative: None,
            })],
            program,
        );
    }

    #[test]
    fn test_if_else_expr() {
        let input = "if (x < y) { x } else { y }";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(
            vec![Stmt::Expr(Expr::If {
                cond: Box::new(Expr::Infix(
                    Infix::LessThan,
                    Box::new(Expr::Ident(Ident(String::from("x")))),
                    Box::new(Expr::Ident(Ident(String::from("y")))),
                )),
                consequence: vec![Stmt::Expr(Expr::Ident(Ident(String::from("x"))))],
                alternative: Some(vec![Stmt::Expr(Expr::Ident(Ident(String::from("y"))))]),
            })],
            program,
        );
    }

    #[test]
    fn test_func_expr() {
        let input = "fn(x, y) { x + y; }";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(
            vec![Stmt::Expr(Expr::Func {
                params: vec![Ident(String::from("x")), Ident(String::from("y"))],
                body: vec![Stmt::Expr(Expr::Infix(
                    Infix::Plus,
                    Box::new(Expr::Ident(Ident(String::from("x")))),
                    Box::new(Expr::Ident(Ident(String::from("y")))),
                ))],
            })],
            program,
        );
    }

    #[test]
    fn test_func_params() {
        let tests = vec![
            ("fn() {};", vec![]),
            ("fn(x) {};", vec![Ident(String::from("x"))]),
            (
                "fn(x, y, z) {};",
                vec![
                    Ident(String::from("x")),
                    Ident(String::from("y")),
                    Ident(String::from("z")),
                ],
            ),
        ];

        for (input, expect) in tests {
            let mut parser = Parser::new(Lexer::new(input));
            let program = parser.parse();

            check_parse_errors(&mut parser);
            assert_eq!(
                vec![Stmt::Expr(Expr::Func {
                    params: expect,
                    body: vec![],
                })],
                program,
            );
        }
    }

    #[test]
    fn test_call_expr() {
        let input = "add(1, 2 * 3, 4 + 5);";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(
            vec![Stmt::Expr(Expr::Call {
                func: Box::new(Expr::Ident(Ident(String::from("add")))),
                args: vec![
                    Expr::Literal(Literal::Int(1)),
                    Expr::Infix(
                        Infix::Multiply,
                        Box::new(Expr::Literal(Literal::Int(2))),
                        Box::new(Expr::Literal(Literal::Int(3))),
                    ),
                    Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::Literal(Literal::Int(4))),
                        Box::new(Expr::Literal(Literal::Int(5))),
                    ),
                ],
            })],
            program,
        );
    }

    #[test]
    fn test_operator_precedence_parsing() {
        let tests = vec![
            (
                "-a * b",
                Stmt::Expr(Expr::Infix(
                    Infix::Multiply,
                    Box::new(Expr::Prefix(
                        Prefix::Minus,
                        Box::new(Expr::Ident(Ident(String::from("a")))),
                    )),
                    Box::new(Expr::Ident(Ident(String::from("b")))),
                )),
            ),
            (
                "!-a",
                Stmt::Expr(Expr::Prefix(
                    Prefix::Not,
                    Box::new(Expr::Prefix(
                        Prefix::Minus,
                        Box::new(Expr::Ident(Ident(String::from("a")))),
                    )),
                )),
            ),
            (
                "a + b + c",
                Stmt::Expr(Expr::Infix(
                    Infix::Plus,
                    Box::new(Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::Ident(Ident(String::from("a")))),
                        Box::new(Expr::Ident(Ident(String::from("b")))),
                    )),
                    Box::new(Expr::Ident(Ident(String::from("c")))),
                )),
            ),
            (
                "a + b - c",
                Stmt::Expr(Expr::Infix(
                    Infix::Minus,
                    Box::new(Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::Ident(Ident(String::from("a")))),
                        Box::new(Expr::Ident(Ident(String::from("b")))),
                    )),
                    Box::new(Expr::Ident(Ident(String::from("c")))),
                )),
            ),
            (
                "a * b * c",
                Stmt::Expr(Expr::Infix(
                    Infix::Multiply,
                    Box::new(Expr::Infix(
                        Infix::Multiply,
                        Box::new(Expr::Ident(Ident(String::from("a")))),
                        Box::new(Expr::Ident(Ident(String::from("b")))),
                    )),
                    Box::new(Expr::Ident(Ident(String::from("c")))),
                )),
            ),
            (
                "a * b / c",
                Stmt::Expr(Expr::Infix(
                    Infix::Divide,
                    Box::new(Expr::Infix(
                        Infix::Multiply,
                        Box::new(Expr::Ident(Ident(String::from("a")))),
                        Box::new(Expr::Ident(Ident(String::from("b")))),
                    )),
                    Box::new(Expr::Ident(Ident(String::from("c")))),
                )),
            ),
            (
                "a + b / c",
                Stmt::Expr(Expr::Infix(
                    Infix::Plus,
                    Box::new(Expr::Ident(Ident(String::from("a")))),
                    Box::new(Expr::Infix(
                        Infix::Divide,
                        Box::new(Expr::Ident(Ident(String::from("b")))),
                        Box::new(Expr::Ident(Ident(String::from("c")))),
                    )),
                )),
            ),
            (
                "a + b * c + d / e - f",
                Stmt::Expr(Expr::Infix(
                    Infix::Minus,
                    Box::new(Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::Infix(
                            Infix::Plus,
                            Box::new(Expr::Ident(Ident(String::from("a")))),
                            Box::new(Expr::Infix(
                                Infix::Multiply,
                                Box::new(Expr::Ident(Ident(String::from("b")))),
                                Box::new(Expr::Ident(Ident(String::from("c")))),
                            )),
                        )),
                        Box::new(Expr::Infix(
                            Infix::Divide,
                            Box::new(Expr::Ident(Ident(String::from("d")))),
                            Box::new(Expr::Ident(Ident(String::from("e")))),
                        )),
                    )),
                    Box::new(Expr::Ident(Ident(String::from("f")))),
                )),
            ),
            (
                "5 > 4 == 3 < 4",
                Stmt::Expr(Expr::Infix(
                    Infix::Equal,
                    Box::new(Expr::Infix(
                        Infix::GreaterThan,
                        Box::new(Expr::Literal(Literal::Int(5))),
                        Box::new(Expr::Literal(Literal::Int(4))),
                    )),
                    Box::new(Expr::Infix(
                        Infix::LessThan,
                        Box::new(Expr::Literal(Literal::Int(3))),
                        Box::new(Expr::Literal(Literal::Int(4))),
                    )),
                )),
            ),
            (
                "5 < 4 != 3 > 4",
                Stmt::Expr(Expr::Infix(
                    Infix::NotEqual,
                    Box::new(Expr::Infix(
                        Infix::LessThan,
                        Box::new(Expr::Literal(Literal::Int(5))),
                        Box::new(Expr::Literal(Literal::Int(4))),
                    )),
                    Box::new(Expr::Infix(
                        Infix::GreaterThan,
                        Box::new(Expr::Literal(Literal::Int(3))),
                        Box::new(Expr::Literal(Literal::Int(4))),
                    )),
                )),
            ),
            (
                "5 >= 4 == 3 <= 4",
                Stmt::Expr(Expr::Infix(
                    Infix::Equal,
                    Box::new(Expr::Infix(
                        Infix::GreaterThanEqual,
                        Box::new(Expr::Literal(Literal::Int(5))),
                        Box::new(Expr::Literal(Literal::Int(4))),
                    )),
                    Box::new(Expr::Infix(
                        Infix::LessThanEqual,
                        Box::new(Expr::Literal(Literal::Int(3))),
                        Box::new(Expr::Literal(Literal::Int(4))),
                    )),
                )),
            ),
            (
                "5 <= 4 != 3 >= 4",
                Stmt::Expr(Expr::Infix(
                    Infix::NotEqual,
                    Box::new(Expr::Infix(
                        Infix::LessThanEqual,
                        Box::new(Expr::Literal(Literal::Int(5))),
                        Box::new(Expr::Literal(Literal::Int(4))),
                    )),
                    Box::new(Expr::Infix(
                        Infix::GreaterThanEqual,
                        Box::new(Expr::Literal(Literal::Int(3))),
                        Box::new(Expr::Literal(Literal::Int(4))),
                    )),
                )),
            ),
            (
                "3 + 4 * 5 == 3 * 1 + 4 * 5",
                Stmt::Expr(Expr::Infix(
                    Infix::Equal,
                    Box::new(Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::Literal(Literal::Int(3))),
                        Box::new(Expr::Infix(
                            Infix::Multiply,
                            Box::new(Expr::Literal(Literal::Int(4))),
                            Box::new(Expr::Literal(Literal::Int(5))),
                        )),
                    )),
                    Box::new(Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::Infix(
                            Infix::Multiply,
                            Box::new(Expr::Literal(Literal::Int(3))),
                            Box::new(Expr::Literal(Literal::Int(1))),
                        )),
                        Box::new(Expr::Infix(
                            Infix::Multiply,
                            Box::new(Expr::Literal(Literal::Int(4))),
                            Box::new(Expr::Literal(Literal::Int(5))),
                        )),
                    )),
                )),
            ),
            ("true", Stmt::Expr(Expr::Literal(Literal::Bool(true)))),
            ("false", Stmt::Expr(Expr::Literal(Literal::Bool(false)))),
            (
                "3 > 5 == false",
                Stmt::Expr(Expr::Infix(
                    Infix::Equal,
                    Box::new(Expr::Infix(
                        Infix::GreaterThan,
                        Box::new(Expr::Literal(Literal::Int(3))),
                        Box::new(Expr::Literal(Literal::Int(5))),
                    )),
                    Box::new(Expr::Literal(Literal::Bool(false))),
                )),
            ),
            (
                "3 < 5 == true",
                Stmt::Expr(Expr::Infix(
                    Infix::Equal,
                    Box::new(Expr::Infix(
                        Infix::LessThan,
                        Box::new(Expr::Literal(Literal::Int(3))),
                        Box::new(Expr::Literal(Literal::Int(5))),
                    )),
                    Box::new(Expr::Literal(Literal::Bool(true))),
                )),
            ),
            (
                "1 + (2 + 3) + 4",
                Stmt::Expr(Expr::Infix(
                    Infix::Plus,
                    Box::new(Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::Literal(Literal::Int(1))),
                        Box::new(Expr::Infix(
                            Infix::Plus,
                            Box::new(Expr::Literal(Literal::Int(2))),
                            Box::new(Expr::Literal(Literal::Int(3))),
                        )),
                    )),
                    Box::new(Expr::Literal(Literal::Int(4))),
                )),
            ),
            (
                "(5 + 5) * 2",
                Stmt::Expr(Expr::Infix(
                    Infix::Multiply,
                    Box::new(Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::Literal(Literal::Int(5))),
                        Box::new(Expr::Literal(Literal::Int(5))),
                    )),
                    Box::new(Expr::Literal(Literal::Int(2))),
                )),
            ),
            (
                "2 / (5 + 5)",
                Stmt::Expr(Expr::Infix(
                    Infix::Divide,
                    Box::new(Expr::Literal(Literal::Int(2))),
                    Box::new(Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::Literal(Literal::Int(5))),
                        Box::new(Expr::Literal(Literal::Int(5))),
                    )),
                )),
            ),
            (
                "-(5 + 5)",
                Stmt::Expr(Expr::Prefix(
                    Prefix::Minus,
                    Box::new(Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::Literal(Literal::Int(5))),
                        Box::new(Expr::Literal(Literal::Int(5))),
                    )),
                )),
            ),
            (
                "!(true == true)",
                Stmt::Expr(Expr::Prefix(
                    Prefix::Not,
                    Box::new(Expr::Infix(
                        Infix::Equal,
                        Box::new(Expr::Literal(Literal::Bool(true))),
                        Box::new(Expr::Literal(Literal::Bool(true))),
                    )),
                )),
            ),
            (
                "a + add(b * c) + d",
                Stmt::Expr(Expr::Infix(
                    Infix::Plus,
                    Box::new(Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::Ident(Ident(String::from("a")))),
                        Box::new(Expr::Call {
                            func: Box::new(Expr::Ident(Ident(String::from("add")))),
                            args: vec![Expr::Infix(
                                Infix::Multiply,
                                Box::new(Expr::Ident(Ident(String::from("b")))),
                                Box::new(Expr::Ident(Ident(String::from("c")))),
                            )],
                        }),
                    )),
                    Box::new(Expr::Ident(Ident(String::from("d")))),
                )),
            ),
            (
                "add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8))",
                Stmt::Expr(Expr::Call {
                    func: Box::new(Expr::Ident(Ident(String::from("add")))),
                    args: vec![
                        Expr::Ident(Ident(String::from("a"))),
                        Expr::Ident(Ident(String::from("b"))),
                        Expr::Literal(Literal::Int(1)),
                        Expr::Infix(
                            Infix::Multiply,
                            Box::new(Expr::Literal(Literal::Int(2))),
                            Box::new(Expr::Literal(Literal::Int(3))),
                        ),
                        Expr::Infix(
                            Infix::Plus,
                            Box::new(Expr::Literal(Literal::Int(4))),
                            Box::new(Expr::Literal(Literal::Int(5))),
                        ),
                        Expr::Call {
                            func: Box::new(Expr::Ident(Ident(String::from("add")))),
                            args: vec![
                                Expr::Literal(Literal::Int(6)),
                                Expr::Infix(
                                    Infix::Multiply,
                                    Box::new(Expr::Literal(Literal::Int(7))),
                                    Box::new(Expr::Literal(Literal::Int(8))),
                                ),
                            ],
                        },
                    ],
                }),
            ),
            (
                "add(a + b + c * d / f + g)",
                Stmt::Expr(Expr::Call {
                    func: Box::new(Expr::Ident(Ident(String::from("add")))),
                    args: vec![Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::Infix(
                            Infix::Plus,
                            Box::new(Expr::Infix(
                                Infix::Plus,
                                Box::new(Expr::Ident(Ident(String::from("a")))),
                                Box::new(Expr::Ident(Ident(String::from("b")))),
                            )),
                            Box::new(Expr::Infix(
                                Infix::Divide,
                                Box::new(Expr::Infix(
                                    Infix::Multiply,
                                    Box::new(Expr::Ident(Ident(String::from("c")))),
                                    Box::new(Expr::Ident(Ident(String::from("d")))),
                                )),
                                Box::new(Expr::Ident(Ident(String::from("f")))),
                            )),
                        )),
                        Box::new(Expr::Ident(Ident(String::from("g")))),
                    )],
                }),
            ),
            (
                "a * [1, 2, 3, 4][b * c] * d",
                Stmt::Expr(Expr::Infix(
                    Infix::Multiply,
                    Box::new(Expr::Infix(
                        Infix::Multiply,
                        Box::new(Expr::Ident(Ident(String::from("a")))),
                        Box::new(Expr::Index(
                            Box::new(Expr::Literal(Literal::Array(vec![
                                Expr::Literal(Literal::Int(1)),
                                Expr::Literal(Literal::Int(2)),
                                Expr::Literal(Literal::Int(3)),
                                Expr::Literal(Literal::Int(4)),
                            ]))),
                            Box::new(Expr::Infix(
                                Infix::Multiply,
                                Box::new(Expr::Ident(Ident(String::from("b")))),
                                Box::new(Expr::Ident(Ident(String::from("c")))),
                            )),
                        )),
                    )),
                    Box::new(Expr::Ident(Ident(String::from("d")))),
                )),
            ),
            (
                "add(a * b[2], b[1], 2 * [1, 2][1])",
                Stmt::Expr(Expr::Call {
                    func: Box::new(Expr::Ident(Ident(String::from("add")))),
                    args: vec![
                        Expr::Infix(
                            Infix::Multiply,
                            Box::new(Expr::Ident(Ident(String::from("a")))),
                            Box::new(Expr::Index(
                                Box::new(Expr::Ident(Ident(String::from("b")))),
                                Box::new(Expr::Literal(Literal::Int(2))),
                            )),
                        ),
                        Expr::Index(
                            Box::new(Expr::Ident(Ident(String::from("b")))),
                            Box::new(Expr::Literal(Literal::Int(1))),
                        ),
                        Expr::Infix(
                            Infix::Multiply,
                            Box::new(Expr::Literal(Literal::Int(2))),
                            Box::new(Expr::Index(
                                Box::new(Expr::Literal(Literal::Array(vec![
                                    Expr::Literal(Literal::Int(1)),
                                    Expr::Literal(Literal::Int(2)),
                                ]))),
                                Box::new(Expr::Literal(Literal::Int(1))),
                            )),
                        ),
                    ],
                }),
            ),
        ];

        for (input, expect) in tests {
            let mut parser = Parser::new(Lexer::new(input));
            let program = parser.parse();

            check_parse_errors(&mut parser);
            assert_eq!(vec![expect], program);
        }
    }
}
