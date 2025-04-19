use std::fs;
use std::path::Path;
use std::process;

mod parser;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <input-file.asp>", args[0]);
        process::exit(1);
    }

    let input_path = Path::new(&args[1]);

    if !input_path.exists() {
        eprintln!("Error: File '{}' does not exist", input_path.display());
        process::exit(1);
    }

    match fs::read_to_string(input_path) {
        Ok(content) => {
            println!("Successfully read file: {}", input_path.display());
            // TODO: Implement parsing logic when parser is more complete
            match parser::parse(&content) {
                Ok(_) => println!("File parsed successfully."),
                Err(e) => {
                    eprintln!("Error parsing file: {}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading file '{}': {}", input_path.display(), e);
            process::exit(1);
        }
    }
}
