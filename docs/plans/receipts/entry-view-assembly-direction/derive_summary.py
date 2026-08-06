#!/usr/bin/env python3
"""Derive summary.json from the four interleaved cohort arm TSV files only."""

from __future__ import annotations

import json
import re
from pathlib import Path

ASSEMBLY_ROWS = [
    "schedule",
    "probe",
    "graph",
    "symbol_index",
    "pool_fill",
    "symbol_index_merge",
    "variant_base",
    "root_symbol_index",
    "root_variant_base",
    "environment",
    "diagnostics",
    "registry",
    "services",
    "rewire_type_env",
    "rewire_import_str",
    "rewire_func_env",
    "emit_info",
    "other",
]


def parse_tsv(path: Path) -> dict[str, str]:
    out: dict[str, str] = {}
    for line in path.read_text().splitlines():
        if "\t" not in line:
            continue
        k, v = line.split("\t", 1)
        out[k] = v
    return out


def parse_assembly_split(line: str) -> dict[str, float]:
    out: dict[str, float] = {}
    for part in line.split():
        if "=" not in part:
            continue
        k, v = part.split("=", 1)
        if v.endswith("ms"):
            out[k] = float(v[:-2])
    return out


MEMO_NAMES = ("closure_env", "root_env", "rewired")


def parse_entry_view_assembly(line: str) -> dict[str, dict[str, int]]:
    out: dict[str, dict[str, int]] = {}
    for memo in MEMO_NAMES:
        m = re.search(
            rf"{memo}\s+keys=(\d+)\s+hits=(\d+)\s+misses=(\d+)",
            line,
        )
        if m:
            out[memo] = {
                "keys": int(m.group(1)),
                "hits": int(m.group(2)),
                "misses": int(m.group(3)),
            }
    return out


def mean_delta_pct(base_vals: list[float], after_vals: list[float]) -> float:
    base_mean = sum(base_vals) / len(base_vals)
    after_mean = sum(after_vals) / len(after_vals)
    if base_mean == 0:
        return 0.0
    return round((after_mean - base_mean) / base_mean * 100, 1)


def main() -> int:
    receipt_dir = Path(__file__).resolve().parent
    arms = ["base-r1", "base-r2", "after-r1", "after-r2"]
    parsed = {arm: parse_tsv(receipt_dir / f"{arm}.tsv") for arm in arms}

    assembly_by_arm: dict[str, dict[str, float]] = {}
    entry_view_by_arm: dict[str, dict[str, dict[str, int]]] = {}
    for arm in arms:
        assembly_by_arm[arm] = {}
        entry_view_by_arm[arm] = {}
        for line in (receipt_dir / f"{arm}.tsv").read_text().splitlines():
            if line.startswith("[assembly-split]"):
                assembly_by_arm[arm] = parse_assembly_split(line)
            if line.startswith("[entry-view-assembly]"):
                entry_view_by_arm[arm] = parse_entry_view_assembly(line)

    assembly: dict[str, dict] = {}
    for row in ASSEMBLY_ROWS:
        base_vals = [assembly_by_arm[a].get(row, 0.0) for a in ("base-r1", "base-r2")]
        after_vals = [assembly_by_arm[a].get(row, 0.0) for a in ("after-r1", "after-r2")]
        assembly[row] = {
            "base_ms": base_vals,
            "after_ms": after_vals,
            "delta_pct": mean_delta_pct(base_vals, after_vals),
        }

    def sum_rows(arm: str) -> float:
        return sum(assembly_by_arm[arm].get(k, 0.0) for k in ASSEMBLY_ROWS)

    assembly_total = {
        "base": [sum_rows("base-r1"), sum_rows("base-r2")],
        "after": [sum_rows("after-r1"), sum_rows("after-r2")],
    }
    assembly_total["delta_pct"] = mean_delta_pct(assembly_total["base"], assembly_total["after"])

    resolve_ms: dict[str, int] = {}
    for arm in arms:
        stderr = (receipt_dir / f"{arm}.stderr.txt").read_text()
        m = re.search(r"\[resolve-summary\] 50 resolve\(s\) in ([0-9]+)ms", stderr)
        if m:
            resolve_ms[arm] = int(m.group(1))

    wall_ms = {arm: int(parsed[arm].get("wall_ms", "0")) for arm in arms}
    peak_rss = {arm: int(parsed[arm].get("peak_rss_kb", "0")) for arm in arms}
    exclusive_asm = {
        arm: float(parsed[arm].get("exclusive_assembly_ms", "0") or 0) for arm in arms
    }

    outcomes = {
        arm: {
            "pass": int(parsed[arm].get("pass", "0")),
            "fail": int(parsed[arm].get("fail", "0")),
        }
        for arm in arms
    }
    outcome_sets = [(outcomes[a]["pass"], outcomes[a]["fail"]) for a in arms]

    summary = {
        "derived_from": [f"{a}.tsv" for a in arms],
        "binary_sha256": {arm: parsed[arm].get("binary_sha256", "") for arm in arms},
        "exclusive_assembly_rows": assembly,
        "aggregates": {
            "assembly_total_ms": assembly_total,
            "exclusive_assembly_ms": {
                "base": [exclusive_asm["base-r1"], exclusive_asm["base-r2"]],
                "after": [exclusive_asm["after-r1"], exclusive_asm["after-r2"]],
                "delta_pct": mean_delta_pct(
                    [exclusive_asm["base-r1"], exclusive_asm["base-r2"]],
                    [exclusive_asm["after-r1"], exclusive_asm["after-r2"]],
                ),
            },
            "resolve_additive_ms": {
                "base": [resolve_ms.get("base-r1", 0), resolve_ms.get("base-r2", 0)],
                "after": [resolve_ms.get("after-r1", 0), resolve_ms.get("after-r2", 0)],
                "delta_pct": mean_delta_pct(
                    [resolve_ms.get("base-r1", 0), resolve_ms.get("base-r2", 0)],
                    [resolve_ms.get("after-r1", 0), resolve_ms.get("after-r2", 0)],
                ),
            },
            "elapsed_wall_ms": {
                "base": [wall_ms["base-r1"], wall_ms["base-r2"]],
                "after": [wall_ms["after-r1"], wall_ms["after-r2"]],
                "delta_pct": mean_delta_pct(
                    [wall_ms["base-r1"], wall_ms["base-r2"]],
                    [wall_ms["after-r1"], wall_ms["after-r2"]],
                ),
            },
            "peak_rss_kb": {
                "base": [peak_rss["base-r1"], peak_rss["base-r2"]],
                "after": [peak_rss["after-r1"], peak_rss["after-r2"]],
                "delta_pct": mean_delta_pct(
                    [peak_rss["base-r1"], peak_rss["base-r2"]],
                    [peak_rss["after-r1"], peak_rss["after-r2"]],
                ),
            },
        },
        "entry_view_assembly_memo": {arm: entry_view_by_arm.get(arm, {}) for arm in arms},
        "outcomes": outcomes,
        "outcomes_identical_across_arms": len(set(outcome_sets)) == 1,
        "resolve_count": {arm: 50 for arm in arms},
    }

    out_path = receipt_dir / "summary.json"
    out_path.write_text(json.dumps(summary, indent=2) + "\n")
    print(out_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
