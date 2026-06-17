# sonar-scan

`sonar-scan` is a native Rust wrapper around the [SonarScanner CLI](https://docs.sonarsource.com/sonarqube-server/analyzing-source-code/scanners/sonarscanner) that allows you
to run SonarQube analysis on any project without modifying your build configuration, without installing Java or downloading the SonarScanner CLI with a preinstalled JVM.

Benefit:

* Provided with portable scan.sh and scan.cmd scripts that automatically download the sonar-scan binary for your platform. Use '${HOME}/.sonar/cache' to cache the binary for future runs.
* Extract the SonarScanner CLI jar from the sonar-scan binary itself.
* Download the Java Runtime Environment (JRE) from the targeted SonarQube Server/Cloud. Use '${HOME}/.sonar/cache' to cache the JRE for future runs and SonarScanner CLI will not have to download it again.
* Requires the smallest possible configuration to be able to benefit from a basic SonarQube analysis without having to configure a Maven, Gradle, npm, or other build system plugin.
* Display complete usage syntax and options with `--help`.
* The only require parameters are the security token (--token) and the server URL if it is different from https://sonarcloud.io

## Execution without installation

### Analyze the current directory

#### URL for Linux/macOS

Source | URL
--- | ---
Latest Release (GitHub Pages) | https://alban-auzeill.github.io/sonar-scan
Latest Release (GitHub Releases) | https://github.com/alban-auzeill/sonar-scan/releases/latest/download/scan.sh
A specific version | https://github.com/alban-auzeill/sonar-scan/releases/download/v1.0.0/scan.sh

#### URL for Windows

Source | URL
--- | ---
Latest Release (GitHub Pages) | https://alban-auzeill.github.io/sonar-scan/scan.cmd
Latest Release (GitHub Releases) | https://github.com/alban-auzeill/sonar-scan/releases/latest/download/scan.cmd
A specific version | https://github.com/alban-auzeill/sonar-scan/releases/download/v1.0.0/scan.cmd

#### Download tools

curl or wget can be used to download the script:

* curl -sSfL "<url>"
* wget -qO- "<url>"

#### Execution

sh or bash can be used for scan.sh:

* curl -sSfL "<url>" | sh
* curl -sSfL "<url>" | sh -s -- ... arguments ...
* curl -sSfL "<url>" | bash
* curl -sSfL "<url>" | bash -s -- ... arguments ...

#### Linux / macOS — execute via pipe

```bash
# Display help
curl -sSfL "https://alban-auzeill.github.io/sonar-scan" | bash -s -- --help

# Analyze using environment variables
export SONAR_TOKEN="sqa_0123456789001234567890"
export SONAR_HOST_URL="http://localhost:9000"
curl -sSfL "https://alban-auzeill.github.io/sonar-scan" | sh

# Analyze using command-line arguments
curl -sSfL "https://alban-auzeill.github.io/sonar-scan" | bash -s -- --token "sqa_0123456789001234567890" --url "http://localhost:9000"
```

#### Windows — execute via cmd.exe

On Windows, `scan.cmd` cannot be piped like a shell script. Download it first, then run it:

```cmd
curl -sSfL "https://alban-auzeill.github.io/sonar-scan/scan.cmd" -o scan.cmd

rem Display help
scan.cmd --help

rem Analyze using environment variables
set SONAR_TOKEN=sqa_0123456789001234567890
set SONAR_HOST_URL=http://localhost:9000
scan.cmd

rem Analyze using command-line arguments
scan.cmd --token "sqa_0123456789001234567890" --url "http://localhost:9000"
```

`scan.cmd` caches `sonar-scan.exe` in `%USERPROFILE%\.sonar\cache` on first run, so subsequent invocations are instant.

#### Windows — execute via PowerShell

```powershell
curl -sSfL "https://alban-auzeill.github.io/sonar-scan/scan.cmd" -o scan.cmd

$env:SONAR_TOKEN = "sqa_0123456789001234567890"
$env:SONAR_HOST_URL = "http://localhost:9000"
.\scan.cmd
```

### Installing sonar-scan in a directory

It is better to install into a directory that is already on the `PATH`.

#### Linux / macOS

```bash
curl -sSfL "https://alban-auzeill.github.io/sonar-scan" | bash -s -- --install "$HOME/bin"

sonar-scan --version
```

#### Windows (cmd.exe)

```cmd
curl -sSfL "https://alban-auzeill.github.io/sonar-scan/scan.cmd" -o scan.cmd
scan.cmd --install "%USERPROFILE%\bin"

sonar-scan --version
```

#### Windows (PowerShell)

```powershell
curl -sSfL "https://alban-auzeill.github.io/sonar-scan/scan.cmd" -o scan.cmd
.\scan.cmd --install "$env:USERPROFILE\bin"

sonar-scan --version
```

## License

This project is licensed under the [GNU Lesser General Public License v3.0](LICENSE).
