//! Python to Sui transpiler
//!
//! Converts a subset of Python to Sui code.

use super::TranspileError;
use regex::Regex;
use std::collections::HashMap;

/// Python to Sui transpiler
pub struct Py2Sui {
    output: Vec<String>,
    var_counter: usize,
    label_counter: i64,
    func_counter: i64,
    var_map: HashMap<String, String>,
    func_map: HashMap<String, i64>,
    is_global: bool,
    func_args: Vec<String>,
    indent_stack: Vec<IndentContext>,
}

#[derive(Debug, Clone)]
enum IndentContext {
    If { end_label: i64 },
    IfElse { else_label: i64, end_label: i64 },
    While { start_label: i64, end_label: i64 },
    For { start_label: i64, end_label: i64, loop_var: String },
    Function,
    Else { end_label: i64 },
}

impl Default for Py2Sui {
    fn default() -> Self {
        Self::new()
    }
}

impl Py2Sui {
    /// Create a new transpiler
    pub fn new() -> Self {
        Self {
            output: Vec::new(),
            var_counter: 0,
            label_counter: 0,
            func_counter: 0,
            var_map: HashMap::new(),
            func_map: HashMap::new(),
            is_global: true,
            func_args: Vec::new(),
            indent_stack: Vec::new(),
        }
    }

    /// Emit a line of Sui code
    fn emit(&mut self, line: &str) {
        self.output.push(line.to_string());
    }

    /// Create a new temporary variable
    fn new_var(&mut self) -> String {
        let var = format!("v{}", self.var_counter);
        self.var_counter += 1;
        var
    }

    /// Create a new label
    fn new_label(&mut self) -> i64 {
        let label = self.label_counter;
        self.label_counter += 1;
        label
    }

    /// Get or create a variable for a Python name
    fn get_var(&mut self, name: &str) -> String {
        // Check if it's a function argument
        if let Some(idx) = self.func_args.iter().position(|a| a == name) {
            return format!("a{}", idx);
        }

        // Check existing mapping
        if let Some(var) = self.var_map.get(name) {
            return var.clone();
        }

        // Create new variable
        let var = if self.is_global {
            let count = self.var_map.values().filter(|v| v.starts_with('g')).count();
            format!("g{}", count)
        } else {
            self.new_var()
        };

        self.var_map.insert(name.to_string(), var.clone());
        var
    }

