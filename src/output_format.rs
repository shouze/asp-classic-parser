use serde_json::json;
use std::env;
use std::fmt;
use std::io::{self, IsTerminal};
use std::path::Path;

/// Available output formats for parsing errors
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    /// Standard ASCII text format
    Ascii,
    /// GitHub Actions compatible problem-matcher format
    Ci,
    /// JSON format for machine processing
    Json,
}

impl OutputFormat {
    /// Parse a format from a string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "ascii" => Ok(OutputFormat::Ascii),
            "ci" => Ok(OutputFormat::Ci),
            "json" => Ok(OutputFormat::Json),
            _ => Err(format!("Unknown output format: {}", s)),
        }
    }

    /// Detect the best output format based on environment
    pub fn detect_format() -> Self {
        // Use CI format in CI environments
        if env::var("CI").map(|v| v == "true").unwrap_or(false) {
            return OutputFormat::Ci;
        }

        // Use ASCII in interactive terminals, CI otherwise
        if io::stdout().is_terminal() {
            OutputFormat::Ascii
        } else {
            OutputFormat::Ci
        }
    }

    /// Format a success message for a file
    pub fn format_success(&self, path: &Path) -> String {
        let path_str = path.display().to_string();
        match self {
            OutputFormat::Ascii => format!("✓ {} parsed successfully", path_str),
            OutputFormat::Ci => format!("::notice file={}::Parsed successfully", path_str),
            OutputFormat::Json => format!(
                "{{\"file\": \"{}\", \"status\": \"success\"}}",
                path_str.replace('\\', "\\\\").replace('\"', "\\\"")
            ),
        }
    }

    /// Format an error message for a file
    pub fn format_error(
        &self,
        file_path: &str,
        line: usize,
        column: usize,
        message: &str,
        severity: &str,
    ) -> String {
        match self {
            OutputFormat::Ascii => {
                format!(
                    "{}:{}:{}: {} - {}",
                    file_path, line, column, severity, message
                )
            }
            OutputFormat::Ci => {
                // GitHub Actions problem-matcher format
                // ::error file={name},line={line},col={col},title={title}::{message}
                format!(
                    "::{} file={},line={},col={},title=ASP Parse Error::{}",
                    severity.to_lowercase(),
                    file_path,
                    line,
                    column,
                    message
                )
            }
            OutputFormat::Json => {
                let json_error = json!({
                    "file": file_path,
                    "line": line,
                    "column": column,
                    "message": message,
                    "severity": severity
                });
                json_error.to_string()
            }
        }
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Ascii => write!(f, "ascii"),
            OutputFormat::Ci => write!(f, "ci"),
            OutputFormat::Json => write!(f, "json"),
        }
    }
}

/// Map parser error severity to a string based on error code
///
/// # Severity Levels
///
/// - **error**: Critical issues that prevent parsing or render code non-functional
/// - **warning**: Potential issues that could cause runtime problems
/// - **notice**: Non-critical style or best practice suggestions
///
/// # Error Code Mapping
///
/// | Error Code | Severity | Description |
/// |------------|----------|-------------|
/// | parse_error | error | Invalid syntax that prevents parsing |
/// | syntax_error | error | Valid parse but invalid language syntax |
/// | encoding_error | error | File encoding issues |
/// | io_error | error | File reading/writing problems |
/// | deprecated_feature | warning | Use of deprecated VBScript features |
/// | best_practice | notice | Suggestions for code improvement |
/// | style_issue | notice | Formatting and style guidance |
pub fn map_severity(error_code: &str) -> &'static str {
    match error_code {
        // Critical errors
        "parse_error" | "syntax_error" | "encoding_error" | "io_error" => "error",

        // Warnings for potential issues
        "deprecated_feature" | "potential_bug" | "compatibility_issue" => "warning",

        // Notices for style and best practices
        "best_practice" | "style_issue" | "performance_tip" => "notice",

        // Default to error for unknown codes
        _ => "error",
    }
}
