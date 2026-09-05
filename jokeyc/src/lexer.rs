//! Lexical analysis (Tokenization).

use std::iter::Peekable;
use std::str::Chars;

/// Valid tokens in the JOCKY language.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Let,
    Print,
    If,
    Else,
    While,
    Ident(String),
    Int(i32),
    Str(String),
    Equals,
    EqEq,
    Less,
    Greater,
    Plus,
    Semicolon,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Eof,
}

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            chars: input.chars().peekable(),
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(&c) = self.chars.peek() {
            match c {
                ' ' | '\n' | '\r' | '\t' => {
                    self.chars.next();
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
                '{' => {
                    self.chars.next();
                    tokens.push(Token::LBrace);
                }
                '}' => {
                    self.chars.next();
                    tokens.push(Token::RBrace);
                }
                '<' => {
                    self.chars.next();
                    tokens.push(Token::Less);
                }
                '>' => {
                    self.chars.next();
                    tokens.push(Token::Greater);
                }
                '=' => {
                    self.chars.next();
                    if self.chars.peek() == Some(&'=') {
                        self.chars.next();
                        tokens.push(Token::EqEq);
                    } else {
                        tokens.push(Token::Equals);
                    }
                }
                '"' => {
                    self.chars.next();
                    let mut s = String::new();
                    while let Some(&ch) = self.chars.peek() {
                        if ch == '"' {
                            break;
                        }
                        s.push(self.chars.next().unwrap());
                    }
                    self.chars.next();
                    tokens.push(Token::Str(s));
                }
                _ if c.is_ascii_alphabetic() => {
                    let mut s = String::new();
                    while let Some(&ch) = self.chars.peek() {
                        if ch.is_ascii_alphanumeric() || ch == '_' {
                            s.push(self.chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    match s.as_str() {
                        "let" => tokens.push(Token::Let),
                        "print" => tokens.push(Token::Print),
                        "if" => tokens.push(Token::If),
                        "else" => tokens.push(Token::Else),
                        "while" => tokens.push(Token::While),
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
                _ => panic!("Unexpected char: {}", c),
            }
        }
        tokens.push(Token::Eof);
        tokens
    }
}
