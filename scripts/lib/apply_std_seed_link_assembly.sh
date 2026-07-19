#!/usr/bin/env bash
# MEASUREMENT-ONLY std-seed-link assembly (throwaway probe tooling — not merge-shipped substrate).
# Faithful rule: entry module .rs bytes UNTOUCHED; only dependency modules seed-linked.
# dissolve-on: modeled cssl_seed_linked_probe_assembly in ferret #6782 harness authority.
set -euo pipefail

_seed_mod_exists() {
  local mod="$1"
  grep -qE "^pub mod ${mod};" "$SEED_LIB_RS"
}

_write_seed_reexport_shim() {
  local dest="$1"
  local seed_mod="$2"
  cat >"$dest" <<EOF
// MEASUREMENT std-seed-link shim — pub use v1_compiler::${seed_mod}
#![allow(clippy::all, dead_code, unused_imports)]
pub use v1_compiler::${seed_mod}::*;
EOF
}

_strip_emitter_v1_rt_imports_for_local_defs() {
  local f="$1"
  [[ -f "$f" ]] || return 0
  # Rust cssl_assemble is authority; bash mirror is best-effort for Witness/Optional only.
  if grep -qE '^pub enum Witness' "$f"; then
    sed -i '/^use crate::v1_rt::Witness;$/d' "$f"
    sed -i '/^use crate::v1_rt::Witness::/d' "$f"
  fi
  if grep -qE '^pub enum Optional' "$f"; then
    sed -i '/^use crate::v1_rt::Optional;$/d' "$f"
    sed -i '/^use crate::v1_rt::Optional::/d' "$f"
  fi
}

_sanitize_emitter_artifact_in_place() {
  local f="$1"
  _strip_emitter_v1_rt_imports_for_local_defs "$f"
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
  local entry_mod entry_file entry_hash_before entry_hash_after
  entry_mod="$(dag_entry_rust_module "$dag_path")"
  entry_file="$src_dir/${entry_mod}.rs"

  [[ -f "$entry_file" ]] || {
    echo "apply_std_seed_link_assembly: missing entry $entry_file" >&2
    return 1
  }

  entry_hash_before="$(sha256sum "$entry_file" | awk '{print $1}')"

  mapfile -t all_mods < <(grep '^pub mod ' "$src_dir/lib.rs" | sed 's/pub mod \([^;]*\);/\1/')

  for mod in "${all_mods[@]}"; do
    case "$mod" in
      NonEmptyVec|NonEmptyBTreeSet|"$entry_mod")
        continue
        ;;
      std_*)
        # Keep gunbc-emitted std_* — seed re-export shims break v2 emit surface (e.g. FreeMonoid).
        _sanitize_emitter_artifact_in_place "$src_dir/$mod.rs"
        ;;
      v2_compiler_*|extdeps_*|v1_compiler_*)
        # Closure manifest membership → emit-retain; seed pub mod is not a replace trigger.
        _sanitize_emitter_artifact_in_place "$src_dir/$mod.rs"
        ;;
      v2_std_*|v1_rt|v2_extdeps_*)
        _sanitize_emitter_artifact_in_place "$src_dir/$mod.rs"
        ;;
      *)
        :
        ;;
    esac
  done

  entry_hash_after="$(sha256sum "$entry_file" | awk '{print $1}')"
  if [[ "$entry_hash_before" != "$entry_hash_after" ]]; then
    echo "apply_std_seed_link_assembly: entry module mutated — refuse" >&2
    return 1
  fi

  return 0
}
