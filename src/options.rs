use serde::Serialize;
use std::collections::BTreeMap;
use std::env;
use std::fmt;
use std::path::PathBuf;

pub const DEFAULT_URL: &str = "https://sonarcloud.io";
pub const DEFAULT_WORK_DIR: &str = ".scannerwork";

pub const HELP: &str = r##"
Run SonarQube analysis on a project without modifying your build configuration.

USAGE:
    sonar-scan [OPTIONS]
    sonar-scan --version          Print sonar-scan version
    sonar-scan --help             Print this help

Both formats are accepted:
  - Options prefixed with '--'. For example: --token=VALUE OR --token VALUE
  - Java property name prefixed with '-D'. For example: -Dsonar.token=VALUE

OPTIONS:

    --token=<TOKEN>               Token used by the scanner to authenticate to your SonarQube
                                  instance. This property has no default value and is mandatory.
                   Env. variable: SONAR_TOKEN
                   Property name: sonar.token

    --dir=<PATH>                  The project's base directory. Use this property when you need the
                                  analysis to take place in a directory other than the one from
                                  which it was started.
                   Default value: <current directory>
                   Env. variable: SONAR_BASE_DIR
                   Property name: sonar.projectBaseDir

    --url=<URL>                   The URL to your SonarQube instance.
                   Default value: https://sonarcloud.io
                   Env. variable: SONAR_HOST_URL
                   Property name: sonar.host.url

    --key=<KEY>                   The project's unique key. Can include up to 400 characters. All
                                  letters, digits, dashes, underscores, periods, and colons are
                                  accepted.
                   Default value: <git repository name or base directory name>
                   Env. variable: SONAR_PROJECT_KEY
                   Property name: sonar.projectKey

    --name=<NAME>                 Name of the project that will be displayed on the web interface.
                   Default value: <value of sonar.projectKey>
                   Env. variable: SONAR_PROJECT_NAME
                   Property name: sonar.projectName

    --version=<VERSION>           The project version. It should be set for branch analysis in case
                                  you use the new code definition based on the previous version.
                   Default value: <empty>
                   Env. variable: SONAR_PROJECT_VERSION
                   Property name: sonar.projectVersion

    --description=<TEXT>          The project description.
                   Default value: <empty>
                   Env. variable: SONAR_PROJECT_DESCRIPTION
                   Property name: sonar.projectDescription

    --build-number=<NUMBER>       The project build number.
                   Default value: <empty>
                   Env. variable: SONAR_PROJECT_BUILD_NUMBER
                   Property name: sonar.analysis.buildNumber

    --sources=<PATHS>             The initial analysis scope for main source code (non-test code) in
                                  the project. Comma-separated paths to directories are included. An
                                  individual file in the list means that the file is included. A
                                  directory in the list means that all analyzable files and
                                  directories recursively below it are included. The path can be
                                  relative (to the sonar.projectBaseDir property) or absolute.
                                  Wildcards (*, ** and ?) are not allowed.
                   Default value: <value of sonar.projectBaseDir>
                   Env. variable: SONAR_PROJECT_SOURCES
                   Property name: sonar.sources

    --tests=<PATHS>               The initial analysis scope for test code in the project.
                                  Same format as '--sources'.
                   Default value: <empty>
                   Env. variable: SONAR_PROJECT_TESTS
                   Property name: sonar.tests

    --os=<OS>                     The operating system of the machine hosting the SonarScanner.
                                  Possible values: windows, linux, macos (or Darwin), alpine
                   Default value: <autodetected value>
                   Env. variable: SONAR_SCANNER_OS
                   Property name: sonar.scanner.os

    --arch=<ARCH>                 The CPU architecture type.
                                  Possible values: x64 (or x86_64, amd64), aarch64 (or arm64)
                   Default value: <autodetected value>
                   Env. variable: SONAR_SCANNER_ARCH
                   Property name: sonar.scanner.arch

    --skip-jre-provisioning       Skip the Java Runtime Environment (JRE) download from the
                                  SonarQube instance.
                   Default value: false
                   Env. variable: SONAR_SCANNER_SKIP_JRE_PROVISIONING
                   Property name: sonar.scanner.skipJreProvisioning

    --java-exe-path=<PATH>        If defined, the SonarScanner runs with this Java executable.
                   Default value: <bin/java of the provisioned or autodetected JRE>
                   Env. variable: SONAR_SCANNER_JAVA_EXE_PATH
                   Property name: sonar.scanner.javaExePath

    --proxy-host=<HOST>           The host name of the proxy server.
                   Default value: <empty>
                   Env. variable: SONAR_SCANNER_PROXY_HOST
                   Property name: sonar.scanner.proxyHost

    --proxy-port=<PORT>           The port of the proxy server.
                   Default value: <same port as sonar.host.url>
                   Env. variable: SONAR_SCANNER_PROXY_PORT
                   Property name: sonar.scanner.proxyPort

    --proxy-user=<USER>           In case of an authenticated proxy: the user name.
                   Default value: <empty>
                   Env. variable: SONAR_SCANNER_PROXY_USER
                   Property name: sonar.scanner.proxyUser

    --proxy-password=<PASSWORD>   In case of an authenticated proxy: the user password.
                   Default value: <empty>
                   Env. variable: SONAR_SCANNER_PROXY_PASSWORD
                   Property name: sonar.scanner.proxyPassword

    --sonar-home=<PATH>           The directory for the scanner cache.
                   Default value: ${HOME}/.sonar
                   Env. variable: SONAR_USER_HOME
                   Property name: sonar.userHome

    --java-opts=<OPTS>            Arguments to pass to the JVM running the scanner engine.
                   Default value: <empty>
                   Env. variable: SONAR_SCANNER_JAVA_OPTS
                   Property name: sonar.scanner.javaOpts

    --work-dir=<DIR>              Path to the working directory used by the SonarScanner during a
                                  project analysis to store temporary data. The path can be relative
                                  (to the sonar.projectBaseDir property) or absolute.
                   Default value: .scannerwork
                   Env. variable: SONAR_SCANNER_WORK_DIR
                   Property name: sonar.working.directory

    --verbose                     When present, adds more details to the analysis logs by activating
                                  the DEBUG mode for the scanner.
                   Property name: sonar.verbose

    --log=<LEVEL>                 The log level.
                                  Possible values: INFO, DEBUG, TRACE
                   Default value: <if sonar.verbose then DEBUG else INFO>
                   Env. variable: SONAR_LOG_LEVEL
                   Property name: sonar.log.level

    --truststore-path=<PATH>      The path to the truststore file.
                   Default value: <sonar.userHome>/ssl/truststore.p12
                   Property name: sonar.scanner.truststorePath

    --truststore-password=<PASS>  The password of the truststore.
                   Default value: changeit
                   Property name: sonar.scanner.truststorePassword

    --keystore-path=<PATH>        The path to the keystore file.
                   Default value: <sonar.userHome>/ssl/keystore.p12
                   Property name: sonar.scanner.keystorePath

    --keystore-password=<PASS>    The password of the keystore file.
                   Default value: sonar
                   Property name: sonar.scanner.keystorePassword

    --encoding=<ENCODING>         Encoding of the source files.
                   Default value: <system encoding>
                   Property name: sonar.sourceEncoding

    --<property>=<value>          Any additional Sonar property, without the 'sonar.' prefix,
                                  where '.' can be replaced with '-'.
                                  For example: '--branch-name' becomes 'sonar.branch.name'

    --scanner-version=<VERSION>   Force the analysis to download this version of the scanner from
                                  binaries.sonarsource.com instead of using the one provided by the
                                  SonarQube instance.
                   Default value: <the version provided by the connected SonarQube instance>

    --dump                        Print all resolved options as JSON and exit.
