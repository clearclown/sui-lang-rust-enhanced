//! Sui to JavaScript transpiler

use super::{TranspileError, Transpiler};
use crate::interpreter::{Instruction, Parser};
use std::collections::{HashMap, HashSet};

/// Sui to JavaScript transpiler
pub struct Sui2Js {
    indent: usize,
    output: Vec<String>,
    /// Whether to generate Node.js compatible code
    nodejs: bool,
    /// Whether to generate ES modules
    esm: bool,
}

impl Default for Sui2Js {
    fn default() -> Self {
        Self::new()
    }
}

impl Sui2Js {
    /// Create a new transpiler
    pub fn new() -> Self {
        Self {
            indent: 0,
            output: Vec::new(),
            nodejs: true,
            esm: false,
        }
    }

    /// Set Node.js compatibility mode
    pub fn set_nodejs(&mut self, nodejs: bool) {
        self.nodejs = nodejs;
    }

    /// Set ES modules mode
    pub fn set_esm(&mut self, esm: bool) {
        self.esm = esm;
    }

    /// Emit a line with current indentation
    fn emit(&mut self, line: &str) {
        let indent_str = "  ".repeat(self.indent);
        self.output.push(format!("{}{}", indent_str, line));
    }

    /// Resolve a value to JavaScript expression
    fn resolve_value(&self, val: &str) -> String {
        val.to_string()
    }

