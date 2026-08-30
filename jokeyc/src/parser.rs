//! Recursive descent parser for building the AST.

use crate::ast::{Expr, Stmt};
use crate::lexer::Token;

/// Constructs an Abstract Syntax Tree from a flat token stream.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn consume(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        self.pos += 1;
        t
    }

    /// Parses the entire token stream into a sequence of statements.
    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while *self.peek() != Token::Eof {
            stmts.push(self.parse_stmt());
        }
        stmts
    }

    fn parse_stmt(&mut self) -> Stmt {
        match self.peek() {
            Token::Let => {
                self.consume(); // consume 'let'
                let Token::Ident(name) = self.consume() else {
                    panic!("Expected identifier")
                };
                assert_eq!(self.consume(), Token::Equals, "Expected '='");
                let expr = self.parse_expr();
                assert_eq!(self.consume(), Token::Semicolon, "Expected ';'");
                Stmt::Let(name, expr)
            }
            Token::Print => {
                self.consume(); // consume 'print'
                assert_eq!(self.consume(), Token::LParen, "Expected '('");
                let expr = self.parse_expr();
                assert_eq!(self.consume(), Token::RParen, "Expected ')'");
                assert_eq!(self.consume(), Token::Semicolon, "Expected ';'");
                Stmt::Print(expr)
            }
            _ => panic!("Parser Error: Unexpected token {:?}", self.peek()),
        }
    }

    fn parse_expr(&mut self) -> Expr {
        let mut left = self.parse_term();
        while *self.peek() == Token::Plus {
            self.consume(); // consume '+'
            let right = self.parse_term();
            left = Expr::Add(Box::new(left), Box::new(right));
        }
        left
    }

    fn parse_term(&mut self) -> Expr {
        match self.consume() {
            Token::Int(i) => Expr::Int(i),
            Token::Str(s) => Expr::Str(s),
            Token::Ident(id) => Expr::Ident(id),
            t => panic!("Parser Error: Expected term, found {:?}", t),
        }
    }
}
