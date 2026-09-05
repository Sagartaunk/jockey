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
    pub fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }
    pub fn consume(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        self.pos += 1;
        t
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while *self.peek() != Token::Eof {
            stmts.push(self.parse_stmt());
        }
        stmts
    }

    pub fn parse_stmt(&mut self) -> Stmt {
        match self.peek() {
            Token::Let => {
                self.consume();
                let Token::Ident(name) = self.consume() else {
                    panic!("Expected identifier")
                };
                assert_eq!(self.consume(), Token::Equals);
                let expr = self.parse_expr();
                assert_eq!(self.consume(), Token::Semicolon);
                Stmt::Let(name, expr)
            }
            Token::Ident(name) => {
                let name = name.clone();
                self.consume();
                assert_eq!(self.consume(), Token::Equals, "Expected '=' for assignment");
                let expr = self.parse_expr();
                assert_eq!(self.consume(), Token::Semicolon);
                Stmt::Assign(name, expr)
            }
            Token::Print => {
                self.consume();
                assert_eq!(self.consume(), Token::LParen);
                let expr = self.parse_expr();
                assert_eq!(self.consume(), Token::RParen);
                assert_eq!(self.consume(), Token::Semicolon);
                Stmt::Print(expr)
            }
            Token::If => {
                self.consume();
                let cond = self.parse_expr();
                let then_branch = self.parse_block();
                let mut else_branch = None;
                if *self.peek() == Token::Else {
                    self.consume();
                    if *self.peek() == Token::If {
                        // Handle `else if` chaining
                        else_branch = Some(Box::new(self.parse_stmt()));
                    } else {
                        else_branch = Some(Box::new(self.parse_block()));
                    }
                }
                Stmt::If(cond, Box::new(then_branch), else_branch)
            }
            Token::While => {
                self.consume();
                let cond = self.parse_expr();
                let body = self.parse_block();
                Stmt::While(cond, Box::new(body))
            }
            Token::LBrace => self.parse_block(),
            t => panic!("Unexpected token at stmt start: {:?}", t),
        }
    }

    pub fn parse_block(&mut self) -> Stmt {
        assert_eq!(self.consume(), Token::LBrace);
        let mut stmts = Vec::new();
        while *self.peek() != Token::RBrace {
            stmts.push(self.parse_stmt());
        }
        self.consume();
        Stmt::Block(stmts)
    }

    // Comparison precedence (==, <, >)
    pub fn parse_expr(&mut self) -> Expr {
        let mut left = self.parse_term();
        loop {
            match self.peek() {
                Token::EqEq => {
                    self.consume();
                    left = Expr::Eq(Box::new(left), Box::new(self.parse_term()));
                }
                Token::Less => {
                    self.consume();
                    left = Expr::Lt(Box::new(left), Box::new(self.parse_term()));
                }
                Token::Greater => {
                    self.consume();
                    left = Expr::Gt(Box::new(left), Box::new(self.parse_term()));
                }
                _ => break,
            }
        }
        left
    }

    // Math precedence (+)
    pub fn parse_term(&mut self) -> Expr {
        let mut left = self.parse_primary();
        while *self.peek() == Token::Plus {
            self.consume();
            left = Expr::Add(Box::new(left), Box::new(self.parse_primary()));
        }
        left
    }

    pub fn parse_primary(&mut self) -> Expr {
        match self.consume() {
            Token::Int(i) => Expr::Int(i),
            Token::Str(s) => Expr::Str(s),
            Token::Ident(id) => Expr::Ident(id),
            t => panic!("Expected expression, got {:?}", t),
        }
    }
}
