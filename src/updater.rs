use colored::Colorize;
use reqwest::blocking::Client;
use semver::Version;
use serde_json::Value;
use std::env::{self, consts};
use std::fs::{self, File};
use std::io::{self, Read, copy};
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

const REPO: &str = "shouze/asp-classic-parser";
const GITHUB_API_URL: &str = "https://api.github.com/repos";

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("Failed to fetch release information: {0}")]
    FetchError(#[from] reqwest::Error),

    #[error("Failed to parse release data: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Failed to parse version: {0}")]
    VersionError(#[from] semver::Error),

    #[error("No suitable release found")]
    NoReleaseFound,

    #[error("No suitable asset found for your platform")]
    NoAssetFound,

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Unsupported platform: {0}-{1}")]
    UnsupportedPlatform(String, String),

    #[error("Failed to extract archive")]
    ExtractionError,

    #[error("Update aborted: downgrade from {0} to {1}")]
    Downgrade(String, String),

    #[error("Failed to verify checksum")]
    ChecksumError,

    #[error("ZIP error: {0}")]
    ZipError(String),
}

// Implement From<zip::result::ZipError> for UpdateError
impl From<zip::result::ZipError> for UpdateError {
    fn from(error: zip::result::ZipError) -> Self {
        UpdateError::ZipError(error.to_string())
    }
}

/// Information about the platform for which to download the release
struct PlatformInfo {
    target: String,
    extension: String,
    bin_name: String,
}

/// Get the current running executable path
fn get_current_exe() -> io::Result<PathBuf> {
    env::current_exe()
}

/// Check if we're running in a development environment
fn is_dev_environment() -> bool {
    let exe_path = match env::current_exe() {
        Ok(path) => path,
        Err(_) => return true, // Assume dev environment on error to be safe
    };

    // Check if we're in a debug or target directory, which would indicate a development build
    exe_path.to_string_lossy().contains("/target/")
        || exe_path.to_string_lossy().contains("\\target\\")
}

/// Get platform specific information for the download
fn get_platform_info() -> Result<PlatformInfo, UpdateError> {
    let arch = match consts::ARCH {
        "x86_64" | "amd64" => "x86_64",
        "x86" | "i686" => "i686",
        "aarch64" | "arm64" => "aarch64",
        _ => {
            return Err(UpdateError::UnsupportedPlatform(
                consts::OS.to_string(),
                consts::ARCH.to_string(),
            ));
        }
    };

    let (target, extension, bin_name) = match consts::OS {
        "macos" | "darwin" => (
            format!("{}-apple-darwin", arch),
            "tar.gz".to_string(),
            "asp-classic-parser".to_string(),
        ),
        "linux" => (
            format!("{}-unknown-linux-gnu", arch),
            "tar.gz".to_string(),
            "asp-classic-parser".to_string(),
        ),
        "windows" => (
            format!("{}-pc-windows-msvc", arch),
            "zip".to_string(),
            "asp-classic-parser.exe".to_string(),
        ),
        _ => {
            return Err(UpdateError::UnsupportedPlatform(
                consts::OS.to_string(),
                consts::ARCH.to_string(),
            ));
        }
    };

    Ok(PlatformInfo {
        target,
        extension,
        bin_name,
    })
}

/// Get the latest release version from GitHub
fn get_latest_release() -> Result<Value, UpdateError> {
    let client = Client::new();
    let url = format!("{}/{}/releases/latest", GITHUB_API_URL, REPO);

    let response = client
        .get(&url)
        .header("User-Agent", "asp-classic-parser-updater")
        .send()?
        .json()?;

    Ok(response)
}

/// Get a specific release version from GitHub
fn get_specific_release(version: &str) -> Result<Value, UpdateError> {
    let client = Client::new();
    // Ensure version has 'v' prefix
    let version_tag = version.to_string();

    let url = format!("{}/{}/releases/tags/{}", GITHUB_API_URL, REPO, version_tag);

    let response = client
        .get(&url)
        .header("User-Agent", "asp-classic-parser-updater")
        .send()?
        .json()?;

    Ok(response)
}

/// Extract asset URL from release data
fn extract_asset_url(
    release_data: &Value,
    platform_info: &PlatformInfo,
) -> Result<(String, String), UpdateError> {
    let tag_name = release_data["tag_name"]
        .as_str()
        .ok_or(UpdateError::NoReleaseFound)?;

    // Extract version number without 'v' prefix
    let version = if let Some(stripped) = tag_name.strip_prefix('v') {
        stripped
    } else {
        tag_name
    };

    let asset_name = format!(
        "asp-classic-parser-{}-{}.{}",
        version, platform_info.target, platform_info.extension
    );

    // Find the asset with matching name
    if let Some(assets) = release_data["assets"].as_array() {
        for asset in assets {
            if let Some(name) = asset["name"].as_str() {
                if name == asset_name {
                    if let Some(url) = asset["browser_download_url"].as_str() {
                        return Ok((url.to_string(), version.to_string()));
                    }
                }
            }
        }
    }

    Err(UpdateError::NoAssetFound)
}

/// Download an asset from the provided URL
fn download_asset(url: &str, target_path: &Path) -> Result<(), UpdateError> {
    let client = Client::new();
    let mut response = client
        .get(url)
        .header("User-Agent", "asp-classic-parser-updater")
        .send()?;

    let mut file = File::create(target_path)?;
    copy(&mut response, &mut file)?;

    Ok(())
}

/// Download the checksum file for verification
fn download_checksum(url: &str, target_path: &Path) -> Result<(), UpdateError> {
    let checksum_url = format!("{}.sha256", url);
    let client = Client::new();

    let response = client
        .get(&checksum_url)
        .header("User-Agent", "asp-classic-parser-updater")
        .send();

    // It's okay if the checksum file doesn't exist
    match response {
        Ok(mut resp) => {
            if resp.status().is_success() {
                let mut file = File::create(target_path)?;
                copy(&mut resp, &mut file)?;
                Ok(())
            } else {
                // Checksum file not found, which is not a critical error
                Ok(())
            }
        }
        Err(_) => Ok(()),
    }
}

/// Verify the checksum of the downloaded asset
fn verify_checksum(
    asset_path: &Path,
    checksum_path: &Path,
    verbose: bool,
) -> Result<bool, UpdateError> {
    // If checksum file doesn't exist or is empty, skip verification
    if !checksum_path.exists() || fs::metadata(checksum_path)?.len() == 0 {
        if verbose {
            println!("No checksum file found, skipping verification");
        }
        return Ok(true); // Return success when no checksum file exists
    }

    let mut checksum_file = File::open(checksum_path)?;
    let mut checksum_content = String::new();
    checksum_file.read_to_string(&mut checksum_content)?;

    // On Windows, use PowerShell to calculate SHA-256
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
            .args(&[
                "-Command",
                &format!(
                    "(Get-FileHash -Algorithm SHA256 -Path '{}').Hash.ToLower()",
                    asset_path.display()
                ),
            ])
            .output()?;

        if output.status.success() {
            let calculated_hash = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_lowercase();
            let expected_hash = checksum_content
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_lowercase();

            return Ok(calculated_hash == expected_hash);
        }
        return Ok(false);
    }

    // On Unix systems, use shasum
    #[cfg(not(target_os = "windows"))]
    {
        // Extract expected hash from checksum file
        let expected_hash = checksum_content
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_lowercase();

        // Calculate actual hash directly
        let output = Command::new("shasum")
            .args(["-a", "256", asset_path.to_str().unwrap()])
            .output()?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let calculated_hash = output_str
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_lowercase();

            if verbose {
                println!("Expected hash: {}", expected_hash);
                println!("Calculated hash: {}", calculated_hash);
            }

            return Ok(calculated_hash == expected_hash);
        }

        // Fallback to direct comparison method if shasum command fails
        let direct_output = Command::new("shasum")
            .args(["-a", "256", "-c", checksum_path.to_str().unwrap()])
            .current_dir(asset_path.parent().unwrap())
            .output()?;

        Ok(direct_output.status.success())
    }
}

