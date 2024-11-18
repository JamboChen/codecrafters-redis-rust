use std::fmt::Display;

use crate::lex::Token;

use super::{Environment, Interpreter, Object, RuntimeError, Statement};

pub trait LoxCallable {
    fn name(&self) -> &str;
    #[allow(dead_code)]
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &Interpreter, arguments: &[Object])
        -> Result<Object, RuntimeError>;
}

#[derive(Clone)]
pub struct LoxFunction {
    name: String,
    params: Vec<Token>,
    body: Vec<Statement>,
    closure: Environment,
}

impl LoxFunction {
    pub fn new(name: &str, params: &[Token], body: &[Statement], closure: &Environment) -> Self {
        LoxFunction {
            name: name.to_string(),
            params: params.to_vec(),
            body: body.to_vec(),
            closure: closure.clone(),
        }
    }
}

impl LoxCallable for LoxFunction {
    fn name(&self) -> &str {
        &self.name
    }

    fn arity(&self) -> usize {
        self.params.len()
    }

    fn call(
        &self,
        _interpreter: &Interpreter,
        arguments: &[Object],
    ) -> Result<Object, RuntimeError> {
        let env = Environment::new_enclosed(&self.closure);
        if self.params.len() != arguments.len() {
            return Err(RuntimeError::TypeError(format!(
                "Expected {} arguments but got {}.",
                self.params.len(),
                arguments.len()
            )));
        }
        for (param, arg) in self.params.iter().zip(arguments.iter()) {
            env.define(param.1.clone(), arg.clone());
        }

        let interpreter = Interpreter { env };
        for stmt in self.body.iter() {
            if let Some(result) = interpreter.interpret_stmt(stmt)? {
                return Ok(result);
            }
        }

        Ok(Object::Nil)
    }
}

pub(super) struct NativeFunction {
    pub name: String,
    pub func: fn() -> Result<Object, RuntimeError>,
}

impl LoxCallable for NativeFunction {
    fn name(&self) -> &str {
        &self.name
    }

    fn arity(&self) -> usize {
        0
    }

    fn call(
        &self,
        _interpreter: &Interpreter,
        _arguments: &[Object],
    ) -> Result<Object, RuntimeError> {
        (self.func)()
    }
}

impl Display for dyn LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name())
    }
}
