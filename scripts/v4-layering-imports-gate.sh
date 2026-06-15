#!/usr/bin/env bash
# Uniform `.dag`-driven layering-imports CI Bool-witness gate transport.
#
# Green pass: layering_imports_scan on the live repo -> clean_tree witness via
# ci-claim-gate; optional lens-unit rows (hardcoded facts, semantics-only).
#
# Perturb pass (--perturb-check): host scanner-execution slice — plant a violation
# under each layer root in a temp tree, run layering_imports_scan, assert the
# scanner-execution witness goes green and clean_tree goes red; remove the plant,
# rescan, assert clean_tree returns green. Mirrors scripts/v4-substrate-equivalence-gate.sh.

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

claim_gate_bin="${CI_CLAIM_GATE:-target/release/ci-claim-gate}"
gunbc_bin="${V2_COMPILER:-target/release/gunbc}"
scan_bin="${LAYERING_IMPORTS_SCAN:-target/release/layering_imports_scan}"
discover_sh="$root/scripts/v4-layering-imports-discover.sh"
gate_entry="src/v4/workflow/layering_imports_gate.dag"
clean_tree_entry="src/v4/test/claim/layering_imports/clean_tree.dag"
clean_tree_fn="clean_tree_no_wrong_direction_imports_holds"
perturb=0

case "${1:-}" in
  --perturb-check) perturb=1 ;;
  "") ;;
  *)
    echo "usage: $0 [--perturb-check]" >&2
    exit 2
    ;;
esac

if [[ ! -x "$claim_gate_bin" ]]; then
  echo "error: ci-claim-gate not found at $claim_gate_bin (build: cargo build -p ci_claim_gate --release)" >&2
  exit 2
fi

if [[ ! -x "$gunbc_bin" ]]; then
  echo "error: gunbc not found at $gunbc_bin (build: cargo build -p v2-compiler --release)" >&2
  exit 2
fi

if [[ ! -x "$scan_bin" ]]; then
  echo "error: layering_imports_scan not found at $scan_bin (build: cargo build -p layering_imports_scan --release)" >&2
  exit 2
fi

run_witness() {
  local manifest_dir="$1"
  local entry="$2"
  local function="$3"
  "$gunbc_bin" run \
    --source-root src/v4 \
    --source-root "$manifest_dir" \
    --entry "$entry" \
    --function "$function" \
    --claim-run
}

emit_manifest() {
  local repo_root="$1"
  local manifest="$2"
  LAYERING_IMPORTS_REPO_ROOT="$repo_root" \
    V4_LAYERING_IMPORTS_MANIFEST="$manifest" \
    bash "$discover_sh" >/dev/null
}

plant_file() {
  local repo_root="$1"
  local relpath="$2"
  local body="$3"
  local dest="$repo_root/$relpath"
  mkdir -p "$(dirname "$dest")"
  printf '%s' "$body" >"$dest"
}

run_scanner_perturb_case() {
  local label="$1"
  local planted_rel="$2"
  local file_body="$3"
  local detect_entry="$4"
  local detect_fn="$5"

  echo "::group::scanner perturb: $label"
  local tmp manifest manifest_dir
  tmp="$(mktemp -d)"
  manifest="$tmp/v4-layering-imports-manifest.dag"
  manifest_dir="$tmp"

  plant_file "$tmp" "$planted_rel" "$file_body"
  emit_manifest "$tmp" "$manifest"

  if ! run_witness "$manifest_dir" "$detect_entry" "$detect_fn"; then
    echo "::error::scanner perturb ${label}: detection witness failed on planted scan"
    rm -rf "$tmp"
    exit 1
  fi

  if run_witness "$manifest_dir" "$clean_tree_entry" "$clean_tree_fn"; then
    echo "::error::scanner perturb ${label}: clean_tree still passed with planted violation"
    rm -rf "$tmp"
    exit 1
  fi

  rm -f "$tmp/$planted_rel"
  emit_manifest "$tmp" "$manifest"
  if ! run_witness "$manifest_dir" "$clean_tree_entry" "$clean_tree_fn"; then
    echo "::error::scanner perturb ${label}: clean_tree failed after removing planted file"
    rm -rf "$tmp"
    exit 1
  fi

  rm -rf "$tmp"
  echo "::endgroup::"
}

