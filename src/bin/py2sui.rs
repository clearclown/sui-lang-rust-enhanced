//! Python to Sui (粋) transpiler CLI

use clap::Parser;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use std::process;

use sui_lang::transpiler::Py2Sui;

#[derive(Parser)]
#[command(name = "py2sui")]
#[command(author = "Sui Contributors")]
#[command(version = sui_lang::VERSION)]
#[command(about = "Python to Sui (粋) transpiler")]
#[command(long_about = r#"
Convert Python code to Sui code.

Supports a subset of Python:
  - Variables and assignment
  - Arithmetic operations (+, -, *, /, %)
  - Comparison operations (<, >, <=, >=, ==, !=)
  - Logical operations (and, or, not)
  - If/elif/else statements
  - While loops
  - For loops with range()
  - Function definitions and calls
  - print() and input()
  - Lists (basic support)

Examples:
  py2sui example.py              # Show converted code
  py2sui example.py -o out.sui   # Output to file
"#)]
struct Cli {
    /// Python source file to convert
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Output file path
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,
}

fn print_demo() {
    println!("{}", "Python to Sui (粋) Transpiler".cyan().bold());
    println!("{}", "=".repeat(50));
    println!();
    println!("Usage:");
    println!("  py2sui <file.py>              # Show converted code");
    println!("  py2sui <file.py> -o out.sui   # Output to file");
    println!();
    println!("{}", "Sample 1 - Fibonacci:".yellow());
    println!("{}", "-".repeat(50));

    let sample1 = r#"
def fibonacci(n):
    if n < 2:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

result = fibonacci(10)
print(result)
"#;

    println!("{}", "Python:".green());
    println!("{}", sample1.trim());
    println!();
    println!("{}", "Sui:".green());

    let mut transpiler = Py2Sui::new();
    match transpiler.transpile_to_sui(sample1) {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("Error: {}", e),
    }

    println!();
    println!("{}", "Sample 2 - While Loop:".yellow());
    println!("{}", "-".repeat(50));

    let sample2 = r#"
x = 0
while x < 10:
    print(x)
    x = x + 1
"#;

    println!("{}", "Python:".green());
    println!("{}", sample2.trim());
    println!();
    println!("{}", "Sui:".green());

    let mut transpiler2 = Py2Sui::new();
    match transpiler2.transpile_to_sui(sample2) {
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
    let mut transpiler = Py2Sui::new();
    let sui_code = match transpiler.transpile_to_sui(&code) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}: {}", "Transpile error".red(), e);
            process::exit(1);
        }
    };

    if let Some(output_path) = cli.output {
        // Write to file
        if let Err(e) = fs::write(&output_path, &sui_code) {
            eprintln!("{}: Failed to write file: {}", "Error".red(), e);
            process::exit(1);
        }
        println!("{} Output saved to {}", "✓".green(), output_path.display());
    } else {
        // Print to stdout
        println!("{}", sui_code);
    }
}
