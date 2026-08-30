//! # JOCKY Compiler Core
//!
//! This library implements the core phases of the JOCKY polymorphic compiler.
//! It transforms JOCKY source code into heavily obfuscated, structurally unique
//! C code that avoids static signatures.

pub mod ast;
pub mod codegen;
pub mod lexer;
pub mod parser;
pub mod rng;
