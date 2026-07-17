#!/usr/bin/env bash
# Curated seed-linked cargo probe for one compiler frontier module.
# Mirrors tools.self_host_curated_seed_linked_harness emit+assemble spine (probe-only).
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <module.dag-path> [shim_lib_rel]" >&2
  exit 2
fi

MODULE_PATH="$1"
SHIM_LIB_REL="${2:-}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

GUNBC="${GUNBC:-$ROOT/target/release/gunbc}"
if [[ ! -x "$GUNBC" ]]; then
  CTRL_BUILD_WRAP_CARGO=0 cargo build --release -p v1-compiler --bin gunbc >/dev/null
  GUNBC="$ROOT/target/release/gunbc"
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

if [[ "$EMIT_OK" -eq 1 ]]; then
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

  cat >"$OUT/Cargo.toml" <<EOF
[package]
name = "v1_compiled"
version = "0.1.0"
edition = "2021"

[features]
text_lookup_work_counter = []

[lib]
path = "src/lib.rs"

[dependencies]
im-rc = { version = "15.1", features = ["serde"] }
serde = { version = "1", features = ["derive", "rc"] }
serde_json = "1"
stacker = "0.1"
lazy_static = "1"
unicode-ident = "1"
unicode-properties = { version = "0.1", features = ["emoji"] }
v1-compiler = { path = "$ROOT/src/v1/stage0" }
EOF

  BUILD_LOG="$OUT/cargo.log"
  if (cd "$OUT" && RUSTC_WRAPPER= CTRL_BUILD_WRAP_CARGO=0 cargo build --release --lib 2>"$BUILD_LOG"); then
    CARGO_VERDICT="green"
  else
    CARGO_VERDICT="refuse"
    FIRST_ERROR="$(grep -m1 -E '^error(\[E[0-9]+\])?:' "$BUILD_LOG" || grep -m1 -E '^error:' "$BUILD_LOG" || head -1 "$BUILD_LOG" || true)"
    if grep -qE 'the name `Int(8|16|32|64|128|256)` is defined multiple times|duplicate definitions with name' "$BUILD_LOG"; then
      MAPPED_GATE="HARNESS_ARTIFACT_std_dup"
    elif grep -qE 'Rc<|im_rc|Optional|ownership' "$BUILD_LOG"; then
      MAPPED_GATE="Gate_A_emitter_Rc_Optional"
    elif grep -qE 'unbound|cannot find|not found in this scope|resolve_' "$BUILD_LOG"; then
      MAPPED_GATE="namespace_resolution"
    elif grep -qE 'wrapper.retained|body_producer|Arrow|Behavior' "$BUILD_LOG"; then
      MAPPED_GATE="Gate_B_body_producer"
    else
      MAPPED_GATE="NEW"
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
  else
    VERDICT="NEW-$(echo "$FIRST_ERROR" | tr ' ' '_' | cut -c1-60)"
  fi
elif [[ "$EMIT_OK" -eq 0 ]]; then
  CARGO_VERDICT="emit_fail"
  FIRST_ERROR="$(head -3 "$EMIT_LOG" | tr '\n' ' ')"
  VERDICT="EMIT_REFUSE"
fi

printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
  "$MODULE_PATH" "$EMIT_SUMMARY" "$CARGO_VERDICT" "$FIRST_ERROR" "$MAPPED_GATE" "$VERDICT"
