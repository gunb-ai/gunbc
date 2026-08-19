#!/usr/bin/env bash
# fast_probe.sh — cheap iteration probe for ONE .dag entry module (measurement-authority tooling
# for materialization_carriers and sibling repair lanes; NOT a corpus survey, and it owns no
# emitter fix). Reuses the established instrument's single authorities (render_cssl_probe_lib_
# cargo_toml.sh for Cargo.toml, the same gunbc/cssl_assemble invocation shape as
# curated_cargo_probe_one.sh) rather than re-deriving them. `curated_cargo_probe_one.sh` remains
# the authority for the milestone-receipt measurement (cargo build --release --lib, full TSV row,
# harness-refuse taxonomy); this script is the fast loop underneath it.
#
# Requirement 1 — stale-binary refusal (load-bearing, per smart-ram-730 2026-08-19): gunbc/
# cssl_assemble are hashed against the current working-tree content of the Rust source that
# produces them (src/v1/**, root Cargo.toml, root Cargo.lock — the v1-compiler workspace), NOT
# their mtime (mtime survives checkouts/rebases and proves nothing about content) and NOT their
# git HEAD SHA alone (uncommitted local edits, which every lane iterates under, would be invisible
# to a HEAD-only check). A stamp file next to the binaries records the content hash this script
# built them from; any mismatch is a hard refusal, never a warning, because a probe that can
# silently measure the wrong compiler is worse than no probe — it fails toward a confident number.
#
# Usage:
#   fast_probe.sh --build                          rebuild gunbc+cssl_assemble, stamp them, exit
#   fast_probe.sh <module.dag> [--imports-only]     probe one module
#   fast_probe.sh <module.dag> --no-diff            skip the before/after delta against the
#                                                    module's own last recorded run
#
# Exit codes: 0 = measured (including a cargo-refuse result — that's data, not a probe failure);
#             1 = STALE_BINARY refusal; 2 = usage error; 3 = emit/harness refusal (mirrors
#             curated_cargo_probe_one.sh's HARNESS_REFUSE convention).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$ROOT"

# shellcheck source=lib/render_cssl_probe_lib_cargo_toml.sh
source "$SCRIPT_DIR/lib/render_cssl_probe_lib_cargo_toml.sh"

GUNBC="$ROOT/target/release/gunbc"
CSSL_ASSEMBLE="$ROOT/target/release/cssl_assemble"
STAMP_FILE="$ROOT/target/release/.fast_probe_source_hash"

# ---- requirement 1: content hash of the source that produces gunbc/cssl_assemble -------------
compute_source_hash() {
  {
    find src/v1 -type f \( -name '*.rs' -o -name 'Cargo.toml' \) -print0
    printf '%s\0' Cargo.toml Cargo.lock
  } | sort -z | xargs -0 sha256sum | sha256sum | awk '{print $1}'
}

refuse_stale_binary() {
  local reason="$1"
  echo "STALE_BINARY: refusing to measure — $reason" >&2
  echo "  gunbc/cssl_assemble may not reflect the source tree being probed." >&2
  echo "  Rebuild and re-stamp with:  $0 --build" >&2
  exit 1
}

if [[ "${1:-}" == "--build" ]]; then
  CTRL_BUILD_WRAP_CARGO=0 cargo build --release -p v1-compiler --bin gunbc --bin cssl_assemble
  compute_source_hash > "$STAMP_FILE"
  echo "built and stamped: $(cat "$STAMP_FILE")"
  exit 0
fi

if [[ ! -x "$GUNBC" || ! -x "$CSSL_ASSEMBLE" ]]; then
  refuse_stale_binary "gunbc or cssl_assemble binary missing at $ROOT/target/release"
fi
if [[ ! -f "$STAMP_FILE" ]]; then
  refuse_stale_binary "no stamp recorded — binaries were not built by this script"
fi
NOW_HASH="$(compute_source_hash)"
STAMPED_HASH="$(cat "$STAMP_FILE")"
if [[ "$NOW_HASH" != "$STAMPED_HASH" ]]; then
  refuse_stale_binary "source hash changed since last --build (stamped $STAMPED_HASH, now $NOW_HASH)"
fi

# ---- usage --------------------------------------------------------------------------------
if [[ $# -lt 1 ]]; then
  echo "usage: $0 <module.dag-path> [--imports-only] [--no-diff]" >&2
  echo "       $0 --build" >&2
  exit 2
fi

MODULE_PATH="$1"
shift
IMPORTS_ONLY=0
NO_DIFF=0
for arg in "$@"; do
  case "$arg" in
    --imports-only) IMPORTS_ONLY=1 ;;
    --no-diff) NO_DIFF=1 ;;
    *) echo "unknown arg: $arg" >&2; exit 2 ;;
  esac
done

SLUG="$(echo "$MODULE_PATH" | sed -E 's#[^A-Za-z0-9]+#_#g')"
WORK_DIR="$ROOT/target/fast-probe/$SLUG"
mkdir -p "$WORK_DIR"

# ---- requirement 2/3: single-module emit into a stable, module-specific crate + target dir ---
CRATE_DIR="$WORK_DIR/crate"
mkdir -p "$CRATE_DIR"
EMIT_LOG="$WORK_DIR/emit.log"
if ! "$GUNBC" compile \
  --source-root dag --source-root src/v2 \
  --entry "$MODULE_PATH" \
  --output-dir "$CRATE_DIR" \
  --target rust \
  --dependency-pool-index primary-precedence \
  >"$EMIT_LOG" 2>&1; then
  echo "EMIT_REFUSE: gunbc compile failed for $MODULE_PATH" >&2
  tail -20 "$EMIT_LOG" >&2
  exit 3
