# ASP Classic Parser

A Rust-based parser for ASP Classic (VBScript) syntax, utilizing the Pest parsing library.

## Features

This parser provides comprehensive coverage of ASP Classic syntax including:

- Basic syntax elements (ASP tags, comment handling, statement separators)
- Support for multiple input methods (files, directories, stdin)
- Recursive processing of directories to find all ASP and VBS files
- Automatic exclusion of VCS and tooling directories (.git, node_modules, etc.)
- Support for different encodings, including ISO-8859-1/Latin-1 commonly used in legacy ASP code
- Detailed error reporting with line numbers and error types
- Colorized output with distinctive symbols for different message types
- Verbose mode for detailed output during parsing
- Multiple output formats for integration with CI systems or machine processing

## Installation

### Option 1: Using the Installation Script (Recommended)

The simplest way to install ASP Classic Parser is by using our installation script which automatically detects your system and downloads the appropriate binary:

```bash
# Download and run the installation script
curl -sSL https://raw.githubusercontent.com/shouze/asp-classic-parser/refs/heads/master/install.sh | bash

# Or with wget
wget -qO- https://raw.githubusercontent.com/shouze/asp-classic-parser/refs/heads/master/install.sh | bash
```

This script will:
- Automatically detect your operating system and architecture
- Download the appropriate binary from GitHub releases
- Install it to `~/.local/bin` (configurable with options)
- Make it executable and ready to use

For more options, you can download the script and run it with the `--help` flag:

```bash
curl -O https://raw.githubusercontent.com/shouze/asp-classic-parser/refs/heads/master/install.sh
chmod +x install.sh
./install.sh --help
```

### Option 2: Manual Download

You can manually download the appropriate binary for your system from the [Releases page](https://github.com/shouze/asp-classic-parser/releases).

### Option 3: Building from Source

```bash
git clone https://github.com/shouze/asp-classic-parser.git
cd asp-classic-parser
cargo build --release
```

The binary will be available at `target/release/asp-classic-parser`.

## Usage

ASP Classic Parser offers several ways to process your ASP Classic and VBScript files:

### Process Individual Files

```bash
# Parse a single file
asp-classic-parser path/to/file.asp

# Parse with verbose output
asp-classic-parser -v path/to/file.asp
```

### Process Directories

```bash
# Recursively find and parse all .asp and .vbs files in a directory
asp-classic-parser path/to/directory

# Process multiple files and directories at once
asp-classic-parser file1.asp file2.vbs directory1 directory2
```

### Process Files from Standard Input

```bash
# Read files from a file list
cat file_list.txt | asp-classic-parser -

# Process output from another command
find . -name "*.asp" | asp-classic-parser -
```

### Parse ASP Code Directly from Standard Input

```bash
# Parse ASP code provided directly through stdin
echo "<% Response.Write \"Hello World\" %>" | asp-classic-parser --stdin

# Pipe code from a file to be parsed (without reading it as a filename)
cat code_snippet.asp | asp-classic-parser --stdin

# Use with formatting options
cat code_snippet.asp | asp-classic-parser --stdin --format=json
```

### Exclusion Options

By default, the parser excludes common VCS and tooling directories (.git, .svn, node_modules, etc.). You can customize this behavior:

```bash
# Add custom exclusions (in addition to defaults)
asp-classic-parser --exclude="*.bak,old_code/**" path/to/directory

# Replace default exclusions with your own patterns
asp-classic-parser --exclude="logs/**,temp/**" --replace-exclude path/to/directory

# Disable all exclusions (including defaults)
asp-classic-parser --replace-exclude path/to/directory
```

### Output Format Options

```bash
# Use the default ASCII format
asp-classic-parser file.asp

# Use GitHub Actions compatible format
asp-classic-parser --format=ci file.asp

# Use JSON format for machine processing
asp-classic-parser --format=json file.asp

# Automatically detect the best format (default)
asp-classic-parser --format=auto file.asp

# Disable colored output
asp-classic-parser --no-color file.asp

# Hide success messages (only show errors and warnings)
asp-classic-parser --quiet-success file.asp
```

The tool supports three output formats:

1. **ASCII** (default): Human-readable plain text output with colorized symbols:
   - ✓ (green check mark) for successfully parsed files
   - ✖ (red X) for errors
   - ⚠ (yellow warning sign) for warnings
   - ℹ (blue info symbol) for notices

2. **CI**: GitHub Actions compatible format with problem matchers

3. **JSON**: Machine-readable structured data

The automatic detection (`--format=auto` or omitting the format) will:
- Use CI format when running in a CI environment (when CI=true)
- Use CI format when output is not to a terminal (when piped)
- Use ASCII format when in an interactive terminal

### Diagnostic Severity Levels

The parser maps different types of issues to three severity levels:

- **Error**: Critical issues that prevent parsing or render code non-functional
- **Warning**: Potential issues that could cause runtime problems
- **Notice**: Non-critical style or best practice suggestions

| Error Code | Severity | Description |
|------------|----------|-------------|
| parse_error | error | Invalid syntax that prevents parsing |
| syntax_error | error | Valid parse but invalid language syntax |
| encoding_error | error | File encoding issues |
| io_error | error | File reading/writing problems |
| deprecated_feature | warning | Use of deprecated VBScript features |
| potential_bug | warning | Code patterns likely to cause runtime issues |
| compatibility_issue | warning | Features with cross-browser compatibility problems |
| best_practice | notice | Suggestions for code improvement |
| style_issue | notice | Formatting and style guidance |
| performance_tip | notice | Performance optimization suggestions |

### Command Line Options

```
Usage: asp-classic-parser [OPTIONS] [FILES/DIRECTORIES...]

Arguments:
  [FILES/DIRECTORIES...]  Files or directories to parse (use '-' for stdin file list)

Options:
  -v, --verbose             Enable verbose output
  -s, --stdin               Parse ASP code received from standard input
  -f, --format=FORMAT       Output format: ascii (default), ci, json, or auto
      --no-color            Disable colored output in terminal
      --quiet-success       Don't show messages for successfully parsed files
  -e, --exclude=PATTERNS    Comma-separated list of glob patterns to exclude
      --replace-exclude     Replace default exclusions with provided patterns
      --strict              Treat warnings as errors (e.g., no-asp-tags)
      --ignore-warnings=WARNINGS  Comma-separated list of warnings to ignore
  -h, --help                Print help
  -V, --version             Print version
```

## Default Exclusions

The following patterns are excluded by default:

- Version control: `.git/**`, `.svn/**`, `.hg/**`, `.bzr/**`
- IDE and editors: `.idea/**`, `.vscode/**`, `.vs/**`
- Build artifacts: `node_modules/**`, `vendor/**`, `dist/**`, `build/**`, `target/**`
- Package managers: `bower_components/**`, `jspm_packages/**`
- Other common directories: `coverage/**`, `logs/**`, `tmp/**`, `temp/**`

## Development Status

This project is under active development. See CHANGELOG.md for version updates and progress on ASP Classic syntax support.

## Contributing

Contributions are welcome! Please see the CONTRIBUTING.md file for guidelines on how to contribute to this project.

## License

This project is licensed under the Apache License 2.0 - see the LICENSE file for details.