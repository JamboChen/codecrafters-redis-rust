mod token;

use std::{iter::Peekable, str::Chars};

pub use token::{Token, TokenType};

pub struct Tokenizer<'a> {
    source: Peekable<Chars<'a>>,
    tokens: Vec<Token>,

    line: usize,
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Tokenizer {
            source: source.chars().peekable(),
            tokens: Vec::new(),
            line: 1,
            pos: 0,
        }
    }

    pub fn tokenize(mut self) -> Vec<Token> {
        while !self.is_at_end() {
            let next_token = self.next_token().unwrap();
            self.tokens.push(next_token);
        }

        self.tokens
            .push(Token::new(TokenType::EOF, "".to_string(), None));

        self.tokens
    }

    fn next_token(&mut self) -> Result<Token, ()> {
        let c = self.next().unwrap();
        let token = match c {
            '{' => Token::new(TokenType::LeftBrace, "{".to_string(), None),
            '}' => Token::new(TokenType::RightBrace, "}".to_string(), None),
            '(' => Token::new(TokenType::LeftParen, "(".to_string(), None),
            ')' => Token::new(TokenType::RightParen, ")".to_string(), None),
            ',' => Token::new(TokenType::Comma, ",".to_string(), None),
            '.' => Token::new(TokenType::Dot, ".".to_string(), None),
            '-' => Token::new(TokenType::Minus, "-".to_string(), None),
            '+' => Token::new(TokenType::Plus, "+".to_string(), None),
            ';' => Token::new(TokenType::Semicolon, ";".to_string(), None),
            '/' => Token::new(TokenType::Slash, "/".to_string(), None),
            '*' => Token::new(TokenType::Star, "*".to_string(), None),
            _ => todo!(),
        };

        Ok(token)
    }

    fn is_at_end(&mut self) -> bool {
        self.source.peek().is_none()
    }

    fn next(&mut self) -> Option<char> {
        self.pos += 1;
        self.source.next()
    }

    fn peek(&mut self) -> Option<&char> {
        self.source.peek()
    }
}
