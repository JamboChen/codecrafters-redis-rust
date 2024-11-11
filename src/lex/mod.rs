mod token;

use std::{iter::Peekable, str::Chars};

use thiserror::Error;
pub use token::{Token, TokenType};

use crate::interpreter::Object;

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

        self.tokens.push(self.new_token(TokenType::Eof, ""));

        (self.tokens, self.error)
    }

    fn next_token(&mut self) -> Result<Option<Token>, TokenizerError> {
        let c = self.next().unwrap();
        let token = match c {
            '{' => self.new_token(TokenType::LeftBrace, "{"),
            '}' => self.new_token(TokenType::RightBrace, "}"),
            '(' => self.new_token(TokenType::LeftParen, "("),
            ')' => self.new_token(TokenType::RightParen, ")"),
            ',' => self.new_token(TokenType::Comma, ","),
            '.' => self.new_token(TokenType::Dot, "."),
            '-' => self.new_token(TokenType::Minus, "-"),
            '+' => self.new_token(TokenType::Plus, "+"),
            ';' => self.new_token(TokenType::Semicolon, ";"),
            '/' => {
                if let Some('/') = self.peek() {
                    self.skip_comment();
                    return Ok(None);
                }
                self.new_token(TokenType::Slash, "/")
            }
            '*' => self.new_token(TokenType::Star, "*"),
            '=' => self.combine_or('=', '=', TokenType::EqualEqual, TokenType::Equal),
            '!' => self.combine_or('!', '=', TokenType::BangEqual, TokenType::Bang),
            '<' => self.combine_or('<', '=', TokenType::LessEqual, TokenType::Less),
            '>' => self.combine_or('>', '=', TokenType::GreaterEqual, TokenType::Greater),
            '"' => self.match_string()?,
            n if n.is_ascii_digit() => self.match_number(n)?,
            s if s.is_alphabetic() || s == '_' => self.match_identifier(s)?,
            s if s.is_whitespace() => return Ok(None),
            _ => return Err(TokenizerError::UnexpectedCharacter(self.line, c)),
        };

        Ok(Some(token))
    }

    fn match_identifier(&mut self, curr: char) -> Result<Token, TokenizerError> {
        let mut identifier = String::new();
        identifier.push(curr);

        while let Some(&c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                identifier.push(c);
                self.next();
            } else {
                break;
            }
        }

        if let Some(token) = match_reserved(&identifier) {
            return Ok(self.new_token(token, &identifier));
        }
        Ok(self.new_token(TokenType::Identifier, &identifier))
    }

    fn match_number(&mut self, curr: char) -> Result<Token, TokenizerError> {
        let mut number = String::new();
        number.push(curr);

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                number.push(*c);
                self.next();
            } else {
                break;
            }
        }

        if let Some('.') = self.peek() {
            number.push('.');
            self.next();
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    number.push(*c);
                    self.next();
                } else {
                    break;
                }
            }
        }

        Ok(Token::new(
            TokenType::Number,
            number.clone(),
            Some(Object::Number(number.parse().unwrap())),
            self.line,
        ))
    }

    fn match_string(&mut self) -> Result<Token, TokenizerError> {
        let mut string = String::new();
        while let Some(c) = self.next() {
            if c == '"' {
                return Ok(Token::new(
                    TokenType::String,
                    format!("\"{}\"", string),
                    Some(Object::String(string)),
                    self.line,
                ));
            }

            if c == '\n' {
                self.line += 1;
            }

            string.push(c);
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
                self.new_token(combined, &format!("{}{}", curr, with))
            }
            _ => self.new_token(single, &curr.to_string()),
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

    fn new_token(&self, token_type: TokenType, lexeme: &str) -> Token {
        Token::new(token_type, lexeme.to_string(), None, self.line)
    }
}

fn match_reserved(identifier: &str) -> Option<TokenType> {
    let ident = match identifier {
        "and" => TokenType::And,
        "class" => TokenType::Class,
        "else" => TokenType::Else,
        "false" => TokenType::False,
        "for" => TokenType::For,
        "fun" => TokenType::Fun,
        "if" => TokenType::If,
        "nil" => TokenType::Nil,
        "or" => TokenType::Or,
        "print" => TokenType::Print,
        "return" => TokenType::Return,
        "super" => TokenType::Super,
        "this" => TokenType::This,
        "true" => TokenType::True,
        "var" => TokenType::Var,
        "while" => TokenType::While,
        _ => return None,
    };

    Some(ident)
}
