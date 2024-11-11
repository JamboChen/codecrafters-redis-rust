mod object;

pub use object::Object;

use crate::{lex::TokenType, parse::Expr};

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {}
    }
}

impl Interpreter {
    pub fn interpret(&self, expr: &Expr) -> Result<(), ()> {
        let value = self.evaluate(expr)?;
        println!("{}", self.stringify(&value));

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
    fn evaluate(&self, expr: &Expr) -> Result<Object, ()> {
        match expr {
            Expr::Literal(obj) => Ok(obj.clone()),
            Expr::Unary(op, right) => self.eval_unary(&op.0, right),
            Expr::Grouping(expr) => self.evaluate(expr),
            _ => todo!(),
        }
    }

    fn eval_unary(&self, op: &TokenType, right: &Expr) -> Result<Object, ()> {
        let right = self.evaluate(right)?;
        match (op, &right) {
            (TokenType::Minus, Object::Number(n)) => Ok(Object::Number(-n)),
            (TokenType::Bang, _) => Ok(Object::Boolean(!self.truthy(&right))),
            _ => Err(()),
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
