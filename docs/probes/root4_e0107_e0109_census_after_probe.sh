#!/usr/bin/env bash
# Generate post-fix Root-4 E0107/E0109 census TSV (canonical-seven, CSSL_STD_SEED_LINK=1).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

PROBE_DIR="${PROBE_DIR:-$ROOT/target/root4-census-after-probe}"
mkdir -p "$PROBE_DIR/logs"
export PROBE_KEEP_LOG_DIR="$PROBE_DIR/logs"

MODULES=(
  src/v2/compiler/06_translate.dag
  src/v2/compiler/04_infer.dag
  src/v2/compiler/05_eval.dag
  src/v2/compiler/05_emit.dag
  src/v2/compiler/emit_host.dag
  src/v2/compiler/emit_module.dag
  src/v2/compiler/materialization_carriers.dag
)

PROBE_TSV="$PROBE_DIR/probe_lines.tsv"
: >"$PROBE_TSV"

for mod in "${MODULES[@]}"; do
  echo "probing $mod..." >&2
  CSSL_STD_SEED_LINK=1 docs/probes/curated_cargo_probe_one.sh "$mod" >>"$PROBE_TSV"
done

OUT_TSV="${1:-$ROOT/docs/probes/root4_e0107_e0109_census_AFTER_2026-07-26.tsv}"
GIT_SHA="$(git rev-parse HEAD)"
GUNBC_SHA="$(sha256sum "$ROOT/target/release/gunbc" | awk '{print $1}')"
BEFORE_TSV="$ROOT/docs/probes/root4_e0107_e0109_census_2026-07-26.tsv"

python3 - "$PROBE_TSV" "$PROBE_DIR/logs" "$OUT_TSV" "$GIT_SHA" "$GUNBC_SHA" "$BEFORE_TSV" <<'PY'
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

probe_tsv, log_dir, out_tsv, git_sha, gunbc_sha, before_tsv = sys.argv[1:7]
log_dir = Path(log_dir)

modules = [
    "src/v2/compiler/06_translate.dag",
    "src/v2/compiler/04_infer.dag",
    "src/v2/compiler/05_eval.dag",
    "src/v2/compiler/05_emit.dag",
    "src/v2/compiler/emit_host.dag",
    "src/v2/compiler/emit_module.dag",
    "src/v2/compiler/materialization_carriers.dag",
]

def parse_histogram(s: str) -> Counter:
    counts = Counter()
    for m in re.finditer(r"error\[(E\d+)\]:(\d+)", s):
        counts[m.group(1)] += int(m.group(2))
    return counts

def classify_e0107(msg: str) -> str:
    if "missing generics for struct `Measure`" in msg:
        return "measure_missing_generics"
    if "type alias takes 0 generic arguments" in msg:
        return "wrong_generic_arity_emit"
    if msg.startswith("missing generics for"):
        return "missing_generics_emit_other"
    return "E0107_unclassified"

def classify_e0109(msg: str) -> str:
    return "checkpoint_scalar_phantom"

probe_rows = {}
with open(probe_tsv, encoding="utf-8") as fh:
    for line in fh:
        parts = line.rstrip("\n").split("\t")
        if len(parts) < 7:
            continue
        mod = parts[0]
        probe_rows[mod] = {
            "emit": parts[1],
            "cargo": parts[2],
            "first_error": parts[3],
            "histogram": parts[6] if len(parts) > 6 else "",
        }

detail_rows = []
msg_totals = Counter()
module_buckets = defaultdict(lambda: Counter())
module_e0107 = Counter()
module_e0109 = Counter()
hist_totals = Counter()

for mod in modules:
    log_name = Path(mod).name.replace(".dag", ".cargo.log")
    log_path = log_dir / log_name
    if not log_path.exists():
        continue
    text = log_path.read_text(encoding="utf-8", errors="replace")
    hist_totals.update(parse_histogram(probe_rows.get(mod, {}).get("histogram", "")))
    for m in re.finditer(
        r"error\[(E0107|E0109)\]:\s*(.+?)(?=\n\s*-->\s*|\nerror\[|\Z)",
        text,
        flags=re.S,
    ):
        code = m.group(1)
        msg = " ".join(m.group(2).split())
        loc_m = re.search(
            rf"error\[{code}\]:[^\n]*\n\s*-->\s*(\S+):(\d+):(\d+)",
            text[m.start() : m.start() + 500],
        )
        loc = f"--> {loc_m.group(1)}:{loc_m.group(2)}:{loc_m.group(3)}" if loc_m else ""
        if code == "E0107":
            bucket = classify_e0107(msg)
            module_e0107[mod] += 1
            module_buckets[mod][bucket] += 1
            msg_totals[(bucket, code, msg)] += 1
            detail_rows.append((mod, code, bucket, msg, loc))
        else:
            bucket = classify_e0109(msg)
            module_e0109[mod] += 1
            module_buckets[mod][bucket] += 1
            msg_totals[(bucket, code, msg)] += 1
            detail_rows.append((mod, code, bucket, msg, loc))

