#!/usr/bin/env sh
# Install gunbc from GitHub Releases (Linux musl + macOS binaries).
#
# Semantic authority: src/v2/install/install.dag (GitHubReleaseTarball channel).
# Target detection: install/release-target-triples.sh (platform_detection.dag projection).
# Binary names: src/v2/workflow/release.dag `release_published_artifact_names`.
#
# Usage:
#   curl -fsSL https://github.com/gunb-ai/gunbc/releases/latest/download/install.sh | sh
#   GUNBC_VERSION=v0.1.0 sh install.sh
#
# Environment:
#   GUNBC_INSTALL_REPO   default gunb-ai/gunbc
#   GUNBC_INSTALL_DIR    default /usr/local/bin
#   GUNBC_VERSION        tag (e.g. v0.1.0) or empty for latest release
#   GUNBC_RELEASE_TARGETS_URL  override URL for release-target-triples.sh (default: same GH
#                              Release tag/latest channel as the gunbc-{triple} binary)
#   GUNBC_INSTALL_USE_LOCAL_TARGETS  if 1, allow ./install/ cwd fallback when piped via sh
#                              (dev-only; default off so curl | sh uses release assets only)

set -eu

REPO="${GUNBC_INSTALL_REPO:-gunb-ai/gunbc}"
INSTALL_DIR="${GUNBC_INSTALL_DIR:-/usr/local/bin}"
VERSION="${GUNBC_VERSION:-}"

# Piped when argv[0] names an interpreter, not a path to this script (covers zsh/ksh/etc.).
gunbc_install_is_piped() {
  case "${0:-}" in
    */*) return 1 ;;
  esac
  case "$(basename "${0:-}")" in
    install.sh) return 1 ;;
  esac
  return 0
}

load_release_target_authority() {
  if [ -n "${GUNBC_RELEASE_TARGET_AUTHORITY_LOADED:-}" ]; then
    return 0
  fi
  if gunbc_install_is_piped; then
    # `curl … | sh` (any POSIX shell): do not source cwd-local ./install — keeps target
    # authority on the same GH Release channel as the binary (INVARIANTS P2/P3).
    if [ "${GUNBC_INSTALL_USE_LOCAL_TARGETS:-}" = "1" ] \
      && [ -f "./install/release-target-triples.sh" ]; then
      # shellcheck source=install/release-target-triples.sh
      . "./install/release-target-triples.sh"
      GUNBC_RELEASE_TARGET_AUTHORITY_LOADED=1
      return 0
    fi
  else
    _install_dir=$(CDPATH= cd -- "$(dirname "$0")" 2>/dev/null && pwd || true)
    if [ -n "$_install_dir" ] && [ -f "$_install_dir/install/release-target-triples.sh" ]; then
      # shellcheck source=install/release-target-triples.sh
      . "$_install_dir/install/release-target-triples.sh"
      GUNBC_RELEASE_TARGET_AUTHORITY_LOADED=1
      return 0
    fi
    if [ -n "$_install_dir" ] && [ -f "$_install_dir/release-target-triples.sh" ]; then
      # shellcheck source=release-target-triples.sh
      . "$_install_dir/release-target-triples.sh"
      GUNBC_RELEASE_TARGET_AUTHORITY_LOADED=1
      return 0
    fi
    if [ -f "./install/release-target-triples.sh" ]; then
      # shellcheck source=install/release-target-triples.sh
      . "./install/release-target-triples.sh"
      GUNBC_RELEASE_TARGET_AUTHORITY_LOADED=1
      return 0
    fi
  fi
  _authority=$(mktemp "${TMPDIR:-/tmp}/gunbc-release-targets.XXXXXX")
  trap 'rm -f "$_authority"' EXIT INT HUP TERM
  _targets_asset="release-target-triples.sh"
  if [ -n "${GUNBC_RELEASE_TARGETS_URL:-}" ]; then
    _targets_url="${GUNBC_RELEASE_TARGETS_URL}"
  elif [ -n "$VERSION" ]; then
    _targets_url="https://github.com/${REPO}/releases/download/${VERSION}/${_targets_asset}"
  else
    _targets_url="https://github.com/${REPO}/releases/latest/download/${_targets_asset}"
  fi
  curl -fsSL "$_targets_url" -o "$_authority"
  # shellcheck source=/dev/null
  . "$_authority"
  rm -f "$_authority"
  trap - EXIT INT HUP TERM
  GUNBC_RELEASE_TARGET_AUTHORITY_LOADED=1
}

detect_target() {
  load_release_target_authority
  detect_release_target
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
