//! Transpiler module for Sui language
//!
//! This module provides transpilers to convert Sui code to other languages,
//! and from other languages to Sui.

mod sui2py;
mod sui2js;
mod py2sui;

pub use sui2py::Sui2Py;
pub use sui2js::Sui2Js;
pub use py2sui::Py2Sui;

use thiserror::Error;

/// Transpiler errors
#[derive(Debug, Error)]
pub enum TranspileError {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Invalid instruction at line {line}: {message}")]
    InvalidInstruction { line: usize, message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Common trait for transpilers
pub trait Transpiler {
    /// Transpile Sui code to target language
    fn transpile(&self, code: &str) -> Result<String, TranspileError>;

    /// Get the file extension for the target language
    fn extension(&self) -> &str;

    /// Get the target language name
    fn language(&self) -> &str;
}
