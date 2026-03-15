#!/usr/bin/env sh
set -eu

REPO="${LUMA_REPO:-tukuyomil032/luma-prism}"
BIN_DIR="${LUMA_BIN_DIR:-$HOME/.local/bin}"
VERSION_INPUT="${LUMA_VERSION:-latest}"
ASSET_URL_OVERRIDE="${LUMA_ASSET_URL:-}"

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

need_cmd curl
need_cmd tar

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin)
    case "$ARCH" in
      x86_64) TARGET="x86_64-apple-darwin" ;;
      arm64|aarch64) TARGET="aarch64-apple-darwin" ;;
      *) echo "Unsupported macOS architecture: $ARCH" >&2; exit 1 ;;
    esac
    EXT="tar.gz"
    ;;
  *)
    echo "This installer currently supports macOS only." >&2
    echo "Download Windows binaries from GitHub Releases." >&2
    exit 1
    ;;
esac

if [ -n "$ASSET_URL_OVERRIDE" ]; then
  URL="$ASSET_URL_OVERRIDE"
else
  if [ "$VERSION_INPUT" = "latest" ]; then
    TAG="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n1 || true)"
    if [ -z "$TAG" ]; then
      TAG="$(curl -fsSL "https://api.github.com/repos/$REPO/tags" | sed -n 's/.*"name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n1)"
    fi
    if [ -z "$TAG" ]; then
      echo "Failed to resolve latest release/tag." >&2
      exit 1
    fi
  else
    case "$VERSION_INPUT" in
      v*) TAG="$VERSION_INPUT" ;;
      *) TAG="v$VERSION_INPUT" ;;
    esac
  fi

  ASSET="luma-prism-${TAG}-${TARGET}.${EXT}"
  URL="https://github.com/${REPO}/releases/download/${TAG}/${ASSET}"
fi

ASSET="$(basename "$URL")"

TMP_DIR="$(mktemp -d)"
cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT INT TERM

mkdir -p "$BIN_DIR"

ARCHIVE_PATH="$TMP_DIR/$ASSET"
echo "Downloading $URL"
curl -fL "$URL" -o "$ARCHIVE_PATH"

tar -xzf "$ARCHIVE_PATH" -C "$TMP_DIR"
install -m 0755 "$TMP_DIR/luma" "$BIN_DIR/luma"

echo "Installed luma to $BIN_DIR/luma"
echo "Run: luma --help"
