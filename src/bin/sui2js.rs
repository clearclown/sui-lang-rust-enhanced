//! Sui (粋) to JavaScript transpiler CLI

use clap::Parser;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use std::process::{self, Command};

use sui_lang::transpiler::Sui2Js;

#[derive(Parser)]
#[command(name = "sui2js")]
#[command(author = "Sui Contributors")]
#[command(version = sui_lang::VERSION)]
#[command(about = "Sui (粋) to JavaScript transpiler")]
#[command(long_about = r#"
Convert Sui code to JavaScript code.

Examples:
  sui2js examples/fibonacci.sui           # Show converted code
  sui2js examples/fibonacci.sui -o fib.js # Output to file
  sui2js examples/fib_args.sui --run 15   # Convert and execute with Node.js
  sui2js examples/fibonacci.sui --browser # Generate browser-compatible code
"#)]
struct Cli {
    /// Sui source file to convert
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Output file path
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,

    /// Convert and run immediately with Node.js
    #[arg(long)]
    run: bool,

    /// Generate browser-compatible code (no Node.js APIs)
    #[arg(long)]
    browser: bool,

    /// Generate ES module code
    #[arg(long)]
    esm: bool,

    /// Arguments to pass when running
    #[arg(value_name = "ARGS", last = true)]
    args: Vec<String>,
}

fn print_demo() {
    println!("{}", "Sui (粋) to JavaScript Transpiler".cyan().bold());
    println!("{}", "=".repeat(50));
    println!();
    println!("Usage:");
    println!("  sui2js <file.sui>           # Show converted code");
    println!("  sui2js <file.sui> -o out.js # Output to file");
    println!("  sui2js <file.sui> --run     # Convert and execute with Node.js");
    println!("  sui2js <file.sui> --browser # Generate browser-compatible code");
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
    println!("{}", "JavaScript:".green());

    let mut transpiler = Sui2Js::new();
    match transpiler.transpile_to_js(sample) {
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
    let mut transpiler = Sui2Js::new();
    transpiler.set_nodejs(!cli.browser);
    transpiler.set_esm(cli.esm);

    let js_code = match transpiler.transpile_to_js(&code) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}: {}", "Transpile error".red(), e);
            process::exit(1);
        }
    };

    if let Some(output_path) = cli.output {
        // Write to file
        if let Err(e) = fs::write(&output_path, &js_code) {
            eprintln!("{}: Failed to write file: {}", "Error".red(), e);
            process::exit(1);
        }
        println!("{} Output saved to {}", "✓".green(), output_path.display());
    } else if cli.run {
        // Run with Node.js
        let mut cmd = Command::new("node");
        cmd.arg("-e").arg(&js_code);

        // Pass arguments via NODE_OPTIONS
        for arg in &cli.args {
            cmd.arg("--");
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
                eprintln!("{}: Failed to run Node.js: {}", "Error".red(), e);
                process::exit(1);
            }
        }
    } else {
        // Print to stdout
        println!("{}", js_code);
    }
}
