use std::io::Write;
use std::path::PathBuf;
use crate::log;
use crate::options::ScannerOptions;

const SCANNER_VERSION: &str = "8.0.1.6346";
const SCANNER_URL: &str = "https://binaries.sonarsource.com/Distribution/sonar-scanner-cli/sonar-scanner-cli-{version}-{os}-{arch}.zip";
const SCANNER_DIR: &str = "sonar-scanner-{version}-{os}-{arch}";

/// Downloads the SonarScanner CLI for the specified operating system and architecture, extracts it
/// to the "${HOME}/.sonar/cache/sonar-scanner-{version}-{os}-{arch}/" directory, and returns the path to
/// the scanner executable.
///
/// # Arguments
///
/// * `cache_dir` - The directory where the scanner will be downloaded and extracted.
/// * `os` - The operating system ["linux" (or "alpine"), "macosx" (or "macos"), "windows"]
/// * `arch` - The CPU architecture type. (e.g., "x64", "aarch64").
/// * `out` - A writer used for logging output.
///
/// # Returns
///
/// The path to the scanner executable.
///
pub fn download_scanner(options: &ScannerOptions,  out: &mut impl Write) -> Result<PathBuf, String> {

    let effective_version : &str = options.scanner_version.as_deref().unwrap_or(SCANNER_VERSION);
    let scanner_os = match options.os.as_str() {
        "mac" | "macos" | "darwin" => "macosx",
        "alpine" => "linux",
        "win" | "win32" => "windows",
        other => other,
    };

    let url = SCANNER_URL
        .replace("{version}", effective_version)
        .replace("{os}", scanner_os)
        .replace("{arch}", &options.arch);

    let scanner_dir_name = SCANNER_DIR
        .replace("{version}", effective_version)
        .replace("{os}", scanner_os)
        .replace("{arch}", &options.arch);

    let scanner_dir = options.sonar_cache.join(&scanner_dir_name);
    let sonar_scanner_bin = if scanner_os == "windows" { "sonar-scanner.bat" } else { "sonar-scanner" };
    let sonar_scanner = scanner_dir.join("bin").join(sonar_scanner_bin);

    if !sonar_scanner.is_file() {
        let zip_path = options.sonar_cache.join(format!("{scanner_dir_name}.zip"));
        std::fs::create_dir_all(&options.sonar_cache).map_err(|e| e.to_string())?;

        log(out, &format!("INFO  Downloading {url}"));
        let response = ureq::get(&url).call().map_err(|e| e.to_string())?;
        let mut zip_file = std::fs::File::create(&zip_path).map_err(|e| e.to_string())?;
        std::io::copy(&mut response.into_body().into_reader(), &mut zip_file).map_err(|e| e.to_string())?;
        drop(zip_file);

        log(out, &format!("INFO  Extracting to {}", scanner_dir.display()));
        let zip_file = std::fs::File::open(&zip_path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(zip_file).map_err(|e| e.to_string())?;
        archive.extract(&options.sonar_cache).map_err(|e| e.to_string())?;
    }

    if !sonar_scanner.is_file() {
        let dir = scanner_dir.display();
        let exe = sonar_scanner.display();
        Err(format!("The SonarScanner CLI has been downloaded from '{url}' and extracted into '{dir}' but the executable file '{exe}' is not found"))
    } else {
        Ok(sonar_scanner)
    }
}

pub struct ScannerPaths {
    pub java_home: PathBuf,
    pub java_exe: PathBuf,
    pub sonar_scanner_jar: PathBuf,
}

#[derive(serde::Deserialize)]
struct JreMetadata {
    id: String,
    filename: String,
    sha256: String,
    #[serde(rename = "javaPath")]
    java_path: String,
}

const SONAR_SCANNER_CLI_JAR: &[u8] =
    include_bytes!("../resources/sonar-scanner-cli/sonar-scanner-cli.jar");
pub const SONAR_SCANNER_CLI_JAR_VERSION: &str =
    include_str!("../resources/sonar-scanner-cli/sonar-scanner-cli.jar.version.txt");
const SONAR_SCANNER_CLI_JAR_SHA256: &str =
    include_str!("../resources/sonar-scanner-cli/sonar-scanner-cli.jar.sha256.txt");

pub fn download_jre_extract_scanner(
    options: &ScannerOptions,
    out: &mut impl Write,
) -> Result<ScannerPaths, String> {
    let java_exe = download_jre(options, out)?;
    let java_home = resolve_java_home(&java_exe)?;
    let sonar_scanner_jar = extract_sonar_scanner_jar(options)?;
    Ok(ScannerPaths { java_home, java_exe, sonar_scanner_jar })
}

fn download_jre(options: &ScannerOptions, out: &mut impl Write) -> Result<PathBuf, String> {
    let base_url = options.url.trim_end_matches('/');
    let bearer = format!("Bearer {}", options.token.as_deref().unwrap_or_default());

    let list_url = format!(
        "{base_url}/api/v2/analysis/jres?os={}&arch={}",
        options.os, options.arch
    );
    let response = ureq::get(&list_url)
        .header("Authorization", &bearer)
        .call()
        .map_err(|e| format!("Failed to fetch JRE metadata from {list_url}: {e}"))?;
    let jre_list: Vec<JreMetadata> =
        serde_json::from_reader(response.into_body().into_reader())
            .map_err(|e| format!("Failed to parse JRE metadata: {e}"))?;
    let jre = jre_list
        .into_iter()
        .next()
        .ok_or_else(|| format!("No JRE available for os={} arch={}", options.os, options.arch))?;

    let java_exe = options.sonar_cache.join(&jre.sha256).join(&jre.java_path);
    if java_exe.is_file() {
        return Ok(java_exe);
    }

    let jre_dir = options.sonar_cache.join(&jre.sha256);
    std::fs::create_dir_all(&jre_dir).map_err(|e| e.to_string())?;

    let archive_path = jre_dir.join(&jre.filename);
    let download_url = format!("{base_url}/api/v2/analysis/jres/{}", jre.id);

    log(out, &format!("INFO  Downloading {download_url}"));
    let response = ureq::get(&download_url)
        .header("Authorization", &bearer)
        .header("Accept", "application/octet-stream")
        .call()
        .map_err(|e| format!("Failed to download JRE from {download_url}: {e}"))?;
    let mut archive_file = std::fs::File::create(&archive_path).map_err(|e| e.to_string())?;
    std::io::copy(&mut response.into_body().into_reader(), &mut archive_file)
        .map_err(|e| e.to_string())?;
    drop(archive_file);

    log(out, &format!("INFO  Extracting to {}", jre_dir.display()));
    extract_archive(&archive_path, &jre_dir)?;

    if !java_exe.is_file() {
        return Err(format!(
            "Java executable not found at {} after extraction",
            java_exe.display()
        ));
    }
    #[cfg(unix)]
    set_executable(&java_exe)?;

    Ok(java_exe)
}

#[cfg(unix)]
fn set_executable(path: &std::path::Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path).map_err(|e| e.to_string())?.permissions();
    perms.set_mode(perms.mode() | 0o111);
    std::fs::set_permissions(path, perms).map_err(|e| e.to_string())
}

