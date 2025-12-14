// MIT License - New code for Luna-RS CLI
// See LICENSE.MIT for full license text

mod core;

use std::path::Path;
use core::converter::Converter;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        print_usage();
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);

    let content = match std::fs::read_to_string(input_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading {}: {}", input_path.display(), e);
            std::process::exit(1);
        }
    };

    let converter = Converter::new();

    // Detect type from extension
    let ext = input_path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let result = match ext.as_str() {
        "lua" => converter.convert_lua_to_tns(&content, output_path, ""),
        "py" => {
            let filename = input_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("script.py");
            converter.convert_python_to_tns(&content, filename, output_path, "")
        }
        _ => converter.convert_text_to_tns(&content, output_path, ""),
    };

    match result {
        Ok(()) => {
            println!("Created {}", output_path.display());
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("Luna-RS v0.1.0 - TI-Nspire .tns file converter");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("    luna-rs <input> <output.tns>");
    eprintln!();
    eprintln!("SUPPORTED INPUT TYPES:");
    eprintln!("    .lua  - Lua script (OS 3.0.2+)");
    eprintln!("    .py   - Python script (CX II OS 5.2+)");
    eprintln!("    .txt  - Plain text with LaTeX math support");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!("    luna-rs script.lua output.tns");
    eprintln!("    luna-rs notes.txt notes.tns");
    eprintln!();
    eprintln!("LATEX MATH NOTATION:");
    eprintln!("    Greek: \\alpha, \\beta, \\gamma → α, β, γ");
    eprintln!("    Superscripts: x^2 → x²");
    eprintln!("    Subscripts: H_2O → H₂O");
    eprintln!("    Operators: \\pm, \\times, \\div, \\leq, \\geq → ±, ×, ÷, ≤, ≥");
}
