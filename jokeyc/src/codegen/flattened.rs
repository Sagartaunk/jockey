//! Control Flow Flattening Backend.
//!
//! Destroys the spatial locality of the generated C code by wrapping
//! every statement in a scrambled `switch` block within an infinite `while` loop,
//! governed by a randomized state machine.

use crate::ast::{Expr, Stmt};
use crate::rng::Rng;
use std::collections::HashMap;

pub struct FlattenedCodeGen {
    rng: Rng,
    sym_table: HashMap<String, String>,
    out: String,
}

impl FlattenedCodeGen {
    pub fn new() -> Self {
        FlattenedCodeGen {
            rng: Rng::new(),
            sym_table: HashMap::new(),
            out: String::new(),
        }
    }

    /// Evaluates an expression into C syntax.
    fn gen_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Int(i) => i.to_string(),
            Expr::Ident(name) => self.sym_table.get(name).unwrap().clone(),
            Expr::Add(l, r) => format!("{} + {}", self.gen_expr(l), self.gen_expr(r)),
            Expr::Sub(l, r) => format!("{} - {}", self.gen_expr(l), self.gen_expr(r)),
            Expr::Multiply(l, r) => format!("{} * {}", self.gen_expr(l), self.gen_expr(r)),
            Expr::Str(_) => panic!("Strings must be handled at the statement level for encryption"),
        }
    }

    /// Injects probabilistic dead-code states to further confuse static analysis.
    fn maybe_junk(&mut self) {
        if self.rng.coin(33) {
            let junk_name = self.rng.fresh_ident(6);
            let junk_val = self.rng.next() & 0xFFFF;
            self.out.push_str(&format!(
                "                int {} = {:#x};\n",
                junk_name, junk_val
            ));
        }
    }

    /// Generates the fully obfuscated C source.
    pub fn generate(&mut self, ast: Vec<Stmt>) -> String {
        self.out.push_str("#include <stdio.h>\n\nint main() {\n");

        // 1. Generate unique state IDs
        let mut states: Vec<u32> = (0..ast.len() + 1)
            .map(|_| (self.rng.next() & 0x7FFFFFFF) as u32)
            .collect();

        let end_state = states.pop().unwrap();
        let start_state = states[0];

        // 2. Initialize the state machine
        self.out
            .push_str(&format!("    unsigned int state = {:#x};\n", start_state));

        // Hoist variable declarations outside the loop so data persists across states
        for stmt in &ast {
            if let Stmt::Let(orig_name, _) = stmt {
                let new_name = self.rng.fresh_ident(6);
                self.sym_table.insert(orig_name.clone(), new_name.clone());
                self.out.push_str(&format!("    int {} = 0;\n", new_name));
            }
        }

        self.out
            .push_str(&format!("    while (state != {:#x}) {{\n", end_state));
        self.out.push_str("        switch(state) {\n");

        // 3. Scramble block indices (Fisher-Yates)
        let mut block_indices: Vec<usize> = (0..ast.len()).collect();
        for i in (1..block_indices.len()).rev() {
            let j = (self.rng.next() as usize) % (i + 1);
            block_indices.swap(i, j);
        }

        // 4. Emit the scrambled blocks
        for i in block_indices {
            let stmt = &ast[i];
            let current_state = states[i];
            let next_state = if i + 1 < ast.len() {
                states[i + 1]
            } else {
                end_state
            };

            self.out
                .push_str(&format!("            case {:#x}:\n", current_state));
            self.out.push_str("            {\n"); // Scope block for local string arrays

            self.maybe_junk();

            match stmt {
                Stmt::Let(orig_name, expr) => {
                    // Evaluate the RHS first (needs &mut self via gen_expr), *then*
                    // look up the target's mangled name. This keeps the immutable
                    // borrow from sym_table.get() from ever overlapping with the
                    // mutable borrow gen_expr needs, so both borrows are legal
                    // without cloning the name string.
                    let val = self.gen_expr(expr);
                    let new_name = self.sym_table.get(orig_name).unwrap();
                    self.out
                        .push_str(&format!("                {} = {};\n", new_name, val));
                }
                Stmt::Print(Expr::Str(s)) => {
                    // In-place string encryption
                    let arr_name = self.rng.fresh_ident(6);
                    let key = (self.rng.next() & 0xFF) as u8;

                    let mut enc = Vec::new();
                    for b in s.bytes() {
                        enc.push(b ^ key);
                    }
                    enc.push(0 ^ key);

                    self.out.push_str(&format!(
                        "                unsigned char {}[] = {{",
                        arr_name
                    ));
                    for (idx, b) in enc.iter().enumerate() {
                        if idx > 0 {
                            self.out.push_str(", ");
                        }
                        self.out.push_str(&format!("{:#04x}", b));
                    }
                    self.out.push_str("};\n");

                    self.out.push_str(&format!(
                        "                for(int i=0; i<{}; i++) {}[i] ^= {:#04x};\n",
                        enc.len(),
                        arr_name,
                        key
                    ));
                    self.out.push_str(&format!(
                        "                printf(\"%s\\n\", {});\n",
                        arr_name
                    ));
                }
                Stmt::Print(expr) => {
                    let val = self.gen_expr(expr);
                    self.out
                        .push_str(&format!("                printf(\"%d\\n\", {});\n", val));
                }
            }

            self.maybe_junk();

            // Advance state
            self.out
                .push_str(&format!("                state = {:#x};\n", next_state));
            self.out.push_str("                break;\n");
            self.out.push_str("            }\n"); // End scope block
        }

        // 5. Default trap
        self.out.push_str("            default:\n");
        self.out
            .push_str(&format!("                state = {:#x};\n", end_state));
        self.out.push_str("                break;\n");

        self.out.push_str("        }\n    }\n");
        self.out.push_str("    return 0;\n}\n");
        self.out.clone()
    }
}
