#!/usr/bin/env bash
# SCAFFOLD — dissolve-on: tools.self_host_curated_seed_linked_harness on main post-#6782
# (+ generic std-seed-link follow-up) retires this hand-shell probe runner; until then it
# projects the cssl emit+assemble+cargo spine for per-module verdict TSV (probe-only).
# dissolve-on alt: gunbc bash-emit #5828 / modeled cssl_probe transport in .dag.
# Authority: cssl_v1_compiled_cargo_toml via dag/tools/self_host_curated_probe_cargo.dag
# (docs/probes/lib/render_cssl_probe_lib_cargo_toml.sh — no parallel Cargo.toml heredoc).
#
# INVOCATION CONTRACT (2026-07-19, calm-boar-697 — durable; do not re-learn expensively):
#   CSSL_STD_SEED_LINK=1  — required for std-seed-link closure assembly via cssl_assemble.
#   PROBE_KEEP_LOG_DIR=<dir> — after each probe, copy cargo.log to <dir>/<module>.cargo.log
#                           for per-error census (e0599_census_extract.sh). Emit/assemble always
#                           use a fresh mktemp OUT per invocation — never reuse this dir as OUT
#                           (gunbc compile does not clear stale emitted .rs across runs).
#   shim_lib_rel (arg 2)  — ONLY the lane's own lib.rs from dag/tools/self_host_<lane>_shims/
#                           when that lane provides one (see behavioral_transport shim_lib_rel).
#   Empty = raw cssl-assembled lib.rs (correct default when no lane shim).
#   PHANTOM-GREEN hazard: wrong-lane lib.rs REPLACES cssl lib.rs entirely → cargo green with
#                           invalid shim (false green). Observed: normalize shim on 00_compile → PHANTOM.
#   FALSE-RED hazard:     missing required lane shim → out-of-scope refusals unrelated to entry.
#                           Observed: FormalNonterminal/BTreeSet without 03_normalize shims.
#   When verdicts flip between runs, diff INVOCATION first (shim path, CSSL_STD_SEED_LINK).
#   Ground-truth discriminator for embedded refusals: rg 'UNRESOLVED_CompilerError' or the rustc
#                           error literal in the emitted crate AFTER cssl_assemble — compile_error!
#                           in source = real emit-residue (no shim can fix); string-only = note.
#   Lane shim authority: dag/tools/self_host_*_behavioral_transport.dag shim_lib_rel per module.
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <module.dag-path> [shim_lib_rel]" >&2
  exit 2
fi

MODULE_PATH="$1"
SHIM_LIB_REL="${2:-}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$ROOT"

# shellcheck source=lib/render_cssl_probe_lib_cargo_toml.sh
source "$SCRIPT_DIR/lib/render_cssl_probe_lib_cargo_toml.sh"

STD_SEED_LINK="${CSSL_STD_SEED_LINK:-0}"

GUNBC="${GUNBC:-$ROOT/target/release/gunbc}"
CSSL_ASSEMBLE="${CSSL_ASSEMBLE:-$ROOT/target/release/cssl_assemble}"
if [[ ! -x "$GUNBC" ]]; then
  CTRL_BUILD_WRAP_CARGO=0 cargo build --release -p v1-compiler --bin gunbc >/dev/null
  GUNBC="$ROOT/target/release/gunbc"
fi
if [[ "$STD_SEED_LINK" == "1" && ! -x "$CSSL_ASSEMBLE" ]]; then
  CTRL_BUILD_WRAP_CARGO=0 cargo build --release -p v1-compiler --bin cssl_assemble >/dev/null
  CSSL_ASSEMBLE="$ROOT/target/release/cssl_assemble"
fi
export GUNBC

KEEP_DIR=""
if [[ -n "${PROBE_KEEP_LOG_DIR:-}" ]]; then
  KEEP_DIR="$PROBE_KEEP_LOG_DIR"
  mkdir -p "$KEEP_DIR"
fi
OUT="$(mktemp -d "${TMPDIR:-/tmp}/cssl-probe.XXXXXX")"
cleanup() { rm -rf "$OUT"; }
trap cleanup EXIT

EMIT_LOG="$OUT/emit.log"
EMIT_OK=0
if "$GUNBC" compile \
  --source-root dag \
  --source-root src/v2 \
  --entry "$MODULE_PATH" \
  --output-dir "$OUT" \
  --target rust \
  --dependency-pool-index primary-precedence \
  >"$EMIT_LOG" 2>&1; then
  EMIT_OK=1
fi

