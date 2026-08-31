//! Abstract Syntax Tree (AST) definitions for the JOCKY language.

/// Represents an evaluable expression.
#[derive(Debug, Clone)]
pub enum Expr {
    /// An integer literal (e.g., `42`)
    Int(i32),
    /// A string literal (e.g., `"Hello"`)
    Str(String),
    /// A variable identifier (e.g., `x`)
    Ident(String),
    /// An addition operation (e.g., `x + 5`)
    Add(Box<Expr>, Box<Expr>),
    /// A subtraction operation (e.g., `x - 5`)
    Sub(Box<Expr>, Box<Expr>),
    /// A multiplication operation (e.g, `x * 5)
    Multiply(Box<Expr>, Box<Expr>),
}

/// Represents a single executable statement.
#[derive(Debug, Clone)]
pub enum Stmt {
    /// Variable assignment: `let x = 5;`
    Let(String, Expr),
    /// Standard output: `print(x);`
    Print(Expr),
}
