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
- Fix mixed ASP/HTML content edge cases

# 0.1.8
- Improve ascii output format by using ✓ (check mark) in case of successfully parsed file, with color \x1b[32m, ✖ (heavy multiplication X) in case of error with color \x1b[31m and  ⚠ (warning sign) with color \x1b[33m) in case of warning. Detect if colors can be used into the terminal and also provide an option to force not using colors.
- Also add an option to not display successfully parsed files, only ones in error and skipped

# 0.1.9
- Add --stdin to parse code received from standard input, returning diagnostics on standard output in the chosen --format.

# 0.1.10
- Add an upgrade command to self update from the latest release (or a specified one). Should work the same than install.sh, but directly integrated into the binary. A warning message will be displayed in case we downgrade to a former release.

# 0.1.11
- Support --config path.toml so project‑wide default options can be stored and overridden hierarchically.

# 0.1.12
- Implement an incremental parsing cache keyed by file hash and CLI options to accelerate repeated runs; invalidate entries on file change. Add a --no-cache option to run by bypassing the cache.

# 0.1.13
- Expose --threads N (default: logical CPU count) for parallel file processing.

# 0.1.14
- Ship a Language Server Protocol (LSP) server that uses the parser for real‑time diagnostics in editors (VS Code, Neovim, etc.).

# 0.1.15
- Extend the release workflow to publish signed, SBOM‑attached binaries for all targets listed in the GitHub Actions matrix.