EMIT_SUMMARY="emit_fail"
if [[ "$EMIT_OK" -eq 1 ]]; then
  if grep -q 'compiled:' "$EMIT_LOG"; then
    EMIT_SUMMARY="$(grep -m1 'compiled:' "$EMIT_LOG" | sed 's/.*compiled: //')"
  else
    FILE_COUNT="$(find "$OUT" -name '*.rs' 2>/dev/null | wc -l | tr -d ' ')"
    EMIT_SUMMARY="${FILE_COUNT}files,unknown_diag"
  fi
fi

CARGO_VERDICT="skip"
FIRST_ERROR=""
MAPPED_GATE=""
ERROR_HISTOGRAM=""
RAW_DUP_PUB_USE="unmeasured"

measure_raw_dup_pub_use() {
  local src_dir="$1"
  python3 - "$src_dir" <<'PY'
import re, sys, pathlib
src = pathlib.Path(sys.argv[1])
findings = []
for f in sorted(src.glob("*.rs")):
    seen = {}
    for line in f.read_text().splitlines():
        t = line.strip()
        if not t.startswith("pub use "):
            continue
        rest = t[len("pub use "):]
        m = re.match(r"^(.*)\{(.*)\}\s*;", rest)
        syms = [s.strip() for s in m.group(2).split(",")] if m else [rest.rstrip(";").split("::")[-1].strip()]
        for s in syms:
            if not s:
                continue
            base = s.split(" as ")[0].strip()
            if base in seen:
                findings.append(f"{f.name}:{base}")
            else:
                seen[base] = True
print(f"{len(findings)}" + (":" + ",".join(findings[:8]) if findings else ""))
PY
}

uncoded_histogram_suffix() {
  local build_log="$1"
  python3 - "$build_log" <<'PY'
import re, sys, collections
path = sys.argv[1]
counts = collections.Counter()
with open(path, encoding="utf-8", errors="replace") as fh:
    for line in fh:
        if not line.startswith("error: "):
            continue
        if "could not compile" in line or "aborting due to" in line:
            continue
        msg = " ".join(line[len("error: "):].split())
        counts[msg] += 1
parts = []
for msg, n in counts.most_common():
    slug = re.sub(r"[^a-zA-Z0-9_]+", "_", msg).strip("_")[:56]
    parts.append(f"uncoded_{slug}:{n}")
print(" ".join(parts))
PY
}

