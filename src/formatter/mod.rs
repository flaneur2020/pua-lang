#![allow(clippy::if_same_then_else)]
use ast::*;
use lexer::unescape::escape_str;

struct FormatConfig {
    max_line_length: usize,
    max_hash_oneline: usize,
}

pub struct Formatter {
    indent: usize,
    column: usize,
    config: FormatConfig,
}

impl Formatter {
    pub fn new() -> Self {
        Formatter {
            indent: 0,
            column: 1,
            config: FormatConfig {
                max_line_length: 80,
                max_hash_oneline: 3,
            },
        }
    }

    fn infix_to_precedence(infix: &Infix) -> Precedence {
        match infix {
            Infix::Plus | Infix::Minus => Precedence::Sum,
            Infix::Multiply | Infix::Divide => Precedence::Product,
            Infix::LessThan | Infix::LessThanEqual => Precedence::LessGreater,
            Infix::GreaterThan | Infix::GreaterThanEqual => Precedence::LessGreater,
            Infix::Equal | Infix::NotEqual => Precedence::Equals,
        }
    }

    fn ignore_semicolon_expr(expr: &Expr) -> bool {
        match expr {
            &Expr::If {
                cond: _,
                consequence: _,
                alternative: _,
            }
            | &Expr::Func { params: _, body: _ } => true,
            _ => false,
        }
    }

    pub fn format(&mut self, program: Program) -> String {
        self.format_block_stmt(program)
    }

    fn indent_str(&self, offset: i32) -> String {
        let indent = self.indent as i32;
        let size = if indent >= offset { indent + offset } else { 0 };

        "  ".repeat(size as usize)
    }

    fn normalize_block_stmt(stmts: BlockStmt) -> BlockStmt {
        stmts
            .iter()
            .enumerate()
            .filter_map(|(i, x)| {
                if i == 0 && *x == Stmt::Blank {
                    None
                } else if i + 1 == stmts.len() && *x == Stmt::Blank {
                    None
                } else if i > 0 && *x == Stmt::Blank && stmts.get(i - 1) == Some(&Stmt::Blank) {
                    None
                } else {
                    Some(x.clone())
                }
            })
            .collect::<Vec<_>>()
    }

    fn format_block_stmt(&mut self, stmts: BlockStmt) -> String {
        let mut result = String::new();
        let list = Self::normalize_block_stmt(stmts);

        for (i, stmt) in list.into_iter().enumerate() {
            self.column = self.indent * 2 + 1;

            if i > 0 {
                result.push('\n');
            }

            let indent_str = if stmt == Stmt::Blank {
                String::new()
            } else {
                self.indent_str(0)
            };

            result.push_str(&format!("{}{}", indent_str, self.format_stmt(stmt)));
        }

        result
    }

    fn format_stmt(&mut self, stmt: Stmt) -> String {
        match stmt {
            Stmt::Let(ident, expr) => self.format_let_stmt(ident, expr),
            Stmt::Return(expr) => self.format_return_stmt(expr),
            Stmt::Expr(expr) => {
                if Self::ignore_semicolon_expr(&expr) {
                    self.format_expr(expr, Precedence::Lowest)
                } else {
                    format!("{};", self.format_expr(expr, Precedence::Lowest))
                }
            }
            Stmt::Blank => String::new(),
        }
    }

    fn format_let_stmt(&mut self, ident: Ident, expr: Expr) -> String {
        let ident_str = self.format_ident_expr(ident);
        let result = format!("let {} = ", ident_str);

        self.column += result.len();

        let expr_str = self.format_expr(expr, Precedence::Lowest);

        format!("{}{};", result, expr_str)
    }

    fn format_return_stmt(&mut self, expr: Expr) -> String {
        let result = String::from("return ");

        self.column += result.len();

        format!("{}{};", result, self.format_expr(expr, Precedence::Lowest))
    }

    fn format_expr(&mut self, expr: Expr, precedence: Precedence) -> String {
        match expr {
            Expr::Ident(ident) => self.format_ident_expr(ident),
            Expr::Literal(literal) => self.format_literal(literal),
            Expr::Prefix(prefix, right) => self.format_prefix_expr(prefix, right),
            Expr::Infix(infix, left, right) => {
                self.format_infix_expr(infix, left, right, precedence)
            }
            Expr::Index(left, index) => self.format_index_expr(left, index),
            Expr::If {
                cond,
                consequence,
                alternative,
            } => self.format_if_expr(cond, consequence, alternative),
            Expr::While { cond, consequence } => self.format_while_expr(cond, consequence),
            Expr::Func { params, body } => self.format_func_expr(params, body),
            Expr::Call { func, args } => self.format_call_expr(func, args),
        }
    }

