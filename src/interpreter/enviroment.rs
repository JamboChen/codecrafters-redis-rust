use std::collections::HashMap;

use super::{InterpreterError, Object};

pub(super) struct Environment {
    values: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Result<Object, InterpreterError> {
        self.values
            .get(name)
            .cloned()
            .ok_or(InterpreterError::UndefinedVariable(name.to_string()))
    }

    pub fn assign(&mut self, name: &str, value: Object) -> Result<(), InterpreterError> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            Ok(())
        } else {
            Err(InterpreterError::UndefinedVariable(name.to_string()))
        }
    }
}
