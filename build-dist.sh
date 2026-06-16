#!/usr/bin/env bash
set -euo pipefail

BINARY_NAME="sonar-scan"
DIST_DIR="target/dist"

RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
BOLD='\033[1m'
RESET='\033[0m'

check_prerequisites() {
  if ! command -v cargo &>/dev/null || ! command -v rustup &>/dev/null; then
    echo -e "${RED}Error: Rust toolchain not found. Install it from https://rustup.rs${RESET}"
    exit 1
  fi
}

# Build one target and copy the binary to dist/ with the given output name.
# Cross-compilation strategy:
#   - Linux targets  : 'cargo-zigbuild' (uses Zig as cross-linker, no Docker needed)
#   - Windows targets: 'cargo xwin' (downloads Windows SDK, works on Linux/macOS)
#   - macOS targets  : native 'cargo' (requires macOS host)
build_target() {
  local triple="$1"
  local output="$2"

  echo -e "\n${BOLD}[$triple]${RESET}"

  case "$triple" in
    *-apple-darwin*)
      if [[ "$(uname -s)" != "Darwin" ]]; then
        echo -e "${YELLOW}  Skipped: macOS targets require a macOS host${RESET}"
        return 0
      fi
      rustup target add "$triple" 2>/dev/null || true
      cargo build --release --target "$triple"
      cp "target/$triple/release/$BINARY_NAME" "$DIST_DIR/$output"
      ;;

    *-windows-*)
      if ! command -v cargo-xwin &>/dev/null; then
        echo -e "${YELLOW}  Skipped: cargo-xwin not installed (run: cargo install cargo-xwin)${RESET}"
        return 0
      fi
      # cargo-xwin uses the host toolchain + target rust-std; rustup target add
      # is enough (rustup toolchain add would install a non-runnable MSVC toolchain).
      rustup target add "$triple" 2>/dev/null || true
      # cargo-xwin passes /imsvc (MSVC-style include flags) to the C compiler.
      # Regular clang doesn't accept /imsvc; clang-cl (--driver-mode=cl) does.
      # The ring crate hard-codes "clang" as the compiler, so we put a wrapper
      # named "clang" first on PATH that transparently adds --driver-mode=cl.
      local wrapper_dir
      wrapper_dir=$(mktemp -d)
      if [[ -d /opt/homebrew/opt/llvm/bin ]]; then
        # Homebrew LLVM clang supports Windows cross-compilation (Apple clang does not).
        cat > "$wrapper_dir/clang" << 'WRAPPER'
#!/bin/bash
exec /opt/homebrew/opt/llvm/bin/clang --driver-mode=cl "$@"
WRAPPER
        export PATH="$wrapper_dir:/opt/homebrew/opt/llvm/bin:$PATH"
      else
        cat > "$wrapper_dir/clang" << 'WRAPPER'
#!/bin/bash
exec clang-real --driver-mode=cl "$@"
WRAPPER
        export PATH="$wrapper_dir:$PATH"
      fi
      chmod +x "$wrapper_dir/clang"
      cargo xwin build --release --target "$triple"
      rm -rf "$wrapper_dir"
      cp "target/$triple/release/${BINARY_NAME}.exe" "$DIST_DIR/$output"
      ;;

    *)
      rustup target add "$triple" 2>/dev/null || true
      if ! command -v cargo-zigbuild &>/dev/null; then
        echo -e "${RED}  Error: cargo-zigbuild not found${RESET}"
        echo -e "${RED}  Install with: brew install zig && cargo install cargo-zigbuild${RESET}"
        exit 1
      fi
      cargo zigbuild --release --target "$triple"
      cp "target/$triple/release/$BINARY_NAME" "$DIST_DIR/$output"
      ;;
  esac

  echo -e "${GREEN}  -> $DIST_DIR/$output${RESET}"
}

main() {
  check_prerequisites
  if [[ -e "$DIST_DIR" ]]; then
    rm -rf "$DIST_DIR"
  fi
  mkdir -p "$DIST_DIR"

  # Linux
  # There is two options for Linux targets:
  # - musl (static "x86_64-unknown-linux-musl" "aarch64-unknown-linux-musl")
  # - gnu (dynamic "x86_64-unknown-linux-gnu" "aarch64-unknown-linux-gnu")
  # The gnu target links against glibc (the GNU C Library), which is standard on Ubuntu, Debian, Fedora, and Arch. It is better for performance, dynamic loading, and compatibility
  # with native libraries. But gnu-linked binaries does not run on distros with older glibc versions (< glibc 2.39, e.g. Alpine Linux) without installing compatibility packages.
  # So we use musl target because it is more portable, performance is not a problem, and we don't need to build gnu-linked binaries.
  build_target "x86_64-unknown-linux-musl"  "sonar-scan-x86_64-linux"
  build_target "aarch64-unknown-linux-musl" "sonar-scan-aarch64-linux"

  # Windows
  build_target "x86_64-pc-windows-msvc"    "sonar-scan-x86_64-windows.exe"
  build_target "aarch64-pc-windows-msvc"   "sonar-scan-aarch64-windows.exe"

  # macOS
  build_target "x86_64-apple-darwin"       "sonar-scan-x86_64-macos"
  build_target "aarch64-apple-darwin"      "sonar-scan-aarch64-macos"

  echo -e "\n${BOLD}Artifacts in $DIST_DIR/:${RESET}"
  ls -lh "$DIST_DIR/"
}

main
