mod enviroment;
mod object;

use enviroment::Environment;
pub use object::Object;
use thiserror::Error;

use crate::{
    lex::TokenType,
    parse::{Expr, Statement},
};

#[derive(Error, Debug)]
pub enum InterpreterError {
    #[error("Operand must be a {0}.")]
    TypeError(String),
    #[error("Undefined variable.")]
    UndefinedVariable,
}

pub struct Interpreter {
    env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            env: Environment::new(),
        }
    }
}

impl Interpreter {
    pub fn interpret(&mut self, stmt: &Statement) -> Result<(), InterpreterError> {
        match stmt {
            Statement::Expression(expr) => {
                self.evaluate(expr)?;
            }
            Statement::Print(expr) => {
                let value = self.evaluate(expr)?;
                println!("{}", self.stringify(&value));
            }
            Statement::Var(name, init) => {
                let value = match init {
                    Some(expr) => self.evaluate(expr)?,
                    None => Object::Nil,
                };
                self.env.define(name.clone(), value);
            }
        };

        Ok(())
    }

    fn stringify(&self, obj: &Object) -> String {
        if let Object::Number(n) = obj {
            n.to_string()
        } else {
            obj.to_string()
        }
    }
}

impl Interpreter {
    pub fn eval(&self, expr: &Expr) -> Result<(), InterpreterError> {
        let value = self.evaluate(expr)?;
        println!("{}", self.stringify(&value));

        Ok(())
    }

    fn evaluate(&self, expr: &Expr) -> Result<Object, InterpreterError> {
        match expr {
            Expr::Literal(obj) => Ok(obj.clone()),
            Expr::Unary(op, right) => self.eval_unary(&op.0, right),
            Expr::Binary(left, op, right) => self.eval_binary(left, &op.0, right),
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Variable(name) => self.env.get(name),
        }
    }

    fn eval_binary(
        &self,
        left: &Expr,
        op: &TokenType,
        right: &Expr,
    ) -> Result<Object, InterpreterError> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match (&left, op, &right) {
            (l, TokenType::EqualEqual, r) => Ok(Object::Boolean(eval_equal(l, r))),
            (l, TokenType::BangEqual, r) => Ok(Object::Boolean(!eval_equal(l, r))),
            (Object::Number(l), op, Object::Number(r)) => binary_number(l, op, r),
            (Object::String(l), TokenType::Plus, Object::String(r)) => {
                Ok(Object::String(format!("{}{}", l, r)))
            }
            _ => Err(InterpreterError::TypeError("number".to_string())),
        }
    }

    fn eval_unary(&self, op: &TokenType, right: &Expr) -> Result<Object, InterpreterError> {
        let right = self.evaluate(right)?;
        match (op, &right) {
            (TokenType::Minus, Object::Number(n)) => Ok(Object::Number(-n)),
            (TokenType::Bang, _) => Ok(Object::Boolean(!self.truthy(&right))),
            _ => Err(InterpreterError::TypeError("number".to_string())),
        }
    }

    fn truthy(&self, obj: &Object) -> bool {
        match obj {
            Object::Nil => false,
            Object::Boolean(b) => *b,
            _ => true,
        }
    }
}

fn eval_equal(left: &Object, right: &Object) -> bool {
    match (left, right) {
        (Object::Number(l), Object::Number(r)) => l == r,
        (Object::String(l), Object::String(r)) => l == r,
        (Object::Boolean(l), Object::Boolean(r)) => l == r,
        (Object::Nil, Object::Nil) => true,
        _ => false,
    }
}

fn binary_number(left: &f64, op: &TokenType, right: &f64) -> Result<Object, InterpreterError> {
    match op {
        TokenType::Plus => Ok(Object::Number(left + right)),
        TokenType::Minus => Ok(Object::Number(left - right)),
        TokenType::Star => Ok(Object::Number(left * right)),
        TokenType::Slash => Ok(Object::Number(left / right)),
        TokenType::Greater => Ok(Object::Boolean(left > right)),
        TokenType::GreaterEqual => Ok(Object::Boolean(left >= right)),
        TokenType::Less => Ok(Object::Boolean(left < right)),
        TokenType::LessEqual => Ok(Object::Boolean(left <= right)),
        TokenType::EqualEqual => Ok(Object::Boolean(left == right)),
        _ => Err(InterpreterError::TypeError("number".to_string())),
    }
}
