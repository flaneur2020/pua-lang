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
            _ => Precedence::Lowest
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
                    "no prefix parse function for %s found \"{:?}\"",
                    self.current_token,
                ),
        ));
    }

    pub fn parse(&mut self) -> Program {
        let mut program: Program = vec![];

        while !self.current_token_is(Token::Eof) {
            match self.parse_stmt() {
                Some(stmt) => program.push(stmt),
                None => {},
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
                None => {},
            }
            self.bump();
        }

        block
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        match self.current_token {
            Token::Let => self.parse_let_stmt(),
            Token::Return => self.parse_return_stmt(),
            _ => self.parse_expr_stmt(),
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

    fn parse_expr_stmt(&mut self) -> Option<Stmt> {
        match self.parse_expr(Precedence::Lowest) {
            Some(expr) => {
                if self.next_token_is(Token::Semicolon) {
                    self.bump();
                }
                Some(Stmt::Expr(expr))
            },
            None => None,
        }
    }

    fn parse_expr(&mut self, precedence: Precedence) -> Option<Expr> {
        // prefix
        let mut left = match self.current_token {
            Token::Ident(_) => self.parse_expr_ident(),
            Token::Int(_) => self.parse_expr_int(),
            Token::Bool(_) => self.parse_expr_bool(),
            Token::Bang | Token::Minus => self.parse_expr_prefix(),
            Token::Lparen => self.parse_expr_grouped(),
            Token::If => self.parse_expr_if(),
            _ => {
                self.error_no_prefix_parser();
                return None;
            }
        };

        // infix
        while !self.next_token_is(Token::Semicolon) && precedence < self.next_token_precedence() {
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
                        left = self.parse_expr_infix(left.unwrap());
                    },
                _ => return left,
            }
        }

        left
    }

    fn parse_expr_ident(&mut self) -> Option<Expr> {
        match self.current_token {
            Token::Ident(ref mut ident) => Some(Expr::Ident(Ident(ident.clone()))), // FIXME Is `.clone()` correct?
            _ => None
        }
    }

    fn parse_expr_int(&mut self) -> Option<Expr> {
        match self.current_token {
            Token::Int(ref mut int) => Some(Expr::Literal(Literal::Int(int.clone()))), // FIXME Is `.clone()` correct?
            _ => None
        }
    }

    fn parse_expr_bool(&mut self) -> Option<Expr> {
        match self.current_token {
            Token::Bool(value) => Some(Expr::Literal(Literal::Bool(value == true))),
            _ => None
        }
    }

    fn parse_expr_prefix(&mut self) -> Option<Expr> {
        let prefix = match self.current_token {
            Token::Bang => Prefix::Not,
            Token::Minus => Prefix::Minus,
            _ => return None,
        };

        self.bump();

        let right = match self.parse_expr(Precedence::Prefix) {
            Some(expr) => expr,
            None => return None,
        };

        Some(Expr::Prefix(prefix, Box::new(right)))
    }

    fn parse_expr_grouped(&mut self) -> Option<Expr> {
        self.bump();

        let expr = self.parse_expr(Precedence::Lowest);

        if !self.expect_next_token(Token::Rparen) {
            None
        } else {
            expr
        }
    }

    fn parse_expr_if(&mut self) -> Option<Expr> {
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

        if self.next_token_is(Token::Else) {
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

    fn parse_expr_infix(&mut self, left: Expr) -> Option<Expr> {
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

        let right = match self.parse_expr(precedence) {
            Some(expr) => expr,
            None => return None,
        };

        Some(Expr::Infix(infix, Box::new(left), Box::new(right)))
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

        let mut parser = Parser::new(Lexer::new(input));
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

        let mut parser = Parser::new(Lexer::new(input));
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

    #[test]
    fn test_ident_expr() {
        let input = "foobar;";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(1, program.len());
        assert_eq!(Stmt::Expr(Expr::Ident(Ident(String::from("foobar")))), program[0]);
    }

    #[test]
    fn test_integer_literal_expr() {
        let input = "5;";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(1, program.len());
        assert_eq!(Stmt::Expr(Expr::Literal(Literal::Int(5))), program[0]);
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
            assert_eq!(1, program.len());
            assert_eq!(expect, program[0]);
        }
    }

    #[test]
    fn test_prefix_expr() {
        let tests = vec![
            ("!5;", Stmt::Expr(Expr::Prefix(Prefix::Not, Box::new(Expr::Literal(Literal::Int(5)))))),
            ("-15;", Stmt::Expr(Expr::Prefix(Prefix::Minus, Box::new(Expr::Literal(Literal::Int(15)))))),
        ];

        for (input, expect) in tests {
            let mut parser = Parser::new(Lexer::new(input));
            let program = parser.parse();

            check_parse_errors(&mut parser);
            assert_eq!(1, program.len());
            assert_eq!(expect, program[0]);
        }
    }

    #[test]
    fn test_infix_expr() {
        let tests = vec![
            ("5 + 5;", Stmt::Expr(Expr::Infix(
                Infix::Plus,
                Box::new(Expr::Literal(Literal::Int(5))),
                Box::new(Expr::Literal(Literal::Int(5))),
            ))),
            ("5 - 5;", Stmt::Expr(Expr::Infix(
                Infix::Minus,
                Box::new(Expr::Literal(Literal::Int(5))),
                Box::new(Expr::Literal(Literal::Int(5))),
            ))),
            ("5 * 5;", Stmt::Expr(Expr::Infix(
                Infix::Multiply,
                Box::new(Expr::Literal(Literal::Int(5))),
                Box::new(Expr::Literal(Literal::Int(5))),
            ))),
            ("5 / 5;", Stmt::Expr(Expr::Infix(
                Infix::Divide,
                Box::new(Expr::Literal(Literal::Int(5))),
                Box::new(Expr::Literal(Literal::Int(5))),
            ))),
            ("5 > 5;", Stmt::Expr(Expr::Infix(
                Infix::GreaterThan,
                Box::new(Expr::Literal(Literal::Int(5))),
                Box::new(Expr::Literal(Literal::Int(5))),
            ))),
            ("5 < 5;", Stmt::Expr(Expr::Infix(
                Infix::LessThan,
                Box::new(Expr::Literal(Literal::Int(5))),
                Box::new(Expr::Literal(Literal::Int(5))),
            ))),
            ("5 == 5;", Stmt::Expr(Expr::Infix(
                Infix::Equal,
                Box::new(Expr::Literal(Literal::Int(5))),
                Box::new(Expr::Literal(Literal::Int(5))),
            ))),
            ("5 != 5;", Stmt::Expr(Expr::Infix(
                Infix::NotEqual,
                Box::new(Expr::Literal(Literal::Int(5))),
                Box::new(Expr::Literal(Literal::Int(5))),
            ))),
            ("5 >= 5;", Stmt::Expr(Expr::Infix(
                Infix::GreaterThanEqual,
                Box::new(Expr::Literal(Literal::Int(5))),
                Box::new(Expr::Literal(Literal::Int(5))),
            ))),
            ("5 <= 5;", Stmt::Expr(Expr::Infix(
                Infix::LessThanEqual,
                Box::new(Expr::Literal(Literal::Int(5))),
                Box::new(Expr::Literal(Literal::Int(5))),
            ))),
        ];

        for (input, expect) in tests {
            let mut parser = Parser::new(Lexer::new(input));
            let program = parser.parse();

            check_parse_errors(&mut parser);

            assert_eq!(1, program.len());
            assert_eq!(expect, program[0]);
        }
    }

    #[test]
    fn test_if_expr() {
        let input = "if (x < y) { x }";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(1, program.len());
        assert_eq!(
            Stmt::Expr(
                Expr::If {
                    cond: Box::new(Expr::Infix(Infix::LessThan, Box::new(Expr::Ident(Ident(String::from("x")))), Box::new(Expr::Ident(Ident(String::from("y")))))),
                    consequence: vec![
                        Stmt::Expr(Expr::Ident(Ident(String::from("x")))),
                    ],
                    alternative: None,
                },
            ),
            program[0],
        );
    }

    #[test]
    fn test_if_else_expr() {
        let input = "if (x < y) { x } else { y }";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse();

        check_parse_errors(&mut parser);
        assert_eq!(1, program.len());
        assert_eq!(
            Stmt::Expr(
                Expr::If {
                    cond: Box::new(Expr::Infix(Infix::LessThan, Box::new(Expr::Ident(Ident(String::from("x")))), Box::new(Expr::Ident(Ident(String::from("y")))))),
                    consequence: vec![
                        Stmt::Expr(Expr::Ident(Ident(String::from("x")))),
                    ],
                    alternative: Some(vec![
                        Stmt::Expr(Expr::Ident(Ident(String::from("y"))))
                    ]),
                },
            ),
            program[0],
        );
    }

    #[test]
    fn test_operator_precedence_parsing() {
        let tests = vec![
            ("-a * b", Stmt::Expr(
                Expr::Infix(
                    Infix::Multiply,
                    Box::new(Expr::Prefix(Prefix::Minus, Box::new(Expr::Ident(Ident(String::from("a")))))),
                    Box::new(Expr::Ident(Ident(String::from("b")))),
                ),
            )),
            ("!-a", Stmt::Expr(
                Expr::Prefix(
                    Prefix::Not,
                    Box::new(
                        Expr::Prefix(
                            Prefix::Minus,
                            Box::new(Expr::Ident(Ident(String::from("a")))),
                        ),
                    ),
                ),
            )),
            ("a + b + c", Stmt::Expr(
                Expr::Infix(
                    Infix::Plus,
                    Box::new(
                        Expr::Infix(
                            Infix::Plus,
                            Box::new(Expr::Ident(Ident(String::from("a")))),
                            Box::new(Expr::Ident(Ident(String::from("b")))),
                        ),
                    ),
                    Box::new(Expr::Ident(Ident(String::from("c")))),
                ),
            )),
            ("a + b - c", Stmt::Expr(
                Expr::Infix(
                    Infix::Minus,
                    Box::new(
                        Expr::Infix(
                            Infix::Plus,
                            Box::new(Expr::Ident(Ident(String::from("a")))),
                            Box::new(Expr::Ident(Ident(String::from("b")))),
                        ),
                    ),
                    Box::new(Expr::Ident(Ident(String::from("c")))),
                ),
            )),
            ("a * b * c", Stmt::Expr(
                Expr::Infix(
                    Infix::Multiply,
                    Box::new(
                        Expr::Infix(
                            Infix::Multiply,
                            Box::new(Expr::Ident(Ident(String::from("a")))),
                            Box::new(Expr::Ident(Ident(String::from("b")))),
                        ),
                    ),
                    Box::new(Expr::Ident(Ident(String::from("c")))),
                ),
            )),
            ("a * b / c", Stmt::Expr(
                Expr::Infix(
                    Infix::Divide,
                    Box::new(
                        Expr::Infix(
                            Infix::Multiply,
                            Box::new(Expr::Ident(Ident(String::from("a")))),
                            Box::new(Expr::Ident(Ident(String::from("b")))),
                        ),
                    ),
                    Box::new(Expr::Ident(Ident(String::from("c")))),
                ),
            )),
            ("a + b / c", Stmt::Expr(
                Expr::Infix(
                    Infix::Plus,
                    Box::new(Expr::Ident(Ident(String::from("a")))),
                    Box::new(
                        Expr::Infix(
                            Infix::Divide,
                            Box::new(Expr::Ident(Ident(String::from("b")))),
                            Box::new(Expr::Ident(Ident(String::from("c")))),
                        ),
                    ),
                ),
            )),
            ("a + b * c + d / e - f", Stmt::Expr(
                Expr::Infix(
                    Infix::Minus,
                    Box::new(
                        Expr::Infix(
                            Infix::Plus,
                            Box::new(
                                Expr::Infix(
                                    Infix::Plus,
                                    Box::new(Expr::Ident(Ident(String::from("a")))),
                                    Box::new(
                                        Expr::Infix(
                                            Infix::Multiply,
                                            Box::new(Expr::Ident(Ident(String::from("b")))),
                                            Box::new(Expr::Ident(Ident(String::from("c")))),
                                        ),
                                    ),
                                ),
                            ),
                            Box::new(
                                Expr::Infix(
                                    Infix::Divide,
                                    Box::new(Expr::Ident(Ident(String::from("d")))),
                                    Box::new(Expr::Ident(Ident(String::from("e")))),
                                ),
                            ),
                        ),
                    ),
                    Box::new(Expr::Ident(Ident(String::from("f")))),
                ),
            )),
            ("5 > 4 == 3 < 4", Stmt::Expr(
                Expr::Infix(
                    Infix::Equal,
                    Box::new(
                        Expr::Infix(
                            Infix::GreaterThan,
                            Box::new(Expr::Literal(Literal::Int(5))),
                            Box::new(Expr::Literal(Literal::Int(4))),
                        ),
                    ),
                    Box::new(
                        Expr::Infix(
                            Infix::LessThan,
                            Box::new(Expr::Literal(Literal::Int(3))),
                            Box::new(Expr::Literal(Literal::Int(4))),
                        ),
                    ),
                ),
            )),
            ("5 < 4 != 3 > 4", Stmt::Expr(
                Expr::Infix(
                    Infix::NotEqual,
                    Box::new(
                        Expr::Infix(
                            Infix::LessThan,
                            Box::new(Expr::Literal(Literal::Int(5))),
                            Box::new(Expr::Literal(Literal::Int(4))),
                        ),
                    ),
                    Box::new(
                        Expr::Infix(
                            Infix::GreaterThan,
                            Box::new(Expr::Literal(Literal::Int(3))),
                            Box::new(Expr::Literal(Literal::Int(4))),
                        ),
                    ),
                ),
            )),
            ("5 >= 4 == 3 <= 4", Stmt::Expr(
                Expr::Infix(
                    Infix::Equal,
                    Box::new(
                        Expr::Infix(
                            Infix::GreaterThanEqual,
                            Box::new(Expr::Literal(Literal::Int(5))),
                            Box::new(Expr::Literal(Literal::Int(4))),
                        ),
                    ),
                    Box::new(
                        Expr::Infix(
                            Infix::LessThanEqual,
                            Box::new(Expr::Literal(Literal::Int(3))),
                            Box::new(Expr::Literal(Literal::Int(4))),
                        ),
                    ),
                ),
            )),
            ("5 <= 4 != 3 >= 4", Stmt::Expr(
                Expr::Infix(
                    Infix::NotEqual,
                    Box::new(
                        Expr::Infix(
                            Infix::LessThanEqual,
                            Box::new(Expr::Literal(Literal::Int(5))),
                            Box::new(Expr::Literal(Literal::Int(4))),
                        ),
                    ),
                    Box::new(
                        Expr::Infix(
                            Infix::GreaterThanEqual,
                            Box::new(Expr::Literal(Literal::Int(3))),
                            Box::new(Expr::Literal(Literal::Int(4))),
                        ),
                    ),
                ),
            )),
            ("3 + 4 * 5 == 3 * 1 + 4 * 5", Stmt::Expr(
                Expr::Infix(
                    Infix::Equal,
                    Box::new(
                        Expr::Infix(
                            Infix::Plus,
                            Box::new(Expr::Literal(Literal::Int(3))),
                            Box::new(
                                Expr::Infix(
                                    Infix::Multiply,
                                    Box::new(Expr::Literal(Literal::Int(4))),
                                    Box::new(Expr::Literal(Literal::Int(5))),
                                ),
                            ),
                        ),
                    ),
                    Box::new(
                        Expr::Infix(
                            Infix::Plus,
                            Box::new(
                                Expr::Infix(
                                    Infix::Multiply,
                                    Box::new(Expr::Literal(Literal::Int(3))),
                                    Box::new(Expr::Literal(Literal::Int(1))),
                                ),
                            ),
                            Box::new(
                                Expr::Infix(
                                    Infix::Multiply,
                                    Box::new(Expr::Literal(Literal::Int(4))),
                                    Box::new(Expr::Literal(Literal::Int(5))),
                                ),
                            ),
                        ),
                    ),
                ),
            )),
            ("true", Stmt::Expr(
                Expr::Literal(Literal::Bool(true)),
            )),
            ("false", Stmt::Expr(
                Expr::Literal(Literal::Bool(false)),
            )),
            ("3 > 5 == false", Stmt::Expr(
                Expr::Infix(
                    Infix::Equal,
                    Box::new(
                        Expr::Infix(
                            Infix::GreaterThan,
                            Box::new(Expr::Literal(Literal::Int(3))),
                            Box::new(Expr::Literal(Literal::Int(5))),
                        ),
                    ),
                    Box::new(Expr::Literal(Literal::Bool(false))),
                ),
            )),
            ("3 < 5 == true", Stmt::Expr(
                Expr::Infix(
                    Infix::Equal,
                    Box::new(
                        Expr::Infix(
                            Infix::LessThan,
                            Box::new(Expr::Literal(Literal::Int(3))),
                            Box::new(Expr::Literal(Literal::Int(5))),
                        ),
                    ),
                    Box::new(Expr::Literal(Literal::Bool(true))),
                ),
            )),
            ("1 + (2 + 3) + 4", Stmt::Expr(
                Expr::Infix(
                    Infix::Plus,
                    Box::new(
                        Expr::Infix(
                            Infix::Plus,
                            Box::new(Expr::Literal(Literal::Int(1))),
                            Box::new(
                                Expr::Infix(
                                    Infix::Plus,
                                    Box::new(Expr::Literal(Literal::Int(2))),
                                    Box::new(Expr::Literal(Literal::Int(3))),
                                ),
                            ),
                        ),
                    ),
                    Box::new(Expr::Literal(Literal::Int(4))),
                ),
            )),
            ("(5 + 5) * 2", Stmt::Expr(
                Expr::Infix(
                    Infix::Multiply,
                    Box::new(
                        Expr::Infix(
                            Infix::Plus,
                            Box::new(Expr::Literal(Literal::Int(5))),
                            Box::new(Expr::Literal(Literal::Int(5))),
                        ),
                    ),
                    Box::new(Expr::Literal(Literal::Int(2))),
                ),
            )),
            ("2 / (5 + 5)", Stmt::Expr(
                Expr::Infix(
                    Infix::Divide,
                    Box::new(Expr::Literal(Literal::Int(2))),
                    Box::new(
                        Expr::Infix(
                            Infix::Plus,
                            Box::new(Expr::Literal(Literal::Int(5))),
                            Box::new(Expr::Literal(Literal::Int(5))),
                        ),
                    ),
                ),
            )),
            ("-(5 + 5)", Stmt::Expr(
                Expr::Prefix(
                    Prefix::Minus,
                    Box::new(
                        Expr::Infix(
                            Infix::Plus,
                            Box::new(Expr::Literal(Literal::Int(5))),
                            Box::new(Expr::Literal(Literal::Int(5))),
                        ),
                    ),
                ),
            )),
            ("!(true == true)", Stmt::Expr(
                Expr::Prefix(
                    Prefix::Not,
                    Box::new(
                        Expr::Infix(
                            Infix::Equal,
                            Box::new(Expr::Literal(Literal::Bool(true))),
                            Box::new(Expr::Literal(Literal::Bool(true))),
                        ),
                    ),
                ),
            )),
        ];

        for (input, expect) in tests {
            let mut parser = Parser::new(Lexer::new(input));
            let program = parser.parse();

            check_parse_errors(&mut parser);

            assert_eq!(1, program.len());
            assert_eq!(expect, program[0]);
        }
    }
}
