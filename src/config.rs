use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur when working with configuration files
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse TOML config: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Invalid configuration value: {0}")]
    #[allow(dead_code)]
    InvalidValue(String),
}

/// Configuration options that can be set in a TOML configuration file
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Config {
    /// Format for output (ascii, ci, json)
    pub format: Option<String>,

    /// Show colored output in terminal
    pub color: Option<bool>,

    /// Verbose output
    pub verbose: Option<bool>,

    /// Hide successful parse messages
    pub quiet_success: Option<bool>,

    /// Treat warnings as errors
    pub strict: Option<bool>,

    /// List of warnings to ignore
    pub ignore_warnings: Option<Vec<String>>,

    /// Comma-separated list of glob patterns to exclude
    pub exclude: Option<String>,

    /// Replace default exclusions instead of extending them
    pub replace_exclude: Option<bool>,
}

impl Config {
    /// Load configuration from a TOML file at the specified path
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Create a default configuration with recommended settings and comments
    pub fn default_with_comments() -> String {
        r#"# ASP Classic Parser Configuration
# This file defines default settings for the parser
# You can place this file in your project directory as:
#   - asp-parser.toml (in the current directory)
#   - .asp-parser.toml (as a hidden file)
# Or in any parent directory, with closer files taking precedence

# Output format: "ascii" (human-readable), "ci" (GitHub Actions), "json" (machine-readable)
# format = "ascii"

# Enable or disable colored output in terminal
# color = true

# Enable verbose output with detailed parsing information
# verbose = false

# Hide success messages for successful files (only show errors and warnings)
# quiet_success = false

# Treat warnings as errors (e.g., files with no ASP tags)
# strict = false

# List of warnings to ignore (e.g., no-asp-tags)
# ignore_warnings = ["no-asp-tags"]

# Comma-separated list of glob patterns to exclude (extends default exclusions)
# exclude = "backup/**,*.tmp"

# Replace default exclusions instead of extending them
# replace_exclude = false
"#
        .to_string()
    }

    /// Look for configuration files in parent directories, starting from the given path
    /// Returns a list of configs from most specific (closest to path) to most general
    pub fn find_configs(start_path: &Path) -> Vec<(PathBuf, Config)> {
        let mut configs = Vec::new();
        let mut current_dir = if start_path.is_file() {
            start_path.parent().map(Path::to_path_buf)
        } else {
            Some(start_path.to_path_buf())
        };

        // Define possible config file names
        let config_filenames = [".asp-parser.toml", "asp-parser.toml"];

        // Walk up directory tree looking for config files
        while let Some(dir) = current_dir {
            for filename in &config_filenames {
                let config_path = dir.join(filename);
                if config_path.exists() {
                    match Config::from_file(&config_path) {
                        Ok(config) => {
                            configs.push((config_path, config));
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Failed to load config from {}: {}",
                                config_path.display(),
                                e
                            );
                        }
                    }
                }
            }

            // Move up to parent directory
            current_dir = dir.parent().map(Path::to_path_buf);
        }

