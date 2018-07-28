extern crate monkey;

use monkey::evaluator::builtins::new_builtins;
use monkey::evaluator::env::Env;
use monkey::evaluator::object::Object;
use monkey::evaluator::Evaluator;
use monkey::lexer::Lexer;
use monkey::parser::Parser;
use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::{c_char, c_void};
use std::rc::Rc;

fn main() {}

extern "C" {
    fn print(input_ptr: *mut c_char);
}

fn internal_print(msg: &str) {
    unsafe {
        let c_str = CString::new(msg).unwrap();
        print(c_str.into_raw());
    }
}

#[no_mangle]
pub fn alloc(size: usize) -> *mut c_void {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    ptr as *mut c_void
}

#[no_mangle]
pub fn dealloc(ptr: *mut c_void, size: usize) {
    unsafe {
        let _buf = Vec::from_raw_parts(ptr, 0, size);
    }
}

#[no_mangle]
pub fn eval(input_ptr: *mut c_char) -> *mut c_char {
    let input = unsafe { CStr::from_ptr(input_ptr).to_string_lossy().into_owned() };

    let mut parser = Parser::new(Lexer::new(&input));
    let program = parser.parse();
    let errors = parser.get_errors();

    if errors.len() > 0 {
        let msg = errors
            .into_iter()
            .map(|e| format!("{}\n", e))
            .collect::<String>();

        return CString::new(msg).unwrap().into_raw();
    }

    let mut env = Env::from(new_builtins());

    env.set(
        String::from("puts"),
        &Object::Builtin(-1, |args| {
            for arg in args {
                internal_print(&format!("{}", arg));
            }
            Object::Null
        }),
    );

    let mut evaluator = Evaluator::new(Rc::new(RefCell::new(env)));
    let evaluated = evaluator.eval(program).unwrap_or(Object::Null);
    let output = format!("{}", evaluated);

    CString::new(output).unwrap().into_raw()
}
