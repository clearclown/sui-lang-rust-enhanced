//! Runtime interpreter for the Sui programming language

use super::{Function, Instruction, Lexer, Parser, ParseError, Value};
use super::lexer::ParsedValue;
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use thiserror::Error;

/// Interpreter errors
#[derive(Debug, Error)]
pub enum InterpreterError {
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("Runtime error at line {line}: {message}")]
    Runtime { line: usize, message: String },

    #[error("Undefined function: {0}")]
    UndefinedFunction(i64),

    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("Array index out of bounds: {index} (length: {length})")]
    IndexOutOfBounds { index: i64, length: usize },

    #[error("Type error: {0}")]
    TypeError(String),

    #[error("Division by zero")]
    DivisionByZero,

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Stack overflow")]
    StackOverflow,
}

/// Execution context for a scope
#[derive(Debug, Clone, Default)]
struct Context {
    /// Local variables (v0, v1, ...)
    local_vars: HashMap<i64, Value>,
    /// Function arguments (a0, a1, ...)
    args: Vec<Value>,
    /// Return value
    return_value: Value,
    /// Whether return was called
    returned: bool,
}

/// Sui interpreter
pub struct Interpreter {
    /// Global variables (g0, g1, ...)
    global_vars: HashMap<i64, Value>,
    /// Function definitions
    functions: HashMap<i64, Function>,
    /// Context stack for nested calls
    context_stack: Vec<Context>,
    /// Current context
    context: Context,
    /// Output buffer
    output: Vec<String>,
    /// Maximum call stack depth
    max_stack_depth: usize,
    /// Debug mode
    debug: bool,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    /// Create a new interpreter
    pub fn new() -> Self {
        Self {
            global_vars: HashMap::new(),
            functions: HashMap::new(),
            context_stack: Vec::new(),
            context: Context::default(),
            output: Vec::new(),
            max_stack_depth: 1000,
            debug: false,
        }
    }

    /// Enable or disable debug mode
    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    /// Set maximum stack depth
    pub fn set_max_stack_depth(&mut self, depth: usize) {
        self.max_stack_depth = depth;
    }

    /// Reset interpreter state
    pub fn reset(&mut self) {
        self.global_vars.clear();
        self.functions.clear();
        self.context_stack.clear();
        self.context = Context::default();
        self.output.clear();
    }

    /// Resolve a value reference to an actual Value
    fn resolve(&self, val: &str) -> Value {
        match Lexer::parse_value(val) {
            ParsedValue::Variable(var) => {
                let prefix = var.chars().next().unwrap();
                let idx: i64 = var[1..].parse().unwrap_or(0);

                match prefix {
                    'v' => self.context.local_vars.get(&idx).cloned().unwrap_or_default(),
                    'g' => self.global_vars.get(&idx).cloned().unwrap_or_default(),
                    'a' => self.context.args.get(idx as usize).cloned().unwrap_or_default(),
                    _ => Value::default(),
                }
            }
            ParsedValue::Integer(n) => Value::Integer(n),
            ParsedValue::Float(f) => Value::Float(f),
            ParsedValue::String(s) => Value::String(s),
        }
    }

    /// Assign a value to a variable
    fn assign(&mut self, var: &str, value: Value) {
        let prefix = var.chars().next().unwrap_or('v');
        let idx: i64 = var[1..].parse().unwrap_or(0);

        match prefix {
            'v' => {
                self.context.local_vars.insert(idx, value);
            }
            'g' => {
                self.global_vars.insert(idx, value);
            }
            _ => {} // Can't assign to arguments
        }
    }

