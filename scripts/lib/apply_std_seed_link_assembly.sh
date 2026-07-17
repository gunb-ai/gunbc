#!/usr/bin/env bash
# PROTOTYPE — generic std-seed-link assembly for curated cargo probes.
# dissolve-on: lands in tools.self_host_curated_seed_linked_harness post-#6782 (ferret authority).
# After gunbc emit: replace re-emitted std/v2_std and seed-resident modules with
# `pub use v1_compiler::…` shims; keep only entry + non-seed v2_compiler/extdeps emit.
set -euo pipefail

_seed_mod_exists() {
  local mod="$1"
  grep -qE "^pub mod ${mod};" "$SEED_LIB_RS"
}

_v2_std_seed_target() {
  local mod="$1"
  case "$mod" in
    v2_std_algebra) echo "usv_pilot_v2_std_algebra" ;;
    v2_std_collection) echo "usv_pilot_v2_std_collection" ;;
    v2_std_node) echo "usv_pilot_v2_std_node" ;;
    v2_std_integer) echo "std_integer" ;;
    *) echo "" ;;
  esac
}

_strip_v2_std_integer_inhabitant_dupes() {
  local f="$1"
  [[ -f "$f" ]] || return 0
  # Emitter emits type aliases + closed enum witness structs with the same names (Int128…UInt8).
  sed -i '/^pub struct Int128;$/,/^pub struct UInt8;$/d' "$f"
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
      # Entry module stays gunbc-emitted.
      continue
      ;;
    std_*)
      if _seed_mod_exists "$mod"; then
        _write_seed_reexport_shim "$src_dir/$mod.rs" "$mod"
      fi
      ;;
    v2_std_*)
      local seed_target
      seed_target="$(_v2_std_seed_target "$mod")"
      if [[ -n "$seed_target" ]] && _seed_mod_exists "$seed_target"; then
        _write_seed_reexport_shim "$src_dir/$mod.rs" "$seed_target"
      elif [[ "$mod" == "v2_std_integer" ]] && _seed_mod_exists "std_integer"; then
        _write_seed_reexport_shim "$src_dir/$mod.rs" "std_integer"
      else
        _strip_v2_std_integer_inhabitant_dupes "$src_dir/$mod.rs"
      fi
      ;;
    v2_compiler_*|extdeps_*|v1_compiler_*)
      if _seed_mod_exists "$mod"; then
        _write_seed_reexport_shim "$src_dir/$mod.rs" "$mod"
      fi
      ;;
    v1_rt)
      if _seed_mod_exists "v1_rt"; then
        _write_seed_reexport_shim "$src_dir/$mod.rs" "v1_rt"
      fi
      ;;
    v2_extdeps_*)
      # v2 extdeps surface — keep gunbc emit unless a seed row appears later.
      :
      ;;
    *)
      :
      ;;
  esac
  done

  # Regenerate lib.rs: preserve emitted crate attrs + NonEmpty* helpers; module list unchanged.
  return 0
}
