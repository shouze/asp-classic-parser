use colored::*;
use serde_json::json;
use std::env;
use std::fmt;
use std::io::{self, IsTerminal};
use std::path::Path;
use std::str::FromStr;

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

/// Configuration for output display settings
#[derive(Debug, Clone)]
pub struct OutputConfig {
    /// The format to use for output
    pub format: OutputFormat,
    /// Whether to use colors in output
    pub use_colors: bool,
    /// Whether to show successful file parsing messages
    pub show_success: bool,
}

impl OutputConfig {
    /// Check if colors should be used in the current environment
    pub fn should_use_colors(&self) -> bool {
        if !self.use_colors {
            return false;
        }

        // Only use colors if:
        // 1. We're using the ASCII format
        // 2. We're in a terminal
        // 3. Color support isn't explicitly disabled by NO_COLOR env var
        self.format == OutputFormat::Ascii
            && io::stdout().is_terminal()
            && env::var("NO_COLOR").is_err()
    }
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ascii" => Ok(OutputFormat::Ascii),
            "ci" => Ok(OutputFormat::Ci),
            "json" => Ok(OutputFormat::Json),
            "auto" => Ok(OutputFormat::detect_format()),
            _ => Err(format!("Unknown output format: {}", s)),
        }
    }
}

impl OutputFormat {
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
}

/// Format a success message for a file
pub fn format_success(config: &OutputConfig, path: &Path) -> String {
    let path_str = path.display().to_string();
    match config.format {
        OutputFormat::Ascii => {
            let prefix = if config.should_use_colors() {
                "✓".green().to_string()
            } else {
                "✓".to_string()
            };
            format!("{} {} parsed successfully", prefix, path_str)
        }
        OutputFormat::Ci => format!("::notice file={}::Parsed successfully", path_str),
        OutputFormat::Json => format!(
            "{{\"file\": \"{}\", \"status\": \"success\"}}",
            path_str.replace('\\', "\\\\").replace('\"', "\\\"")
        ),
    }
}

/// Format an error message for a file
pub fn format_error(
    config: &OutputConfig,
    file_path: &str,
    line: usize,
    column: usize,
    message: &str,
    severity: &str,
) -> String {
    match config.format {
        OutputFormat::Ascii => {
            let (prefix, formatted_severity) = match severity {
                "error" => {
                    if config.should_use_colors() {
                        ("✖".red().to_string(), "error".red().to_string())
                    } else {
                        ("✖".to_string(), "error".to_string())
                    }
                }
                "warning" => {
                    if config.should_use_colors() {
                        ("⚠".yellow().to_string(), "warning".yellow().to_string())
                    } else {
                        ("⚠".to_string(), "warning".to_string())
                    }
                }
                _ => {
                    if config.should_use_colors() {
                        ("ℹ".blue().to_string(), severity.blue().to_string())
                    } else {
                        ("ℹ".to_string(), severity.to_string())
                    }
                }
            };

            format!(
                "{} {}:{}:{}: {} - {}",
                prefix, file_path, line, column, formatted_severity, message
            )
        }
        OutputFormat::Ci => {
            // GitHub Actions problem-matcher format
            // ::error file={name},line={line},col={col},title={title}::{message}
            format!(
                "::{} file={},line={},col={},title=ASP Parse {}::{}",
                severity.to_lowercase(),
                file_path,
                line,
                column,
                severity.to_uppercase(),
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

/// Format a summary message at the end of parsing
pub fn format_summary(
    config: &OutputConfig,
    success_count: usize,
    fail_count: usize,
    skipped_count: usize,
) -> String {
    match config.format {
        OutputFormat::Ascii => {
            let mut summary = if config.should_use_colors() {
                format!(
                    "Parsing complete: {} succeeded, {} failed, {} skipped",
                    success_count.to_string().green(),
                    if fail_count > 0 {
                        fail_count.to_string().red()
                    } else {
                        fail_count.to_string().normal()
                    },
                    if skipped_count > 0 {
                        skipped_count.to_string().yellow()
                    } else {
                        skipped_count.to_string().normal()
                    }
                )
            } else {
                format!(
                    "Parsing complete: {} succeeded, {} failed, {} skipped",
                    success_count, fail_count, skipped_count
                )
            };

            // Show the specific "skipped - no ASP tags" message if any files were skipped
            if skipped_count > 0 {
                let skipped_msg = format!("{} files skipped – no ASP tags", skipped_count);
                summary.push('\n');
                if config.should_use_colors() {
                    summary.push_str(&skipped_msg.yellow().to_string());
                } else {
                    summary.push_str(&skipped_msg);
                }
            }

            summary
        }
        OutputFormat::Ci => {
            let mut summary = format!(
                "::notice::ASP Classic Parser: {} files succeeded, {} files failed",
                success_count, fail_count
            );

            if skipped_count > 0 {
                summary.push_str(&format!(
                    "\n::notice::ASP Classic Parser: {} files skipped – no ASP tags",
                    skipped_count
                ));
            }

            summary
        }
        OutputFormat::Json => {
            format!(
                "{{\"summary\": {{\"total\": {}, \"success\": {}, \"failed\": {}, \"skipped\": {}, \"skipped_reason\": \"no ASP tags\"}}}}",
                success_count + fail_count + skipped_count,
                success_count,
                fail_count,
                skipped_count
            )
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
        "deprecated_feature" | "potential_bug" | "compatibility_issue" | "no-asp-tags" => "warning",

        // Notices for style and best practices
        "best_practice" | "style_issue" | "performance_tip" => "notice",

        // Default to error for unknown codes
        _ => "error",
    }
}
