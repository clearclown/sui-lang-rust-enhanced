# Sui (粋) - A Programming Language for LLMs

> A line-based programming language optimized for accurate LLM code generation

[日本語版 README](README_ja.md)

## Overview

**Sui (粋)** is a programming language named after the Japanese aesthetic concept meaning "refined" and "elimination of excess." It is designed with LLM (Large Language Model) code generation accuracy as the top priority.

**This repository provides a Rust implementation** with enhanced performance, REPL support, and additional transpiler targets (JavaScript, WebAssembly).

## Features

### Core Features
- Line-based syntax optimized for LLM generation
- High-performance Rust interpreter
- Multiple transpiler targets (Python, JavaScript)
- Interactive REPL mode
- WebAssembly support for browser execution

### Design Principles

1. **Line Independence** - Each line is completely self-contained
2. **Minimal Bracket Matching** - Nesting only for function blocks `{}`
3. **Single-Character Instructions** - Maximum token efficiency
4. **Sequential Variables** - No meaningful names needed (v0, v1, g0, a0)
5. **Explicit Control Flow** - Labels and jumps

## Installation

### From crates.io (Recommended)

```bash
cargo install sui-lang
```

### From source

```bash
git clone https://github.com/TakatoHonda/sui-lang.git
cd sui-lang
cargo install --path .
```

### With additional features

```bash
# Full features (REPL, colored output)
cargo install sui-lang --features full

# Minimal installation
cargo install sui-lang --no-default-features
```

### Legacy Python Version

```bash
# PyPI
pip install sui-lang

# Homebrew (macOS/Linux)
brew tap TakatoHonda/sui
brew install sui-lang
```

## Quick Start

### Interpreter

```bash
# Run sample (shows demo)
sui

# Run file
sui examples/fibonacci.sui

# Run with arguments
sui examples/fib_args.sui 15

# Validate syntax
sui --validate examples/fibonacci.sui

# Start REPL
sui --repl
```

### Transpiler (Sui → Python)

```bash
# Show converted code
sui2py examples/fibonacci.sui

# Output to file
sui2py examples/fibonacci.sui -o fib.py

# Convert and execute
sui2py examples/fibonacci.sui --run
```

### Transpiler (Sui → JavaScript) [NEW]

```bash
# Show converted code
sui2js examples/fibonacci.sui

# Output to file
sui2js examples/fibonacci.sui -o fib.js

# Convert and execute with Node.js
sui2js examples/fibonacci.sui --run

# Generate browser-compatible code
sui2js examples/fibonacci.sui --browser
```

### REPL Mode [NEW]

```bash
sui --repl

# Commands in REPL:
# :help  - Show help
# :reset - Reset interpreter state
# :vars  - Show variables
# :quit  - Exit
```

## Syntax

### Instructions

| Instr | Format | Description |
|-------|--------|-------------|
| `=` | `= var value` | Assignment |
| `+` | `+ result a b` | Addition |
| `-` | `- result a b` | Subtraction |
| `*` | `* result a b` | Multiplication |
| `/` | `/ result a b` | Division |
| `%` | `% result a b` | Modulo |
| `<` | `< result a b` | Less than (0/1) |
| `>` | `> result a b` | Greater than (0/1) |
| `~` | `~ result a b` | Equality (0/1) |
| `!` | `! result a` | NOT |
| `&` | `& result a b` | AND |
| `\|` | `\| result a b` | OR |
| `?` | `? cond label` | Conditional jump |
| `@` | `@ label` | Unconditional jump |
| `:` | `: label` | Label definition |
| `#` | `# id argc {` | Function definition start |
| `}` | `}` | Function definition end |
| `$` | `$ result func args...` | Function call |
| `^` | `^ value` | Return |
| `[` | `[ var size` | Array create |
| `]` | `] result arr idx` | Array read |
| `{` | `{ arr idx value` | Array write |
| `.` | `. value` | Output |
| `,` | `, var` | Input |

### Variables

| Format | Meaning |
|--------|---------|
| `v0`, `v1`, ... | Local variables |
| `g0`, `g1`, ... | Global variables |
| `a0`, `a1`, ... | Function arguments |
| `g100` | argc (command-line argument count) |
| `g101`, `g102`, ... | argv (command-line arguments) |

