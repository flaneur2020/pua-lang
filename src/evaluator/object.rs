use ast::*;
use evaluator::env::*;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(PartialEq, Clone, Debug)]
pub enum Object {
    Int(i64),
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
            Object::Bool(ref value) => write!(f, "{}", value),
            Object::Func(_, _, _) => write!(f, "fn()"),
            Object::Null => write!(f, "null"),
            Object::ReturnValue(ref value) => write!(f, "{}", value),
            Object::Error(ref value) => write!(f, "{}", value),
        }
    }
}
