#!/usr/bin/env bash
# SCAFFOLD — dissolve-on: tools.self_host_curated_seed_linked_harness on main post-#6782
# (+ generic std-seed-link follow-up) retires this hand-shell probe runner; until then it
# projects the cssl emit+assemble+cargo spine for per-module verdict TSV (probe-only).
# dissolve-on alt: gunbc bash-emit #5828 / modeled cssl_probe transport in .dag.
# Authority: cssl_v1_compiled_cargo_toml via dag/tools/self_host_curated_probe_cargo.dag
# (scripts/lib/render_cssl_probe_lib_cargo_toml.sh — no parallel Cargo.toml heredoc).
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <module.dag-path> [shim_lib_rel]" >&2
  exit 2
fi

MODULE_PATH="$1"
SHIM_LIB_REL="${2:-}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# shellcheck source=lib/render_cssl_probe_lib_cargo_toml.sh
source "$ROOT/scripts/lib/render_cssl_probe_lib_cargo_toml.sh"
# shellcheck source=lib/apply_std_seed_link_assembly.sh
source "$ROOT/scripts/lib/apply_std_seed_link_assembly.sh"

STD_SEED_LINK="${CSSL_STD_SEED_LINK:-0}"

GUNBC="${GUNBC:-$ROOT/target/release/gunbc}"
if [[ ! -x "$GUNBC" ]]; then
  CTRL_BUILD_WRAP_CARGO=0 cargo build --release -p v1-compiler --bin gunbc >/dev/null
  GUNBC="$ROOT/target/release/gunbc"
fi
export GUNBC

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

if [[ "$EMIT_OK" -eq 1 ]]; then
  if [[ "$STD_SEED_LINK" == "1" ]]; then
    if ! apply_std_seed_link_assembly "$OUT" "$ROOT/$MODULE_PATH" "$ROOT"; then
      CARGO_VERDICT="harness_refuse"
      FIRST_ERROR="std-seed-link assembly failed or entry mutated"
      MAPPED_GATE="HARNESS_SEED_LINK"
      VERDICT="HARNESS_REFUSE"
      printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
        "$MODULE_PATH" "$EMIT_SUMMARY" "$CARGO_VERDICT" "$FIRST_ERROR" "$MAPPED_GATE" "$VERDICT"
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
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
      "$MODULE_PATH" "$EMIT_SUMMARY" "$CARGO_VERDICT" "$FIRST_ERROR" "$MAPPED_GATE" "$VERDICT"
    exit 0
  fi

  BUILD_LOG="$OUT/cargo.log"
  if (cd "$OUT" && RUSTC_WRAPPER= CTRL_BUILD_WRAP_CARGO=0 cargo build --release --lib 2>"$BUILD_LOG"); then
    CARGO_VERDICT="green"
  else
    CARGO_VERDICT="refuse"
    FIRST_ERROR="$(grep -m1 -E '^error(\[E[0-9]+\])?:' "$BUILD_LOG" || grep -m1 -E '^error:' "$BUILD_LOG" || head -1 "$BUILD_LOG" || true)"
    # RULE 1: any duplicate-definition is harness std-dup — never a gate verdict.
    if grep -qE 'is defined multiple times|defined multiple times' "$BUILD_LOG"; then
      MAPPED_GATE="HARNESS_ARTIFACT_std_dup"
    elif grep -qE 'UNRESOLVED_CompilerError' "$BUILD_LOG"; then
      MAPPED_GATE="UNKNOWN_unresolved"
    elif grep -qE 'expected item after attributes|expected one of|unexpected token' "$BUILD_LOG"; then
      MAPPED_GATE="UNKNOWN_emit_shape"
    elif grep -qE 'error\[E0382\]|error\[E0507\]|error\[E0597\]|cannot move out of|cannot borrow|mismatched types.*Rc<|expected .+ found .+Rc<' "$BUILD_LOG"; then
      MAPPED_GATE="Gate_A_emitter_Rc_Optional"
    elif grep -qE 'unbound|cannot find .+ in this scope|not found in this scope|resolve_' "$BUILD_LOG"; then
      MAPPED_GATE="namespace_resolution"
    elif grep -qE 'wrapper\.retained|body_producer|Arrow|Behavior' "$BUILD_LOG"; then
      MAPPED_GATE="Gate_B_body_producer"
    else
      MAPPED_GATE="UNKNOWN"
    fi
  fi
fi

VERDICT="PENDING"
if [[ "$CARGO_VERDICT" == "green" ]]; then
  VERDICT="PHANTOM"
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

printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
  "$MODULE_PATH" "$EMIT_SUMMARY" "$CARGO_VERDICT" "$FIRST_ERROR" "$MAPPED_GATE" "$VERDICT"