    /// Parse an expression and return the result variable
    fn parse_expr(&mut self, expr: &str) -> String {
        let expr = expr.trim();

        // Integer literal
        if let Ok(n) = expr.parse::<i64>() {
            let var = self.new_var();
            self.emit(&format!("= {} {}", var, n));
            return var;
        }

        // Float literal
        if let Ok(f) = expr.parse::<f64>() {
            let var = self.new_var();
            self.emit(&format!("= {} {}", var, f));
            return var;
        }

        // String literal
        if (expr.starts_with('"') && expr.ends_with('"'))
            || (expr.starts_with('\'') && expr.ends_with('\''))
        {
            let var = self.new_var();
            let content = &expr[1..expr.len() - 1];
            self.emit(&format!("= {} \"{}\"", var, content));
            return var;
        }

        // Boolean and special values
        if expr == "True" {
            let var = self.new_var();
            self.emit(&format!("= {} 1", var));
            return var;
        }
        if expr == "False" || expr == "None" {
            let var = self.new_var();
            self.emit(&format!("= {} 0", var));
            return var;
        }

        // Comparison operators (handle before arithmetic to get precedence right)
        for (op_str, sui_op) in [
            ("==", "~"),
            ("!=", "!~"),
            ("<=", "<="),
            (">=", ">="),
            ("<", "<"),
            (">", ">"),
        ] {
            if let Some(idx) = self.find_operator(expr, op_str) {
                let left = self.parse_expr(&expr[..idx]);
                let right = self.parse_expr(&expr[idx + op_str.len()..]);
                let result = self.new_var();

                match sui_op {
                    "~" => self.emit(&format!("~ {} {} {}", result, left, right)),
                    "!~" => {
                        let tmp = self.new_var();
                        self.emit(&format!("~ {} {} {}", tmp, left, right));
                        self.emit(&format!("! {} {}", result, tmp));
                    }
                    "<=" => {
                        let tmp1 = self.new_var();
                        let tmp2 = self.new_var();
                        self.emit(&format!("< {} {} {}", tmp1, left, right));
                        self.emit(&format!("~ {} {} {}", tmp2, left, right));
                        self.emit(&format!("| {} {} {}", result, tmp1, tmp2));
                    }
                    ">=" => {
                        let tmp1 = self.new_var();
                        let tmp2 = self.new_var();
                        self.emit(&format!("> {} {} {}", tmp1, left, right));
                        self.emit(&format!("~ {} {} {}", tmp2, left, right));
                        self.emit(&format!("| {} {} {}", result, tmp1, tmp2));
                    }
                    "<" => self.emit(&format!("< {} {} {}", result, left, right)),
                    ">" => self.emit(&format!("> {} {} {}", result, left, right)),
                    _ => {}
                }
                return result;
            }
        }

        // Logical operators
        if let Some(idx) = self.find_keyword(expr, " and ") {
            let left = self.parse_expr(&expr[..idx]);
            let right = self.parse_expr(&expr[idx + 5..]);
            let result = self.new_var();
            self.emit(&format!("& {} {} {}", result, left, right));
            return result;
        }

        if let Some(idx) = self.find_keyword(expr, " or ") {
            let left = self.parse_expr(&expr[..idx]);
            let right = self.parse_expr(&expr[idx + 4..]);
            let result = self.new_var();
            self.emit(&format!("| {} {} {}", result, left, right));
            return result;
        }

        if expr.starts_with("not ") {
            let operand = self.parse_expr(&expr[4..]);
            let result = self.new_var();
            self.emit(&format!("! {} {}", result, operand));
            return result;
        }

        // Arithmetic operators (lowest precedence first for correct parsing)
        for (op_str, sui_op) in [("+", "+"), ("-", "-")] {
            if let Some(idx) = self.find_operator_rtl(expr, op_str) {
                if idx > 0 {
                    let left = self.parse_expr(&expr[..idx]);
                    let right = self.parse_expr(&expr[idx + 1..]);
                    let result = self.new_var();
                    self.emit(&format!("{} {} {} {}", sui_op, result, left, right));
                    return result;
                }
            }
        }

        for (op_str, sui_op) in [("*", "*"), ("/", "/"), ("%", "%")] {
            if let Some(idx) = self.find_operator_rtl(expr, op_str) {
                let left = self.parse_expr(&expr[..idx]);
                let right = self.parse_expr(&expr[idx + 1..]);
                let result = self.new_var();
                self.emit(&format!("{} {} {} {}", sui_op, result, left, right));
                return result;
            }
        }

        // Unary minus
        if expr.starts_with('-') && expr.len() > 1 {
            let operand = self.parse_expr(&expr[1..]);
            let result = self.new_var();
            self.emit(&format!("- {} 0 {}", result, operand));
            return result;
        }

        // Parenthesized expression
        if expr.starts_with('(') && expr.ends_with(')') {
            return self.parse_expr(&expr[1..expr.len() - 1]);
        }

        // Function call
        if let Some(paren_idx) = expr.find('(') {
            if expr.ends_with(')') {
                let func_name = &expr[..paren_idx];
                let args_str = &expr[paren_idx + 1..expr.len() - 1];

                // Built-in functions
                match func_name {
                    "print" => {
                        let args = self.split_args(args_str);
                        for arg in args {
                            let arg_var = self.parse_expr(&arg);
                            self.emit(&format!(". {}", arg_var));
                        }
                        return self.new_var();
                    }
                    "input" => {
                        let result = self.new_var();
                        self.emit(&format!(", {}", result));
                        return result;
                    }
                    "len" => {
                        let result = self.new_var();
                        let args = self.split_args(args_str);
                        if !args.is_empty() {
                            let arg_var = self.parse_expr(&args[0]);
                            self.emit(&format!("R {} \"len\" {}", result, arg_var));
                        } else {
                            self.emit(&format!("= {} 0", result));
                        }
                        return result;
                    }
                    "int" | "float" | "str" | "abs" | "round" | "max" | "min" => {
                        let result = self.new_var();
                        let args = self.split_args(args_str);
                        let arg_vars: Vec<String> = args.iter().map(|a| self.parse_expr(a)).collect();
                        self.emit(&format!("R {} \"{}\" {}", result, func_name, arg_vars.join(" ")));
                        return result;
                    }
                    "range" => {
                        // range() returns a placeholder - handled specially in for loops
                        let result = self.new_var();
                        self.emit(&format!("= {} 0", result));
                        return result;
                    }
                    _ => {
                        // User-defined function
                        if let Some(&func_id) = self.func_map.get(func_name) {
                            let args = self.split_args(args_str);
                            let arg_vars: Vec<String> =
                                args.iter().map(|a| self.parse_expr(a)).collect();
                            let result = self.new_var();
                            self.emit(&format!("$ {} {} {}", result, func_id, arg_vars.join(" ")));
                            return result;
                        }
                    }
                }
            }
        }

        // Array subscript
        if let Some(bracket_idx) = expr.find('[') {
            if expr.ends_with(']') {
                let arr_name = &expr[..bracket_idx];
                let idx_str = &expr[bracket_idx + 1..expr.len() - 1];
                let arr_var = self.get_var(arr_name);
                let idx_var = self.parse_expr(idx_str);
                let result = self.new_var();
                self.emit(&format!("] {} {} {}", result, arr_var, idx_var));
                return result;
            }
        }

        // List literal
        if expr.starts_with('[') && expr.ends_with(']') {
            let content = &expr[1..expr.len() - 1];
            let elements = self.split_args(content);
            let result = self.new_var();
            self.emit(&format!("[ {} {}", result, elements.len()));
            for (i, elem) in elements.iter().enumerate() {
                let val = self.parse_expr(elem);
                self.emit(&format!("{{ {} {} {}", result, i, val));
            }
            return result;
        }

        // Simple variable name
        self.get_var(expr)
    }

