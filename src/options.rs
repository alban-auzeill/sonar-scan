use crate::resolve::infer_missing_options;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

// Options not parsed but computed
pub const IS_SONARCLOUD: &'static str = "sonar.scanner.internal.isSonarCloud";

pub const OPTIONS: &'static [&OptDesc] = &[
    TOKEN,
    PROJECT_BASE_DIR,
    HOST_URL,
    SONARCLOUD_URL,
    API_BASE_URL,
    ORGANIZATION,
    REGION,
    PROJECT_KEY,
    PROJECT_NAME,
    PROJECT_VERSION,
    PROJECT_DESCRIPTION,
    BUILD_NUMBER,
    SOURCES,
    TESTS,
    JAVA_BINARIES,
    JAVA_LIBRARIES,
    JAVA_TEST_BINARIES,
    JAVA_TEST_LIBRARIES,
    OS,
    ARCH,
    SKIP_JRE_PROVISIONING,
    JAVA_EXE_PATH,
    PROXY_HOST,
    PROXY_PORT,
    PROXY_USER,
    PROXY_PASSWORD,
    SONAR_HOME,
    JAVA_OPTS,
    WORKING_DIR,
    VERBOSE,
    LOG_LEVEL,
    TRUSTSTORE_PATH,
    TRUSTSTORE_PASSWORD,
    KEYSTORE_PATH,
    KEYSTORE_PASSWORD,
    SOURCE_ENCODING,
    CLI_VERSION,
    DUMP_PROPERTIES,
];

pub const TOKEN: &'static OptDesc = &OptDesc {
    option: "token",
    bool_flag: false,
    env: "SONAR_TOKEN",
    property: "sonar.token",
    usage: r##"
    --token=<TOKEN>               Token used by the scanner to authenticate to your SonarQube
                                  instance. This property has no default value and is mandatory.
"##,
};

pub const PROJECT_BASE_DIR: &'static OptDesc = &OptDesc {
    option: "dir",
    bool_flag: false,
    env: "SONAR_BASE_DIR",
    property: "sonar.projectBaseDir",
    usage: r##"
    --dir=<PATH>                  The project's base directory. Use this property when you need the
                                  analysis to take place in a directory other than the one from
                                  which it was started.
                   Default value: <current directory>
"##,
};

pub const HOST_URL: &'static OptDesc = &OptDesc {
    option: "url",
    bool_flag: false,
    env: "SONAR_HOST_URL",
    property: "sonar.host.url",
    usage: r##"
    --url=<URL>                   The URL to your SonarQube instance.
                   Default value: https://sonarcloud.io
"##,
};

pub const SONARCLOUD_URL: &'static OptDesc = &OptDesc {
    option: "sonarcloud-url",
    bool_flag: false,
    env: "SONAR_CLOUD_URL",
    property: "sonar.scanner.sonarcloudUrl",
    usage: r##"
    --sonarcloud-url=<URL>        The URL to your SonarQube Cloud instance.
                                  (alias for 'sonar.host.url' in the context of SonarQube Cloud)
                   Default value: https://sonarcloud.io
"##,
};

pub const API_BASE_URL: &'static OptDesc = &OptDesc {
    option: "api-url",
    bool_flag: false,
    env: "SONAR_API_BASE_URL",
    property: "sonar.scanner.apiBaseUrl",
    usage: r##"
    --api-url=<URL>               The URL to the web API of your SonarQube Cloud instance.
                   Default value: <automatically computed based on your region>
"##,
};

pub const ORGANIZATION: &'static OptDesc = &OptDesc {
    option: "org",
    bool_flag: false,
    env: "SONAR_ORGANIZATION",
    property: "sonar.organization",
    usage: r##"
    --org=<URL>                   (Only for SonarQube Cloud) The project organization.
                   Default value: <Organization of the given token if there is only one>
"##,
};

pub const REGION: &'static OptDesc = &OptDesc {
    option: "region",
    bool_flag: false,
    env: "SONAR_REGION",
    property: "sonar.region",
    usage: r##"
    --region=<URL>                (Only for SonarQube Cloud) The SonarQube Cloud region:
                                  Possible values: 'us', missing or emtpy.
                                  * 'us' region set 'sonar.host.url' to 'https://sonarqube.us'
                                  * missing or emtpy set 'sonar.host.url' to 'https://sonarcloud.io'
                   Default value: <emtpy>
"##,
};

pub const PROJECT_KEY: &'static OptDesc = &OptDesc {
    option: "key",
    bool_flag: false,
    env: "SONAR_PROJECT_KEY",
    property: "sonar.projectKey",
    usage: r##"
    --key=<KEY>                   The project's unique key. Can include up to 400 characters. All
                                  letters, digits, dashes, underscores, periods, and colons are
                                  accepted.
                   Default value: <git repository name or base directory name>
"##,
};

pub const PROJECT_NAME: &'static OptDesc = &OptDesc {
    option: "name",
    bool_flag: false,
    env: "SONAR_PROJECT_NAME",
    property: "sonar.projectName",
    usage: r##"
    --name=<NAME>                 Name of the project that will be displayed on the web interface.
                   Default value: <value of sonar.projectKey>
"##,
};

pub const PROJECT_VERSION: &'static OptDesc = &OptDesc {
    option: "version",
    bool_flag: false,
    env: "SONAR_PROJECT_VERSION",
    property: "sonar.projectVersion",
    usage: r##"
    --version=<VERSION>           The project version. It should be set for branch analysis in case
                                  you use the new code definition based on the previous version.
                   Default value: <empty>
"##,
};

pub const PROJECT_DESCRIPTION: &'static OptDesc = &OptDesc {
    option: "description",
    bool_flag: false,
    env: "SONAR_PROJECT_DESCRIPTION",
    property: "sonar.projectDescription",
    usage: r##"
    --description=<TEXT>          The project description.
                   Default value: <empty>
"##,
};

pub const BUILD_NUMBER: &'static OptDesc = &OptDesc {
    option: "build-number",
    bool_flag: false,
    env: "SONAR_PROJECT_BUILD_NUMBER",
    property: "sonar.analysis.buildNumber",
    usage: r##"
    --build-number=<NUMBER>       The project build number.
                   Default value: <empty>
"##,
};

