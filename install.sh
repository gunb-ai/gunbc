#!/usr/bin/env sh
# Install gunbc from GitHub Releases (musl Linux + macOS binaries).
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/gunb-ai/gunbc/main/install.sh | sh
#   GUNBC_VERSION=v0.1.0 sh install.sh
#
# Environment:
#   GUNBC_INSTALL_REPO   default gunb-ai/gunbc
#   GUNBC_INSTALL_DIR    default /usr/local/bin
#   GUNBC_VERSION        tag (e.g. v0.1.0) or empty for latest release

set -eu

REPO="${GUNBC_INSTALL_REPO:-gunb-ai/gunbc}"
INSTALL_DIR="${GUNBC_INSTALL_DIR:-/usr/local/bin}"
VERSION="${GUNBC_VERSION:-}"

detect_target() {
  os=$(uname -s | tr '[:upper:]' '[:lower:]')
  arch=$(uname -m)

  case "$os" in
    linux)
      case "$arch" in
        x86_64 | amd64) printf '%s\n' 'x86_64-unknown-linux-musl' ;;
        aarch64 | arm64) printf '%s\n' 'aarch64-unknown-linux-musl' ;;
        *)
          echo "install.sh: unsupported Linux architecture: $arch" >&2
          exit 1
          ;;
      esac
      ;;
    darwin)
      case "$arch" in
        x86_64) printf '%s\n' 'x86_64-apple-darwin' ;;
        arm64 | aarch64) printf '%s\n' 'aarch64-apple-darwin' ;;
        *)
          echo "install.sh: unsupported macOS architecture: $arch" >&2
          exit 1
          ;;
      esac
      ;;
    *)
      echo "install.sh: unsupported OS: $os (need Linux or macOS)" >&2
      exit 1
      ;;
  esac
}

asset_url() {
  target=$1
  asset="gunbc-${target}"
  if [ -n "$VERSION" ]; then
    printf 'https://github.com/%s/releases/download/%s/%s\n' "$REPO" "$VERSION" "$asset"
  else
    printf 'https://github.com/%s/releases/latest/download/%s\n' "$REPO" "$asset"
  fi
}

install_binary() {
  url=$1
  dest=$2
  tmp=$(mktemp "${TMPDIR:-/tmp}/gunbc-install.XXXXXX")
  trap 'rm -f "$tmp"' EXIT INT HUP TERM

  echo "Downloading $url"
  curl -fsSL "$url" -o "$tmp"
  chmod +x "$tmp"

  if [ -w "$INSTALL_DIR" ]; then
    mv "$tmp" "$dest"
  else
    echo "Installing to $dest (sudo required)"
    sudo mv "$tmp" "$dest"
  fi
  trap - EXIT INT HUP TERM
}

main() {
  target=$(detect_target)
  url=$(asset_url "$target")
  dest="${INSTALL_DIR}/gunbc"

  install_binary "$url" "$dest"
  echo "Installed gunbc -> $dest ($target)"
  "$dest" --version 2>/dev/null || "$dest" --help >/dev/null 2>&1 || true
}

main "$@"
