# 0.1.4
- Exclude common VCS and tooling directories by default (e.g. .git, .svn, .hg, .idea, node_modules, …) during file discovery. Provide a default comma‑separated glob list in --exclude, which users can extend or replace;
- Add unit tests that verify the exclusion logic on Windows, macOS and Linux path separators.

# 0.1.5
- Implement a GitHub Actions problem‑matcher output (::error file=…,line=…,col=…,title=…::message) so the CI can annotate parsing errors inline. Automatically select this format when CI=true or when stdout is not a TTY; still overridable with --format (ascii, ci, json).
- Document the mapping between parser diagnostics and problem‑matcher severity; include sample logs in the README.

# 0.1.6
- Emit a warning/no-asp-tags diagnostic (exit 0) for files that contain no <% or %> tags instead of failing. Introduce --strict to turn the warning into an error, and --ignore-warnings=no-asp-tags to suppress it.
- Print a summary line such as 3 files skipped – no ASP tags at the end of the run.

# 0.1.7
- Treat a trailing </html> followed by any combination of CR, LF or CRLF as valid; do not raise a fatal error.
- Add regression tests covering Windows‑style CRLF endings and mixed newline scenarios.

# 0.1.8
- Add --stdin to parse code received from standard input, returning diagnostics on standard output in the chosen --format.
- Support --config path.toml so project‑wide default options can be stored and overridden hierarchically.

# 0.1.9
- Implement an incremental parsing cache keyed by file hash and CLI options to accelerate repeated runs; invalidate entries on file change.
- Expose --threads N (default: logical CPU count) for parallel file processing.

# 0.1.10
- Ship a Language Server Protocol (LSP) server that uses the parser for real‑time diagnostics in editors (VS Code, Neovim, etc.).
- Extend the release workflow to publish signed, SBOM‑attached binaries for all targets listed in the GitHub Actions matrix.