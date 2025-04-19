use std::process::Command;
use std::str;

#[test]
fn test_upgrade_command_exists() {
    // Simply test that the upgrade command is recognized
    let output = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg("upgrade")
        .arg("--help")
        .output()
        .expect("Failed to execute process");

    let stdout = str::from_utf8(&output.stdout).unwrap();

    // Check if the output contains the expected help information
    assert!(stdout.contains("Upgrade to the latest version"));
    assert!(stdout.contains("--version"));
    assert!(output.status.success());
}

// Note: We cannot fully test the actual upgrade functionality in automated tests
// as it would modify the binary itself. However, we can test that the command
// structure works and specific error paths are handled correctly.

#[test]
fn test_upgrade_invalid_version() {
    // Test that an invalid version number is rejected properly
    let output = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg("upgrade")
        .arg("--version")
        .arg("not-a-version")
        .output()
        .expect("Failed to execute process");

    // The command should fail with a version parsing error
    assert!(!output.status.success());

    let stderr = str::from_utf8(&output.stderr).unwrap();
    assert!(stderr.contains("Error during upgrade") || stderr.contains("version"));
}

#[test]
fn test_upgrade_dev_environment() {
    // In a test environment, the upgrade should detect it's running from a development
    // build and refuse to self-update
    let output = Command::new(env!("CARGO_BIN_EXE_asp-classic-parser"))
        .arg("upgrade")
        .output()
        .expect("Failed to execute process");

    // It should fail because we're in a development environment
    assert!(!output.status.success());

    let stderr = str::from_utf8(&output.stderr).unwrap();
    assert!(stderr.contains("development mode"));
}