fn extract_archive(archive_path: &std::path::Path, dest: &std::path::Path) -> Result<(), String> {
    let name = archive_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        let file = std::fs::File::open(archive_path).map_err(|e| e.to_string())?;
        let gz = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(gz);
        archive.unpack(dest).map_err(|e| e.to_string())?;
    } else if name.ends_with(".zip") {
        let file = std::fs::File::open(archive_path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        archive.extract(dest).map_err(|e| e.to_string())?;
    } else {
        return Err(format!("Unsupported archive format: {name}"));
    }
    Ok(())
}

fn resolve_java_home(java_exe: &std::path::Path) -> Result<PathBuf, String> {
    let bin_dir = java_exe
        .parent()
        .ok_or_else(|| format!("Cannot determine bin directory from: {}", java_exe.display()))?;
    let parent = bin_dir
        .parent()
        .ok_or_else(|| format!("Cannot determine parent of bin: {}", bin_dir.display()))?;
    let java_home = if parent.file_name().and_then(|n| n.to_str()) == Some("jre") {
        parent
            .parent()
            .ok_or_else(|| format!("Cannot determine JAVA_HOME from: {}", java_exe.display()))?
            .to_path_buf()
    } else {
        parent.to_path_buf()
    };
    Ok(java_home)
}

fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(data).iter().map(|b| format!("{b:02x}")).collect()
}

