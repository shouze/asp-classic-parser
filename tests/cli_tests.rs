use std::fs;
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

// Test the CLI functionality for detecting and processing ASP files
#[test]
fn test_cli_file_processing() {
    // Create a temporary directory with test files
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create sample ASP files
    let asp_file_path = temp_path.join("test.asp");
    fs::write(&asp_file_path, "<% Response.Write \"Hello World\" %>")
        .expect("Failed to write test.asp");

    let vbs_file_path = temp_path.join("test.vbs");
    // Fix: Add proper ASP tags to VBS file to make it valid for our parser
    fs::write(&vbs_file_path, "<% MsgBox \"Hello World\" %>").expect("Failed to write test.vbs");

    // Create a subdirectory with more files
    let subdir_path = temp_path.join("subdir");
    fs::create_dir(&subdir_path).expect("Failed to create subdirectory");

    let nested_asp_path = subdir_path.join("nested.asp");
    fs::write(&nested_asp_path, "<% Response.Write \"Nested\" %>")
        .expect("Failed to write nested.asp");

    // Run the CLI with the temp directory, verbose flag, and explicitly disable exclusions
    let output = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg(temp_path.to_str().unwrap())
        .arg("--verbose")
        .arg("--replace-exclude") // Explicitly disable default exclusions
        .arg("--format=ascii") // Force ASCII format for consistent test results
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);

    println!("CLI Output: {}", stdout);

    // Check if the CLI found the correct number of files
    assert!(
        stdout.contains("Found 3 files to parse"),
        "CLI should find 3 files, got: {}",
        stdout
    );

    // Check if individual files were detected
    assert!(
        stdout.contains(&format!("Parsing file: {}", asp_file_path.display())),
        "Should parse test.asp file"
    );
    assert!(
        stdout.contains(&format!("Parsing file: {}", vbs_file_path.display())),
        "Should parse test.vbs file"
    );
    assert!(
        stdout.contains(&format!("Parsing file: {}", nested_asp_path.display())),
        "Should parse nested.asp file"
    );

    // Check summary contains correct counts
    assert!(
        stdout.contains("Parsing complete: 3 succeeded, 0 failed"),
        "All files should parse successfully"
    );
}

// Test non-verbose mode (default)
#[test]
fn test_cli_non_verbose_output() {
    // Create a temporary directory with test file
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a sample ASP file
    let asp_file_path = temp_path.join("quiet.asp");
    fs::write(&asp_file_path, "<% Response.Write \"Hello World\" %>")
        .expect("Failed to write quiet.asp");

    // Run the CLI without verbose flag, but with ASCII format
    let output = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg(asp_file_path.to_str().unwrap())
        .arg("--format=ascii")
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Non-verbose output: {}", stdout);

    // Check that it shows success message with the correct format
    assert!(
        stdout.contains(&format!(
            "✓ {} parsed successfully",
            asp_file_path.display()
        )),
        "Should show file parsed successfully message"
    );

    // Check that it does NOT show other verbose output
    assert!(
        !stdout.contains("Rule:"),
        "Should not show Rule details in non-verbose mode"
    );
    assert!(
        !stdout.contains("Found"),
        "Should not show 'Found X files' in non-verbose mode"
    );
    assert!(
        !stdout.contains("Parsing file:"),
        "Should not show 'Parsing file' in non-verbose mode"
    );
}

