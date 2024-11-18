mod callable;
mod enviroment;
mod object;

use std::{rc::Rc, time::SystemTime};

use callable::NativeFunction;
use enviroment::Environment;
pub use object::Object;
use thiserror::Error;

use crate::{
    lex::{Token, TokenType},
    parse::{Expr, Statement},
};

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("{0}")]
    TypeError(String),
    #[error("Undefined variable '{0}'.")]
    UndefinedVariable(String),
}

pub struct Interpreter {
    env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        let env = Environment::new();
        env.define(
            "clock".to_string(),
            Object::Callable(Rc::new(NativeFunction {
                name: "clock".to_string(),
                func: || {
                    Ok(Object::Number(
                        SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as f64,
                    ))
                },
            })),
        );
        Interpreter { env }
    }
}

impl Interpreter {
    pub fn interpret(&self, stmts: &[Statement]) -> Result<Option<Object>, RuntimeError> {
        for stmt in stmts {
            if let Some(value) = self.interpret_stmt(stmt)? {
                return Ok(Some(value));
            }
        }

        Ok(None)
    }

    fn interpret_stmt(&self, stmt: &Statement) -> Result<Option<Object>, RuntimeError> {
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
            Statement::Block(stmts) => {
                let env = Environment::new_enclosed(&self.env);
                let interpreter = Interpreter { env };
                match interpreter.interpret(stmts)? {
                    Some(value) => return Ok(Some(value)),
                    None => {}
                }
            }
            Statement::If(cond, then_branch, else_branch) => {
                let cond = self.evaluate(cond)?;
                let value = if self.truthy(&cond) {
                    self.interpret_stmt(then_branch)?
                } else if let Some(else_branch) = else_branch {
                    self.interpret_stmt(else_branch)?
                } else {
                    None
                };

                if let Some(value) = value {
                    return Ok(Some(value));
                }
            }
            Statement::While(cond, body) => {
                while self.truthy(&self.evaluate(cond)?) {
                    if let Some(value) = self.interpret_stmt(body)? {
                        return Ok(Some(value));
                    }
                }
            }
            Statement::Function(name, params, body) => {
                let func = callable::LoxFunction::new(name, params, body, &self.env);
                self.env
                    .define(name.clone(), Object::Callable(Rc::new(func)));
            }
            Statement::Return(_, expr) => {
                let value = match expr {
                    Some(expr) => self.evaluate(expr)?,
                    None => Object::Nil,
                };
                return Ok(Some(value));
            }
        };

        Ok(None) // no return value
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
    pub fn eval(&self, expr: &Expr) -> Result<(), RuntimeError> {
        let value = self.evaluate(expr)?;
        println!("{}", self.stringify(&value));

        Ok(())
    }

    fn evaluate(&self, expr: &Expr) -> Result<Object, RuntimeError> {
        match expr {
            Expr::Literal(obj) => Ok(obj.clone()),
            Expr::Unary(op, right) => self.eval_unary(&op.0, right),
            Expr::Binary(left, op, right) => self.eval_binary(left, &op.0, right),
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Variable(name) => self.env.get(name),
            Expr::Assign(name, value) => {
                let value = self.evaluate(value)?;
                self.env.assign(name, value.clone())?;
                Ok(value)
            }
            Expr::Logical(left, op, right) => self.eval_logical(left, &op.0, right),
            Expr::Call(callee, paren, args) => self.eval_call(callee, paren, args),
        }
    }

    fn eval_call(
        &self,
        callee: &Expr,
        _paren: &Token,
        args: &Vec<Expr>,
    ) -> Result<Object, RuntimeError> {
        let callee = self.evaluate(callee)?;
        let mut arguments = Vec::new();
        for arg in args {
            arguments.push(self.evaluate(arg)?);
        }

        let Object::Callable(callable) = callee else {
            return Err(RuntimeError::TypeError("callable".to_string()));
        };

        callable.call(self, &arguments)
    }

    fn eval_logical(
        &self,
        left: &Expr,
        op: &TokenType,
        right: &Expr,
    ) -> Result<Object, RuntimeError> {
        let left = self.evaluate(left)?;
        let left_truthy = self.truthy(&left);

        match (left_truthy, op) {
            (true, TokenType::Or) => Ok(left),
            (false, TokenType::Or) => Ok(self.evaluate(right)?),
            (true, TokenType::And) => Ok(self.evaluate(right)?),
            (false, TokenType::And) => Ok(left),
            _ => unreachable!(),
        }
    }
    fn eval_binary(
        &self,
        left: &Expr,
        op: &TokenType,
        right: &Expr,
    ) -> Result<Object, RuntimeError> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match (&left, op, &right) {
            (l, TokenType::EqualEqual, r) => Ok(Object::Boolean(eval_equal(l, r))),
            (l, TokenType::BangEqual, r) => Ok(Object::Boolean(!eval_equal(l, r))),
            (Object::Number(l), op, Object::Number(r)) => binary_number(l, op, r),
            (Object::String(l), TokenType::Plus, Object::String(r)) => {
                Ok(Object::String(format!("{}{}", l, r)))
            }
            _ => Err(RuntimeError::TypeError("Operand must be a number.".to_string())),
        }
    }

    fn eval_unary(&self, op: &TokenType, right: &Expr) -> Result<Object, RuntimeError> {
        let right = self.evaluate(right)?;
        match (op, &right) {
            (TokenType::Minus, Object::Number(n)) => Ok(Object::Number(-n)),
            (TokenType::Bang, _) => Ok(Object::Boolean(!self.truthy(&right))),
            _ => Err(RuntimeError::TypeError("Operand must be a number.".to_string())),
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

fn binary_number(left: &f64, op: &TokenType, right: &f64) -> Result<Object, RuntimeError> {
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

        _ => Err(RuntimeError::TypeError("Operand must be a number.".to_string())),
    }
}