fn extract_sonar_scanner_jar(options: &ScannerOptions) -> Result<PathBuf, String> {
    let version = SONAR_SCANNER_CLI_JAR_VERSION.trim();
    let expected = SONAR_SCANNER_CLI_JAR_SHA256.trim();
    let jar_path = options
        .sonar_cache
        .join(expected)
        .join(format!("sonar-scanner-cli-{version}.jar"));

    if jar_path.is_file() {
        let data = std::fs::read(&jar_path).map_err(|e| e.to_string())?;
        let actual = sha256_hex(&data);
        if actual != expected {
            return Err(format!(
                "SHA256 mismatch for {}: expected {expected}, got {actual}",
                jar_path.display()
            ));
        }
        return Ok(jar_path);
    }

    std::fs::create_dir_all(jar_path.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(&jar_path, SONAR_SCANNER_CLI_JAR).map_err(|e| e.to_string())?;

    let actual = sha256_hex(SONAR_SCANNER_CLI_JAR);
    if actual != expected {
        return Err(format!(
            "SHA256 mismatch for embedded jar: expected {expected}, got {actual}"
        ));
    }
    Ok(jar_path)
}

#[cfg(test)]
#[cfg(feature = "integration-tests")]
mod integration_tests {
    use indoc::indoc;
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn download_the_scanner_in_a_temporary_directory() {
        let mut out = Vec::new();

        let tmp_dir = tempdir().unwrap();
        let canonical_tmp_dir = tmp_dir.path().canonicalize().unwrap();
        let options: ScannerOptions = ScannerOptions {
            scanner_version: None,
            os: "macos".to_string(),
            arch: "aarch64".to_string(),
            sonar_home: canonical_tmp_dir.clone(),
            sonar_cache: canonical_tmp_dir.join("cache"),
            ..Default::default()
        };
        std::fs::create_dir_all(&options.sonar_cache).unwrap();

        let scanner_executable = download_scanner(&options, &mut out).unwrap();

        assert_eq!(scanner_executable.to_string_lossy(), options.sonar_cache.join("sonar-scanner-8.0.1.6346-macosx-aarch64/bin/sonar-scanner").to_string_lossy());

        let expected_scanner_dir = canonical_tmp_dir.join("cache").join("sonar-scanner-8.0.1.6346-macosx-aarch64");
        assert_eq!(String::from_utf8(out).unwrap(), indoc! {r#"
        12:00:00.000 INFO  Downloading https://binaries.sonarsource.com/Distribution/sonar-scanner-cli/sonar-scanner-cli-8.0.1.6346-macosx-aarch64.zip
        12:00:00.000 INFO  Extracting to {scanner_dir}
        "#}.replace("{scanner_dir}", &expected_scanner_dir.to_str().unwrap()));

        tmp_dir.close().unwrap();
    }

    #[test]
    fn download_an_old_the_scanner_in_a_temporary_directory() {
        let mut out = Vec::new();

        let tmp_dir = tempdir().unwrap();
        let canonical_tmp_dir = tmp_dir.path().canonicalize().unwrap();
        let options: ScannerOptions = ScannerOptions {
            scanner_version: Some("7.3.0.5189".to_string()),
            os: "linux".to_string(),
            arch: "x64".to_string(),
            sonar_home: canonical_tmp_dir.clone(),
            sonar_cache: canonical_tmp_dir.join("cache"),
            ..Default::default()
        };
        std::fs::create_dir_all(&options.sonar_cache).unwrap();

        let scanner_executable = download_scanner(&options, &mut out).unwrap();

        assert_eq!(scanner_executable.to_string_lossy(), options.sonar_cache.join("sonar-scanner-7.3.0.5189-linux-x64/bin/sonar-scanner").to_string_lossy());

        let expected_scanner_dir = canonical_tmp_dir.join("cache").join("sonar-scanner-7.3.0.5189-linux-x64");
        assert_eq!(String::from_utf8(out).unwrap(), indoc! {r#"
        12:00:00.000 INFO  Downloading https://binaries.sonarsource.com/Distribution/sonar-scanner-cli/sonar-scanner-cli-7.3.0.5189-linux-x64.zip
        12:00:00.000 INFO  Extracting to {scanner_dir}
        "#}.replace("{scanner_dir}", &expected_scanner_dir.to_str().unwrap()));

        tmp_dir.close().unwrap();
    }
}