"##;

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
                "invalid log level: {s} (expected INFO, DEBUG or TRACE)"
            )),
        }
    }
}

fn is_false(b: &bool) -> bool {
    !b
}

#[derive(Debug, Serialize, Default)]
pub struct ScannerOptions {
    pub dir: PathBuf,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tests: Option<String>,
    pub os: String,
    pub arch: String,
    #[serde(skip_serializing_if = "is_false")]
    pub skip_jre_provisioning: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub java_exe_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_password: Option<String>,
    pub sonar_home: PathBuf,
    pub sonar_cache: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub java_opts: Option<String>,
    pub work_dir: String,
    #[serde(skip_serializing_if = "is_false")]
    pub verbose: bool,
    pub log_level: LogLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truststore_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truststore_password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keystore_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keystore_password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_encoding: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scanner_version: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub scanner_properties: BTreeMap<String, String>,
    #[serde(skip)]
    pub dump: bool,
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

pub fn resolve_os_alias(os: &str) -> &str {
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
}

pub fn resolve_arch_alias(arch: &str) -> &str {
    match arch.to_lowercase().as_str() {
        "x64" | "x86_64" | "x86-64" | "amd64" => "x64",
        "aarch64" | "arm64" => "aarch64",
        _ => arch,
    }
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

// Extract the value for --prefix=VALUE or --prefix VALUE.
// For the space form, increments *i to consume the value arg (but only if value doesn't start with --).
fn arg_value(arg: &str, prefix: &str, args: &[String], i: &mut usize) -> Option<String> {
    let eq_prefix = format!("{prefix}=");
    if let Some(v) = arg.strip_prefix(&eq_prefix) {
        Some(v.to_string())
    } else if arg == prefix {
        let next = args.get(*i + 1);
        if let Some(v) = next.filter(|v| !v.starts_with("--")) {
            *i += 1;
            Some(v.clone())
        } else {
            None
        }
    } else {
        None
    }
}

pub fn parse_options(
    args: &[String],
    env_fn: &dyn Fn(&str) -> Option<String>,
) -> Result<ScannerOptions, String> {
    let url_from_env = env_fn("SONAR_HOST_URL");
    let mut url_explicit = url_from_env.is_some();
    let mut url = url_from_env.unwrap_or_else(|| DEFAULT_URL.to_string());

    let mut dir_str: Option<String> = env_fn("SONAR_BASE_DIR");
    let mut dir_explicit = dir_str.is_some();

    let mut token: Option<String> = env_fn("SONAR_TOKEN");
    let mut key: Option<String> = env_fn("SONAR_PROJECT_KEY");
    let mut name: Option<String> = env_fn("SONAR_PROJECT_NAME");
    let mut version: Option<String> = env_fn("SONAR_PROJECT_VERSION");
    let mut description: Option<String> = env_fn("SONAR_PROJECT_DESCRIPTION");
    let mut build_number: Option<String> = env_fn("SONAR_PROJECT_BUILD_NUMBER");
    let mut sources: Option<String> = env_fn("SONAR_PROJECT_SOURCES");
    let mut tests: Option<String> = env_fn("SONAR_PROJECT_TESTS");

    let os_from_env = env_fn("SONAR_SCANNER_OS");
    let mut os_explicit = os_from_env.is_some();
    let mut os: String = match os_from_env {
        Some(v) => v,
        None => default_os()?.to_owned(),
    };

    let arch_from_env = env_fn("SONAR_SCANNER_ARCH");
    let mut arch_explicit = arch_from_env.is_some();
    let mut arch: String = match arch_from_env {
        Some(v) => v,
        None => default_arch()?.to_owned(),
    };

    let mut skip_jre_provisioning = env_fn("SONAR_SCANNER_SKIP_JRE_PROVISIONING")
        .map(|v| v == "true")
        .unwrap_or(false);
    let mut java_exe_path: Option<PathBuf> =
        env_fn("SONAR_SCANNER_JAVA_EXE_PATH").map(PathBuf::from);
    let mut proxy_host: Option<String> = env_fn("SONAR_SCANNER_PROXY_HOST");
    let mut proxy_port: Option<u16> = env_fn("SONAR_SCANNER_PROXY_PORT")
        .map(|v| {
            v.parse::<u16>()
                .map_err(|_| format!("invalid proxy port: {v}"))
        })
        .transpose()?;
    let mut proxy_user: Option<String> = env_fn("SONAR_SCANNER_PROXY_USER");
    let mut proxy_password: Option<String> = env_fn("SONAR_SCANNER_PROXY_PASSWORD");

    let sonar_home_from_env = env_fn("SONAR_USER_HOME");
    let mut sonar_home_explicit = sonar_home_from_env.is_some();
    let mut sonar_home_str: Option<String> = sonar_home_from_env;

    let mut java_opts: Option<String> = env_fn("SONAR_SCANNER_JAVA_OPTS");
    let work_dir_from_env = env_fn("SONAR_SCANNER_WORK_DIR");
    let mut work_dir_explicit = work_dir_from_env.is_some();
    let mut work_dir = work_dir_from_env.unwrap_or_else(|| DEFAULT_WORK_DIR.to_string());
    let mut verbose = false;
    let log_level_from_env = env_fn("SONAR_LOG_LEVEL");
    let mut log_level_explicit = log_level_from_env.is_some();
    let mut log_level = log_level_from_env
        .map(|v| LogLevel::parse(&v))
        .transpose()?
        .unwrap_or(LogLevel::INFO);
    let mut truststore_path: Option<String> = None;
    let mut truststore_password: Option<String> = None;
    let mut keystore_path: Option<String> = None;
    let mut keystore_password: Option<String> = None;
    let mut source_encoding: Option<String> = None;
    let mut scanner_version: Option<String> = None;
    let mut dump = false;
    let mut extra_properties: BTreeMap<String, String> = BTreeMap::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();

        if arg == "--dump" {
            dump = true;
        } else if arg == "--verbose" {
            verbose = true;
        } else if arg == "--skip-jre-provisioning" {
            skip_jre_provisioning = true;
        } else if let Some(v) = arg_value(arg, "--dir", args, &mut i) {
            dir_str = Some(v);
            dir_explicit = true;
        } else if let Some(v) = arg_value(arg, "--url", args, &mut i) {
            url = v;
            url_explicit = true;
        } else if let Some(v) = arg_value(arg, "--token", args, &mut i) {
            token = Some(v);
        } else if let Some(v) = arg_value(arg, "--key", args, &mut i) {
            key = Some(v);
        } else if let Some(v) = arg_value(arg, "--name", args, &mut i) {
            name = Some(v);
        } else if let Some(v) = arg_value(arg, "--version", args, &mut i) {
            version = Some(v);
        } else if let Some(v) = arg_value(arg, "--description", args, &mut i) {
            description = Some(v);
        } else if let Some(v) = arg_value(arg, "--build-number", args, &mut i) {
            build_number = Some(v);
        } else if let Some(v) = arg_value(arg, "--sources", args, &mut i) {
            sources = Some(v);
        } else if let Some(v) = arg_value(arg, "--tests", args, &mut i) {
            tests = Some(v);
        } else if let Some(v) = arg_value(arg, "--os", args, &mut i) {
            os = resolve_os_alias(&v).to_owned();
            os_explicit = true;
        } else if let Some(v) = arg_value(arg, "--arch", args, &mut i) {
            arch = resolve_arch_alias(&v).to_owned();
            arch_explicit = true;
        } else if let Some(v) = arg_value(arg, "--java-exe-path", args, &mut i) {
            java_exe_path = Some(PathBuf::from(v));
        } else if let Some(v) = arg_value(arg, "--proxy-host", args, &mut i) {
            proxy_host = Some(v);
        } else if let Some(v) = arg_value(arg, "--proxy-port", args, &mut i) {
            proxy_port = Some(
                v.parse::<u16>()
                    .map_err(|_| format!("invalid proxy port: {v}"))?,
            );
        } else if let Some(v) = arg_value(arg, "--proxy-user", args, &mut i) {
            proxy_user = Some(v);
        } else if let Some(v) = arg_value(arg, "--proxy-password", args, &mut i) {
            proxy_password = Some(v);
        } else if let Some(v) = arg_value(arg, "--sonar-home", args, &mut i) {
            sonar_home_str = Some(v);
            sonar_home_explicit = true;
        } else if let Some(v) = arg_value(arg, "--java-opts", args, &mut i) {
            java_opts = Some(v);
        } else if let Some(v) = arg_value(arg, "--work-dir", args, &mut i) {
            work_dir = v;
            work_dir_explicit = true;
        } else if let Some(v) = arg_value(arg, "--log", args, &mut i) {
            log_level = LogLevel::parse(&v)?;
            log_level_explicit = true;
        } else if let Some(v) = arg_value(arg, "--truststore-path", args, &mut i) {
            truststore_path = Some(v);
        } else if let Some(v) = arg_value(arg, "--truststore-password", args, &mut i) {
            truststore_password = Some(v);
        } else if let Some(v) = arg_value(arg, "--keystore-path", args, &mut i) {
            keystore_path = Some(v);
        } else if let Some(v) = arg_value(arg, "--keystore-password", args, &mut i) {
            keystore_password = Some(v);
        } else if let Some(v) = arg_value(arg, "--encoding", args, &mut i) {
            source_encoding = Some(v);
        } else if let Some(v) = arg_value(arg, "--scanner-version", args, &mut i) {
            scanner_version = Some(v);
        } else if arg.starts_with("--") {
            let rest = &arg[2..];
            if let Some(eq) = rest.find('=') {
                let prop_key = format!("sonar.{}", rest[..eq].replace('-', "."));
                let prop_val = rest[eq + 1..].to_string();
                extra_properties.insert(prop_key, prop_val);
            } else {
                let next = args.get(i + 1).filter(|v| !v.starts_with("--"));
                match next {
                    Some(v) => {
                        let prop_key = format!("sonar.{}", rest.replace('-', "."));
                        extra_properties.insert(prop_key, v.clone());
                        i += 1;
                    }
                    None => return Err(format!("missing value for argument: {arg}")),
                }
            }
        } else if let Some(darg) = arg.strip_prefix("-D") {
            if let Some(eq) = darg.find('=') {
                let prop = &darg[..eq];
                let val = darg[eq + 1..].to_string();
                match prop {
                    "sonar.token" => token = Some(val),
                    "sonar.projectBaseDir" => { dir_str = Some(val); dir_explicit = true; }
                    "sonar.host.url" => { url = val; url_explicit = true; }
                    "sonar.projectKey" => key = Some(val),
                    "sonar.projectName" => name = Some(val),
                    "sonar.projectVersion" => version = Some(val),
                    "sonar.projectDescription" => description = Some(val),
                    "sonar.analysis.buildNumber" => build_number = Some(val),
                    "sonar.sources" => sources = Some(val),
                    "sonar.tests" => tests = Some(val),
                    "sonar.scanner.os" => { os = resolve_os_alias(&val).to_owned(); os_explicit = true; }
                    "sonar.scanner.arch" => { arch = resolve_arch_alias(&val).to_owned(); arch_explicit = true; }
                    "sonar.scanner.skipJreProvisioning" => skip_jre_provisioning = val == "true",
                    "sonar.scanner.javaExePath" => java_exe_path = Some(PathBuf::from(val)),
                    "sonar.scanner.proxyHost" => proxy_host = Some(val),
                    "sonar.scanner.proxyPort" => proxy_port = Some(
                        val.parse::<u16>().map_err(|_| format!("invalid proxy port: {val}"))?,
                    ),
                    "sonar.scanner.proxyUser" => proxy_user = Some(val),
                    "sonar.scanner.proxyPassword" => proxy_password = Some(val),
                    "sonar.userHome" => { sonar_home_str = Some(val); sonar_home_explicit = true; }
                    "sonar.scanner.javaOpts" => java_opts = Some(val),
                    "sonar.working.directory" => { work_dir = val; work_dir_explicit = true; }
                    "sonar.verbose" => verbose = val == "true",
                    "sonar.log.level" => { log_level = LogLevel::parse(&val)?; log_level_explicit = true; }
                    "sonar.scanner.truststorePath" => truststore_path = Some(val),
                    "sonar.scanner.truststorePassword" => truststore_password = Some(val),
                    "sonar.scanner.keystorePath" => keystore_path = Some(val),
                    "sonar.scanner.keystorePassword" => keystore_password = Some(val),
                    "sonar.sourceEncoding" => source_encoding = Some(val),
                    _ => { extra_properties.insert(prop.to_string(), val); }
                }
            } else {
                return Err(format!("missing '=' in argument: {arg}"));
            }
        } else {
            return Err(format!("unexpected argument: {arg}"));
        }

        i += 1;
    }

    let dir = match dir_str {
        Some(ref s) => PathBuf::from(s)
            .canonicalize()
            .map_err(|_| format!("Project path does not exist: {s}"))?,
        None => env::current_dir().map_err(|e| e.to_string())?,
    };

    let sonar_home = match sonar_home_str {
        Some(ref s) => PathBuf::from(s),
        None => {
            let home = env_fn("HOME")
                .or_else(|| env_fn("USERPROFILE"))
                .ok_or_else(|| "User HOME directory not found".to_string())?;
            PathBuf::from(home).join(".sonar")
        }
    };

    let sonar_cache = sonar_home.join("cache");

    let mut scanner_properties = extra_properties;

    if dir_explicit {
        scanner_properties.insert(
            "sonar.projectBaseDir".to_string(),
            dir.to_string_lossy().into_owned(),
        );
    }
    if url_explicit {
        scanner_properties.insert("sonar.host.url".to_string(), url.clone());
    }
    if let Some(ref v) = token {
        scanner_properties.insert("sonar.token".to_string(), v.clone());
    }
    if let Some(ref v) = key {
        scanner_properties.insert("sonar.projectKey".to_string(), v.clone());
    }
    if let Some(ref v) = name {
        scanner_properties.insert("sonar.projectName".to_string(), v.clone());
    }
    if let Some(ref v) = version {
        scanner_properties.insert("sonar.projectVersion".to_string(), v.clone());
    }
    if let Some(ref v) = description {
        scanner_properties.insert("sonar.projectDescription".to_string(), v.clone());
    }
    if let Some(ref v) = build_number {
        scanner_properties.insert("sonar.analysis.buildNumber".to_string(), v.clone());
    }
    if let Some(ref v) = sources {
        scanner_properties.insert("sonar.sources".to_string(), v.clone());
    }
    if let Some(ref v) = tests {
        scanner_properties.insert("sonar.tests".to_string(), v.clone());
    }
    if os_explicit {
        scanner_properties.insert("sonar.scanner.os".to_string(), os.clone());
    }
    if arch_explicit {
        scanner_properties.insert("sonar.scanner.arch".to_string(), arch.clone());
    }
    if skip_jre_provisioning {
        scanner_properties.insert(
            "sonar.scanner.skipJreProvisioning".to_string(),
            "true".to_string(),
        );
    }
    if let Some(ref v) = java_exe_path {
        scanner_properties.insert(
            "sonar.scanner.javaExePath".to_string(),
            v.to_string_lossy().into_owned(),
        );
    }
    if let Some(ref v) = proxy_host {
        scanner_properties.insert("sonar.scanner.proxyHost".to_string(), v.clone());
    }
    if let Some(v) = proxy_port {
        scanner_properties.insert("sonar.scanner.proxyPort".to_string(), v.to_string());
    }
    if let Some(ref v) = proxy_user {
        scanner_properties.insert("sonar.scanner.proxyUser".to_string(), v.clone());
    }
    if let Some(ref v) = proxy_password {
        scanner_properties.insert("sonar.scanner.proxyPassword".to_string(), v.clone());
    }
    if sonar_home_explicit {
        scanner_properties.insert(
            "sonar.userHome".to_string(),
            sonar_home.to_string_lossy().into_owned(),
        );
    }
    if let Some(ref v) = java_opts {
        scanner_properties.insert("sonar.scanner.javaOpts".to_string(), v.clone());
    }
    if work_dir_explicit {
        scanner_properties.insert("sonar.working.directory".to_string(), work_dir.clone());
    }
    if verbose {
        scanner_properties.insert("sonar.verbose".to_string(), "true".to_string());
    }
    if log_level_explicit {
        scanner_properties.insert("sonar.log.level".to_string(), log_level.to_string());
    }
    if let Some(ref v) = truststore_path {
        scanner_properties.insert("sonar.scanner.truststorePath".to_string(), v.clone());
    }
    if let Some(ref v) = truststore_password {
        scanner_properties.insert("sonar.scanner.truststorePassword".to_string(), v.clone());
    }
    if let Some(ref v) = keystore_path {
        scanner_properties.insert("sonar.scanner.keystorePath".to_string(), v.clone());
    }
    if let Some(ref v) = keystore_password {
        scanner_properties.insert("sonar.scanner.keystorePassword".to_string(), v.clone());
    }
    if let Some(ref v) = source_encoding {
        scanner_properties.insert("sonar.sourceEncoding".to_string(), v.clone());
    }

    Ok(ScannerOptions {
        dir,
        url,
        token,
        key,
        name,
        version,
        description,
        build_number,
        sources,
        tests,
        os,
        arch,
        skip_jre_provisioning,
        java_exe_path,
        proxy_host,
        proxy_port,
        proxy_user,
        proxy_password,
        sonar_home,
        sonar_cache,
        java_opts,
        work_dir,
        verbose,
        log_level,
        truststore_path,
        truststore_password,
        keystore_path,
        keystore_password,
        source_encoding,
        scanner_version,
        scanner_properties,
        dump,
    })
}

impl ScannerOptions {
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    fn fixed_env(k: &str) -> Option<String> {
        match k {
            "HOME" => Some("/home/user".to_string()),
            _ => None,
        }
    }

