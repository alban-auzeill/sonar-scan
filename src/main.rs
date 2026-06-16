mod options;
mod resolve;
mod sonar_scanner_cli;

use options::{ScannerOptions, HELP, parse_options};
use resolve::infer_missing_options;
use sonar_scanner_cli::{download_jre_extract_scanner, download_scanner};
#[cfg(not(test))]
use chrono::{Local};
#[cfg(test)]
use chrono::{Local, TimeZone};
use std::env;
use std::io::{self, Write};
use std::process::{self, Command, Stdio};
use crate::options::LogLevel;
use crate::sonar_scanner_cli::SONAR_SCANNER_CLI_JAR_VERSION;

fn log(out: &mut impl Write, message: &str) {
    #[cfg(not(test))]
    let now = Local::now();
    #[cfg(test)]
    let now = Local.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap();

    let timestamp = now.format("%H:%M:%S%.3f").to_string();
    writeln!(out, "{timestamp} {}", message).ok();
}

fn scan_project(options: &ScannerOptions, out: &mut impl Write) -> Result<i32, String> {
    log(out, &format!("INFO  sonar-scan {}", env!("CARGO_PKG_VERSION")));
    log(out, &format!("INFO  Project: {}", options.dir.display()));

    let d_params: Vec<String> = options
        .scanner_properties
        .iter()
        .map(|(k, v)| format!("-D{k}={v}"))
        .collect();

    let status = if let Some(scanner_version) = &options.scanner_version {
        log(out, &format!("INFO  Using SonarScanner CLI: {}", &scanner_version));
        let sonar_scanner = download_scanner(options, out)?;
        if options.log_level == LogLevel::DEBUG || options.log_level == LogLevel::TRACE {
            log(out, &format!("DEBUG  Scanner : {}", sonar_scanner.display()));
        }
        Command::new(&sonar_scanner)
            .args(&d_params)
            .current_dir(&options.dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit())
            .spawn()
            .map_err(|e| format!("Failed to start SonarScanner: {e}"))?
            .wait()
            .map_err(|e| format!("Failed to wait for SonarScanner process: {e}"))?
    } else {
        log(out, &format!("INFO  Using embedded SonarScanner CLI: {}", SONAR_SCANNER_CLI_JAR_VERSION.trim()));
        let paths = download_jre_extract_scanner(options, out)?;
        if options.log_level == LogLevel::DEBUG || options.log_level == LogLevel::TRACE {
            log(out, &format!("DEBUG  JAVA_HOME : {}", paths.java_home.display()));
            log(out, &format!("DEBUG  java      : {}", paths.java_exe.display()));
            log(out, &format!("DEBUG  jar       : {}", paths.sonar_scanner_jar.display()));
        }
        let project_home = options.dir.to_string_lossy();
        let mut cmd = Command::new(&paths.java_exe);
        cmd
            .arg("-Djava.awt.headless=true")
            .arg("-Djdk.http.auth.tunneling.disabledSchemes=")
            .arg("-classpath")
            .arg(&paths.sonar_scanner_jar)
            .args(&d_params)
            .arg(format!("-Dproject.home={project_home}"))
            .arg("org.sonarsource.scanner.cli.Main")
            .current_dir(&options.dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit());
        if env::var_os("JAVA_HOME").is_none() {
            cmd.env("JAVA_HOME", &paths.java_home);
        }
        cmd.spawn()
            .map_err(|e| format!("Failed to start Java: {e}"))?
            .wait()
            .map_err(|e| format!("Failed to wait for Java process: {e}"))?
    };

    Ok(status.code().unwrap_or(1))
}

fn scan(
    args: impl Iterator<Item = String>,
    env_fn: &dyn Fn(&str) -> Option<String>,
    out: &mut impl Write,
    err: &mut impl Write,
) -> i32 {
    let args: Vec<String> = args.skip(1).collect();

    if args.len() == 1 && args[0] == "--version" {
        writeln!(out, "{}", env!("CARGO_PKG_VERSION")).ok();
        return 0;
    }
    if args.len() == 1 && args[0] == "--help" {
        writeln!(out, "{HELP}").ok();
        return 0;
    }

    let options = match parse_options(&args, env_fn) {
        Ok(o) => o,
        Err(msg) => {
            log(err, &format!("ERROR  {msg}"));
            return 1;
        }
    };

    let options = infer_missing_options(options);

    if options.dump {
        writeln!(out, "{}", options.to_json()).ok();
        return 0;
    }

    if options.token.is_none() {
        log(err, "ERROR  Missing required option: --token or SONAR_TOKEN environment variable");
        return 1;
    }

    match scan_project(&options, out) {
        Ok(code) => code,
        Err(msg) => {
            log(err, &format!("ERROR  {msg}"));
            1
        }
    }
}

