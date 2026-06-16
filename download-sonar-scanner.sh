#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

function download_scanner() {
  local SCANNER_VERSION="${1:-}"
  local METADATA_URL="https://repox.jfrog.io/artifactory/sonarsource-public-releases/org/sonarsource/scanner/cli/sonar-scanner-cli/maven-metadata.xml"

  if [[ "${SCANNER_VERSION}" == "list" ]]; then
    echo "List sonar-scanner-cli versions from: ${METADATA_URL}"
    curl -sSLf -o - -u "${ARTIFACTORY_USER}:${ARTIFACTORY_PASSWORD}" "${METADATA_URL}"  | grep -oP '(?<=<version>)[^<]+' | sort --unique --version-sort
    return 0
  fi

  if [[ -z "${SCANNER_VERSION}" ]]; then
    SCANNER_VERSION="$(curl -sSLf -o - -u "${ARTIFACTORY_USER}:${ARTIFACTORY_PASSWORD}" "${METADATA_URL}"  |  grep -oP '(?<=<release>)[^<]+')"
    if [[ -z "${SCANNER_VERSION}" ]]; then
      echo "Failed to determine sonar-scanner-cli's latest release from ${METADATA_URL}"
      return 1
    fi
    echo "Downloading sonar-scanner-cli version: ${SCANNER_VERSION} (latest release)"
  else
    echo "Downloading sonar-scanner-cli version: ${SCANNER_VERSION}"
  fi

  local SCANNER_DIR="${SCRIPT_DIR}/resources/sonar-scanner-cli"
  if [[ -d "${SCANNER_DIR}" ]]; then
    echo "Removing    : ${SCANNER_DIR}"
    rm -rf "${SCANNER_DIR}"
  fi

  mkdir -p "${SCANNER_DIR}"

  local JAR_URL="https://repox.jfrog.io/artifactory/sonarsource-public-releases/org/sonarsource/scanner/cli/sonar-scanner-cli/${SCANNER_VERSION}/sonar-scanner-cli-${SCANNER_VERSION}.jar"
  local JAR_PATH="${SCANNER_DIR}/sonar-scanner-cli.jar"
  local SHA_PATH="${JAR_PATH}.sha256.txt"
  local VERSION_PATH="${JAR_PATH}.version.txt"

  echo "Source      : ${JAR_URL}"
  echo "Destination : ${JAR_PATH}"
  curl -sSLf -o "${JAR_PATH}" -u "${ARTIFACTORY_USER}:${ARTIFACTORY_PASSWORD}" "${JAR_URL}"
  shasum -a 256 "${JAR_PATH}" | awk '{printf "%s", $1}' > "${SHA_PATH}"
  echo "sha256      : $(cat "${SHA_PATH}")"
  echo -n "${SCANNER_VERSION}" > "${VERSION_PATH}"
  echo "version     : $(cat "${VERSION_PATH}")"
}

download_scanner "$@"
