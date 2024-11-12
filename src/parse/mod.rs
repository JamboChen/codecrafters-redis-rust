use crate::{
    interpreter::Object,
    lex::{Token, TokenType},
};
pub use ast::{Expr, Statement};
use thiserror::Error;

mod ast;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("[line {0}] Error at '{1}': Expect expression.")]
    UnexpectedToken(usize, String),
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn parse(&mut self) -> (Vec<Statement>, Vec<ParserError>) {
        let mut stmts = Vec::new();
        let mut errors = Vec::new();

        while !self.is_at_end() {
            match self.declaration() {
                Ok(expr) => stmts.push(expr),
                Err(e) => errors.push(e),
            }
        }

        (stmts, errors)
    }

    pub fn parse_expr(&mut self) -> (Vec<Expr>, Vec<ParserError>) {
        let mut exprs = Vec::new();
        let mut errors = Vec::new();

        while !self.is_at_end() {
            match self.expression() {
                Ok(expr) => exprs.push(expr),
                Err(e) => errors.push(e),
            }
        }

        (exprs, errors)
    }
}

impl Parser {
    fn declaration(&mut self) -> Result<Statement, ParserError> {
        match self.peek().0 {
            TokenType::Var => self.var_decl(),
            _ => self.statement(),
        }
    }

    fn var_decl(&mut self) -> Result<Statement, ParserError> {
        self.expected(TokenType::Var)?;
        let name = self.expected(TokenType::Identifier)?.clone();
        let expr = if self.peek().0 == TokenType::Equal {
            self.next();
            Some(self.expression()?)
        } else {
            None
        };

        self.expected(TokenType::Semicolon)?;

        Ok(Statement::Var(name.1, expr))
    }

    fn statement(&mut self) -> Result<Statement, ParserError> {
        match self.peek().0 {
            TokenType::Print => self.print_statement(),
            TokenType::LeftBrace => self.block_statement(),
            TokenType::If => self.if_statment(),
            TokenType::While => self.while_statment(),
            TokenType::For => self.for_statment(),
            _ => self.expression_statement(),
        }
    }

    fn for_statment(&mut self) -> Result<Statement, ParserError> {
        self.expected(TokenType::For)?;
        self.expected(TokenType::LeftParen)?;
        let init = match self.peek().0 {
            TokenType::Semicolon => {
                self.pos += 1;
                None
            }
            TokenType::Var => Some(self.var_decl()?),
            _ => Some(self.expression_statement()?),
        };
        let condition = match self.peek().0 {
            TokenType::Semicolon => {
                self.pos += 1;
                Expr::Literal(Object::Boolean(true))
            }
            _ => self.expression()?,
        };
        self.expected(TokenType::Semicolon)?;
        let increment = match self.peek().0 {
            TokenType::RightParen => None,
            _ => Some(self.expression()?),
        };
        self.expected(TokenType::RightParen)?;

        let mut body = self.statement()?;
        if let Some(increment) = increment {
            body = Statement::Block(vec![body, Statement::Expression(increment)]);
        }

        body = Statement::While(condition, Box::new(body));

        if let Some(init) = init {
            body = Statement::Block(vec![init, body]);
        }

        Ok(body)
    }

    fn while_statment(&mut self) -> Result<Statement, ParserError> {
        self.expected(TokenType::While)?;
        self.expected(TokenType::LeftParen)?;
        let condition = self.expression()?;
        self.expected(TokenType::RightParen)?;
        let body = Box::new(self.statement()?);

        Ok(Statement::While(condition, body))
    }

    fn if_statment(&mut self) -> Result<Statement, ParserError> {
        self.expected(TokenType::If)?;
        self.expected(TokenType::LeftParen)?;
        let condition = self.expression()?;
        self.expected(TokenType::RightParen)?;
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.peek().0 == TokenType::Else {
            self.next();
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Statement::If(condition, then_branch, else_branch))
    }

