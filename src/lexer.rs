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
                        token_type: TokenType::EQ,
                        literal: String::from(&self.input[start_pos..self.next_pos]),
                    };
                } else {
                    tok = Self::new_token(TokenType::ASSIGN, self.ch);
                }
            }
            b'+' => {
                tok = Self::new_token(TokenType::PLUS, self.ch);
            }
            b'-' => {
                tok = Self::new_token(TokenType::MINUS, self.ch);
            }
            b'!' => {
                if self.nextch_is(b'=') {
                    let start_pos = self.pos;
                    self.read_char();
                    tok = Token {
                        token_type: TokenType::NOT_EQ,
                        literal: String::from(&self.input[start_pos..self.next_pos]),
                    };
                } else {
                    tok = Self::new_token(TokenType::BANG, self.ch);
                }
            }
            b'/' => {
                tok = Self::new_token(TokenType::SLASH, self.ch);
            }
            b'*' => {
                tok = Self::new_token(TokenType::ASTERISK, self.ch);
            }
            b'<' => {
                tok = Self::new_token(TokenType::LT, self.ch);
            }
            b'>' => {
                tok = Self::new_token(TokenType::GT, self.ch);
            }
            b'(' => {
                tok = Self::new_token(TokenType::LPAREN, self.ch);
            }
            b')' => {
                tok = Self::new_token(TokenType::RPAREN, self.ch);
            }
            b'{' => {
                tok = Self::new_token(TokenType::LBRACE, self.ch);
            }
            b'}' => {
                tok = Self::new_token(TokenType::RBRACE, self.ch);
            }
            b',' => {
                tok = Self::new_token(TokenType::COMMA, self.ch);
            }
            b';' => {
                tok = Self::new_token(TokenType::SEMICOLON, self.ch);
            }
            b'a'...b'z' | b'A'...b'Z' | b'_' => {
                return self.consume_identifier();
            }
            b'0'...b'9' => {
                return self.consume_number();
            }
            0 => {
                tok = Token {
                    token_type: TokenType::EOF,
                    literal: String::from(""),
                };
            }
            _ => {
                tok = Self::new_token(TokenType::ILLEGAL, self.ch);
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
            "fn" => TokenType::FUNCTION,
            "let" => TokenType::LET,
            "true" => TokenType::TRUE,
            "false" => TokenType::FALSE,
            "if" => TokenType::IF,
            "else" => TokenType::ELSE,
            "return" => TokenType::RETURN,
            _ => TokenType::IDENT,
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
            token_type: TokenType::INT,
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
            (TokenType::LET, String::from("let")),
            (TokenType::IDENT, String::from("five")),
            (TokenType::ASSIGN, String::from("=")),
            (TokenType::INT, String::from("5")),
            (TokenType::SEMICOLON, String::from(";")),
            (TokenType::LET, String::from("let")),
            (TokenType::IDENT, String::from("ten")),
            (TokenType::ASSIGN, String::from("=")),
            (TokenType::INT, String::from("10")),
            (TokenType::SEMICOLON, String::from(";")),
            (TokenType::LET, String::from("let")),
            (TokenType::IDENT, String::from("add")),
            (TokenType::ASSIGN, String::from("=")),
            (TokenType::FUNCTION, String::from("fn")),
            (TokenType::LPAREN, String::from("(")),
            (TokenType::IDENT, String::from("x")),
            (TokenType::COMMA, String::from(",")),
            (TokenType::IDENT, String::from("y")),
            (TokenType::RPAREN, String::from(")")),
            (TokenType::LBRACE, String::from("{")),
            (TokenType::IDENT, String::from("x")),
            (TokenType::PLUS, String::from("+")),
            (TokenType::IDENT, String::from("y")),
            (TokenType::SEMICOLON, String::from(";")),
            (TokenType::RBRACE, String::from("}")),
            (TokenType::SEMICOLON, String::from(";")),
            (TokenType::LET, String::from("let")),
            (TokenType::IDENT, String::from("result")),
            (TokenType::ASSIGN, String::from("=")),
            (TokenType::IDENT, String::from("add")),
            (TokenType::LPAREN, String::from("(")),
            (TokenType::IDENT, String::from("five")),
            (TokenType::COMMA, String::from(",")),
            (TokenType::IDENT, String::from("ten")),
            (TokenType::RPAREN, String::from(")")),
            (TokenType::SEMICOLON, String::from(";")),
            (TokenType::BANG, String::from("!")),
            (TokenType::MINUS, String::from("-")),
            (TokenType::SLASH, String::from("/")),
            (TokenType::ASTERISK, String::from("*")),
            (TokenType::INT, String::from("5")),
            (TokenType::SEMICOLON, String::from(";")),
            (TokenType::INT, String::from("5")),
            (TokenType::LT, String::from("<")),
            (TokenType::INT, String::from("10")),
            (TokenType::GT, String::from(">")),
            (TokenType::INT, String::from("5")),
            (TokenType::SEMICOLON, String::from(";")),
            (TokenType::IF, String::from("if")),
            (TokenType::LPAREN, String::from("(")),
            (TokenType::INT, String::from("5")),
            (TokenType::LT, String::from("<")),
            (TokenType::INT, String::from("10")),
            (TokenType::RPAREN, String::from(")")),
            (TokenType::LBRACE, String::from("{")),
            (TokenType::RETURN, String::from("return")),
            (TokenType::TRUE, String::from("true")),
            (TokenType::SEMICOLON, String::from(";")),
            (TokenType::RBRACE, String::from("}")),
            (TokenType::ELSE, String::from("else")),
            (TokenType::LBRACE, String::from("{")),
            (TokenType::RETURN, String::from("return")),
            (TokenType::FALSE, String::from("false")),
            (TokenType::SEMICOLON, String::from(";")),
            (TokenType::RBRACE, String::from("}")),
            (TokenType::INT, String::from("10")),
            (TokenType::EQ, String::from("==")),
            (TokenType::INT, String::from("10")),
            (TokenType::SEMICOLON, String::from(";")),
            (TokenType::INT, String::from("10")),
            (TokenType::NOT_EQ, String::from("!=")),
            (TokenType::INT, String::from("9")),
            (TokenType::SEMICOLON, String::from(";")),
            (TokenType::EOF, String::from("")),
        ];

        let mut lexer = Lexer::new(input);

        for (token_type, literal) in tests {
            let tok = lexer.next_token();

            assert_eq!(token_type, tok.token_type);
            assert_eq!(literal, tok.literal);
        }
    }
}
