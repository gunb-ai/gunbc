#!/usr/bin/env sh
# Single shell authority for release binary target triples (gunbc-{triple} asset names).
# Semantic authority: src/v4/workflow/release.dag `release_build_matrix` (targets via
# `release_published_target_triples` = projection of matrix row targets).
# Do not edit target strings here without updating the model first.
# Ratchet: v4_workflow_release_dag_smoke_test::v4_workflow_release_target_authority_single_writer

set -eu

# Ordered list — must match release.dag release_build_matrix row target order.
# Sole home of published triple literals; OS/arch detection resolves by triple string
# (not matrix row index) so reordering release_build_matrix cannot install wrong assets.
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

# Resolve a published triple by exact string (order-independent vs matrix row index).
release_published_target_lookup() {
  want=$(printf '%s' "$1" | tr -d '[:space:]')
  while IFS= read -r line || [ -n "$line" ]; do
    line=$(printf '%s' "$line" | tr -d '[:space:]')
    [ -z "$line" ] && continue
    if [ "$line" = "$want" ]; then
      printf '%s\n' "$line"
      return 0
    fi
  done <<EOF
$RELEASE_PUBLISHED_TARGET_TRIPLES
EOF
  echo "release-target-triples: triple not in published list: $want" >&2
  return 1
}

detect_release_target() {
  os=$(uname -s | tr '[:upper:]' '[:lower:]')
  arch=$(uname -m)
  triple=

  case "$os" in
    linux)
      case "$arch" in
        x86_64 | amd64) triple=x86_64-unknown-linux-musl ;;
        aarch64 | arm64) triple=aarch64-unknown-linux-musl ;;
        *)
          echo "release-target-triples: unsupported Linux architecture: $arch" >&2
          exit 1
          ;;
      esac
      ;;
    darwin)
      case "$arch" in
        x86_64) triple=x86_64-apple-darwin ;;
        arm64 | aarch64) triple=aarch64-apple-darwin ;;
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

  release_published_target_lookup "$triple"
}
