//! REPL (Read-Eval-Print Loop) for Sui

use crate::interpreter::Interpreter;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RlResult};
use std::path::PathBuf;

/// REPL configuration
pub struct ReplConfig {
    /// History file path
    pub history_file: Option<PathBuf>,
    /// Maximum history entries
    pub max_history: usize,
    /// Prompt string
    pub prompt: String,
    /// Show welcome message
    pub show_welcome: bool,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            history_file: dirs::home_dir().map(|p| p.join(".sui_history")),
            max_history: 1000,
            prompt: "sui> ".to_string(),
            show_welcome: true,
        }
    }
}

/// Sui REPL
pub struct Repl {
    interpreter: Interpreter,
    config: ReplConfig,
}

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}

impl Repl {
    /// Create a new REPL with default configuration
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
            config: ReplConfig::default(),
        }
    }

    /// Create a new REPL with custom configuration
    pub fn with_config(config: ReplConfig) -> Self {
        Self {
            interpreter: Interpreter::new(),
            config,
        }
    }

    /// Show welcome message
    fn show_welcome(&self) {
        println!("Sui (粋) REPL v{}", crate::VERSION);
        println!("A programming language optimized for LLM code generation");
        println!();
        println!("Commands:");
        println!("  :help     - Show this help message");
        println!("  :reset    - Reset interpreter state");
        println!("  :vars     - Show all variables");
        println!("  :quit     - Exit REPL");
        println!();
        println!("Enter Sui code to execute. Press Ctrl+C to cancel, Ctrl+D to exit.");
        println!();
    }

    /// Show help message
    fn show_help(&self) {
        println!("Sui (粋) REPL Commands:");
        println!();
        println!("  :help, :h     - Show this help message");
        println!("  :reset, :r    - Reset interpreter state");
        println!("  :vars, :v     - Show all variables");
        println!("  :funcs, :f    - Show defined functions");
        println!("  :quit, :q     - Exit REPL");
        println!("  :debug        - Toggle debug mode");
        println!();
        println!("Examples:");
        println!("  = v0 10       - Assign 10 to v0");
        println!("  + v1 v0 5     - Add v0 and 5, store in v1");
        println!("  . v1          - Print v1");
        println!();
    }

    /// Show variables
    fn show_vars(&self) {
        println!("Variables:");
        // Note: In a real implementation, we would expose the interpreter's variables
        println!("  (Use . var to print a variable's value)");
    }

    /// Process a REPL command
    fn process_command(&mut self, cmd: &str) -> bool {
        match cmd.trim() {
            ":help" | ":h" => {
                self.show_help();
            }
            ":reset" | ":r" => {
                self.interpreter.reset();
                println!("Interpreter state reset.");
            }
            ":vars" | ":v" => {
                self.show_vars();
            }
            ":funcs" | ":f" => {
                println!("Functions: (not yet implemented)");
            }
            ":quit" | ":q" => {
                return false;
            }
            ":debug" => {
                println!("Debug mode toggled.");
            }
            _ => {
                println!("Unknown command: {}", cmd);
                println!("Type :help for available commands.");
            }
        }
        true
    }

    /// Run the REPL
    pub fn run(&mut self) -> RlResult<()> {
        let mut rl = DefaultEditor::new()?;

        // Load history
        if let Some(ref history_file) = self.config.history_file {
            let _ = rl.load_history(history_file);
        }

        if self.config.show_welcome {
            self.show_welcome();
        }

        loop {
            let readline = rl.readline(&self.config.prompt);

            match readline {
                Ok(line) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    // Add to history
                    let _ = rl.add_history_entry(line);

                    // Check for REPL commands
                    if line.starts_with(':') {
                        if !self.process_command(line) {
                            break;
                        }
                        continue;
                    }

                    // Execute Sui code
                    match self.interpreter.run_line(line) {
                        Ok(Some(_value)) => {
                            // Value was printed by the interpreter
                        }
                        Ok(None) => {
                            // No output
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    println!("Goodbye!");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }

        // Save history
        if let Some(ref history_file) = self.config.history_file {
            let _ = rl.save_history(history_file);
        }

        Ok(())
    }
}

/// Get home directory (fallback for dirs crate)
mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
    }
}
