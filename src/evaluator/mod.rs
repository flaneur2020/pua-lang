pub mod object;

use ast::*;
use self::object::*;

#[derive(Debug)]
pub struct Evaluator {}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {}
    }

    pub fn eval(&mut self, program: Program) -> Object {
        self.eval_block_stmt(program).unwrap_or(Object::Null)
    }

    fn eval_block_stmt(&mut self, stmts: BlockStmt) -> Option<Object> {
        stmts.into_iter().fold(None, |_, x| self.eval_stmt(x))
    }

    fn eval_stmt(&mut self, stmt: Stmt) -> Option<Object> {
        match stmt {
            Stmt::Expr(expr) => self.eval_expr(expr),
            _ => None,
        }
    }

    fn eval_expr(&mut self, expr: Expr) -> Option<Object> {
        match expr {
            Expr::Literal(literal) => self.eval_literal(literal),
            Expr::Prefix(prefix, right_expr) => if let Some(right) = self.eval_expr(*right_expr) {
                self.eval_prefix_expr(prefix, right)
            } else {
                None
            }
            Expr::Infix(infix, left_expr, right_expr) => {
                let left = self.eval_expr(*left_expr);
                let right = self.eval_expr(*right_expr);
                if left.is_some() && right.is_some() {
                    self.eval_infix_expr(infix, left.unwrap(), right.unwrap())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn eval_prefix_expr(&mut self, prefix: Prefix, right: Object) -> Option<Object> {
        match prefix {
            Prefix::Not => Some(self.eval_not_op_expr(right)),
            Prefix::Minus => self.eval_minus_prefix_op_expr(right),
            _ => None,
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

    fn eval_minus_prefix_op_expr(&mut self, right: Object) -> Option<Object> {
        match right {
            Object::Int(value) => Some(Object::Int(-value)),
            _ => None,
        }
    }

    fn eval_infix_expr(&mut self, infix: Infix, left: Object, right: Object) -> Option<Object> {
        match left {
            Object::Int(left_value) => if let Object::Int(right_value) = right {
                Some(self.eval_infix_int_expr(infix, left_value, right_value))
            } else {
                None
            },
            _ => None,
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
}

#[cfg(test)]
mod tests {
    use lexer::Lexer;
    use parser::Parser;
    use evaluator::*;
    use evaluator::object::*;

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
}
