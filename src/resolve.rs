use regex::Regex;
use crate::options::ScannerOptions;

pub fn infer_missing_options(mut options: ScannerOptions) -> ScannerOptions {
    options = resolve_host_url(options);
    options = resolve_key(options);
    options = resolve_name(options);
    options = resolve_java_binaries(options);
    options
}

fn resolve_java_binaries(mut options: ScannerOptions) -> ScannerOptions {
    if !options.scanner_properties.contains_key("sonar.java.binaries") {
        let dir = options.sonar_cache.join("empty_directory");
        if std::fs::create_dir_all(&dir).is_ok() {
            options.scanner_properties.insert(
                "sonar.java.binaries".to_string(),
                dir.to_string_lossy().into_owned(),
            );
        }
    }
    options
}

fn resolve_name(mut options: ScannerOptions) -> ScannerOptions {
    if options.name.is_none() {
        if let Some(key) = &options.key {
            options.name = Some(key.clone());
            options
                .scanner_properties
                .insert("sonar.projectName".to_string(), key.clone());
        }
    }
    options
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

fn resolve_host_url(mut options: ScannerOptions) -> ScannerOptions {
    // Step 1: Clean known URL properties
    for key in ["sonar.host.url", "sonar.scanner.sonarcloudUrl", "sonar.scanner.apiBaseUrl"] {
        if let Some(url) = options.scanner_properties.get(key).cloned() {
            options.scanner_properties.insert(key.to_string(), clean_url(&url));
        }
    }

    // Step 2: Validate sonar.region (only 'us' or empty/absent are supported)
    let region_is_set = options.scanner_properties.contains_key("sonar.region");
    let region = options.scanner_properties
        .get("sonar.region")
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
    let host_url = options.scanner_properties.get("sonar.host.url").cloned();
    let sonarcloud_url = options.scanner_properties.get("sonar.scanner.sonarcloudUrl").cloned();

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
            options.scanner_properties.insert("sonar.host.url".to_string(), cloud.clone());
            true
        }
        (None, None) => {
            // No URL set — pick default based on region
            let (host, cloud_url) = if region == "us" {
                ("https://sonarqube.us", "https://sonarqube.us")
            } else {
                ("https://sonarcloud.io", "https://sonarcloud.io")
            };
            options.scanner_properties.insert("sonar.host.url".to_string(), host.to_string());
            options.scanner_properties.insert("sonar.scanner.sonarcloudUrl".to_string(), cloud_url.to_string());
            true
        }
        (Some(host), None) => {
            // host.url is set — classify by URL pattern or explicit region
            if is_sonarqube_cloud_us(host) {
                if !region_is_set {
                    options.scanner_properties.insert("sonar.region".to_string(), "us".to_string());
                }
                true
            } else if is_sonarqube_cloud_eu(host) {
                if !region_is_set {
                    options.scanner_properties.insert("sonar.region".to_string(), String::new());
                }
                true
            } else if region_is_set && (region == "us" || region.is_empty()) {
                // Custom SonarCloud URL (staging/dev) indicated by explicit region
                if !options.scanner_properties.contains_key("sonar.scanner.sonarcloudUrl") {
                    options.scanner_properties.insert("sonar.scanner.sonarcloudUrl".to_string(), host.clone());
                }
                true
            } else {
                // SonarQube Server
                false
            }
        }
    };

    options.scanner_properties.insert(
        "sonar.scanner.internal.isSonarCloud".to_string(),
        is_sonar_cloud.to_string(),
    );

    // Step 4: Set apiBaseUrl if not already present
    if !options.scanner_properties.contains_key("sonar.scanner.apiBaseUrl") {
        let host = options.scanner_properties
            .get("sonar.host.url")
            .cloned()
            .unwrap_or_default();
        let api_base_url = if is_sonar_cloud {
            host
        } else {
            format!("{host}/api/v2")
        };
        options.scanner_properties.insert("sonar.scanner.apiBaseUrl".to_string(), api_base_url);
    }

    options
}

fn resolve_key(mut options: ScannerOptions) -> ScannerOptions {
    if options.key.is_none() {
        options = resolve_key_from_git_repository_name(options);
        if options.key.is_none() {
            options = resolve_key_from_directory_name(options);
        }
    }
    options
}

fn resolve_key_from_directory_name(mut options: ScannerOptions) -> ScannerOptions {
    options.key = options
        .dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned());
    if let Some(key) = &options.key {
        options
            .scanner_properties
            .insert("sonar.projectKey".to_string(), key.clone());
    }
    options
}

fn resolve_key_from_git_repository_name(mut options: ScannerOptions) -> ScannerOptions {
    let git_config = options.dir.join(".git/config");
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
                                options.key = Some(name.to_string());
                                options.scanner_properties.insert(
                                    "sonar.projectKey".to_string(),
                                    name.to_string(),
                                );
                                return options;
                            }
                        }
                    }
                }
            }
        }
    }
    options
}


#[cfg(test)]
mod tests {
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