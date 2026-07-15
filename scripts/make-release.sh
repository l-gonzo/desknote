#!/usr/bin/env bash
set -Eeuo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="$(awk -F'"' '/^version = / {print $2; exit}' "$ROOT_DIR/Cargo.toml")"
NAME="note-desktop-$VERSION"
OUT_DIR="${1:-$(dirname "$ROOT_DIR")}" 
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
cp -a "$ROOT_DIR" "$TMP/$NAME"
rm -rf "$TMP/$NAME/target" "$TMP/$NAME/dist" "$TMP/$NAME/.git"
(
  cd "$TMP"
  tar -czf "$OUT_DIR/$NAME.tar.gz" "$NAME"
  zip -qr "$OUT_DIR/$NAME.zip" "$NAME"
)
sha256sum "$OUT_DIR/$NAME.tar.gz" "$OUT_DIR/$NAME.zip" > "$OUT_DIR/$NAME-SHA256SUMS.txt"
echo "Release creada en $OUT_DIR"
