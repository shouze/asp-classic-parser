# ASP Classic Parser Changelog

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.14] - 2025-04-23

### Added
- Language Server Protocol (LSP) implementation for real-time diagnostics in editors
- New binary `asp-classic-lsp` for standalone LSP server functionality
- Support for LSP features including:
  - Real-time error diagnostics with line/column information
  - Hover information for ASP/VBScript objects and keywords
  - Code completion for common ASP objects and methods
  - Document symbols for navigation (functions, classes, variables)
  - Smart detection of code inside ASP tags
- Multiple connection options (stdio and TCP) for editor integration
- Caching of diagnostics to improve performance in large files
- Detailed documentation for VS Code and Neovim integration

## [0.1.13] - 2025-04-23

### Added
- Parallel file processing with `--threads N` option for improved performance
- Automatically detects the number of logical CPU cores and uses them by default
- Configuration option `threads` in TOML files to set default thread count
- Thread pool management to prevent excessive resource usage
- Smart fallback to sequential processing for single files

## [0.1.12] - 2025-04-22

### Added
- Incremental parsing cache for improved performance on repeated runs
- Automatic file content hash calculation to detect changes
- Cache invalidation based on file content and CLI options
- New `--no-cache` option to bypass the cache and force re-parsing
- Cache configuration option in TOML configuration files
- Cache statistics output in verbose mode

### Changed
- Parse files only when needed (using cached results when available)
- Automatically clean old cache entries (older than 24 hours by default)
- Improved error handling for cache operations
- Cache stored in user's cache directory for persistence between runs

## [0.1.11] - 2025-04-22

### Added
- Support for TOML configuration files with hierarchical discovery
- Added `--config` / `-c` option to explicitly specify a configuration file
- Automatic discovery of configuration files (asp-parser.toml or .asp-parser.toml) in the current directory and its parents
- New `init-config` subcommand to generate a default configuration template
- Support for writing the template to a file with `init-config --output FILE`
- Comprehensive documentation of all available configuration options
- Integration tests for configuration functionality

### Changed
- Refactored code to better separate configuration logic
- Command-line arguments maintain priority over configuration file values
- Enhanced user experience with helpful messages when working with configuration files

## [0.1.10] - 2025-04-19

### Added
- New `upgrade` command to self-update the parser to the latest version
- Support for specifying a target version with `upgrade --version VERSION`
- Automatic platform detection for downloading the appropriate binary
- Checksum verification of downloaded packages
- Downgrade protection with warning when attempting to install an older version
- Detailed progress information during the upgrade process with `--verbose` flag

### Changed
- Restructured CLI to support subcommands while maintaining backward compatibility
- Enhanced error handling for network and file system operations
- Improved platform detection for cross-platform compatibility

## [0.1.9] - 2025-04-19

### Added
- New `--stdin` / `-s` option to parse code received directly from standard input
- Improved error reporting when parsing from stdin with appropriate file reference as `<stdin>`
- Comprehensive tests for stdin parsing capabilities including error handling and "no ASP tags" scenarios

### Changed
- Clarified usage help text to distinguish between stdin for file list (using `-`) and stdin for code content (using `--stdin`)
- Enhanced code organization with a dedicated function for parsing stdin content

## [0.1.8] - 2025-04-19

### Added
- Enhanced ASCII output format with colored symbols for different message types:
  - ✓ (green check mark) for successfully parsed files
  - ✖ (red X) for errors
  - ⚠ (yellow warning sign) for warnings
  - ℹ (blue info symbol) for notices
- New `--no-color` option to disable colored output in terminals
- New `--quiet-success` option to hide messages for successfully parsed files
- Automatic color support detection based on terminal capabilities and environment
- Improved summary output with colored statistics

### Changed
- Made output more visually distinct and easier to scan
- Refactored output formatting code for better organization and extensibility
- Updated error type handling to support new output formatting options

## [0.1.7] - 2025-04-19

### Fixed
- Resolved edge cases in mixed ASP/HTML content parsing
- Improved handling of ASP tags embedded within HTML structure
- Added support for both single and double quoted strings in ASP expressions
- Enhanced robustness of the grammar to handle various real-world ASP coding patterns

