# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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