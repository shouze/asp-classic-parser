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

    // Run the CLI without verbose flag
    let output = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg(asp_file_path.to_str().unwrap())
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check that it shows "File parsed successfully" even in non-verbose mode
    assert!(
        stdout.contains(&format!(
            "File parsed successfully: {}",
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

    // Run the CLI with the Latin-1 file in verbose mode
    let output = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg(latin1_file_path.to_str().unwrap())
        .arg("--verbose")
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check if the file was successfully parsed despite encoding
    assert!(
        stdout.contains("File parsed successfully"),
        "Latin-1 file should parse successfully, got: {}",
        stdout
    );

    // Check summary contains correct count
    assert!(
        stdout.contains("Parsing complete: 1 succeeded, 0 failed"),
        "Latin-1 file should parse successfully, got: {}",
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
    use std::io::Write;
    use std::process::{Command, Stdio};

    // Create a command with stdin piped
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg("-") // Indicate stdin input
        .arg("--verbose")
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
        stdout.contains("File parsed successfully"),
        "File should parse successfully via stdin, got stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}
