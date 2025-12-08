//! Parser for the Sui programming language

use super::{Function, Instruction, Lexer};
use thiserror::Error;

/// Parser errors
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Invalid instruction '{0}' at line {1}")]
    InvalidInstruction(String, usize),

    #[error("Missing arguments for '{0}' at line {1}: expected {2}, got {3}")]
    MissingArguments(String, usize, usize, usize),

    #[error("Invalid function definition at line {0}")]
    InvalidFunctionDef(usize),

    #[error("Unmatched function brace at line {0}")]
    UnmatchedBrace(usize),

    #[error("Parse error at line {0}: {1}")]
    General(usize, String),
}

/// Parser for Sui source code
pub struct Parser;

impl Parser {
    /// Parse a single line of tokens into an instruction
    pub fn parse_line(tokens: &[String], line_num: usize) -> Result<Instruction, ParseError> {
        if tokens.is_empty() {
            return Ok(Instruction::Empty);
        }

        let op = tokens[0].as_str();
        let args: Vec<&str> = tokens[1..].iter().map(|s| s.as_str()).collect();

        match op {
            // Comment lines start with ;
            ";" => Ok(Instruction::Comment),

            // Import: _ "path/to/module.sui"
            "_" => {
                Self::check_args(op, &args, 1, line_num)?;
                // Remove quotes from path if present
                let path = args[0].trim_matches('"').to_string();
                Ok(Instruction::Import { path })
            }

            // Assignment: = var value
            "=" => {
                Self::check_args(op, &args, 2, line_num)?;
                Ok(Instruction::Assign {
                    target: args[0].to_string(),
                    value: args[1].to_string(),
                })
            }

            // Addition: + result a b
            "+" => {
                Self::check_args(op, &args, 3, line_num)?;
                Ok(Instruction::Add {
                    result: args[0].to_string(),
                    a: args[1].to_string(),
                    b: args[2].to_string(),
                })
            }

            // Subtraction: - result a b
            "-" => {
                Self::check_args(op, &args, 3, line_num)?;
                Ok(Instruction::Sub {
                    result: args[0].to_string(),
                    a: args[1].to_string(),
                    b: args[2].to_string(),
                })
            }

            // Multiplication: * result a b
            "*" => {
                Self::check_args(op, &args, 3, line_num)?;
                Ok(Instruction::Mul {
                    result: args[0].to_string(),
                    a: args[1].to_string(),
                    b: args[2].to_string(),
                })
            }

            // Division: / result a b
            "/" => {
                Self::check_args(op, &args, 3, line_num)?;
                Ok(Instruction::Div {
                    result: args[0].to_string(),
                    a: args[1].to_string(),
                    b: args[2].to_string(),
                })
            }

            // Modulo: % result a b
            "%" => {
                Self::check_args(op, &args, 3, line_num)?;
                Ok(Instruction::Mod {
                    result: args[0].to_string(),
                    a: args[1].to_string(),
                    b: args[2].to_string(),
                })
            }

            // Less than: < result a b
            "<" => {
                Self::check_args(op, &args, 3, line_num)?;
                Ok(Instruction::Lt {
                    result: args[0].to_string(),
                    a: args[1].to_string(),
                    b: args[2].to_string(),
                })
            }

            // Greater than: > result a b
            ">" => {
                Self::check_args(op, &args, 3, line_num)?;
                Ok(Instruction::Gt {
                    result: args[0].to_string(),
                    a: args[1].to_string(),
                    b: args[2].to_string(),
                })
            }

            // Equality: ~ result a b
            "~" => {
                Self::check_args(op, &args, 3, line_num)?;
                Ok(Instruction::Eq {
                    result: args[0].to_string(),
                    a: args[1].to_string(),
                    b: args[2].to_string(),
                })
            }

            // NOT: ! result a
            "!" => {
                Self::check_args(op, &args, 2, line_num)?;
                Ok(Instruction::Not {
                    result: args[0].to_string(),
                    a: args[1].to_string(),
                })
            }

            // AND: & result a b
            "&" => {
                Self::check_args(op, &args, 3, line_num)?;
                Ok(Instruction::And {
                    result: args[0].to_string(),
                    a: args[1].to_string(),
                    b: args[2].to_string(),
                })
            }

            // OR: | result a b
            "|" => {
                Self::check_args(op, &args, 3, line_num)?;
                Ok(Instruction::Or {
                    result: args[0].to_string(),
                    a: args[1].to_string(),
                    b: args[2].to_string(),
                })
            }

            // Conditional jump: ? cond label
            "?" => {
                Self::check_args(op, &args, 2, line_num)?;
                let label = args[1]
                    .parse()
                    .map_err(|_| ParseError::General(line_num, format!("Invalid label: {}", args[1])))?;
                Ok(Instruction::CondJump {
                    cond: args[0].to_string(),
                    label,
                })
            }

            // Unconditional jump: @ label
            "@" => {
                Self::check_args(op, &args, 1, line_num)?;
                let label = args[0]
                    .parse()
                    .map_err(|_| ParseError::General(line_num, format!("Invalid label: {}", args[0])))?;
                Ok(Instruction::Jump { label })
            }

            // Label definition: : label
            ":" => {
                Self::check_args(op, &args, 1, line_num)?;
                let id = args[0]
                    .parse()
                    .map_err(|_| ParseError::General(line_num, format!("Invalid label: {}", args[0])))?;
                Ok(Instruction::Label { id })
            }

            // Function definition: # id argc {
            "#" => {
                if args.len() < 3 || args.last() != Some(&"{") {
                    return Err(ParseError::InvalidFunctionDef(line_num));
                }
                let id = args[0]
                    .parse()
                    .map_err(|_| ParseError::General(line_num, format!("Invalid function id: {}", args[0])))?;
                let argc = args[1]
                    .parse()
                    .map_err(|_| ParseError::General(line_num, format!("Invalid argc: {}", args[1])))?;
                Ok(Instruction::FuncDef { id, argc })
            }

            // Function end: }
            "}" => Ok(Instruction::FuncEnd),

            // Function call: $ result func_id args...
            "$" => {
                Self::check_args(op, &args, 2, line_num)?;
                let func_id = args[1]
                    .parse()
                    .map_err(|_| ParseError::General(line_num, format!("Invalid function id: {}", args[1])))?;
                let call_args = args[2..].iter().map(|s| s.to_string()).collect();
                Ok(Instruction::Call {
                    result: args[0].to_string(),
                    func_id,
                    args: call_args,
                })
            }

            // Return: ^ value
            "^" => {
                Self::check_args(op, &args, 1, line_num)?;
                Ok(Instruction::Return {
                    value: args[0].to_string(),
                })
            }

            // Array create: [ var size
            "[" => {
                Self::check_args(op, &args, 2, line_num)?;
                Ok(Instruction::ArrayCreate {
                    var: args[0].to_string(),
                    size: args[1].to_string(),
                })
            }

            // Array read: ] result arr idx
            "]" => {
                Self::check_args(op, &args, 3, line_num)?;
                Ok(Instruction::ArrayRead {
                    result: args[0].to_string(),
                    arr: args[1].to_string(),
                    idx: args[2].to_string(),
                })
            }

            // Array write: { arr idx value (must have 3 args to distinguish from block start)
            "{" if args.len() >= 3 => {
                Ok(Instruction::ArrayWrite {
                    arr: args[0].to_string(),
                    idx: args[1].to_string(),
                    value: args[2].to_string(),
                })
            }

            // Block start (part of function def, or empty array write)
            "{" => Ok(Instruction::Empty),

            // Output: . value
            "." => {
                Self::check_args(op, &args, 1, line_num)?;
                Ok(Instruction::Output {
                    value: args[0].to_string(),
                })
            }

            // Input: , var
            "," => {
                Self::check_args(op, &args, 1, line_num)?;
                Ok(Instruction::Input {
                    var: args[0].to_string(),
                })
            }

            // Rust FFI: R result "func" args...
            // Also accept P for Python compatibility
            "R" | "P" => {
                Self::check_args(op, &args, 2, line_num)?;
                let func_args = args[2..].iter().map(|s| s.to_string()).collect();
                Ok(Instruction::RustFFI {
                    result: args[0].to_string(),
                    func: args[1].to_string(),
                    args: func_args,
                })
            }

            // Unknown instruction
            _ => Err(ParseError::InvalidInstruction(op.to_string(), line_num)),
        }
    }

