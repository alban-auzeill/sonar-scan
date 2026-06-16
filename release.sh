#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
BOLD='\033[1m'
RESET='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# ── Git repository state checks ──────────────────────────────────────────────
echo -e "${BOLD}Checking git repository state...${RESET}"

BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "${BRANCH}" != "main" ]; then
    echo -e "${RED}Error: must be on branch 'main' (currently on '${BRANCH}').${RESET}" >&2
    exit 1
fi
echo "  Branch         : ${BRANCH}"

if ! git diff --quiet || ! git diff --cached --quiet; then
    echo -e "${RED}Error: there are uncommitted local changes.${RESET}" >&2
    git status --short >&2
    exit 1
fi
echo "  Working tree   : clean"

git fetch --quiet origin main
LOCAL=$(git rev-parse HEAD)
REMOTE=$(git rev-parse origin/main)
if [ "${LOCAL}" != "${REMOTE}" ]; then
    echo -e "${RED}Error: local commits have not been pushed to origin/main.${RESET}" >&2
    git log origin/main..HEAD --oneline >&2
    exit 1
fi
echo "  Remote sync    : up to date"

# ── Latest published release ──────────────────────────────────────────────────
echo -e "${BOLD}Checking latest GitHub release...${RESET}"
LATEST_TAG=$(gh release list --limit 1 --json tagName --jq '.[0].tagName' 2>/dev/null || true)
echo "  Latest release : ${LATEST_TAG:-<none>}"

# ── Version from Cargo.toml ───────────────────────────────────────────────────
VERSION=$(awk -F'"' '/^version[[:space:]]*=[[:space:]]*"/ {print $2; exit}' Cargo.toml)
NEW_TAG="v${VERSION}"
echo "  New version    : ${NEW_TAG}"

# ── Guard: already released ───────────────────────────────────────────────────
if [ "${LATEST_TAG}" = "${NEW_TAG}" ]; then
    echo -e "\n${YELLOW}Version ${VERSION} is already the latest release. Bump Cargo.toml to create a new release.${RESET}"
    exit 1
fi

# ── Validation builds ─────────────────────────────────────────────────────────
echo -e "\n${BOLD}Running build.sh...${RESET}"
./build.sh

echo -e "\n${BOLD}Running build-dist.sh...${RESET}"
./build-dist.sh

echo -e "\n${BOLD}Running integration-tests.sh...${RESET}"
./integration-tests.sh

# ── Collect assets ────────────────────────────────────────────────────────────
ASSETS=()
for f in scan.sh scan.cmd target/dist/*; do
    [ -f "$f" ] && ASSETS+=("$f")
done

if [ ${#ASSETS[@]} -eq 0 ]; then
    echo -e "${RED}No assets found to upload.${RESET}" >&2
    exit 1
fi

echo -e "\n${BOLD}Assets to upload:${RESET}"
for f in "${ASSETS[@]}"; do
    echo "  $f"
done

# ── Create GitHub release ─────────────────────────────────────────────────────
echo -e "\n${BOLD}Creating release ${NEW_TAG}...${RESET}"
gh release create "${NEW_TAG}" \
    --title "${NEW_TAG}" \
    --generate-notes \
    "${ASSETS[@]}"

echo -e "\n${GREEN}Release ${NEW_TAG} created successfully.${RESET}"