    /// Transpile a block of instructions
    fn transpile_block(&mut self, instructions: &[Instruction], is_function: bool) {
        // Collect labels
        let labels: HashSet<i64> = instructions
            .iter()
            .filter_map(|instr| {
                if let Instruction::Label { id } = instr {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();

        // Use state machine pattern if labels exist
        if !labels.is_empty() {
            self.emit("let _state = -1;");
            self.emit("while (true) {");
            self.indent += 1;
            self.emit("_state++;");

            // Map labels to state numbers
            let mut state_map: HashMap<i64, usize> = HashMap::new();
            state_map.insert(-1, 0);
            let mut state_num = 1;

            for label in labels.iter() {
                state_map.insert(*label, state_num);
                state_num += 1;
            }

            // Group instructions by state
            let mut states: HashMap<usize, Vec<&Instruction>> = HashMap::new();
            states.insert(0, Vec::new());
            let mut current = 0;

            for instr in instructions {
                match instr {
                    Instruction::Label { id } => {
                        current = *state_map.get(id).unwrap_or(&0);
                        states.entry(current).or_default();
                    }
                    Instruction::FuncEnd => {}
                    _ => {
                        states.entry(current).or_default().push(instr);
                    }
                }
            }

            // Generate code for each state
            let mut sorted_states: Vec<_> = states.keys().copied().collect();
            sorted_states.sort();

            self.emit("switch (_state) {");
            self.indent += 1;

            for state_id in sorted_states {
                self.emit(&format!("case {}:", state_id));
                self.indent += 1;

                let state_lines = states.get(&state_id).map(|v| v.as_slice()).unwrap_or(&[]);
                for instr in state_lines {
                    self.transpile_instruction(instr, &state_map, is_function);
                }

                // State transition
                let needs_transition = state_lines.is_empty()
                    || !matches!(
                        state_lines.last(),
                        Some(Instruction::CondJump { .. })
                            | Some(Instruction::Jump { .. })
                            | Some(Instruction::Return { .. })
                    );

                if needs_transition {
                    let next_state = state_id + 1;
                    if states.contains_key(&next_state) {
                        self.emit(&format!("_state = {} - 1;", next_state));
                        self.emit("continue;");
                    } else {
                        self.emit("break;");
                    }
                }

                self.indent -= 1;
            }

            self.indent -= 1;
            self.emit("}");
            self.emit("break;");
            self.indent -= 1;
            self.emit("}");
        } else {
            // Simple case: no labels
            for instr in instructions {
                if !matches!(instr, Instruction::FuncEnd) {
                    self.transpile_instruction(instr, &HashMap::new(), is_function);
                }
            }
        }
    }

    /// Transpile a single instruction
    fn transpile_instruction(
        &mut self,
        instr: &Instruction,
        state_map: &HashMap<i64, usize>,
        _is_function: bool,
    ) {
        match instr {
            Instruction::Empty | Instruction::Comment | Instruction::Label { .. } | Instruction::Import { .. } => {
                // Import is handled at runtime, skip in transpilation
            }

            Instruction::Assign { target, value } => {
                self.emit(&format!("{} = {};", target, self.resolve_value(value)));
            }

            Instruction::Add { result, a, b } => {
                self.emit(&format!(
                    "{} = {} + {};",
                    result,
                    self.resolve_value(a),
                    self.resolve_value(b)
                ));
            }

            Instruction::Sub { result, a, b } => {
                self.emit(&format!(
                    "{} = {} - {};",
                    result,
                    self.resolve_value(a),
                    self.resolve_value(b)
                ));
            }

            Instruction::Mul { result, a, b } => {
                self.emit(&format!(
                    "{} = {} * {};",
                    result,
                    self.resolve_value(a),
                    self.resolve_value(b)
                ));
            }

            Instruction::Div { result, a, b } => {
                self.emit(&format!(
                    "{} = {} / {};",
                    result,
                    self.resolve_value(a),
                    self.resolve_value(b)
                ));
            }

            Instruction::Mod { result, a, b } => {
                self.emit(&format!(
                    "{} = {} % {};",
                    result,
                    self.resolve_value(a),
                    self.resolve_value(b)
                ));
            }

            Instruction::Lt { result, a, b } => {
                self.emit(&format!(
                    "{} = {} < {} ? 1 : 0;",
                    result,
                    self.resolve_value(a),
                    self.resolve_value(b)
                ));
            }

            Instruction::Gt { result, a, b } => {
                self.emit(&format!(
                    "{} = {} > {} ? 1 : 0;",
                    result,
                    self.resolve_value(a),
                    self.resolve_value(b)
                ));
            }

            Instruction::Eq { result, a, b } => {
                self.emit(&format!(
                    "{} = {} === {} ? 1 : 0;",
                    result,
                    self.resolve_value(a),
                    self.resolve_value(b)
                ));
            }

            Instruction::Not { result, a } => {
                self.emit(&format!("{} = {} ? 0 : 1;", result, self.resolve_value(a)));
            }

            Instruction::And { result, a, b } => {
                self.emit(&format!(
                    "{} = ({} && {}) ? 1 : 0;",
                    result,
                    self.resolve_value(a),
                    self.resolve_value(b)
                ));
            }

            Instruction::Or { result, a, b } => {
                self.emit(&format!(
                    "{} = ({} || {}) ? 1 : 0;",
                    result,
                    self.resolve_value(a),
                    self.resolve_value(b)
                ));
            }

            Instruction::CondJump { cond, label } => {
                if let Some(&state) = state_map.get(label) {
                    self.emit(&format!("if ({}) {{", self.resolve_value(cond)));
                    self.indent += 1;
                    self.emit(&format!("_state = {} - 1;", state));
                    self.emit("continue;");
                    self.indent -= 1;
                    self.emit("}");
                }
            }

            Instruction::Jump { label } => {
                if let Some(&state) = state_map.get(label) {
                    self.emit(&format!("_state = {} - 1;", state));
                    self.emit("continue;");
                }
            }

            Instruction::FuncDef { .. } | Instruction::FuncEnd => {}

            Instruction::Call { result, func_id, args } => {
                let args_str = args
                    .iter()
                    .map(|a| self.resolve_value(a))
                    .collect::<Vec<_>>()
                    .join(", ");
                self.emit(&format!("{} = f{}({});", result, func_id, args_str));
            }

            Instruction::Return { value } => {
                self.emit(&format!("return {};", self.resolve_value(value)));
            }

            Instruction::ArrayCreate { var, size } => {
                self.emit(&format!(
                    "{} = new Array({}).fill(0);",
                    var,
                    self.resolve_value(size)
                ));
            }

            Instruction::ArrayRead { result, arr, idx } => {
                self.emit(&format!(
                    "{} = {}[Math.floor({})];",
                    result,
                    self.resolve_value(arr),
                    self.resolve_value(idx)
                ));
            }

            Instruction::ArrayWrite { arr, idx, value } => {
                self.emit(&format!(
                    "{}[Math.floor({})] = {};",
                    self.resolve_value(arr),
                    self.resolve_value(idx),
                    self.resolve_value(value)
                ));
            }

            Instruction::Output { value } => {
                self.emit(&format!("console.log({});", self.resolve_value(value)));
            }

            Instruction::Input { var } => {
                if self.nodejs {
                    self.emit(&format!(
                        "{} = parseInt(require('readline-sync').question('> ')) || 0;",
                        var
                    ));
                } else {
                    self.emit(&format!("{} = parseInt(prompt('> ')) || 0;", var));
                }
            }

            Instruction::RustFFI { result, func, args } => {
                // Generate JavaScript FFI call
                let args_str = args
                    .iter()
                    .map(|a| self.resolve_value(a))
                    .collect::<Vec<_>>()
                    .join(", ");
                let func_str = self.resolve_value(func);
                // Remove quotes if present
                let func_clean = func_str.trim_matches('"');

                // Map Python/Rust functions to JavaScript equivalents
                let js_call = match func_clean {
                    // Math functions
                    "math.sqrt" => format!("Math.sqrt({})", args_str),
                    "math.pow" => format!("Math.pow({})", args_str),
                    "math.sin" => format!("Math.sin({})", args_str),
                    "math.cos" => format!("Math.cos({})", args_str),
                    "math.tan" => format!("Math.tan({})", args_str),
                    "math.abs" | "abs" => format!("Math.abs({})", args_str),
                    "math.floor" => format!("Math.floor({})", args_str),
                    "math.ceil" => format!("Math.ceil({})", args_str),
                    "math.round" | "round" => format!("Math.round({})", args_str),
                    "max" => format!("Math.max({})", args_str),
                    "min" => format!("Math.min({})", args_str),
                    // String/type functions
                    "len" => {
                        if let Some(arg) = args.first() {
                            format!("{}.length", self.resolve_value(arg))
                        } else {
                            "0".to_string()
                        }
                    }
                    "int" => format!("parseInt({})", args_str),
                    "float" => format!("parseFloat({})", args_str),
                    "str" => format!("String({})", args_str),
                    // Random
                    "random.randint" => {
                        if args.len() >= 2 {
                            let a = self.resolve_value(&args[0]);
                            let b = self.resolve_value(&args[1]);
                            format!(
                                "Math.floor(Math.random() * ({} - {} + 1)) + {}",
                                b, a, a
                            )
                        } else {
                            "0".to_string()
                        }
                    }
                    // Default: try to call as-is
                    _ => {
                        if func_clean.contains('.') {
                            format!("{}({})", func_clean, args_str)
                        } else {
                            format!("{}({})", func_clean, args_str)
                        }
                    }
                };

                self.emit(&format!("{} = {};", result, js_call));
            }
        }
    }

    /// Transpile Sui code to JavaScript
    pub fn transpile_to_js(&mut self, code: &str) -> Result<String, TranspileError> {
        self.output.clear();
        self.indent = 0;

        // Parse the code
        let (instructions, functions) =
            Parser::parse(code).map_err(|e| TranspileError::Parse(e.to_string()))?;

        // Header
        self.emit("// Auto-generated from Sui");
        if self.esm {
            self.emit("// ES Module");
        }
        self.emit("");

        // Global variables from command-line arguments
        self.emit("// Global variables from command-line arguments");
        if self.nodejs {
            self.emit("const _args = process.argv.slice(2);");
        } else {
            self.emit("const _args = [];");
        }
        self.emit("let g100 = _args.length;");
        self.emit("for (let _i = 0; _i < _args.length; _i++) {");
        self.indent += 1;
        self.emit("const _val = parseInt(_args[_i]);");
        self.emit("globalThis[`g${101 + _i}`] = isNaN(_val) ? _args[_i] : _val;");
        self.indent -= 1;
        self.emit("}");
        self.emit("");

        // Declare all variables
        self.emit("// Variable declarations");
        self.emit("let v0, v1, v2, v3, v4, v5, v6, v7, v8, v9;");
        self.emit("let g0, g1, g2, g3, g4, g5, g6, g7, g8, g9;");
        self.emit("");

        // Output function definitions
        for func in &functions {
            let args_str = (0..func.arg_count)
                .map(|i| format!("a{}", i))
                .collect::<Vec<_>>()
                .join(", ");
            self.emit(&format!("function f{}({}) {{", func.id, args_str));
            self.indent += 1;

            // Declare local variables
            self.emit("let v0, v1, v2, v3, v4, v5, v6, v7, v8, v9;");

            if !func.body.is_empty() {
                self.transpile_block(&func.body, true);
            }

            self.indent -= 1;
            self.emit("}");
            self.emit("");
        }

        // Output main code
        self.emit("// Main");
        if !instructions.is_empty() {
            self.transpile_block(&instructions, false);
        }

        Ok(self.output.join("\n"))
    }
}

impl Transpiler for Sui2Js {
    fn transpile(&self, code: &str) -> Result<String, TranspileError> {
        let mut transpiler = Sui2Js::new();
        transpiler.transpile_to_js(code)
    }

    fn extension(&self) -> &str {
        "js"
    }

    fn language(&self) -> &str {
        "JavaScript"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_transpile() {
        let code = r#"
= v0 10
+ v1 v0 5
. v1
"#;
        let mut transpiler = Sui2Js::new();
        let result = transpiler.transpile_to_js(code).unwrap();
        assert!(result.contains("v0 = 10;"));
        assert!(result.contains("v1 = v0 + 5;"));
        assert!(result.contains("console.log(v1);"));
    }

    #[test]
    fn test_function_transpile() {
        let code = r#"
# 0 1 {
+ v0 a0 1
^ v0
}
$ g0 0 5
. g0
"#;
        let mut transpiler = Sui2Js::new();
        let result = transpiler.transpile_to_js(code).unwrap();
        assert!(result.contains("function f0(a0)"));
        assert!(result.contains("g0 = f0(5);"));
    }
}
