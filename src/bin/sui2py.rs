//! Sui (粋) to Python transpiler CLI

use clap::Parser;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use std::process::{self, Command};

use sui_lang::transpiler::Sui2Py;

#[derive(Parser)]
#[command(name = "sui2py")]
#[command(author = "Sui Contributors")]
#[command(version = sui_lang::VERSION)]
#[command(about = "Sui (粋) to Python transpiler")]
#[command(long_about = r#"
Convert Sui code to Python code.

Examples:
  sui2py examples/fibonacci.sui           # Show converted code
  sui2py examples/fibonacci.sui -o fib.py # Output to file
  sui2py examples/fib_args.sui --run 15   # Convert and execute
"#)]
struct Cli {
    /// Sui source file to convert
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Output file path
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,

    /// Convert and run immediately
    #[arg(long)]
    run: bool,

    /// Arguments to pass when running
    #[arg(value_name = "ARGS", last = true)]
    args: Vec<String>,
}

fn print_demo() {
    println!("{}", "Sui (粋) to Python Transpiler".cyan().bold());
    println!("{}", "=".repeat(50));
    println!();
    println!("Usage:");
    println!("  sui2py <file.sui>           # Show converted code");
    println!("  sui2py <file.sui> -o out.py # Output to file");
    println!("  sui2py <file.sui> --run     # Convert and execute");
    println!();
    println!("{}", "Sample:".yellow());
    println!("{}", "-".repeat(50));

    let sample = r#"
= v0 10
+ v1 v0 5
. v1
"#;

    println!("{}", "Sui:".green());
    println!("{}", sample.trim());
    println!();
    println!("{}", "Python:".green());

    let mut transpiler = Sui2Py::new();
    match transpiler.transpile_to_python(sample) {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn main() {
    let cli = Cli::parse();

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

    // Read source file
    let code = match fs::read_to_string(&file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}: Failed to read file: {}", "Error".red(), e);
            process::exit(1);
        }
    };

    // Transpile
    let mut transpiler = Sui2Py::new();
    let python_code = match transpiler.transpile_to_python(&code) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}: {}", "Transpile error".red(), e);
            process::exit(1);
        }
    };

    if let Some(output_path) = cli.output {
        // Write to file
        if let Err(e) = fs::write(&output_path, &python_code) {
            eprintln!("{}: Failed to write file: {}", "Error".red(), e);
            process::exit(1);
        }
        println!("{} Output saved to {}", "✓".green(), output_path.display());
    } else if cli.run {
        // Run with Python
        let mut cmd = Command::new("python3");
        cmd.arg("-c").arg(&python_code);

        for arg in &cli.args {
            cmd.arg(arg);
        }

        let status = cmd.status();
        match status {
            Ok(s) => {
                if !s.success() {
                    process::exit(s.code().unwrap_or(1));
                }
            }
            Err(e) => {
                eprintln!("{}: Failed to run Python: {}", "Error".red(), e);
                process::exit(1);
            }
        }
    } else {
        // Print to stdout
        println!("{}", python_code);
    }
}
