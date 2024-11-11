use std::fmt::Display;

use crate::{interpreter::Object, lex::Token};

pub enum Expr {
    Literal(Object),
    Unary(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Variable(String),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Expr::Literal(obj) => obj.to_string(),
            Expr::Unary(token, right) => format!("({} {})", token.1, right),
            Expr::Binary(left, token, right) => format!("({} {} {})", token.1, left, right),
            Expr::Grouping(expr) => format!("(group {})", expr),
            Expr::Variable(name) => name.to_string(),
        };

        write!(f, "{}", output)
    }
}

pub enum Statement {
    Print(Expr),
    Expression(Expr),
    Var(String, Option<Expr>),
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Statement::Expression(expr) => format!("{}", expr),
            _ => "Not implemented".to_string(),
        };

        write!(f, "{}", output)
    }
}