total_e0107 = sum(module_e0107.values())
total_e0109 = sum(module_e0109.values())
bucket_totals = Counter()
for mod in modules:
    for b, n in module_buckets[mod].items():
        bucket_totals[b] += n

lines = []
lines.append("label\tRoot-4_E0107_E0109_census_AFTER")
lines.append("classifier_stamp\trule1-first-error-plus-residual-histogram-v3-uncoded-split")
lines.append("protocol\tCSSL_STD_SEED_LINK=1; curated_cargo_probe_one.sh spine; empty shim")
lines.append(f"git_sha\t{git_sha}")
lines.append(f"gunbc_sha\t{gunbc_sha}")
lines.append(f"before_receipt\t{Path(before_tsv).name}")
lines.append(f"total_E0107\t{total_e0107}")
lines.append(f"total_E0109\t{total_e0109}")
lines.append(f"total_E0107_plus_E0109\t{total_e0107 + total_e0109}")
for code in ("E0308", "E0599", "E0277", "E0369"):
    lines.append(f"total_{code}\t{hist_totals.get(code, 0)}")
lines.append("")
lines.append("bucket\tcount\tshare_of_E0107_E0109")
denom = total_e0107 + total_e0109 or 1
for bucket in (
    "missing_generics_emit_other",
    "wrong_generic_arity_emit",
    "measure_missing_generics",
    "checkpoint_scalar_phantom",
    "E0107_unclassified",
    "E0109_other",
):
    n = bucket_totals.get(bucket, 0)
    if n:
        lines.append(f"{bucket}\t{n}\t{n/denom:.4f}")
lines.append("")
lines.append(
    "module\tE0107\tE0109\tcheckpoint_scalar_phantom\tmeasure_missing_generics\t"
    "missing_generics_emit_other\twrong_generic_arity_emit\tE0107_unclassified\tE0109_other"
)
for mod in modules:
    b = module_buckets[mod]
    lines.append(
        f"{mod}\t{module_e0107[mod]}\t{module_e0109[mod]}\t"
        f"{b.get('checkpoint_scalar_phantom', 0)}\t{b.get('measure_missing_generics', 0)}\t"
        f"{b.get('missing_generics_emit_other', 0)}\t{b.get('wrong_generic_arity_emit', 0)}\t"
        f"{b.get('E0107_unclassified', 0)}\t{b.get('E0109_other', 0)}"
    )
lines.append("")
lines.append("normalized_message\tbucket\tE_code\tcount")
for (bucket, code, msg), n in sorted(msg_totals.items(), key=lambda x: (-x[1], x[0][2])):
    lines.append(f"{msg}\t{bucket}\t{code}\t{n}")
lines.append("")
lines.append("detail_module\tE_code\tbucket\tmessage\tlocation_sample")
for mod, code, bucket, msg, loc in detail_rows:
    lines.append(f"{mod}\t{code}\t{bucket}\t{msg}\t{loc}")
lines.append("")
lines.append("probe_module\temit\tcargo\tfirst_error\tresidual_histogram")
for mod in modules:
    row = probe_rows.get(mod, {})
    lines.append(
        f"{mod}\t{row.get('emit', '')}\t{row.get('cargo', '')}\t"
        f"{row.get('first_error', '')}\t{row.get('histogram', '')}"
    )

Path(out_tsv).write_text("\n".join(lines) + "\n", encoding="utf-8")
print(f"wrote {out_tsv}", file=sys.stderr)
print(f"E0107={total_e0107} E0109={total_e0109} buckets={dict(bucket_totals)}", file=sys.stderr)
print(
    f"E0308={hist_totals.get('E0308',0)} E0599={hist_totals.get('E0599',0)} "
    f"E0277={hist_totals.get('E0277',0)} E0369={hist_totals.get('E0369',0)}",
    file=sys.stderr,
)
PY

echo "AFTER census: $OUT_TSV" >&2
