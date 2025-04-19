use clap::{Arg, Command};
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::process;

mod file_utils;
mod parser;

/// Parse a single file and report results
fn parse_file(path: &std::path::Path, verbose: bool) -> bool {
    if verbose {
        println!("Parsing file: {}", path.display());
    }

    match file_utils::read_file_with_encoding(path) {
        Ok(content) => {
            match parser::parse(&content, verbose) {
                Ok(_) => {
                    // Always show success message, even in non-verbose mode
                    println!("File parsed successfully: {}", path.display());
                    true
                }
                Err(e) => {
                    eprintln!("Error parsing file '{}': {}", path.display(), e);
                    false
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading file '{}': {}", path.display(), e);
            false
        }
    }
}

fn main() {
    let matches = Command::new("ASP Classic Parser")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Sébastien Houzé")
        .about("Parse and analyze ASP Classic files")
        .arg(
            Arg::new("files")
                .help("Files or directories to parse (use '-' for stdin)")
                .action(clap::ArgAction::Append)
                .required(false),
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .help("Enable verbose output")
                .action(clap::ArgAction::SetTrue)
                .required(false),
        )
        .get_matches();

    let mut paths_to_parse: Vec<PathBuf> = Vec::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    let verbose = matches.get_flag("verbose");

    // Process paths provided as arguments
    if let Some(files) = matches.get_many::<String>("files") {
        for file in files {
            if file == "-" {
                // Handle stdin input
                let stdin = io::stdin();
                // Use map_while(Result::ok) to safely handle IO errors
                for path in stdin.lock().lines().map_while(Result::ok) {
                    paths_to_parse.push(PathBuf::from(path));
                }
            } else {
                paths_to_parse.push(PathBuf::from(file));
            }
        }
    }

    // If no inputs were provided, show usage information
    if paths_to_parse.is_empty() {
        eprintln!("Error: No input files or directories specified.");
        eprintln!("Usage: asp-classic-parser [FILES/DIRECTORIES...] or - (for stdin)");
        process::exit(1);
    }

    // Process all specified paths
    let mut files_to_parse = Vec::new();

    for path in paths_to_parse {
        if !path.exists() {
            eprintln!(
                "Warning: Path '{}' does not exist, skipping",
                path.display()
            );
            continue;
        }

        if path.is_dir() {
            // For directories, find all ASP/VBS files recursively
            match file_utils::find_asp_files(&path) {
                Ok(found_files) => {
                    files_to_parse.extend(found_files);
                }
                Err(e) => {
                    eprintln!("Error scanning directory '{}': {}", path.display(), e);
                }
            }
        } else {
            // Add individual files directly
            files_to_parse.push(path);
        }
    }

    // Parse all collected files
    if verbose {
        println!("Found {} files to parse", files_to_parse.len());
    }

    for file_path in files_to_parse {
        if parse_file(&file_path, verbose) {
            success_count += 1;
        } else {
            fail_count += 1;
        }
    }

    // Report summary only in verbose mode or if there were failures
    if verbose || fail_count > 0 {
        println!(
            "Parsing complete: {} succeeded, {} failed",
            success_count, fail_count
        );
    }

    // Return non-zero exit code if any file failed to parse
    if fail_count > 0 {
        process::exit(1);
    }
}
