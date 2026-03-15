#!/usr/bin/env sh
set -eu

BIN_DIR="${LUMA_BIN_DIR:-$HOME/.local/bin}"
TARGET="$BIN_DIR/luma"

if [ -f "$TARGET" ]; then
  rm -f "$TARGET"
  echo "Removed: $TARGET"
else
  echo "Not found: $TARGET"
fi

echo "Uninstall complete."
