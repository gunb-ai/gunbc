#!/usr/bin/env sh
# Single shell authority for release binary target triples (gunbc-{triple} asset names).
# Semantic authority: src/v4/workflow/release.dag `release_build_matrix` (targets via
# `release_published_target_triples` = projection of matrix row targets).
# Do not edit target strings here without updating the model first.
# Ratchet: v4_workflow_release_dag_smoke_test::v4_workflow_release_target_authority_single_writer

set -eu

# Ordered list — must match release.dag release_build_matrix row target order.
# Sole home of published triple literals; detector selects by index only.
RELEASE_PUBLISHED_TARGET_TRIPLES="
x86_64-unknown-linux-musl
aarch64-unknown-linux-musl
x86_64-apple-darwin
aarch64-apple-darwin
"

# Print the nth non-empty line (1-based) from RELEASE_PUBLISHED_TARGET_TRIPLES.
release_published_target_at() {
  n=$1
  i=0
  while IFS= read -r line || [ -n "$line" ]; do
    line=$(printf '%s' "$line" | tr -d '[:space:]')
    [ -z "$line" ] && continue
    i=$((i + 1))
    if [ "$i" -eq "$n" ]; then
      printf '%s\n' "$line"
      return 0
    fi
  done <<EOF
$RELEASE_PUBLISHED_TARGET_TRIPLES
EOF
  echo "release-target-triples: invalid target index $n" >&2
  return 1
}

detect_release_target() {
  os=$(uname -s | tr '[:upper:]' '[:lower:]')
  arch=$(uname -m)
  idx=

  case "$os" in
    linux)
      case "$arch" in
        x86_64 | amd64) idx=1 ;;
        aarch64 | arm64) idx=2 ;;
        *)
          echo "release-target-triples: unsupported Linux architecture: $arch" >&2
          exit 1
          ;;
      esac
      ;;
    darwin)
      case "$arch" in
        x86_64) idx=3 ;;
        arm64 | aarch64) idx=4 ;;
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

  release_published_target_at "$idx"
}
