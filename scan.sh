#!/bin/sh
set -eu

SCRIPT_DIR="$(cd -- "$(dirname -- "$0")" && pwd)"
SONAR_SCAN_VERSION="1.0.0"

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

  _binary="${SCRIPT_DIR}/target/dist/sonar-scan-${_arch}-${_os_suffix}"
  if [ ! -e "${_binary}" ]; then
      _binary="${HOME}/.sonar/cache/sonar-scan-${SONAR_SCAN_VERSION}/sonar-scan-${_arch}-${_os_suffix}"
    if [ ! -e "${_binary}" ]; then
      mkdir -p "${HOME}/.sonar/cache/sonar-scan-${SONAR_SCAN_VERSION}"
      _url="https://github.com/alban-auzeill/sonar-scan/releases/download/v${SONAR_SCAN_VERSION}/sonar-scan-${_arch}-${_os_suffix}"
      if command -v curl >/dev/null 2>&1; then
          if ! curl -sSLf -o "${_binary}" "${_url}"; then
              echo "Error: curl failed to download ${_url} into ${_binary}" >&2
              rm -f "${_binary}"
              return 1
          fi
      elif command -v wget >/dev/null 2>&1; then
          if ! wget -q -O "${_binary}" --max-redirect=20 "${_url}"; then
              echo "Error: wget failed to download ${_url} into ${_binary}" >&2
              rm -f "${_binary}"
              return 1
          fi
      else
          echo "Error: Neither curl nor wget found." >&2
          return 1
      fi
      if [ ! -s "${_binary}" ]; then
          echo "Error: ${_binary} not found or empty" >&2
          rm -f "${_binary}"
          return 1
      fi
      chmod +x "${_binary}"
    fi
  fi

  if [ "${1:-}" = "--install" ] && [ ! -d "${2:-}" ]; then
    echo "Error: --install need an existing directory as argument: ${2:-}" >&2
    return 1
  elif [ "${1:-}" = "--install" ]; then
    _installed_binary="$(cd -- "${2:-}" && pwd)/sonar-scan"
    cp "${_binary}" "${_installed_binary}"
    echo "Installed successfully in ${_installed_binary}"
    return 0
  else
    exec "${_binary}" "$@"
  fi
}

fn_scan "$@"