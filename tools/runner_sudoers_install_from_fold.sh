#!/usr/bin/env bash
# Install the four runner sudoers files from the bytes the COMMITTED fold emitted.
#
# Input: the run's stdout, which carries per artifact
#     @@FILE <path> sha256=<hex> bytes=<n>
#     ...base64 of the bytes the fold produced for that path...
#     @@END <path>
# i.e. exactly what the run documented in the PR body prints. This script does nothing but base64-decode
# into the committed path -- it edits no line, sorts nothing, and adds nothing. If the decoded bytes differ
# from what is committed, that is the fold's doing, not this script's.
#
# Usage: runner_sudoers_install_from_fold.sh <fold-output-file>
#
# Post-install accounting (also printed): per host, how many unmask / enable --now / start lines the
# committed bytes carry, against the number of committed CI slots for that host.
set -euo pipefail

[ $# -eq 1 ] || { echo "usage: $0 <fold-output-file>" >&2; exit 2; }
src="$1"
root="$(git rev-parse --show-toplevel)"

for host in srv1 srv2 srv3 srv4; do
  path="provisioning/$host/gunbc-ghrunner.sudoers"
  header=$(grep -m1 "^@@FILE $path " "$src" || true)
  [ -n "$header" ] || { echo "FATAL: the fold emitted no bytes for $path" >&2; exit 1; }
  want_sha=$(awk '{print $3}' <<<"$header" | cut -d= -f2)
  want_bytes=$(awk '{print $4}' <<<"$header" | cut -d= -f2)
  out=$(mktemp)
  awk -v p="@@FILE $path " -v e="@@END $path" 'f && $0==e {exit} f {print} $0 ~ "^"p {f=1}' "$src" | tr -d '[:space:]' | base64 -d > "$out"
  actual=$(wc -c < "$out")
  if [ "$actual" != "$want_bytes" ]; then
    echo "FATAL: $path decoded to $actual bytes, the fold reported $want_bytes" >&2; exit 1
  fi
  got_sha=$(sha256sum "$out" | cut -c1-${#want_sha})
  if [ "$got_sha" != "$want_sha" ]; then
    echo "FATAL: $path sha256 $got_sha, the fold reported $want_sha" >&2; exit 1
  fi
  if ! cmp -s "$out" "$root/$path"; then
    echo "  $path: differs from what is committed -> install"
    cp "$out" "$root/$path"
  else
    echo "  $path: identical to what is committed"
  fi
  rm -f "$out"
done

echo "== committed bytes, per host =="
for host in srv1 srv2 srv3 srv4; do
  f="$root/provisioning/$host/gunbc-ghrunner.sudoers"
  desired=$(grep -c "runner-$host-[0-9]" "$f" || true)
  echo "$host: unmask=$(grep -c 'systemctl unmask' "$f" || true) enable=$(grep -c 'systemctl enable --now' "$f" || true) start=$(grep -c 'systemctl start' "$f" || true) unit_rows=$desired"
done