fn main() {
    process::exit(scan(
        env::args(),
        &|k| env::var(k).ok(),
        &mut io::stdout(),
        &mut io::stderr(),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn run(args: &[&str]) -> (i32, String, String) {
        let mut out = Vec::new();
        let mut err = Vec::new();
        // Pass HOME/USERPROFILE so sonar_home can be resolved; block all other env vars
        // to keep tests isolated from the caller's environment.
        let code = scan(
            args.iter().map(|s| s.to_string()),
            &|k| match k {
                "HOME" | "USERPROFILE" => env::var(k).ok(),
                _ => None,
            },
            &mut out,
            &mut err,
        );
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    #[test]
    fn version_flag_prints_version_and_exits_0() {
        let (code, out, err) = run(&["sonar-scan", "--version"]);
        assert_eq!(code, 0);
        assert_eq!(out.trim_end(), env!("CARGO_PKG_VERSION"));
        assert_eq!(err.trim_end(), "");
    }

    #[test]
    fn help_flag_prints_help_and_exits_0() {
        let (code, out, err) = run(&["sonar-scan", "--help"]);
        assert_eq!(code, 0);
        assert!(out.contains("sonar-scan"));
        assert!(out.contains("--dir"));
        assert!(out.contains("--token"));
        assert_eq!(err.trim_end(), "");
    }

    #[test]
    fn without_argument_prints_current_dir_and_project_name() {
        let (code, out, err) = run(&["sonar-scan"]);
        assert_eq!(code, 1);
        assert_eq!(err.trim_end(), "12:00:00.000 ERROR  Missing required option: --token or SONAR_TOKEN environment variable");
        assert_eq!(out.trim_end(),"");
    }

    #[test]
    fn with_nonexistent_dir_prints_error_and_exits_1() {
        let (code, out, err) = run(&["sonar-scan", "--dir=/nonexistent/path"]);
        assert_eq!(code, 1);
        assert_eq!(err.trim_end(), "12:00:00.000 ERROR  Project path does not exist: /nonexistent/path");
        assert_eq!(out.trim_end(), "");
    }

    #[test]
    fn unexpected_positional_argument_returns_error() {
        let (code, _out, err) = run(&["sonar-scan", "some-path"]);
        assert_eq!(code, 1);
        assert!(err.contains("unexpected argument"));
    }
}

#[cfg(test)]
#[cfg(feature = "integration-tests")]
mod integration_tests {
    use std::env;
    use regex::Regex;

    #[test]
    fn with_dir_argument_prints_path_and_folder_name() {
        // Use a dir without .git so project_name falls back to the folder name
        let tmp = env::temp_dir()
            .canonicalize()
            .unwrap()
            .join("sonar-scan-test-dir");
        std::fs::create_dir_all(&tmp).unwrap();
        let (_code, out, _err) = crate::tests::run(&[
            "sonar-scan",
            "-Dsonar.scanner.internal.dumpToFile=scan.properties",
            "--token=123",
            "--scanner-version=8.0.1.6346",
            &format!("--dir={}", tmp.display())]);
        let content = std::fs::read_to_string(tmp.join("scan.properties"))
            .expect("scan.properties should have been created by the scanner");

        std::fs::remove_dir(&tmp).ok();
        assert!(out.contains("12:00:00.000 INFO  sonar-scan"));
        assert!(out.contains("12:00:00.000 INFO  Using SonarScanner CLI: 8.0.1.6346"));
        assert!(
            Regex::new(r"(?m)^12:00:00\.000 INFO  Project: .*sonar-scan-test-dir\r?$").unwrap().is_match(&out),
            "'Project: .*sonar-scan-test-dir' not found in:\n {content}"
        );
        assert!(
            Regex::new(r"(?m)^sonar\.token=123\r?$").unwrap().is_match(&content),
            "'sonar.token=123' not found in:\n {content}"
        );
        assert!(
            Regex::new(r"(?m)^sonar\.projectBaseDir=.*sonar-scan-test-dir\r?$").unwrap().is_match(&content),
            "'sonar.projectBaseDir=.*sonar-scan-test-dir' not found in:\n {content}"
        );
        assert!(
            Regex::new(r"(?m)^sonar\.projectKey=sonar-scan-test-dir\r?$").unwrap().is_match(&content),
            "'sonar.projectKey=sonar-scan-test-dir' not found in:\n {content}"
        );
        assert!(
            Regex::new(r"(?m)^sonar\.projectName=sonar-scan-test-dir\r?$").unwrap().is_match(&content),
            "'sonar.projectName=sonar-scan-test-dir' not found in:\n {content}"
        );

        assert!(
            Regex::new(r"(?m)^sonar\.java\.binaries=.*empty_directory\r?$").unwrap().is_match(&content),
            "'sonar.java.binaries=.*empty_directory' not found in:\n {content}"
        );
        assert!(
            Regex::new(r"(?m)^java\.class\.version=65\.0\r?$").unwrap().is_match(&content),
            "Invalid 'java.class.version=65.0' not found in:\n {content}"
        );
    }
}