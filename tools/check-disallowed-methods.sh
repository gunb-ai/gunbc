#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
allowlist_file="$root_dir/tools/disallowed-methods-allowlist.txt"

if [[ ! -f "$allowlist_file" ]]; then
  echo "ERROR: allowlist not found: $allowlist_file" >&2
  exit 1
fi

pattern='^[[:space:]]*#\!?\[allow\(clippy::disallowed_methods\)\]'

declare -A found_counts

if command -v rg >/dev/null 2>&1; then
  matches_cmd=(rg -n "$pattern" -g'*.rs' "$root_dir")
else
  matches_cmd=(
    grep -R -n -E "$pattern" "$root_dir"
    --include='*.rs'
    --exclude-dir=.git
    --exclude-dir=target
    --exclude-dir=buck-out
    --exclude-dir=bin
  )
fi

while IFS= read -r line; do
  [[ -z "$line" ]] && continue
  file="${line%%:*}"
  if [[ "$file" == "$root_dir/"* ]]; then
    file="${file#"$root_dir"/}"
  fi
  found_counts["$file"]=$(( ${found_counts["$file"]:-0} + 1 ))
done < <("${matches_cmd[@]}" || true)

declare -A allowed_counts

while IFS= read -r line; do
  [[ -z "$line" ]] && continue
  [[ "$line" =~ ^# ]] && continue

  file="${line%%:*}"
  count="${line#*:}"

  if [[ "$file" == "$count" ]]; then
    echo "ERROR: malformed allowlist entry: $line" >&2
    exit 1
  fi

  if ! [[ "$count" =~ ^[0-9]+$ ]]; then
    echo "ERROR: invalid count in allowlist entry: $line" >&2
    exit 1
  fi

  allowed_counts["$file"]="$count"
done < "$allowlist_file"

exit_code=0

for file in "${!found_counts[@]}"; do
  if [[ -z "${allowed_counts[$file]+x}" ]]; then
    echo "ERROR: disallowed allow found outside allowlist: $file (${found_counts[$file]} occurrence(s))" >&2
    exit_code=1
    continue
  fi

  expected="${allowed_counts[$file]}"
  actual="${found_counts[$file]}"
  if [[ "$expected" -ne "$actual" ]]; then
    echo "ERROR: disallowed allow count mismatch in $file (expected $expected, found $actual)" >&2
    exit_code=1
  fi
done

for file in "${!allowed_counts[@]}"; do
  if [[ -z "${found_counts[$file]+x}" ]]; then
    echo "ERROR: allowlist entry has no matching allow in repo: $file" >&2
    exit_code=1
  fi
done

if [[ "$exit_code" -ne 0 ]]; then
  exit "$exit_code"
fi

echo "OK: disallowed_methods allowances match allowlist"