// Test the encoding fallback functionality with a file that's not UTF-8
#[test]
fn test_cli_encoding_fallback() {
    // Create a temporary directory
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a Latin-1 encoded file with special characters
    let latin1_file_path = temp_path.join("latin1.asp");
    let mut file = fs::File::create(&latin1_file_path).expect("Failed to create latin1.asp");

    // ISO-8859-1 bytes for "<%\n' Commentaire avec des accents: é è à ç\nResponse.Write \"Bonjour\"\n%>"
    let latin1_content =
        b"<%\n' Commentaire avec des accents: \xE9 \xE8 \xE0 \xE7\nResponse.Write \"Bonjour\"\n%>";
    file.write_all(latin1_content)
        .expect("Failed to write Latin-1 file");

    // Run the CLI with the Latin-1 file in verbose mode and ASCII format
    let output = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg(latin1_file_path.to_str().unwrap())
        .arg("--verbose")
        .arg("--format=ascii")
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Latin-1 test output: {}", stdout);

    // Check if the file was successfully parsed despite encoding
    assert!(
        stdout.contains(&format!(
            "✓ {} parsed successfully",
            latin1_file_path.display()
        )),
        "Latin-1 file should parse successfully, got: {}",
        stdout
    );

    // Check summary contains correct count
    assert!(
        stdout.contains("Parsing complete: 1 succeeded, 0 failed"),
        "Latin-1 file should parse successfully in summary, got: {}",
        stdout
    );
}

