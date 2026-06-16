# sonar-scan

A simple command-line tool to run SonarQube analysis on any project without modifying your build configuration.

No Sonar Maven plugin, Gradle plugin, or project-level setup required. Just download the binary and run it.

# Building the project

## Requirements

- A running [SonarQube](https://www.sonarsource.com/products/sonarqube/) instance
- SonarQube project credentials (project key, token, and server URL)

To build from source:

- [Rust](https://www.rust-lang.org/tools/install) (edition 2024, Rust 1.85+)
- [Zig](https://ziglang.org/download/) + [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild) - `brew install zig && cargo install cargo-zigbuild` - Linux targets (no Docker needed)
- [cargo-xwin](https://github.com/rust-cross/cargo-xwin) - `cargo install cargo-xwin` - Windows MSVC targets (downloads the Windows SDK, works on Linux/macOS)
- [LLVM](https://llvm.org/) - `brew install llvm` - required by `cargo-xwin` for Windows targets on macOS
- macOS targets require a macOS host (native `cargo`, no extra tools needed)

## Build instructions

Download the latest version of the sonar-scanner-cli jar:
```bash
./download-sonar-scanner.sh
```

Or a specific version:
```bash
./download-sonar-scanner.sh 8.1.0.6389
```

To build for the current host only:

```bash
./build.sh
```

The binary will be available at `target/release/sonar-scan`.

To build release binaries for all supported platforms (Linux, Windows, macOS × x86\_64 and aarch64), use the provided `build-dist.sh` script:

```bash
./build-dist.sh
```

Binaries are written to the `target/dist/` directory:

| File | Platform |
|---|---|
| `sonar-scan-x86_64-linux` | Linux x86\_64 (static musl) |
| `sonar-scan-aarch64-linux` | Linux aarch64 (static musl) |
| `sonar-scan-x86_64-windows.exe` | Windows x86\_64 |
| `sonar-scan-aarch64-windows.exe` | Windows aarch64 |
| `sonar-scan-x86_64-macos` | macOS Intel |
| `sonar-scan-aarch64-macos` | macOS Apple Silicon |

Integration tests can be run with:

```bash
./integration-tests.sh
```

## Usage

If `sonar-scan` is placed in the root of the project you want to analyze, run it from there:

```bash
./sonar-scan
```

If `sonar-scan` is located elsewhere, pass the project directory as an argument:

```bash
./sonar-scan --dir /path/to/project
```

## License

This project is licensed under the [GNU Lesser General Public License v3.0](LICENSE).