if [[ "$EMIT_OK" -eq 1 ]]; then
  RAW_DUP_PUB_USE="$(measure_raw_dup_pub_use "$OUT/src")"
  if [[ "$STD_SEED_LINK" == "1" ]]; then
    if ! "$CSSL_ASSEMBLE" --out-dir "$OUT" --entry-dag "$MODULE_PATH" --root "$ROOT" >"$OUT/assemble.log" 2>&1; then
      CARGO_VERDICT="harness_refuse"
      FIRST_ERROR="$(grep -m1 'CSSL_ASSEMBLE: REFUSED' "$OUT/assemble.log" || head -1 "$OUT/assemble.log")"
      MAPPED_GATE="HARNESS_SEED_LINK"
      VERDICT="HARNESS_REFUSE"
      printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
        "$MODULE_PATH" "$EMIT_SUMMARY" "$CARGO_VERDICT" "$FIRST_ERROR" "$MAPPED_GATE" "$VERDICT" "$ERROR_HISTOGRAM" "$RAW_DUP_PUB_USE"
      exit 0
    fi
  fi

  if [[ -n "$SHIM_LIB_REL" && -f "$ROOT/$SHIM_LIB_REL" ]]; then
    cp "$ROOT/$SHIM_LIB_REL" "$OUT/src/lib.rs"
    shim_dir="$(dirname "$ROOT/$SHIM_LIB_REL")"
    for f in "$shim_dir"/*.rs; do
      [[ -f "$f" ]] || continue
      base="$(basename "$f")"
      [[ "$base" == "lib.rs" ]] && continue
      cp "$f" "$OUT/src/$base"
    done
  fi

  if ! render_cssl_probe_lib_cargo_toml "$ROOT" "$OUT/Cargo.toml"; then
    CARGO_VERDICT="harness_refuse"
    FIRST_ERROR="cssl harness authority unavailable"
    MAPPED_GATE="HARNESS_MISSING"
    VERDICT="HARNESS_REFUSE"
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
      "$MODULE_PATH" "$EMIT_SUMMARY" "$CARGO_VERDICT" "$FIRST_ERROR" "$MAPPED_GATE" "$VERDICT" "$ERROR_HISTOGRAM" "$RAW_DUP_PUB_USE"
    exit 0
  fi

  BUILD_LOG="$OUT/cargo.log"
  if (cd "$OUT" && RUSTC_WRAPPER= CTRL_BUILD_WRAP_CARGO=0 cargo build --release --lib 2>"$BUILD_LOG"); then
    CARGO_VERDICT="green"
    ERROR_HISTOGRAM="clean"
  else
    CARGO_VERDICT="refuse"
    ERROR_HISTOGRAM="$(grep -oE '^error\[E[0-9]+\]' "$BUILD_LOG" | sort | uniq -c | sort -rn | awk '{printf "%s%s:%s", sep, $2, $1; sep=" "}' || true)"
    UNCODED_SUFFIX="$(uncoded_histogram_suffix "$BUILD_LOG")"
    if [[ -z "$ERROR_HISTOGRAM" ]]; then
      ERROR_HISTOGRAM="${UNCODED_SUFFIX:-uncoded_only:0}"
    elif [[ -n "$UNCODED_SUFFIX" ]]; then
      ERROR_HISTOGRAM="$ERROR_HISTOGRAM $UNCODED_SUFFIX"
    fi
    FIRST_ERROR="$(grep -m1 -E '^error(\[E[0-9]+\])?:' "$BUILD_LOG" || grep -m1 -E '^error:' "$BUILD_LOG" || head -1 "$BUILD_LOG" || true)"
    if echo "$FIRST_ERROR" | grep -qE 'is defined multiple times|defined multiple times'; then
      MAPPED_GATE="HARNESS_ARTIFACT_std_dup"
    elif echo "$FIRST_ERROR" | grep -qE 'UNRESOLVED_CompilerError'; then
      MAPPED_GATE="UNKNOWN_unresolved"
    elif echo "$FIRST_ERROR" | grep -qE 'expected item after attributes|expected one of|unexpected token'; then
      MAPPED_GATE="UNKNOWN_emit_shape"
    elif echo "$FIRST_ERROR" | grep -qE 'error\[E0432\]: unresolved import|cannot find .+ in this scope|not found in this scope|unbound'; then
      MAPPED_GATE="namespace_resolution"
    elif echo "$FIRST_ERROR" | grep -qE 'error\[E0382\]|error\[E0507\]|error\[E0597\]|cannot move out of|cannot borrow|mismatched types.*Rc<|expected .+ found .+Rc<'; then
      MAPPED_GATE="Gate_A_emitter_Rc_Optional"
    elif echo "$FIRST_ERROR" | grep -qE 'wrapper\.retained|body_producer|Arrow|Behavior'; then
      MAPPED_GATE="Gate_B_body_producer"
    else
      MAPPED_GATE="UNKNOWN"
    fi
    if [[ -n "${PROBE_KEEP_LOG_DIR:-}" ]]; then
      mkdir -p "$PROBE_KEEP_LOG_DIR"
      cp "$BUILD_LOG" "$PROBE_KEEP_LOG_DIR/$(basename "$MODULE_PATH" .dag).cargo.log"
    fi
  fi
  if [[ -n "$KEEP_DIR" && -f "${BUILD_LOG:-}" ]]; then
    mod_base="$(basename "$MODULE_PATH" .dag)"
    cp "$BUILD_LOG" "$KEEP_DIR/${mod_base}.cargo.log"
  fi
fi

VERDICT="PENDING"
if [[ "$CARGO_VERDICT" == "green" ]]; then
  if [[ -n "$SHIM_LIB_REL" ]]; then
    VERDICT="PHANTOM"
  else
    VERDICT="CARGO_GREEN"
  fi
elif [[ "$CARGO_VERDICT" == "refuse" ]]; then
  if [[ "$MAPPED_GATE" == "HARNESS_ARTIFACT_std_dup" ]]; then
    VERDICT="HARNESS_ARTIFACT"
  elif [[ "$MAPPED_GATE" == "Gate_A_emitter_Rc_Optional" ]]; then
    VERDICT="CONFIRMED-Gate_A"
  elif [[ "$MAPPED_GATE" == "Gate_B_body_producer" ]]; then
    VERDICT="CONFIRMED-Gate_B"
  elif [[ "$MAPPED_GATE" == "namespace_resolution" ]]; then
    VERDICT="CONFIRMED-namespace"
  elif [[ "$MAPPED_GATE" == UNKNOWN_* ]]; then
    VERDICT="UNKNOWN-$(echo "$FIRST_ERROR" | tr ' ' '_' | cut -c1-80)"
  else
    VERDICT="UNKNOWN-$(echo "$FIRST_ERROR" | tr ' ' '_' | cut -c1-80)"
  fi
elif [[ "$EMIT_OK" -eq 0 ]]; then
  CARGO_VERDICT="emit_fail"
  FIRST_ERROR="$(head -3 "$EMIT_LOG" | tr '\n' ' ')"
  VERDICT="EMIT_REFUSE"
fi

printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
  "$MODULE_PATH" "$EMIT_SUMMARY" "$CARGO_VERDICT" "$FIRST_ERROR" "$MAPPED_GATE" "$VERDICT" "$ERROR_HISTOGRAM" "$RAW_DUP_PUB_USE"