run_scanner_perturb_receipts() {
  run_scanner_perturb_case \
    "src/v4/std × v4.compiler.* prefix" \
    "src/v4/std/_perturb_scanner_std_v4_prefix.dag" \
    $'module v4.std._perturb_scanner_std_v4_prefix\nimport v4.compiler.tokenize { tokenize }\ndata perturb: Bool = true\n' \
    "src/v4/test/claim/layering_imports/scanner/std_v4_prefix.dag" \
    "scanner_std_v4_prefix_detects_holds"

  run_scanner_perturb_case \
    "src/v4/extdeps × v4.compiler.* prefix" \
    "src/v4/extdeps/_perturb_scanner_extdeps_v4_prefix.dag" \
    $'module v4.extdeps._perturb_scanner_extdeps_v4_prefix\nimport v4.compiler.parse { parse }\ndata perturb: Bool = true\n' \
    "src/v4/test/claim/layering_imports/scanner/extdeps_v4_prefix.dag" \
    "scanner_extdeps_v4_prefix_detects_holds"

  run_scanner_perturb_case \
    "src/v3/std × v3.compiler exact" \
    "src/v3/std/_perturb_scanner_v3_std_exact.dag" \
    $'module v3.std._perturb_scanner_v3_std_exact\nimport v3.compiler\ndata perturb: Bool = true\n' \
    "src/v4/test/claim/layering_imports/scanner/v3_std_exact.dag" \
    "scanner_v3_std_exact_detects_holds"

  run_scanner_perturb_case \
    "dsl/std × v3.compiler.* prefix" \
    "dsl/std/_perturb_scanner_dsl_std_v3_prefix.dag" \
    $'module dsl.std._perturb_scanner_dsl_std_v3_prefix\nimport v3.compiler.parse { parse }\ndata perturb: Bool = true\n' \
    "src/v4/test/claim/layering_imports/scanner/dsl_std_v3_prefix.dag" \
    "scanner_dsl_std_v3_prefix_detects_holds"

  run_scanner_perturb_case \
    "dsl/extdeps × v4.compiler exact" \
    "dsl/extdeps/_perturb_scanner_dsl_extdeps_v4_exact.dag" \
    $'module dsl.extdeps._perturb_scanner_dsl_extdeps_v4_exact\nimport v4.compiler\ndata perturb: Bool = true\n' \
    "src/v4/test/claim/layering_imports/scanner/dsl_extdeps_v4_exact.dag" \
    "scanner_dsl_extdeps_v4_exact_detects_holds"

  run_scanner_perturb_case \
    "src/v4/std parse-level pre-resolve text scan" \
    "src/v4/std/_perturb_scanner_parse_level_pre_resolve.dag" \
    $'module v4.std._perturb_scanner_parse_level_pre_resolve\nimport v4.compiler.tokenize { tokenize }\nimport totally.broken.unresolved.module { foo }\ndata x: DoesNotExist = unresolved garbage {\n' \
    "src/v4/test/claim/layering_imports/scanner/parse_level_pre_resolve.dag" \
    "scanner_parse_level_pre_resolve_detects_holds"
}

manifest="$(
  LAYERING_IMPORTS_REPO_ROOT="$root" \
    V4_LAYERING_IMPORTS_MANIFEST="${V4_LAYERING_IMPORTS_MANIFEST:-target/v4-layering-imports-manifest.dag}" \
    bash "$discover_sh"
)"
manifest_dir="$(dirname "$manifest")"

"$claim_gate_bin" \
  --source-root src/v4 \
  --source-root "$manifest_dir" \
  --gate-entry "$gate_entry" \
  --rows-fn layering_imports_claim_run_rows_tsv \
  --notice-title "layering imports host scan"

"$claim_gate_bin" \
  --source-root src/v4 \
  --gate-entry "$gate_entry" \
  --rows-fn layering_imports_lens_unit_claim_run_rows_tsv \
  --notice-title "layering imports lens unit"

if [[ "$perturb" -eq 1 ]]; then
  run_scanner_perturb_receipts
fi

echo "::notice title=layering imports::host scanner-execution receipts passed"