        // Reverse so most specific (closest to path) is first
        configs.reverse();
        configs
    }

    /// Merge configuration options with another config,
    /// where this config's values take precedence over the other
    pub fn merge(&self, other: &Config) -> Config {
        Config {
            format: self.format.clone().or_else(|| other.format.clone()),
            color: self.color.or(other.color),
            verbose: self.verbose.or(other.verbose),
            quiet_success: self.quiet_success.or(other.quiet_success),
            strict: self.strict.or(other.strict),
            ignore_warnings: match (&self.ignore_warnings, &other.ignore_warnings) {
                (Some(ours), Some(theirs)) => {
                    let mut merged = ours.clone();
                    merged.extend(theirs.iter().cloned());
                    Some(merged)
                }
                (Some(ours), None) => Some(ours.clone()),
                (None, Some(theirs)) => Some(theirs.clone()),
                (None, None) => None,
            },
            exclude: self.exclude.clone().or_else(|| other.exclude.clone()),
            replace_exclude: self.replace_exclude.or(other.replace_exclude),
        }
    }

    /// Apply this configuration to the given arguments map
    /// Only sets values that aren't already set in the arguments
    pub fn apply_to_args(&self, args: &mut HashMap<String, String>) {
        // Only set values if they're not already defined in args
        if let Some(format) = &self.format {
            args.entry("format".to_string()).or_insert(format.clone());
        }

        if let Some(color) = self.color {
            let value = if color { "true" } else { "false" };
            args.entry("color".to_string()).or_insert(value.to_string());
        }

        if let Some(verbose) = self.verbose {
            let value = if verbose { "true" } else { "false" };
            args.entry("verbose".to_string())
                .or_insert(value.to_string());
        }

        if let Some(quiet_success) = self.quiet_success {
            let value = if quiet_success { "true" } else { "false" };
            args.entry("quiet-success".to_string())
                .or_insert(value.to_string());
        }

        if let Some(strict) = self.strict {
            let value = if strict { "true" } else { "false" };
            args.entry("strict".to_string())
                .or_insert(value.to_string());
        }

        if let Some(warnings) = &self.ignore_warnings {
            let joined = warnings.join(",");
            args.entry("ignore-warnings".to_string()).or_insert(joined);
        }

        if let Some(exclude) = &self.exclude {
            args.entry("exclude".to_string()).or_insert(exclude.clone());
        }

        if let Some(replace_exclude) = self.replace_exclude {
            let value = if replace_exclude { "true" } else { "false" };
            args.entry("replace-exclude".to_string())
                .or_insert(value.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_from_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
format = "json"
color = false
verbose = true
strict = true
ignore_warnings = ["no-asp-tags", "unused-variable"]
exclude = "node_modules,*.min.js"
"#
        )
        .unwrap();

        let config = Config::from_file(file.path()).unwrap();

        assert_eq!(config.format, Some("json".to_string()));
        assert_eq!(config.color, Some(false));
        assert_eq!(config.verbose, Some(true));
        assert_eq!(config.strict, Some(true));
        assert_eq!(
            config.ignore_warnings,
            Some(vec![
                "no-asp-tags".to_string(),
                "unused-variable".to_string()
            ])
        );
        assert_eq!(config.exclude, Some("node_modules,*.min.js".to_string()));
    }

    #[test]
    fn test_config_merge() {
        let config1 = Config {
            format: Some("json".to_string()),
            color: Some(false),
            verbose: None,
            quiet_success: None,
            strict: Some(true),
            ignore_warnings: Some(vec!["no-asp-tags".to_string()]),
            exclude: None,
            replace_exclude: None,
        };

        let config2 = Config {
            format: Some("ci".to_string()),
            color: None,
            verbose: Some(true),
            quiet_success: Some(true),
            strict: None,
            ignore_warnings: Some(vec!["unused-variable".to_string()]),
            exclude: Some("node_modules".to_string()),
            replace_exclude: None,
        };

        // config1 takes precedence over config2
        let merged = config1.merge(&config2);

        assert_eq!(merged.format, Some("json".to_string())); // From config1
        assert_eq!(merged.color, Some(false)); // From config1
        assert_eq!(merged.verbose, Some(true)); // From config2
        assert_eq!(merged.quiet_success, Some(true)); // From config2
        assert_eq!(merged.strict, Some(true)); // From config1

        // Merged warnings from both
        assert!(merged.ignore_warnings.is_some());
        let warnings = merged.ignore_warnings.unwrap();
        assert!(warnings.contains(&"no-asp-tags".to_string()));
        assert!(warnings.contains(&"unused-variable".to_string()));

        assert_eq!(merged.exclude, Some("node_modules".to_string())); // From config2
    }

    #[test]
    fn test_apply_to_args() {
        let config = Config {
            format: Some("json".to_string()),
            color: Some(false),
            verbose: Some(true),
            quiet_success: None,
            strict: Some(true),
            ignore_warnings: Some(vec!["no-asp-tags".to_string()]),
            exclude: None,
            replace_exclude: None,
        };

        let mut args = HashMap::new();

        // Pre-existing value shouldn't be overwritten
        args.insert("format".to_string(), "ascii".to_string());

        config.apply_to_args(&mut args);

        // Shouldn't be overwritten
        assert_eq!(args.get("format"), Some(&"ascii".to_string()));

        // Should be set from config
        assert_eq!(args.get("color"), Some(&"false".to_string()));
        assert_eq!(args.get("verbose"), Some(&"true".to_string()));
        assert_eq!(args.get("strict"), Some(&"true".to_string()));
        assert_eq!(
            args.get("ignore-warnings"),
            Some(&"no-asp-tags".to_string())
        );

        // These weren't in config, so shouldn't be in args
        assert!(args.get("quiet-success").is_none());
        assert!(args.get("exclude").is_none());
        assert!(args.get("replace-exclude").is_none());
    }

    #[test]
    fn test_default_with_comments() {
        let config_str = Config::default_with_comments();

        // Verify we have comments for all configuration options
        assert!(config_str.contains("# format ="));
        assert!(config_str.contains("# color ="));
        assert!(config_str.contains("# verbose ="));
        assert!(config_str.contains("# quiet_success ="));
        assert!(config_str.contains("# strict ="));
        assert!(config_str.contains("# ignore_warnings ="));
        assert!(config_str.contains("# exclude ="));
        assert!(config_str.contains("# replace_exclude ="));
    }
}