## Examples

### Fibonacci

```sui
# 0 1 {
< v0 a0 2
! v1 v0
? v1 1
^ a0
: 1
- v2 a0 1
$ v3 0 v2
- v4 a0 2
$ v5 0 v4
+ v6 v3 v5
^ v6
}
= g0 10
$ g1 0 g0
. g1
```

**Output**: `55`

### FizzBuzz

```sui
= v0 1
: 0
> v1 v0 100
? v1 9
% v2 v0 15
~ v3 v2 0
? v3 1
% v4 v0 3
~ v5 v4 0
? v5 2
% v6 v0 5
~ v7 v6 0
? v7 3
. v0
@ 4
: 1
. "FizzBuzz"
@ 4
: 2
. "Fizz"
@ 4
: 3
. "Buzz"
@ 4
: 4
+ v0 v0 1
@ 0
: 9
```

## Library Usage (Rust)

```rust
use sui_lang::Interpreter;

fn main() {
    let code = r#"
= v0 10
+ v1 v0 5
. v1
"#;

    let mut interpreter = Interpreter::new();
    let output = interpreter.run(code, &[]).unwrap();
    println!("Output: {:?}", output);  // ["15"]
}
```

## File Structure

```
sui-lang/
├── Cargo.toml          # Rust package configuration
├── README.md           # This file (English)
├── README_ja.md        # Japanese README
├── LICENSE             # MIT License
├── src/
│   ├── lib.rs          # Library root
│   ├── bin/
│   │   ├── sui.rs      # Main interpreter CLI
│   │   ├── sui2py.rs   # Sui → Python transpiler
│   │   └── sui2js.rs   # Sui → JavaScript transpiler
│   ├── interpreter/    # Core interpreter
│   │   ├── mod.rs
│   │   ├── lexer.rs
│   │   ├── parser.rs
│   │   ├── runtime.rs
│   │   └── value.rs
│   ├── transpiler/     # Transpilers
│   │   ├── mod.rs
│   │   ├── sui2py.rs
│   │   └── sui2js.rs
│   ├── repl/           # REPL implementation
│   └── wasm/           # WebAssembly bindings
├── examples/           # Example Sui programs
│   ├── fibonacci.sui
│   ├── fib_args.sui
│   ├── fizzbuzz.sui
│   ├── list_sum.sui
│   ├── args_demo.sui
│   └── ffi_demo.sui
├── tests/              # Integration tests
├── benches/            # Benchmarks
└── prompts/            # LLM prompts
    ├── system_prompt.md
    └── examples.md
```

## LLM Integration

Sui is designed for LLM code generation. Use the prompts in `prompts/` directory:

1. Copy the system prompt from `prompts/system_prompt.md`
2. Paste it into ChatGPT / Claude / Gemini / etc.
3. Ask to generate Sui code for your task
4. Run with `sui your_code.sui`

See [prompts/examples.md](prompts/examples.md) for prompt templates and expected outputs.

## Why Sui?

### Name Origin

**Sui (粋)** - A Japanese word meaning "refined," "sophisticated," or "the essence." It represents the aesthetic of eliminating excess and keeping only what is essential.

### Avoiding LLM Weaknesses

| LLM Weakness | Sui's Solution |
|--------------|----------------|
| Bracket mismatch | Only `{}` for functions |
| Long-range dependencies | Each line is independent |
| Variable name typos | Only sequential numbers (v0, v1...) |
| Complex nesting | No nesting, decompose to temp variables |

### Performance (Rust vs Python)

| Benchmark | Python | Rust | Speedup |
|-----------|--------|------|---------|
| Fibonacci(20) | ~50ms | ~1ms | ~50x |
| Loop 10000 | ~100ms | ~2ms | ~50x |
| Array ops | ~80ms | ~1ms | ~80x |

## Roadmap

- [x] Rust interpreter
- [x] Transpiler (Python output)
- [x] Transpiler (JavaScript output)
- [x] REPL mode
- [x] WebAssembly bindings
- [ ] Type annotations (optional)
- [ ] LLVM IR output
- [ ] LSP (Language Server Protocol)
- [ ] Debugger

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License
