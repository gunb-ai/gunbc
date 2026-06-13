#!/usr/bin/env bash
# Compare per-entry claim_batch (current gate) vs one multi-entry claim_batch (lens pattern).
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
shard="${1:?usage: $0 <a|b>}"
bin_batch="${CLAIM_BATCH:-target/release/claim_batch}"
gate_model="src/v4/test/claim/workflow/claim_witness_corpus_ci_runner.dag"
rows_data="claim_witness_corpus_shard_${shard}_rows"

now_ms() { date +%s%3N; }

# Extract pass rows as entry\tfunction lines (same awk as gate).
mapfile -t pass_pairs < <(
  awk -v list="$rows_data" -v gm="$gate_model" '
    BEGIN { fn = gm }
    $0 ~ "^data " list ": " { in_list = 1; next }
    in_list && /^\]/ { in_list = 0 }
    in_list && /^  corpus_ci_gate_row_/ {
      gsub(/^  /, ""); gsub(/,.*/, ""); members[++n] = $0
    }
    END {
      for (i = 1; i <= n; i++) {
        name = members[i]
        in_row = 0; entry = ""; fn = ""; expect = ""
      }
    }
  ' "$gate_model" 2>/dev/null || true
)

# Use gate script's projection via a small inline python for reliability.
mapfile -t pass_rows < <(python3 - "$gate_model" "$rows_data" <<'PY'
import re, sys
from pathlib import Path
text = Path(sys.argv[1]).read_text()
list_name = sys.argv[2]
m = re.search(rf"data {re.escape(list_name)}:.*?\[(.*?)\]", text, re.S)
if not m:
    raise SystemExit(1)
members = [x.strip().rstrip(",") for x in m.group(1).splitlines() if x.strip()]
for member in members:
    rm = re.search(
        rf"data {re.escape(member)}: ClaimWitnessCorpusClaimRunRow = ClaimWitnessCorpusClaimRunRow \{{(.*?)\}}",
        text,
        re.S,
    )
    if not rm:
        continue
    body = rm.group(1)
    def field(name):
        fm = re.search(rf'{name}: "([^"]*)"', body)
        return fm.group(1) if fm else ""
    expect = "pass" if "ExpectPass" in body else "fail"
    if expect != "pass":
        continue
    print(f"{field('entry')}\t{field('function')}")
PY
)

declare -A entry_fns
declare -a entry_order
for line in "${pass_rows[@]}"; do
  IFS=$'\t' read -r entry fn <<< "$line"
  if [[ -z "${entry_fns[$entry]+x}" ]]; then
    entry_order+=("$entry")
  fi
  entry_fns[$entry]+="${fn},"
done

echo "shard=${shard} pass_witnesses=${#pass_rows[@]} distinct_entries=${#entry_order[@]}"

# Current gate pattern: one claim_batch per entry.
per_entry_s=0
t0=$(now_ms)
for e in "${entry_order[@]}"; do
  fns="${entry_fns[$e]%,}"
  "$bin_batch" --source-root src/v4 --entry "$e" --functions "$fns" --claim-run >/dev/null
done
per_entry_s=$(( ($(now_ms) - t0) / 1000 ))
echo "per_entry_claim_batch_wall_s=${per_entry_s}"

# Lens pattern: single multi-entry claim_batch.
multi_args=(--source-root src/v4)
for line in "${pass_rows[@]}"; do
  IFS=$'\t' read -r entry fn <<< "$line"
  multi_args+=(--entry "$entry" --function "$fn")
done
t0=$(now_ms)
"${bin_batch}" "${multi_args[@]}" --claim-run >/dev/null
multi_s=$(( ($(now_ms) - t0) / 1000 ))
echo "multi_entry_claim_batch_wall_s=${multi_s}"
echo "delta_saved_s=$(( per_entry_s - multi_s )) (${#entry_order[@]} index builds -> 1)"
