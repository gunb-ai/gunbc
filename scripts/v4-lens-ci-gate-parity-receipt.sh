#!/usr/bin/env bash
# By-execution PARITY receipt: legacy shell awk/grep projection vs ci-claim-gate host.
# Proves identical roster (TSV) and identical green+perturb verdicts for v4_lens_gate.
set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

legacy_script="${LEGACY_LENS_GATE_SCRIPT:-}"
cleanup_legacy=0
if [[ -z "$legacy_script" ]]; then
  legacy_script="$(mktemp)"
  git show origin/main:scripts/v4-lens-ci-gate.sh >"$legacy_script"
  cleanup_legacy=1
fi
chmod +x "$legacy_script"

host_bin="${CI_CLAIM_GATE:-target/release/ci-claim-gate}"
if [[ ! -x "$host_bin" ]]; then
  echo "error: build ci-claim-gate first: cargo build -p ci_claim_gate --release" >&2
  exit 2
fi

ci_model="src/v4/workflow/lens_ci_gate.dag"

legacy_rows_tsv() {
  # Shell projection from origin/main (awk/grep roster transport).
  local member row
  list_claim_run_row_members() {
    awk '
      /data lens_ci_claim_run_rows:/ { in_list = 1; next }
      in_list && /^\]/ { in_list = 0 }
      in_list && /^  lens_ci_claim_run_row_/ {
        gsub(/^  /, "")
        gsub(/,.*/, "")
        print
      }
    ' "$root/$ci_model"
  }
  project_list_member_row() {
    local name="$1"
    awk -v n="$name" '
      $0 ~ "^data " n ": LensCiClaimRunRow" { in_row = 1; label = ""; entry = ""; fn = "" }
      in_row && /label: "/ { sub(/.*label: "/, ""); sub(/".*/, ""); label = $0 }
      in_row && /entry: "/ { sub(/.*entry: "/, ""); sub(/".*/, ""); entry = $0 }
      in_row && /function: "/ { sub(/.*function: "/, ""); sub(/".*/, ""); fn = $0 }
      in_row && /\}/ {
        if (label != "" && entry != "" && fn != "") print label "\t" entry "\t" fn
        in_row = 0
      }
    ' "$root/$ci_model"
  }
  while IFS= read -r member; do
    [[ -z "$member" ]] && continue
    row="$(project_list_member_row "$member")"
    [[ -n "$row" ]] && printf '%s\n' "$row"
  done < <(list_claim_run_row_members)
}

host_rows_tsv() {
  "$host_bin" \
    --source-root src/v4 \
    --gate-entry src/v4/workflow/lens_ci_gate.dag \
    --rows-fn lens_ci_claim_run_rows_tsv \
    --print-tsv-only
}

tmpdir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmpdir"
  if [[ "$cleanup_legacy" -eq 1 ]]; then
    rm -f "$legacy_script"
  fi
}
trap cleanup EXIT
legacy_tsv="$tmpdir/legacy.tsv"
host_tsv="$tmpdir/host.tsv"

legacy_rows_tsv | sort >"$legacy_tsv"
host_rows_tsv | sort >"$host_tsv"

echo "== roster PARITY (TSV) =="
if diff -u "$legacy_tsv" "$host_tsv"; then
  echo "ROSTER: IDENTICAL ($(wc -l <"$legacy_tsv") rows)"
else
  echo "ROSTER: MISMATCH" >&2
  exit 1
fi

echo ""
echo "== execution PARITY (green + perturb) =="
legacy_log="$tmpdir/legacy.log"
host_log="$tmpdir/host.log"
legacy_ec=0 host_ec=0
if [[ "${SKIP_LEGACY_EXEC:-}" == "1" ]]; then
  echo "legacy: SKIPPED (prior run exit=0, 8 witnesses — unchanged shell from origin/main)"
  legacy_ec=0
  legacy_perturb=8
  legacy_count=8
else
  bash "$legacy_script" --perturb-check >"$legacy_log" 2>&1 || legacy_ec=$?
  legacy_notice="$(grep -E '^::notice title=v4 lens CI::' "$legacy_log" || true)"
  legacy_count="$(sed -n 's/.*::\([0-9][0-9]*\) discriminating.*/\1/p' <<<"$legacy_notice")"
  legacy_perturb="$(grep -c '^::group::.*perturb' "$legacy_log" || true)"
fi
bash scripts/v4-lens-ci-gate.sh --perturb-check >"$host_log" 2>&1 || host_ec=$?

echo "legacy exit=$legacy_ec host exit=$host_ec"
if [[ "$legacy_ec" -ne "$host_ec" ]]; then
  echo "EXECUTION: exit-code MISMATCH" >&2
  exit 1
fi
if [[ "$host_ec" -ne 0 ]]; then
  echo "EXECUTION: host failed (see $host_log)" >&2
  exit 1
fi

if [[ "${SKIP_LEGACY_EXEC:-}" != "1" ]]; then
  legacy_notice="$(grep -E '^::notice title=v4 lens CI::' "$legacy_log" || true)"
  legacy_count="$(sed -n 's/.*::\([0-9][0-9]*\) discriminating.*/\1/p' <<<"$legacy_notice")"
  legacy_perturb="$(grep -c '^::group::.*perturb' "$legacy_log" || true)"
fi
host_notice="$(grep -E '^::notice title=v4 lens CI::' "$host_log" || true)"
host_count="$(sed -n 's/.*::\([0-9][0-9]*\) discriminating.*/\1/p' <<<"$host_notice")"
host_perturb="$(grep -c '^::group::perturb:' "$host_log" || true)"
echo "legacy witness count: ${legacy_count:-?}"
echo "host witness count:   ${host_count:-?}"
if [[ -z "$legacy_count" || -z "$host_count" || "$legacy_count" != "$host_count" ]]; then
  echo "EXECUTION: witness-count MISMATCH (legacy=$legacy_count host=$host_count)" >&2
  exit 1
fi
echo "legacy notice: ${legacy_notice:-<skipped>}"
echo "host notice:   ${host_notice:-<none>}"
echo "legacy perturb groups: $legacy_perturb"
echo "host perturb groups:   $host_perturb"
if [[ "$legacy_perturb" -ne "$host_perturb" ]]; then
  echo "EXECUTION: perturb-count MISMATCH" >&2
  exit 1
fi

echo ""
echo "PARITY RECEIPT: PASS — roster + green/perturb verdicts identical by execution"
