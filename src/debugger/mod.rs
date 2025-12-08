//! Step debugger for Sui language
//!
//! Provides interactive debugging capabilities:
//! - Breakpoints (by line number)
//! - Step/Next/Continue
//! - Variable inspection
//! - Call stack viewing

use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead, Write};

use crate::interpreter::{Function, Instruction, Lexer, Parser, ParseError, ParsedValue, Value};

/// Debugger state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugState {
    /// Running normally
    Running,
    /// Paused at breakpoint or step
    Paused,
    /// Single-stepping
    Stepping,
    /// Finished running
    Finished,
}

/// Debug event
#[derive(Debug, Clone)]
pub enum DebugEvent {
    /// Hit a breakpoint
    Breakpoint(usize),
    /// Step completed
    Step,
    /// Finished running
    Finished,
    /// Error occurred
    Error(String),
}

/// Stack frame for debugging
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Function ID (-1 for main)
    pub func_id: i64,
    /// Current line in the function
    pub line: usize,
    /// Local variables
    pub locals: HashMap<i64, Value>,
    /// Function arguments
    pub args: Vec<Value>,
}

/// Sui debugger
pub struct Debugger {
    breakpoints: HashSet<usize>,
    state: DebugState,
    current_line: usize,
    instructions: Vec<(usize, Instruction)>,
    functions: HashMap<i64, Function>,
    global_vars: HashMap<i64, Value>,
    call_stack: Vec<StackFrame>,
    current_frame: StackFrame,
    output: Vec<String>,
    labels: HashMap<i64, usize>,
    ip: usize,
    source_lines: Vec<String>,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            breakpoints: HashSet::new(),
            state: DebugState::Paused,
            current_line: 0,
            instructions: Vec::new(),
            functions: HashMap::new(),
            global_vars: HashMap::new(),
            call_stack: Vec::new(),
            current_frame: StackFrame {
                func_id: -1,
                line: 0,
                locals: HashMap::new(),
                args: Vec::new(),
            },
            output: Vec::new(),
            labels: HashMap::new(),
            ip: 0,
            source_lines: Vec::new(),
        }
    }

    pub fn load(&mut self, code: &str) -> Result<(), ParseError> {
        self.source_lines = code.lines().map(|s| s.to_string()).collect();
        let (instructions, functions) = Parser::parse(code)?;

        self.instructions.clear();
        for (i, instr) in instructions.iter().enumerate() {
            self.instructions.push((i + 1, instr.clone()));
        }

        self.labels.clear();
        for (i, (_, instr)) in self.instructions.iter().enumerate() {
            if let Instruction::Label { id } = instr {
                self.labels.insert(*id, i);
            }
        }

        self.functions.clear();
        for func in functions {
            self.functions.insert(func.id, func);
        }

        self.ip = 0;
        self.state = DebugState::Paused;
        self.global_vars.clear();
        self.call_stack.clear();
        self.current_frame = StackFrame {
            func_id: -1, line: 0, locals: HashMap::new(), args: Vec::new(),
        };
        self.output.clear();
        Ok(())
    }

    pub fn set_breakpoint(&mut self, line: usize) { self.breakpoints.insert(line); }
    pub fn remove_breakpoint(&mut self, line: usize) { self.breakpoints.remove(&line); }
    pub fn clear_breakpoints(&mut self) { self.breakpoints.clear(); }
    pub fn breakpoints(&self) -> &HashSet<usize> { &self.breakpoints }
    pub fn state(&self) -> DebugState { self.state }
    pub fn current_line(&self) -> usize { self.current_line }
    pub fn source_at(&self, line: usize) -> Option<&str> {
        self.source_lines.get(line.saturating_sub(1)).map(|s| s.as_str())
    }

    fn resolve(&self, val: &str) -> Value {
        match Lexer::parse_value(val) {
            ParsedValue::Variable(var) => {
                let prefix = var.chars().next().unwrap();
                let idx: i64 = var[1..].parse().unwrap_or(0);
                match prefix {
                    'v' => self.current_frame.locals.get(&idx).cloned().unwrap_or_default(),
                    'g' => self.global_vars.get(&idx).cloned().unwrap_or_default(),
                    'a' => self.current_frame.args.get(idx as usize).cloned().unwrap_or_default(),
                    _ => Value::default(),
                }
            }
            ParsedValue::Integer(n) => Value::Integer(n),
            ParsedValue::Float(f) => Value::Float(f),
            ParsedValue::String(s) => Value::String(s),
        }
    }

    fn assign(&mut self, var: &str, value: Value) {
        let prefix = var.chars().next().unwrap_or('v');
        let idx: i64 = var[1..].parse().unwrap_or(0);
        match prefix {
            'v' => { self.current_frame.locals.insert(idx, value); }
            'g' => { self.global_vars.insert(idx, value); }
            _ => {}
        }
    }

    fn run_instruction(&mut self, instr: &Instruction) -> Result<Option<i64>, String> {
        match instr {
            Instruction::Empty | Instruction::Comment | Instruction::FuncDef { .. } | Instruction::FuncEnd | Instruction::Import { .. } => {
                // Import is handled during loading, no-op during execution
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
                let val = if self.resolve(a).is_truthy() { Value::Integer(0) } else { Value::Integer(1) };
                self.assign(result, val);
            }
            Instruction::And { result, a, b } => {
                let val = if self.resolve(a).is_truthy() && self.resolve(b).is_truthy() { Value::Integer(1) } else { Value::Integer(0) };
                self.assign(result, val);
            }
            Instruction::Or { result, a, b } => {
                let val = if self.resolve(a).is_truthy() || self.resolve(b).is_truthy() { Value::Integer(1) } else { Value::Integer(0) };
                self.assign(result, val);
            }
            Instruction::CondJump { cond, label } => {
                if self.resolve(cond).is_truthy() { return Ok(Some(*label)); }
            }
            Instruction::Jump { label } => { return Ok(Some(*label)); }
            Instruction::Label { .. } => {}
            Instruction::Call { result, func_id, args } => {
                let resolved_args: Vec<Value> = args.iter().map(|a| self.resolve(a)).collect();
                let old_frame = std::mem::replace(&mut self.current_frame, StackFrame {
                    func_id: *func_id, line: 0, locals: HashMap::new(),
                    args: resolved_args,
                });
                self.call_stack.push(old_frame);
                let func = self.functions.get(func_id).cloned()
                    .ok_or_else(|| format!("Undefined function: {}", func_id))?;
                let mut func_labels: HashMap<i64, usize> = HashMap::new();
                for (i, instr) in func.body.iter().enumerate() {
                    if let Instruction::Label { id } = instr { func_labels.insert(*id, i); }
                }
                let mut fi = 0;
                let mut return_val = Value::Integer(0);
                while fi < func.body.len() {
                    let jump = self.run_instruction(&func.body[fi])?;
                    if let Instruction::Return { value } = &func.body[fi] {
                        return_val = self.resolve(value);
                        break;
                    }
                    if let Some(label) = jump {
                        if let Some(&pos) = func_labels.get(&label) { fi = pos; } else { fi += 1; }
                    } else { fi += 1; }
                }
                self.current_frame = self.call_stack.pop().unwrap();
                self.assign(result, return_val);
            }
            Instruction::Return { .. } => {}
            Instruction::ArrayCreate { var, size } => {
                let size = self.resolve(size).to_int() as usize;
                self.assign(var, Value::Array(vec![Value::Integer(0); size]));
            }
            Instruction::ArrayRead { result, arr, idx } => {
                let array = self.resolve(arr);
                let index = self.resolve(idx).to_int();
                let val = match array {
                    Value::Array(ref a) if index >= 0 && (index as usize) < a.len() => a[index as usize].clone(),
                    _ => Value::Integer(0),
                };
                self.assign(result, val);
            }
            Instruction::ArrayWrite { arr, idx, value } => {
                let index = self.resolve(idx).to_int();
                let val = self.resolve(value);
                let prefix = arr.chars().next().unwrap_or('v');
                let var_idx: i64 = arr[1..].parse().unwrap_or(0);
                let array = match prefix {
                    'v' => self.current_frame.locals.get_mut(&var_idx),
                    'g' => self.global_vars.get_mut(&var_idx),
                    _ => None,
                };
                if let Some(Value::Array(ref mut a)) = array {
                    if index >= 0 && (index as usize) < a.len() { a[index as usize] = val; }
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
                io::stdout().flush().ok();
                let stdin = io::stdin();
                let line = stdin.lock().lines().next().unwrap_or(Ok(String::new())).unwrap_or_default();
                let val = if let Ok(n) = line.trim().parse::<i64>() { Value::Integer(n) }
                else if let Ok(f) = line.trim().parse::<f64>() { Value::Float(f) }
                else { Value::String(line.trim().to_string()) };
                self.assign(var, val);
            }
            Instruction::RustFFI { result, func, args } => {
                let func_name = self.resolve(func).to_string();
                let resolved_args: Vec<Value> = args.iter().map(|a| self.resolve(a)).collect();
                let val = self.call_builtin(&func_name, &resolved_args);
                self.assign(result, val);
            }
        }
        Ok(None)
    }

    fn call_builtin(&self, func: &str, args: &[Value]) -> Value {
        let func_name = func.rsplit('.').next().unwrap_or(func);
        match func_name {
            "sqrt" => Value::Float(args.first().map(|v| v.to_float()).unwrap_or(0.0).sqrt()),
            "abs" => {
                let x = args.first().map(|v| v.to_float()).unwrap_or(0.0);
                if x.fract() == 0.0 { Value::Integer(x.abs() as i64) } else { Value::Float(x.abs()) }
            }
            "len" => match args.first() {
                Some(Value::String(s)) => Value::Integer(s.len() as i64),
                Some(Value::Array(a)) => Value::Integer(a.len() as i64),
                _ => Value::Integer(0),
            },
            _ => Value::Integer(0),
        }
    }

    pub fn step(&mut self) -> DebugEvent {
        if self.ip >= self.instructions.len() {
            self.state = DebugState::Finished;
            return DebugEvent::Finished;
        }
        let (line, instr) = self.instructions[self.ip].clone();
        self.current_line = line;
        self.current_frame.line = line;
        match self.run_instruction(&instr) {
            Ok(jump) => {
                if let Some(label) = jump {
                    if let Some(&pos) = self.labels.get(&label) { self.ip = pos; } else { self.ip += 1; }
                } else { self.ip += 1; }
                if self.ip >= self.instructions.len() {
                    self.state = DebugState::Finished;
                    DebugEvent::Finished
                } else {
                    self.state = DebugState::Paused;
                    DebugEvent::Step
                }
            }
            Err(e) => { self.state = DebugState::Finished; DebugEvent::Error(e) }
        }
    }

    pub fn resume(&mut self) -> DebugEvent {
        self.state = DebugState::Running;
        loop {
            if self.ip >= self.instructions.len() {
                self.state = DebugState::Finished;
                return DebugEvent::Finished;
            }
            let (line, instr) = self.instructions[self.ip].clone();
            self.current_line = line;
            self.current_frame.line = line;
            if self.breakpoints.contains(&line) && self.state == DebugState::Running {
                self.state = DebugState::Paused;
                return DebugEvent::Breakpoint(line);
            }
            match self.run_instruction(&instr) {
                Ok(jump) => {
                    if let Some(label) = jump {
                        if let Some(&pos) = self.labels.get(&label) { self.ip = pos; } else { self.ip += 1; }
                    } else { self.ip += 1; }
                }
                Err(e) => { self.state = DebugState::Finished; return DebugEvent::Error(e); }
            }
            if self.ip < self.instructions.len() {
                let next_line = self.instructions[self.ip].0;
                if self.breakpoints.contains(&next_line) {
                    self.current_line = next_line;
                    self.state = DebugState::Paused;
                    return DebugEvent::Breakpoint(next_line);
                }
            }
        }
    }

    pub fn locals(&self) -> &HashMap<i64, Value> { &self.current_frame.locals }
    pub fn globals(&self) -> &HashMap<i64, Value> { &self.global_vars }
    pub fn args(&self) -> &[Value] { &self.current_frame.args }
    pub fn call_stack(&self) -> &[StackFrame] { &self.call_stack }
    pub fn output(&self) -> &[String] { &self.output }
    pub fn inspect(&self, expr: &str) -> Option<Value> { Some(self.resolve(expr)) }

    pub fn run_interactive(&mut self) {
        println!("Sui Debugger - Type 'help' for commands\n");
        if let Some(src) = self.source_at(1) { println!("=> 1: {}", src); }
        let stdin = io::stdin();
        loop {
            print!("(sui-dbg) ");
            io::stdout().flush().ok();
            let mut input = String::new();
            if stdin.lock().read_line(&mut input).is_err() { break; }
            let cmd: Vec<&str> = input.trim().split_whitespace().collect();
            if cmd.is_empty() { continue; }
            match cmd[0] {
                "help" | "h" => {
                    println!("Commands:");
                    println!("  step, s        - Run one instruction");
                    println!("  continue, c    - Continue until breakpoint");
                    println!("  break N, b N   - Set breakpoint at line N");
                    println!("  delete N, d N  - Remove breakpoint at line N");
                    println!("  list, l        - Show source around current line");
                    println!("  locals         - Show local variables");
                    println!("  globals        - Show global variables");
                    println!("  print E, p E   - Inspect expression E");
                    println!("  backtrace, bt  - Show call stack");
                    println!("  quit, q        - Exit debugger");
                }
                "step" | "s" => {
                    let event = self.step();
                    self.print_event(&event);
                    if self.state == DebugState::Finished { println!("Program finished."); break; }
                }
                "continue" | "c" => {
                    let event = self.resume();
                    self.print_event(&event);
                    if self.state == DebugState::Finished { println!("Program finished."); break; }
                }
                "break" | "b" => {
                    if let Some(line_str) = cmd.get(1) {
                        if let Ok(line) = line_str.parse::<usize>() {
                            self.set_breakpoint(line);
                            println!("Breakpoint set at line {}", line);
                        }
                    } else { println!("Breakpoints: {:?}", self.breakpoints); }
                }
                "delete" | "d" => {
                    if let Some(line_str) = cmd.get(1) {
                        if let Ok(line) = line_str.parse::<usize>() {
                            self.remove_breakpoint(line);
                            println!("Breakpoint removed at line {}", line);
                        }
                    }
                }
                "list" | "l" => {
                    let start = self.current_line.saturating_sub(3);
                    let end = (self.current_line + 4).min(self.source_lines.len());
                    for i in start..end {
                        let marker = if i + 1 == self.current_line { "=>" } else { "  " };
                        let bp = if self.breakpoints.contains(&(i + 1)) { "*" } else { " " };
                        if let Some(src) = self.source_at(i + 1) { println!("{}{} {:3}: {}", marker, bp, i + 1, src); }
                    }
                }
                "locals" => {
                    println!("Local variables:");
                    let mut vars: Vec<_> = self.current_frame.locals.iter().collect();
                    vars.sort_by_key(|(k, _)| *k);
                    for (idx, val) in vars { println!("  v{} = {}", idx, val); }
                }
                "globals" => {
                    println!("Global variables:");
                    let mut vars: Vec<_> = self.global_vars.iter().collect();
                    vars.sort_by_key(|(k, _)| *k);
                    for (idx, val) in vars { println!("  g{} = {}", idx, val); }
                }
                "print" | "p" => {
                    if let Some(expr) = cmd.get(1) {
                        if let Some(val) = self.inspect(expr) { println!("{} = {}", expr, val); }
                    }
                }
                "backtrace" | "bt" => {
                    println!("Call stack:");
                    for (i, frame) in self.call_stack.iter().rev().enumerate() {
                        let name = if frame.func_id < 0 { "main".to_string() } else { format!("func_{}", frame.func_id) };
                        println!("  #{} {} at line {}", i, name, frame.line);
                    }
                    let name = if self.current_frame.func_id < 0 { "main".to_string() } else { format!("func_{}", self.current_frame.func_id) };
                    println!("  #0 {} at line {} (current)", name, self.current_line);
                }
                "quit" | "q" => { println!("Exiting debugger."); break; }
                _ => { println!("Unknown command: {}. Type 'help' for commands.", cmd[0]); }
            }
        }
    }

    fn print_event(&self, event: &DebugEvent) {
        match event {
            DebugEvent::Breakpoint(line) => {
                println!("Breakpoint at line {}", line);
                if let Some(src) = self.source_at(*line) { println!("=> {}: {}", line, src); }
            }
            DebugEvent::Step => {
                if let Some(src) = self.source_at(self.current_line) { println!("=> {}: {}", self.current_line, src); }
            }
            DebugEvent::Finished => { println!("Done."); }
            DebugEvent::Error(e) => { println!("Error: {}", e); }
        }
    }
}

impl Default for Debugger { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debugger_step() {
        let mut dbg = Debugger::new();
        dbg.load("= v0 10\n+ v1 v0 5\n. v1").unwrap();
        let event = dbg.step();
        assert!(matches!(event, DebugEvent::Step));
        assert_eq!(dbg.current_line(), 1);
        let event = dbg.step();
        assert!(matches!(event, DebugEvent::Step));
        let event = dbg.step();
        assert!(matches!(event, DebugEvent::Finished));
    }

    #[test]
    fn test_debugger_breakpoint() {
        let mut dbg = Debugger::new();
        dbg.load("= v0 10\n+ v1 v0 5\n. v1").unwrap();
        dbg.set_breakpoint(2);
        let event = dbg.resume();
        assert!(matches!(event, DebugEvent::Breakpoint(2)));
    }

    #[test]
    fn test_debugger_locals() {
        let mut dbg = Debugger::new();
        dbg.load("= v0 42\n= v1 100").unwrap();
        dbg.step();
        assert_eq!(dbg.locals().get(&0), Some(&Value::Integer(42)));
        dbg.step();
        assert_eq!(dbg.locals().get(&1), Some(&Value::Integer(100)));
    }
}