// Test the stdin input method with hyphen (-)
#[test]
fn test_cli_stdin_input() {
    // Create a temporary directory with test files
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a sample ASP file
    let asp_file_path = temp_path.join("stdin_test.asp");
    fs::write(&asp_file_path, "<% Response.Write \"Hello from stdin\" %>")
        .expect("Failed to write stdin_test.asp");

    // Use a cross-platform approach with explicit Stdio handling
    use std::process::{Command, Stdio};

    // Create a command with stdin piped and force ASCII format for consistent testing
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg("-") // Indicate stdin input
        .arg("--verbose")
        .arg("--format=ascii") // Force ASCII format
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped()) // Capture stderr too for better debugging
        .spawn()
        .expect("Failed to spawn CLI process");

    // Write the file path to stdin
    if let Some(mut stdin) = cmd.stdin.take() {
        stdin
            .write_all(asp_file_path.to_string_lossy().as_bytes())
            .expect("Failed to write to stdin");
        // stdin is closed automatically when dropped
    } else {
        panic!("Failed to open stdin");
    }

    // Get the output
    let output = cmd.wait_with_output().expect("Failed to wait for CLI");

    // Convert stdout and stderr to strings for inspection
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Print output for debugging
    println!("CLI Output (stdout): {}", stdout);
    if !stderr.is_empty() {
        println!("CLI Error (stderr): {}", stderr);
    }

    // Check if the file was found
    assert!(
        stdout.contains("Found 1 files to parse"),
        "CLI should find 1 file via stdin, got stdout: {}, stderr: {}",
        stdout,
        stderr
    );

    // Check if the file was successfully parsed
    assert!(
        stdout.contains(&format!(
            "✓ {} parsed successfully",
            asp_file_path.display()
        )),
        "File should parse successfully via stdin, got stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

// Test for the no-asp-tags warning and related flags (--strict, --ignore-warnings)
#[test]
fn test_cli_no_asp_tags() {
    // Create a temporary directory with test files
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a sample HTML file with no ASP tags
    let html_file_path = temp_path.join("no_asp_tags.html");
    fs::write(
        &html_file_path,
        "<html><body><h1>Hello World</h1></body></html>",
    )
    .expect("Failed to write no_asp_tags.html");

    // Create a sample ASP file with valid ASP tags
    let asp_file_path = temp_path.join("valid_tags.asp");
    fs::write(&asp_file_path, "<% Response.Write \"Hello World\" %>")
        .expect("Failed to write valid_tags.asp");

    // Test 1: Default behavior - should treat no-asp-tags as a warning
    let output = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg(html_file_path.to_str().unwrap())
        .arg("--verbose")
        .arg("--format=ascii")
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let exit_code = output.status.code().unwrap_or(-1);

    println!("Default behavior stderr: {}", stderr);
    println!("Default behavior stdout: {}", stdout);

    // Should show a warning and exit with code 0 (success)
    assert!(
        stderr.contains("warning") && stderr.contains("No ASP tags found"),
        "Default behavior should show a warning about no ASP tags"
    );
    assert_eq!(
        exit_code, 0,
        "Default behavior should exit with code 0 (success)"
    );
    assert!(stdout.contains("1 skipped"), "Should report 1 file skipped");
    assert!(
        stdout.contains("1 files skipped – no ASP tags"),
        "Should display the summary line about skipped files"
    );

    // Test 2: With --strict option - should treat no-asp-tags as an error
    let output = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg(html_file_path.to_str().unwrap())
        .arg("--verbose")
        .arg("--format=ascii")
        .arg("--strict")
        .output()
        .expect("Failed to execute CLI");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let exit_code = output.status.code().unwrap_or(-1);

    println!("Strict mode stderr: {}", stderr);

    // Should show an error and exit with code 1 (failure)
    assert!(
        stderr.contains("error") && stderr.contains("No ASP tags found"),
        "Strict mode should treat no ASP tags as an error"
    );
    assert_eq!(exit_code, 1, "Strict mode should exit with code 1 (error)");

    // Test 3: With --ignore-warnings=no-asp-tags - should not show the warning
    let output = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg(html_file_path.to_str().unwrap())
        .arg("--verbose")
        .arg("--format=ascii")
        .arg("--ignore-warnings=no-asp-tags")
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("Ignore warnings stdout: {}", stdout);
    println!("Ignore warnings stderr: {}", stderr);

    // Should not show a warning but still count it as skipped
    assert!(
        !stderr.contains("No ASP tags found"),
        "Should not show warning when it's explicitly ignored"
    );
    assert!(
        stdout.contains("1 skipped"),
        "Should still report 1 file skipped even when warnings are ignored"
    );

    // Test 4: Mixed files - one valid and one without ASP tags
    // Note: For this test, we need to use both files explicitly as arguments since
    // find_asp_files() only looks for .asp and .vbs extensions automatically
    let output = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg(asp_file_path.to_str().unwrap()) // Explicitly add the ASP file
        .arg(html_file_path.to_str().unwrap()) // Explicitly add the HTML file
        .arg("--verbose")
        .arg("--format=ascii")
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("Mixed files stdout: {}", stdout);
    println!("Mixed files stderr: {}", stderr);

    // Should successfully parse the valid file and skip the one without ASP tags
    assert!(
        stdout.contains("Found 2 files to parse") || stdout.contains("Parsing file"),
        "Should process both files: {}",
        stdout
    );
    assert!(
        stdout.contains("Parsing complete: 1 succeeded, 0 failed, 1 skipped"),
        "Should report correct counts for mixed files"
    );
    assert!(
        stderr.contains("No ASP tags found"),
        "Should show warning for the HTML file"
    );
    assert!(
        stdout.contains(&format!(
            "✓ {} parsed successfully",
            asp_file_path.display()
        )),
        "Should show success for the valid ASP file"
    );
}

// Test for the new colored output features and symbols in v0.1.8
#[test]
fn test_cli_colored_output() {
    // Create a temporary directory with test files
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a sample ASP file
    let asp_file_path = temp_path.join("colored_test.asp");
    fs::write(
        &asp_file_path,
        "<% Response.Write \"Test colored output\" %>",
    )
    .expect("Failed to write colored_test.asp");

    // Create a file with syntax error
    let error_file_path = temp_path.join("error_test.asp");
    fs::write(&error_file_path, "<% Response.Write \"Missing closing tag")
        .expect("Failed to write error_test.asp");

    // Create an HTML file (no ASP tags - will generate warning)
    let html_file_path = temp_path.join("warning_test.html");
    fs::write(
        &html_file_path,
        "<html><body>No ASP tags here</body></html>",
    )
    .expect("Failed to write warning_test.html");

    // Test with default settings (colors enabled)
    let output = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg(asp_file_path.to_str().unwrap())
        .arg(error_file_path.to_str().unwrap())
        .arg(html_file_path.to_str().unwrap())
        .arg("--format=ascii")
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("Colored output stdout: {}", stdout);
    println!("Colored output stderr: {}", stderr);

    // Check for colored output markers
    // Note: This only verifies the characters, as we cannot reliably check for ANSI color codes in tests
    assert!(
        stdout.contains("✓"),
        "Success output should contain checkmark symbol"
    );

    assert!(stderr.contains("✖"), "Error output should contain X symbol");

    assert!(
        stderr.contains("⚠"),
        "Warning output should contain warning symbol"
    );
}

// Test the --no-color option
#[test]
fn test_cli_no_color_option() {
    // Create a temporary directory with test file
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a sample ASP file
    let asp_file_path = temp_path.join("no_color_test.asp");
    fs::write(
        &asp_file_path,
        "<% Response.Write \"Test no-color option\" %>",
    )
    .expect("Failed to write no_color_test.asp");

    // Run with --no-color option
    let output = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg(asp_file_path.to_str().unwrap())
        .arg("--format=ascii")
        .arg("--no-color")
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check for the success message with checkmark but without color codes
    assert!(
        stdout.contains("✓") && stdout.contains("parsed successfully"),
        "No-color output should still contain checkmark symbol: {}",
        stdout
    );

    // We can't directly verify absence of color codes in a reliable way across platforms,
    // but we can check that the basic formatting is there
}

// Test the --quiet-success option
#[test]
fn test_cli_quiet_success_option() {
    // Create a temporary directory with test files
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a successful ASP file
    let success_file_path = temp_path.join("quiet_success.asp");
    fs::write(&success_file_path, "<% Response.Write \"Hello\" %>")
        .expect("Failed to write quiet_success.asp");

    // Create an HTML file (no ASP tags - will generate warning)
    let html_file_path = temp_path.join("warning_success.html");
    fs::write(
        &html_file_path,
        "<html><body>No ASP tags here</body></html>",
    )
    .expect("Failed to write warning_success.html");

    // Test with --quiet-success option
    let output = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg(success_file_path.to_str().unwrap())
        .arg(html_file_path.to_str().unwrap())
        .arg("--format=ascii")
        .arg("--quiet-success")
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("Quiet success stdout: {}", stdout);
    println!("Quiet success stderr: {}", stderr);

    // Success message should NOT be present
    assert!(
        !stdout.contains(&format!(
            "{} parsed successfully",
            success_file_path.display()
        )),
        "Success message should not be shown with --quiet-success option"
    );

    // Warning and summary should still be present
    assert!(
        stderr.contains("No ASP tags found"),
        "Warning message should still be shown with --quiet-success option"
    );

    assert!(
        stdout.contains("Parsing complete"),
        "Summary should still be shown with --quiet-success option"
    );
}

// Test the --stdin option for direct code parsing in v0.1.9
#[test]
fn test_cli_stdin_direct_parsing() {
    use std::process::{Command, Stdio};

    // Prepare sample ASP code
    let asp_code = "<% Response.Write \"Direct stdin parsing test\" %>";

    // Create a command with stdin piped and force ASCII format for consistent testing
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg("--stdin") // Use the new stdin option
        .arg("--format=ascii") // Force ASCII format
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped()) // Capture stderr too for better debugging
        .spawn()
        .expect("Failed to spawn CLI process");

    // Write ASP code directly to stdin
    if let Some(mut stdin) = cmd.stdin.take() {
        stdin
            .write_all(asp_code.as_bytes())
            .expect("Failed to write ASP code to stdin");
        // stdin is closed automatically when dropped
    } else {
        panic!("Failed to open stdin");
    }

    // Get the output
    let output = cmd.wait_with_output().expect("Failed to wait for CLI");

    // Convert stdout and stderr to strings for inspection
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let exit_code = output.status.code().unwrap_or(-1);

    // Print output for debugging
    println!("Stdin parsing output (stdout): {}", stdout);
    if !stderr.is_empty() {
        println!("Stdin parsing error (stderr): {}", stderr);
    }

    // Check the exit code is successful
    assert_eq!(
        exit_code, 0,
        "Should exit with code 0 (success), got: {}",
        exit_code
    );

    // Check for success message
    assert!(
        stdout.contains("✓ <stdin> parsed successfully"),
        "Should show success message for stdin parsing, got: {}",
        stdout
    );

    // Check we don't have any error output
    assert!(
        stderr.is_empty(),
        "Should not have error output, got: {}",
        stderr
    );
}

