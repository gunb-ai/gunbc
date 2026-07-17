#!/usr/bin/env bash
# PROTOTYPE — generic std-seed-link assembly for curated cargo probes.
# dissolve-on: lands in tools.self_host_curated_seed_linked_harness post-#6782 (ferret authority).
# After gunbc emit: replace seed-resident std_* / extdeps_* / v2_compiler_* (when in seed)
# with `pub use v1_compiler::…` shims; keep gunbc-emitted v2_std_* + non-seed closure;
# sanitize emitter duplicate-definition artifacts (Int128…UInt8, Witness) in kept files.
set -euo pipefail

_seed_mod_exists() {
  local mod="$1"
  grep -qE "^pub mod ${mod};" "$SEED_LIB_RS"
}

_strip_v2_std_integer_inhabitant_dupes() {
  local f="$1"
  [[ -f "$f" ]] || return 0
  sed -i '/^pub struct Int128;$/,/^pub struct UInt8;$/d' "$f"
}

_sanitize_v2_std_witness_self_conflict() {
  local f="$1"
  [[ -f "$f" ]] || return 0
  [[ "$(basename "$f")" == "v2_std_witness.rs" ]] || return 0
  if grep -q '^pub enum Witness' "$f"; then
    sed -i '/^use crate::v1_rt::Witness;$/d' "$f"
    sed -i '/^use crate::v1_rt::Witness::/d' "$f"
  fi
}

_sanitize_kept_v2_std_file() {
  local f="$1"
  _strip_v2_std_integer_inhabitant_dupes "$f"
  _sanitize_v2_std_witness_self_conflict "$f"
}

_write_seed_reexport_shim() {
  local dest="$1"
  local seed_mod="$2"
  cat >"$dest" <<EOF
// PROTOTYPE std-seed-link shim — pub use v1_compiler::${seed_mod}
#![allow(clippy::all, dead_code, unused_imports)]
pub use v1_compiler::${seed_mod}::*;
EOF
}

dag_entry_rust_module() {
  local dag_path="$1"
  local mod_line
  mod_line="$(grep -m1 '^module ' "$dag_path" | sed 's/^module //')"
  echo "${mod_line//./_}"
}

apply_std_seed_link_assembly() {
  local out_dir="$1"
  local dag_path="$2"
  local root="${3:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"

  SEED_LIB_RS="$root/src/v1/stage0/src/lib.rs"
  local src_dir="$out_dir/src"
  local entry_mod
  entry_mod="$(dag_entry_rust_module "$dag_path")"
  local emitted_lib="$src_dir/lib.rs"

  [[ -f "$emitted_lib" ]] || {
    echo "apply_std_seed_link_assembly: missing $emitted_lib" >&2
    return 1
  }

  mapfile -t all_mods < <(grep '^pub mod ' "$emitted_lib" | sed 's/pub mod \([^;]*\);/\1/')

  for mod in "${all_mods[@]}"; do
    case "$mod" in
      NonEmptyVec|NonEmptyBTreeSet)
        continue
        ;;
      "$entry_mod")
        continue
        ;;
      std_*)
        if _seed_mod_exists "$mod"; then
          _write_seed_reexport_shim "$src_dir/$mod.rs" "$mod"
        fi
        ;;
      v2_std_*)
        # Keep full gunbc emit for v2_std_* (usv_pilot stubs are incomplete); sanitize dupes.
        _sanitize_kept_v2_std_file "$src_dir/$mod.rs"
        ;;
      v2_compiler_*|extdeps_*|v1_compiler_*)
        if _seed_mod_exists "$mod"; then
          _write_seed_reexport_shim "$src_dir/$mod.rs" "$mod"
        fi
        ;;
      v1_rt)
        # Keep emitted v1_rt — seed re-export changes Witness resolution for v2_std.witness.
        _sanitize_kept_v2_std_file "$src_dir/$mod.rs"
        ;;
      v2_extdeps_*)
        :
        ;;
      *)
        :
        ;;
    esac
  done

  return 0
}
