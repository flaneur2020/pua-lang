use ast::*;
use evaluator::env::*;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(PartialEq, Clone, Debug)]
pub enum Object {
    Int(i64),
    String(String),
    Bool(bool),
    Func(Vec<Ident>, BlockStmt, Rc<RefCell<Env>>),
    Null,
    ReturnValue(Box<Object>),
    Error(String),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Object::Int(ref value) => write!(f, "{}", value),
            Object::String(ref value) => write!(f, "{}", value),
            Object::Bool(ref value) => write!(f, "{}", value),
            Object::Func(ref params, _, _) => {
                let mut param_string = String::new();
                for Ident(s) in params {
                    param_string.push_str(&s);
                }
                write!(f, "fn({}) {{ ... }}", param_string)
            }
            Object::Null => write!(f, "null"),
            Object::ReturnValue(ref value) => write!(f, "{}", value),
            Object::Error(ref value) => write!(f, "{}", value),
        }
    }
}
