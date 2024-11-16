use crate::lex::Token;

use super::{Environment, Interpreter, InterpreterError, Object, Statement};

pub trait LoxCallable {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &Interpreter,
        arguments: &Vec<Object>,
    ) -> Result<Object, InterpreterError>;
}

#[derive(Clone)]
pub struct LoxFunction {
    params: Vec<Token>,
    body: Vec<Statement>,
}

impl LoxFunction {
    pub fn new(params: &Vec<Token>, body: &Vec<Statement>) -> Self {
        LoxFunction {
            params: params.clone(),
            body: body.clone(),
        }
    }
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        self.params.len()
    }

    fn call(
        &self,
        interpreter: &Interpreter,
        arguments: &Vec<Object>,
    ) -> Result<Object, InterpreterError> {
        let env = Environment::new_enclosed(&interpreter.env);
        for (param, arg) in self.params.iter().zip(arguments.iter()) {
            env.define(param.1.clone(), arg.clone());
        }

        let interpreter = Interpreter { env };
        for stmt in self.body.iter() {
            interpreter.interpret(stmt)?;
        }

        Ok(Object::Nil)
    }
}

pub(super) struct NativeFunction {
    pub func: fn() -> Result<Object, InterpreterError>,
}

impl LoxCallable for NativeFunction {
    fn arity(&self) -> usize {
        0
    }

    fn call(
        &self,
        _interpreter: &Interpreter,
        _arguments: &Vec<Object>,
    ) -> Result<Object, InterpreterError> {
        (self.func)()
    }
}
