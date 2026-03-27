#!/bin/sh

set -eu

if [ "$#" -ne 5 ]; then
  echo "usage: $0 <version> <target> <archive-ext> <binary-path> <output-dir>" >&2
  exit 1
fi

VERSION="$1"
TARGET="$2"
ARCHIVE_EXT="$3"
BINARY_PATH="$4"
OUTPUT_DIR="$5"
APP_NAME="sorted"

PYTHON_BIN=""
if command -v python3 >/dev/null 2>&1; then
  PYTHON_BIN="python3"
elif command -v python >/dev/null 2>&1; then
  PYTHON_BIN="python"
else
  echo "python3 or python is required to package release archives" >&2
  exit 1
fi

if [ ! -f "$BINARY_PATH" ]; then
  echo "binary not found: $BINARY_PATH" >&2
  exit 1
fi

mkdir -p "$OUTPUT_DIR"

case "$TARGET" in
  *windows*)
    EXECUTABLE_NAME="${APP_NAME}.exe"
    ;;
  *)
    EXECUTABLE_NAME="${APP_NAME}"
    ;;
esac

ARCHIVE_NAME="${APP_NAME}-${VERSION}-${TARGET}.${ARCHIVE_EXT}"
STAGING_DIR="$(mktemp -d)"
PACKAGE_ROOT="${STAGING_DIR}/${APP_NAME}-${VERSION}-${TARGET}"

cleanup() {
  rm -rf "$STAGING_DIR"
}

trap cleanup EXIT INT TERM

mkdir -p "$PACKAGE_ROOT"
cp "$BINARY_PATH" "${PACKAGE_ROOT}/${EXECUTABLE_NAME}"
chmod 755 "${PACKAGE_ROOT}/${EXECUTABLE_NAME}" || true

if [ -f "README.md" ]; then
  cp "README.md" "${PACKAGE_ROOT}/README.md"
fi

"$PYTHON_BIN" - "$PACKAGE_ROOT" "$OUTPUT_DIR/$ARCHIVE_NAME" "$ARCHIVE_EXT" <<'PY'
import pathlib
import sys
import tarfile
import zipfile

source_dir = pathlib.Path(sys.argv[1])
archive_path = pathlib.Path(sys.argv[2])
archive_ext = sys.argv[3]

if archive_ext == "tar.gz":
    with tarfile.open(archive_path, "w:gz") as tar:
        tar.add(source_dir, arcname=source_dir.name)
elif archive_ext == "zip":
    with zipfile.ZipFile(archive_path, "w", compression=zipfile.ZIP_DEFLATED) as zf:
        for path in sorted(source_dir.rglob("*")):
            if path.is_file():
                zf.write(path, arcname=path.relative_to(source_dir.parent))
else:
    raise SystemExit(f"unsupported archive extension: {archive_ext}")
PY

echo "Packaged ${OUTPUT_DIR}/${ARCHIVE_NAME}"
