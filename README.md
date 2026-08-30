# JOCKY Programming Language

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](#)
[![Version](https://img.shields.io/badge/version-0.1.0-blue)](#)

JOCKY is a minimal, statically structured programming language with a highly specialized compiler. Instead of optimizing for speed, the JOCKY compiler optimizes for **chaos**. It translates `.jockeyc` source files into heavily obfuscated, structurally polymorphic C code designed to thwart static analysis, signature scanning, and reverse engineering.

It is built entirely in Rust with zero external dependencies, utilizing a custom Lexer, Recursive Descent Parser, and PRNG.

---

## 🚀 Quick Start

### 1. Installation

Ensure you have the Rust toolchain installed, then clone the repository and build the compiler:

```bash
git clone https://github.com/yourusername/jocky.git
cd jocky
cargo build --release
```

### 2. Compilation Pipeline

JOCKY is a source-to-source transpiler. You pass it a `.jockeyc` script, and it spits out obfuscated C code.

```bash
# 1. Compile JOCKY source to obfuscated C
./target/release/jockyc target.jockeyc > obfuscated.c

# 2. Compile the C output with GCC/Clang
gcc -std=c99 obfuscated.c -o my_program

# 3. Execute
./my_program
```

---

## 📖 Language Specification

JOCKY is designed to be simple to write but absolute hell to reverse-engineer. Every valid statement must be terminated with a semicolon (`;`).

### Data Types

JOCKY currently supports two primitive data types mapped directly to C:

- **Integer**: 32-bit signed integers (e.g., `42`, `-10`)
- **String**: Text enclosed in double quotes (e.g., `"System init"`). Note: strings can currently only be used inside output statements.

### Variable Declaration (`let`)

Variables are declared and assigned using the `let` keyword.
Under the hood, the compiler strips your variable names entirely and replaces them with randomized 6-character identifiers (e.g., `int kLsPqa = 10;`).

**Syntax:**

```
let <identifier> = <expression>;
```

**Examples:**

```
let base = 100;
let offset = 42;
let total = base + offset + 50;
```

### Output (`print`)

The `print` statement evaluates an expression and outputs it to stdout, followed by a newline. Parentheses are strictly required.

**Syntax:**

```
print(<expression>);
```

**Examples:**

```
// Integer evaluation
print(total);
print(total + 10);

// String literals (triggers compile-time XOR encryption)
print("Initializing payload...");
```

### Complete Code Example

```
let seed = 1000;
let modifier = 337;
let key = seed + modifier;

print("--- JOCKY CORE RUNTIME ---");
print("Computed Security Key:");
print(key);

let finalHash = key + 500 + 25;
print("Final Verification Checksum:");
print(finalHash);
```

---

## ⚠️ Semantic Constraints & Caveats

When writing JOCKY code, keep the following parser rules in mind:

- **No string variables**: Strings cannot be assigned to variables (`let msg = "hello";` will panic). They must be passed directly into `print(...)`. This ensures string encryption occurs securely at the statement scope.
- **Strict parentheses**: `print("hello");` is valid. `print "hello";` will result in a syntax parsing error.
- **No unary negation/subtraction**: Only positive integers and the binary `+` operator are currently supported by the recursive descent parser.
- **No comments**: The lexer does not currently ignore `//` or `/* */`. Including them will throw an unexpected character error.

---

## 🛡️ Under the Hood: The Obfuscation Engine

When compiling JOCKY code, the backend automatically applies four layers of hardening:

### 1. Control Flow Flattening (CFF)
Linear execution order is destroyed. Every statement is placed into an isolated case block inside an infinite `while` loop, controlled by a randomized state machine switch.

### 2. Compile-Time String Encryption
Strings are never stored in plaintext within the binary. They are encrypted with a dynamic XOR key during compilation. The compiler emits a localized byte array in C and an inline loop that decrypts the string into memory fractions of a millisecond before printing.

### 3. Symbol Mangling
All variable names defined in `.jockeyc` are stripped from the AST and replaced with random 6-character alphanumeric identifiers.

### 4. Dead Code / Junk Insertion
The compiler rolls a probabilistic check (33% chance per block) to generate junk variables with arbitrary hex values. This shifts stack frame layouts and breaks basic-block signature matching.