    fn format_ident_expr(&mut self, ident: Ident) -> String {
        let Ident(ident_str) = ident;
        self.column += ident_str.len();
        ident_str
    }

    fn format_literal(&mut self, literal: Literal) -> String {
        match literal {
            Literal::Int(value) => self.format_int_literal(value),
            Literal::String(value) => self.format_string_literal(value),
            Literal::Bool(value) => self.format_bool_literal(value),
            Literal::Array(value) => self.format_array_literal(value, false),
            Literal::Hash(value) => self.format_hash_literal(value, false),
        }
    }

    fn format_int_literal(&mut self, value: i64) -> String {
        let result = value.to_string();
        self.column += result.len();
        result
    }

    fn format_string_literal(&mut self, value: String) -> String {
        let result = escape_str(&value);
        self.column += result.len();
        result
    }

    fn format_bool_literal(&mut self, value: bool) -> String {
        let result = value.to_string();
        self.column += result.len();
        result
    }

    fn format_array_literal(&mut self, arr: Vec<Expr>, wrap: bool) -> String {
        let mut result = String::new();
        let original = arr.clone();
        let total = original.len();

        if wrap {
            self.indent += 1;
        }

        for (i, expr) in arr.into_iter().enumerate() {
            let expr_str = self.format_expr(expr, Precedence::Lowest);

            if wrap {
                if i == 0 {
                    result.push('\n');
                } else {
                    result.push_str(",\n");
                }

                result.push_str(&format!("{}{}", self.indent_str(0), expr_str));

                if i + 1 == total {
                    self.indent -= 1;
                    result.push_str(&format!("\n{}", self.indent_str(0)));
                }
            } else if i > 0 {
                result.push_str(&format!(", {}", expr_str));
            } else {
                result.push_str(&expr_str);
            }
        }

        if !wrap && self.column + result.len() + 2 > self.config.max_line_length {
            return self.format_array_literal(original, true);
        }

        format!("[{}]", result)
    }

    fn format_hash_literal(&mut self, hash: Vec<(Expr, Expr)>, wrap: bool) -> String {
        let mut result = String::new();
        let original = hash.clone();
        let total = original.len();

        if !wrap && total > self.config.max_hash_oneline {
            return self.format_hash_literal(original, true);
        }

        if wrap {
            self.indent += 1;
        }

        for (i, (key, value)) in hash.into_iter().enumerate() {
            let key_str = self.format_expr(key, Precedence::Lowest);
            let value_str = self.format_expr(value, Precedence::Lowest);

            if wrap {
                if i > 0 {
                    result.push(',');
                }

                result.push_str(&format!(
                    "\n{}{}: {}",
                    self.indent_str(0),
                    key_str,
                    value_str
                ));

                if i + 1 == total {
                    self.indent -= 1;
                    result.push_str(&format!("\n{}", self.indent_str(0)));
                }
            } else {
                if i == 0 {
                    result.push(' ');
                } else {
                    result.push_str(", ");
                }

                result.push_str(&format!("{}: {}", key_str, value_str));

                if i + 1 == total {
                    result.push(' ');
                }
            }
        }

        if !wrap && self.column + result.len() > self.config.max_line_length {
            return self.format_hash_literal(original, true);
        }

        format!("{{{}}}", result)
    }

    fn format_infix_expr(
        &mut self,
        infix: Infix,
        left: Box<Expr>,
        right: Box<Expr>,
        precedence: Precedence,
    ) -> String {
        let current_precedence = Self::infix_to_precedence(&infix);
        let left_str = self.format_expr(*left, current_precedence.clone());
        let right_str = self.format_expr(*right, current_precedence.clone());

        if precedence > current_precedence {
            format!("({} {} {})", left_str, infix, right_str)
        } else {
            format!("{} {} {}", left_str, infix, right_str)
        }
    }

    fn format_prefix_expr(&mut self, prefix: Prefix, right: Box<Expr>) -> String {
        let right_str = self.format_expr(*right, Precedence::Prefix);

        format!("{}{}", prefix, right_str)
    }

    fn format_index_expr(&mut self, left: Box<Expr>, index: Box<Expr>) -> String {
        let left_str = self.format_expr(*left, Precedence::Lowest);
        let index_str = self.format_expr(*index, Precedence::Lowest);

        format!("{}[{}]", left_str, index_str)
    }

