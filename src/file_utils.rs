use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

/// Unit tests for the file and encoding utilities
/// These tests verify the behavior of the new features:
/// 1. Recursive ASP file finding
/// 2. Encoding fallback support
/// 3. Exclusion logic
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

        // Run the function we're testing - pass "--replace-exclude" to clear default exclusions
        let found_files = find_asp_files(temp_path, &["--replace-exclude".to_string()])
            .expect("Finding files failed");

        // Log the found files for debugging
        println!("Found {} ASP/VBS files:", found_files.len());
        for file in &found_files {
            println!("  - {}", file.display());
        }

        // Verify results - should find 3 ASP/VBS files
        assert_eq!(found_files.len(), 3, "Should find exactly 3 ASP/VBS files");

        // Check if each expected ASP/VBS file is in the result
        let mut found_asp = false;
        let mut found_vbs = false;
        let mut found_nested = false;

        for file in &found_files {
            let file_str = file.to_string_lossy().to_string();
            if file_str.ends_with("file1.asp") {
                found_asp = true;
            } else if file_str.ends_with("file2.vbs") {
                found_vbs = true;
            } else if file_str.ends_with("file4.asp") {
                found_nested = true;
            }
        }

        assert!(found_asp, "Missing file1.asp");
        assert!(found_vbs, "Missing file2.vbs");
        assert!(found_nested, "Missing subdir/file4.asp");
    }

    /// Test exclusion of VCS and other directories
    #[test]
    fn test_exclusion_directories() {
        // Create a temporary directory structure
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();

        // Create various directories that should be excluded
        for dir in &[".git", ".svn", ".hg", "node_modules", ".idea"] {
            fs::create_dir(temp_path.join(dir))
                .expect(&format!("Failed to create {} directory", dir));

            // Add an ASP file in each directory
            let file_path = temp_path.join(dir).join("excluded.asp");
            fs::write(&file_path, "This should be excluded")
                .expect("Failed to write excluded file");
        }

        // Create a regular directory that should be included
        fs::create_dir(temp_path.join("include_dir")).expect("Failed to create include_dir");
        let included_file = temp_path.join("include_dir").join("included.asp");
        fs::write(&included_file, "This should be included")
            .expect("Failed to write included file");

        // Clear any default exclude patterns and only use our custom ones
        let exclude_list = vec![
            "--replace-exclude".to_string(),
            ".git".to_string(),
            ".svn".to_string(),
            ".hg".to_string(),
            "node_modules".to_string(),
            ".idea".to_string(),
        ];

        // Run the search with our explicit exclusions
        let found_files = find_asp_files(temp_path, &exclude_list).expect("Finding files failed");

        // Log the found files for debugging
        println!("Found with custom exclusions: {} files", found_files.len());
        for file in &found_files {
            println!("  - {}", file.display());
        }

        // Should only find the file in the regular directory
        assert_eq!(found_files.len(), 1, "Should only find 1 ASP file");

        // Check if the found file is the included one
        let found_included = found_files
            .iter()
            .any(|p| p.to_string_lossy().to_string().contains("include_dir"));
        assert!(found_included, "Should find the included file");

        // Now test with additional exclusion - exclude the include_dir too
        let exclude_with_include_dir = vec![
            "--replace-exclude".to_string(),
            ".git".to_string(),
            ".svn".to_string(),
            ".hg".to_string(),
            "node_modules".to_string(),
            ".idea".to_string(),
            "include_dir".to_string(),
        ];

        let found_files_custom = find_asp_files(temp_path, &exclude_with_include_dir)
            .expect("Finding files with custom exclude failed");

        println!(
            "Found with include_dir exclusion: {} files",
            found_files_custom.len()
        );
        for file in &found_files_custom {
            println!("  - {}", file.display());
        }

        assert_eq!(
            found_files_custom.len(),
            0,
            "Should find 0 files with include_dir exclude"
        );
    }

    /// Test handling of path separators across different OS
    #[test]
    fn test_path_separators() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();

        // Create nested directories more reliably for cross-platform testing
        let nested_path = temp_path.join("a").join("b").join("c");
        fs::create_dir_all(&nested_path).expect("Failed to create nested directories");

        // Create test ASP file
        let test_file = nested_path.join("test.asp");
        fs::write(&test_file, "Test content").expect("Failed to write test file");

        // For test stability, we'll use --replace-exclude to clear defaults then add our test pattern

        // POSIX style
        let posix_exclude = vec!["--replace-exclude".to_string(), "a/b".to_string()];
        let found_posix = find_asp_files(temp_path, &posix_exclude).expect("POSIX exclude failed");
        assert_eq!(found_posix.len(), 0, "POSIX style exclude should work");

        // Windows style
        let windows_exclude = vec!["--replace-exclude".to_string(), "a\\b".to_string()];
        let found_windows =
            find_asp_files(temp_path, &windows_exclude).expect("Windows exclude failed");
        assert_eq!(found_windows.len(), 0, "Windows style exclude should work");

        // Just the directory name
        let simple_exclude = vec!["--replace-exclude".to_string(), "b".to_string()];
        let found_simple =
            find_asp_files(temp_path, &simple_exclude).expect("Simple exclude failed");
        assert_eq!(
            found_simple.len(),
            0,
            "Simple directory name exclude should work"
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

/// Default patterns to exclude from file search
pub fn default_exclude_patterns() -> Vec<String> {
    vec![
        // Version control systems
        ".git".to_string(),
        ".svn".to_string(),
        ".hg".to_string(),
        ".bzr".to_string(),
        // IDE and editor files
        ".idea".to_string(),
        ".vscode".to_string(),
        ".vs".to_string(),
        // Build artifacts and dependencies
        "node_modules".to_string(),
        "vendor".to_string(),
        "dist".to_string(),
        "target".to_string(),
        "build".to_string(),
        "_build".to_string(),
        // Package manager directories
        "bower_components".to_string(),
        "jspm_packages".to_string(),
        // Other common directories to exclude
        "coverage".to_string(),
        "logs".to_string(),
        "tmp".to_string(),
        "temp".to_string(),
    ]
}

/// Helper function to find ASP and VBScript files recursively, respecting exclude patterns
pub fn find_asp_files(dir: &Path, exclude_patterns: &[String]) -> io::Result<Vec<PathBuf>> {
    // Check for empty dirs early to avoid problems
    if !dir.exists() || !dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut asp_files = Vec::new();

    // Prepare all exclusion patterns
    let mut all_exclude_patterns = Vec::new();

    // Check if the special --replace-exclude flag is present
    let replace_defaults = exclude_patterns.contains(&"--replace-exclude".to_string());

    // Add default exclusions if we're not replacing them
    if !replace_defaults {
        all_exclude_patterns.extend(default_exclude_patterns());
    }

    // Add custom exclusion patterns (except the special flag)
    all_exclude_patterns.extend(
        exclude_patterns
            .iter()
            .filter(|&p| p != "--replace-exclude")
            .cloned(),
    );

    // Find all ASP and VBS files using a simpler, more direct approach
    find_files_simple(dir, &mut asp_files, &all_exclude_patterns)?;

    Ok(asp_files)
}

/// A simpler implementation to find ASP/VBS files that works reliably cross-platform
fn find_files_simple(
    dir: &Path,
    files: &mut Vec<PathBuf>,
    exclude_patterns: &[String],
) -> io::Result<()> {
    // Stack for iterative directory traversal (more reliable than recursion)
    let mut dirs_to_process = vec![dir.to_path_buf()];

    while let Some(current_dir) = dirs_to_process.pop() {
        // Skip this directory if it should be excluded
        if should_exclude(&current_dir, exclude_patterns) {
            continue;
        }

        // Process entries in this directory
        if let Ok(entries) = fs::read_dir(&current_dir) {
            for entry_result in entries {
                if let Ok(entry) = entry_result {
                    let path = entry.path();

                    if path.is_dir() {
                        // Add to stack for later processing if not excluded
                        dirs_to_process.push(path);
                    } else if has_asp_extension(&path) && !should_exclude(&path, exclude_patterns) {
                        // Add ASP/VBS files that aren't excluded
                        files.push(path);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Check if a path has an ASP or VBS extension
fn has_asp_extension(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        ext_str == "asp" || ext_str == "vbs"
    } else {
        false
    }
}

/// Check if a path should be excluded based on the patterns
fn should_exclude(path: &Path, patterns: &[String]) -> bool {
    // We need a simple, cross-platform approach that works reliably

    // First, get the path as a string for easier comparison
    let path_str = path.to_string_lossy().to_string();

    // Next, check if the path contains any of the exclusion patterns
    for pattern in patterns {
        // Basic checks first - for directory name matching
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            if name_str == pattern.as_str() {
                return true;
            }
        }

        // For multi-part paths, normalize slashes and compare more broadly
        let norm_pattern = pattern.replace('\\', "/");
        let norm_path = path_str.replace('\\', "/");

        // Check various matching possibilities:

        // 1. Exact match or path contains pattern
        if norm_path.contains(&norm_pattern) {
            return true;
        }

        // 2. For patterns with slashes, try different combination with the path components
        if norm_pattern.contains('/') {
            // Split path into components for piecewise matching
            let path_components: Vec<&str> = norm_path.split('/').collect();

            // Look for consecutive components that match the pattern
            for window_size in 2..=path_components.len() {
                for i in 0..=path_components.len() - window_size {
                    let path_segment = path_components[i..i + window_size].join("/");
                    if path_segment == norm_pattern {
                        return true;
                    }
                }
            }
        }
    }

    false
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
