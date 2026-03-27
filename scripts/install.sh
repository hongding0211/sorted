#!/bin/sh

set -eu

APP_NAME="sorted"
DEFAULT_REPO="hongding0211/sorted"
DEFAULT_INSTALL_DIR="${HOME}/.local/bin"
REPO="${SORTED_REPO:-$DEFAULT_REPO}"
INSTALL_DIR="${SORTED_INSTALL_DIR:-$DEFAULT_INSTALL_DIR}"
REQUESTED_VERSION="${SORTED_VERSION:-}"
API_BASE="${SORTED_API_BASE:-https://api.github.com}"

usage() {
  cat <<'EOF'
Install the latest Sorted release from GitHub.

Usage:
  install.sh [version]

Environment:
  SORTED_REPO         GitHub repository in owner/name form
  SORTED_INSTALL_DIR  Target directory for the executable
  SORTED_VERSION      Version override, for example v0.1.0
  SORTED_API_BASE     Alternate GitHub API base URL
EOF
}

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  usage
  exit 0
fi

if [ -n "${1:-}" ]; then
  REQUESTED_VERSION="$1"
fi

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required tool: $1" >&2
    exit 1
  fi
}

need_cmd uname
need_cmd mktemp
need_cmd chmod
need_cmd rm
need_cmd mkdir
need_cmd cp
need_cmd curl

PYTHON_BIN=""
if command -v python3 >/dev/null 2>&1; then
  PYTHON_BIN="python3"
elif command -v python >/dev/null 2>&1; then
  PYTHON_BIN="python"
else
  echo "missing required tool: python3 (or python)" >&2
  exit 1
fi

OS_NAME="$(uname -s)"
ARCH_NAME="$(uname -m)"

case "$OS_NAME" in
  Darwin)
    case "$ARCH_NAME" in
      arm64|aarch64)
        TARGET="aarch64-apple-darwin"
        ARCHIVE_EXT="tar.gz"
        need_cmd tar
        ;;
      x86_64)
        TARGET="x86_64-apple-darwin"
        ARCHIVE_EXT="tar.gz"
        need_cmd tar
        ;;
      *)
        echo "unsupported CPU architecture for macOS: $ARCH_NAME" >&2
        exit 1
        ;;
    esac
    ;;
  Linux)
    case "$ARCH_NAME" in
      x86_64|amd64)
        TARGET="x86_64-unknown-linux-gnu"
        ARCHIVE_EXT="tar.gz"
        need_cmd tar
        ;;
      *)
        echo "unsupported CPU architecture for Linux: $ARCH_NAME" >&2
        exit 1
        ;;
    esac
    ;;
  MINGW*|MSYS*|CYGWIN*)
    echo "the shell installer currently supports macOS and Linux. Download the Windows zip from GitHub Releases instead." >&2
    exit 1
    ;;
  *)
    echo "unsupported operating system: $OS_NAME" >&2
    exit 1
    ;;
esac

normalize_version() {
  case "$1" in
    v*) printf '%s' "$1" ;;
    *) printf 'v%s' "$1" ;;
  esac
}

if [ -n "$REQUESTED_VERSION" ]; then
  VERSION="$(normalize_version "$REQUESTED_VERSION")"
  RELEASE_ENDPOINT="${API_BASE}/repos/${REPO}/releases/tags/${VERSION}"
else
  RELEASE_ENDPOINT="${API_BASE}/repos/${REPO}/releases/latest"
fi

TMP_DIR="$(mktemp -d)"
cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT INT TERM

METADATA_PATH="${TMP_DIR}/release.json"
curl --fail --silent --show-error --location "$RELEASE_ENDPOINT" -o "$METADATA_PATH"

RELEASE_INFO="$("$PYTHON_BIN" - "$METADATA_PATH" "$TARGET" "$ARCHIVE_EXT" <<'PY'
import json
import pathlib
import sys

metadata = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
target = sys.argv[2]
archive_ext = sys.argv[3]
tag_name = metadata.get("tag_name")
assets = metadata.get("assets", [])
expected = f"sorted-{tag_name}-{target}.{archive_ext}"

for asset in assets:
    if asset.get("name") == expected:
        print(tag_name)
        print(asset.get("browser_download_url", ""))
        break
else:
    raise SystemExit(f"no release asset matches {expected}")
PY
)"

VERSION="$(printf '%s\n' "$RELEASE_INFO" | sed -n '1p')"
ASSET_URL="$(printf '%s\n' "$RELEASE_INFO" | sed -n '2p')"

if [ -z "$ASSET_URL" ]; then
  echo "failed to resolve a downloadable asset for ${TARGET}" >&2
  exit 1
fi

ARCHIVE_PATH="${TMP_DIR}/${APP_NAME}.${ARCHIVE_EXT}"
EXTRACT_DIR="${TMP_DIR}/extract"
mkdir -p "$EXTRACT_DIR" "$INSTALL_DIR"

echo "Downloading ${APP_NAME} ${VERSION} for ${TARGET}..."
curl --fail --silent --show-error --location "$ASSET_URL" -o "$ARCHIVE_PATH"

case "$ARCHIVE_EXT" in
  tar.gz)
    tar -xzf "$ARCHIVE_PATH" -C "$EXTRACT_DIR"
    ;;
  zip)
    need_cmd unzip
    unzip -q "$ARCHIVE_PATH" -d "$EXTRACT_DIR"
    ;;
  *)
    echo "unsupported archive extension: $ARCHIVE_EXT" >&2
    exit 1
    ;;
esac

BIN_PATH="$(find "$EXTRACT_DIR" -type f -name "$APP_NAME" | head -n 1)"
if [ -z "$BIN_PATH" ]; then
  echo "downloaded archive did not contain ${APP_NAME}" >&2
  exit 1
fi

cp "$BIN_PATH" "${INSTALL_DIR}/${APP_NAME}"
chmod 755 "${INSTALL_DIR}/${APP_NAME}"

echo "Installed ${APP_NAME} to ${INSTALL_DIR}/${APP_NAME}"

case ":${PATH}:" in
  *":${INSTALL_DIR}:"*)
    echo "Run '${APP_NAME} --help' to get started."
    ;;
  *)
    echo "Add ${INSTALL_DIR} to your PATH before running '${APP_NAME}'."
    echo "Example:"
    echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    ;;
esac
