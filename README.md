# Sui (粋) - A Programming Language for LLMs

[![Crates.io](https://img.shields.io/crates/v/sui-lang.svg)](https://crates.io/crates/sui-lang)
[![Documentation](https://docs.rs/sui-lang/badge.svg)](https://docs.rs/sui-lang)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/clearclown/sui-lang-rust-enhanced/actions/workflows/ci.yml/badge.svg)](https://github.com/clearclown/sui-lang-rust-enhanced/actions)

> A line-based programming language optimized for accurate LLM code generation

[日本語版 README](README_ja.md) | [Online Playground](https://clearclown.github.io/sui-lang-rust-enhanced/)

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

### Transpiler (Sui → JavaScript)

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

### Transpiler (Python → Sui)

Convert Python code to Sui for LLM-friendly output:

```bash
# Convert Python to Sui
py2sui script.py

# Output to file
py2sui script.py -o script.sui

# Example conversion
echo 'x = 10
y = x + 5
print(y)' | py2sui

# Output:
# = v0 10
# + v1 v0 5
# . v1
```

### REPL Mode

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
| `R`/`P` | `R result "func" args...` | FFI call |

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
│   │   ├── sui2py.rs   # Sui → Python transpiler CLI
│   │   ├── sui2js.rs   # Sui → JavaScript transpiler CLI
│   │   └── py2sui.rs   # Python → Sui transpiler CLI
│   ├── interpreter/    # Core interpreter
│   │   ├── mod.rs
│   │   ├── lexer.rs    # Tokenization
│   │   ├── parser.rs   # AST generation
│   │   ├── runtime.rs  # Execution engine
│   │   └── value.rs    # Value types
│   ├── transpiler/     # Transpilers
│   │   ├── mod.rs
│   │   ├── sui2py.rs   # Sui → Python
│   │   ├── sui2js.rs   # Sui → JavaScript
│   │   └── py2sui.rs   # Python → Sui
│   ├── repl/           # Interactive REPL
│   │   └── mod.rs
│   └── wasm/           # WebAssembly bindings
│       └── mod.rs
├── examples/           # Example Sui programs
│   ├── fibonacci.sui   # Recursive Fibonacci
│   ├── fib_args.sui    # Fibonacci with CLI args
│   ├── fizzbuzz.sui    # Classic FizzBuzz
│   ├── list_sum.sui    # Array operations
│   ├── args_demo.sui   # Command-line arguments
│   └── ffi_demo.sui    # FFI function calls
├── tests/              # Integration tests
│   ├── comprehensive_test.rs
│   └── integration_test.rs
├── benches/            # Performance benchmarks
│   └── interpreter.rs
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

### The Problem with LLM Code Generation

Current LLMs struggle with certain code generation patterns. Research from [ACM TOSEM](https://dl.acm.org/doi/10.1145/3770084) shows that Domain-Specific Languages (DSLs) present additional challenges due to their unique syntax and data scarcity.

| LLM Weakness | Traditional Code | Sui's Solution |
|--------------|-----------------|----------------|
| Bracket mismatch | `if (x) { if (y) { ... } }` | Only `{}` for functions |
| Long-range dependencies | Variables used 100+ lines later | Each line is independent |
| Variable name typos | `userCount` vs `userCont` | Sequential numbers (v0, v1...) |
| Complex nesting | Nested callbacks/conditionals | Flat structure with labels |
| Context window limits | Large codebase understanding | Minimal token usage |

### AI-Friendly Design Philosophy

Inspired by research from [MoonBit](https://www.moonbitlang.com/blog/ai-coding) and [JetBrains](https://blog.jetbrains.com/kotlin/2024/05/ai-friendly-programming-languages-the-kotlin-story/), Sui implements key AI-friendly principles:

1. **Structured Output Compatibility** - Sui's line-based syntax avoids JSON escaping issues that [complicate LLM code generation](https://medium.com/@mne/improving-llm-code-generation-my-best-practices-eb88b128303a)
2. **Minimal Syntax Surface** - Single-character instructions reduce hallucination rates
3. **Deterministic Parsing** - No ambiguous grammar rules
4. **Context Efficiency** - Programs use fewer tokens than equivalent Python/JS

### Performance (Rust vs Python)

| Benchmark | Python | Rust | Speedup |
|-----------|--------|------|---------|
| Fibonacci(20) | ~50ms | ~1ms | **~50x** |
| Loop 10000 | ~100ms | ~2ms | **~50x** |
| Array ops | ~80ms | ~1ms | **~80x** |
| WASM binary | N/A | ~50KB | Instant load |

### Comparison with Other Approaches

| Feature | Sui | [LMQL](https://lmql.ai/) | Python | JSON DSL |
|---------|-----|------|--------|----------|
| LLM output parsing | Trivial | Complex | Medium | Error-prone |
| Token efficiency | Excellent | Medium | Low | Medium |
| Bracket matching | Minimal | Required | Required | Required |
| Runtime environment | Native/WASM/Browser | Python | Python | Parser needed |
| Learning curve | Low | Medium | Low | Low |

## FFI (Foreign Function Interface)

Sui supports calling builtin functions using the `R` (or `P`) command:

```sui
; Math functions
R v0 "math.sqrt" 16        ; v0 = 4.0
R v1 "pow" 2 10            ; v1 = 1024.0
R v2 "sin" 0               ; v2 = 0.0
R v3 "cos" 0               ; v3 = 1.0

; String/Array functions
R v4 "len" "hello"         ; v4 = 5
R v5 "abs" -42             ; v5 = 42
R v6 "max" 10 20 5 30      ; v6 = 30
R v7 "min" 10 20 5 30      ; v7 = 5

; Type conversion
R v8 "int" "123"           ; v8 = 123
R v9 "float" "3.14"        ; v9 = 3.14
R v10 "str" 42             ; v10 = "42"
R v11 "round" 3.14159 2    ; v11 = 3.14

; Random
R v12 "random.randint" 1 100  ; v12 = random 1-100
```

## WebAssembly Support

Sui compiles to WebAssembly for browser execution with near-native performance:

```bash
# Build WASM module
wasm-pack build --target web

# Use in browser
```

```html
<script type="module">
import init, { run_sui } from './pkg/sui_lang.js';

await init();
const output = run_sui(`
= v0 10
+ v1 v0 5
. v1
`);
console.log(output); // ["15"]
</script>
```

Benefits of [Rust + WebAssembly](https://rustwasm.github.io/book/):
- **Small binary size**: ~50KB (vs Go's 2MB+ minimum)
- **No runtime overhead**: Direct compilation to WASM
- **Memory safety**: Rust's guarantees carry over to WASM

## Roadmap

### Completed
- [x] High-performance Rust interpreter
- [x] Transpiler (Sui → Python)
- [x] Transpiler (Sui → JavaScript)
- [x] Transpiler (Python → Sui)
- [x] Interactive REPL mode
- [x] WebAssembly bindings
- [x] FFI support (builtin functions)
- [x] Comprehensive test suite (115+ tests)

### In Progress
- [ ] Online playground (WASM-based)
- [ ] VS Code extension

### Planned
- [ ] [LSP (Language Server Protocol)](https://microsoft.github.io/language-server-protocol/) - Using [tower-lsp](https://github.com/ebkalderon/tower-lsp)
- [ ] Step debugger with breakpoints
- [ ] [LLVM IR](https://mcyoung.xyz/2023/08/01/llvm-ir/) output for native compilation
- [ ] Type annotations (optional static typing)
- [ ] Package manager for Sui modules
- [ ] Jupyter kernel integration

## Research & References

Sui's design is informed by recent research in LLM code generation:

- **[LLM Code Generation Survey](https://dl.acm.org/doi/10.1145/3770084)** - ACM TOSEM survey on challenges with DSLs
- **[Awesome Code LLM](https://github.com/codefuse-ai/Awesome-Code-LLM)** - Curated list of code generation research
- **[Structured Output Best Practices](https://www.timlrx.com/blog/generating-structured-output-from-llms)** - Why simple output formats matter
- **[MoonBit AI-Friendly Design](https://www.moonbitlang.com/blog/ai-coding)** - Block-based language design for AI
- **[LMQL](https://lmql.ai/)** - Language Model Query Language for constrained generation

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Setup

```bash
# Clone and build
git clone https://github.com/clearclown/sui-lang-rust-enhanced.git
cd sui-lang-rust-enhanced
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench

# Build with all features
cargo build --features full
```

### Areas for Contribution

- **LSP Implementation** - Help build IDE support using [tower-lsp](https://github.com/ebkalderon/tower-lsp)
- **LLVM Backend** - Native compilation via [llvm-sys](https://crates.io/crates/llvm-sys)
- **Documentation** - Improve examples and tutorials
- **Testing** - Add more edge case tests

## License

MIT License

---

<p align="center">
  <b>Sui (粋)</b> - Refined code generation for the AI era
</p>
