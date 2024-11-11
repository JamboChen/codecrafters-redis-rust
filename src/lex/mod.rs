mod token;

use std::{iter::Peekable, str::Chars};

use thiserror::Error;
pub use token::{Token, TokenType};

#[derive(Error, Debug)]
pub enum TokenizerError {
    #[error("[line {0}] Error: Unexpected character: {1}")]
    UnexpectedCharacter(usize, char),
}

pub struct Tokenizer<'a> {
    source: Peekable<Chars<'a>>,
    tokens: Vec<Token>,
    error: Vec<TokenizerError>,
    line: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Tokenizer {
            source: source.chars().peekable(),
            tokens: Vec::new(),
            error: Vec::new(),
            line: 1,
        }
    }

    pub fn tokenize(mut self) -> (Vec<Token>, Vec<TokenizerError>) {
        while !self.is_at_end() {
            match self.next_token() {
                Ok(token) => self.tokens.push(token),
                Err(e) => self.error.push(e),
            }
        }

        self.tokens
            .push(Token::new(TokenType::EOF, "".to_string(), None));

        (self.tokens, self.error)
    }

    fn next_token(&mut self) -> Result<Token, TokenizerError> {
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
            '=' => self.match_equal()?,
            _ => return Err(TokenizerError::UnexpectedCharacter(self.line, c)),
        };

        Ok(token)
    }

    fn match_equal(&mut self) -> Result<Token, TokenizerError> {
        match self.peek() {
            Some('=') => {
                self.next();
                Ok(Token::new(TokenType::EqualEqual, "==".to_string(), None))
            }
            _ => Ok(Token::new(TokenType::Equal, "=".to_string(), None)),
        }
    }

    fn is_at_end(&mut self) -> bool {
        self.source.peek().is_none()
    }

    fn next(&mut self) -> Option<char> {
        let c = self.source.next();
        if c == Some('\n') {
            self.line += 1;
        }
        c
    }

    fn peek(&mut self) -> Option<&char> {
        self.source.peek()
    }
}
