//! Sui to Python transpiler

use super::{TranspileError, Transpiler};
use crate::interpreter::{Parser, Instruction};
use std::collections::{HashMap, HashSet};

/// Sui to Python transpiler
pub struct Sui2Py {
    indent: usize,
    output: Vec<String>,
}

impl Default for Sui2Py {
    fn default() -> Self {
        Self::new()
    }
}

impl Sui2Py {
    /// Create a new transpiler
    pub fn new() -> Self {
        Self {
            indent: 0,
            output: Vec::new(),
        }
    }

    /// Emit a line with current indentation
    fn emit(&mut self, line: &str) {
        let indent_str = "    ".repeat(self.indent);
        self.output.push(format!("{}{}", indent_str, line));
    }

    /// Resolve a value to Python expression
    fn resolve_value(&self, val: &str) -> String {
        // Variables and literals are passed through
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
            self.emit("_state = -1");
            self.emit("while True:");
            self.indent += 1;
            self.emit("_state += 1");

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

            for state_id in sorted_states {
                self.emit(&format!("if _state == {}:", state_id));
                self.indent += 1;

                let state_lines = states.get(&state_id).map(|v| v.as_slice()).unwrap_or(&[]);
                if state_lines.is_empty() {
                    self.emit("pass");
                } else {
                    for instr in state_lines {
                        self.transpile_instruction(instr, &state_map, is_function);
                    }
                }

                // State transition
                let _last_op = state_lines.last().map(|i| std::mem::discriminant(*i));
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
                        self.emit(&format!("_state = {} - 1", next_state));
                        self.emit("continue");
                    } else {
                        self.emit("break");
                    }
                }

