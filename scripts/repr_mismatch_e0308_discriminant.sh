#!/usr/bin/env bash
# Discriminating E0308 measurement for emit_representation_mismatch (#6959).
# Compares CSSL/FaithfulFreeMonoid isolated probe vs HostNative whole-tree seed check.
# Usage: repr_mismatch_e0308_discriminant.sh [module.dag]
set -euo pipefail

MODULE_PATH="${1:-src/v2/compiler/04_infer.dag}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# shellcheck source=lib/render_cssl_probe_lib_cargo_toml.sh
source "$ROOT/scripts/lib/render_cssl_probe_lib_cargo_toml.sh"

HEAD="$(git rev-parse --short HEAD)"
MODULE_BASENAME="$(basename "$MODULE_PATH" .dag)"

count_e0308() { grep -c 'error\[E0308\]' "$1" 2>/dev/null || true; }
count_rustc_errors() { grep -cE '^error(\[E[0-9]+\])?:' "$1" 2>/dev/null || true; }
first_rustc_error() { grep -m1 -E '^error(\[E[0-9]+\])?:' "$1" 2>/dev/null || echo "(no rustc error line)"; }

measure_cssl_faithful() {
  local out emit_log build_log
  out="$(mktemp -d "${TMPDIR:-/tmp}/repr-cssl.XXXXXX")"
  emit_log="$out/emit.log"
  build_log="$out/cargo.log"

  local gunbc="${GUNBC:-$ROOT/target/release/gunbc}"
  local cssl="${CSSL_ASSEMBLE:-$ROOT/target/release/cssl_assemble}"
  if [[ ! -x "$gunbc" ]]; then
    CTRL_BUILD_WRAP_CARGO=0 cargo build --release -p v1-compiler --bin gunbc >/dev/null
    gunbc="$ROOT/target/release/gunbc"
  fi
  if [[ ! -x "$cssl" ]]; then
    CTRL_BUILD_WRAP_CARGO=0 cargo build --release -p v1-compiler --bin cssl_assemble >/dev/null
    cssl="$ROOT/target/release/cssl_assemble"
  fi

  local emit_ok=0
  if "$gunbc" compile \
    --source-root dag \
    --source-root src/v2 \
    --entry "$MODULE_PATH" \
    --output-dir "$out" \
    --target rust \
    --dependency-pool-index primary-precedence \
    >"$emit_log" 2>&1; then
    emit_ok=1
  fi

  local cargo_ran=0
  if [[ "$emit_ok" -eq 1 ]]; then
    if CSSL_STD_SEED_LINK=1 "$cssl" --out-dir "$out" --entry-dag "$MODULE_PATH" --root "$ROOT" >"$out/assemble.log" 2>&1; then
      if render_cssl_probe_lib_cargo_toml "$ROOT" "$out/Cargo.toml"; then
        if (cd "$out" && RUSTC_WRAPPER= CTRL_BUILD_WRAP_CARGO=0 cargo build --release --lib 2>"$build_log"); then
          : # green
        else
          cargo_ran=1
        fi
      fi
    fi
  fi

  local e0308=0 total=0
  if [[ "$cargo_ran" -eq 1 ]]; then
    e0308="$(count_e0308 "$build_log")"
    total="$(count_rustc_errors "$build_log")"
  fi

  printf 'path=CSSL_FaithfulFreeMonoid\tmodule=%s\thead=%s\temit_ok=%s\tcargo_ran=%s\te0308=%s\ttotal_rustc_errors=%s\tfirst_error=%s\n' \
    "$MODULE_PATH" "$HEAD" "$emit_ok" "$cargo_ran" "$e0308" "$total" "$(first_rustc_error "$build_log")"

  rm -rf "$out"
}

measure_hostnative_whole_tree() {
  local build_log
  build_log="$(mktemp "${TMPDIR:-/tmp}/repr-host.XXXXXX.log")"

  local ran=0
  if (cd "$ROOT" && RUSTC_WRAPPER= CTRL_BUILD_WRAP_CARGO=0 cargo check -p v1-compiler --lib 2>"$build_log"); then
    ran=1
  else
    ran=1
  fi

  local e0308_all e0308_infer
  e0308_all="$(count_e0308 "$build_log")"
  e0308_infer="$(grep 'error\[E0308\]' "$build_log" 2>/dev/null | grep -c 'v2_compiler_infer\|v1_compiler_infer' || true)"
  local total
  total="$(count_rustc_errors "$build_log")"

  printf 'path=HostNative_whole_tree_seed\tmodule=%s\thead=%s\tcargo_ran=%s\te0308_all=%s\te0308_infer_files=%s\ttotal_rustc_errors=%s\tfirst_error=%s\n' \
    "$MODULE_PATH" "$HEAD" "$ran" "$e0308_all" "$e0308_infer" "$total" "$(first_rustc_error "$build_log")"

  rm -f "$build_log"
}

echo "# repr_mismatch E0308 discriminant head=$HEAD module=$MODULE_PATH"
measure_cssl_faithful
measure_hostnative_whole_tree
