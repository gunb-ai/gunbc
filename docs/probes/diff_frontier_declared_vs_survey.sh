#!/usr/bin/env bash
# Compare declared frontier.dag measured_probe facts against an independent survey TSV.
# Refuses (non-zero exit) on mismatch — oracle on each side, never writes rows.
# SCAFFOLD — dissolve-on: totality-wall witness binds roster↔manifest without this shell diff.
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
FRONTIER="$ROOT/src/v2/compiler/self_host/frontier.dag"
TSV="${1:-$ROOT/docs/probes/frontier_probe_exact_head_survey_2026-08-03.tsv}"

if [[ ! -f "$TSV" ]]; then
  echo "diff_frontier_declared_vs_survey: missing TSV: $TSV" >&2
  exit 2
fi
if [[ ! -f "$FRONTIER" ]]; then
  echo "diff_frontier_declared_vs_survey: missing frontier: $FRONTIER" >&2
  exit 2
fi

python3 - "$FRONTIER" "$TSV" <<'PY'
import re
import sys
from pathlib import Path

frontier_path, tsv_path = sys.argv[1], sys.argv[2]
text = Path(frontier_path).read_text()

survey: dict[str, tuple[str, str, str]] = {}
with open(tsv_path) as f:
    f.readline()
    for line in f:
        parts = line.rstrip("\n").split("\t")
        if len(parts) < 5:
            continue
        module, _, blocker, stage, reason = parts[:5]
        survey[module] = (blocker, stage, reason)

# Extract declared measured_probe per module_path from roster rows.
declared: dict[str, dict[str, str]] = {}
row_re = re.compile(
    r"data compiler_frontier_row_\w+: CompilerModuleFrontierRow = [\s\S]*?\n\)",
    re.MULTILINE,
)
for block in row_re.findall(text):
    path_m = re.search(r'module_path: "([^"]+)"', block)
    if not path_m:
        continue
    path = path_m.group(1)
    blocker_m = re.search(r"measured_blocker: ([^\n,]+)", block)
    stage_m = re.search(r"located_stage: (ProbeStage\w+)", block)
    reason_m = re.search(r"located_reason: (\^?\w+)", block)
    if not blocker_m or not stage_m or not reason_m:
        continue
    blocker = blocker_m.group(1).strip()
    if blocker == "compiler_frontier_measured_upstream_unresolved_blocker":
        blocker = "UpstreamSemanticRefusal"
    declared[path] = {
        "blocker": blocker,
        "stage": stage_m.group(1),
        "reason": reason_m.group(1),
    }

mismatches = []
missing_declared = []
missing_survey = []

for module, (s_blocker, s_stage, s_reason) in sorted(survey.items()):
    if module not in declared:
        missing_declared.append(module)
        continue
    d = declared[module]
    fields = []
    if d["blocker"] != s_blocker:
        fields.append(f"blocker declared={d['blocker']} survey={s_blocker}")
    if d["stage"] != s_stage:
        fields.append(f"stage declared={d['stage']} survey={s_stage}")
    d_reason = d["reason"]
    if not d_reason.startswith("^"):
        d_reason = f"^{d_reason}"
    if d_reason != s_reason:
        fields.append(f"reason declared={d_reason} survey={s_reason}")
    if fields:
        mismatches.append((module, fields))

for module in sorted(declared):
    if module not in survey:
        missing_survey.append(module)

print("=== frontier declared vs survey oracle ===")
print(f"survey rows: {len(survey)}")
print(f"declared rows scanned: {len(declared)}")
print(f"mismatches: {len(mismatches)}")
for module, fields in mismatches:
    print(f"MISMATCH {module}")
    for f in fields:
        print(f"  {f}")
if missing_declared:
    print(f"missing declared row for {len(missing_declared)} survey module(s)")
    for m in missing_declared:
        print(f"  NO_ROW {m}")
if missing_survey:
    print(f"survey missing {len(missing_survey)} declared module(s) (ok if partial survey)")

if mismatches or missing_declared:
    sys.exit(1)
print("OK: all surveyed modules match declared measured_probe")
PY
