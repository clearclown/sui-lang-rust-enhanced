//! # Sui (粋) - A Programming Language for LLMs
//!
//! Sui is a line-based programming language optimized for accurate LLM code generation.
//!
//! ## Features
//!
//! - **Line Independence**: Each line is completely self-contained
//! - **Minimal Bracket Matching**: Nesting only for function blocks `{}`
//! - **Single-Character Instructions**: Maximum token efficiency
//! - **Sequential Variables**: No meaningful names needed (v0, v1, g0, a0)
//! - **Explicit Control Flow**: Labels and jumps
//!
//! ## Example
//!
//! ```rust
//! use sui_lang::Interpreter;
//!
//! let code = r#"
//! = v0 10
//! + v1 v0 5
//! . v1
//! "#;
//!
//! let mut interpreter = Interpreter::new();
//! let output = interpreter.run(code, &[]).unwrap();
//! assert_eq!(output, vec!["15"]);
//! ```

pub mod interpreter;
pub mod transpiler;
pub mod debugger;

#[cfg(feature = "repl")]
pub mod repl;

#[cfg(feature = "wasm")]
pub mod wasm;

// Re-exports for convenience
pub use interpreter::{Interpreter, InterpreterError, Value};
pub use transpiler::{Sui2Py, Sui2Js, Py2Sui, TranspileError};
pub use debugger::Debugger;

/// Sui language version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude module for common imports
pub mod prelude {
    pub use crate::interpreter::{Interpreter, InterpreterError, Value};
    pub use crate::transpiler::{Sui2Py, Sui2Js, Py2Sui, TranspileError};
}
