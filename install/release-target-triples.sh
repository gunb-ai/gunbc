#!/usr/bin/env sh
# POSIX install target detection — hand-synced projection of:
#   - src/v2/extdeps/platform_detection.dag (uname arch/OS mapping tables)
#   - src/v2/install/install.dag `release_published_targets` (Linux musl + macOS rows)
# Triple strings must match src/v2/workflow/release.dag `release_build_matrix` targets.
# Ratchet: release.dag `release_build_matrix` published-triple parity (smoke retired B7).
#
# Do not edit target strings here without updating the model first.

set -eu

# Ordered list — POSIX-installable subset of release.dag release_build_matrix.
RELEASE_PUBLISHED_TARGET_TRIPLES="
x86_64-unknown-linux-musl
aarch64-unknown-linux-musl
x86_64-apple-darwin
aarch64-apple-darwin
"

detect_release_target() {
  os=$(uname -s | tr '[:upper:]' '[:lower:]')
  arch=$(uname -m)

  case "$os" in
    linux)
      case "$arch" in
        x86_64 | amd64) printf '%s\n' 'x86_64-unknown-linux-musl' ;;
        aarch64 | arm64) printf '%s\n' 'aarch64-unknown-linux-musl' ;;
        *)
          echo "release-target-triples: unsupported Linux architecture: $arch" >&2
          exit 1
          ;;
      esac
      ;;
    darwin)
      case "$arch" in
        x86_64) printf '%s\n' 'x86_64-apple-darwin' ;;
        arm64 | aarch64) printf '%s\n' 'aarch64-apple-darwin' ;;
        *)
          echo "release-target-triples: unsupported macOS architecture: $arch" >&2
          exit 1
          ;;
      esac
      ;;
    *)
      echo "release-target-triples: unsupported OS: $os (POSIX install supports Linux and macOS only; Windows: download gunbc-*.exe from GitHub Releases or build from source — see README)" >&2
      exit 1
      ;;
  esac
}