    /// Find operator position, skipping parentheses
    fn find_operator(&self, expr: &str, op: &str) -> Option<usize> {
        let mut depth = 0;
        let chars: Vec<char> = expr.chars().collect();
        let op_chars: Vec<char> = op.chars().collect();

        for i in 0..chars.len() {
            match chars[i] {
                '(' | '[' => depth += 1,
                ')' | ']' => depth -= 1,
                '"' | '\'' => {
                    // Skip strings
                    let quote = chars[i];
                    let mut j = i + 1;
                    while j < chars.len() && chars[j] != quote {
                        j += 1;
                    }
                }
                _ => {}
            }

            if depth == 0 && i + op_chars.len() <= chars.len() {
                let slice: String = chars[i..i + op_chars.len()].iter().collect();
                if slice == op {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Find operator from right to left (for left-associative operators)
    fn find_operator_rtl(&self, expr: &str, op: &str) -> Option<usize> {
        let mut depth = 0;
        let chars: Vec<char> = expr.chars().collect();

        for i in (0..chars.len()).rev() {
            match chars[i] {
                ')' | ']' => depth += 1,
                '(' | '[' => depth -= 1,
                _ => {}
            }

            if depth == 0 && chars[i].to_string() == op {
                // Make sure this isn't part of a two-char operator
                if i > 0 && (chars[i - 1] == '=' || chars[i - 1] == '<' || chars[i - 1] == '>' || chars[i - 1] == '!') {
                    continue;
                }
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    continue;
                }
                return Some(i);
            }
        }
        None
    }

    /// Find a keyword in expression
    fn find_keyword(&self, expr: &str, keyword: &str) -> Option<usize> {
        let mut depth = 0;
        let chars: Vec<char> = expr.chars().collect();
        let kw_chars: Vec<char> = keyword.chars().collect();

        for i in 0..chars.len() {
            match chars[i] {
                '(' | '[' => depth += 1,
                ')' | ']' => depth -= 1,
                _ => {}
            }

            if depth == 0 && i + kw_chars.len() <= chars.len() {
                let slice: String = chars[i..i + kw_chars.len()].iter().collect();
                if slice == keyword {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Split function arguments
    fn split_args(&self, args_str: &str) -> Vec<String> {
        if args_str.trim().is_empty() {
            return Vec::new();
        }

        let mut result = Vec::new();
        let mut current = String::new();
        let mut depth = 0;
        let mut in_string = false;
        let mut string_char = '"';

        for c in args_str.chars() {
            if !in_string && (c == '"' || c == '\'') {
                in_string = true;
                string_char = c;
                current.push(c);
            } else if in_string && c == string_char {
                in_string = false;
                current.push(c);
            } else if in_string {
                current.push(c);
            } else if c == '(' || c == '[' {
                depth += 1;
                current.push(c);
            } else if c == ')' || c == ']' {
                depth -= 1;
                current.push(c);
            } else if c == ',' && depth == 0 {
                result.push(current.trim().to_string());
                current = String::new();
            } else {
                current.push(c);
            }
        }

        if !current.trim().is_empty() {
            result.push(current.trim().to_string());
        }

        result
    }

    /// Get indentation level
    fn get_indent(&self, line: &str) -> usize {
        line.chars().take_while(|&c| c == ' ' || c == '\t').count()
    }

    /// Parse a line of Python code
    fn parse_line(&mut self, line: &str, _current_indent: usize) {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            return;
        }

        // Assignment with augmented operators
        let aug_ops = [("+=", "+"), ("-=", "-"), ("*=", "*"), ("/=", "/"), ("%=", "%")];
        for (py_op, sui_op) in aug_ops {
            if let Some(idx) = trimmed.find(py_op) {
                let target = trimmed[..idx].trim();
                let value = trimmed[idx + 2..].trim();
                let target_var = self.get_var(target);
                let value_var = self.parse_expr(value);
                self.emit(&format!("{} {} {} {}", sui_op, target_var, target_var, value_var));
                return;
            }
        }

        // Simple assignment
        if let Some(idx) = self.find_assignment(trimmed) {
            let target = trimmed[..idx].trim();
            let value = trimmed[idx + 1..].trim();

            // Array subscript assignment
            if let Some(bracket_idx) = target.find('[') {
                if target.ends_with(']') {
                    let arr_name = &target[..bracket_idx];
                    let idx_str = &target[bracket_idx + 1..target.len() - 1];
                    let arr_var = self.get_var(arr_name);
                    let idx_var = self.parse_expr(idx_str);
                    let value_var = self.parse_expr(value);
                    self.emit(&format!("{{ {} {} {}", arr_var, idx_var, value_var));
                    return;
                }
            }

            let value_var = self.parse_expr(value);
            let target_var = self.get_var(target);
            self.emit(&format!("= {} {}", target_var, value_var));
            return;
        }

        // If statement
        if trimmed.starts_with("if ") && trimmed.ends_with(':') {
            let cond_str = &trimmed[3..trimmed.len() - 1];
            let cond = self.parse_expr(cond_str);
            let not_cond = self.new_var();
            self.emit(&format!("! {} {}", not_cond, cond));

            let end_label = self.new_label();
            self.emit(&format!("? {} {}", not_cond, end_label));

            self.indent_stack.push(IndentContext::If { end_label });
            return;
        }

        // Elif statement
        if trimmed.starts_with("elif ") && trimmed.ends_with(':') {
            // Handle like else + if
            if let Some(IndentContext::If { end_label }) = self.indent_stack.pop() {
                let new_end = self.new_label();
                self.emit(&format!("@ {}", new_end));
                self.emit(&format!(": {}", end_label));

                let cond_str = &trimmed[5..trimmed.len() - 1];
                let cond = self.parse_expr(cond_str);
                let not_cond = self.new_var();
                self.emit(&format!("! {} {}", not_cond, cond));

                let elif_end = self.new_label();
                self.emit(&format!("? {} {}", not_cond, elif_end));

                self.indent_stack.push(IndentContext::IfElse {
                    else_label: elif_end,
                    end_label: new_end,
                });
            }
            return;
        }

        // Else statement
        if trimmed == "else:" {
            match self.indent_stack.pop() {
                Some(IndentContext::If { end_label }) => {
                    let new_end = self.new_label();
                    self.emit(&format!("@ {}", new_end));
                    self.emit(&format!(": {}", end_label));
                    self.indent_stack.push(IndentContext::Else { end_label: new_end });
                }
                Some(IndentContext::IfElse { else_label, end_label }) => {
                    self.emit(&format!("@ {}", end_label));
                    self.emit(&format!(": {}", else_label));
                    self.indent_stack.push(IndentContext::Else { end_label });
                }
                other => {
                    if let Some(ctx) = other {
                        self.indent_stack.push(ctx);
                    }
                }
            }
            return;
        }

        // While statement
        if trimmed.starts_with("while ") && trimmed.ends_with(':') {
            let cond_str = &trimmed[6..trimmed.len() - 1];

            let start_label = self.new_label();
            let end_label = self.new_label();

            self.emit(&format!(": {}", start_label));

            let cond = self.parse_expr(cond_str);
            let not_cond = self.new_var();
            self.emit(&format!("! {} {}", not_cond, cond));
            self.emit(&format!("? {} {}", not_cond, end_label));

            self.indent_stack
                .push(IndentContext::While { start_label, end_label });
            return;
        }

        // For statement (only range supported)
        if trimmed.starts_with("for ") && trimmed.contains(" in ") && trimmed.ends_with(':') {
            let re = Regex::new(r"for\s+(\w+)\s+in\s+range\s*\((.+)\)\s*:").unwrap();
            if let Some(caps) = re.captures(trimmed) {
                let loop_var_name = caps.get(1).unwrap().as_str();
                let range_args = caps.get(2).unwrap().as_str();
                let args = self.split_args(range_args);

                let (start_val, end_expr) = if args.len() == 1 {
                    ("0".to_string(), args[0].clone())
                } else {
                    (args[0].clone(), args[1].clone())
                };

                let loop_var = self.get_var(loop_var_name);
                let start_var = self.parse_expr(&start_val);
                self.emit(&format!("= {} {}", loop_var, start_var));

                let end_var = self.parse_expr(&end_expr);

                let start_label = self.new_label();
                let end_label = self.new_label();

                self.emit(&format!(": {}", start_label));

                let cond = self.new_var();
                self.emit(&format!("< {} {} {}", cond, loop_var, end_var));
                let not_cond = self.new_var();
                self.emit(&format!("! {} {}", not_cond, cond));
                self.emit(&format!("? {} {}", not_cond, end_label));

                self.indent_stack.push(IndentContext::For {
                    start_label,
                    end_label,
                    loop_var: loop_var.clone(),
                });
                return;
            }
        }

        // Function definition
        if trimmed.starts_with("def ") && trimmed.ends_with(':') {
            let re = Regex::new(r"def\s+(\w+)\s*\(([^)]*)\)\s*:").unwrap();
            if let Some(caps) = re.captures(trimmed) {
                let func_name = caps.get(1).unwrap().as_str();
                let params_str = caps.get(2).unwrap().as_str();

                let func_id = self.func_counter;
                self.func_counter += 1;
                self.func_map.insert(func_name.to_string(), func_id);

                let params: Vec<String> = if params_str.trim().is_empty() {
                    Vec::new()
                } else {
                    params_str.split(',').map(|s| s.trim().to_string()).collect()
                };

                self.emit(&format!("# {} {} {{", func_id, params.len()));

                // Update context for function body
                self.is_global = false;
                self.var_counter = 0;
                self.func_args = params;

                self.indent_stack.push(IndentContext::Function);
                return;
            }
        }

        // Return statement
        if trimmed.starts_with("return") {
            let value_str = trimmed[6..].trim();
            if value_str.is_empty() {
                self.emit("^ 0");
            } else {
                let value = self.parse_expr(value_str);
                self.emit(&format!("^ {}", value));
            }
            return;
        }

        // Print statement (Python 2 style, also catches function call)
        if trimmed.starts_with("print(") && trimmed.ends_with(')') {
            let args_str = &trimmed[6..trimmed.len() - 1];
            let args = self.split_args(args_str);
            for arg in args {
                let arg_var = self.parse_expr(&arg);
                self.emit(&format!(". {}", arg_var));
            }
            return;
        }

        // Pass statement
        if trimmed == "pass" {
            return;
        }

        // Expression statement (function call, etc.)
        if trimmed.contains('(') {
            self.parse_expr(trimmed);
        }
    }

    /// Find assignment operator (not comparison ==)
    fn find_assignment(&self, s: &str) -> Option<usize> {
        let chars: Vec<char> = s.chars().collect();
        let mut depth = 0;
        let mut in_string = false;
        let mut string_char = '"';

        for i in 0..chars.len() {
            let c = chars[i];

            if !in_string && (c == '"' || c == '\'') {
                in_string = true;
                string_char = c;
            } else if in_string && c == string_char {
                in_string = false;
            } else if !in_string {
                if c == '(' || c == '[' {
                    depth += 1;
                } else if c == ')' || c == ']' {
                    depth -= 1;
                } else if c == '=' && depth == 0 {
                    // Make sure it's not ==, !=, <=, >=
                    let prev = if i > 0 { chars[i - 1] } else { ' ' };
                    let next = if i + 1 < chars.len() { chars[i + 1] } else { ' ' };

                    if prev != '=' && prev != '!' && prev != '<' && prev != '>' && next != '=' {
                        return Some(i);
                    }
                }
            }
        }
        None
    }

    /// Close a block based on indentation
    fn close_blocks(&mut self, new_indent: usize, prev_indent: usize) {
        // Close blocks when dedenting
        while !self.indent_stack.is_empty() && new_indent < prev_indent {
            if let Some(ctx) = self.indent_stack.pop() {
                match ctx {
                    IndentContext::If { end_label } => {
                        self.emit(&format!(": {}", end_label));
                    }
                    IndentContext::IfElse { else_label, end_label } => {
                        self.emit(&format!(": {}", else_label));
                        self.emit(&format!(": {}", end_label));
                    }
                    IndentContext::Else { end_label } => {
                        self.emit(&format!(": {}", end_label));
                    }
                    IndentContext::While { start_label, end_label } => {
                        self.emit(&format!("@ {}", start_label));
                        self.emit(&format!(": {}", end_label));
                    }
                    IndentContext::For { start_label, end_label, loop_var } => {
                        self.emit(&format!("+ {} {} 1", loop_var, loop_var));
                        self.emit(&format!("@ {}", start_label));
                        self.emit(&format!(": {}", end_label));
                    }
                    IndentContext::Function => {
                        self.emit("}");
                        self.is_global = true;
                        self.func_args.clear();
                    }
                }
            }
            break;
        }
    }

    /// Transpile Python code to Sui
    pub fn transpile_to_sui(&mut self, code: &str) -> Result<String, TranspileError> {
        self.output.clear();
        self.var_counter = 0;
        self.label_counter = 0;
        self.var_map.clear();
        self.indent_stack.clear();
        self.is_global = true;
        self.func_args.clear();

        let lines: Vec<&str> = code.lines().collect();
        let mut prev_indent = 0;

        // First pass: collect function names
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with("def ") && trimmed.ends_with(':') {
                let re = Regex::new(r"def\s+(\w+)\s*\(").unwrap();
                if let Some(caps) = re.captures(trimmed) {
                    let func_name = caps.get(1).unwrap().as_str();
                    self.func_map.insert(func_name.to_string(), self.func_counter);
                    self.func_counter += 1;
                }
            }
        }
        self.func_counter = 0;

        // Second pass: transpile
        for line in lines {
            let current_indent = self.get_indent(line);
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Handle dedent
            if current_indent < prev_indent {
                self.close_blocks(current_indent, prev_indent);
            }

            self.parse_line(line, current_indent);
            prev_indent = current_indent;
        }

        // Close any remaining blocks
        self.close_blocks(0, prev_indent);

        Ok(self.output.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_assignment() {
        let mut t = Py2Sui::new();
        let result = t.transpile_to_sui("x = 10").unwrap();
        assert!(result.contains("= g0 10") || result.contains("= v0 10"));
    }

    #[test]
    fn test_arithmetic() {
        let mut t = Py2Sui::new();
        let result = t.transpile_to_sui("x = 5 + 3").unwrap();
        assert!(result.contains("+"));
    }

    #[test]
    fn test_while_loop() {
        let mut t = Py2Sui::new();
        let code = r#"
x = 0
while x < 10:
    print(x)
    x = x + 1
"#;
        let result = t.transpile_to_sui(code).unwrap();
        assert!(result.contains(":")); // Has labels
        assert!(result.contains("@")); // Has jump
    }

    #[test]
    fn test_function_def() {
        let mut t = Py2Sui::new();
        let code = r#"
def add(a, b):
    return a + b

result = add(3, 4)
print(result)
"#;
        let result = t.transpile_to_sui(code).unwrap();
        assert!(result.contains("# 0 2 {")); // Function definition
        assert!(result.contains("^")); // Return
        assert!(result.contains("$")); // Function call
    }
}
