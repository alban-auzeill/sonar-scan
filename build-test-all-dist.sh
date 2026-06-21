#!/usr/bin/env bash
set -euo pipefail

main() {
  ./build.sh && ./build-dist.sh && ./integration-tests.sh
}

main