    /// Check minimum argument count
    fn check_args(op: &str, args: &[&str], min: usize, line_num: usize) -> Result<(), ParseError> {
        if args.len() < min {
            Err(ParseError::MissingArguments(
                op.to_string(),
                line_num,
                min,
                args.len(),
            ))
        } else {
            Ok(())
        }
    }

    /// Parse complete source code into instructions and collect functions
    pub fn parse(code: &str) -> Result<(Vec<Instruction>, Vec<Function>), ParseError> {
        let token_lines = Lexer::parse(code);
        let mut instructions = Vec::new();
        let mut functions = Vec::new();

        let mut i = 0;
        let mut line_num = 1;

        while i < token_lines.len() {
            let tokens = &token_lines[i];
            let instr = Self::parse_line(tokens, line_num)?;

            match &instr {
                Instruction::FuncDef { id, argc } => {
                    // Collect function body
                    let func_id = *id;
                    let arg_count = *argc;
                    let mut body = Vec::new();
                    i += 1;
                    line_num += 1;
                    let mut depth = 1;

                    while i < token_lines.len() && depth > 0 {
                        let inner_tokens = &token_lines[i];
                        let inner_instr = Self::parse_line(inner_tokens, line_num)?;

                        match &inner_instr {
                            Instruction::FuncDef { .. } => {
                                depth += 1;
                                body.push(inner_instr);
                            }
                            Instruction::FuncEnd => {
                                depth -= 1;
                                if depth > 0 {
                                    body.push(inner_instr);
                                }
                            }
                            _ => {
                                body.push(inner_instr);
                            }
                        }

                        i += 1;
                        line_num += 1;
                    }

                    if depth != 0 {
                        return Err(ParseError::UnmatchedBrace(line_num));
                    }

                    functions.push(Function {
                        id: func_id,
                        arg_count,
                        body,
                    });
                }
                Instruction::FuncEnd => {
                    // Standalone } - skip
                    i += 1;
                    line_num += 1;
                }
                _ => {
                    instructions.push(instr);
                    i += 1;
                    line_num += 1;
                }
            }
        }

        Ok((instructions, functions))
    }

    /// Validate source code without executing
    pub fn validate(code: &str) -> Vec<ParseError> {
        let token_lines = Lexer::parse(code);
        let mut errors = Vec::new();

        for (i, tokens) in token_lines.iter().enumerate() {
            if let Err(e) = Self::parse_line(tokens, i + 1) {
                errors.push(e);
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_assignment() {
        let tokens = vec!["=".to_string(), "v0".to_string(), "10".to_string()];
        let instr = Parser::parse_line(&tokens, 1).unwrap();
        assert!(matches!(instr, Instruction::Assign { .. }));
    }

    #[test]
    fn test_parse_function_def() {
        let code = r#"
# 0 1 {
+ v0 a0 1
^ v0
}
"#;
        let (instrs, funcs) = Parser::parse(code).unwrap();
        assert!(instrs.is_empty());
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].id, 0);
        assert_eq!(funcs[0].arg_count, 1);
    }

    #[test]
    fn test_validate() {
        let code = "= v0 10\n+ v1 v0 5";
        let errors = Parser::validate(code);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_error() {
        let code = "= v0";
        let errors = Parser::validate(code);
        assert!(!errors.is_empty());
    }
}
