pub mod object;
pub mod env;

use ast::*;
use evaluator::object::*;
use evaluator::env::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct Evaluator {
    env: Rc<RefCell<Env>>,
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {
            env: Rc::new(RefCell::new(Env::new())),
        }
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
                Object::ReturnValue(value) => return *value,
                Object::Error(msg) => return Object::Error(msg),
                obj => result = obj,
            }
        }

        result
    }

    fn eval_block_stmt(&mut self, stmts: BlockStmt) -> Object {
        let mut result = Object::Null;

        for stmt in stmts {
            match self.eval_stmt(stmt) {
                Object::ReturnValue(value) => return Object::ReturnValue(value),
                Object::Error(msg) => return Object::Error(msg),
                obj => result = obj,
            }
        }

        result
    }

    fn eval_stmt(&mut self, stmt: Stmt) -> Object {
        match stmt {
            Stmt::Let(ident, expr) => {
                let value = self.eval_expr(expr);
                if Self::is_error(&value) {
                    value
                } else {
                    let Ident(name) = ident;
                    self.env.borrow_mut().set(name, &value);
                    value
                }
            },
            Stmt::Expr(expr) => self.eval_expr(expr),
            Stmt::Return(expr) => {
                let value = self.eval_expr(expr);
                if Self::is_error(&value) {
                    value
                } else {
                    Object::ReturnValue(Box::new(value))
                }
            },
        }
    }

    fn eval_expr(&mut self, expr: Expr) -> Object {
        match expr {
            Expr::Ident(ident) => self.eval_ident(ident),
            Expr::Literal(literal) => self.eval_literal(literal),
            Expr::Prefix(prefix, right_expr) => {
                let right = self.eval_expr(*right_expr);
                self.eval_prefix_expr(prefix, right)
            },
            Expr::Infix(infix, left_expr, right_expr) => {
                let left = self.eval_expr(*left_expr);
                let right = self.eval_expr(*right_expr);
                self.eval_infix_expr(infix, left, right)
            },
            Expr::If { cond, consequence, alternative } => self.eval_if_expr(*cond, consequence, alternative),
            Expr::Func { params, body } => Object::Func(params, body, Rc::clone(&self.env)),
            Expr::Call { func, args } => self.eval_call_expr(func, args),
        }
    }

    fn eval_ident(&mut self, ident: Ident) -> Object {
        let Ident(name) = ident;

        match self.env.borrow_mut().get(name.clone()) {
            Some(value) => value,
            None => Object::Error(String::from(format!("identifier not found: {}", name))),
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

    fn eval_literal(&mut self, literal: Literal) -> Object {
        match literal {
            Literal::Int(value) => Object::Int(value),
            Literal::Bool(value) => Object::Bool(value),
        }
    }

    fn eval_if_expr(&mut self, cond: Expr, consequence: BlockStmt, alternative: Option<BlockStmt>) -> Object {
        let cond = self.eval_expr(cond);

        if Self::is_truthy(cond) {
            self.eval_block_stmt(consequence)
        } else if let Some(alt) = alternative {
            self.eval_block_stmt(alt)
        } else {
            Object::Null
        }
    }

    fn eval_call_expr(&mut self, func: Box<Expr>, args: Vec<Expr>) -> Object {
        let (params, body, env) = match self.eval_expr(*func) {
            Object::Func(params, body, env) => (params, body, env),
            o => return Object::Error(format!("{} is not valid function", o)),
        };

        if params.len() != args.len() {
            return Object::Error(format!("wrong number of arguments: {} expected but {} given", params.len(), args.len()));
        }

        let args = args.iter().map(|e| self.eval_expr(e.clone())).collect::<Vec<_>>();
        let current_env = Rc::clone(&self.env);
        let mut scoped_env = Env::new_with_outer(Rc::clone(&env));
        let list = params.iter().zip(args.iter());
        for (_, (ident, o)) in list.enumerate() {
            let Ident(name) = ident.clone();
            scoped_env.set(name, o);
        }

        self.env = Rc::new(RefCell::new(scoped_env));

        let object = self.eval_block_stmt(body);

        self.env = current_env;

        object
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
    fn test_let_stmt() {
        let tests = vec![
            ("let a = 5; a;", Object::Int(5)),
            ("let a = 5 * 5; a;", Object::Int(25)),
            ("let a = 5; let b = a; b;", Object::Int(5)),
            ("let a = 5; let b = a; let c = a + b + 5; c;", Object::Int(15)),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }

    #[test]
    fn test_fn_object() {
        let input = "fn(x) { x + 2; };";

        assert_eq!(
            Object::Func(
                vec![Ident(String::from("x"))],
                vec![
                    Stmt::Expr(
                        Expr::Infix(
                            Infix::Plus,
                            Box::new(Expr::Ident(Ident(String::from("x")))),
                            Box::new(Expr::Literal(Literal::Int(2))),
                        ),
                    ),
                ],
                Rc::new(RefCell::new(Env::new())),
            ),
            eval(input),
        );
    }

    #[test]
    fn test_fn_application() {
        let tests = vec![
            ("let identity = fn(x) { x; }; identity(5);", Object::Int(5)),
            ("let identity = fn(x) { return x; }; identity(5);", Object::Int(5)),
            ("let double = fn(x) { x * 2; }; double(5);", Object::Int(10)),
            ("let add = fn(x, y) { x + y; }; add(5, 5);", Object::Int(10)),
            ("let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));", Object::Int(20)),
            ("fn(x) { x; }(5)", Object::Int(5)),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }

    #[test]
    fn test_closures() {
        let input = r#"
let newAdder = fn(x) {
  fn(y) { x + y };
}

let addTwo = newAdder(2);
addTwo(2);
        "#;

        assert_eq!(Object::Int(4), eval(input));
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
            ("foobar", Object::Error(String::from("identifier not found: foobar"))),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }
}
