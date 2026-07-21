#!/usr/bin/env bash
# Measure E0308 on EMITTED .rs output (not committed seed lib).
# Paths:
#   namespace = gunbc compile --entry with reference-derived closure (warm-lark Rule-1;
#               requires gunbc built from a tree with load_sources_for_entry in compile handler)
#   import    = gunbc compile --entry with import-edge closure only (main default)
#   cssl      = isolated CSSL probe (CSSL_STD_SEED_LINK=1 assemble; FaithfulFreeMonoid class)
# Usage: repr_mismatch_emitted_e0308_probe.sh <module.dag> [namespace|import|cssl]
set -euo pipefail

MODULE_PATH="${1:?usage: $0 <module.dag> [namespace|import|cssl]}"
MODE="${2:-namespace}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# shellcheck source=lib/render_cssl_probe_lib_cargo_toml.sh
source "$ROOT/scripts/lib/render_cssl_probe_lib_cargo_toml.sh"

GUNBC="${GUNBC:-$ROOT/target/release/gunbc}"
CSSL="${CSSL_ASSEMBLE:-$ROOT/target/release/cssl_assemble}"
if [[ ! -x "$GUNBC" ]]; then
  CTRL_BUILD_WRAP_CARGO=0 cargo build --release -p v1-compiler --bin gunbc --bin cssl_assemble >/dev/null
  GUNBC="$ROOT/target/release/gunbc"
  CSSL="$ROOT/target/release/cssl_assemble"
fi

count_e0308() { grep -c 'error\[E0308\]' "$1" 2>/dev/null || true; }
count_rustc_errors() { grep -cE '^error(\[E[0-9]+\])?:' "$1" 2>/dev/null || true; }
module_rs_slug() {
  local dag="$1"
  local base
  base="$(basename "$dag" .dag)"
  echo "v2_compiler_${base}"
}

OUT="$(mktemp -d "${TMPDIR:-/tmp}/repr-emitted.XXXXXX")"
EMIT_LOG="$OUT/emit.log"
BUILD_LOG="$OUT/cargo.log"
RS_SLUG="$(module_rs_slug "$MODULE_PATH")"

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
    EMIT_SUMMARY="$(find "$OUT" -name '*.rs' 2>/dev/null | wc -l | tr -d ' ')files"
  fi
fi

CSSL_RAN=0
if [[ "$EMIT_OK" -eq 1 && "$MODE" == "cssl" ]]; then
  if CSSL_STD_SEED_LINK=1 "$CSSL" --out-dir "$OUT" --entry-dag "$MODULE_PATH" --root "$ROOT" >"$OUT/assemble.log" 2>&1; then
    CSSL_RAN=1
  fi
fi

CARGO_RAN=0
if [[ "$EMIT_OK" -eq 1 ]]; then
  if render_cssl_probe_lib_cargo_toml "$ROOT" "$OUT/Cargo.toml"; then
    if (cd "$OUT" && RUSTC_WRAPPER= CTRL_BUILD_WRAP_CARGO=0 cargo build --release --lib 2>"$BUILD_LOG"); then
      CARGO_RAN=green
    else
      CARGO_RAN=refuse
    fi
  else
    CARGO_RAN=harness_fail
  fi
fi

E0308_ALL=0 E0308_MODULE=0 TOTAL=0
if [[ "$CARGO_RAN" == refuse ]]; then
  E0308_ALL="$(count_e0308 "$BUILD_LOG")"
  TOTAL="$(count_rustc_errors "$BUILD_LOG")"
  E0308_MODULE="$(grep 'error\[E0308\]' "$BUILD_LOG" 2>/dev/null | grep -c "$RS_SLUG" || true)"
fi

FIRST="$(grep -m1 -E '^error(\[E[0-9]+\])?:' "$BUILD_LOG" 2>/dev/null || echo '(none)')"
HEAD="$(git rev-parse --short HEAD)"
printf 'path=EMITTED_%s\tmodule=%s\thead=%s\tgunbc=%s\temit_ok=%s\temit_summary=%s\tcssl_ran=%s\tcargo=%s\te0308_all=%s\te0308_module_rs=%s\ttotal_rustc_errors=%s\tfirst_error=%s\n' \
  "$MODE" "$MODULE_PATH" "$HEAD" "$GUNBC" "$EMIT_OK" "$EMIT_SUMMARY" "$CSSL_RAN" "$CARGO_RAN" \
  "$E0308_ALL" "$E0308_MODULE" "$TOTAL" "$FIRST"

if [[ "${KEEP_PROBE_OUT:-0}" == "1" ]]; then
  echo "probe_out=$OUT" >&2
else
  rm -rf "$OUT"
fi
