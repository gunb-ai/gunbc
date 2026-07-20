#!/usr/bin/env bash
# SCAFFOLD — dissolve-on: tools.self_host_curated_seed_linked_harness on main post-#6782
# (+ generic std-seed-link follow-up) retires this hand-shell histogram orchestrator; until
# then it runs the cssl emit+assemble+cargo spine per frontier module and aggregates full
# E-code histograms (all codes + counts + sites, NOT first_error).
# dissolve-on alt: gunbc bash-emit #5828 / modeled cssl_probe sweep transport in .dag.
# Authority: cssl_v1_compiled_cargo_toml via dag/tools/self_host_curated_probe_cargo.dag
# (scripts/lib/render_cssl_probe_lib_cargo_toml.sh — no parallel Cargo.toml heredoc).
# One-shot Gate-2 confidence probe (swift-bee-614 Phase 1, 2026-07-20).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
export CSSL_STD_SEED_LINK=1
GUNBC="$ROOT/target/release/gunbc"
CSSL_ASSEMBLE="$ROOT/target/release/cssl_assemble"
PROBE_LIB="$ROOT/scripts/lib/render_cssl_probe_lib_cargo_toml.sh"
HEAD="$(git rev-parse --short=12 HEAD)"
DATE_TAG="2026-07-20"
OUT_DIR="$ROOT/docs/probes"
REPORT="$OUT_DIR/confidence_probe_gate2_histogram_${DATE_TAG}.tsv"
SUMMARY="$OUT_DIR/confidence_probe_gate2_histogram_${DATE_TAG}.md"

MODULES=(
  src/v2/compiler/00_compile.dag
  src/v2/compiler/01_tokenize.dag
  src/v2/compiler/02_parse.dag
  src/v2/compiler/03_body_producer.dag
  src/v2/compiler/03_ingest.dag
  src/v2/compiler/03_name_resolve.dag
  src/v2/compiler/03_normalize.dag
  src/v2/compiler/03_resolve.dag
  src/v2/compiler/04_infer.dag
  src/v2/compiler/05_emit.dag
  src/v2/compiler/05_emit_orchestration.dag
  src/v2/compiler/05_eval.dag
  src/v2/compiler/06_translate.dag
  src/v2/compiler/07_target_carriers.dag
  src/v2/compiler/discovery_enumeration.dag
  src/v2/compiler/emit_host.dag
  src/v2/compiler/emit_module.dag
  src/v2/compiler/emit_produced.dag
  src/v2/compiler/emit_semantic_decl.dag
  src/v2/compiler/fold_lowering.dag
  src/v2/compiler/materialization_carriers.dag
  src/v2/compiler/parse_engine_hooks.dag
  src/v2/compiler/program_assembly.dag
  src/v2/compiler/program_partition.dag
  src/v2/compiler/self_host.dag
  src/v2/compiler/source_authority.dag
  src/v2/compiler/use_site_verdict.dag
)

mkdir -p "$OUT_DIR"
{
  echo "# Gate 2 honest frontier — full E-code histogram (swift-bee-614 Phase 1)"
  echo "# main_head=$HEAD  probed_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)  CSSL_STD_SEED_LINK=1"
  echo -e "module\temit_ok\tcargo_verdict\ttotal_errors\tunique_ecodes\tecode_histogram\tall_sites"
} >"$REPORT"

AGG_ECODES="/tmp/gate2_agg_ecodes.txt"
: >"$AGG_ECODES"

for m in "${MODULES[@]}"; do
  echo "==> $m" >&2
  OUT="$(mktemp -d /tmp/gate2-probe.XXXXXX)"
  EMIT_OK=0
  if "$GUNBC" compile \
    --source-root dag \
    --source-root src/v2 \
    --entry "$m" \
    --output-dir "$OUT" \
    --target rust \
    --dependency-pool-index primary-precedence \
    >"$OUT/emit.log" 2>&1; then
    EMIT_OK=1
  fi

  CARGO_VERDICT="skip"
  TOTAL_ERRORS=0
  UNIQUE_ECODES=0
  ECODE_HIST=""
  ALL_SITES=""

  if [[ "$EMIT_OK" -eq 1 ]]; then
    if ! "$CSSL_ASSEMBLE" --out-dir "$OUT" --entry-dag "$m" --root "$ROOT" >"$OUT/assemble.log" 2>&1; then
      CARGO_VERDICT="harness_refuse"
    else
      # shellcheck source=/dev/null
      source "$PROBE_LIB"
      if render_cssl_probe_lib_cargo_toml "$ROOT" "$OUT/Cargo.toml"; then
        if (cd "$OUT" && RUSTC_WRAPPER= CTRL_BUILD_WRAP_CARGO=0 cargo build --release --lib 2>"$OUT/cargo.log"); then
          CARGO_VERDICT="green"
        else
          CARGO_VERDICT="refuse"
          PARSE_OUT="$(python3 - "$OUT/cargo.log" "$AGG_ECODES" << 'PY'
