// filepath: /Users/sebastienhouze/src/asp-classic-parser/src/cache.rs
//! Incremental parsing cache implementation
//!
//! This module implements a cache for parsed files based on their content hash and parsing options.
//! It helps accelerate repeated runs by avoiding re-parsing files that haven't changed.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use thiserror::Error;

/// Cache-related errors
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Serialization/deserialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid cache entry")]
    #[allow(dead_code)]
    InvalidEntry,
}

/// Result type for cache operations
pub type CacheResult<T> = Result<T, CacheError>;

/// Represents a hashed file entry in the cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Path to the file (relative to cache creation)
    pub file_path: PathBuf,

    /// SHA-256 hash of the file content
    pub content_hash: String,

    /// Timestamp of when the entry was added to the cache
    pub timestamp: SystemTime,

    /// Whether the file was successfully parsed
    pub success: bool,

    /// Options hash used for parsing (to invalidate when options change)
    pub options_hash: String,

    /// Error message if parsing failed
    pub error_message: Option<String>,
}

/// Cache for parsed files
#[derive(Debug, Serialize, Deserialize)]
pub struct Cache {
    /// Map of file paths to cache entries
    entries: HashMap<String, CacheEntry>,

    /// When the cache was last modified
    last_modified: SystemTime,

    /// Version of the cache format
    version: String,