// Test the --stdin option with invalid ASP code
#[test]
fn test_cli_stdin_with_errors() {
    use std::process::{Command, Stdio};

    // Prepare invalid ASP code (missing closing tag)
    let invalid_asp_code = "<% Response.Write \"Missing closing tag";

    // Create a command with stdin piped
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg("--stdin")
        .arg("--format=ascii")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI process");

    // Write invalid ASP code to stdin
    if let Some(mut stdin) = cmd.stdin.take() {
        stdin
            .write_all(invalid_asp_code.as_bytes())
            .expect("Failed to write invalid ASP code to stdin");
    } else {
        panic!("Failed to open stdin");
    }

    // Get the output
    let output = cmd.wait_with_output().expect("Failed to wait for CLI");

    // Convert stdout and stderr to strings for inspection
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let exit_code = output.status.code().unwrap_or(-1);

    println!("Stdin error parsing output (stdout): {}", stdout);
    println!("Stdin error parsing stderr: {}", stderr);

    // Check the exit code indicates an error
    assert_eq!(
        exit_code, 1,
        "Should exit with code 1 (error), got: {}",
        exit_code
    );

    // Check that error output contains the error symbol and useful message
    assert!(
        stderr.contains("✖") && stderr.contains("<stdin>"),
        "Error output should contain error symbol and reference stdin"
    );

    // Check that summary shows 1 file failed
    assert!(
        stdout.contains("0 succeeded, 1 failed"),
        "Summary should show 1 file failed"
    );
}

