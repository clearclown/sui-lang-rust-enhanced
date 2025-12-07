//! Sui (粋) - Main interpreter CLI

use clap::Parser;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use std::process;

use sui_lang::interpreter::{Interpreter, Parser as SuiParser};

#[derive(Parser)]
#[command(name = "sui")]
#[command(author = "Sui Contributors")]
#[command(version = sui_lang::VERSION)]
#[command(about = "Sui (粋) - A programming language optimized for LLM code generation")]
#[command(long_about = r#"
Sui (粋) is a line-based programming language optimized for accurate LLM code generation.

Design Principles:
  1. Line Independence - Each line is completely self-contained
  2. Minimal Bracket Matching - Nesting only for function blocks {}
  3. Single-Character Instructions - Maximum token efficiency
  4. Sequential Variables - No meaningful names needed (v0, v1, g0, a0)
  5. Explicit Control Flow - Labels and jumps

Examples:
  sui examples/fibonacci.sui          # Run a Sui file
  sui examples/fib_args.sui 15        # Run with arguments
  sui --validate examples/fizzbuzz.sui # Validate syntax
  sui --repl                           # Start interactive REPL
"#)]
struct Cli {
    /// Sui source file to run
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Arguments to pass to the Sui program
    #[arg(value_name = "ARGS")]
    args: Vec<String>,

    /// Validate the source file without running
    #[arg(short, long)]
    validate: bool,

    /// Start interactive REPL
    #[arg(short, long)]
    repl: bool,

    /// Enable debug mode
    #[arg(short, long)]
    debug: bool,

    /// Show verbose output
    #[arg(long)]
    verbose: bool,
}

fn print_demo() {
    println!("{}", "Sui (粋) - Programming Language for LLMs".cyan().bold());
    println!("{}", "=".repeat(50));
    println!();
    println!("Usage:");
    println!("  sui <file.sui> [args...]");
    println!("  sui --validate <file.sui>");
    println!("  sui --repl");
    println!();
    println!("Argument access:");
    println!("  g100 = argument count (argc)");
    println!("  g101 = first argument");
    println!("  g102 = second argument");
    println!("  ...");
    println!();
    println!("{}", "Sample execution:".yellow());
    println!("{}", "-".repeat(50));

    // Fibonacci sample
    let fib_code = r#"
; Fibonacci function
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

; Main
= g0 10
$ g1 0 g0
. g1
"#;

    println!("{}", "Sui (Fibonacci):".green());
    println!("{}", fib_code.trim());
    println!();
    println!("{}", "Result:".green());

    let mut interp = Interpreter::new();
    if let Err(e) = interp.run(fib_code, &[]) {
        eprintln!("Error: {}", e);
    }

    println!();
    println!("{}", "-".repeat(50));

    // Loop sample
    let loop_code = r#"
= v0 0
: 0
< v1 v0 10
! v2 v1
? v2 1
. v0
+ v0 v0 1
@ 0
: 1
"#;

    println!("{}", "Sui (0-9 Loop):".green());
    println!("{}", loop_code.trim());
    println!();
    println!("{}", "Result:".green());

    let mut interp = Interpreter::new();
    if let Err(e) = interp.run(loop_code, &[]) {
        eprintln!("Error: {}", e);
    }

    println!();
    println!("{}", "-".repeat(50));
    println!();
    println!("{}", "Features of Sui:".yellow());
    println!();
    println!("{} Minimal bracket matching (only {{}} for functions)", "✓".green());
    println!("{} Line-by-line validation possible", "✓".green());
    println!("{} Error localization by line number", "✓".green());
    println!("{} Maximum token efficiency", "✓".green());
}

fn validate_file(path: &PathBuf) -> bool {
    let code = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}: Failed to read file: {}", "Error".red(), e);
            return false;
        }
    };

    let errors = SuiParser::validate(&code);

    if errors.is_empty() {
        println!("{} Validation successful", "✓".green());
        true
    } else {
        println!("{}", "Validation errors:".red());
        for e in errors {
            println!("  {}", e);
        }
        false
    }
}

fn run_file(path: &PathBuf, args: &[String], debug: bool) {
    let code = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}: Failed to read file: {}", "Error".red(), e);
            process::exit(1);
        }
    };

    let mut interp = Interpreter::new();
    interp.set_debug(debug);

    if let Err(e) = interp.run(&code, args) {
        eprintln!("{}: {}", "Error".red(), e);
        process::exit(1);
    }
}

#[cfg(feature = "repl")]
fn run_repl() {
    use sui_lang::repl::Repl;

    let mut repl = Repl::new();
    if let Err(e) = repl.run() {
        eprintln!("{}: {}", "REPL Error".red(), e);
        process::exit(1);
    }
}

#[cfg(not(feature = "repl"))]
fn run_repl() {
    eprintln!("{}: REPL feature is not enabled", "Error".red());
    eprintln!("Compile with: cargo build --features repl");
    process::exit(1);
}

fn main() {
    let cli = Cli::parse();

    // REPL mode
    if cli.repl {
        run_repl();
        return;
    }

    // If no file specified, show demo
    let Some(file) = cli.file else {
        print_demo();
        return;
    };

    // Check file exists
    if !file.exists() {
        eprintln!("{}: File not found: {}", "Error".red(), file.display());
        process::exit(1);
    }

    // Validate mode
    if cli.validate {
        let success = validate_file(&file);
        process::exit(if success { 0 } else { 1 });
    }

    // Run mode
    run_file(&file, &cli.args, cli.debug);
}
