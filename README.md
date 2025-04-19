# ASP Classic Parser

A Rust-based parser for ASP Classic (VBScript) syntax, utilizing the Pest parsing library.

## Features

This parser provides comprehensive coverage of ASP Classic syntax including:

- Basic syntax elements (ASP tags, comment handling, statement separators)
- Support for multiple input methods (files, directories, stdin)
- Recursive processing of directories to find all ASP and VBS files
- Support for different encodings, including ISO-8859-1/Latin-1 commonly used in legacy ASP code
- Detailed error reporting with line numbers and error types
- Verbose mode for detailed output during parsing

## Installation

### Option 1: Using the Installation Script (Recommended)

The simplest way to install ASP Classic Parser is by using our installation script which automatically detects your system and downloads the appropriate binary:

```bash
# Download and run the installation script
curl -sSL https://raw.githubusercontent.com/yourusername/asp-classic-parser/main/install.sh | bash

# Or with wget
wget -qO- https://raw.githubusercontent.com/yourusername/asp-classic-parser/main/install.sh | bash
```

This script will:
- Automatically detect your operating system and architecture
- Download the appropriate binary from GitHub releases
- Install it to `~/.local/bin` (configurable with options)
- Make it executable and ready to use

For more options, you can download the script and run it with the `--help` flag:

```bash
curl -O https://raw.githubusercontent.com/yourusername/asp-classic-parser/main/install.sh
chmod +x install.sh
./install.sh --help
```

### Option 2: Manual Download

You can manually download the appropriate binary for your system from the [Releases page](https://github.com/yourusername/asp-classic-parser/releases).

### Option 3: Building from Source

```bash
git clone https://github.com/yourusername/asp-classic-parser.git
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

### Command Line Options

```
Usage: asp-classic-parser [OPTIONS] [FILES/DIRECTORIES...]

Arguments:
  [FILES/DIRECTORIES...]  Files or directories to parse (use '-' for stdin)

Options:
  -v, --verbose           Enable verbose output
  -h, --help              Print help
  -V, --version           Print version
```

## Development Status

This project is under active development. See CHANGELOG.md for version updates and progress on ASP Classic syntax support.

## Contributing

Contributions are welcome! Please see the CONTRIBUTING.md file for guidelines on how to contribute to this project.

## License

This project is licensed under the Apache License 2.0 - see the LICENSE file for details.