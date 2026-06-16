use crate::options::ScannerOptions;

pub fn infer_missing_options(mut options: ScannerOptions) -> ScannerOptions {
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
