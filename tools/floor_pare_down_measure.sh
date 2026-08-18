#!/usr/bin/env bash
# Extract in-floor witness wall times from a witnesses workflow log and rank pare-down targets.
#
# Population: witnesses with eval_wall >= 500ms (hard cutoff) and the 500-525ms
# pollable marginal band ("censored at the wall" for pare-down ranking).
#
# Usage:
#   gh run view <run-id> --log > /tmp/floor.log
#   tools/floor_pare_down_measure.sh /tmp/floor.log [run-id]
#
# Or fetch directly:
#   tools/floor_pare_down_measure.sh --run <run-id>

set -euo pipefail

if [[ "${1:-}" == "--run" ]]; then
  run_id="${2:?run id required}"
  log="$(mktemp)"
  trap 'rm -f "$log"' EXIT
  gh run view "$run_id" --log >"$log"
  exec "$0" "$log" "$run_id"
fi

log_path="${1:?usage: $0 <floor-workflow.log> [run-id]}"
run_tag="${2:-floor_log}"
out_dir="$(cd "$(dirname "$0")" && pwd)"

python3 - "$log_path" "$run_tag" "$out_dir" <<'PY'
from __future__ import annotations

import re
import sys
from collections import defaultdict
from pathlib import Path

GANTT_RE = re.compile(
    r"\^\[\[2m[^\]]+\^\[\[0m \^?\[\[31m.\^?\[\[0m ([^\^]+) "
    r"\^\[\[2m\(([^)]+)\)\^?\[\[0m ([\d.]+)ms"
)

HARD_CUTOFF_MS = 500
WARN_MS = 100
CENSORED_BAND_HI_MS = 525


def classify(entry: str, function: str) -> str:
    e = entry.lower()
    if "grounding_lens" in e or "whole_tree" in function.lower():
        return "whole_corpus_pool_parse_builtin"
    if "resolution_divergence" in e or "silent_pick" in function.lower():
        return "unpollable_host_builtin_giant"
    if "decl_facts" in e:
        return "decl_facts_reflection_scan"
    if "rust_test_fixtures_import" in e or "import_closure" in e:
        return "import_closure_scan"
    if "roster_gate" in e or "identity_captured_navigation" in e:
        return "repeated_prepare_grammar_lens_scan"
    if "where_refinement" in e:
        return "compile_dag_rust_emit_check_mini_compile"
    if "ci_heal_skew_guard" in e:
        return "ci_heal_bash_script_emission"
    if "languages_consumer_census" in e or "data_decl_ratchet" in e:
        return "languages_filesystem_census"
    if "live_read_classification" in e or e.endswith("g2_"):
        return "live_read_g2_classification"
    if "live_deploy" in e:
        return "live_deploy_script_emit"
    if "roadmap_" in e:
        return "roadmap_page_html_emit"
    if "/execution/" in e or e.startswith("v2.test.execution"):
        return "emit_host_execution_witness"
    if "dag_arrow_lambda" in e:
        return "dag_arrow_lambda_emit_fold"
    if "trait_derive" in e:
        return "rust_target_model_construction"
    return "pollable_marginal_other"


def parse_log(path: Path) -> list[tuple[float, str, str]]:
    rows: list[tuple[float, str, str]] = []
    for line in path.read_text(errors="replace").splitlines():
        if "leg=PreparedSubject)" not in line or "ms" not in line:
            continue
        m = GANTT_RE.search(line)
        if not m:
            continue
        fn = m.group(1).strip()
        entry = m.group(2).replace(" leg=PreparedSubject", "").strip()
        ms = float(m.group(3))
        rows.append((ms, entry, fn))
    return rows


def main() -> int:
    log_path = Path(sys.argv[1])
    run_tag = sys.argv[2]
    out_dir = Path(sys.argv[3])
    rows = parse_log(log_path)
    if not rows:
        print("no witness gantt rows parsed", file=sys.stderr)
        return 1

    over_hard = sorted([r for r in rows if r[0] >= HARD_CUTOFF_MS], key=lambda r: -r[0])
    censored_band = [r for r in over_hard if r[0] <= CENSORED_BAND_HI_MS]
    over_warn = [r for r in rows if r[0] >= WARN_MS]

    all_path = out_dir / f"floor_pare_down_all_ge_{HARD_CUTOFF_MS}ms_run_{run_tag}.tsv"
    band_path = out_dir / f"floor_censored_band_{HARD_CUTOFF_MS}_{CENSORED_BAND_HI_MS}ms_run_{run_tag}.tsv"
    group_path = out_dir / f"floor_pare_down_groups_ge_{HARD_CUTOFF_MS}ms_run_{run_tag}.tsv"

    def write_rows(path: Path, subset: list[tuple[float, str, str]]) -> None:
        lines = ["rank\tin_floor_wall_ms\tentry\tfunction\tcause_class\tcensored_band\n"]
        for i, (ms, entry, fn) in enumerate(subset, 1):
            cause = classify(entry, fn)
            band = "yes" if HARD_CUTOFF_MS <= ms <= CENSORED_BAND_HI_MS else "no"
            lines.append(f"{i}\t{ms:.1f}\t{entry}\t{fn}\t{cause}\t{band}\n")
        path.write_text("".join(lines))

    write_rows(all_path, over_hard)
    write_rows(band_path, sorted(censored_band, key=lambda r: -r[0]))

    groups: dict[str, list[float]] = defaultdict(list)
    for ms, entry, fn in over_hard:
        groups[classify(entry, fn)].append(ms)

    group_lines = [
        "rank\tcause_class\twitness_count\ttotal_wall_ms\tmax_wall_ms\tmedian_wall_ms\n"
    ]
    rollups = []
    for cause, times in groups.items():
        times_sorted = sorted(times)
        total = sum(times_sorted)
        med = times_sorted[len(times_sorted) // 2]
        rollups.append((total, cause, len(times_sorted), max(times_sorted), med))
    for i, (total, cause, count, mx, med) in enumerate(
        sorted(rollups, key=lambda x: -x[0]), 1
    ):
        group_lines.append(f"{i}\t{cause}\t{count}\t{total:.1f}\t{mx:.1f}\t{med:.1f}\n")
    group_path.write_text("".join(group_lines))

    print(f"parsed_gantt_rows={len(rows)}")
    print(f"over_{WARN_MS}ms_warn={len(over_warn)} (log is warn-threshold biased; not full corpus)")
    print(f"over_{HARD_CUTOFF_MS}ms_hard={len(over_hard)}")
    print(f"censored_band_{HARD_CUTOFF_MS}_{CENSORED_BAND_HI_MS}ms={len(censored_band)}")
    print(f"wrote {all_path.name}")
    print(f"wrote {band_path.name}")
    print(f"wrote {group_path.name}")
    print("\nTop 10 by in-floor wall (true measured, not budget failures):")
    for ms, entry, fn in over_hard[:10]:
        print(f"  {ms:10.1f}ms  {entry}::{fn}  [{classify(entry, fn)}]")
    return 0


raise SystemExit(main())
PY
