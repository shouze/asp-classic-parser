use std::fs;
use std::io::{self, Read};
use std::path::Path;

/// Unit tests for the file and encoding utilities
/// These tests verify the behavior of the new features:
/// 1. Recursive ASP file finding
/// 2. Encoding fallback support
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    /// Test that find_asp_files finds ASP and VBS files recursively
    #[test]
    fn test_find_asp_files() {
        // Create a temporary directory structure
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();

        // Create the directory structure with ASP and non-ASP files
        fs::create_dir(temp_path.join("subdir")).expect("Failed to create subdirectory");

        // Create files with different extensions
        let files = [
            (temp_path.join("file1.asp"), "ASP content"),
            (temp_path.join("file2.vbs"), "VBS content"),
            (temp_path.join("file3.txt"), "Text content"),
            (
                temp_path.join("subdir").join("file4.asp"),
                "Nested ASP content",
            ),
            (temp_path.join("subdir").join("file5.html"), "HTML content"),
        ];

        // Write test files
        for (path, content) in &files {
            let mut file = fs::File::create(path).expect("Failed to create test file");
            file.write_all(content.as_bytes())
                .expect("Failed to write test file");
        }

        // Run the function we're testing
        let found_files = find_asp_files(temp_path).expect("Finding files failed");

        // Verify results - should find 3 ASP/VBS files
        assert_eq!(found_files.len(), 3, "Should find exactly 3 ASP/VBS files");

        // Check that each ASP/VBS file is in the result
        assert!(
            found_files.contains(&temp_path.join("file1.asp")),
            "Missing file1.asp"
        );
        assert!(
            found_files.contains(&temp_path.join("file2.vbs")),
            "Missing file2.vbs"
        );
        assert!(
            found_files.contains(&temp_path.join("subdir").join("file4.asp")),
            "Missing subdir/file4.asp"
        );

        // Verify non-ASP files are not included
        assert!(
            !found_files.contains(&temp_path.join("file3.txt")),
            "Should not include .txt files"
        );
        assert!(
            !found_files.contains(&temp_path.join("subdir").join("file5.html")),
            "Should not include .html files"
        );
    }

    /// Test reading files with different encodings
    #[test]
    fn test_read_file_with_encoding() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();

        // Create a UTF-8 file
        let utf8_path = temp_path.join("utf8_file.asp");
        fs::write(&utf8_path, "UTF-8 content with unicode: 你好")
            .expect("Failed to write UTF-8 file");

        // Create a Latin-1 file (manually writing bytes)
        let latin1_path = temp_path.join("latin1_file.asp");
        let latin1_content = b"Latin-1 content with special chars: \xE9\xE8\xE0"; // é è à in Latin-1
        fs::write(&latin1_path, latin1_content).expect("Failed to write Latin-1 file");

        // Test reading UTF-8 file
        let utf8_result = read_file_with_encoding(&utf8_path).expect("Failed to read UTF-8 file");
        assert!(
            utf8_result.contains("你好"),
            "UTF-8 file should maintain Unicode characters"
        );

        // Test reading Latin-1 file
        let latin1_result =
            read_file_with_encoding(&latin1_path).expect("Failed to read Latin-1 file");
        assert!(
            latin1_result.contains("éèà"),
            "Latin-1 special chars should be correctly converted"
        );
    }
}

/// Helper function to find ASP and VBScript files recursively (exposed for testing)
pub fn find_asp_files(dir: &Path) -> io::Result<Vec<std::path::PathBuf>> {
    let mut asp_files = Vec::new();

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively search subdirectories
                let mut sub_asp_files = find_asp_files(&path)?;
                asp_files.append(&mut sub_asp_files);
            } else if let Some(ext) = path.extension() {
                // Check if the file has an ASP or VBScript extension
                if ext.eq_ignore_ascii_case("asp") || ext.eq_ignore_ascii_case("vbs") {
                    asp_files.push(path);
                }
            }
        }
    }

    Ok(asp_files)
}

/// Helper function to read file with encoding fallback (exposed for testing)
pub fn read_file_with_encoding(path: &Path) -> io::Result<String> {
    // First try to read as UTF-8
    match fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(_) => {
            // If UTF-8 reading fails, try with latin1 (ISO-8859-1) encoding
            // which is commonly used in legacy ASP Classic files
            let mut file = fs::File::open(path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;

            // Convert from Latin-1 (ISO-8859-1) to UTF-8
            // Latin-1 has a direct 1:1 mapping for the first 256 Unicode code points
            let content = buffer.iter().map(|&b| b as char).collect::<String>();

            Ok(content)
        }
    }
}
