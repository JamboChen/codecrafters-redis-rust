use crate::{
    interpreter::Object,
    lex::{Token, TokenType},
};
use expr::Expr;

mod expr;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn from_tokens(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> (Vec<Expr>, Vec<()>) {
        let mut exprs = Vec::new();
        let mut errors = Vec::new();

        while !self.is_at_end() {
            match self.expression() {
                Ok(expr) => exprs.push(expr),
                Err(_) => errors.push(()),
            }
        }

        (exprs, errors)
    }

    fn expression(&mut self) -> Result<Expr, ()> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ()> {
        let mut expr = self.comparison()?;

        while let TokenType::EqualEqual | TokenType::BangEqual = self.peek().0 {
            let operator = self.next().clone();
            let right = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ()> {
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

    fn term(&mut self) -> Result<Expr, ()> {
        let mut expr = self.factor()?;

        while let TokenType::Minus | TokenType::Plus = self.peek().0 {
            let operator = self.next().clone();
            let right = self.factor()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ()> {
        let mut expr = self.unary()?;

        while let TokenType::Star | TokenType::Slash = self.peek().0 {
            let operator = self.next().clone();
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ()> {
        // let peeked = self.next().clone();
        // match peeked.0 {
        //     TokenType::Minus => {
        //         let right = self.unary()?;
        //         Ok(Expr::Unary(peeked, Box::new(right)))
        //     }
        //     TokenType::Bang => {
        //         let right = self.unary()?;
        //         Ok(Expr::Unary(peeked, Box::new(right)))
        //     }
        //     _ => self.primary(),
        // }
        if let TokenType::Minus | TokenType::Bang = self.peek().0 {
            let operator = self.next().clone();
            let right = self.unary()?;
            Ok(Expr::Unary(operator, Box::new(right)))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, ()> {
        let peeked = self.next();
        let expr = match peeked.0 {
            TokenType::Number | TokenType::String => Expr::Literal(peeked.2.clone().unwrap()),
            TokenType::True => Expr::Literal(Object::Boolean(true)),
            TokenType::False => Expr::Literal(Object::Boolean(false)),
            TokenType::Nil => Expr::Literal(Object::Nil),
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.expected(TokenType::RightParen).unwrap();
                expr
            }
            _ => return Err(()),
        };

        Ok(expr)
    }

    /// Expects the next token to be of the given type.
    /// If not, rasies an error.
    fn expected(&mut self, token_type: TokenType) -> Result<(), ()> {
        if self.peek().0 == token_type {
            self.pos += 1;
            Ok(())
        } else {
            Err(())
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
        self.pos >= self.tokens.len()
    }
}
