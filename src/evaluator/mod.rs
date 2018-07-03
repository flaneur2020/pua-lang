pub mod object;

use ast::*;
use self::object::*;

#[derive(Debug)]
pub struct Evaluator {}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {}
    }

    fn is_truthy(obj: Object) -> bool {
        match obj {
            Object::Null | Object::Bool(false) => false,
            _ => true,
        }
    }

    fn error(msg: String) -> Object {
        Object::Error(msg)
    }

    fn is_error(obj: &Object) -> bool {
        match obj {
            Object::Error(_) => true,
            _ => false,
        }
    }

    pub fn eval(&mut self, program: Program) -> Object {
        let mut result = Object::Null;

        for stmt in program {
            match self.eval_stmt(stmt) {
                Some(Object::ReturnValue(value)) => return *value,
                Some(Object::Error(msg)) => return Object::Error(msg),
                obj => result = obj.unwrap_or(Object::Null),
            }
        }

        result
    }

    fn eval_block_stmt(&mut self, stmts: BlockStmt) -> Option<Object> {
        let mut result = None;

        for stmt in stmts {
            match self.eval_stmt(stmt) {
                Some(Object::ReturnValue(value)) => return Some(Object::ReturnValue(value)),
                Some(Object::Error(msg)) => return Some(Object::Error(msg)),
                obj => result = obj,
            }
        }

        result
    }

    fn eval_stmt(&mut self, stmt: Stmt) -> Option<Object> {
        match stmt {
            Stmt::Expr(expr) => self.eval_expr(expr),
            Stmt::Return(expr) => {
                if let Some(value) = self.eval_expr(expr) {
                    if Self::is_error(&value) {
                        Some(value)
                    } else {
                        Some(Object::ReturnValue(Box::new(value)))
                    }
                } else {
                    None
                }
            },
            _ => None,
        }
    }

    fn eval_expr(&mut self, expr: Expr) -> Option<Object> {
        match expr {
            Expr::Literal(literal) => self.eval_literal(literal),
            Expr::Prefix(prefix, right_expr) => if let Some(right) = self.eval_expr(*right_expr) {
                Some(self.eval_prefix_expr(prefix, right))
            } else {
                None
            },
            Expr::Infix(infix, left_expr, right_expr) => {
                let left = self.eval_expr(*left_expr);
                let right = self.eval_expr(*right_expr);
                if left.is_some() && right.is_some() {
                    Some(self.eval_infix_expr(infix, left.unwrap(), right.unwrap()))
                } else {
                    None
                }
            },
            Expr::If { cond, consequence, alternative } => self.eval_if_expr(*cond, consequence, alternative),
            _ => None,
        }
    }

    fn eval_prefix_expr(&mut self, prefix: Prefix, right: Object) -> Object {
        match prefix {
            Prefix::Not => self.eval_not_op_expr(right),
            Prefix::Minus => self.eval_minus_prefix_op_expr(right),
            _ => Self::error(String::from(format!("unknown operator: {} {}", prefix, right))),
        }
    }

    fn eval_not_op_expr(&mut self, right: Object) -> Object {
        match right {
            Object::Bool(true) => Object::Bool(false),
            Object::Bool(false) => Object::Bool(true),
            Object::Null => Object::Bool(true),
            _ => Object::Bool(false),
        }
    }

    fn eval_minus_prefix_op_expr(&mut self, right: Object) -> Object {
        match right {
            Object::Int(value) => Object::Int(-value),
            _ => Self::error(String::from(format!("unknown operator: -{}", right))),
        }
    }

    fn eval_infix_expr(&mut self, infix: Infix, left: Object, right: Object) -> Object {
        match left {
            Object::Int(left_value) => if let Object::Int(right_value) = right {
                self.eval_infix_int_expr(infix, left_value, right_value)
            } else {
                Self::error(String::from(format!("type mismatch: {} {} {}", left, infix, right)))
            },
            _ => Self::error(String::from(format!("unknown operator: {} {} {}", left, infix, right))),
        }
    }

    fn eval_infix_int_expr(&mut self, infix: Infix, left: i64, right: i64) -> Object {
        match infix {
            Infix::Plus => Object::Int(left + right),
            Infix::Minus => Object::Int(left - right),
            Infix::Multiply => Object::Int(left * right),
            Infix::Divide => Object::Int(left / right),
            Infix::LessThan => Object::Bool(left < right),
            Infix::LessThanEqual => Object::Bool(left <= right),
            Infix::GreaterThan => Object::Bool(left > right),
            Infix::GreaterThanEqual => Object::Bool(left >= right),
            Infix::Equal => Object::Bool(left == right),
            Infix::NotEqual => Object::Bool(left != right),
        }
    }

    fn eval_literal(&mut self, literal: Literal) -> Option<Object> {
        match literal {
            Literal::Int(value) => Some(Object::Int(value)),
            Literal::Bool(value) => Some(Object::Bool(value)),
        }
    }

    fn eval_if_expr(&mut self, cond: Expr, consequence: BlockStmt, alternative: Option<BlockStmt>) -> Option<Object> {
        let cond = match self.eval_expr(cond) {
            Some(cond) => cond,
            None => return None,
        };

        if Self::is_truthy(cond) {
            self.eval_block_stmt(consequence)
        } else if let Some(alt) = alternative {
            self.eval_block_stmt(alt)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use lexer::Lexer;
    use parser::Parser;
    use evaluator::*;

    fn eval(input: &str) -> Object {
        Evaluator::new().eval(Parser::new(Lexer::new(input)).parse())
    }

    #[test]
    fn test_int_expr() {
        let tests = vec![
            ("5", Object::Int(5)),
            ("10", Object::Int(10)),
            ("-5", Object::Int(-5)),
            ("-10", Object::Int(-10)),
            ("5 + 5 + 5 + 5 - 10", Object::Int(10)),
            ("2 * 2 * 2 * 2 * 2", Object::Int(32)),
            ("-50 + 100 + -50", Object::Int(0)),
            ("5 * 2 + 10", Object::Int(20)),
            ("5 + 2 * 10", Object::Int(25)),
            ("20 + 2 * -10", Object::Int(0)),
            ("50 / 2 * 2 + 10", Object::Int(60)),
            ("2 * (5 + 10)", Object::Int(30)),
            ("3 * 3 * 3 + 10", Object::Int(37)),
            ("3 * (3 * 3) + 10", Object::Int(37)),
            ("(5 + 10 * 2 + 15 / 3) * 2 + -10", Object::Int(50)),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }

    #[test]
    fn test_boolean_expr() {
        let tests = vec![
            ("true", Object::Bool(true)),
            ("false", Object::Bool(false)),
            ("1 < 2", Object::Bool(true)),
            ("1 > 2", Object::Bool(false)),
            ("1 < 1", Object::Bool(false)),
            ("1 > 1", Object::Bool(false)),
            ("1 >= 1", Object::Bool(true)),
            ("1 <= 1", Object::Bool(true)),
            ("1 >= 2", Object::Bool(false)),
            ("1 <= 1", Object::Bool(true)),
            ("2 <= 1", Object::Bool(false)),
            ("1 == 1", Object::Bool(true)),
            ("1 != 1", Object::Bool(false)),
            ("1 == 2", Object::Bool(false)),
            ("1 != 2", Object::Bool(true)),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }

    #[test]
    fn test_not_operator() {
        let tests = vec![
            ("!true", Object::Bool(false)),
            ("!false", Object::Bool(true)),
            ("!!true", Object::Bool(true)),
            ("!!false", Object::Bool(false)),
            ("!!5", Object::Bool(true)),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }

    #[test]
    fn test_if_else_expr() {
        let tests = vec![
            ("if (true) { 10 }", Object::Int(10)),
            ("if (false) { 10 }", Object::Null),
            ("if (1) { 10 }", Object::Int(10)),
            ("if (1 < 2) { 10 }", Object::Int(10)),
            ("if (1 > 2) { 10 }", Object::Null),
            ("if (1 > 2) { 10 } else { 20 }", Object::Int(20)),
            ("if (1 < 2) { 10 } else { 20 }", Object::Int(10)),
            ("if (1 <= 2) { 10 }", Object::Int(10)),
            ("if (1 >= 2) { 10 }", Object::Null),
            ("if (1 >= 2) { 10 } else { 20 }", Object::Int(20)),
            ("if (1 <= 2) { 10 } else { 20 }", Object::Int(10)),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }

    #[test]
    fn test_return_stmt() {
        let tests = vec![
            ("return 10;", Object::Int(10)),
            ("return 10; 9;", Object::Int(10)),
            ("return 2 * 5; 9;", Object::Int(10)),
            ("9; return 2 * 5; 9;", Object::Int(10)),
            (r#"
if (10 > 1) {
  if (10 > 1) {
    return 10;
  }
  return 1;
}"#, Object::Int(10)),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }

    #[test]
    fn test_error_handling() {
        let tests = vec![
            ("5 + true", Object::Error(String::from("type mismatch: 5 + true"))),
            ("5 + true; 5;", Object::Error(String::from("type mismatch: 5 + true"))),
            ("-true", Object::Error(String::from("unknown operator: -true"))),
            ("5; true + false; 5;", Object::Error(String::from("unknown operator: true + false"))),
            ("if (10 > 1) { true + false; }", Object::Error(String::from("unknown operator: true + false"))),
            (r#"
if (10 > 1) {
  if (10 > 1) {
    return true + false;
  }
  return 1;
}"#, Object::Error(String::from("unknown operator: true + false"))),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }
}
