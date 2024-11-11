mod object;

pub use object::Object;

use crate::parse::Expr;

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {}
    }
}

impl Interpreter {
    pub fn interpret(&self, expr: &Expr) -> Result<(), ()> {
        let value = self.evaluate(expr)?;
        println!("{}", value);

        Ok(())
    }
}

impl Interpreter {
    fn evaluate(&self, expr: &Expr) -> Result<Object, ()> {
        match expr {
            Expr::Literal(obj) => Ok(obj.clone()),
            _ => todo!(),
        }
    }
}