    fn cur_dir() -> String {
        env::current_dir().unwrap().to_string_lossy().into_owned()
    }

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

    #[test]
    fn parse_options_defaults() {
        let dir = cur_dir();
        let os = default_os().unwrap();
        let arch = default_arch().unwrap();
        let options = parse_options(&[], &fixed_env).unwrap();
        assert_eq!(options.to_json(), indoc! {r#"
        {
          "dir": "{dir}",
          "url": "https://sonarcloud.io",
          "os": "{os}",
          "arch": "{arch}",
          "sonar_home": "/home/user/.sonar",
          "sonar_cache": "/home/user/.sonar/cache",
          "work_dir": ".scannerwork",
          "log_level": "INFO"
        }"#}.replace("{dir}", &dir).replace("{os}", &os).replace("{arch}", &arch));
    }

    #[test]
    fn parse_options_eq_form() {
        let dir = cur_dir();
        let os = default_os().unwrap();
        let arch = default_arch().unwrap();
        let options = parse_options(
            &[
                format!("--dir={dir}"),
                "--url=http://localhost:9000".to_string(),
                "--token=sqa_1234567890".to_string(),
                "--key=my-project".to_string(),
                "--log=DEBUG".to_string(),
            ],
            &fixed_env,
        )
        .unwrap();
        assert_eq!(options.to_json(), indoc! {r#"
        {
          "dir": "{dir}",
          "url": "http://localhost:9000",
          "token": "sqa_1234567890",
          "key": "my-project",
          "os": "{os}",
          "arch": "{arch}",
          "sonar_home": "/home/user/.sonar",
          "sonar_cache": "/home/user/.sonar/cache",
          "work_dir": ".scannerwork",
          "log_level": "DEBUG",
          "scanner_properties": {
            "sonar.host.url": "http://localhost:9000",
            "sonar.log.level": "DEBUG",
            "sonar.projectBaseDir": "{dir}",
            "sonar.projectKey": "my-project",
            "sonar.token": "sqa_1234567890"
          }
        }"#}.replace("{dir}", &dir).replace("{os}", &os).replace("{arch}", &arch));
    }

    #[test]
    fn parse_options_space_form() {
        let dir = cur_dir();
        let os = default_os().unwrap();
        let arch = default_arch().unwrap();
        let options = parse_options(
            &[
                "--token".to_string(),
                "mytoken".to_string(),
                "--dir".to_string(),
                dir.clone(),
            ],
            &fixed_env,
        )
        .unwrap();
        assert_eq!(options.to_json(), indoc! {r#"
        {
          "dir": "{dir}",
          "url": "https://sonarcloud.io",
          "token": "mytoken",
          "os": "{os}",
          "arch": "{arch}",
          "sonar_home": "/home/user/.sonar",
          "sonar_cache": "/home/user/.sonar/cache",
          "work_dir": ".scannerwork",
          "log_level": "INFO",
          "scanner_properties": {
            "sonar.projectBaseDir": "{dir}",
            "sonar.token": "mytoken"
          }
        }"#}.replace("{dir}", &dir).replace("{os}", &os).replace("{arch}", &arch));
    }

    #[test]
    fn parse_options_unknown_arg_becomes_sonar_property() {
        let dir = cur_dir();
        let os = default_os().unwrap();
        let arch = default_arch().unwrap();
        let options =
            parse_options(&["--some-custom-prop=hello".to_string()], &fixed_env).unwrap();
        assert_eq!(options.to_json(), indoc! {r#"
        {
          "dir": "{dir}",
          "url": "https://sonarcloud.io",
          "os": "{os}",
          "arch": "{arch}",
          "sonar_home": "/home/user/.sonar",
          "sonar_cache": "/home/user/.sonar/cache",
          "work_dir": ".scannerwork",
          "log_level": "INFO",
          "scanner_properties": {
            "sonar.some.custom.prop": "hello"
          }
        }"#}.replace("{dir}", &dir).replace("{os}", &os).replace("{arch}", &arch));
    }

    #[test]
    fn parse_options_verbose_adds_property() {
        let dir = cur_dir();
        let os = default_os().unwrap();
        let arch = default_arch().unwrap();
        let options = parse_options(&["--verbose".to_string()], &fixed_env).unwrap();
        assert_eq!(options.to_json(), indoc! {r#"
        {
          "dir": "{dir}",
          "url": "https://sonarcloud.io",
          "os": "{os}",
          "arch": "{arch}",
          "sonar_home": "/home/user/.sonar",
          "sonar_cache": "/home/user/.sonar/cache",
          "work_dir": ".scannerwork",
          "verbose": true,
          "log_level": "INFO",
          "scanner_properties": {
            "sonar.verbose": "true"
          }
        }"#}.replace("{dir}", &dir).replace("{os}", &os).replace("{arch}", &arch));
    }

    #[test]
    fn parse_options_skip_jre_provisioning() {
        let dir = cur_dir();
        let os = default_os().unwrap();
        let arch = default_arch().unwrap();
        let options =
            parse_options(&["--skip-jre-provisioning".to_string()], &fixed_env).unwrap();
        assert_eq!(options.to_json(), indoc! {r#"
        {
          "dir": "{dir}",
          "url": "https://sonarcloud.io",
          "os": "{os}",
          "arch": "{arch}",
          "skip_jre_provisioning": true,
          "sonar_home": "/home/user/.sonar",
          "sonar_cache": "/home/user/.sonar/cache",
          "work_dir": ".scannerwork",
          "log_level": "INFO",
          "scanner_properties": {
            "sonar.scanner.skipJreProvisioning": "true"
          }
        }"#}.replace("{dir}", &dir).replace("{os}", &os).replace("{arch}", &arch));
    }

    #[test]
    fn parse_options_env_vars_override_defaults() {
        let dir = cur_dir();
        let os = default_os().unwrap();
        let arch = default_arch().unwrap();
        let options = parse_options(&[], &|k| match k {
            "SONAR_HOST_URL" => Some("http://my-sonar:9000".to_string()),
            "SONAR_TOKEN" => Some("envtoken".to_string()),
            "HOME" => Some("/home/user".to_string()),
            _ => None,
        })
        .unwrap();
        assert_eq!(options.to_json(), indoc! {r#"
        {
          "dir": "{dir}",
          "url": "http://my-sonar:9000",
          "token": "envtoken",
          "os": "{os}",
          "arch": "{arch}",
          "sonar_home": "/home/user/.sonar",
          "sonar_cache": "/home/user/.sonar/cache",
          "work_dir": ".scannerwork",
          "log_level": "INFO",
          "scanner_properties": {
            "sonar.host.url": "http://my-sonar:9000",
            "sonar.token": "envtoken"
          }
        }"#}.replace("{dir}", &dir).replace("{os}", &os).replace("{arch}", &arch));
    }

    #[test]
    fn to_json_omits_none_and_false_fields() {
        let options = parse_options(
            &["--dump".to_string(), "--token=secret".to_string()],
            &fixed_env,
        )
        .unwrap();
        assert!(options.dump);
        let json = options.to_json();
        assert!(json.contains("\"dir\""));
        assert!(json.contains("\"url\""));
        assert!(json.contains("\"token\": \"secret\""));
        assert!(json.contains("\"os\""));
        assert!(json.contains("\"arch\""));
        assert!(json.contains("\"sonar_home\""));
        assert!(json.contains("\"sonar_cache\""));
        assert!(json.contains("\"work_dir\""));
        assert!(json.contains("\"log_level\""));
        assert!(!json.contains("\"key\""));
        assert!(!json.contains("\"verbose\""));
        assert!(!json.contains("\"skip_jre_provisioning\""));
    }

    #[test]
    fn to_json_includes_scanner_properties_when_set() {
        let options = parse_options(
            &[
                "--url=http://localhost:9000".to_string(),
                "--token=tok".to_string(),
            ],
            &fixed_env,
        )
        .unwrap();
        let json = options.to_json();
        assert!(json.contains("\"scanner_properties\""));
        assert!(json.contains("\"sonar.host.url\""));
        assert!(json.contains("\"sonar.token\""));
    }
}
