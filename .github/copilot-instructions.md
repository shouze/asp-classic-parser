# GitHub Copilot Instructions

1. **Code Language**  
   - All Rust code, including comments, must be written entirely in **English**.
   - All documentation and commit messages should be consistent and clear in English.

2. **Commit Convention**  
   - Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:
     - **Format:**
       ```
       <type>[optional scope]: <description>

       [optional body]

       [optional footer(s)]
       ```
     - **Common types** include:
       - `feat` (new feature)
       - `fix` (bug fix)
       - `docs` (documentation changes)
       - `style` (formatting, code style)
       - `refactor` (code restructuring without feature changes or bug fixes)
       - `test` (adding or modifying tests)
       - `chore` (misc tasks that do not change code logic)

3. **Clean Code and Organization**  
   - Split code into multiple files (modules) when a single `.rs` file exceeds **500 lines** of code.
   - Keep unit/integration tests in the `tests/` directory (e.g., `tests/expressions.rs`, `tests/statements.rs`, etc.).
   - Run `cargo fmt` for consistent formatting.
   - Run `cargo clippy -- -D warnings` to ensure no warnings remain.

4. **Parser Coverage**  
   - The parser should aim for **complete** coverage of ASP Classic (VBScript) syntax and features.
   - All subtle VBScript aspects (e.g., `Option Explicit`, line continuation `_`, `With…End With`, etc.) must be included.
   - Test extensively with valid (`passing/`) and invalid (`failing/`) fixtures.

5. **Documentation and Versioning**  
   - Use Semantic Versioning (`MAJOR.MINOR.PATCH`).
   - Provide doc comments in English for all functions, structs, and modules.
   - Keep `README.md` and `CHANGELOG.md` up to date after every significant change.

6. **Error Handling and Linter Features**  
   - Provide clear and structured parsing error messages (including line number and error type).
   - Plan to expose an API (crate) offering an AST for ASP Classic code, facilitating linter or advanced tooling integrations.