// Test the --stdin option with content having no ASP tags
#[test]
fn test_cli_stdin_no_asp_tags() {
    use std::process::{Command, Stdio};

    // Prepare HTML content with no ASP tags
    let html_content = "<html><body><h1>No ASP tags here</h1></body></html>";

    // Create a command with stdin piped
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg("--stdin")
        .arg("--format=ascii")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI process");

    // Write HTML content to stdin
    if let Some(mut stdin) = cmd.stdin.take() {
        stdin
            .write_all(html_content.as_bytes())
            .expect("Failed to write HTML content to stdin");
    } else {
        panic!("Failed to open stdin");
    }

    // Get the output
    let output = cmd.wait_with_output().expect("Failed to wait for CLI");

    // Convert stdout and stderr to strings for inspection
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let exit_code = output.status.code().unwrap_or(-1);

    println!("Stdin no-asp-tags output (stdout): {}", stdout);
    println!("Stdin no-asp-tags stderr: {}", stderr);

    // Check the exit code is successful
    assert_eq!(
        exit_code, 0,
        "Should exit with code 0 (success) despite no ASP tags, got: {}",
        exit_code
    );

    // Check that warning output contains the warning symbol
    assert!(
        stderr.contains("⚠") && stderr.contains("No ASP tags found"),
        "Should show warning about no ASP tags"
    );

    // Check that summary shows 1 file skipped
    assert!(
        stdout.contains("0 succeeded, 0 failed, 1 skipped"),
        "Summary should show 1 file skipped"
    );
}

