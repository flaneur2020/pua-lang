use evaluator::object::*;
use std::collections::HashMap;

pub fn new_builtins() -> HashMap<String, Object> {
    let mut builtins = HashMap::new();
    builtins.insert(String::from("len"), Object::Builtin(monkey_len));
    builtins
}

fn monkey_len(args: Vec<Object>) -> Object {
    if args.len() != 1 {
        return Object::Error(String::from(format!(
            "wrong number of arguments. got={}, want=1",
            args.len(),
        )));
    }

    match &args[0] {
        Object::String(s) => Object::Int(s.len() as i64),
        o => Object::Error(String::from(format!(
            "argument to `len` not supported, got {}",
            o
        ))),
    }
}
