use token::{Token, TokenType};

pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
    next_pos: usize,
    ch: u8,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer {
            input,
            pos: 0,
            next_pos: 0,
            ch: 0,
        };

        lexer.read_char();

        return lexer;
    }

    fn new_token(token_type: TokenType, ch: u8) -> Token {
        Token {
            token_type,
            literal: String::from_utf8(vec![ch]).unwrap(),
        }
    }

    fn read_char(&mut self) {
        if self.next_pos >= self.input.len() {
            self.ch = 0;
        } else {
            self.ch = self.input.as_bytes()[self.next_pos];
        }
        self.pos = self.next_pos;
        self.next_pos += 1;
    }

    fn nextch(&mut self) -> u8 {
        if self.next_pos >= self.input.len() {
            return 0;
        } else {
            return self.input.as_bytes()[self.next_pos];
        }
    }

    fn nextch_is(&mut self, ch: u8) -> bool {
        self.nextch() == ch
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.ch {
                b' ' | b'\t' | b'\n' | b'\r' => {
                    self.read_char();
                }
                _ => {
                    break;
                }
            }
        }
    }

    pub fn next_token(&mut self) -> Token {
        let tok;

        self.skip_whitespace();

        match self.ch {
            b'=' => {
                if self.nextch_is(b'=') {
                    let start_pos = self.pos;
                    self.read_char();
                    tok = Token {
                        token_type: TokenType::Eq,
                        literal: String::from(&self.input[start_pos..self.next_pos]),
                    };
                } else {
                    tok = Self::new_token(TokenType::Assign, self.ch);
                }
            }
            b'+' => {
                tok = Self::new_token(TokenType::Plus, self.ch);
            }
            b'-' => {
                tok = Self::new_token(TokenType::Minus, self.ch);
            }
            b'!' => {
                if self.nextch_is(b'=') {
                    let start_pos = self.pos;
                    self.read_char();
                    tok = Token {
                        token_type: TokenType::NotEq,
                        literal: String::from(&self.input[start_pos..self.next_pos]),
                    };
                } else {
                    tok = Self::new_token(TokenType::Bang, self.ch);
                }
            }
            b'/' => {
                tok = Self::new_token(TokenType::Slash, self.ch);
            }
            b'*' => {
                tok = Self::new_token(TokenType::Asterisk, self.ch);
            }
            b'<' => {
                tok = Self::new_token(TokenType::Lt, self.ch);
            }
            b'>' => {
                tok = Self::new_token(TokenType::Gt, self.ch);
            }
            b'(' => {
                tok = Self::new_token(TokenType::Lparen, self.ch);
            }
            b')' => {
                tok = Self::new_token(TokenType::Rparen, self.ch);
            }
            b'{' => {
                tok = Self::new_token(TokenType::Lbrace, self.ch);
            }
            b'}' => {
                tok = Self::new_token(TokenType::Rbrace, self.ch);
            }
            b',' => {
                tok = Self::new_token(TokenType::Comma, self.ch);
            }
            b';' => {
                tok = Self::new_token(TokenType::Semicolon, self.ch);
            }
            b'a'...b'z' | b'A'...b'Z' | b'_' => {
                return self.consume_identifier();
            }
            b'0'...b'9' => {
                return self.consume_number();
            }
            0 => {
                tok = Token {
                    token_type: TokenType::Eof,
                    literal: String::from(""),
                };
            }
            _ => {
                tok = Self::new_token(TokenType::Illegal, self.ch);
            }
        }

        self.read_char();

        return tok;
    }

    fn consume_identifier(&mut self) -> Token {
        let start_pos = self.pos;

        loop {
            match self.ch {
                b'a'...b'z' | b'A'...b'Z' | b'_' => {
                    self.read_char();
                }
                _ => {
                    break;
                }
            }
        }

        let literal = &self.input[start_pos..self.pos];

        let token_type = match literal {
            "fn" => TokenType::Function,
            "let" => TokenType::Let,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "return" => TokenType::Return,
            _ => TokenType::Ident,
        };

        Token {
            token_type,
            literal: String::from(literal),
        }
    }

    fn consume_number(&mut self) -> Token {
        let start_pos = self.pos;

        loop {
            match self.ch {
                b'0'...b'9' => {
                    self.read_char();
                }
                _ => {
                    break;
                }
            }
        }

        Token {
            token_type: TokenType::Int,
            literal: String::from(&self.input[start_pos..self.pos]),
        }
    }
}