/// Extract the downloaded archive
fn extract_archive(
    archive_path: &Path,
    platform_info: &PlatformInfo,
    output_dir: &Path,
) -> Result<PathBuf, UpdateError> {
    let bin_path = output_dir.join(&platform_info.bin_name);

    if platform_info.extension == "zip" {
        let file = File::open(archive_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;

            if file.name() == platform_info.bin_name {
                let mut outfile = File::create(&bin_path)?;
                copy(&mut file, &mut outfile)?;

                // Make executable on Unix
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&bin_path)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&bin_path, perms)?;
                }
            }
        }
    } else if platform_info.extension == "tar.gz" {
        let file = File::open(archive_path)?;
        let decompressed = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(decompressed);

        archive.unpack(output_dir)?;

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&bin_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&bin_path, perms)?;
        }
    } else {
        return Err(UpdateError::ExtractionError);
    }

    if !bin_path.exists() {
        return Err(UpdateError::ExtractionError);
    }

    Ok(bin_path)
}

/// Replace current executable with the new one
fn replace_executable(new_exe_path: &Path) -> Result<(), UpdateError> {
    let current_exe = get_current_exe()?;

    // Different strategies based on platform
    #[cfg(windows)]
    {
        // On Windows, we can't replace a running executable, so we create a batch file
        // that will copy the new executable over the old one after this process exits
        let batch_path = current_exe.with_extension("bat");
        let batch_content = format!(
            "@echo off\n\
             :loop\n\
             ping -n 2 127.0.0.1 > nul\n\
             copy /Y \"{}\" \"{}\"\n\
             if errorlevel 1 goto loop\n\
             del \"%~f0\"\n",
            new_exe_path.display(),
            current_exe.display()
        );

        let mut batch_file = File::create(&batch_path)?;
        batch_file.write_all(batch_content.as_bytes())?;

        // Execute the batch file in the background
        Command::new("cmd")
            .args(&["/C", "start", "/b", batch_path.to_str().unwrap()])
            .spawn()?;
    }

    #[cfg(unix)]
    {
        // On Unix, we can directly replace the executable if we have permission
        fs::copy(new_exe_path, &current_exe)?;
    }

    Ok(())
}

