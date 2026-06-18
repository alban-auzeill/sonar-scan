#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if [ -z "${1:-}" ]; then
    echo -e "Error: missing version argument." >&2
    gh release list
    exit 1
fi

# ── Git repository state checks ──────────────────────────────────────────────
echo -e "Checking git repository state..."

BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "${BRANCH}" != "main" ]; then
    echo -e "Error: must be on branch 'main' (currently on '${BRANCH}')." >&2
    exit 1
fi
echo "  Branch         : ${BRANCH}"

if ! git diff --quiet || ! git diff --cached --quiet; then
    echo -e "Error: there are uncommitted local changes." >&2
    git status --short >&2
    exit 1
fi
echo "  Working tree   : clean"

git fetch --quiet origin main
LOCAL=$(git rev-parse HEAD)
REMOTE=$(git rev-parse origin/main)
if [ "${LOCAL}" != "${REMOTE}" ]; then
    echo -e "Error: local commits have not been pushed to origin/main." >&2
    git log origin/main..HEAD --oneline >&2
    exit 1
fi
echo "  Remote sync    : up to date"

# ── Update the version ───────────────────────────────────────────────────────

VERSION="${1}"

# Update version in Cargo.toml, scan.sh, scan.cmd, scan.ps1
sed -i '' -E "s/^version = \".*\"$/version = \"${VERSION}\"/" Cargo.toml
sed -i '' -E "s/^SONAR_SCAN_VERSION=\".*\"/SONAR_SCAN_VERSION=\"${VERSION}\"/" scan.sh
sed -i '' -E "s/SONAR_SCAN_VERSION = \".*\"$/SONAR_SCAN_VERSION = \"${VERSION}\"/" scan.ps1
sed -i '' -E "s/\"SONAR_SCAN_VERSION=.*\"$/\"SONAR_SCAN_VERSION=${VERSION}\"/" scan.cmd

./build.sh

# ── Create a commit and push ─────────────────────────────────────────────────

git add Cargo.toml Cargo.lock scan.sh scan.cmd scan.ps1
git commit -m "Bump version to ${VERSION}"
git push origin main
