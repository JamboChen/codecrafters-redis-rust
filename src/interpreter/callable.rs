use std::fmt::Display;

use crate::lex::Token;

use super::{Environment, Interpreter, InterpreterError, Object, Statement};

pub trait LoxCallable {
    fn name(&self) -> &str;
    #[allow(dead_code)]
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &Interpreter,
        arguments: &[Object],
    ) -> Result<Object, InterpreterError>;
}

#[derive(Clone)]
pub struct LoxFunction {
    name: String,
    params: Vec<Token>,
    body: Vec<Statement>,
}

impl LoxFunction {
    pub fn new(name: &str, params: &[Token], body: &[Statement]) -> Self {
        LoxFunction {
            name: name.to_string(),
            params: params.to_vec(),
            body: body.to_vec(),
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
        interpreter: &Interpreter,
        arguments: &[Object],
    ) -> Result<Object, InterpreterError> {
        let env = Environment::new_enclosed(&interpreter.env);

        for (param, arg) in self.params.iter().zip(arguments.iter()) {
            env.define(param.1.clone(), arg.clone());
        }

        let interpreter = Interpreter { env };
        for stmt in self.body.iter() {
            if let Some(result) = interpreter.interpret(stmt)? {
                return Ok(result);
            }
        }

        Ok(Object::Nil)
    }
}

pub(super) struct NativeFunction {
    pub name: String,
    pub func: fn() -> Result<Object, InterpreterError>,
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
    ) -> Result<Object, InterpreterError> {
        (self.func)()
    }
}

impl Display for dyn LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name())
    }
}
