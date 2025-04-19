# Contributing to ASP Classic Parser

Thank you for your interest in contributing to the ASP Classic Parser project! This document provides guidelines and instructions for contributing.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for everyone.

## How to Contribute

### Reporting Bugs

If you encounter a bug, please create an issue with the following information:
- A clear and descriptive title.
- A detailed description of the steps to reproduce the bug.
- Expected behavior and what actually happened.
- If possible, include sample code that demonstrates the issue.
- Information about your environment (OS, Rust version, etc.).

### Suggesting Enhancements

If you have an idea for an enhancement, please create an issue with:
- A clear and descriptive title.
- A detailed description of the proposed enhancement.
- An explanation of why this enhancement would be useful.
- If applicable, examples of how the enhancement would work.

### Pull Requests

1. Fork the repository.
2. Create a new branch for your changes.
3. Make your changes following the code style and organization guidelines.
4. Add tests for your changes.
5. Ensure all tests pass by running `cargo test`.
6. Ensure your code meets our style guidelines with `cargo fmt` and `cargo clippy -- -D warnings`.
7. Submit a pull request with a clear description of the changes.

## Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/shouze/asp-classic-parser.git
   cd asp-classic-parser
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run tests:
   ```bash
   cargo test
   ```

## Coding Standards

- All code, including comments, must be written in English.
- Follow the Rust style guide and use `cargo fmt` to format your code.
- Use `cargo clippy -- -D warnings` to check for common mistakes and ensure code quality.
- Write comprehensive documentation for all public functions, structs, and modules.
- Add appropriate tests for new functionality.

## Commit Messages

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Common types include:
- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation changes
- `style`: Formatting changes
- `refactor`: Code restructuring without behavioral changes
- `test`: Adding or modifying tests
- `chore`: Miscellaneous tasks

Examples:
- `feat(parser): add support for variable declarations`
- `fix(error): improve error handling for invalid ASP files`
- `docs(readme): update installation instructions`

## License

By contributing to this project, you agree that your contributions will be licensed under the project's [Apache 2.0 License](LICENSE).