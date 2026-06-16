#!/bin/sh
set -eu

SCRIPT_DIR="$(cd -- "$(dirname -- "$0")" && pwd)"

fn_scan() {

  case "$(uname -m)" in
      x86_64|x86-64|amd64) _arch="x86_64" ;;
            aarch64|arm64) _arch="aarch64" ;;
                        *) printf 'Unsupported architecture: %s\n' "$(uname -m)" >&2; exit 1 ;;
  esac

  case "$(uname -s)" in
                    Darwin) _os_suffix="macos" ;;
                     Linux) _os_suffix="linux" ;;
      CYGWIN*|MINGW*|MSYS*) _os_suffix="windows.exe" ;;
                         *) printf 'Unsupported operating system: %s\n' "$(uname -s)" >&2; exit 1 ;;
  esac

  # customize environment variables here

  _binary="$SCRIPT_DIR/target/dist/sonar-scan-${_arch}-${_os_suffix}"

  if [ ! -f "${_binary}" ]; then
      printf 'Binary not found: %s\n' "${_binary}" >&2
      printf 'Run ./build-dist.sh to build the binaries.\n' >&2
      exit 1
  fi

  exec "${_binary}" "$@"
}

fn_scan "$@"