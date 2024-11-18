use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{RuntimeError, Object};

#[derive(Clone)]
pub(super) struct Environment {
    inner: Rc<RefCell<EnvironmentImpl>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            inner: Rc::new(RefCell::new(EnvironmentImpl::new())),
        }
    }

    pub fn new_enclosed(enclosing: &Environment) -> Self {
        Environment {
            inner: Rc::new(RefCell::new(EnvironmentImpl {
                values: HashMap::new(),
                enclosing: Some(enclosing.clone()),
            })),
        }
    }

    pub fn define(&self, name: String, value: Object) {
        self.inner.borrow_mut().define(name, value);
    }

    pub fn get(&self, name: &str) -> Result<Object, RuntimeError> {
        self.inner.borrow().get(name)
    }

    pub fn assign(&self, name: &str, value: Object) -> Result<(), RuntimeError> {
        self.inner.borrow_mut().assign(name, value)
    }
}

struct EnvironmentImpl {
    values: HashMap<String, Object>,
    enclosing: Option<Environment>,
}

impl EnvironmentImpl {
    fn new() -> Self {
        EnvironmentImpl {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    fn define(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }

    fn get(&self, name: &str) -> Result<Object, RuntimeError> {
        if let Some(value) = self.values.get(name) {
            Ok(value.clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.get(name)
        } else {
            Err(RuntimeError::UndefinedVariable(name.to_string()))
        }
    }

    fn assign(&mut self, name: &str, value: Object) -> Result<(), RuntimeError> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            Ok(())
        } else if let Some(enclosing) = &mut self.enclosing {
            enclosing.assign(name, value)
        } else {
            Err(RuntimeError::UndefinedVariable(name.to_string()))
        }
    }
}
