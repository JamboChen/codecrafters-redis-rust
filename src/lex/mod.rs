mod token;

use std::{iter::Peekable, str::Chars};

use thiserror::Error;
pub use token::{Token, TokenType};

use crate::parse::Object;

#[derive(Error, Debug)]
pub enum TokenizerError {
    #[error("[line {0}] Error: Unexpected character: {1}")]
    UnexpectedCharacter(usize, char),
    #[error("[line {0}] Error: Unterminated string.")]
    UnexpectedString(usize),
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
                Ok(Some(token)) => self.tokens.push(token),
                Ok(None) => continue,
                Err(e) => self.error.push(e),
            }
        }

        self.tokens
            .push(Token::new(TokenType::EOF, "".to_string(), None));

        (self.tokens, self.error)
    }

    fn next_token(&mut self) -> Result<Option<Token>, TokenizerError> {
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
            '/' => {
                if let Some('/') = self.peek() {
                    self.skip_comment();
                    return Ok(None);
                }
                Token::new(TokenType::Slash, "/".to_string(), None)
            }
            '*' => Token::new(TokenType::Star, "*".to_string(), None),
            '=' => self.combine_or('=', '=', TokenType::EqualEqual, TokenType::Equal),
            '!' => self.combine_or('!', '=', TokenType::BangEqual, TokenType::Bang),
            '<' => self.combine_or('<', '=', TokenType::LessEqual, TokenType::Less),
            '>' => self.combine_or('>', '=', TokenType::GreaterEqual, TokenType::Greater),
            '"' => self.match_string()?,
            s if s.is_whitespace() => return Ok(None),
            _ => return Err(TokenizerError::UnexpectedCharacter(self.line, c)),
        };

        Ok(Some(token))
    }

    fn match_string(&mut self) -> Result<Token, TokenizerError> {
        let mut string = String::new();
        while !self.is_at_end() {
            match self.next() {
                Some('"') => {
                    return Ok(Token::new(
                        TokenType::String,
                        format!("\"{}\"", string),
                        Some(Object::String(string)),
                    ))
                }
                Some(c) => string.push(c),
                None => return Err(TokenizerError::UnexpectedCharacter(self.line, '"')),
            }
        }

        Err(TokenizerError::UnexpectedString(self.line))
    }

    fn skip_comment(&mut self) {
        while let Some(c) = self.next() {
            if c == '\n' {
                break;
            }
        }
    }

    fn combine_or(
        &mut self,
        curr: char,
        with: char,
        combined: TokenType,
        single: TokenType,
    ) -> Token {
        match self.peek() {
            Some(c) if *c == with => {
                self.next();
                Token::new(combined, format!("{}{}", curr, with), None)
            }
            _ => Token::new(single, curr.to_string(), None),
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
