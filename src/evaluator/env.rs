use evaluator::object::*;
use std::collections::HashMap;

#[derive(PartialEq, Clone, Debug)]
pub struct Env {
    store: HashMap<String, Object>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            store: HashMap::new(),
        }
    }

    pub fn get(&mut self, name: String) -> Option<Object> {
        match self.store.get(&name) {
            Some(value) => Some(value.clone()),
            None => None,
        }
    }

    pub fn set(&mut self, name: String, value: &Object) {
        self.store.insert(name, value.clone());
    }
}
