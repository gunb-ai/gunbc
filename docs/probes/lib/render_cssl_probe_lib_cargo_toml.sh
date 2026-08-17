#!/usr/bin/env bash
# SCAFFOLD — dissolve-on: tools.self_host_curated_seed_linked_harness on main post-#6782
# (+ generic std-seed-link follow-up) retires this hand-shell Cargo.toml reader; until then
# this helper invokes curated_probe_cargo_toml_write_from_cssl_authority via dag/tools/self_host_curated_probe_cargo.dag
# (ProcessExit + extdeps.filesystem.filesystem_io Filesystem.Write — no stdout capture, no NotProcessExit
# value-printing seam; #8286 wall). dissolve-on alt: gunbc bash-emit #5828.
set -euo pipefail

render_cssl_probe_lib_cargo_toml() {
  local root="$1"
  local out_path="$2"
  local gunbc="${GUNBC:-$root/target/release/gunbc}"
  local harness_rel="dag/tools/self_host_curated_seed_linked_harness.dag"

  if [[ ! -f "$root/$harness_rel" ]]; then
    echo "curated_cargo_probe: missing $harness_rel" >&2
    return 1
  fi

  if [[ ! -x "$gunbc" ]]; then
    echo "curated_cargo_probe: gunbc not built at $gunbc" >&2
    return 1
  fi

  if ! (
    cd "$root"
    "$gunbc" run \
      --source-root dag \
      --source-root src/v2 \
      --entry dag/tools/self_host_curated_probe_cargo.dag \
      --function curated_probe_cargo_toml_write_from_cssl_authority \
      --arg "out_path=$out_path"
  ); then
    echo "curated_cargo_probe: gunbc authority-write failed" >&2
    return 1
  fi

  if [[ ! -s "$out_path" ]]; then
    echo "curated_cargo_probe: missing or empty manifest at $out_path" >&2
    return 1
  fi
}
