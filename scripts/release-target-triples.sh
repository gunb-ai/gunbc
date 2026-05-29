#!/usr/bin/env sh
# Single shell authority for release binary target triples (gunbc-{triple} asset names).
# Semantic authority: src/v4/workflow/release.dag `release_build_matrix` (targets via
# `release_published_target_triples` = projection of matrix row targets).
# Do not edit target strings here without updating the model first.
# Ratchet: v4_workflow_release_dag_smoke_test::v4_workflow_release_target_authority_single_writer

set -eu

# Ordered list — must match release.dag release_build_matrix row target order.
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
      echo "release-target-triples: unsupported OS: $os (need Linux or macOS)" >&2
      exit 1
      ;;
  esac
}
