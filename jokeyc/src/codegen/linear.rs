//! Linear Polymorphic Backend.
//!
//! Translates the JOCKY AST into C code. Execution flow remains linear,
//! but variable names, junk code, and encryption keys mutate on every build.
//! Supports nested blocks, loops, and control flow.

use crate::ast::{Expr, Stmt};
use crate::rng::Rng;
use std::collections::HashMap;

#[derive(Clone, PartialEq)]
pub enum VarType {
    Int,
    Str,
}

pub struct LinearCodeGen {
    rng: Rng,
    // Tracks both the generated C name and the type for printf formatting
    sym_table: HashMap<String, (String, VarType)>,
}

impl LinearCodeGen {
    pub fn new() -> Self {
        LinearCodeGen {
            rng: Rng::new(),
            sym_table: HashMap::new(),
        }
    }

    fn gen_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Int(i) => i.to_string(),
            Expr::Ident(name) => self.sym_table.get(name).unwrap().0.clone(),
            Expr::Add(l, r) => format!("({} + {})", self.gen_expr(l), self.gen_expr(r)),
            Expr::Eq(l, r) => format!("({} == {})", self.gen_expr(l), self.gen_expr(r)),
            Expr::Lt(l, r) => format!("({} < {})", self.gen_expr(l), self.gen_expr(r)),
            Expr::Gt(l, r) => format!("({} > {})", self.gen_expr(l), self.gen_expr(r)),
            Expr::Str(_) => panic!("Strings handled at statement level"),
        }
    }

    fn emit_junk(&mut self, out: &mut String) {
        if self.rng.coin(33) {
            let junk_name = self.rng.fresh_ident(6);
            let junk_val = self.rng.next() & 0xFFFF;
            out.push_str(&format!(
                "    int {} = {:#x}; // junk\n",
                junk_name, junk_val
            ));
        }
    }

    fn emit_encrypted_string(&mut self, s: &str, var_name: &str, out: &mut String) {
        let key = (self.rng.next() & 0xFF) as u8;
        let mut enc = Vec::new();
        for b in s.bytes() {
            enc.push(b ^ key);
        }
        enc.push(0 ^ key);

        out.push_str(&format!("    unsigned char {}[] = {{", var_name));
        for (i, b) in enc.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            out.push_str(&format!("{:#04x}", b));
        }
        out.push_str("};\n");
        out.push_str(&format!(
            "    for(int i=0; i<{}; i++) {}[i] ^= {:#04x};\n",
            enc.len(),
            var_name,
            key
        ));
    }

    fn gen_stmt(&mut self, stmt: &Stmt, out: &mut String) {
        self.emit_junk(out);

        match stmt {
            Stmt::Let(orig, Expr::Str(s)) => {
                let new_name = self.rng.fresh_ident(6);
                self.sym_table
                    .insert(orig.clone(), (new_name.clone(), VarType::Str));
                self.emit_encrypted_string(s, &new_name, out);
            }
            Stmt::Let(orig, expr) => {
                let new_name = self.rng.fresh_ident(6);
                self.sym_table
                    .insert(orig.clone(), (new_name.clone(), VarType::Int));
                let val = self.gen_expr(expr);
                out.push_str(&format!("    int {} = {};\n", new_name, val));
            }
            Stmt::Assign(orig, expr) => {
                let (c_name, _) = self.sym_table.get(orig).unwrap().clone();
                let val = self.gen_expr(expr);
                out.push_str(&format!("    {} = {};\n", c_name, val));
            }
            Stmt::Print(Expr::Str(s)) => {
                let tmp_name = self.rng.fresh_ident(6);
                out.push_str("    {\n");
                self.emit_encrypted_string(s, &tmp_name, out);
                out.push_str(&format!("    printf(\"%s\\n\", {});\n", tmp_name));
                out.push_str("    }\n");
            }
            Stmt::Print(Expr::Ident(name)) => {
                let (c_name, v_type) = self.sym_table.get(name).unwrap();
                match v_type {
                    VarType::Int => out.push_str(&format!("    printf(\"%d\\n\", {});\n", c_name)),
                    VarType::Str => out.push_str(&format!("    printf(\"%s\\n\", {});\n", c_name)),
                }
            }
            Stmt::Print(expr) => {
                let val = self.gen_expr(expr);
                out.push_str(&format!("    printf(\"%d\\n\", {});\n", val));
            }
            Stmt::Block(stmts) => {
                out.push_str("    {\n");
                for s in stmts {
                    self.gen_stmt(s, out);
                }
                out.push_str("    }\n");
            }
            Stmt::If(cond, then_b, else_b) => {
                let c_cond = self.gen_expr(cond);
                out.push_str(&format!("    if ({}) \n", c_cond));
                self.gen_stmt(then_b, out);
                if let Some(eb) = else_b {
                    out.push_str("    else \n");
                    self.gen_stmt(eb, out);
                }
            }
            Stmt::While(cond, body) => {
                let c_cond = self.gen_expr(cond);
                out.push_str(&format!("    while ({}) \n", c_cond));
                self.gen_stmt(body, out);
            }
        }

        self.emit_junk(out);
    }

    pub fn generate(&mut self, ast: Vec<Stmt>) -> String {
        let mut out = String::new();
        out.push_str("#include <stdio.h>\n\nint main() {\n");
        for stmt in ast {
            self.gen_stmt(&stmt, &mut out);
        }
        out.push_str("    return 0;\n}\n");
        out
    }
}