    fn block_statement(&mut self) -> Result<Statement, ParserError> {
        self.expected(TokenType::LeftBrace)?;
        let mut stmts = Vec::new();

        while self.peek().0 != TokenType::RightBrace && !self.is_at_end() {
            stmts.push(self.declaration()?);
        }

        self.expected(TokenType::RightBrace)?;

        Ok(Statement::Block(stmts))
    }

    fn print_statement(&mut self) -> Result<Statement, ParserError> {
        self.expected(TokenType::Print)?;
        let expr = self.expression()?;
        self.expected(TokenType::Semicolon)?;
        Ok(Statement::Print(expr))
    }

    fn expression_statement(&mut self) -> Result<Statement, ParserError> {
        let expr = self.expression()?;
        self.expected(TokenType::Semicolon)?;
        Ok(Statement::Expression(expr))
    }
}

impl Parser {
    pub fn from_tokens(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn expression(&mut self) -> Result<Expr, ParserError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParserError> {
        let expr = self.logic_or()?;

        if self.peek().0 == TokenType::Equal {
            self.next();
            let value = self.assignment()?;
            match expr {
                Expr::Variable(name) => Ok(Expr::Assign(name, Box::new(value))),
                _ => Err(ParserError::UnexpectedToken(
                    self.peek().3,
                    self.peek().1.clone(),
                )),
            }
        } else {
            Ok(expr)
        }
    }

    fn logic_or(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.logic_and()?;
        while self.peek().0 == TokenType::Or {
            let operator = self.next().clone();
            let right = self.logic_and()?;
            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn logic_and(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.equality()?;
        while self.peek().0 == TokenType::And {
            let operator = self.next().clone();
            let right = self.equality()?;
            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.comparison()?;

        while let TokenType::EqualEqual | TokenType::BangEqual = self.peek().0 {
            let operator = self.next().clone();
            let right = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.term()?;

        while let TokenType::Greater
        | TokenType::GreaterEqual
        | TokenType::Less
        | TokenType::LessEqual
        | TokenType::EqualEqual
        | TokenType::BangEqual = self.peek().0
        {
            let operator = self.next().clone();
            let right = self.term()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.factor()?;

        while let TokenType::Minus | TokenType::Plus = self.peek().0 {
            let operator = self.next().clone();
            let right = self.factor()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.unary()?;

        while let TokenType::Star | TokenType::Slash = self.peek().0 {
            let operator = self.next().clone();
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParserError> {
        if let TokenType::Minus | TokenType::Bang = self.peek().0 {
            let operator = self.next().clone();
            let right = self.unary()?;
            Ok(Expr::Unary(operator, Box::new(right)))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, ParserError> {
        let peeked = self.next();
        let expr = match peeked.0 {
            TokenType::Number | TokenType::String => Expr::Literal(peeked.2.clone().unwrap()),
            TokenType::True => Expr::Literal(Object::Boolean(true)),
            TokenType::False => Expr::Literal(Object::Boolean(false)),
            TokenType::Nil => Expr::Literal(Object::Nil),
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.expected(TokenType::RightParen)?;
                Expr::Grouping(Box::new(expr))
            }
            TokenType::Identifier => Expr::Variable(peeked.1.to_string()),
            _ => return Err(ParserError::UnexpectedToken(peeked.3, peeked.1.to_string())),
        };

        Ok(expr)
    }

    /// Expects the next token to be of the given type.
    /// If not, rasies an error.
    fn expected(&mut self, token_type: TokenType) -> Result<&Token, ParserError> {
        if self.peek().0 == token_type {
            Ok(self.next())
        } else {
            Err(ParserError::UnexpectedToken(
                self.peek().3,
                self.peek().1.clone(),
            ))
        }
    }

    fn next(&mut self) -> &Token {
        self.pos += 1;
        &self.tokens[self.pos - 1]
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn is_at_end(&self) -> bool {
        self.peek().0 == TokenType::Eof
    }
}
