#!/usr/bin/env bash
set -euo pipefail

main() {
  cargo test integration_tests --features integration-tests
}

main
