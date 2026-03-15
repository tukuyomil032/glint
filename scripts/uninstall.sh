#!/usr/bin/env sh
set -eu

BIN_DIR="${GLINT_BIN_DIR:-$HOME/.local/bin}"
TARGET="$BIN_DIR/glint"

if [ -f "$TARGET" ]; then
  rm -f "$TARGET"
  echo "Removed: $TARGET"
else
  echo "Not found: $TARGET"
fi

echo "Uninstall complete."