pub const SOURCES: &'static OptDesc = &OptDesc {
    option: "sources",
    bool_flag: false,
    env: "SONAR_PROJECT_SOURCES",
    property: "sonar.sources",
    usage: r##"
    --sources=<PATHS>             The initial analysis scope for main source code (non-test code) in
                                  the project. Comma-separated paths to directories are included. An
                                  individual file in the list means that the file is included. A
                                  directory in the list means that all analyzable files and
                                  directories recursively below it are included. The path can be
                                  relative (to the sonar.projectBaseDir property) or absolute.
                                  Wildcards (*, ** and ?) are not allowed.
                   Default value: <value of sonar.projectBaseDir>
"##,
};

pub const TESTS: &'static OptDesc = &OptDesc {
    option: "tests",
    bool_flag: false,
    env: "SONAR_PROJECT_TESTS",
    property: "sonar.tests",
    usage: r##"
    --tests=<PATHS>               The initial analysis scope for test code in the project.
                                  Same format as '--sources'.
                   Default value: <empty>
"##,
};

pub const JAVA_BINARIES: &'static OptDesc = &OptDesc {
    option: "java-binaries",
    bool_flag: false,
    env: "SONAR_JAVA_BINARIES",
    property: "sonar.java.binaries",
    usage: r##"
    ----java-binaries=<PATHS>     Comma-separated paths to directories containing the compiled
                                  bytecode files corresponding to your source files.
                   Default value: <empty>
"##,
};

pub const JAVA_LIBRARIES: &'static OptDesc = &OptDesc {
    option: "java-libraries",
    bool_flag: false,
    env: "SONAR_JAVA_LIBRARIES",
    property: "sonar.java.libraries",
    usage: r##"
    ----java-libraries=<PATHS>    Comma-separated paths to files with third-party libraries (JAR or
                                  Zip files) used by your project.
                                  Wildcards can be used:
                                  sonar.java.libraries=path/to/Library.jar,directory/**/*.jar
                   Default value: <empty>
"##,
};

pub const JAVA_TEST_BINARIES: &'static OptDesc = &OptDesc {
    option: "java-test-binaries",
    bool_flag: false,
    env: "SONAR_JAVA_TEST_BINARIES",
    property: "sonar.java.test.binaries",
    usage: r##"
    --java-test-binaries=<PATHS>  Comma-separated paths to directories containing the compiled
                                  bytecode files corresponding to your test files.
                   Default value: <empty>
"##,
};

pub const JAVA_TEST_LIBRARIES: &'static OptDesc = &OptDesc {
    option: "java-test-libraries",
    bool_flag: false,
    env: "SONAR_JAVA_TEST_LIBRARIES",
    property: "sonar.java.test.libraries",
    usage: r##"
    --java-test-libraries=<PATHS> Comma-separated paths to files with third-party libraries (JAR or
                                  Zip files) used by your tests. (For example, this should include
                                  the junit jar). Wildcards can be used:
                                  sonar.java.test.libraries=directory/**/*.jar
                   Default value: <empty>
"##,
};

pub const OS: &'static OptDesc = &OptDesc {
    option: "os",
    bool_flag: false,
    env: "SONAR_SCANNER_OS",
    property: "sonar.scanner.os",
    usage: r##"
    --os=<OS>                     The operating system of the machine hosting the SonarScanner.
                                  Possible values: windows, linux, macos (or Darwin), alpine
                   Default value: <autodetected value>
"##,
};

pub const ARCH: &'static OptDesc = &OptDesc {
    option: "arch",
    bool_flag: false,
    env: "SONAR_SCANNER_ARCH",
    property: "sonar.scanner.arch",
    usage: r##"
    --arch=<ARCH>                 The CPU architecture type.
                                  Possible values: x64 (or x86_64, amd64), aarch64 (or arm64)
                   Default value: <autodetected value>
"##,
};

pub const SKIP_JRE_PROVISIONING: &'static OptDesc = &OptDesc {
    option: "skip-jre-provisioning",
    bool_flag: true,
    env: "SONAR_SCANNER_SKIP_JRE_PROVISIONING",
    property: "sonar.scanner.skipJreProvisioning",
    usage: r##"
    --skip-jre-provisioning       Skip the Java Runtime Environment (JRE) download from the
                                  SonarQube instance.
                   Default value: false
"##,
};

pub const JAVA_EXE_PATH: &'static OptDesc = &OptDesc {
    option: "java-exe-path",
    bool_flag: false,
    env: "SONAR_SCANNER_JAVA_EXE_PATH",
    property: "sonar.scanner.javaExePath",
    usage: r##"
    --java-exe-path=<PATH>        If defined, the SonarScanner runs with this Java executable.
                   Default value: <bin/java of the provisioned or autodetected JRE>
"##,
};

pub const PROXY_HOST: &'static OptDesc = &OptDesc {
    option: "proxy-host",
    bool_flag: false,
    env: "SONAR_SCANNER_PROXY_HOST",
    property: "sonar.scanner.proxyHost",
    usage: r##"
    --proxy-host=<HOST>           The host name of the proxy server.
                   Default value: <empty>
"##,
};

pub const PROXY_PORT: &'static OptDesc = &OptDesc {
    option: "proxy-port",
    bool_flag: false,
    env: "SONAR_SCANNER_PROXY_PORT",
    property: "sonar.scanner.proxyPort",
    usage: r##"
    --proxy-port=<PORT>           The port of the proxy server.
                   Default value: <same port as sonar.host.url>
"##,
};

pub const PROXY_USER: &'static OptDesc = &OptDesc {
    option: "proxy-user",
    bool_flag: false,
    env: "SONAR_SCANNER_PROXY_USER",
    property: "sonar.scanner.proxyUser",
    usage: r##"
    --proxy-user=<USER>           In case of an authenticated proxy: the user name.
                   Default value: <empty>
"##,
};

pub const PROXY_PASSWORD: &'static OptDesc = &OptDesc {
    option: "proxy-password",
    bool_flag: false,
    env: "SONAR_SCANNER_PROXY_PASSWORD",
    property: "sonar.scanner.proxyPassword",
    usage: r##"
    --proxy-password=<PASSWORD>   In case of an authenticated proxy: the user password.
                   Default value: <empty>
"##,
};

pub const SONAR_HOME: &'static OptDesc = &OptDesc {
    option: "sonar-home",
    bool_flag: false,
    env: "SONAR_USER_HOME",
    property: "sonar.userHome",
    usage: r##"
    --sonar-home=<PATH>           The directory for the scanner cache.
                   Default value: ${HOME}/.sonar
"##,
};

pub const JAVA_OPTS: &'static OptDesc = &OptDesc {
    option: "java-opts",
    bool_flag: false,
    env: "SONAR_SCANNER_JAVA_OPTS",
    property: "sonar.scanner.javaOpts",
    usage: r##"
    --java-opts=<OPTS>            Arguments to pass to the JVM running the scanner engine.
                   Default value: <empty>