    fn format_if_expr(
        &mut self,
        cond: Box<Expr>,
        consequence: BlockStmt,
        alternative: Option<BlockStmt>,
    ) -> String {
        let cond_str = self.format_expr(*cond, Precedence::Lowest);

        self.indent += 1;

        let consequence_str = self.format_block_stmt(consequence);

        let result = match alternative {
            Some(alternative_expr) => {
                let alternative_str = self.format_block_stmt(alternative_expr);
                let indent_str = self.indent_str(-1);
                format!(
                    "if ({}) {{\n{}\n{}}} else {{\n{}\n{}}}",
                    cond_str, consequence_str, indent_str, alternative_str, indent_str,
                )
            }
            None => {
                let indent_str = self.indent_str(-1);
                format!(
                    "if ({}) {{\n{}\n{}}}",
                    cond_str, consequence_str, indent_str
                )
            }
        };

        self.indent -= 1;

        result
    }

    fn format_while_expr(&mut self, cond: Box<Expr>, consequence: BlockStmt) -> String {
        let cond_str = self.format_expr(*cond, Precedence::Lowest);
        self.indent += 1;

        let consequence_str = self.format_block_stmt(consequence);
        let indent_str = self.indent_str(-1);
        self.indent -= 1;

        let result = format!(
            "while ({}) {{\n{}\n{}}}",
            cond_str, consequence_str, indent_str
        );
        result
    }

    fn format_func_expr(&mut self, params: Vec<Ident>, body: BlockStmt) -> String {
        let mut params_str = String::new();

        for (i, param) in params.into_iter().enumerate() {
            if i > 0 {
                params_str.push_str(", ");
            }

            params_str.push_str(&self.format_ident_expr(param));
        }

        self.indent += 1;

        let body_str = self.format_block_stmt(body);

        self.indent -= 1;

        format!(
            "fn({}) {{\n{}\n{}}}",
            params_str,
            body_str,
            self.indent_str(0)
        )
    }

    fn format_call_expr(&mut self, func: Box<Expr>, args: Vec<Expr>) -> String {
        let func_str = self.format_expr(*func, Precedence::Lowest);
        let mut args_str = String::new();

        for (i, arg) in args.into_iter().enumerate() {
            if i > 0 {
                args_str.push_str(", ");
            }

            args_str.push_str(&self.format_expr(arg, Precedence::Lowest));
        }

        format!("{}({})", func_str, args_str)
    }
}

#[cfg(test)]
mod tests {
    use formatter::*;
    use lexer::*;
    use parser::*;

    fn format(input: &str) -> String {
        Formatter::new().format(Parser::new(Lexer::new(input)).parse())
    }

