use std::env;
use regex::Regex;
use crate::options::{default_os, ScannerOptions};

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

fn resolve_host_url(mut options: ScannerOptions) -> ScannerOptions {

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
    use indoc::indoc;

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