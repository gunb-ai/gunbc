#!/usr/bin/env bash
# Gate-1: full-tree v2 parse-clean MINUS enumerated per-file allowlist.
#
# Runs `gunbc compile` over all 231 dsl modules. Any parse error in a file NOT on
# dsl_v2_parse_allowlist fails the gate (tree-wide teeth). Allowlisted files may
# still error without failing gate-1 (grandfathered v2-parser-debt).

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/gunbc}"
allowlist=(
  "dsl/gunbc/ci_emission.dag"
  "dsl/std/unicode.dag"
)

if [[ ! -x "$bin" ]]; then
  echo "error: gunbc not found at $bin" >&2
  exit 2
fi

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

log="$tmpdir/compile.log"
set +e
"$bin" compile --source-root dsl --output-dir "$tmpdir/out" --target rust >"$log" 2>&1
compile_status=$?
set -e

declare -A allow=()
for path in "${allowlist[@]}"; do
  allow["$path"]=1
done

declare -A failing=()
while IFS= read -r line; do
  if [[ "$line" =~ ^error\[(dsl/[^]]+\.dag):[0-9]+:[0-9]+\]: ]]; then
    failing["${BASH_REMATCH[1]}"]=1
  fi
done <"$log"

unexpected=()
for path in "${!failing[@]}"; do
  if [[ -z "${allow[$path]:-}" ]]; then
    unexpected+=("$path")
  fi
done

if ((${#unexpected[@]} > 0)); then
  echo "gate-1 FAIL: parse error(s) outside allowlist:" >&2
  for path in "${unexpected[@]}"; do
    echo "  - $path" >&2
  done
  echo "--- compile log ---" >&2
  cat "$log" >&2
  exit 1
fi

# Allowlist entries must still fail today (debt not silently paid without list shrink).
for path in "${allowlist[@]}"; do
  if [[ -z "${failing[$path]:-}" ]]; then
    echo "gate-1 FAIL: allowlist entry no longer errors — shrink dsl_v2_parse_allowlist: $path" >&2
    exit 1
  fi
done

if [[ "$compile_status" -eq 0 ]]; then
  echo "gate-1 FAIL: full-tree compile unexpectedly clean — allowlist is stale" >&2
  exit 1
fi

echo "gate-1 PASS: full-tree parse debt confined to ${#allowlist[@]} allowlisted file(s)"
