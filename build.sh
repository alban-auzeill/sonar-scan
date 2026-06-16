#!/usr/bin/env bash
set -euo pipefail

main() {
  # Check Prerequisites
  if ! command -v cargo &>/dev/null || ! command -v rustup &>/dev/null; then
    echo "Error: Rust toolchain not found. Install it from https://rustup.rs"
    exit 1
  fi

  if [[ ! -e "resources/sonar-scanner-cli/sonar-scanner-cli.jar" ]]; then
    ./download-sonar-scanner.sh
  fi

  # Test and Build
  cargo test
  cargo build --release

  echo "Binary is located at target/release/sonar-scan"
}

main
