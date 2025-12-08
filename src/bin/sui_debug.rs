//! Sui Debugger CLI
//!
//! Interactive step debugger for Sui programs.

use clap::Parser;
use std::fs;
use sui_lang::debugger::Debugger;

#[derive(Parser)]
#[command(name = "sui-debug")]
#[command(about = "Interactive debugger for Sui programs")]
#[command(version)]
struct Args {
    /// Sui source file to debug
    file: String,

    /// Set breakpoints at these lines (comma-separated)
    #[arg(short, long, value_delimiter = ',')]
    breakpoints: Option<Vec<usize>>,
}

fn main() {
    let args = Args::parse();

    // Read source file
    let code = match fs::read_to_string(&args.file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", args.file, e);
            std::process::exit(1);
        }
    };

    // Create debugger
    let mut debugger = Debugger::new();

    // Load code
    if let Err(e) = debugger.load(&code) {
        eprintln!("Parse error: {}", e);
        std::process::exit(1);
    }

    // Set initial breakpoints
    if let Some(bps) = args.breakpoints {
        for bp in bps {
            debugger.set_breakpoint(bp);
            println!("Breakpoint set at line {}", bp);
        }
    }

    // Run interactive debugger
    debugger.run_interactive();
}
