# Sui Language Support for VS Code

Syntax highlighting, snippets, and Language Server Protocol (LSP) support for **Sui (粋)** - a programming language optimized for LLM code generation.

## Features

- **Language Server Protocol (LSP)**: Real-time diagnostics, hover information, document symbols
- **Syntax Highlighting**: Full support for all Sui instructions
- **Code Snippets**: Quick templates for common patterns
- **Language Configuration**: Comment toggling, bracket matching

## Installation

### Prerequisites

For full LSP support, install the Sui language server:

```bash
cargo install sui-lang --features lsp
```

### From VSIX (Local)

```bash
cd vscode-sui
npm install
npm run compile
npm run package
code --install-extension sui-lang-0.2.0.vsix
```

### Manual Installation

1. Copy the `vscode-sui` folder to your VS Code extensions directory:
   - **Windows**: `%USERPROFILE%\.vscode\extensions\sui-lang`
   - **macOS/Linux**: `~/.vscode/extensions/sui-lang`
2. Restart VS Code

## LSP Features

### Diagnostics

Real-time syntax error detection as you type:

- Invalid instructions
- Missing arguments
- Unmatched braces
- Invalid function definitions

### Hover Information

Hover over any instruction to see:

- Instruction syntax
- Brief description
- Usage examples

### Document Symbols

Navigate your code with the Outline view (Ctrl+Shift+O):

- Functions (`# id argc {`)
- Labels (`: label`)

## Configuration

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `sui.lsp.enabled` | boolean | `true` | Enable/disable the language server |
| `sui.lsp.path` | string | `"sui-lsp"` | Path to the sui-lsp executable |

## Syntax Highlighting

The extension highlights:

| Element | Color |
|---------|-------|
| Comments (`;`) | Gray |
| Instructions (`= + - * / %`) | Keywords |
| Variables (`v0`, `g0`, `a0`) | Variables |
| Numbers | Constants |
| Strings (`"..."`) | Strings |
| Labels (`: 0`, `@ 1`) | Labels |
| Functions (`# 0 1 { }`) | Functions |

## Snippets

| Prefix | Description |
|--------|-------------|
| `func` | Function definition |
| `assign` | Variable assignment |
| `if` | Conditional pattern |
| `loop` | For loop pattern |
| `while` | While loop pattern |
| `print` | Output value |
| `call` | Function call |
| `array` | Create array |
| `aget` | Array read |
| `aset` | Array write |
| `ffi` | FFI call |
| `fibonacci` | Complete Fibonacci example |
| `fizzbuzz` | Complete FizzBuzz example |

## Example

```sui
; Fibonacci sequence
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

## Commands

| Command | Description |
|---------|-------------|
| `Sui: Restart Server` | Restart the language server |

## Troubleshooting

### Language server not starting

1. Ensure `sui-lsp` is in your PATH:
   ```bash
   which sui-lsp
   ```

2. If not found, install it:
   ```bash
   cargo install sui-lang --features lsp
   ```

3. Or set the full path in settings:
   ```json
   {
     "sui.lsp.path": "/path/to/sui-lsp"
   }
   ```

### Disable LSP

If you only want syntax highlighting without LSP:

```json
{
  "sui.lsp.enabled": false
}
```

## License

MIT