/// Check if a version is greater than the current version
fn is_version_greater(current: &str, new: &str) -> Result<bool, UpdateError> {
    let current_version = Version::parse(current)?;
    let new_version = Version::parse(new)?;

    Ok(new_version > current_version)
}

/// Print a colored status message
fn print_status(message: &str, is_error: bool) {
    if is_error {
        eprintln!("{}", message.bright_red());
    } else {
        println!("{}", message.bright_green());
    }
}

/// Perform the self-update process
pub fn self_update(
    specified_version: Option<&str>,
    verbose: bool,
    force: bool,
) -> Result<(), UpdateError> {
    // Don't update if in development environment
    if is_dev_environment() {
        print_status(
            "Self-update unavailable in development mode. Use `cargo build` instead.",
            true,
        );
        return Err(UpdateError::IoError(io::Error::new(
            io::ErrorKind::Other,
            "Cannot self-update in development mode",
        )));
    }

    // Get current version from Cargo.toml
    let current_version = env!("CARGO_PKG_VERSION");
    print_status(&format!("Current version: {}", current_version), false);

    // Get platform information
    let platform_info = get_platform_info()?;
    if verbose {
        println!("Detected platform: {}", platform_info.target);
    }

    // Fetch release information
    let release_data = match specified_version {
        Some(version) => {
            print_status(&format!("Fetching release {}", version), false);
            get_specific_release(version)?
        }
        None => {
            print_status("Fetching latest release information...", false);
            get_latest_release()?
        }
    };

    // Extract release version and asset URL
    let (asset_url, version) = extract_asset_url(&release_data, &platform_info)?;

    // If specified version is lower than current, warn and confirm
    if let Some(v) = specified_version {
        let v_str = v.trim_start_matches('v');
        if let Ok(false) = is_version_greater(current_version, v_str) {
            print_status(
                &format!("Warning: downgrading from {} to {}", current_version, v_str),
                true,
            );

            // Abort unless force is specified
            if !force {
                print_status(
                    "Downgrade aborted. Use --force to downgrade to an older version.",
                    true,
                );
                return Err(UpdateError::Downgrade(
                    current_version.to_string(),
                    v_str.to_string(),
                ));
            }

            print_status("Force flag detected. Proceeding with downgrade...", false);
        }
    } else if !is_version_greater(current_version, &version)? {
        print_status(
            &format!(
                "You are already on the latest version ({})",
                current_version
            ),
            false,
        );
        return Ok(());
    }

    print_status(&format!("Downloading version {}...", version), false);
    if verbose {
        println!("Download URL: {}", asset_url);
    }

    // Create a temporary directory for the download
    let temp_dir = tempfile::Builder::new()
        .prefix("asp-classic-parser-update-")
        .tempdir()?;

    // Download the asset
    let asset_path = temp_dir.path().join(format!(
        "asp-classic-parser-{}.{}",
        version, platform_info.extension
    ));
    download_asset(&asset_url, &asset_path)?;

    // Download and verify checksum if available
    let checksum_path = temp_dir.path().join(format!(
        "asp-classic-parser-{}.{}.sha256",
        version, platform_info.extension
    ));
    download_checksum(&asset_url, &checksum_path)?;

    // Verify checksum if it was downloaded
    if checksum_path.exists() && fs::metadata(&checksum_path)?.len() > 0 {
        print_status("Verifying download...", false);
        if verify_checksum(&asset_path, &checksum_path, verbose)? {
            print_status("Checksum verification passed.", false);
        } else {
            print_status("Checksum verification failed!", true);
            return Err(UpdateError::ChecksumError);
        }
    } else if verbose {
        println!("Skipping checksum verification (no checksum file available)");
    }

    // Extract the archive
    print_status("Extracting update...", false);
    let new_exe_path = extract_archive(&asset_path, &platform_info, temp_dir.path())?;

    // Replace the current executable
    print_status("Installing update...", false);
    replace_executable(&new_exe_path)?;

    print_status(&format!("Successfully updated to {}!", version), false);
    print_status("Restart the application to use the new version.", false);

    Ok(())
}
