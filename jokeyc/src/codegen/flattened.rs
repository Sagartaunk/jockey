//! Control Flow Flattening Backend.
//!
//! Destroys spatial locality by transforming AST blocks into scrambled
//! state machines. Features "Block-Level Flattening": nested loops and
//! if-statements recursively spawn their own isolated, flattened state machines.

use crate::ast::{Expr, Stmt};
use crate::rng::Rng;
use std::collections::HashMap;

#[derive(Clone, PartialEq)]
pub enum VarType {
    Int,
    Str,
}

pub struct FlattenedCodeGen {
    rng: Rng,
    sym_table: HashMap<String, (String, VarType)>,
}

impl FlattenedCodeGen {
    pub fn new() -> Self {
        FlattenedCodeGen {
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
            Expr::Str(_) => panic!("Strings must be handled at the statement level"),
        }
    }

    fn gen_junk(&mut self) -> String {
        if self.rng.coin(33) {
            let junk_name = self.rng.fresh_ident(6);
            let junk_val = self.rng.next() & 0xFFFF;
            format!("                int {} = {:#x};\n", junk_name, junk_val)
        } else {
            String::new()
        }
    }

    /// Recursively compiles a list of statements into a standalone,
    /// flattened C state machine loop.
    fn gen_flattened_block(&mut self, stmts: &[Stmt]) -> String {
        let mut out = String::new();
        out.push_str("    {\n"); // Scope for this state machine

        // 1. Hoist Variable Declarations
        // C requires variables to be declared outside the switch scope
        // if they carry data across different case states.
        for stmt in stmts {
            if let Stmt::Let(orig, expr) = stmt {
                let new_name = self.rng.fresh_ident(6);
                let is_str = matches!(expr, Expr::Str(_));
                let v_type = if is_str { VarType::Str } else { VarType::Int };

                self.sym_table
                    .insert(orig.clone(), (new_name.clone(), v_type));

                if is_str {
                    if let Expr::Str(s) = expr {
                        out.push_str(&format!(
                            "        unsigned char {}[{}];\n",
                            new_name,
                            s.len() + 1
                        ));
                    }
                } else {
                    out.push_str(&format!("        int {} = 0;\n", new_name));
                }
            }
        }

        // 2. Setup States
        let mut states: Vec<u32> = (0..stmts.len() + 1)
            .map(|_| (self.rng.next() & 0x7FFFFFFF) as u32)
            .collect();
        let end_state = states.pop().unwrap();
        let start_state = states[0];
        let state_var = self.rng.fresh_ident(8);

        out.push_str(&format!(
            "        unsigned int {} = {:#x};\n",
            state_var, start_state
        ));
        out.push_str(&format!(
            "        while ({} != {:#x}) {{\n",
            state_var, end_state
        ));
        out.push_str(&format!("            switch({}) {{\n", state_var));

        // 3. Scramble Blocks (Fisher-Yates)
        let mut indices: Vec<usize> = (0..stmts.len()).collect();
        for i in (1..indices.len()).rev() {
            let j = (self.rng.next() as usize) % (i + 1);
            indices.swap(i, j);
        }

        // 4. Emit Cases
        for i in indices {
            let stmt = &stmts[i];
            let current = states[i];
            let next = if i + 1 < stmts.len() {
                states[i + 1]
            } else {
                end_state
            };

            out.push_str(&format!("                case {:#x}: {{\n", current));
            out.push_str(&self.gen_junk());

            match stmt {
                Stmt::Let(orig, expr) | Stmt::Assign(orig, expr) => {
                    let (c_name, v_type) = self.sym_table.get(orig).unwrap().clone();
                    if v_type == VarType::Int {
                        let val = self.gen_expr(expr);
                        out.push_str(&format!("                    {} = {};\n", c_name, val));
                    } else if let Expr::Str(s) = expr {
                        // Populate the hoisted string buffer using an encrypted local payload
                        let key = (self.rng.next() & 0xFF) as u8;
                        let mut enc = Vec::new();
                        for b in s.bytes() {
                            enc.push(b ^ key);
                        }
                        enc.push(0 ^ key);

                        out.push_str(&format!(
                            "                    unsigned char tmp_{}[] = {{",
                            c_name
                        ));
                        for (idx, b) in enc.iter().enumerate() {
                            if idx > 0 {
                                out.push_str(", ");
                            }
                            out.push_str(&format!("{:#04x}", b));
                        }
                        out.push_str("};\n");
                        out.push_str(&format!("                    for(int i=0; i<{}; i++) {}[i] = tmp_{}[i] ^ {:#04x};\n", enc.len(), c_name, c_name, key));
                    }
                }
                Stmt::Print(Expr::Str(s)) => {
                    let arr_name = self.rng.fresh_ident(6);
                    let key = (self.rng.next() & 0xFF) as u8;
                    let mut enc = Vec::new();
                    for b in s.bytes() {
                        enc.push(b ^ key);
                    }
                    enc.push(0 ^ key);

                    out.push_str(&format!(
                        "                    unsigned char {}[] = {{",
                        arr_name
                    ));
                    for (idx, b) in enc.iter().enumerate() {
                        if idx > 0 {
                            out.push_str(", ");
                        }
                        out.push_str(&format!("{:#04x}", b));
                    }
                    out.push_str("};\n");
                    out.push_str(&format!(
                        "                    for(int i=0; i<{}; i++) {}[i] ^= {:#04x};\n",
                        enc.len(),
                        arr_name,
                        key
                    ));
                    out.push_str(&format!(
                        "                    printf(\"%s\\n\", {});\n",
                        arr_name
                    ));
                }
                Stmt::Print(Expr::Ident(name)) => {
                    let (c_name, v_type) = self.sym_table.get(name).unwrap();
                    match v_type {
                        VarType::Int => out.push_str(&format!(
                            "                    printf(\"%d\\n\", {});\n",
                            c_name
                        )),
                        VarType::Str => out.push_str(&format!(
                            "                    printf(\"%s\\n\", {});\n",
                            c_name
                        )),
                    }
                }
                Stmt::Print(expr) => {
                    let val = self.gen_expr(expr);
                    out.push_str(&format!(
                        "                    printf(\"%d\\n\", {});\n",
                        val
                    ));
                }
                Stmt::Block(inner_stmts) => {
                    out.push_str(&self.gen_flattened_block(inner_stmts));
                }
                Stmt::If(cond, then_b, else_b) => {
                    let c_cond = self.gen_expr(cond);
                    out.push_str(&format!("                    if ({}) \n", c_cond));

                    // Recursively flatten the body blocks!
                    if let Stmt::Block(b) = &**then_b {
                        out.push_str(&self.gen_flattened_block(b));
                    }

                    if let Some(eb) = else_b {
                        out.push_str("                    else \n");
                        if let Stmt::Block(b) = &**eb {
                            out.push_str(&self.gen_flattened_block(b));
                        } else if let Stmt::If(_, _, _) = &**eb {
                            // Handle 'else if' gracefully
                            let mut wrap = Vec::new();
                            wrap.push((**eb).clone());
                            out.push_str(&self.gen_flattened_block(&wrap));
                        }
                    }
                }
                Stmt::While(cond, body) => {
                    let c_cond = self.gen_expr(cond);
                    out.push_str(&format!("                    while ({}) \n", c_cond));
                    if let Stmt::Block(b) = &**body {
                        out.push_str(&self.gen_flattened_block(b));
                    }
                }
            }

            out.push_str(&self.gen_junk());
            out.push_str(&format!(
                "                    {} = {:#x};\n",
                state_var, next
            ));
            out.push_str("                    break;\n");
            out.push_str("                }\n");
        }

        out.push_str("                default:\n");
        out.push_str(&format!(
            "                    {} = {:#x};\n",
            state_var, end_state
        ));
        out.push_str("                    break;\n");
        out.push_str("            }\n        }\n    }\n");

        out
    }

    pub fn generate(&mut self, ast: Vec<Stmt>) -> String {
        let mut out = String::new();
        out.push_str("#include <stdio.h>\n\nint main() {\n");

        // Feed the root AST into the recursive flattener
        out.push_str(&self.gen_flattened_block(&ast));

        out.push_str("    return 0;\n}\n");
        out
    }
}
