use std::env;
use std::path::{Path, PathBuf};
use regex::Regex;
use crate::options::{LogLevel, ScannerOptions};
use crate::props;

pub fn infer_missing_options(options: &mut ScannerOptions) -> Result<(), String> {
    initialize_log_level(options)?;
    initialize_sonar_home(options)?;
    initialize_project_base_directory(options)?;
    initialize_os(options)?;
    initialize_arch(options)?;
    initialize_java_exe_path(options)?;
    initialize_host_url(options)?;
    initialize_key(options)?;
    initialize_name(options);
    initialize_java_binaries(options)?;
    initialize_proxy_port(options)?;
    Ok(())
}

fn canonicalize_property(property_name: &str , path: &str) -> Result<String, String> {
    let path_buf = PathBuf::from(path)
        .canonicalize()
        .map_err(|e| format!("Fail to canonicalize '{property_name}' path '{path}': {e}"))?;
    Ok(path_to_string(&path_buf)?.to_owned())
}

fn path_to_string(path: &Path) -> Result<&str, String> {
    Ok(path.to_str().ok_or_else(|| "Path contains invalid UTF-8".to_string())?)
}

fn initialize_log_level(options: &mut ScannerOptions) -> Result<(), String> {
    if let Some(level) = options.sonar_properties.get(props::LOG_LEVEL) {
        let upper_case_level = level.to_uppercase();
        LogLevel::parse(&upper_case_level)?;
        options.sonar_properties.insert(props::LOG_LEVEL.to_string(), upper_case_level);
    }
    Ok(())
}

fn initialize_sonar_home(options: &mut ScannerOptions) -> Result<(), String> {
    if !options.sonar_properties.contains_key(props::SONAR_HOME) {
        let mut path = dirs::home_dir()
            .ok_or_else(|| "Could not determine the user's home directory.".to_owned())?;
        path.push(".sonar");
        options.sonar_properties.insert(props::SONAR_HOME.to_string(), path_to_string(&path)?.to_owned());
    }
    Ok(())
}

fn initialize_project_base_directory(options: &mut ScannerOptions) -> Result<(), String> {
    let dir = if let Some(base_dir) = options.sonar_properties.get(props::PROJECT_BASE_DIR) {
        PathBuf::from(base_dir)
            .canonicalize()
            .map_err(|e| format!("Fail to canonicalize '{}' directory '{base_dir}': {e}", props::PROJECT_BASE_DIR))?
    } else {
        env::current_dir()
            .map_err(|e| format!("Fail access current directory: {e}"))?
            .canonicalize()
            .map_err(|e| format!("Fail to canonicalize the current directory: {e}"))?
    };
    options.sonar_properties.insert(props::PROJECT_BASE_DIR.to_string(), path_to_string(&dir)?.to_owned());
    Ok(())
}

fn initialize_java_exe_path(options: &mut ScannerOptions) -> Result<(), String> {
    if let Some(path) = options.sonar_properties.get(props::JAVA_EXE_PATH) {
        options.sonar_properties.insert(props::JAVA_EXE_PATH.to_string(), canonicalize_property(props::JAVA_EXE_PATH, path.as_str())?);
    }
    Ok(())
}

fn initialize_java_binaries(options: &mut ScannerOptions) -> Result<(), String> {
    if !options.sonar_properties.contains_key(props::JAVA_BINARIES) {
        let dir = options.sonar_cache()?.join("empty_directory");
        let dir_str = dir
            .to_str()
            .ok_or_else(|| format!("Invalid Unicode in path: {:?}", dir.to_string_lossy()))?;
        if let Err(e) = std::fs::create_dir_all(&dir) {
            return Err(format!("Failed to create directory '{}': {}", dir.display(), e));
        }
        options.sonar_properties.insert(props::JAVA_BINARIES.to_owned(), dir_str.to_owned());
    }
    Ok(())
}

fn initialize_proxy_port(options: &mut ScannerOptions) -> Result<(), String> {
    if options.sonar_properties.contains_key(props::PROXY_HOST) {
        if let Some(port_str) = options.sonar_properties.get(props::PROXY_PORT) {
            let port: u16 =port_str.parse().map_err(|_| format!("Invalid proxy port: {}", port_str))?;
            options.sonar_properties.insert("sonar.scanner.proxyPort".to_string(), port.to_string());
        }
    }
    Ok(())
}