// Test the new configuration file functionality in v0.1.11
#[test]
fn test_cli_config_file() {
    use std::fs::File;
    use std::io::Write;
    use std::process::Command;

    // Create a temporary directory with test files
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create a sample ASP file
    let asp_file_path = temp_path.join("config_test.asp");
    fs::write(&asp_file_path, "<% Response.Write \"Config test\" %>")
        .expect("Failed to write config_test.asp");

    // Create an HTML file (no ASP tags - will generate warning if not ignored)
    let html_file_path = temp_path.join("no_asp_tags.html");
    fs::write(
        &html_file_path,
        "<html><body>No ASP tags here</body></html>",
    )
    .expect("Failed to write no_asp_tags.html");

    // Create a configuration file
    let config_file_path = temp_path.join("asp-parser.toml");
    let mut config_file = File::create(&config_file_path).expect("Failed to create config file");
    writeln!(
        config_file,
        r#"
# ASP Parser Configuration
format = "ascii"       # Use ASCII format
color = false          # Disable colored output
verbose = true         # Enable verbose output
ignore_warnings = ["no-asp-tags"]  # Ignore "no ASP tags" warnings
"#
    )
    .expect("Failed to write config content");

    // Test 1: Use --config to explicitly specify the config file
    let output = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg(asp_file_path.to_str().unwrap())
        .arg(html_file_path.to_str().unwrap())
        .arg("--config")
        .arg(config_file_path.to_str().unwrap())
        .output()
        .expect("Failed to execute CLI with config");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("Config test stdout: {}", stdout);
    println!("Config test stderr: {}", stderr);

    // Verify verbose output is enabled via config
    assert!(
        stdout.contains("Using output format: ascii"),
        "Should show verbose message about output format"
    );

    // With the files we're using, we should see both files being processed
    assert!(
        stdout.contains("Found 2 files to parse"),
        "Should show that 2 files were found"
    );

    // The HTML file should still be skipped even if warning is suppressed
    assert!(stdout.contains("1 skipped"), "Should report 1 file skipped");

    // Test 2: Auto-discovery of configuration file
    // Create a new subdirectory
    let subdir_path = temp_path.join("subdir");
    fs::create_dir(&subdir_path).expect("Failed to create subdirectory");

    // Create a test file in the subdirectory
    let subdir_asp_path = subdir_path.join("subdir_test.asp");
    fs::write(&subdir_asp_path, "<% Response.Write \"Subdir test\" %>")
        .expect("Failed to write subdir_test.asp");

    // Run the parser from the subdirectory without explicitly specifying config
    let output = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg(subdir_asp_path.to_str().unwrap())
        .current_dir(&subdir_path) // Run from the subdirectory
        .output()
        .expect("Failed to execute CLI with auto-discovered config");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // The config in the parent directory should be auto-discovered
    assert!(
        stdout.contains("Using output format: ascii"),
        "Auto-discovered config should enable verbose mode: {}",
        stdout
    );

    // Test 3: CLI arguments should override config file settings
    let output = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg(asp_file_path.to_str().unwrap())
        .arg("--config")
        .arg(config_file_path.to_str().unwrap())
        .arg("--no-color") // Already false in config, but this is explicit
        .arg("--quiet-success") // Not in config, should be applied
        .output()
        .expect("Failed to execute CLI with config and overrides");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should NOT show the success message due to --quiet-success CLI flag overriding config
    assert!(
        !stdout.contains(&format!("{} parsed successfully", asp_file_path.display())),
        "Success message should not be shown with --quiet-success option"
    );
}

// Test the init-config functionality
#[test]
fn test_cli_init_config() {
    // Create a temporary directory
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("test-config.toml");

    // Test 1: Generate config template to a file
    let output = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg("init-config")
        .arg("--output")
        .arg(output_path.to_str().unwrap())
        .output()
        .expect("Failed to execute CLI with init-config");

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Init-config output: {}", stdout);

    // Verify the file was created
    assert!(
        output_path.exists(),
        "Configuration file should have been created"
    );

    // Verify file contains the expected template content
    let content = fs::read_to_string(&output_path).expect("Failed to read config file");
    assert!(
        content.contains("# ASP Classic Parser Configuration"),
        "Config file should have the title comment"
    );
    assert!(
        content.contains("# format ="),
        "Config file should contain format option"
    );
    assert!(
        content.contains("# ignore_warnings ="),
        "Config file should contain ignore_warnings option"
    );

    // Test 2: Generate config template to stdout
    let output = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg("init-config")
        .output()
        .expect("Failed to execute CLI with init-config to stdout");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify that stdout contains the expected template
    assert!(
        stdout.contains("# ASP Classic Parser Configuration"),
        "stdout should contain the config template"
    );
    assert!(
        stdout.contains("# format ="),
        "stdout should contain format option"
    );
    assert!(
        stdout.contains("# ignore_warnings ="),
        "stdout should contain ignore_warnings option"
    );
}
