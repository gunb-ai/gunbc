#!/usr/bin/env bash
# SCAFFOLD — dissolve-on: tools.self_host_curated_seed_linked_harness on main post-#6782
# (+ generic std-seed-link follow-up) retires this hand-shell Cargo.toml reader; until then
# this helper reads cssl_v1_compiled_probe_lib_cargo_toml via dag/tools/self_host_curated_probe_cargo.dag
# (single authority — no parallel dependency manifest, no sed fork). dissolve-on alt: gunbc bash-emit #5828.
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

  local tmp_toml err_log
  tmp_toml="$(mktemp "${TMPDIR:-/tmp}/cssl-probe-cargo.XXXXXX")"
  # shellcheck disable=SC2064
  trap "rm -f '$tmp_toml'" RETURN

  if ! (
    cd "$root"
    "$gunbc" run \
      --source-root dag \
      --source-root src/v2 \
      --entry dag/tools/self_host_curated_probe_cargo.dag \
      --function curated_probe_cargo_toml_from_cssl_authority \
      --arg "out_path=$tmp_toml" >/dev/null
  ); then
    err_log="$(
      cd "$root"
      "$gunbc" run \
        --source-root dag \
        --source-root src/v2 \
        --entry dag/tools/self_host_curated_probe_cargo.dag \
        --function curated_probe_cargo_toml_from_cssl_authority \
        --arg "out_path=$tmp_toml" 2>&1 >/dev/null || true
    )"
    echo "curated_cargo_probe: gunbc authority-read failed" >&2
    [[ -n "$err_log" ]] && echo "$err_log" >&2
    return 1
  fi

  if [[ ! -s "$tmp_toml" ]]; then
    echo "curated_cargo_probe: gunbc authority-read produced empty Cargo.toml" >&2
    return 1
  fi

  cp "$tmp_toml" "$out_path"
}