    /// Execute a single instruction
    fn execute_instruction(
        &mut self,
        instr: &Instruction,
    ) -> Result<(bool, Option<i64>), InterpreterError> {
        match instr {
            Instruction::Empty | Instruction::Comment | Instruction::FuncDef { .. } | Instruction::FuncEnd => {
                // No-op
            }

            Instruction::Assign { target, value } => {
                let val = self.resolve(value);
                self.assign(target, val);
            }

            Instruction::Add { result, a, b } => {
                let val = self.resolve(a).add(&self.resolve(b));
                self.assign(result, val);
            }

            Instruction::Sub { result, a, b } => {
                let val = self.resolve(a).sub(&self.resolve(b));
                self.assign(result, val);
            }

            Instruction::Mul { result, a, b } => {
                let val = self.resolve(a).mul(&self.resolve(b));
                self.assign(result, val);
            }

            Instruction::Div { result, a, b } => {
                let val = self.resolve(a).div(&self.resolve(b));
                self.assign(result, val);
            }

            Instruction::Mod { result, a, b } => {
                let val = self.resolve(a).modulo(&self.resolve(b));
                self.assign(result, val);
            }

            Instruction::Lt { result, a, b } => {
                let val = self.resolve(a).lt(&self.resolve(b));
                self.assign(result, val);
            }

            Instruction::Gt { result, a, b } => {
                let val = self.resolve(a).gt(&self.resolve(b));
                self.assign(result, val);
            }

            Instruction::Eq { result, a, b } => {
                let val = self.resolve(a).eq_val(&self.resolve(b));
                self.assign(result, val);
            }

            Instruction::Not { result, a } => {
                let val = if self.resolve(a).is_truthy() {
                    Value::Integer(0)
                } else {
                    Value::Integer(1)
                };
                self.assign(result, val);
            }

            Instruction::And { result, a, b } => {
                let val = if self.resolve(a).is_truthy() && self.resolve(b).is_truthy() {
                    Value::Integer(1)
                } else {
                    Value::Integer(0)
                };
                self.assign(result, val);
            }

            Instruction::Or { result, a, b } => {
                let val = if self.resolve(a).is_truthy() || self.resolve(b).is_truthy() {
                    Value::Integer(1)
                } else {
                    Value::Integer(0)
                };
                self.assign(result, val);
            }

            Instruction::CondJump { cond, label } => {
                if self.resolve(cond).is_truthy() {
                    return Ok((true, Some(*label)));
                }
            }

            Instruction::Jump { label } => {
                return Ok((true, Some(*label)));
            }

            Instruction::Label { .. } => {
                // Labels are handled during execution flow
            }

            Instruction::Call { result, func_id, args } => {
                // Check stack depth
                if self.context_stack.len() >= self.max_stack_depth {
                    return Err(InterpreterError::StackOverflow);
                }

                // Get function
                let func = self
                    .functions
                    .get(func_id)
                    .cloned()
                    .ok_or(InterpreterError::UndefinedFunction(*func_id))?;

                // Evaluate arguments
                let call_args: Vec<Value> = args.iter().map(|a| self.resolve(a)).collect();

                // Save context
                let old_context = std::mem::replace(
                    &mut self.context,
                    Context {
                        args: call_args,
                        ..Default::default()
                    },
                );
                self.context_stack.push(old_context);

                // Execute function body
                self.execute_block(&func.body)?;

                // Get return value
                let return_val = self.context.return_value.clone();

                // Restore context
                self.context = self.context_stack.pop().unwrap();

                // Store result
                self.assign(result, return_val);
            }

            Instruction::Return { value } => {
                self.context.return_value = self.resolve(value);
                self.context.returned = true;
                return Ok((false, None));
            }

            Instruction::ArrayCreate { var, size } => {
                let size = self.resolve(size).to_int() as usize;
                let arr = vec![Value::Integer(0); size];
                self.assign(var, Value::Array(arr));
            }

            Instruction::ArrayRead { result, arr, idx } => {
                let array = self.resolve(arr);
                let index = self.resolve(idx).to_int();

                let val = match array {
                    Value::Array(ref a) => {
                        if index >= 0 && (index as usize) < a.len() {
                            a[index as usize].clone()
                        } else {
                            Value::Integer(0)
                        }
                    }
                    _ => Value::Integer(0),
                };
                self.assign(result, val);
            }

            Instruction::ArrayWrite { arr, idx, value } => {
                let index = self.resolve(idx).to_int();
                let val = self.resolve(value);

                // Get the variable reference
                let prefix = arr.chars().next().unwrap_or('v');
                let var_idx: i64 = arr[1..].parse().unwrap_or(0);

                let array = match prefix {
                    'v' => self.context.local_vars.get_mut(&var_idx),
                    'g' => self.global_vars.get_mut(&var_idx),
                    _ => None,
                };

                if let Some(Value::Array(ref mut a)) = array {
                    if index >= 0 && (index as usize) < a.len() {
                        a[index as usize] = val;
                    }
                }
            }

            Instruction::Output { value } => {
                let val = self.resolve(value);
                let output = val.to_string();
                self.output.push(output.clone());
                println!("{}", output);
            }

            Instruction::Input { var } => {
                print!("> ");
                io::stdout().flush()?;

                let stdin = io::stdin();
                let line = stdin.lock().lines().next().unwrap_or(Ok(String::new()))?;

                let val = if let Ok(n) = line.trim().parse::<i64>() {
                    Value::Integer(n)
                } else if let Ok(f) = line.trim().parse::<f64>() {
                    Value::Float(f)
                } else {
                    Value::String(line.trim().to_string())
                };

                self.assign(var, val);
            }

            Instruction::RustFFI { result, func, args } => {
                let func_name = self.resolve(func).to_string();
                let resolved_args: Vec<Value> = args.iter().map(|a| self.resolve(a)).collect();
                let val = self.call_builtin(&func_name, &resolved_args);
                self.assign(result, val);
            }
        }

        Ok((true, None))
    }

