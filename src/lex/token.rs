use std::fmt::Display;

use crate::parse::Object;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum TokenType {
    // Single-character tokens.
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    EOF,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            TokenType::LeftBrace => "LEFT_BRACE",
            TokenType::RightBrace => "RIGHT_BRACE",
            TokenType::LeftParen => "LEFT_PAREN",
            TokenType::RightParen => "RIGHT_PAREN",
            TokenType::COMMA => "COMMA",
            TokenType::DOT => "DOT",
            TokenType::MINUS => "MINUS",
            TokenType::PLUS => "PLUS",
            TokenType::SEMICOLON => "SEMICOLON",
            TokenType::SLASH => "SLASH",
            TokenType::Bang => "BANG",
            TokenType::BangEqual => "BANG_EQUAL",
            TokenType::Equal => "EQUAL",
            TokenType::EqualEqual => "EQUAL_EQUAL",
            TokenType::Greater => "GREATER",
            TokenType::GreaterEqual => "GREATER_EQUAL",
            TokenType::Less => "LESS",
            TokenType::LessEqual => "LESS_EQUAL",
            TokenType::Identifier => "IDENTIFIER",
            TokenType::String => "STRING",
            TokenType::Number => "NUMBER",
            TokenType::And => "AND",
            TokenType::Class => "CLASS",
            TokenType::Else => "ELSE",
            TokenType::False => "FALSE",
            TokenType::Fun => "FUN",
            TokenType::For => "FOR",
            TokenType::If => "IF",
            TokenType::Nil => "NIL",
            TokenType::Or => "OR",
            TokenType::Print => "PRINT",
            TokenType::Return => "RETURN",
            TokenType::Super => "SUPER",
            TokenType::This => "THIS",
            TokenType::True => "TRUE",
            TokenType::Var => "VAR",
            TokenType::While => "WHILE",
            TokenType::EOF => "EOF",
        };

        write!(f, "{}", output)
    }
}

pub struct Token(TokenType, String, Option<Object>);

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, literal: Option<Object>) -> Self {
        Token(token_type, lexeme, literal)
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let literal = match &self.2 {
            Some(literal) => format!("{}", literal),
            None => "null".to_string(),
        };
        write!(f, "{} {} {}", self.0, self.1, literal)
    }
}