fn initialize_name(options: &mut ScannerOptions) {
    if !options.sonar_properties.contains_key(props::PROJECT_NAME) {
        if let Some(key) = options.sonar_properties.get(props::PROJECT_KEY) {
            options.sonar_properties.insert(props::PROJECT_NAME.to_string(), key.clone());
        }
    }
}

fn is_sonarqube_cloud_eu(url: &str) -> bool {
    Regex::new(r"^(?:https?://)?(?:(?:www|dev\d|dev[1-2]\d)\.)?(?:sonarcloud|sc-staging|sc-dev\d|sc-dev[1-2]\d)\.io(?:/.*)?$")
        .unwrap().is_match(url)
}

fn is_sonarqube_cloud_us(url: &str) -> bool {
    Regex::new(r"^(?:https?://)?(?:(?:www|dev\dus\d)\.)?(?:sonarqube\.us|us-sc-staging\.io|sc-dev\dus\d\.io)(?:/.*)?$")
        .unwrap().is_match(url)
}

fn clean_url(url: &str) -> String {
    let s = url.trim();
    let s = if s.starts_with("http://") || s.starts_with("https://") {
        s.to_string()
    } else {
        format!("https://{s}")
    };
    s.trim_end_matches('/').to_string()
}

fn initialize_host_url(options: &mut ScannerOptions) -> Result<(), String> {
    // Step 1: Clean known URL properties
    for key in [props::HOST_URL, props::SONARCLOUD_URL, props::API_BASE_URL] {
        if let Some(url) = options.sonar_properties.get(key).cloned() {
            options.sonar_properties.insert(key.to_string(), clean_url(&url));
        }
    }

    // Step 2: Validate sonar.region (only 'us' or empty/absent are supported)
    let region_is_set = options.sonar_properties.contains_key(props::REGION);
    let region = options.sonar_properties
        .get(props::REGION)
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if !region.is_empty() && region != "us" {
        eprintln!(
            "ERROR  Unsupported region '{}'. List of supported regions: ['us']. \
             Please check the 'sonar.region' property or the 'SONAR_REGION' environment variable.",
            region
        );
        std::process::exit(1);
    }

    // Step 3: Determine host URL and whether this is SonarQube Cloud
    let host_url = options.sonar_properties.get(props::HOST_URL).cloned();
    let sonarcloud_url = options.sonar_properties.get(props::SONARCLOUD_URL).cloned();

    let is_sonar_cloud = match (&host_url, &sonarcloud_url) {
        (Some(host), Some(cloud)) => {
            // Both URLs are set — they must agree
            if host != cloud {
                eprintln!(
                    "ERROR  The arguments 'sonar.host.url' and 'sonar.scanner.sonarcloudUrl' are \
                     both set and are different. Please set either 'sonar.host.url' for SonarQube \
                     or 'sonar.scanner.sonarcloudUrl' for SonarCloud."
                );
                std::process::exit(1);
            }
            true
        }
        (None, Some(cloud)) => {
            // Only sonarcloudUrl is set — mirror it into host.url
            options.sonar_properties.insert(props::HOST_URL.to_string(), cloud.clone());
            true
        }
        (None, None) => {
            // No URL set — pick default based on region
            let cloud_url = if region == "us" { "https://sonarqube.us" } else { "https://sonarcloud.io" };
            options.sonar_properties.insert(props::HOST_URL.to_string(), cloud_url.to_string());
            options.sonar_properties.insert(props::SONARCLOUD_URL.to_string(), cloud_url.to_string());
            true
        }
        (Some(host), None) => {
            // host.url is set — classify by URL pattern or explicit region
            if is_sonarqube_cloud_us(host) {
                options.sonar_properties.insert(props::SONARCLOUD_URL.to_string(), host.to_string());
                if !region_is_set {
                    options.sonar_properties.insert(props::REGION.to_string(), "us".to_string());
                }
                true
            } else if is_sonarqube_cloud_eu(host) {
                options.sonar_properties.insert(props::SONARCLOUD_URL.to_string(), host.to_string());
                if !region_is_set {
                    options.sonar_properties.insert(props::REGION.to_string(), String::new());
                }
                true
            } else if region_is_set && (region == "us" || region.is_empty()) {
                // Custom SonarCloud URL (staging/dev) indicated by explicit region
                if !options.sonar_properties.contains_key(props::SONARCLOUD_URL) {
                    options.sonar_properties.insert(props::SONARCLOUD_URL.to_string(), host.clone());
                }
                true
            } else {
                // SonarQube Server
                false
            }
        }
    };

    options.sonar_properties.insert(
        props::IS_SONARCLOUD.to_string(),
        is_sonar_cloud.to_string(),
    );

    // Step 4: Set apiBaseUrl if not already present
    if !options.sonar_properties.contains_key(props::API_BASE_URL) {
        let host = options.sonar_properties
            .get(props::HOST_URL)
            .cloned()
            .unwrap_or_default();
        let api_base_url = if is_sonar_cloud {
            let re = Regex::new(r"://(?:[^./]+\.)?([^./]+\.[^./]+(?:/.*|$))").unwrap();
            re.replace(&host, "://api.$1").to_string()
        } else {
            format!("{host}/api/v2")
        };
        options.sonar_properties.insert(props::API_BASE_URL.to_string(), api_base_url);
    }

    Ok(())
}