import re, sys
from collections import Counter
log = open(sys.argv[1]).read()
agg = open(sys.argv[2], 'a')
ecode_pattern = re.compile(r'^error\[(E\d+)\]:', re.MULTILINE)
ecodes = ecode_pattern.findall(log)
counts = Counter(ecodes)
lines = log.split('\n')
errors = []
current_ecode = None
current_sites = []
for line in lines:
    m = ecode_pattern.match(line)
    if m:
        if current_ecode:
            errors.append((current_ecode, current_sites))
        current_ecode = m.group(1)
        current_sites = []
    sm = re.match(r'^\s*-->\s+([^:]+):(\d+):', line)
    if sm and current_ecode:
        site = f"{sm.group(1)}:{sm.group(2)}"
        if site not in current_sites:
            current_sites.append(site)
if current_ecode:
    errors.append((current_ecode, current_sites))
for ecode, sites in errors:
    for s in sites:
        agg.write(f"{ecode}\t{s}\n")
hist = ";".join(f"{e}:{c}" for e,c in sorted(counts.items(), key=lambda x:(-x[1],x[0])))
sites = ";".join(s for _,ss in errors for s in ss)
print(f"{len(ecodes)}\t{len(counts)}\t{hist}\t{sites}")
PY
)"
          IFS=$'\t' read -r TOTAL_ERRORS UNIQUE_ECODES ECODE_HIST ALL_SITES <<<"$PARSE_OUT"
        fi
      else
        CARGO_VERDICT="harness_refuse"
      fi
    fi
  else
    CARGO_VERDICT="emit_fail"
  fi

  printf '%s\t%d\t%s\t%s\t%s\t%s\t%s\n' \
    "$m" "$EMIT_OK" "$CARGO_VERDICT" "${TOTAL_ERRORS:-0}" "${UNIQUE_ECODES:-0}" "${ECODE_HIST:-}" "${ALL_SITES:-}" >>"$REPORT"
  rm -rf "$OUT"
done

python3 - "$AGG_ECODES" "$SUMMARY" "$HEAD" << 'PY'
import sys
from collections import Counter, defaultdict
agg_file, summary_file, head = sys.argv[1], sys.argv[2], sys.argv[3]
ecode_sites = defaultdict(list)
ecode_counts = Counter()
for line in open(agg_file):
    line = line.strip()
    if not line: continue
    ecode, site = line.split('\t', 1)
    ecode_counts[ecode] += 1
    if site not in ecode_sites[ecode]:
        ecode_sites[ecode].append(site)

total = sum(ecode_counts.values())
with open(summary_file, 'w') as f:
    f.write(f"# Gate 2 Aggregate E-Code Histogram (swift-bee-614 Phase 1)\n\n")
    f.write(f"**HEAD:** `{head}`  \n")
    f.write(f"**TOTAL_ERRORS:** {total}  \n")
    f.write(f"**UNIQUE_E_CODES:** {len(ecode_counts)}  \n\n")
    f.write("## E-Code Histogram\n\n")
    f.write("| E-Code | Count | Unique Sites |\n")
    f.write("|--------|------:|-------------:|\n")
    for ecode, count in sorted(ecode_counts.items(), key=lambda x:(-x[1],x[0])):
        f.write(f"| {ecode} | {count} | {len(ecode_sites[ecode])} |\n")
    f.write("\n## Sites per E-Code\n\n")
    for ecode, count in sorted(ecode_counts.items(), key=lambda x:(-x[1],x[0])):
        f.write(f"### {ecode} ({count} occurrences, {len(ecode_sites[ecode])} unique sites)\n\n")
        for s in ecode_sites[ecode][:30]:
            f.write(f"- `{s}`\n")
        if len(ecode_sites[ecode]) > 30:
            f.write(f"- ... +{len(ecode_sites[ecode])-30} more\n")
        f.write("\n")
print(f"wrote {summary_file}")
PY

echo "wrote $REPORT" >&2
echo "wrote $SUMMARY" >&2