"##,
};

pub const WORKING_DIR: &'static OptDesc = &OptDesc {
    option: "work-dir",
    bool_flag: false,
    env: "SONAR_SCANNER_WORK_DIR",
    property: "sonar.working.directory",
    usage: r##"
    --work-dir=<DIR>              Path to the working directory used by the SonarScanner during a
                                  project analysis to store temporary data. The path can be relative
                                  (to the sonar.projectBaseDir property) or absolute.
                   Default value: .scannerwork
"##,
};

pub const VERBOSE: &'static OptDesc = &OptDesc {
    option: "verbose",
    bool_flag: true,
    env: "SONAR_VERBOSE",
    property: "sonar.verbose",
    usage: r##"
    --verbose                     When present, adds more details to the analysis logs by activating
                                  the DEBUG mode for the scanner.
                   Default value: false
"##,
};

pub const LOG_LEVEL: &'static OptDesc = &OptDesc {
    option: "log",
    bool_flag: false,
    env: "SONAR_LOG_LEVEL",
    property: "sonar.log.level",
    usage: r##"
    --log=<LEVEL>                 The log level.
                                  Possible values: INFO, DEBUG, TRACE
                   Default value: <if sonar.verbose then DEBUG else INFO>
"##,
};

pub const TRUSTSTORE_PATH: &'static OptDesc = &OptDesc {
    option: "truststore-path",
    bool_flag: false,
    env: "SONAR_SCANNER_TRUSTSTORE_PATH",
    property: "sonar.scanner.truststorePath",
    usage: r##"
    --truststore-path=<PATH>      The path to the truststore file.
                   Default value: <sonar.userHome>/ssl/truststore.p12
"##,
};

pub const TRUSTSTORE_PASSWORD: &'static OptDesc = &OptDesc {
    option: "truststore-password",
    bool_flag: false,
    env: "SONAR_SCANNER_TRUSTSTORE_PASSWORD",
    property: "sonar.scanner.truststorePassword",
    usage: r##"
    --truststore-password=<PASS>  The password of the truststore.
                   Default value: changeit
"##,
};

pub const KEYSTORE_PATH: &'static OptDesc = &OptDesc {
    option: "keystore-path",
    bool_flag: false,
    env: "SONAR_SCANNER_KEYSTORE_PATH",
    property: "sonar.scanner.keystorePath",
    usage: r##"
    --keystore-path=<PATH>        The path to the keystore file.
                   Default value: <sonar.userHome>/ssl/keystore.p12
"##,
};

pub const KEYSTORE_PASSWORD: &'static OptDesc = &OptDesc {
    option: "keystore-password",
    bool_flag: false,
    env: "SONAR_SCANNER_KEYSTORE_PASSWORD",
    property: "sonar.scanner.keystorePassword",
    usage: r##"
    --keystore-password=<PASS>    The password of the keystore file.
                   Default value: sonar
"##,
};

pub const SOURCE_ENCODING: &'static OptDesc = &OptDesc {
    option: "encoding",
    bool_flag: false,
    env: "SONAR_SOURCE_ENCODING",
    property: "sonar.sourceEncoding",
    usage: r##"
    --encoding=<ENCODING>         Encoding of the source files.
                   Default value: <system encoding>
"##,
};

pub const CLI_VERSION: &'static OptDesc = &OptDesc {
    option: "scanner-cli-version",
    bool_flag: false,
    env: "SONAR_SCANNER_INTERNAL_CLI_VERSION",
    property: "sonar.scanner.internal.cli.version",
    usage: r##"
    --scanner-cli-version=<VER>   Force the analysis to download this version of the scanner from
                                  binaries.sonarsource.com instead of using the one provided by the
                                  sonar-scan executable.
                   Default value: <the version embedded in the sonar-scan executable>
"##,
};

pub const DUMP_PROPERTIES: &'static OptDesc = &OptDesc {
    option: "dump",
    bool_flag: true,
    env: "SONAR_SCANNER_INTERNAL_DUMP_PROPERTIES",
    property: "sonar.scanner.internal.dump.properties",
    usage: r##"
    --dump                        Print all resolved options as JSON and exit.
                   Default value: false
"##,
};

pub const HELP_START: &str = r##"
Run SonarQube analysis on a project without modifying your build configuration.

USAGE:
    sonar-scan [OPTIONS]
    sonar-scan --version          Print sonar-scan version
    sonar-scan --help             Print this help

Both formats are accepted:
  - Options prefixed with '--'. For example: --token=VALUE OR --token VALUE
  - Java property name prefixed with '-D'. For example: -Dsonar.token=VALUE

OPTIONS:
"##;

pub const HELP_END: &str = r##"
    --<property>=<value>          Any additional Sonar property, without the 'sonar.' prefix,
                                  where '.' can be replaced with '-'.
                                  For example: '--branch-name' becomes 'sonar.branch.name'
"##;

pub fn usage() -> String {
    let mut usage = String::new();
    usage.push_str(HELP_START);
    for opt in OPTIONS {
        usage.push_str(opt.usage);
        if !opt.env.starts_with("SONAR_SCANNER_INTERNAL_") {
            let env = opt.env;
            usage.push_str(&format!("                   Env. variable: {env}\n"));
        }
        if !opt.property.starts_with("sonar.scanner.internal.") {
            let prop = opt.property;
            usage.push_str(&format!("                   Property name: {prop}\n"));
        }
    }
    usage.push_str(HELP_END);
    usage
}

#[derive(Debug, PartialEq, Serialize, Default)]
pub enum LogLevel {
    #[default]
    INFO,
    DEBUG,
    TRACE,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::INFO => write!(f, "INFO"),
            LogLevel::DEBUG => write!(f, "DEBUG"),
            LogLevel::TRACE => write!(f, "TRACE"),
        }
    }
}

impl LogLevel {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "INFO" => Ok(LogLevel::INFO),
            "DEBUG" => Ok(LogLevel::DEBUG),
            "TRACE" => Ok(LogLevel::TRACE),
            _ => Err(format!(
                "Invalid '{}' value '{s}', expected 'INFO', 'DEBUG' or 'TRACE'.",
                LOG_LEVEL.property
            )),
        }
    }
}

#[derive(Serialize)]
pub struct ScannerOptions {
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub sonar_properties: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub other_args: Vec<String>,
}

