#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
allowlist_file="$root_dir/tools/disallowed-methods-allowlist.txt"

if [[ ! -f "$allowlist_file" ]]; then
  echo "ERROR: allowlist not found: $allowlist_file" >&2
  exit 1
fi

pattern='allow\([^)]*clippy::disallowed_methods'

declare -A found_counts

if command -v rg >/dev/null 2>&1; then
  matches_cmd=(rg -n --no-ignore-vcs "$pattern" -g'*.rs' "$root_dir")
else
  matches_cmd=(
    grep -R -n -E "$pattern" "$root_dir"
    --include='*.rs'
    --exclude-dir=.git
    --exclude-dir=target
    --exclude-dir=buck-out
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

declare -a allowed_prefixes

while IFS= read -r line; do
  [[ -z "$line" ]] && continue
  [[ "$line" =~ ^# ]] && continue

  # Allow inline comments after '#'
  line="${line%%#*}"
  line="$(echo "$line" | xargs)"
  [[ -z "$line" ]] && continue
  allowed_prefixes+=("$line")
done < "$allowlist_file"

exit_code=0

is_allowed() {
  local file="$1"
  if [[ "$file" == */tests/* ]]; then
    return 0
  fi
  for prefix in "${allowed_prefixes[@]}"; do
    if [[ "$file" == "$prefix"* ]]; then
      return 0
    fi
  done
  return 1
}

for file in "${!found_counts[@]}"; do
  if ! is_allowed "$file"; then
    echo "ERROR: disallowed allow found outside allowlist: $file (${found_counts[$file]} occurrence(s))" >&2
    exit_code=1
  fi
done

if [[ "$exit_code" -ne 0 ]]; then
  exit "$exit_code"
fi

echo "OK: disallowed_methods allowances match allowlist"
