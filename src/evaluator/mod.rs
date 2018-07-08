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

    pub fn eval(&mut self, program: Program) -> Option<Object> {
        let mut result = None;

        for stmt in program {
            match self.eval_stmt(stmt) {
                Some(Object::ReturnValue(value)) => return Some(*value),
                Some(Object::Error(msg)) => return Some(Object::Error(msg)),
                obj => result = obj,
            }
        }

        result
    }

    fn eval_block_stmt(&mut self, stmts: BlockStmt) -> Object {
        let mut result = Object::Null;

        for stmt in stmts {
            match self.eval_stmt(stmt) {
                Some(Object::ReturnValue(value)) => return Object::ReturnValue(value),
                Some(Object::Error(msg)) => return Object::Error(msg),
                Some(obj) => result = obj,
                None => return Object::Null,
            }
        }

        result
    }

    fn eval_stmt(&mut self, stmt: Stmt) -> Option<Object> {
        match stmt {
            Stmt::Let(ident, expr) => {
                let value = match self.eval_expr(expr) {
                    Some(value) => value,
                    None => return None,
                };
                if Self::is_error(&value) {
                    Some(value)
                } else {
                    let Ident(name) = ident;
                    self.env.borrow_mut().set(name, &value);
                    None
                }
            },
            Stmt::Expr(expr) => self.eval_expr(expr),
            Stmt::Return(expr) => {
                let value = match self.eval_expr(expr) {
                    Some(value) => value,
                    None => return None,
                };
                if Self::is_error(&value) {
                    Some(value)
                } else {
                    Some(Object::ReturnValue(Box::new(value)))
                }
            },
        }
    }

    fn eval_expr(&mut self, expr: Expr) -> Option<Object> {
        match expr {
            Expr::Ident(ident) => Some(self.eval_ident(ident)),
            Expr::Literal(literal) => Some(self.eval_literal(literal)),
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
            Expr::Func { params, body } => Some(Object::Func(params, body, Rc::clone(&self.env))),
            Expr::Call { func, args } => Some(self.eval_call_expr(func, args)),
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

    fn eval_if_expr(&mut self, cond: Expr, consequence: BlockStmt, alternative: Option<BlockStmt>) -> Option<Object> {
        let cond = match self.eval_expr(cond) {
            Some(cond) => cond,
            None => return None,
        };

        if Self::is_truthy(cond) {
            Some(self.eval_block_stmt(consequence))
        } else if let Some(alt) = alternative {
            Some(self.eval_block_stmt(alt))
        } else {
            None
        }
    }

    fn eval_call_expr(&mut self, func: Box<Expr>, args: Vec<Expr>) -> Object {
        let (params, body, env) = match self.eval_expr(*func) {
            Some(Object::Func(params, body, env)) => (params, body, env),
            Some(o) => return Object::Error(format!("{} is not valid function", o)),
            None => return Object::Null,
        };

        if params.len() != args.len() {
            return Object::Error(format!("wrong number of arguments: {} expected but {} given", params.len(), args.len()));
        }

        let args = args
            .iter()
            .map(|e| self.eval_expr(e.clone()).unwrap_or(Object::Null))
            .collect::<Vec<_>>();

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

    fn eval(input: &str) -> Option<Object> {
        Evaluator::new().eval(Parser::new(Lexer::new(input)).parse())
    }

    #[test]
    fn test_int_expr() {
        let tests = vec![
            ("5", Some(Object::Int(5))),
            ("10", Some(Object::Int(10))),
            ("-5", Some(Object::Int(-5))),
            ("-10", Some(Object::Int(-10))),
            ("5 + 5 + 5 + 5 - 10", Some(Object::Int(10))),
            ("2 * 2 * 2 * 2 * 2", Some(Object::Int(32))),
            ("-50 + 100 + -50", Some(Object::Int(0))),
            ("5 * 2 + 10", Some(Object::Int(20))),
            ("5 + 2 * 10", Some(Object::Int(25))),
            ("20 + 2 * -10", Some(Object::Int(0))),
            ("50 / 2 * 2 + 10", Some(Object::Int(60))),
            ("2 * (5 + 10)", Some(Object::Int(30))),
            ("3 * 3 * 3 + 10", Some(Object::Int(37))),
            ("3 * (3 * 3) + 10", Some(Object::Int(37))),
            ("(5 + 10 * 2 + 15 / 3) * 2 + -10", Some(Object::Int(50))),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }

    #[test]
    fn test_boolean_expr() {
        let tests = vec![
            ("true", Some(Object::Bool(true))),
            ("false", Some(Object::Bool(false))),
            ("1 < 2", Some(Object::Bool(true))),
            ("1 > 2", Some(Object::Bool(false))),
            ("1 < 1", Some(Object::Bool(false))),
            ("1 > 1", Some(Object::Bool(false))),
            ("1 >= 1", Some(Object::Bool(true))),
            ("1 <= 1", Some(Object::Bool(true))),
            ("1 >= 2", Some(Object::Bool(false))),
            ("1 <= 1", Some(Object::Bool(true))),
            ("2 <= 1", Some(Object::Bool(false))),
            ("1 == 1", Some(Object::Bool(true))),
            ("1 != 1", Some(Object::Bool(false))),
            ("1 == 2", Some(Object::Bool(false))),
            ("1 != 2", Some(Object::Bool(true))),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }

    #[test]
    fn test_not_operator() {
        let tests = vec![
            ("!true", Some(Object::Bool(false))),
            ("!false", Some(Object::Bool(true))),
            ("!!true", Some(Object::Bool(true))),
            ("!!false", Some(Object::Bool(false))),
            ("!!5", Some(Object::Bool(true))),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }

    #[test]
    fn test_if_else_expr() {
        let tests = vec![
            ("if (true) { 10 }", Some(Object::Int(10))),
            ("if (false) { 10 }", None),
            ("if (1) { 10 }", Some(Object::Int(10))),
            ("if (1 < 2) { 10 }", Some(Object::Int(10))),
            ("if (1 > 2) { 10 }", None),
            ("if (1 > 2) { 10 } else { 20 }", Some(Object::Int(20))),
            ("if (1 < 2) { 10 } else { 20 }", Some(Object::Int(10))),
            ("if (1 <= 2) { 10 }", Some(Object::Int(10))),
            ("if (1 >= 2) { 10 }", None),
            ("if (1 >= 2) { 10 } else { 20 }", Some(Object::Int(20))),
            ("if (1 <= 2) { 10 } else { 20 }", Some(Object::Int(10))),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }

    #[test]
    fn test_return_stmt() {
        let tests = vec![
            ("return 10;", Some(Object::Int(10))),
            ("return 10; 9;", Some(Object::Int(10))),
            ("return 2 * 5; 9;", Some(Object::Int(10))),
            ("9; return 2 * 5; 9;", Some(Object::Int(10))),
            (r#"
if (10 > 1) {
  if (10 > 1) {
    return 10;
  }
  return 1;
}"#, Some(Object::Int(10))),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }

    #[test]
    fn test_let_stmt() {
        let tests = vec![
            ("let a = 5; a;", Some(Object::Int(5))),
            ("let a = 5 * 5; a;", Some(Object::Int(25))),
            ("let a = 5; let b = a; b;", Some(Object::Int(5))),
            ("let a = 5; let b = a; let c = a + b + 5; c;", Some(Object::Int(15))),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }

    #[test]
    fn test_fn_object() {
        let input = "fn(x) { x + 2; };";

        assert_eq!(
            Some(Object::Func(
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
            )),
            eval(input),
        );
    }

    #[test]
    fn test_fn_application() {
        let tests = vec![
            ("let identity = fn(x) { x; }; identity(5);", Some(Object::Int(5))),
            ("let identity = fn(x) { return x; }; identity(5);", Some(Object::Int(5))),
            ("let double = fn(x) { x * 2; }; double(5);", Some(Object::Int(10))),
            ("let add = fn(x, y) { x + y; }; add(5, 5);", Some(Object::Int(10))),
            ("let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));", Some(Object::Int(20))),
            ("fn(x) { x; }(5)", Some(Object::Int(5))),
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

        assert_eq!(Some(Object::Int(4)), eval(input));
    }

    #[test]
    fn test_error_handling() {
        let tests = vec![
            ("5 + true", Some(Object::Error(String::from("type mismatch: 5 + true")))),
            ("5 + true; 5;", Some(Object::Error(String::from("type mismatch: 5 + true")))),
            ("-true", Some(Object::Error(String::from("unknown operator: -true")))),
            ("5; true + false; 5;", Some(Object::Error(String::from("unknown operator: true + false")))),
            ("if (10 > 1) { true + false; }", Some(Object::Error(String::from("unknown operator: true + false")))),
            (r#"
if (10 > 1) {
  if (10 > 1) {
    return true + false;
  }
  return 1;
}"#, Some(Object::Error(String::from("unknown operator: true + false")))),
            ("foobar", Some(Object::Error(String::from("identifier not found: foobar")))),
        ];

        for (input, expect) in tests {
            assert_eq!(expect, eval(input));
        }
    }

// FIXME Someday, I want to run Z Combinator...
//     #[test]
//     fn test_z_combinator() {
//         let input = r#"
// let z = fn(f) {
//   return fn(x) {
//     return f(fn(y) {
//       return x(x)(y);
//     });
//   }(fn(x) {
//     return f(fn(y) {
//       return x(x)(y);
//     });
//   });
// };
//
// return z(fn(f) {
//   return fn(n) {
//     if (n == 0) {
//       1
//     } else {
//       n * f(n - 1)
//     }
//   };
// })(5);
//         "#;
//
//         assert_eq!(Some(Object::Int(120)), eval(input));
//     }
}