pub struct OptDesc {
    pub option: &'static str,
    pub bool_flag: bool,
    pub env: &'static str,
    pub property: &'static str,
    pub usage: &'static str,
}

impl ScannerOptions {
    pub fn project_base_directory(&self) -> Result<PathBuf, String> {
        self.required_path(PROJECT_BASE_DIR.property)
    }

    pub fn log_level(&self) -> LogLevel {
        if let Some(level) = self.get(LOG_LEVEL) {
            LogLevel::parse(level).unwrap_or_else(|_| LogLevel::INFO)
        } else {
            LogLevel::INFO
        }
    }

    pub fn show_debug_log(&self) -> bool {
        let level = self.log_level();
        level == LogLevel::DEBUG || level == LogLevel::TRACE
    }

    pub fn sonar_home(&self) -> Result<PathBuf, String> {
        Ok(PathBuf::from(self.required(SONAR_HOME)?))
    }

    pub fn sonar_cache(&self) -> Result<PathBuf, String> {
        Ok(self.sonar_home()?.join("cache"))
    }

    pub fn required(&self, opt: &OptDesc) -> Result<&String, String> {
        if let Some(url) = self.get(opt) {
            Ok(url)
        } else {
            Err(format!("Missing required property: {}", opt.property))
        }
    }

    pub fn has(&self, opt: &OptDesc) -> bool {
        self.sonar_properties.contains_key(opt.property)
    }

    pub fn get(&self, opt: &OptDesc) -> Option<&String> {
        self.sonar_properties.get(opt.property)
    }

    pub fn set(&mut self, opt: &OptDesc, value: String) -> Option<String> {
        self.sonar_properties.insert(opt.property.to_owned(), value)
    }

    pub fn set_str(&mut self, opt: &OptDesc, value: &str) -> Option<String> {
        self.sonar_properties.insert(opt.property.to_owned(), value.to_owned())
    }

    pub fn required_path(&self, property: &str) -> Result<PathBuf, String> {
        if let Some(path_str) = self.sonar_properties.get(property) {
            let path = PathBuf::from_str(path_str).map_err(|e| {
                format!("Invalid path for property '{property}' with value '{path_str}': {e}")
            })?;
            Ok(path)
        } else {
            Err(format!("Missing required property: {property}"))
        }
    }

    pub fn is_true(&self, opt: &OptDesc) -> bool {
        if let Some(value) = self.get(opt) {
            value == "true"
        } else {
            false
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
    }
}

#[derive(Debug, PartialEq)]
enum ArgKind {
    DoubleDash,      // e.g., --foo
    DoubleDashEqual, // e.g., --foo=123
    DashDEqual,      // e.g., -Dfoo=123
}

#[derive(Debug, PartialEq)]
struct ParsedArg {
    kind: ArgKind,
    name: String,
    value: String,
}

fn parse_arg(arg: &str) -> Option<ParsedArg> {
    if let Some(rest) = arg.strip_prefix("--") {
        if rest.is_empty() {
            // "--"
            return None;
        }
        if let Some((name, value)) = rest.split_once('=') {
            if name.is_empty() {
                return None;
            } // e.g. "--=123"
            Some(ParsedArg {
                // e.g. "--foo=123"
                kind: ArgKind::DoubleDashEqual,
                name: name.to_string(),
                value: value.to_string(),
            })
        } else {
            Some(ParsedArg {
                // e.g. "--foo"
                kind: ArgKind::DoubleDash,
                name: rest.to_string(),
                value: String::new(),
            })
        }
    } else if let Some(rest) = arg.strip_prefix("-D") {
        if let Some((name, value)) = rest.split_once('=') {
            if name.is_empty() {
                return None;
            } // e.g. "-D=123"
            Some(ParsedArg {
                // e.g. "-Dfoo=123"
                kind: ArgKind::DashDEqual,
                name: name.to_string(),
                value: value.to_string(),
            })
        } else {
            None // e.g. "-Dfoo"
        }
    } else {
        None // e.g. "foo" or ""
    }
}

fn find_option(arg: &ParsedArg) -> Option<&'static OptDesc> {
    match arg.kind {
        ArgKind::DoubleDash | ArgKind::DoubleDashEqual => {
            for opt in OPTIONS {
                if opt.option == arg.name {
                    return Some(opt);
                }
            }
        }
        ArgKind::DashDEqual => {
            for opt in OPTIONS {
                if opt.property == arg.name {
                    return Some(opt);
                }
            }
        }
    }
    None
}

pub fn parse_options(
    args: &[String],
    env_fn: &dyn Fn(&str) -> Option<String>,
) -> Result<ScannerOptions, String> {
    let mut options = ScannerOptions {
        sonar_properties: BTreeMap::new(),
        other_args: Vec::new(),
    };
    add_options_from_args(&mut options, &args)?;
    add_missing_options_from_env(&mut options, env_fn);
    infer_missing_options(&mut options)?;
    Ok(options)
}

fn add_missing_options_from_env(
    options: &mut ScannerOptions,
    env_fn: &dyn Fn(&str) -> Option<String>,
) {
    let properties = &mut options.sonar_properties;
    for opt in OPTIONS {
        if !properties.contains_key(opt.property) {
            if let Some(value) = env_fn(opt.env) {
                properties.insert(
                    opt.property.to_string(),
                    if opt.bool_flag {
                        value.to_lowercase()
                    } else {
                        value
                    },
                );
            }
        }
    }
}

