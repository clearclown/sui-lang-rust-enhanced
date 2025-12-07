//! Sui Interpreter Module
//!
//! This module contains the core interpreter for the Sui programming language.

mod lexer;
mod parser;
mod runtime;
mod value;

pub use lexer::Lexer;
pub use parser::{Parser, ParseError};
pub use runtime::{Interpreter, InterpreterError};
pub use value::Value;

/// Token types for the Sui language
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Instruction character (=, +, -, etc.)
    Instruction(char),
    /// Variable (v0, g1, a2)
    Variable(String),
    /// Integer literal
    Integer(i64),
    /// Float literal
    Float(f64),
    /// String literal
    String(String),
    /// Label number
    Label(i64),
    /// Opening brace for function definition
    OpenBrace,
    /// Closing brace
    CloseBrace,
}

/// Instruction types
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// Assignment: = var value
    Assign { target: String, value: String },
    /// Addition: + result a b
    Add { result: String, a: String, b: String },
    /// Subtraction: - result a b
    Sub { result: String, a: String, b: String },
    /// Multiplication: * result a b
    Mul { result: String, a: String, b: String },
    /// Division: / result a b
    Div { result: String, a: String, b: String },
    /// Modulo: % result a b
    Mod { result: String, a: String, b: String },
    /// Less than: < result a b
    Lt { result: String, a: String, b: String },
    /// Greater than: > result a b
    Gt { result: String, a: String, b: String },
    /// Equality: ~ result a b
    Eq { result: String, a: String, b: String },
    /// NOT: ! result a
    Not { result: String, a: String },
    /// AND: & result a b
    And { result: String, a: String, b: String },
    /// OR: | result a b
    Or { result: String, a: String, b: String },
    /// Conditional jump: ? cond label
    CondJump { cond: String, label: i64 },
    /// Unconditional jump: @ label
    Jump { label: i64 },
    /// Label definition: : label
    Label { id: i64 },
    /// Function definition: # id argc {
    FuncDef { id: i64, argc: i64 },
    /// Function end: }
    FuncEnd,
    /// Function call: $ result func_id args...
    Call { result: String, func_id: i64, args: Vec<String> },
    /// Return: ^ value
    Return { value: String },
    /// Array create: [ var size
    ArrayCreate { var: String, size: String },
    /// Array read: ] result arr idx
    ArrayRead { result: String, arr: String, idx: String },
    /// Array write: { arr idx value
    ArrayWrite { arr: String, idx: String, value: String },
    /// Output: . value
    Output { value: String },
    /// Input: , var
    Input { var: String },
    /// Rust FFI: R result "func" args...
    RustFFI { result: String, func: String, args: Vec<String> },
    /// Comment (ignored)
    Comment,
    /// Empty line (ignored)
    Empty,
}

/// Function definition storage
#[derive(Debug, Clone)]
pub struct Function {
    pub id: i64,
    pub arg_count: i64,
    pub body: Vec<Instruction>,
}