    #[test]
    fn test_blank() {
        let tests = vec![
            (
                r#"1000;

1000;


1000;



1000;"#,
                r#"1000;

1000;

1000;

1000;"#,
            ),
            (
                r#"if (x) {

  1000;


  1000;

}"#,
                r#"if (x) {
  1000;

  1000;
}"#,
            ),
        ];

        for (input, expect) in tests {
            assert_eq!(String::from(expect), format(input));
        }
    }

    #[test]
    fn test_literal() {
        let tests = vec![
            ("1000", "1000;"),
            ("\"foo\"", "\"foo\";"),
            ("true", "true;"),
            ("false", "false;"),
            (
                "[ 0 , 1  ,  \"str\",true  ,   false    ]",
                "[0, 1, \"str\", true, false];",
            ),
            (
                "[123456789, 123456789, 123456789, 123456789, 123456789, 123456789, 123456789, 123456789, 123456789]",
                r#"[
  123456789,
  123456789,
  123456789,
  123456789,
  123456789,
  123456789,
  123456789,
  123456789,
  123456789
];"#,
            ),
            (
                "[\"124567890124567890124567890124567890124567890124567890124567890124567890124567890\"]",
                r#"[
  "124567890124567890124567890124567890124567890124567890124567890124567890124567890"
];"#,
            ),
            (
                "{      \"key\"   : \"value\"}",
                "{ \"key\": \"value\" };",
            ),
            (
                "{1:1, 2:2, 3:3}",
                "{ 1: 1, 2: 2, 3: 3 };",
            ),
            (
                "{1:1, 2:2, 3:3, 4:4}",
                r#"{
  1: 1,
  2: 2,
  3: 3,
  4: 4
};"#,
            ),
            (
                "{\"123456789123456789123456789123456789123456789123456789123456789123456789\": true}",
                r#"{
  "123456789123456789123456789123456789123456789123456789123456789123456789": true
};"#,
            ),
        ];

        for (input, expect) in tests {
            assert_eq!(String::from(expect), format(input));
        }
    }

    #[test]
    fn test_let_stmt() {
        let tests = vec![
            ("let    foo= 1000", "let foo = 1000;"),
            (
                "let    test        =    \"string\"",
                "let test = \"string\";",
            ),
            (
                "let   hoge =[0,1, 2 ,3  ]",
                "let hoge = [0, 1, 2, 3];",
            ),
            (
                "let abcdefghij = [12345678, 12345678, 12345678, 12345678, 12345678, 12345678, 1234];",
                r#"let abcdefghij = [
  12345678,
  12345678,
  12345678,
  12345678,
  12345678,
  12345678,
  1234
];"#,
            ),
            (
                "let aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa = {\"fooo\": \"abcdefg\"};",
                r#"let aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa = {
  "fooo": "abcdefg"
};"#
            ),
        ];

        for (input, expect) in tests {
            assert_eq!(String::from(expect), format(input));
        }
    }

    #[test]
    fn test_return_stmt() {
        let tests = vec![
            ("return   100", "return 100;"),
            ("return [100,100]", "return [100, 100];"),
            ("return [\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"]", r#"return [
  "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
];"#),
        ];

        for (input, expect) in tests {
            assert_eq!(String::from(expect), format(input));
        }
    }

    #[test]
    fn test_operator() {
        let tests = vec![
            // infix
            ("1+1", "1 + 1;"),
            ("(2+    2)", "2 + 2;"),
            ("(2 + 2)   * 5", "(2 + 2) * 5;"),
            ("2/(5+5  )", "2 / (5 + 5);"),
            ("2   / 5+5  ", "2 / 5 + 5;"),
            // prefix
            ("-  5", "-5;"),
            ("! true", "!true;"),
            ("-(  4*    5   )", "-(4 * 5);"),
            ("!((10-2)  / 4)", "!((10 - 2) / 4);"),
        ];

        for (input, expect) in tests {
            assert_eq!(String::from(expect), format(input));
        }
    }

    #[test]
    fn test_index_expr() {
        let tests = vec![
            ("foo[ 0  ]", "foo[0];"),
            ("foo[   1*2 ]", "foo[1 * 2];"),
            ("foo [   \"key\" ]", "foo[\"key\"];"),
        ];

        for (input, expect) in tests {
            assert_eq!(String::from(expect), format(input));
        }
    }

    #[test]
    fn test_if_expr() {
        let tests = vec![
            (
                "if(x){x}",
                r#"if (x) {
  x;
}"#,
            ),
            (
                "if(x){true}else{false}",
                r#"if (x) {
  true;
} else {
  false;
}"#,
            ),
            (
                r#"if (x) {
  let arr = [123456789, 123456789, 123456789, 123456789, 123456789, 123456789, 123456789];
  let obj = {"keeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeey": "valueeeeeeeeeeeeeeeeeeeeeeee"};
}"#,
                r#"if (x) {
  let arr = [
    123456789,
    123456789,
    123456789,
    123456789,
    123456789,
    123456789,
    123456789
  ];
  let obj = {
    "keeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeey": "valueeeeeeeeeeeeeeeeeeeeeeee"
  };
}"#,
            ),
            (
                r#"if(x){
if(         y){if(z)        { z; }}
}"#,
                r#"if (x) {
  if (y) {
    if (z) {
      z;
    }
  }
}"#,
            ),
            (
                r#"if(x){if(y){if(z){z;}}else{if(z){z;}}}else{if(y){if(z){z;}}else{if(z){z; }
}
}"#,
                r#"if (x) {
  if (y) {
    if (z) {
      z;
    }
  } else {
    if (z) {
      z;
    }
  }
} else {
  if (y) {
    if (z) {
      z;
    }
  } else {
    if (z) {
      z;
    }
  }
}"#,
            ),
        ];

        for (input, expect) in tests {
            assert_eq!(String::from(expect), format(input));
        }
    }

    #[test]
    fn test_func_expr() {
        let tests = vec![
            (
                "fn (  x ){     x}",
                r#"fn(x) {
  x;
}"#,
            ),
            (
                r#"fn   (x)        {
fn    (y) {y;}
}"#,
                r#"fn(x) {
  fn(y) {
    y;
  }
}"#,
            ),
        ];

        for (input, expect) in tests {
            assert_eq!(String::from(expect), format(input));
        }
    }

    #[test]
    fn test_call_expr() {
        let tests = vec![
            ("foo(    x)", "foo(x);"),
            ("foo(1 *        1)", "foo(1 * 1);"),
            ("foo((2 * 2 * 2))", "foo(2 * 2 * 2);"),
            ("foo(x,y,z)", "foo(x, y, z);"),
            ("arr[  \"hoge\" ](x,y,z)", "arr[\"hoge\"](x, y, z);"),
        ];

        for (input, expect) in tests {
            assert_eq!(String::from(expect), format(input));
        }
    }

    #[test]
    fn test_block_stmt() {
        let tests = vec![(
            "1000; 1000; 1000;",
            r#"1000;
1000;
1000;"#,
        )];

        for (input, expect) in tests {
            assert_eq!(String::from(expect), format(input));
        }
    }
}
