//! Lexical analysis (Tokenization).

use std::iter::Peekable;
use std::str::Chars;

/// Valid tokens in the JOCKY language.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Let,
    Print,
    Ident(String),
    Int(i32),
    Str(String),
    Equals,
    Plus,
    Minus,
    Multiply,
    Semicolon,
    LParen,
    RParen,
    Eof,
}

/// Consumes a raw source string and emits a stream of tokens.
pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            chars: input.chars().peekable(),
        }
    }

    /// Scans the entire input and returns a vector of tokens.
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(&c) = self.chars.peek() {
            match c {
                ' ' | '\n' | '\r' | '\t' => {
                    self.chars.next();
                }
                '=' => {
                    self.chars.next();
                    tokens.push(Token::Equals);
                }
                '+' => {
                    self.chars.next();
                    tokens.push(Token::Plus);
                }
                ';' => {
                    self.chars.next();
                    tokens.push(Token::Semicolon);
                }
                '(' => {
                    self.chars.next();
                    tokens.push(Token::LParen);
                }
                ')' => {
                    self.chars.next();
                    tokens.push(Token::RParen);
                }
                '"' => {
                    self.chars.next(); // consume opening quote
                    let mut s = String::new();
                    while let Some(&ch) = self.chars.peek() {
                        if ch == '"' {
                            break;
                        }
                        s.push(self.chars.next().unwrap());
                    }
                    self.chars.next(); // consume closing quote
                    tokens.push(Token::Str(s));
                }
                '-' => {
                    self.chars.next();
                    tokens.push(Token::Minus);
                }
                '*' => {
                    self.chars.next();
                    tokens.push(Token::Multiply);
                }
                _ if c.is_ascii_alphabetic() => {
                    let mut s = String::new();
                    while let Some(&ch) = self.chars.peek() {
                        if ch.is_ascii_alphanumeric() {
                            s.push(self.chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    match s.as_str() {
                        "let" => tokens.push(Token::Let),
                        "print" => tokens.push(Token::Print),
                        _ => tokens.push(Token::Ident(s)),
                    }
                }
                _ if c.is_ascii_digit() => {
                    let mut s = String::new();
                    while let Some(&ch) = self.chars.peek() {
                        if ch.is_ascii_digit() {
                            s.push(self.chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Int(s.parse().unwrap()));
                }
                _ => panic!("Lexer Error: Unexpected character '{}'", c),
            }
        }
        tokens.push(Token::Eof);
        tokens
    }
}