fn initialize_key(options: &mut ScannerOptions) -> Result<(), String> {
    if !options.sonar_properties.contains_key(props::PROJECT_KEY) {
        let key = if let Some(name) = git_repository_name(&options) {
            name
        } else {
            project_base_directory_name(&options)?
        };
        options.sonar_properties.insert(props::PROJECT_KEY.to_owned(), key.to_owned());
    }
    Ok(())
}

fn project_base_directory_name(options: &ScannerOptions) -> Result<String, String> {
    let base_dir = options.project_base_directory()?;
    let key = base_dir
        .file_name()
        .ok_or_else(|| format!("'file_name()' is empty for '{}'", props::PROJECT_BASE_DIR))?
        .to_str()
        .ok_or_else(|| "Path contains invalid UTF-8".to_string())?;
    Ok(key.to_owned())
}

fn git_repository_name(options: &ScannerOptions) -> Option<String> {
    let project_base_directory: &PathBuf = &options.project_base_directory().ok()?;
    let git_config = project_base_directory.join(".git/config");
    if let Ok(content) = std::fs::read_to_string(git_config) {
        let mut in_origin = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == r#"[remote "origin"]"# {
                in_origin = true;
            } else if trimmed.starts_with('[') {
                in_origin = false;
            } else if in_origin {
                if let Some(rest) = trimmed.strip_prefix("url") {
                    if let Some(rest) = rest.trim_start().strip_prefix('=') {
                        let url = rest.trim();
                        if let Some(name) = url.rsplit('/').next() {
                            let name = name.trim_end_matches(".git");
                            if !name.is_empty() {
                                return Some(name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn initialize_os(options: &mut ScannerOptions) -> Result<(), String> {
    let validated_os = if let Some(os) = options.sonar_properties.get(props::OS) {
        match os.to_lowercase().as_str() {
            "linux" | "gnu/linux" | "unix" => "linux",
            "alpine" | "alpinelinux" | "alpine-linux" => "alpine",
            "macos" | "mac" | "macosx" | "darwin" | "osx" => "macos",
            "windows" | "win" | "win32" | "win64" => "windows",
            s if s.starts_with("mingw") || s.starts_with("cygwin") || s.starts_with("msys") => {
                "windows"
            }
            _ => os,
        }
    } else {
        default_os()?
    };
    options.sonar_properties.insert(props::OS.to_owned(), validated_os.to_owned());
    Ok(())
}

pub fn default_os() -> Result<&'static str, String> {
    match env::consts::OS {
        "linux" => {
            let is_alpine = std::fs::read_to_string("/etc/os-release")
                .map(|content| content.lines().any(|line| line == "ID=alpine"))
                .unwrap_or(false);
            Ok(if is_alpine { "alpine" } else { "linux" })
        }
        "macos" => Ok("macos"),
        "windows" => Ok("windows"),
        os => Err(format!("unsupported operating system: {os}")),
    }
}

fn initialize_arch(options: &mut ScannerOptions) -> Result<(), String> {
    let validated_arch = if let Some(arch) = options.sonar_properties.get(props::ARCH) {
        match arch.to_lowercase().as_str() {
            "x64" | "x86_64" | "x86-64" | "amd64" => "x64",
            "aarch64" | "arm64" => "aarch64",
            _ => arch,
        }
    } else {
        default_arch()?
    };
    options.sonar_properties.insert(props::ARCH.to_owned(), validated_arch.to_owned());
    Ok(())
}

pub fn default_arch() -> Result<&'static str, String> {
    // Unsupported: m68k, mips, mips32r6, mips64, mips64r6, csky, powerpc, powerpc64, riscv32,
    //              riscv64, s390x, sparc, sparc64, hexagon, loongarch32, loongarch64
    match env::consts::ARCH {
        "x86_64" | "x86" => Ok("x64"),
        "aarch64" | "arm" => Ok("aarch64"),
        arch => Err(format!("unsupported architecture: {arch}")),
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn default_os_returns_known_value() {
        let os = default_os().expect("current OS should be supported");
        assert!(["linux", "alpine", "macos", "windows"].contains(&os));
    }

    #[test]
    fn default_arch_returns_known_value() {
        let arch = default_arch().expect("current arch should be supported");
        assert!(["x64", "aarch64"].contains(&arch));
    }

    use super::*;
    #[test]
    fn resolve_sonarqube_or_sonarcloud() {

        let dest_name = |url: &str| -> &'static str {
            if is_sonarqube_cloud_us(url) {
                if is_sonarqube_cloud_eu(url) {
                    "SQC eu + us !!!"
                } else {
                    "SQC us"
                }
            } else if is_sonarqube_cloud_eu(url) {
                "SQC eu"
            } else {
                "SQS"
            }
        };

        // SonarQube Cloud EU
        assert_eq!(dest_name("sonarcloud.io"), "SQC eu");
        assert_eq!(dest_name("https://sonarcloud.io/"), "SQC eu");
        assert_eq!(dest_name("https://sonarcloud.io"), "SQC eu");
        assert_eq!(dest_name("https://www.sonarcloud.io"), "SQC eu");
        assert_eq!(dest_name("http://www.sonarcloud.io"), "SQC eu");
        assert_eq!(dest_name("https://dev1.sc-dev1.io"), "SQC eu");
        assert_eq!(dest_name("https://dev2.sc-dev2.io"), "SQC eu");
        assert_eq!(dest_name("https://dev3.sc-dev3.io"), "SQC eu");
        assert_eq!(dest_name("https://dev4.sc-dev4.io"), "SQC eu");
        assert_eq!(dest_name("https://dev5.sc-dev5.io"), "SQC eu");
        assert_eq!(dest_name("https://dev6.sc-dev6.io"), "SQC eu");
        assert_eq!(dest_name("https://dev7.sc-dev7.io"), "SQC eu");
        assert_eq!(dest_name("https://dev8.sc-dev8.io"), "SQC eu");
        assert_eq!(dest_name("https://dev9.sc-dev9.io"), "SQC eu");
        assert_eq!(dest_name("https://dev10.sc-dev10.io"), "SQC eu");
        assert_eq!(dest_name("https://dev11.sc-dev11.io"), "SQC eu");
        assert_eq!(dest_name("https://dev12.sc-dev12.io"), "SQC eu");
        assert_eq!(dest_name("https://dev13.sc-dev13.io"), "SQC eu");
        assert_eq!(dest_name("https://dev14.sc-dev14.io"), "SQC eu");
        assert_eq!(dest_name("https://dev15.sc-dev15.io"), "SQC eu");
        assert_eq!(dest_name("https://dev16.sc-dev16.io"), "SQC eu");
        assert_eq!(dest_name("https://dev17.sc-dev17.io"), "SQC eu");
        assert_eq!(dest_name("https://dev18.sc-dev18.io"), "SQC eu");
        assert_eq!(dest_name("https://dev19.sc-dev19.io"), "SQC eu");
        assert_eq!(dest_name("https://dev20.sc-dev20.io"), "SQC eu");
        assert_eq!(dest_name("https://sc-staging.io"), "SQC eu");

        // SonarQube Cloud US
        assert_eq!(dest_name("sonarqube.us"), "SQC us");
        assert_eq!(dest_name("https://sonarqube.us/"), "SQC us");
        assert_eq!(dest_name("https://sonarqube.us"), "SQC us");
        assert_eq!(dest_name("http://sonarqube.us"), "SQC us");
        assert_eq!(dest_name("https://www.sonarqube.us"), "SQC us");
        assert_eq!(dest_name("https://dev1us1.sc-dev1us1.io"), "SQC us");
        assert_eq!(dest_name("https://dev2us1.sc-dev2us1.io"), "SQC us");
        assert_eq!(dest_name("https://us-sc-staging.io"), "SQC us");

        // SonarQube Server
        assert_eq!(dest_name("https://sonarqube.example.com/"), "SQS");
        assert_eq!(dest_name("http://sonarqube.example.com"), "SQS");
        assert_eq!(dest_name("sonarqube.example.com"), "SQS");
        assert_eq!(dest_name("https://localhost:9000/"), "SQS");
    }
}