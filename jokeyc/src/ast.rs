//! Abstract Syntax Tree (AST) definitions for the JOCKY language.

/// Represents an evaluable expression.
#[derive(Debug, Clone)]
pub enum Expr {
    Int(i32),
    Str(String),
    Ident(String),
    Add(Box<Expr>, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
}

/// Represents a single executable statement.
#[derive(Debug, Clone)]
pub enum Stmt {
    Let(String, Expr),
    Assign(String, Expr),
    Print(Expr),
    Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
}
