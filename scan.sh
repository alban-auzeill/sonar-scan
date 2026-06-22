#!/bin/sh
set -eu

SONAR_SCAN_VERSION="1.5.0"

fn_scan() {

  case "$(uname -m)" in
      x86_64|x86-64|amd64) _arch="x86_64" ;;
            aarch64|arm64) _arch="aarch64" ;;
                    s390x) _arch="s390x" ;;
                        *) printf 'Unsupported architecture: %s\n' "$(uname -m)" >&2; exit 1 ;;
  esac

  case "$(uname -s)" in
                    Darwin) _os_suffix="macos" ;;
                     Linux) _os_suffix="linux" ;;
      CYGWIN*|MINGW*|MSYS*) _os_suffix="windows.exe" ;;
                    OS/390) _os_suffix="zos" ;;
                         *) printf 'Unsupported operating system: %s\n' "$(uname -s)" >&2; exit 1 ;;
  esac

  if [ "${1:-}" = "--install" ] && [ ! -d "${2:-}" ]; then
    echo "Error: --install need an existing directory as argument: ${2:-}" >&2
    return 1
  elif [ "${1:-}" = "--install" ]; then
    _destination_dir="$(cd -- "${2:-}" && pwd)"
    echo "Installing sonar-scan into ${_destination_dir}"
    _binary="${_destination_dir}/sonar-scan"
  else
    _binary="${HOME}/.sonar/cache/sonar-scan-${SONAR_SCAN_VERSION}/sonar-scan-${_arch}-${_os_suffix}"
    if [ -e "${_binary}" ]; then
      exec "${_binary}" "$@"
      return "$?"
    fi
    mkdir -p "${HOME}/.sonar/cache/sonar-scan-${SONAR_SCAN_VERSION}"
  fi

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

  if [ "${1:-}" = "--install" ]; then
     return 0
  else
     exec "${_binary}" "$@"
     return "$?"
  fi
}

fn_scan "$@"