    /// Call a built-in function (Rust FFI)
    fn call_builtin(&self, func: &str, args: &[Value]) -> Value {
        // Extract the function name from module.func format
        let func_name = func.rsplit('.').next().unwrap_or(func);

        match func_name {
            // Math functions
            "sqrt" => {
                let x = args.first().map(|v| v.to_float()).unwrap_or(0.0);
                Value::Float(x.sqrt())
            }
            "pow" => {
                let base = args.first().map(|v| v.to_float()).unwrap_or(0.0);
                let exp = args.get(1).map(|v| v.to_float()).unwrap_or(0.0);
                Value::Float(base.powf(exp))
            }
            "sin" => {
                let x = args.first().map(|v| v.to_float()).unwrap_or(0.0);
                Value::Float(x.sin())
            }
            "cos" => {
                let x = args.first().map(|v| v.to_float()).unwrap_or(0.0);
                Value::Float(x.cos())
            }
            "tan" => {
                let x = args.first().map(|v| v.to_float()).unwrap_or(0.0);
                Value::Float(x.tan())
            }
            "floor" => {
                let x = args.first().map(|v| v.to_float()).unwrap_or(0.0);
                Value::Integer(x.floor() as i64)
            }
            "ceil" => {
                let x = args.first().map(|v| v.to_float()).unwrap_or(0.0);
                Value::Integer(x.ceil() as i64)
            }
            "round" => {
                let x = args.first().map(|v| v.to_float()).unwrap_or(0.0);
                if args.len() >= 2 {
                    let decimals = args[1].to_int() as i32;
                    let factor = 10_f64.powi(decimals);
                    Value::Float((x * factor).round() / factor)
                } else {
                    Value::Integer(x.round() as i64)
                }
            }
            "abs" => {
                let x = args.first().map(|v| v.to_float()).unwrap_or(0.0);
                if x.fract() == 0.0 {
                    Value::Integer(x.abs() as i64)
                } else {
                    Value::Float(x.abs())
                }
            }
            "log" => {
                let x = args.first().map(|v| v.to_float()).unwrap_or(0.0);
                Value::Float(x.ln())
            }
            "log10" => {
                let x = args.first().map(|v| v.to_float()).unwrap_or(0.0);
                Value::Float(x.log10())
            }
            "exp" => {
                let x = args.first().map(|v| v.to_float()).unwrap_or(0.0);
                Value::Float(x.exp())
            }

            // Comparison/selection functions
            "max" => {
                if args.is_empty() {
                    return Value::Integer(0);
                }
                let mut max_val = args[0].to_float();
                for arg in &args[1..] {
                    let v = arg.to_float();
                    if v > max_val {
                        max_val = v;
                    }
                }
                if max_val.fract() == 0.0 {
                    Value::Integer(max_val as i64)
                } else {
                    Value::Float(max_val)
                }
            }
            "min" => {
                if args.is_empty() {
                    return Value::Integer(0);
                }
                let mut min_val = args[0].to_float();
                for arg in &args[1..] {
                    let v = arg.to_float();
                    if v < min_val {
                        min_val = v;
                    }
                }
                if min_val.fract() == 0.0 {
                    Value::Integer(min_val as i64)
                } else {
                    Value::Float(min_val)
                }
            }

            // String/length functions
            "len" => {
                if let Some(arg) = args.first() {
                    match arg {
                        Value::String(s) => Value::Integer(s.len() as i64),
                        Value::Array(a) => Value::Integer(a.len() as i64),
                        _ => Value::Integer(0),
                    }
                } else {
                    Value::Integer(0)
                }
            }

            // Type conversion
            "int" => {
                let x = args.first().map(|v| v.to_int()).unwrap_or(0);
                Value::Integer(x)
            }
            "float" => {
                let x = args.first().map(|v| v.to_float()).unwrap_or(0.0);
                Value::Float(x)
            }
            "str" => {
                let s = args.first().map(|v| v.to_string()).unwrap_or_default();
                Value::String(s)
            }

            // Random (simple pseudo-random)
            "randint" => {
                let min = args.first().map(|v| v.to_int()).unwrap_or(0);
                let max = args.get(1).map(|v| v.to_int()).unwrap_or(100);
                // Simple pseudo-random using time
                let seed = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos() as i64)
                    .unwrap_or(0);
                let range = (max - min + 1).max(1);
                Value::Integer(min + (seed.abs() % range))
            }

            // Unknown function
            _ => {
                eprintln!("Warning: Unknown builtin function '{}'", func);
                Value::Integer(0)
            }
        }
    }

    /// Execute a block of instructions
    fn execute_block(&mut self, instructions: &[Instruction]) -> Result<(), InterpreterError> {
        // Collect label positions
        let mut labels: HashMap<i64, usize> = HashMap::new();
        for (i, instr) in instructions.iter().enumerate() {
            if let Instruction::Label { id } = instr {
                labels.insert(*id, i);
            }
        }

        let mut i = 0;
        while i < instructions.len() {
            if self.context.returned {
                break;
            }

            let (cont, jump_label) = self.execute_instruction(&instructions[i])?;

            if !cont {
                break;
            }

            if let Some(label) = jump_label {
                if let Some(&pos) = labels.get(&label) {
                    i = pos;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        Ok(())
    }

    /// Run Sui code
    ///
    /// # Arguments
    /// * `code` - Sui source code
    /// * `args` - Command-line arguments (accessible as g100=argc, g101=argv[0], ...)
    ///
    /// # Returns
    /// Vector of output strings
    pub fn run(&mut self, code: &str, args: &[String]) -> Result<Vec<String>, InterpreterError> {
        self.reset();

        // Set command-line arguments
        // g100 = argc (number of arguments)
        // g101, g102, ... = argv[0], argv[1], ...
        self.global_vars.insert(100, Value::Integer(args.len() as i64));
        for (i, arg) in args.iter().enumerate() {
            let val = if let Ok(n) = arg.parse::<i64>() {
                Value::Integer(n)
            } else if let Ok(f) = arg.parse::<f64>() {
                Value::Float(f)
            } else {
                Value::String(arg.clone())
            };
            self.global_vars.insert(101 + i as i64, val);
        }

        // Parse code
        let (instructions, functions) = Parser::parse(code)?;

        // Store functions
        for func in functions {
            self.functions.insert(func.id, func);
        }

        // Execute main code
        self.execute_block(&instructions)?;

        Ok(self.output.clone())
    }

    /// Run a single line of code (for REPL)
    pub fn run_line(&mut self, line: &str) -> Result<Option<Value>, InterpreterError> {
        let tokens = Lexer::tokenize_line(line);
        if tokens.is_empty() {
            return Ok(None);
        }

        let instr = Parser::parse_line(&tokens, 1)?;

        match &instr {
            Instruction::Output { value } => {
                let val = self.resolve(value);
                self.output.push(val.to_string());
                println!("{}", val);
                Ok(Some(val))
            }
            _ => {
                self.execute_instruction(&instr)?;
                Ok(None)
            }
        }
    }

    /// Get current output
    pub fn get_output(&self) -> &[String] {
        &self.output
    }

    /// Get a global variable value
    pub fn get_global(&self, idx: i64) -> Option<&Value> {
        self.global_vars.get(&idx)
    }

    /// Set a global variable value
    pub fn set_global(&mut self, idx: i64, value: Value) {
        self.global_vars.insert(idx, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_assignment() {
        let mut interp = Interpreter::new();
        let code = "= g0 42\n. g0";
        let output = interp.run(code, &[]).unwrap();
        assert_eq!(output, vec!["42"]);
    }

    #[test]
    fn test_arithmetic() {
        let mut interp = Interpreter::new();
        let code = r#"
= v0 10
+ v1 v0 5
. v1
"#;
        let output = interp.run(code, &[]).unwrap();
        assert_eq!(output, vec!["15"]);
    }

    #[test]
    fn test_loop() {
        let mut interp = Interpreter::new();
        let code = r#"
= v0 0
: 0
< v1 v0 5
! v2 v1
? v2 1
. v0
+ v0 v0 1
@ 0
: 1
"#;
        let output = interp.run(code, &[]).unwrap();
        assert_eq!(output, vec!["0", "1", "2", "3", "4"]);
    }

    #[test]
    fn test_function() {
        let mut interp = Interpreter::new();
        let code = r#"
# 0 1 {
+ v0 a0 1
^ v0
}
$ g0 0 5
. g0
"#;
        let output = interp.run(code, &[]).unwrap();
        assert_eq!(output, vec!["6"]);
    }

    #[test]
    fn test_fibonacci() {
        let mut interp = Interpreter::new();
        let code = r#"
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
"#;
        let output = interp.run(code, &[]).unwrap();
        assert_eq!(output, vec!["55"]);
    }

    #[test]
    fn test_array() {
        let mut interp = Interpreter::new();
        let code = r#"
[ v0 5
{ v0 2 42
] v1 v0 2
. v1
"#;
        let output = interp.run(code, &[]).unwrap();
        assert_eq!(output, vec!["42"]);
    }

    #[test]
    fn test_string_output() {
        let mut interp = Interpreter::new();
        let code = r#"
. "Hello World"
"#;
        let output = interp.run(code, &[]).unwrap();
        assert_eq!(output, vec!["Hello World"]);
    }

    #[test]
    fn test_command_line_args() {
        let mut interp = Interpreter::new();
        let code = r#"
. g100
. g101
"#;
        let output = interp.run(code, &["42".to_string()]).unwrap();
        assert_eq!(output, vec!["1", "42"]);
    }
}
