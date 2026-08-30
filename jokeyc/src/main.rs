//! JOCKY CLI
//!
//! Orchestrates the pipeline: Source -> Lexer -> Parser -> Codegen -> Output.

use std::env;
use std::fs;

use jokeyc::codegen::flattened::FlattenedCodeGen;
use jokeyc::lexer::Lexer;
use jokeyc::parser::Parser;

fn main() {
    // 1. Get the filename from the command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <source_file.jockeyc>", args[0]);
        std::process::exit(1);
    }
    let filename = &args[1];

    // 2. Read the source file
    let source = match fs::read_to_string(filename) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            std::process::exit(1);
        }
    };

    // Optional: Print a nice header to stderr so it doesn't interfere
    // if the user pipes stdout to a .c file
    eprintln!("======================================");
    eprintln!("     JOCKY COMPILER POC (CFG MODE)    ");
    eprintln!("======================================");
    eprintln!("Compiling: {}\n", filename);

    // 3. Tokenize and Parse Source
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse();

    if ast.is_empty() {
        eprintln!("Error: No valid statements found in {}", filename);
        std::process::exit(1);
    }

    // 4. Run the Control Flow Flattening Backend
    let mut backend = FlattenedCodeGen::new();
    let build = backend.generate(ast);

    // 5. Output the generated C code to standard output
    println!("{}", build);
}
