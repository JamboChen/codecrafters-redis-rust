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
            Expr::Grouping(expr) => self.evaluate(expr),
            _ => todo!(),
        }
    }
}