    /// Maximum age of cache entries before automatic invalidation (in seconds)
    max_age_secs: u64,
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache {
    /// Create a new, empty cache
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            last_modified: SystemTime::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            max_age_secs: 86400, // 24 hours default
        }
    }

    /// Compute a hash for CLI options to detect when parsing options change
    pub fn hash_options(options: &[String]) -> String {
        let options_str = options.join(",");
        let mut hasher = Sha256::new();
        hasher.update(options_str.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Compute a hash of a file's contents
    pub fn hash_file(path: &Path) -> CacheResult<String> {
        let content = fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Get the path to the cache file
    pub fn get_cache_path() -> PathBuf {
        // Check for environment variable override first
        if let Ok(cache_dir_override) = std::env::var("ASP_PARSER_CACHE_DIR") {
            return PathBuf::from(cache_dir_override).join("parse_cache.json");
        }

        let cache_dir = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("./.cache"));

        // Create app-specific cache directory
        let app_cache_dir = cache_dir.join("asp-classic-parser");

        // Ensure the directory exists
        if !app_cache_dir.exists() {
            let _ = fs::create_dir_all(&app_cache_dir);
        }

        app_cache_dir.join("parse_cache.json")
    }

    /// Load the cache from disk
    pub fn load() -> Self {
        let cache_path = Self::get_cache_path();

        if !cache_path.exists() {
            return Self::new();
        }

        match fs::read_to_string(&cache_path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(cache) => cache,
                Err(e) => {
                    eprintln!("Warning: Failed to parse cache file: {}", e);
                    Self::new()
                }
            },
            Err(e) => {
                eprintln!("Warning: Failed to read cache file: {}", e);
                Self::new()
            }
        }
    }

    /// Save the cache to disk
    pub fn save(&self) -> CacheResult<()> {
        let cache_path = Self::get_cache_path();
        let cache_dir = cache_path.parent().unwrap();

        if !cache_dir.exists() {
            fs::create_dir_all(cache_dir)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(cache_path, json)?;
        Ok(())
    }

    /// Check if a file is in the cache and hasn't changed
    pub fn is_valid(&self, path: &Path, options_hash: &str) -> CacheResult<bool> {
        let path_str = path.to_string_lossy().to_string();

        if let Some(entry) = self.entries.get(&path_str) {
            // Check if the entry is too old
            if let Ok(age) = entry.timestamp.elapsed() {
                if age > Duration::from_secs(self.max_age_secs) {
                    return Ok(false);
                }
            }

            // Check if options have changed
            if entry.options_hash != options_hash {
                return Ok(false);
            }

            // Check if file content has changed
            let current_hash = Self::hash_file(path)?;
            Ok(current_hash == entry.content_hash)
        } else {
            Ok(false)
        }
    }

    /// Add or update a file in the cache
    pub fn update(&mut self, path: &Path, success: bool, options_hash: &str) -> CacheResult<()> {
        self.update_with_error(path, success, options_hash, None)
    }

    /// Add or update a file in the cache with error information
    pub fn update_with_error(
        &mut self,
        path: &Path,
        success: bool,
        options_hash: &str,
        error_message: Option<String>,
    ) -> CacheResult<()> {
        let path_str = path.to_string_lossy().to_string();
        let content_hash = Self::hash_file(path)?;

        let entry = CacheEntry {
            file_path: path.to_path_buf(),
            content_hash,
            timestamp: SystemTime::now(),
            success,
            options_hash: options_hash.to_string(),
            error_message,
        };

        self.entries.insert(path_str, entry);
        self.last_modified = SystemTime::now();
        Ok(())
    }

    /// Get the error message for a file if it exists
    pub fn get_error_message(&self, path: &Path) -> Option<String> {
        let path_str = path.to_string_lossy().to_string();
        self.entries
            .get(&path_str)
            .and_then(|entry| entry.error_message.clone())
    }

    /// Check if a file was successfully parsed according to the cache
    pub fn was_successful(&self, path: &Path) -> Option<bool> {
        let path_str = path.to_string_lossy().to_string();
        self.entries.get(&path_str).map(|entry| entry.success)
    }

    /// Remove a file from the cache
    #[allow(dead_code)]
    pub fn remove(&mut self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_string();

        if self.entries.remove(&path_str).is_some() {
            self.last_modified = SystemTime::now();
            true
        } else {
            false
        }
    }

    /// Clean old entries from the cache
    pub fn clean_old_entries(&mut self) -> usize {
        let now = SystemTime::now();
        let max_age = Duration::from_secs(self.max_age_secs);

        let old_entries: Vec<String> = self
            .entries
            .iter()
            .filter_map(|(path, entry)| match entry.timestamp.elapsed() {
                Ok(age) if age > max_age => Some(path.clone()),
                _ => None,
            })
            .collect();

        let count = old_entries.len();

        for path in old_entries {
            self.entries.remove(&path);
        }

        if count > 0 {
            self.last_modified = now;
        }

        count
    }

    /// Get the number of entries in the cache
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the cache is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Set the maximum age for cache entries
    #[allow(dead_code)]
    pub fn set_max_age(&mut self, seconds: u64) {
        self.max_age_secs = seconds;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::thread::sleep;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Test content").unwrap();

        let hash = Cache::hash_file(file.path()).unwrap();
        assert!(!hash.is_empty());

        // Create another file with the same content - should have the same hash
        let mut file2 = NamedTempFile::new().unwrap();
        writeln!(file2, "Test content").unwrap();

        let hash2 = Cache::hash_file(file2.path()).unwrap();
        assert_eq!(hash, hash2);

        // Create a file with different content - should have a different hash
        let mut file3 = NamedTempFile::new().unwrap();
        writeln!(file3, "Different content").unwrap();

        let hash3 = Cache::hash_file(file3.path()).unwrap();
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_cache_update_and_is_valid() {
        let mut cache = Cache::new();
        let options = vec!["--format=ascii".to_string(), "--verbose".to_string()];
        let options_hash = Cache::hash_options(&options);

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Test content for cache").unwrap();

        // Update the cache with this file
        cache.update(file.path(), true, &options_hash).unwrap();

        // Check if it's valid
        assert!(cache.is_valid(file.path(), &options_hash).unwrap());

        // Check with different options - should be invalid
        let different_options = vec!["--format=json".to_string()];
        let different_hash = Cache::hash_options(&different_options);
        assert!(!cache.is_valid(file.path(), &different_hash).unwrap());

        // Modify the file - should become invalid
        writeln!(file, "Modified content").unwrap();
        assert!(!cache.is_valid(file.path(), &options_hash).unwrap());
    }

    #[test]
    fn test_was_successful() {
        let mut cache = Cache::new();
        let options_hash = "test_hash";

        let mut success_file = NamedTempFile::new().unwrap();
        writeln!(success_file, "Success file").unwrap();

        let mut fail_file = NamedTempFile::new().unwrap();
        writeln!(fail_file, "Failure file").unwrap();

        cache
            .update(success_file.path(), true, options_hash)
            .unwrap();
        cache.update(fail_file.path(), false, options_hash).unwrap();

        assert_eq!(cache.was_successful(success_file.path()), Some(true));
        assert_eq!(cache.was_successful(fail_file.path()), Some(false));

        // Test with a file not in the cache
        let not_in_cache = NamedTempFile::new().unwrap();
        assert_eq!(cache.was_successful(not_in_cache.path()), None);
    }

    #[test]
    fn test_error_message() {
        let mut cache = Cache::new();
        let options_hash = "test_hash";

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Test file with error").unwrap();

        let error_msg = "Parse error at line 5".to_string();
        cache
            .update_with_error(file.path(), false, options_hash, Some(error_msg.clone()))
            .unwrap();

        assert_eq!(cache.get_error_message(file.path()), Some(error_msg));

        // Successful file should have no error message
        let mut success_file = NamedTempFile::new().unwrap();
        writeln!(success_file, "Success file").unwrap();
        cache
            .update(success_file.path(), true, options_hash)
            .unwrap();

        assert_eq!(cache.get_error_message(success_file.path()), None);
    }

    #[test]
    fn test_remove() {
        let mut cache = Cache::new();
        let options_hash = "test_hash";

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Test content").unwrap();

        cache.update(file.path(), true, options_hash).unwrap();
        assert_eq!(cache.len(), 1);

        // Remove the file
        assert!(cache.remove(file.path()));
        assert_eq!(cache.len(), 0);

        // Try removing again - should return false
        assert!(!cache.remove(file.path()));
    }

    #[test]
    fn test_clean_old_entries() {
        let mut cache = Cache::new();
        let options_hash = "test_hash";

        // Set a very short max age for testing
        cache.set_max_age(1); // 1 second

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Test content").unwrap();

        cache.update(file.path(), true, options_hash).unwrap();
        assert_eq!(cache.len(), 1);

        // Wait for the entry to become old
        sleep(Duration::from_secs(2));

        let cleaned = cache.clean_old_entries();
        assert_eq!(cleaned, 1);
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_hash_options() {
        let options1 = vec!["--format=ascii".to_string(), "--verbose".to_string()];
        let options2 = vec!["--verbose".to_string(), "--format=ascii".to_string()];
        let options3 = vec!["--format=json".to_string(), "--verbose".to_string()];

        let hash1 = Cache::hash_options(&options1);
        let hash2 = Cache::hash_options(&options2);
        let hash3 = Cache::hash_options(&options3);

        // Order matters in our current implementation
        assert_ne!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_ne!(hash2, hash3);
    }
}