## [0.1.6] - 2025-04-19

### Added
- New warning system for files without ASP tags instead of treating them as errors
- Added `--strict` option to treat "no-asp-tags" warnings as errors
- Added `--ignore-warnings` option to suppress specific warnings (e.g., `--ignore-warnings=no-asp-tags`)
- Added summary line showing count of skipped files (e.g., "3 files skipped – no ASP tags")
- Enhanced error type system with `AspErrorKind` enum for better error categorization

### Changed
- Improved file handling logic now returns more detailed `ParseResult` (Success/Error/Skipped)
- Refined error and warning messages for better readability
- Updated tests to comprehensively verify new warning behavior options

## [0.1.5] - 2025-04-19

### Added
- Implemented multiple output formats for parsing errors:
  - ASCII: Human-readable plain text format (default in interactive terminals)
  - CI: GitHub Actions compatible problem-matcher format with inline annotations
  - JSON: Machine-readable structured data for tooling integration
- Added automatic format detection based on environment:
  - Uses CI format when running in CI environments (CI=true)
  - Uses CI format when output is not to a terminal (piped output)
  - Uses ASCII format when in an interactive terminal
- Added `--format` / `-f` CLI option to explicitly select output format
- Added comprehensive documentation about severity mappings in code and README

### Changed
- Restructured error reporting with standardized severity levels:
  - Error: Critical issues that prevent parsing or render code non-functional
  - Warning: Potential issues that could cause runtime problems
  - Notice: Non-critical style or best practice suggestions
- Updated tests to be compatible with all output formats

## [0.1.4] - 2025-04-19

### Added
- Automatic exclusion of common VCS and tooling directories (.git, .svn, .hg, node_modules, etc.) during file discovery
- New `--exclude` option that accepts a comma-separated list of glob patterns to exclude files/directories
- Added `--replace-exclude` option to replace default exclusions instead of extending them
- Comprehensive unit tests for exclusion logic on different operating systems (Windows, macOS, Linux)

## [0.1.3] - 2025-04-19

### Fixed
- Fixed VBS file parsing by properly adding ASP tags in test files
- Improved error handling in stdin processing with safer `map_while(Result::ok)` pattern
- Removed unused `parse_quiet` function for cleaner codebase
- Fixed documentation comments formatting for better readability
- Resolved all Clippy warnings for higher code quality

## [0.1.2] - 2025-04-19

### Added
- Enhanced CLI with support for multiple input methods:
  - Multiple files and directories as command line arguments
  - Recursive directory traversal to find all .asp and .vbs files
  - Reading file/directory paths from stdin with POSIX-style hyphen (-)
- Support for non-UTF-8 encoded files (ISO-8859-1/Latin-1) commonly found in legacy ASP Classic code
- New module `file_utils` with helper functions for file operations
- Comprehensive test coverage for new file handling and CLI features
- Improved error reporting and summary statistics

### Changed
- Reduced verbosity in default output mode - now only displays essential information
- Added `--verbose`/`-v` flag for detailed output during parsing
- Changed from `--stdin`/`-s` to standard POSIX hyphen (`-`) for stdin input
- Always show "File parsed successfully" message for successful files

## [0.1.1] - 2025-04-19

### Added
- Apache License 2.0 file
- Comprehensive CONTRIBUTING.md guidelines for contributors

### Changed
- Improved CI/CD with separate workflows for testing and releases
- Enhanced GitHub Actions configuration with cross-platform build support
  - Added build targets for Linux, macOS, and Windows
  - Configured multiple architectures (x86, x86_64, aarch64)
- Replaced deprecated GitHub Actions with modern equivalents
- Structured release process with proper versioning and asset management

## [0.1.0] - 2025-04-19

### Added
- Project initialization with basic structure
- Basic ASP Classic syntax parsing (Stage 1)
  - ASP delimiters and tags (<%, %>, <%=)
  - Comment handling (single-line comments with ' and REM)
  - Response.Write statement recognition
  - Statement separator (:) support
  - Line continuation with underscore (_)
- Documentation setup (README, CHANGELOG)
- Error handling with detailed error messages
- CI/CD GitHub Actions workflow
- Comprehensive tests for basic syntax including error detection