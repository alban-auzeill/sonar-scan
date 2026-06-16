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


## Help

```bash
./sonar-scan --help
```

## Usage

```bash
export SONAR_TOKEN="sqa_012345678901234567890123456789"
export SONAR_HOST_URL="http://localhost:9000"
sonar-scan
```

OR

```bash
sonar-scan --token "sqa_012345678901234567890123456789" --url "http://localhost:9000"
```

## License

This project is licensed under the [GNU Lesser General Public License v3.0](LICENSE).
