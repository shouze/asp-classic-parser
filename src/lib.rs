#![allow(deprecated)]

/// ASP Classic Parser - A parser for ASP Classic (VBScript) syntax
///
/// This library provides parsing capabilities for ASP Classic code using the Pest parsing library.
/// It serves as both a standalone command-line tool and a reusable library for integration
/// with other Rust applications.
// Export the parser module publicly
pub mod parser;

// Export the file utilities module
pub mod file_utils;

// Export the configuration module
pub mod config;

// Export the output formatting module
pub mod output_format;

// Export the caching utilities
pub mod cache;

// Export the self-update utilities
pub mod updater;

// Export the LSP server module
pub mod lsp;