#[cfg(test)]
mod tests {
    use lexer::Lexer;
    use token::TokenType;

    #[test]
    fn test_next_token() {
        let input = r#"let five = 5;
let ten = 10;

let add = fn(x, y) {
  x + y;
};

let result = add(five, ten);
!-/*5;
5 < 10 > 5;

if (5 < 10) {
  return true;
} else {
  return false;
}

10 == 10;
10 != 9;
"#;

        let tests = vec![
            (TokenType::Let, String::from("let")),
            (TokenType::Ident, String::from("five")),
            (TokenType::Assign, String::from("=")),
            (TokenType::Int, String::from("5")),
            (TokenType::Semicolon, String::from(";")),
            (TokenType::Let, String::from("let")),
            (TokenType::Ident, String::from("ten")),
            (TokenType::Assign, String::from("=")),
            (TokenType::Int, String::from("10")),
            (TokenType::Semicolon, String::from(";")),
            (TokenType::Let, String::from("let")),
            (TokenType::Ident, String::from("add")),
            (TokenType::Assign, String::from("=")),
            (TokenType::Function, String::from("fn")),
            (TokenType::Lparen, String::from("(")),
            (TokenType::Ident, String::from("x")),
            (TokenType::Comma, String::from(",")),
            (TokenType::Ident, String::from("y")),
            (TokenType::Rparen, String::from(")")),
            (TokenType::Lbrace, String::from("{")),
            (TokenType::Ident, String::from("x")),
            (TokenType::Plus, String::from("+")),
            (TokenType::Ident, String::from("y")),
            (TokenType::Semicolon, String::from(";")),
            (TokenType::Rbrace, String::from("}")),
            (TokenType::Semicolon, String::from(";")),
            (TokenType::Let, String::from("let")),
            (TokenType::Ident, String::from("result")),
            (TokenType::Assign, String::from("=")),
            (TokenType::Ident, String::from("add")),
            (TokenType::Lparen, String::from("(")),
            (TokenType::Ident, String::from("five")),
            (TokenType::Comma, String::from(",")),
            (TokenType::Ident, String::from("ten")),
            (TokenType::Rparen, String::from(")")),
            (TokenType::Semicolon, String::from(";")),
            (TokenType::BANG, String::from("!")),
            (TokenType::MINUS, String::from("-")),
            (TokenType::Slash, String::from("/")),
            (TokenType::Asterisk, String::from("*")),
            (TokenType::Int, String::from("5")),
            (TokenType::Semicolon, String::from(";")),
            (TokenType::Int, String::from("5")),
            (TokenType::Lt, String::from("<")),
            (TokenType::Int, String::from("10")),
            (TokenType::Gt, String::from(">")),
            (TokenType::Int, String::from("5")),
            (TokenType::Semicolon, String::from(";")),
            (TokenType::If, String::from("if")),
            (TokenType::Lparen, String::from("(")),
            (TokenType::Int, String::from("5")),
            (TokenType::Lt, String::from("<")),
            (TokenType::Int, String::from("10")),
            (TokenType::Rparen, String::from(")")),
            (TokenType::Lbrace, String::from("{")),
            (TokenType::Return, String::from("return")),
            (TokenType::True, String::from("true")),
            (TokenType::Semicolon, String::from(";")),
            (TokenType::Rbrace, String::from("}")),
            (TokenType::Else, String::from("else")),
            (TokenType::Lbrace, String::from("{")),
            (TokenType::Return, String::from("return")),
            (TokenType::False, String::from("false")),
            (TokenType::Semicolon, String::from(";")),
            (TokenType::Rbrace, String::from("}")),
            (TokenType::Int, String::from("10")),
            (TokenType::Eq, String::from("==")),
            (TokenType::Int, String::from("10")),
            (TokenType::Semicolon, String::from(";")),
            (TokenType::Int, String::from("10")),
            (TokenType::NotEq, String::from("!=")),
            (TokenType::Int, String::from("9")),
            (TokenType::Semicolon, String::from(";")),
            (TokenType::Eof, String::from("")),
        ];

        let mut lexer = Lexer::new(input);

        for (token_type, literal) in tests {
            let tok = lexer.next_token();

            assert_eq!(token_type, tok.token_type);
            assert_eq!(literal, tok.literal);
        }
    }
}
