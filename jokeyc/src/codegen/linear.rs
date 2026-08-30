//! Linear Polymorphic Backend.
//!
//! This module implements a standard, top-to-bottom polymorphic code generator.
//! It translates the JOCKY AST into C code while randomly mutating variable
//! names, injecting dead code, and encrypting strings.
//!
//! While execution flow remains linear (unlike the `flattened` backend), the
//! resulting binary will have a completely different structural signature and
//! hash on every compilation.

use crate::ast::{Expr, Stmt};
use crate::rng::Rng;
use std::collections::HashMap;

/// The linear polymorphic code generator.
pub struct LinearCodeGen {
    rng: Rng,
    sym_table: HashMap<String, String>,
    out: String,
}

impl LinearCodeGen {
    /// Initializes a new linear code generator with a fresh RNG state.
    pub fn new() -> Self {
        LinearCodeGen {
            rng: Rng::new(),
            sym_table: HashMap::new(),
            out: String::new(),
        }
    }

    /// Recursively generates C expressions from AST expression nodes.
    fn gen_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Int(i) => i.to_string(),
            Expr::Ident(name) => self
                .sym_table
                .get(name)
                .expect("LinearCodeGen Error: Undefined variable referenced")
                .clone(),
            Expr::Add(l, r) => format!("{} + {}", self.gen_expr(l), self.gen_expr(r)),
            Expr::Str(_) => {
                panic!("LinearCodeGen Error: Strings must be handled at the statement level")
            }
        }
    }

    /// Probabilistically injects dead C code (junk variables) to shift
    /// instruction offsets and break basic-block signatures.
    fn maybe_junk(&mut self) {
        // 33% chance to insert dead code
        if self.rng.coin(33) {
            let junk_name = self.rng.fresh_ident(6);
            let junk_val = self.rng.next() & 0xFFFF;
            self.out.push_str(&format!(
                "    int {} = {:#x}; // junk padding\n",
                junk_name, junk_val
            ));
        }
    }

    /// Takes a fully parsed AST and generates the final, obfuscated C source code.
    pub fn generate(&mut self, ast: Vec<Stmt>) -> String {
        self.out.push_str("#include <stdio.h>\n\nint main() {\n");

        for stmt in ast {
            // Randomly inject padding before the statement
            self.maybe_junk();

            match stmt {
                Stmt::Let(orig_name, expr) => {
                    // Generate a totally random 6-character name for the C variable
                    let new_name = self.rng.fresh_ident(6);
                    self.sym_table.insert(orig_name, new_name.clone());

                    let val = self.gen_expr(&expr);
                    self.out
                        .push_str(&format!("    int {} = {};\n", new_name, val));
                }
                Stmt::Print(Expr::Str(s)) => {
                    let arr_name = self.rng.fresh_ident(6);
                    // Generate a random 8-bit XOR key for this specific string instance
                    let key = (self.rng.next() & 0xFF) as u8;

                    // Encrypt the string at compile time (including the null terminator)
                    let mut enc = Vec::new();
                    for b in s.bytes() {
                        enc.push(b ^ key);
                    }
                    enc.push(0 ^ key);

                    // Emit the encrypted byte array
                    self.out
                        .push_str(&format!("    unsigned char {}[] = {{", arr_name));
                    for (i, b) in enc.iter().enumerate() {
                        if i > 0 {
                            self.out.push_str(", ");
                        }
                        self.out.push_str(&format!("{:#04x}", b));
                    }
                    self.out.push_str("};\n");

                    // Emit the inline decryption loop using the dynamic key
                    self.out.push_str(&format!(
                        "    for(int i = 0; i < {}; i++) {}[i] ^= {:#04x};\n",
                        enc.len(),
                        arr_name,
                        key
                    ));
                    self.out
                        .push_str(&format!("    printf(\"%s\\n\", {});\n", arr_name));
                }
                Stmt::Print(expr) => {
                    let val = self.gen_expr(&expr);
                    self.out
                        .push_str(&format!("    printf(\"%d\\n\", {});\n", val));
                }
            }

            // Randomly inject padding after the statement
            self.maybe_junk();
        }

        self.out.push_str("    return 0;\n}\n");
        self.out.clone()
    }
}
