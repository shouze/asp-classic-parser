# ASP Classic Parser Development Protocol (Rust + Pest)

## 1. Project Initialization

1. Base Project Structure:
  ```bash
  cargo new asp-classic-parser --bin
  cd asp-classic-parser
  mkdir -p src/parser fixtures/passing fixtures/failing tests
  touch src/parser/grammar.pest
  ```
2. Cargo.toml Declarations:
```toml
[dependencies]
pest = "2.5"
pest_derive = "2.5"
clap = "4.4"
```
3. Git Repository Initialization and Documentation Files:
```sh
git init
echo "# asp-classic-parser" > README.md
touch CHANGELOG.md CONTRIBUTING.md
curl -o LICENSE https://www.apache.org/licenses/LICENSE-2.0.txt
git add .
git commit -m "chore(init): initialize project with basic structure and documentation"
```
4. Language Rules:
- All Rust code, including comments, must be written in English.
- Auxiliary documents (README, CONTRIBUTING, etc.) can be in English for consistency, even if these guidelines are described here in French.

## 2. Commit Conventions

Follow the Conventional Commits specification:
- Format:

```txt
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

- Common Types:
- feat: new feature
- fix: bug fix
- docs: documentation changes
- style: style changes (indentation, formatting)
- refactor: code overhaul without adding features or fixing bugs
- test: adding or modifying tests
- chore: miscellaneous tasks not affecting the source code

Examples:
- feat(parser): add support for variable declarations
- fix(cli): handle empty directory input gracefully
- docs(readme): update usage instructions

## . Clean Code Best Practices

1. Modularization
   - Split code into multiple files (modules) as soon as a single .rs exceeds 500 lines.
   - For example:
   - src/parser/expressions.rs
   - src/parser/statements.rs
   - src/parser/functions.rs
   - src/parser/objects.rs
2. Test Organization
   - Place unit/integration tests in the tests/ directory:
   - tests/expressions.rs
   - tests/statements.rs
   - tests/functions.rs
   - etc.
3. Strict Lint Enforcement
   - Use cargo fmt for formatting.
   - Use cargo clippy -- -D warnings to ensure there are no warnings.

## 4. English Coding Rules

- The code (functions, variables, comments) must be 100% in English.
- Regularly verify that all contributions (including commits, docstrings) meet this requirement to maintain consistency and facilitate collaboration.

## 5. Parser Exhaustiveness and VBScript Nuances

In addition to major features (controls, functions, ASP objects), be sure to cover VBScript peculiarities:
- Option Explicit: force explicit variable declaration
- : (colon) for chaining multiple statements on one line
- Line continuation: _ at the end of a line
- With ... End With: blocks that simplify object property access
- Exit For, Exit Do, Exit Function, Exit Sub
- VBScript classes (Class ... End Class)
- Const: constant declaration
- Native constants (e.g., vbCrLf, vbTab, etc.)

These points must be included in the grammar to achieve 100% syntactic coverage.


## 6. Implementation Plan and Validation Steps

The plan below draws on W3Schools documentation and practical experience. Adjust priorities as needed based on real-world requirements and nuances that must be integrated early (e.g., With, Option Explicit).

1. Stage 1: Basic Syntax
   - <% ... %>
   - Comments (', REM)
   - Response.Write
   - : character and line continuation _
2. Stage 2: Variable Declarations
   - Dim, Public, Private
   - Assignments, arrays (1D/multi), Empty, Null, Nothing
   - Option Explicit
3. Stage 3: Control Structures
   - If...Then, Else, ElseIf, Select Case
   - For...Next, For Each...Next, Do...Loop, While...Wend
   - Exit For, Exit Do, etc.
4. Stage 4: Procedures and Functions
   - Sub ... End Sub, Function ... End Function
   - Calls with or without Call
   - Exit Function, Exit Sub, parameters (ByVal, ByRef)
   - With ... End With (optional or in a dedicated module)
5. Stage 5: Built-in Functions
   - String functions (Len, Mid, InStr, Replace, etc.)
   - Date/Time functions (Now, DateAdd, DateDiff, etc.)
   - Conversions (CInt, CStr, etc.)
6. Stage 6: ASP Objects
   - Request, Response, Session, Application, Server
   - Cookies, forms (Request.Form, Request.QueryString)
7. Stage 7: File Inclusions
   - <!--#include file="..." -->, <!--#include virtual="..." -->
8. Stage 8: global.asa File
   - Application_OnStart, Application_OnEnd
   - Session_OnStart, Session_OnEnd
9. Stage 9: Error Handling
   - On Error Resume Next, On Error GoTo 0
   - Err object (properties: Number, Description, etc.)
10. Stage 10: Database Access (ADO)
    - Connection, Recordset, Command, etc.
    - Methods/properties (Open, Execute, Fields, etc.)
11. Stage 11: File Manipulation
    - FileSystemObject, TextStream, Drive, File, Folder
    - Methods (CreateTextFile, OpenTextFile, Read, Write, etc.)
12. Stage 12: Advanced / Miscellaneous Functions
    - AdRotator, Browser, ContentLinking, ContentRotator, etc.
    - VBScript classes (Class ... End Class) if needed

## 7. Testing Methodology and Fixtures

1. Unit Tests
   - For each feature (e.g., loops, variables), include both success and failure cases.
2. Fixtures
   - passing/: valid examples (e.g., fixtures/passing/control_structures.asp)
   - failing/: deliberately incorrect examples (e.g., fixtures/failing/control_structures.asp)
   - Cover a wide range of scenarios (includes, line continuations, nested structures, etc.).
3. Validation
   - Run cargo test after each implementation.
   - Check coverage (Tarpaulin or equivalent).
   - Set up Continuous Integration (GitHub Actions or similar).

## 8. Versioning and Documentation

1. Versioning
    - Increment the version using cargo set-version after each major    implementation.
    - Use Semantic Versioning (MAJOR.MINOR.PATCH).
2. Documentation
   - Update README.md and CHANGELOG.md after each addition.
   - Document the parser’s API (functions, structs) with doc comments in   English.
   - Provide instructions on how to use the binary (e.g., parse file.asp).
3. Maintenance
   - Schedule periodic audits or refactoring (e.g., every X sprints).
   - Manage issue tracking (GitHub or other).

## 9. Error Generation and Diagnostic Messages

- User Feedback:
- Parsing errors should be clear, ideally pointing to the line in question and the nature of the error.
- For linting purposes, provide structured messages (warnings, correction suggestions).


## 10. Outlook for an ASP Classic Linter

1. **Public API**  
   - Expose the parser as a Rust crate (e.g., `parse()` functions, etc.).  
   - Provide an AST that accurately models ASP code.
2. **Extensibility**  
   - Allow lint rules to be added as plugins or dedicated modules, facilitating clean separation of concerns and easier maintenance.
3. **Roadmap**  
   - **Lint Rule Implementation**: After stabilizing the core parser, write additional lint rules (e.g., undeclared variables, `Option Explicit`, etc.) to ensure best practices and help detect common mistakes.  
   - **Testing on Real Projects**: Validate the parser and lint rules on existing ASP Classic codebases to gather feedback and refine the ruleset and architecture.  
   - **Performance Optimization**: Investigate and improve parsing speed, especially for large ASP projects with complex or nested structures.  
   - **CI/CD Integration**: Provide or document how to integrate the parser with popular CI/CD pipelines (e.g., GitHub Actions, GitLab CI) for automated testing and linting.  
   - **Documentation and Ecosystem Growth**: Encourage community contributions by continuously updating tutorials, API docs, and usage examples; consider providing sample projects or a dedicated website/portal.  
   - **Long-Term Enhancements**: Explore advanced features such as partial code analysis, auto-fixes or suggestions, and integration with broader tooling ecosystems (IDEs, editors, etc.).