                self.indent -= 1;
            }

            self.emit("break");
            self.indent -= 1;
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
                self.emit(&format!("{} = {}", target, self.resolve_value(value)));
            }

            Instruction::Add { result, a, b } => {
                self.emit(&format!(
                    "{} = {} + {}",
                    result,
                    self.resolve_value(a),
                    self.resolve_value(b)
                ));
            }

            Instruction::Sub { result, a, b } => {
                self.emit(&format!(
                    "{} = {} - {}",
                    result,
                    self.resolve_value(a),
                    self.resolve_value(b)
                ));
            }

            Instruction::Mul { result, a, b } => {
                self.emit(&format!(
                    "{} = {} * {}",
                    result,
                    self.resolve_value(a),
                    self.resolve_value(b)
                ));
            }

            Instruction::Div { result, a, b } => {
                self.emit(&format!(
                    "{} = {} / {}",
                    result,
                    self.resolve_value(a),
                    self.resolve_value(b)
                ));
            }

            Instruction::Mod { result, a, b } => {
                self.emit(&format!(
                    "{} = {} % {}",
                    result,
                    self.resolve_value(a),
                    self.resolve_value(b)
                ));
            }

            Instruction::Lt { result, a, b } => {
                self.emit(&format!(
                    "{} = 1 if {} < {} else 0",
                    result,
                    self.resolve_value(a),
                    self.resolve_value(b)
                ));
            }

            Instruction::Gt { result, a, b } => {
                self.emit(&format!(
                    "{} = 1 if {} > {} else 0",
                    result,
                    self.resolve_value(a),
                    self.resolve_value(b)
                ));
            }

            Instruction::Eq { result, a, b } => {
                self.emit(&format!(
                    "{} = 1 if {} == {} else 0",
                    result,
                    self.resolve_value(a),
                    self.resolve_value(b)
                ));
            }

            Instruction::Not { result, a } => {
                self.emit(&format!(
                    "{} = 0 if {} else 1",
                    result,
                    self.resolve_value(a)
                ));
            }

            Instruction::And { result, a, b } => {
                self.emit(&format!(
                    "{} = 1 if ({} and {}) else 0",
                    result,
                    self.resolve_value(a),
                    self.resolve_value(b)
                ));
            }

            Instruction::Or { result, a, b } => {
                self.emit(&format!(
                    "{} = 1 if ({} or {}) else 0",
                    result,
                    self.resolve_value(a),
                    self.resolve_value(b)
                ));
            }

            Instruction::CondJump { cond, label } => {
                if let Some(&state) = state_map.get(label) {
                    self.emit(&format!("if {}:", self.resolve_value(cond)));
                    self.indent += 1;
                    self.emit(&format!("_state = {} - 1", state));
                    self.emit("continue");
                    self.indent -= 1;
                }
            }

            Instruction::Jump { label } => {
                if let Some(&state) = state_map.get(label) {
                    self.emit(&format!("_state = {} - 1", state));
                    self.emit("continue");
                }
            }

            Instruction::FuncDef { .. } | Instruction::FuncEnd => {}

            Instruction::Call { result, func_id, args } => {
                let args_str = args
                    .iter()
                    .map(|a| self.resolve_value(a))
                    .collect::<Vec<_>>()
                    .join(", ");
                self.emit(&format!("{} = f{}({})", result, func_id, args_str));
            }

            Instruction::Return { value } => {
                self.emit(&format!("return {}", self.resolve_value(value)));
            }

            Instruction::ArrayCreate { var, size } => {
                self.emit(&format!("{} = [0] * {}", var, self.resolve_value(size)));
            }

            Instruction::ArrayRead { result, arr, idx } => {
                self.emit(&format!(
                    "{} = {}[int({})]",
                    result,
                    self.resolve_value(arr),
                    self.resolve_value(idx)
                ));
            }

            Instruction::ArrayWrite { arr, idx, value } => {
                self.emit(&format!(
                    "{}[int({})] = {}",
                    self.resolve_value(arr),
                    self.resolve_value(idx),
                    self.resolve_value(value)
                ));
            }

            Instruction::Output { value } => {
                self.emit(&format!("print({})", self.resolve_value(value)));
            }

            Instruction::Input { var } => {
                self.emit("_input = input()");
                self.emit("try:");
                self.indent += 1;
                self.emit(&format!("{} = int(_input)", var));
                self.indent -= 1;
                self.emit("except ValueError:");
                self.indent += 1;
                self.emit(&format!("{} = _input", var));
                self.indent -= 1;
            }

            Instruction::RustFFI { result, func, args } => {
                // Generate Python FFI call
                let args_str = args
                    .iter()
                    .map(|a| self.resolve_value(a))
                    .collect::<Vec<_>>()
                    .join(", ");
                // Extract module and function name
                let func_str = self.resolve_value(func);
                // Remove quotes if present
                let func_clean = func_str.trim_matches('"');

                if func_clean.contains('.') {
                    // Module function: import and call
                    let parts: Vec<&str> = func_clean.rsplitn(2, '.').collect();
                    let func_name = parts[0];
                    let module = parts.get(1).unwrap_or(&"");
                    self.emit(&format!("import {}", module));
                    self.emit(&format!("{} = {}.{}({})", result, module, func_name, args_str));
                } else {
                    // Builtin function
                    self.emit(&format!("{} = {}({})", result, func_clean, args_str));
                }
            }
        }
    }

    /// Transpile Sui code to Python
    pub fn transpile_to_python(&mut self, code: &str) -> Result<String, TranspileError> {
        self.output.clear();
        self.indent = 0;

        // Parse the code
        let (instructions, functions) =
            Parser::parse(code).map_err(|e| TranspileError::Parse(e.to_string()))?;

        // Header
        self.emit("#!/usr/bin/env python3");
        self.emit("# Auto-generated from Sui");
        self.emit("");

        // Global variables from command-line arguments
        self.emit("# Global variables from command-line arguments");
        self.emit("import sys");
        self.emit("g100 = len(sys.argv) - 1");
        self.emit("for _i, _arg in enumerate(sys.argv[1:]):");
        self.indent += 1;
        self.emit("try:");
        self.indent += 1;
        self.emit("globals()[f'g{101 + _i}'] = int(_arg)");
        self.indent -= 1;
        self.emit("except ValueError:");
        self.indent += 1;
        self.emit("globals()[f'g{101 + _i}'] = _arg");
        self.indent -= 1;
        self.indent -= 1;
        self.emit("");

        // Output function definitions
        for func in &functions {
            let args_str = (0..func.arg_count)
                .map(|i| format!("a{}", i))
                .collect::<Vec<_>>()
                .join(", ");
            self.emit(&format!("def f{}({}):", func.id, args_str));
            self.indent += 1;

            if func.body.is_empty() {
                self.emit("pass");
            } else {
                self.transpile_block(&func.body, true);
            }

            self.indent -= 1;
            self.emit("");
        }

        // Output main code
        self.emit("# Main");
        if instructions.is_empty() {
            self.emit("pass");
        } else {
            self.transpile_block(&instructions, false);
        }

        Ok(self.output.join("\n"))
    }
}

impl Transpiler for Sui2Py {
    fn transpile(&self, code: &str) -> Result<String, TranspileError> {
        let mut transpiler = Sui2Py::new();
        transpiler.transpile_to_python(code)
    }

    fn extension(&self) -> &str {
        "py"
    }

    fn language(&self) -> &str {
        "Python"
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
        let mut transpiler = Sui2Py::new();
        let result = transpiler.transpile_to_python(code).unwrap();
        assert!(result.contains("v0 = 10"));
        assert!(result.contains("v1 = v0 + 5"));
        assert!(result.contains("print(v1)"));
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
        let mut transpiler = Sui2Py::new();
        let result = transpiler.transpile_to_python(code).unwrap();
        assert!(result.contains("def f0(a0):"));
        assert!(result.contains("g0 = f0(5)"));
    }
}
