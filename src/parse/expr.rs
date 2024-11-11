use std::fmt::Display;

use crate::{interpreter::Object, lex::Token};

pub enum Expr {
    Literal(Object),
    Unary(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Expr::Literal(obj) => obj.to_string(),
            Expr::Unary(token, right) => format!("({} {})", token.1, right),
            Expr::Binary(left, token, right) => format!("({} {} {})", token.0, left, right),
            Expr::Grouping(expr) => format!("(group {})", expr),
        };

        write!(f, "{}", output)
    }
}