fn add_options_from_args(options: &mut ScannerOptions, args: &[String]) -> Result<(), String> {
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if let Some(parsed_arg) = parse_arg(&args[i]) {
            if let Some(opt) = find_option(&parsed_arg) {
                let value: String = if opt.bool_flag && parsed_arg.kind == ArgKind::DoubleDash {
                    "true".to_string()
                } else if parsed_arg.kind == ArgKind::DoubleDashEqual
                    || parsed_arg.kind == ArgKind::DashDEqual
                {
                    parsed_arg.value
                } else {
                    // DoubleDash && !bool_flag
                    let Some(next) = args.get(i + 1).filter(|v| !v.starts_with("--")) else {
                        return Err(format!("Missing value for argument: {arg}"));
                    };
                    i += 1;
                    next.clone()
                };
                options
                    .sonar_properties
                    .insert(opt.property.to_string(), value);
            } else {
                if parsed_arg.kind == ArgKind::DashDEqual && parsed_arg.name.starts_with("sonar.") {
                    options
                        .sonar_properties
                        .insert(parsed_arg.name, parsed_arg.value);
                } else {
                    options.other_args.push(arg.clone());
                }
            }
        } else {
            options.other_args.push(arg.clone());
        };
        i += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use regex::Regex;
    use std::collections::HashMap;
    use std::env;
    use tempfile::tempdir;

    fn cur_dir() -> String {
        env::current_dir().unwrap().to_string_lossy().into_owned()
    }

    fn user_dir() -> String {
        dirs::home_dir().unwrap().to_string_lossy().into_owned()
    }

    fn remove_os(input: &str) -> String {
        let re = Regex::new(r#""sonar\.scanner\.os": "(?:linux|alpine|macos|windows)""#).unwrap();
        re.replace(input, r#""sonar.scanner.os"": "<os>""#)
            .to_string()
    }

    fn remove_arch(input: &str) -> String {
        let re = Regex::new(r#""sonar\.scanner\.arch": "(?:aarch64|x64)""#).unwrap();
        re.replace(input, r#""sonar.scanner.arch"": "<arch>""#)
            .to_string()
    }

    fn json_from_parse_options(args: &[&str], env: &[(&str, &str)]) -> Result<String, String> {
        let string_args: Vec<String> = args.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        let env_map: HashMap<&str, &str> = env.iter().copied().collect();
        let env_closure =
            |key: &str| -> Option<String> { env_map.get(key).map(|val| val.to_string()) };
        let options = parse_options(&string_args, &env_closure)?;
        let mut json = options
            .to_json()
            .replace(&cur_dir(), "<current-dir>")
            .replace(&user_dir(), "<user-home>");
        json = remove_os(&json);
        json = remove_arch(&json);
        json = json.replace("\\", "/");
        Ok(json)
    }

    #[test]
    fn parse_options_defaults() -> Result<(), String> {
        assert_eq!(
            json_from_parse_options(/*args*/ &[], /*env*/ &[])?,
            indoc! {r#"
        {
          "sonar_properties": {
            "sonar.host.url": "https://sonarcloud.io",
            "sonar.java.binaries": "<user-home>/.sonar/cache/empty_directory",
            "sonar.projectBaseDir": "<current-dir>",
            "sonar.projectKey": "sonar-scan",
            "sonar.projectName": "sonar-scan",
            "sonar.scanner.apiBaseUrl": "https://api.sonarcloud.io",
            "sonar.scanner.arch"": "<arch>",
            "sonar.scanner.internal.isSonarCloud": "true",
            "sonar.scanner.os"": "<os>",
            "sonar.scanner.sonarcloudUrl": "https://sonarcloud.io",
            "sonar.userHome": "<user-home>/.sonar"
          }
        }"#}
        );
        Ok(())
    }

    #[test]
    fn parse_options_eq_form() -> Result<(), String> {
        let tmp_dir = tempdir().unwrap();
        let tmp_dir_path = tmp_dir.path().to_owned().canonicalize().unwrap();
        let tmp_dir_str = tmp_dir_path.to_string_lossy().into_owned();
        let project_dir = tmp_dir_path.join("my-project");
        std::fs::create_dir_all(&project_dir).unwrap();
        let java_bin_dir = tmp_dir_path.join("jvm-25").join("jre").join("bin");
        std::fs::create_dir_all(&java_bin_dir).unwrap();
        let java_exe_path = java_bin_dir.join("java");
        std::fs::write(&java_exe_path, "java exe ...").unwrap();
        let java_exe_str = java_exe_path.to_string_lossy().into_owned();
        let project_dir_str = project_dir.to_string_lossy();
        let home_dir = dirs::home_dir().unwrap().to_string_lossy().into_owned();

        assert_eq!(
            json_from_parse_options(
                /*args*/
                &[
                    "--token=sqa_1234567890",
                    format!("--dir={project_dir_str}").as_str(),
                    "--url=http://localhost:9000",
                    "--org=BigCorp",
                    "--region=us",
                    "--key=my-project-key",
                    "--name=My Project Name",
                    "--version=1.2.3",
                    "--description=Main Application",
                    "--build-number=42",
                    "--sources=src/main/java",
                    "--tests=src/test/java",
                    "--java-binaries=target/classes",
                    "--java-libraries=target/libs/*.jar",
                    "--java-test-binaries=target/test-classes",
                    "--java-test-libraries=target/libs/*.jar,target/test-libs/*.jar",
                    "--os=windows",
                    "--arch=x64",
                    "--skip-jre-provisioning",
                    format!("--java-exe-path={java_exe_str}").as_str(),
                    "--proxy-host=https://proxy.bigcorp.com",
                    "--proxy-port=666",
                    "--proxy-user=paul",
                    "--proxy-password=Paul123456",
                    format!("--sonar-home={home_dir}/.sonar-custom").as_str(),
                    "--java-opts=-Xmx1024m",
                    "--work-dir=.custom-work-dir",
                    "--verbose",
                    "--log=DEBUG",
                    format!("--truststore-path={home_dir}/custom-truststore/ssl/truststore.p12")
                        .as_str(),
                    "--truststore-password=TruststorePassword",
                    format!("--keystore-path={home_dir}/custom-keystore/ssl/keystore.p12").as_str(),
                    "--keystore-password=KeystorePassword",
                    "--encoding=UTF-8",
                    "--scanner-cli-version=7.2.0.5079",
                    "--dump",
                ],
                /*env*/ &[]
            )?
            .replace(&tmp_dir_str, "<tmp-dir>"),
            indoc! {r#"
        {
          "sonar_properties": {
            "sonar.analysis.buildNumber": "42",
            "sonar.host.url": "http://localhost:9000",
            "sonar.java.binaries": "target/classes",
            "sonar.java.libraries": "target/libs/*.jar",
            "sonar.java.test.binaries": "target/test-classes",
            "sonar.java.test.libraries": "target/libs/*.jar,target/test-libs/*.jar",
            "sonar.log.level": "DEBUG",
            "sonar.organization": "BigCorp",
            "sonar.projectBaseDir": "<tmp-dir>/my-project",
            "sonar.projectDescription": "Main Application",
            "sonar.projectKey": "my-project-key",
            "sonar.projectName": "My Project Name",
            "sonar.projectVersion": "1.2.3",
            "sonar.region": "us",
            "sonar.scanner.apiBaseUrl": "http://localhost:9000",
            "sonar.scanner.arch"": "<arch>",
            "sonar.scanner.internal.cli.version": "7.2.0.5079",
            "sonar.scanner.internal.dump.properties": "true",
            "sonar.scanner.internal.isSonarCloud": "true",
            "sonar.scanner.javaExePath": "<tmp-dir>/jvm-25/jre/bin/java",
            "sonar.scanner.javaOpts": "-Xmx1024m",
            "sonar.scanner.keystorePassword": "KeystorePassword",
            "sonar.scanner.keystorePath": "<user-home>/custom-keystore/ssl/keystore.p12",
            "sonar.scanner.os"": "<os>",
            "sonar.scanner.proxyHost": "https://proxy.bigcorp.com",
            "sonar.scanner.proxyPassword": "Paul123456",
            "sonar.scanner.proxyPort": "666",
            "sonar.scanner.proxyUser": "paul",
            "sonar.scanner.skipJreProvisioning": "true",
            "sonar.scanner.sonarcloudUrl": "http://localhost:9000",
            "sonar.scanner.truststorePassword": "TruststorePassword",
            "sonar.scanner.truststorePath": "<user-home>/custom-truststore/ssl/truststore.p12",
            "sonar.sourceEncoding": "UTF-8",
            "sonar.sources": "src/main/java",
            "sonar.tests": "src/test/java",
            "sonar.token": "sqa_1234567890",
            "sonar.userHome": "<user-home>/.sonar-custom",
            "sonar.verbose": "true",
            "sonar.working.directory": ".custom-work-dir"
          }
        }"#}
        );
        tmp_dir.close().unwrap();
        Ok(())
    }

    #[test]
    fn parse_options_space_form() -> Result<(), String> {
        let tmp_dir = tempdir().unwrap();
        let tmp_dir_path = tmp_dir.path().to_owned().canonicalize().unwrap();
        let tmp_dir_str = tmp_dir_path.to_string_lossy().into_owned();
        let project_dir = tmp_dir_path.join("my-project");
        std::fs::create_dir_all(&project_dir).unwrap();
        let java_bin_dir = tmp_dir_path.join("jvm-25").join("jre").join("bin");
        std::fs::create_dir_all(&java_bin_dir).unwrap();
        let java_exe_path = java_bin_dir.join("java");
        std::fs::write(&java_exe_path, "java exe ...").unwrap();
        let java_exe_str = java_exe_path.to_string_lossy().into_owned();
        let project_dir_str = project_dir.to_string_lossy();
        let home_dir = dirs::home_dir().unwrap().to_string_lossy().into_owned();

        assert_eq!(
            json_from_parse_options(
                /*args*/
                &[
                    "--token",
                    "sqa_1234567890",
                    "--dir",
                    &project_dir_str,
                    "--url",
                    "http://localhost:9000",
                    "--org",
                    "BigCorp",
                    "--region",
                    "us",
                    "--key",
                    "my-project-key",
                    "--name",
                    "My Project Name",
                    "--version",
                    "1.2.3",
                    "--description",
                    "Main Application",
                    "--build-number",
                    "42",
                    "--sources",
                    "src/main/java",
                    "--tests",
                    "src/test/java",
                    "--java-binaries",
                    "target/classes",
                    "--java-libraries",
                    "target/libs/*.jar",
                    "--java-test-binaries",
                    "target/test-classes",
                    "--java-test-libraries",
                    "target/libs/*.jar,target/test-libs/*.jar",
                    "--os",
                    "windows",
                    "--arch",
                    "x64",
                    "--skip-jre-provisioning",
                    "--java-exe-path",
                    &java_exe_str,
                    "--proxy-host",
                    "https://proxy.bigcorp.com",
                    "--proxy-port",
                    "666",
                    "--proxy-user",
                    "paul",
                    "--proxy-password",
                    "Paul123456",
                    "--sonar-home",
                    format!("{home_dir}/.sonar-custom").as_str(),
                    "--java-opts",
                    "-Xmx1024m",
                    "--work-dir",
                    ".custom-work-dir",
                    "--verbose",
                    "--log",
                    "DEBUG",
                    "--truststore-path",
                    format!("{home_dir}/custom-truststore/ssl/truststore.p12").as_str(),
                    "--truststore-password",
                    "TruststorePassword",
                    "--keystore-path",
                    format!("{home_dir}/custom-keystore/ssl/keystore.p12").as_str(),
                    "--keystore-password",
                    "KeystorePassword",
                    "--encoding",
                    "UTF-8",
                    "--scanner-cli-version",
                    "7.2.0.5079",
                    "--dump",
                ],
                /*env*/ &[]
            )?
            .replace(&tmp_dir_str, "<tmp-dir>"),
            indoc! {r#"
        {
          "sonar_properties": {
            "sonar.analysis.buildNumber": "42",
            "sonar.host.url": "http://localhost:9000",
            "sonar.java.binaries": "target/classes",
            "sonar.java.libraries": "target/libs/*.jar",
            "sonar.java.test.binaries": "target/test-classes",
            "sonar.java.test.libraries": "target/libs/*.jar,target/test-libs/*.jar",
            "sonar.log.level": "DEBUG",
            "sonar.organization": "BigCorp",
            "sonar.projectBaseDir": "<tmp-dir>/my-project",
            "sonar.projectDescription": "Main Application",
            "sonar.projectKey": "my-project-key",
            "sonar.projectName": "My Project Name",
            "sonar.projectVersion": "1.2.3",
            "sonar.region": "us",
            "sonar.scanner.apiBaseUrl": "http://localhost:9000",
            "sonar.scanner.arch"": "<arch>",
            "sonar.scanner.internal.cli.version": "7.2.0.5079",
            "sonar.scanner.internal.dump.properties": "true",
            "sonar.scanner.internal.isSonarCloud": "true",
            "sonar.scanner.javaExePath": "<tmp-dir>/jvm-25/jre/bin/java",
            "sonar.scanner.javaOpts": "-Xmx1024m",
            "sonar.scanner.keystorePassword": "KeystorePassword",
            "sonar.scanner.keystorePath": "<user-home>/custom-keystore/ssl/keystore.p12",
            "sonar.scanner.os"": "<os>",
            "sonar.scanner.proxyHost": "https://proxy.bigcorp.com",
            "sonar.scanner.proxyPassword": "Paul123456",
            "sonar.scanner.proxyPort": "666",
            "sonar.scanner.proxyUser": "paul",
            "sonar.scanner.skipJreProvisioning": "true",
            "sonar.scanner.sonarcloudUrl": "http://localhost:9000",
            "sonar.scanner.truststorePassword": "TruststorePassword",
            "sonar.scanner.truststorePath": "<user-home>/custom-truststore/ssl/truststore.p12",
            "sonar.sourceEncoding": "UTF-8",
            "sonar.sources": "src/main/java",
            "sonar.tests": "src/test/java",
            "sonar.token": "sqa_1234567890",
            "sonar.userHome": "<user-home>/.sonar-custom",
            "sonar.verbose": "true",
            "sonar.working.directory": ".custom-work-dir"
          }
        }"#}
        );
        tmp_dir.close().unwrap();
        Ok(())
    }

    #[test]
    fn parse_env_variables() -> Result<(), String> {
        let tmp_dir = tempdir().unwrap();
        let tmp_dir_path = tmp_dir.path().to_owned().canonicalize().unwrap();
        let tmp_dir_str = tmp_dir_path.to_string_lossy().into_owned();
        let project_dir = tmp_dir_path.join("my-project");
        std::fs::create_dir_all(&project_dir).unwrap();
        let java_bin_dir = tmp_dir_path.join("jvm-25").join("jre").join("bin");
        std::fs::create_dir_all(&java_bin_dir).unwrap();
        let java_exe_path = java_bin_dir.join("java");
        std::fs::write(&java_exe_path, "java exe ...").unwrap();
        let java_exe_str = java_exe_path.to_string_lossy().into_owned();
        let project_dir_str = project_dir.to_string_lossy();
        let home_dir = dirs::home_dir().unwrap().to_string_lossy().into_owned();

        assert_eq!(
            json_from_parse_options(
                /*args*/ &[],
                /*env*/
                &[
                    ("SONAR_TOKEN", "sqa_1234567890"),
                    ("SONAR_BASE_DIR", &project_dir_str),
                    ("SONAR_HOST_URL", "http://localhost:9000"),
                    ("SONAR_ORGANIZATION", "BigCorp"),
                    ("SONAR_REGION", "us"),
                    ("SONAR_PROJECT_KEY", "my-project-key"),
                    ("SONAR_PROJECT_NAME", "My Project Name"),
                    ("SONAR_PROJECT_VERSION", "1.2.3"),
                    ("SONAR_PROJECT_DESCRIPTION", "Main Application"),
                    ("SONAR_PROJECT_BUILD_NUMBER", "42"),
                    ("SONAR_PROJECT_SOURCES", "src/main/java"),
                    ("SONAR_PROJECT_TESTS", "src/test/java"),
                    ("SONAR_JAVA_BINARIES", "target/classes"),
                    ("SONAR_JAVA_LIBRARIES", "target/libs/*.jar"),
                    ("SONAR_JAVA_TEST_BINARIES", "target/test-classes"),
                    (
                        "SONAR_JAVA_TEST_LIBRARIES",
                        "target/libs/*.jar,target/test-libs/*.jar"
                    ),
                    ("SONAR_SCANNER_OS", "windows"),
                    ("SONAR_SCANNER_ARCH", "x64"),
                    ("SONAR_SCANNER_SKIP_JRE_PROVISIONING", "true"),
                    ("SONAR_SCANNER_JAVA_EXE_PATH", &java_exe_str),
                    ("SONAR_SCANNER_PROXY_HOST", "https://proxy.bigcorp.com"),
                    ("SONAR_SCANNER_PROXY_PORT", "666"),
                    ("SONAR_SCANNER_PROXY_USER", "paul"),
                    ("SONAR_SCANNER_PROXY_PASSWORD", "Paul123456"),
                    (
                        "SONAR_USER_HOME",
                        format!("{home_dir}/.sonar-custom").as_str()
                    ),
                    ("SONAR_SCANNER_JAVA_OPTS", "-Xmx1024m"),
                    ("SONAR_SCANNER_WORK_DIR", ".custom-work-dir"),
                    ("SONAR_VERBOSE", "true"),
                    ("SONAR_LOG_LEVEL", "DEBUG"),
                    (
                        "SONAR_SCANNER_TRUSTSTORE_PATH",
                        format!("{home_dir}/custom-truststore/ssl/truststore.p12").as_str()
                    ),
                    ("SONAR_SCANNER_TRUSTSTORE_PASSWORD", "TruststorePassword"),
                    (
                        "SONAR_SCANNER_KEYSTORE_PATH",
                        format!("{home_dir}/custom-keystore/ssl/keystore.p12").as_str()
                    ),
                    ("SONAR_SCANNER_KEYSTORE_PASSWORD", "KeystorePassword"),
                    ("SONAR_SOURCE_ENCODING", "UTF-8"),
                    ("SONAR_SCANNER_INTERNAL_CLI_VERSION", "7.2.0.5079"),
                    ("SONAR_SCANNER_INTERNAL_DUMP_PROPERTIES", "true"),
                ]
            )?
            .replace(&tmp_dir_str, "<tmp-dir>"),
            indoc! {r#"
        {
          "sonar_properties": {
            "sonar.analysis.buildNumber": "42",
            "sonar.host.url": "http://localhost:9000",
            "sonar.java.binaries": "target/classes",
            "sonar.java.libraries": "target/libs/*.jar",
            "sonar.java.test.binaries": "target/test-classes",
            "sonar.java.test.libraries": "target/libs/*.jar,target/test-libs/*.jar",
            "sonar.log.level": "DEBUG",
            "sonar.organization": "BigCorp",
            "sonar.projectBaseDir": "<tmp-dir>/my-project",
            "sonar.projectDescription": "Main Application",
            "sonar.projectKey": "my-project-key",
            "sonar.projectName": "My Project Name",
            "sonar.projectVersion": "1.2.3",
            "sonar.region": "us",
            "sonar.scanner.apiBaseUrl": "http://localhost:9000",
            "sonar.scanner.arch"": "<arch>",
            "sonar.scanner.internal.cli.version": "7.2.0.5079",
            "sonar.scanner.internal.dump.properties": "true",
            "sonar.scanner.internal.isSonarCloud": "true",
            "sonar.scanner.javaExePath": "<tmp-dir>/jvm-25/jre/bin/java",
            "sonar.scanner.javaOpts": "-Xmx1024m",
            "sonar.scanner.keystorePassword": "KeystorePassword",
            "sonar.scanner.keystorePath": "<user-home>/custom-keystore/ssl/keystore.p12",
            "sonar.scanner.os"": "<os>",
            "sonar.scanner.proxyHost": "https://proxy.bigcorp.com",
            "sonar.scanner.proxyPassword": "Paul123456",
            "sonar.scanner.proxyPort": "666",
            "sonar.scanner.proxyUser": "paul",
            "sonar.scanner.skipJreProvisioning": "true",
            "sonar.scanner.sonarcloudUrl": "http://localhost:9000",
            "sonar.scanner.truststorePassword": "TruststorePassword",
            "sonar.scanner.truststorePath": "<user-home>/custom-truststore/ssl/truststore.p12",
            "sonar.sourceEncoding": "UTF-8",
            "sonar.sources": "src/main/java",
            "sonar.tests": "src/test/java",
            "sonar.token": "sqa_1234567890",
            "sonar.userHome": "<user-home>/.sonar-custom",
            "sonar.verbose": "true",
            "sonar.working.directory": ".custom-work-dir"
          }
        }"#}
        );
        tmp_dir.close().unwrap();
        Ok(())
    }

    #[test]
    fn parse_options_unknown_arg_becomes_other_args() -> Result<(), String> {
        assert_eq!(
            json_from_parse_options(
                /*args*/
                &[
                    "--some-custom-prop=hello",
                    "--some-other-prop",
                    "hello",
                    "-Dsonar.unknown.prop=123",
                ],
                /*env*/ &[]
            )?,
            indoc! {r#"
        {
          "sonar_properties": {
            "sonar.host.url": "https://sonarcloud.io",
            "sonar.java.binaries": "<user-home>/.sonar/cache/empty_directory",
            "sonar.projectBaseDir": "<current-dir>",
            "sonar.projectKey": "sonar-scan",
            "sonar.projectName": "sonar-scan",
            "sonar.scanner.apiBaseUrl": "https://api.sonarcloud.io",
            "sonar.scanner.arch"": "<arch>",
            "sonar.scanner.internal.isSonarCloud": "true",
            "sonar.scanner.os"": "<os>",
            "sonar.scanner.sonarcloudUrl": "https://sonarcloud.io",
            "sonar.unknown.prop": "123",
            "sonar.userHome": "<user-home>/.sonar"
          },
          "other_args": [
            "--some-custom-prop=hello",
            "--some-other-prop",
            "hello"
          ]
        }"#}
        );
        Ok(())
    }

    #[test]
    fn parse_options_override_env_vars() -> Result<(), String> {
        assert_eq!(
            json_from_parse_options(
                /*args*/
                &[
                    "--token",
                    "sqa_1234567890",
                    "--url",
                    "http://localhost:9000",
                    "--org",
                    "BigCorp",
                    "--verbose=false",
                ],
                /*env*/
                &[
                    ("SONAR_TOKEN", "sqa_6666666666"),
                    ("SONAR_HOST_URL", "http://my.server.com:9000"),
                    ("SONAR_ORGANIZATION", "DefaultCorp"),
                    ("SONAR_VERBOSE", "true"),
                ]
            )?,
            indoc! {r#"
        {
          "sonar_properties": {
            "sonar.host.url": "http://localhost:9000",
            "sonar.java.binaries": "<user-home>/.sonar/cache/empty_directory",
            "sonar.organization": "BigCorp",
            "sonar.projectBaseDir": "<current-dir>",
            "sonar.projectKey": "sonar-scan",
            "sonar.projectName": "sonar-scan",
            "sonar.scanner.apiBaseUrl": "http://localhost:9000/api/v2",
            "sonar.scanner.arch"": "<arch>",
            "sonar.scanner.internal.isSonarCloud": "false",
            "sonar.scanner.os"": "<os>",
            "sonar.token": "sqa_1234567890",
            "sonar.userHome": "<user-home>/.sonar",
            "sonar.verbose": "false"
          }
        }"#}
        );
        Ok(())
    }

    #[test]
    fn parse_no_region_custom_url_select_sonarqube_server() -> Result<(), String> {
        assert_eq!(
            json_from_parse_options(
                /*args*/ &["--url", "https://server.com", "--org", "BigCorp",],
                /*env*/ &[]
            )?,
            indoc! {r#"
        {
          "sonar_properties": {
            "sonar.host.url": "https://server.com",
            "sonar.java.binaries": "<user-home>/.sonar/cache/empty_directory",
            "sonar.organization": "BigCorp",
            "sonar.projectBaseDir": "<current-dir>",
            "sonar.projectKey": "sonar-scan",
            "sonar.projectName": "sonar-scan",
            "sonar.scanner.apiBaseUrl": "https://server.com/api/v2",
            "sonar.scanner.arch"": "<arch>",
            "sonar.scanner.internal.isSonarCloud": "false",
            "sonar.scanner.os"": "<os>",
            "sonar.userHome": "<user-home>/.sonar"
          }
        }"#}
        );
        Ok(())
    }

    #[test]
    fn parse_region_us() -> Result<(), String> {
        assert_eq!(
            json_from_parse_options(/*args*/ &["--region", "us",], /*env*/ &[])?,
            indoc! {r#"
        {
          "sonar_properties": {
            "sonar.host.url": "https://sonarqube.us",
            "sonar.java.binaries": "<user-home>/.sonar/cache/empty_directory",
            "sonar.projectBaseDir": "<current-dir>",
            "sonar.projectKey": "sonar-scan",
            "sonar.projectName": "sonar-scan",
            "sonar.region": "us",
            "sonar.scanner.apiBaseUrl": "https://api.sonarqube.us",
            "sonar.scanner.arch"": "<arch>",
            "sonar.scanner.internal.isSonarCloud": "true",
            "sonar.scanner.os"": "<os>",
            "sonar.scanner.sonarcloudUrl": "https://sonarqube.us",
            "sonar.userHome": "<user-home>/.sonar"
          }
        }"#}
        );
        Ok(())
    }

    #[test]
    fn parse_sonarcloud_dev_eu() -> Result<(), String> {
        assert_eq!(
            json_from_parse_options(
                /*args*/ &["--url", "https://dev5.sc-dev5.io",],
                /*env*/ &[]
            )?,
            indoc! {r#"
        {
          "sonar_properties": {
            "sonar.host.url": "https://dev5.sc-dev5.io",
            "sonar.java.binaries": "<user-home>/.sonar/cache/empty_directory",
            "sonar.projectBaseDir": "<current-dir>",
            "sonar.projectKey": "sonar-scan",
            "sonar.projectName": "sonar-scan",
            "sonar.region": "",
            "sonar.scanner.apiBaseUrl": "https://api.sc-dev5.io",
            "sonar.scanner.arch"": "<arch>",
            "sonar.scanner.internal.isSonarCloud": "true",
            "sonar.scanner.os"": "<os>",
            "sonar.scanner.sonarcloudUrl": "https://dev5.sc-dev5.io",
            "sonar.userHome": "<user-home>/.sonar"
          }
        }"#}
        );
        Ok(())
    }
}