fi

if [[ "$IMPORTS_ONLY" -eq 1 ]]; then
  echo "IMPORTS_OK: $MODULE_PATH resolves and emits cleanly (no cargo step run)"
  exit 0
fi

if ! "$CSSL_ASSEMBLE" --out-dir "$CRATE_DIR" --entry-dag "$MODULE_PATH" --root "$ROOT" \
  >"$WORK_DIR/assemble.log" 2>&1; then
  echo "HARNESS_REFUSE: cssl_assemble failed for $MODULE_PATH" >&2
  tail -20 "$WORK_DIR/assemble.log" >&2
  exit 3
fi

if ! render_cssl_probe_lib_cargo_toml "$ROOT" "$CRATE_DIR/Cargo.toml"; then
  echo "HARNESS_REFUSE: cssl probe Cargo.toml authority unavailable" >&2
  exit 3
fi

# ---- requirement 4: cargo check --message-format=json, in the stable module-scoped target dir
CHECK_JSON="$WORK_DIR/check.json"
(
  cd "$CRATE_DIR"
  CARGO_TARGET_DIR="$WORK_DIR/target" RUSTC_WRAPPER= CTRL_BUILD_WRAP_CARGO=0 \
    cargo check --release --lib --message-format=json
) >"$CHECK_JSON" 2>"$WORK_DIR/check.stderr.log" || true

# ---- requirement 5: dedup by (code, primary span, normalized cause signature) ----------------
# requirement 6: report the before/after delta at (code, primary span) grain, not as counts.
RESULT_JSON="$WORK_DIR/result.json"
python3 - "$CHECK_JSON" "$RESULT_JSON" "$MODULE_PATH" "$NOW_HASH" <<'PY'
import json, sys, hashlib

check_path, result_path, module_path, source_hash = sys.argv[1:5]

sites = {}
with open(check_path, encoding="utf-8", errors="replace") as f:
    for line in f:
        line = line.strip()
        if not line:
            continue
        try:
            obj = json.loads(line)
        except json.JSONDecodeError:
            continue
        if obj.get("reason") != "compiler-message":
            continue
        m = obj.get("message", {})
        if m.get("level") != "error":
            continue
        code = (m.get("code") or {}).get("code") or "uncoded"
        primary = next((s for s in m.get("spans", []) if s.get("is_primary")), None)
        if not primary:
            continue
        children = tuple(sorted(c.get("message", "") for c in m.get("children", [])))
        cause_sig = hashlib.sha256(
            (m.get("message", "") + "|" + "|".join(children)).encode("utf-8")
        ).hexdigest()[:12]
        key = (code, primary.get("file_name"), primary.get("line_start"),
               primary.get("column_start"), cause_sig)
        # dedup: last write wins, identical key collapses to one row
        sites[key] = {
            "code": code,
            "file": primary.get("file_name"),
            "line": primary.get("line_start"),
            "col": primary.get("column_start"),
            "message": m.get("message", ""),
            "children": list(children),
            "cause_sig": cause_sig,
        }

result = {
    "module": module_path,
    "source_hash": source_hash,
    "count": len(sites),
    "sites": sorted(sites.values(), key=lambda s: (s["file"] or "", s["line"] or 0, s["col"] or 0)),
}
with open(result_path, "w") as f:
    json.dump(result, f, indent=2)
print(f"{len(sites)} error sites (deduped by code+span+cause)")
PY

# ---- before/after at (code, primary span) grain, never as counts alone -----------------------
PREV_JSON="$WORK_DIR/result.prev.json"
if [[ "$NO_DIFF" -eq 0 && -f "$PREV_JSON" ]]; then
  python3 - "$PREV_JSON" "$RESULT_JSON" <<'PY'
import json, sys

with open(sys.argv[1]) as f:
    prev = json.load(f)
with open(sys.argv[2]) as f:
    now = json.load(f)

def key(s):
    return (s["code"], s["file"], s["line"], s["col"])

prev_keys = {key(s): s for s in prev["sites"]}
now_keys = {key(s): s for s in now["sites"]}

removed = [s for k, s in prev_keys.items() if k not in now_keys]
added = [s for k, s in now_keys.items() if k not in prev_keys]

def show(prefix, s):
    print(f"  {prefix} {s['code']} {s['file']}:{s['line']}:{s['col']}  {s['message']}")
    # children carried verbatim — this is the exact text two same-code sites disagreed on
    # in the 2026-08-19 1a/1b dispute; a delta that drops it cannot tell the mechanisms apart.
    for c in s.get("children", []):
        print(f"      child: {c}")

print(f"\n--- delta vs last run of this module ({prev['count']} -> {now['count']}) ---")
if not removed and not added:
    print("no site-level change (count may still be unchanged for the same reason, or by coincidence)")
for s in removed:
    show("- REMOVED", s)
for s in added:
    show("+ ADDED  ", s)
PY
elif [[ "$NO_DIFF" -eq 0 ]]; then
  echo "no prior run recorded for this module — this run becomes the baseline"
fi

cp "$RESULT_JSON" "$PREV_JSON"
