# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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