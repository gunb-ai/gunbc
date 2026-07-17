#!/usr/bin/env bash
# SCAFFOLD — dissolve-on: tools.self_host_curated_seed_linked_harness on main post-#6782
# (+ generic std-seed-link follow-up) retires hand-shell Cargo.toml projection; until then
# this helper reads cssl_v1_compiled_cargo_toml via dag/tools/self_host_curated_probe_cargo.dag
# (single authority — no parallel dependency manifest). dissolve-on alt: gunbc bash-emit #5828.
set -euo pipefail

render_cssl_probe_lib_cargo_toml() {
  local root="$1"
  local out_path="$2"
  local gunbc="${GUNBC:-$root/target/release/gunbc}"
  local harness_rel="dag/tools/self_host_curated_seed_linked_harness.dag"
  local tmp_auth=""

  if [[ ! -f "$root/$harness_rel" ]]; then
    if ! git show "origin/session/snappy-ferret-198:$harness_rel" >/dev/null 2>&1; then
      echo "curated_cargo_probe: missing $harness_rel (checkout ferret branch or wait for #6782 on main)" >&2
      return 1
    fi
    tmp_auth="$root/target/cssl-harness-authority-snapshot"
    mkdir -p "$tmp_auth/dag/tools"
    git show "origin/session/snappy-ferret-198:$harness_rel" >"$tmp_auth/$harness_rel"
  fi

  if [[ ! -x "$gunbc" ]]; then
    echo "curated_cargo_probe: gunbc not built at $gunbc" >&2
    return 1
  fi

  local -a source_roots=(dag src/v2)
  if [[ -n "$tmp_auth" ]]; then
    source_roots=("target/cssl-harness-authority-snapshot" "${source_roots[@]}")
  fi

  local witness_toml
  witness_toml="$(
    cd "$root"
    "$gunbc" run \
      "${source_roots[@]/#/--source-root }" \
      --entry dag/tools/self_host_curated_probe_cargo.dag \
      --function curated_probe_cargo_toml_from_cssl_authority 2>/dev/null
  )"

  if [[ -n "$tmp_auth" ]]; then
    rm -rf "$tmp_auth"
  fi

  if [[ -z "$witness_toml" ]]; then
    echo "curated_cargo_probe: failed to read cssl_v1_compiled_cargo_toml authority" >&2
    return 1
  fi

  # Probe uses `cargo build --lib`; project witness [[bin]] manifest from cssl authority.
  printf '%s\n' "$witness_toml" \
    | sed '/^\[\[bin\]\]/,/^path = /d' \
    | sed '/^name = "witness"$/d' \
    >"$out_path"
  {
    echo ""
    echo "[lib]"
    echo 'path = "src/lib.rs"'
  } >>"$out_path"
}
