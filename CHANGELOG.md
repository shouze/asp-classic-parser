# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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