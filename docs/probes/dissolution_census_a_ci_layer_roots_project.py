#!/usr/bin/env python3
"""Project dissolution census A from the canonical TSV observation.

Authority: docs/probes/dissolution_census_a_ci_layer_roots_2026-08-03.tsv
Generated: summary JSON + human-readable markdown report.

    python3 docs/probes/dissolution_census_a_ci_layer_roots_project.py write
    python3 docs/probes/dissolution_census_a_ci_layer_roots_project.py verify
    python3 docs/probes/dissolution_census_a_ci_layer_roots_project.py verify-red
"""
from __future__ import annotations

import json
import sys
from collections import Counter
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
TSV_PATH = REPO_ROOT / "docs/probes/dissolution_census_a_ci_layer_roots_2026-08-03.tsv"
SUMMARY_PATH = REPO_ROOT / "docs/probes/dissolution_census_a_ci_layer_roots_2026-08-03.summary.json"
REPORT_PATH = REPO_ROOT / "docs/plans/dissolution-census-a-ci-layer-roots.md"

OBSERVATION_HEAD = "44126ca1de0"
OBSERVATION_FILE = "dag/gunbc/ci_layer_roots.dag"
PINNED_DATE = "2026-08-03"
CLASSIFIER_ID = "dag-note-prose-census-lexical-v1"
CLASSIFIER_NOTE = (
    "Lexical sentence classifier (same honesty bound as dag-note-prose-census §6); "
    "shares are ±10pp."
)

TYPED_ROW_GROUPS = frozenset(
    {"witnessexclusionrow", "rehomedbinwetrow", "substratelonglanerow"}
)
SYNTHETIC_RED_ID = "synthetic_orphan_admission_witness_test.dag"


def parse_tsv_rows(text: str) -> list[dict[str, str | int | bool]]:
    rows: list[dict[str, str | int | bool]] = []
    for line in text.splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        parts = line.split("\t")
        if parts[0] == "structural_group":
            continue
        if len(parts) < 10:
            raise ValueError(f"malformed TSV row ({len(parts)} fields): {line[:120]!r}")
        rows.append(
            {
                "structural_group": parts[0],
                "carrier": parts[1],
                "id": parts[2],
                "field": parts[3],
                "bytes": int(parts[4]),
                "is_ref": parts[5] == "True",
                "semantic_class": parts[6],
                "migration_target": parts[7],
                "has_dissolve_trigger": parts[8] == "True",
                "preview": parts[9],
            }
        )
    return rows


def is_synthetic_red_control(row: dict[str, str | int | bool]) -> bool:
    return (
        row["structural_group"] == "witnessexclusionrow"
        and SYNTHETIC_RED_ID in str(row["id"])
        and row["field"] in {"reason", "dissolve_on"}
        and row["is_ref"] is False
    )


def summarize(rows: list[dict[str, str | int | bool]]) -> dict:
    structural_groups = Counter(str(r["structural_group"]) for r in rows)
    inline_rows = [r for r in rows if not r["is_ref"]]
    template_refs = sum(1 for r in rows if r["is_ref"])
    semantic_classes_inline = Counter(str(r["semantic_class"]) for r in inline_rows)
    bytes_inline = sum(int(r["bytes"]) for r in inline_rows)

    module_note_rows = [r for r in inline_rows if r["structural_group"] == "module_note"]
    shared_template_rows = [
        r for r in inline_rows if r["structural_group"] == "shared_template"
    ]
    per_row_rows = [
        r for r in inline_rows if r["structural_group"] in TYPED_ROW_GROUPS
    ]
    synthetic_red = [r for r in inline_rows if is_synthetic_red_control(r)]

    all_inline = len(inline_rows)
    brief_grain = len(module_note_rows) + len(per_row_rows)
    brief_claimed = brief_grain - len(synthetic_red)

    return {
        "observation": {
            "head": OBSERVATION_HEAD,
            "file": OBSERVATION_FILE,
            "pinned_date": PINNED_DATE,
            "classifier": CLASSIFIER_ID,
            "classifier_note": CLASSIFIER_NOTE,
            "scope_note": (
                "Dated observation pinned at the named HEAD. Evidence for selecting "
                "dissolution work — not a claim about current main. Re-read live "
                "gunbc.ci_layer_roots before acting on any row."
            ),
        },
        "head": OBSERVATION_HEAD,
        "file": OBSERVATION_FILE,
        "total_sites": len(rows),
        "inline_prose": all_inline,
        "template_refs": template_refs,
        "structural_groups": dict(sorted(structural_groups.items())),
        "semantic_classes_inline": dict(sorted(semantic_classes_inline.items())),
        "bytes_inline": bytes_inline,
        "byte_partition_inline": {
            "module_note": {
                "count": len(module_note_rows),
                "bytes": sum(int(r["bytes"]) for r in module_note_rows),
            },
            "shared_template": {
                "count": len(shared_template_rows),
                "bytes": sum(int(r["bytes"]) for r in shared_template_rows),
            },
            "per_row_typed": {
                "count": len(per_row_rows),
                "bytes": sum(int(r["bytes"]) for r in per_row_rows),
            },
        },
        "grain_reconciliation": {
            "all_inline_prose": all_inline,
            "brief_grain_excl_templates": brief_grain,
            "brief_claimed_excl_templates_and_synthetic_red_control": brief_claimed,
            "brief_claimed": brief_claimed,
            "synthetic_red_control_fields_excluded_for_135": [
                "witness_exclusion_frontier/synthetic_orphan_admission_witness_test.dag "
                "reason+dissolve_on (NoConsumer RED control)"
            ],
            "note": (
                "Brief grain excludes shared excl_* templates (already field-shaped). "
                "Brief 135 further excludes the 2 inline NoConsumer RED-control fields."
            ),
        },
        "per_row_inline_prose": len(per_row_rows),
    }


def inline_prose_by_group(rows: list[dict[str, str | int | bool]]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for row in rows:
        if row["is_ref"]:
            continue
        group = str(row["structural_group"])
        counts[group] = counts.get(group, 0) + 1
    return counts


def format_kib(bytes_val: int) -> str:
    return f"{bytes_val / 1024:.1f} KiB"


def render_markdown(summary: dict, rows: list[dict[str, str | int | bool]]) -> str:
    obs = summary["observation"]
    inline_by_group = inline_prose_by_group(rows)
    partition = summary["byte_partition_inline"]
    grains = summary["grain_reconciliation"]
    sem = summary["semantic_classes_inline"]
    groups = summary["structural_groups"]
    inline_total = summary["inline_prose"]

    group_labels = {
        "witnessexclusionrow": "WitnessExclusionRow",
        "substratelonglanerow": "SubstrateLongLaneRow",
        "rehomedbinwetrow": "RehomedBinWetRow",
        "module_note": "module `*_note`",
        "shared_template": "`excl_*` shared templates",
    }

    lines = [
        "# Dissolution census A — `gunbc.ci_layer_roots` prose markers",
        "",
        f"**Status:** census complete, measured at `{obs['head']}`, {obs['pinned_date']}. "
        "No prose deleted, no carriers migrated.",
        "",
        "**Authority:** the revision-pinned TSV observation at "
        "`docs/probes/dissolution_census_a_ci_layer_roots_2026-08-03.tsv`. "
        "This markdown is a **generated projection** — regenerate with:",
        "",
        "```",
        "python3 docs/probes/dissolution_census_a_ci_layer_roots_project.py write",
        "```",
        "",
        f"**Classifier:** `{obs['classifier']}` — {obs['classifier_note']}",
        "",
        f"**Scope note:** {obs['scope_note']}",
        "",
        "**Scope:** every prose-bearing site inside `dag/gunbc/ci_layer_roots.dag` — the CI "
        "floor's single-authority witness-layer, discovery-exclusion, and falsifier-roster "
        "carrier (25.3 KiB prose mass per "
        "[dag-note-prose-census.md](dag-note-prose-census.md) §1).",
        "",
        "**Instrument:** row-level register at "
        "`docs/probes/dissolution_census_a_ci_layer_roots_2026-08-03.tsv` "
        f"({summary['total_sites']} sites) plus the generated "
        "`docs/probes/dissolution_census_a_ci_layer_roots_2026-08-03.summary.json`. "
        "**Grain key:** a *site* is one `reason`, `dissolve_on`, or `data String` field; "
        "a site is *inline prose* when `is_ref=False` (literal string body); template-ref "
        "sites (`reason: excl_*`) carry `is_ref=True` and are not prose.",
        "",
        "---",
        "",
        "## 0. Count reconciliation (brief claimed 135)",
        "",
        "Three grains — do not conflate:",
        "",
        "| grain | count | bytes | definition |",
        "|---|---|---|---|",
        f"| **All inline prose** (every `is_ref=False` site) | **{grains['all_inline_prose']}** "
        f"| {format_kib(summary['bytes_inline'])} ({summary['bytes_inline']:,} B) | "
        "Full census population |",
        f"| — module `*_note` | {partition['module_note']['count']} | "
        f"{format_kib(partition['module_note']['bytes'])} | Authority essays |",
        f"| — `excl_*` shared templates | {partition['shared_template']['count']} | "
        f"{format_kib(partition['shared_template']['bytes'])} | "
        "Classification-scoped reason/dissolve templates |",
        f"| — per-row `reason` / `dissolve_on` on typed rows | "
        f"**{partition['per_row_typed']['count']}** | "
        f"{format_kib(partition['per_row_typed']['bytes'])} | "
        "`WitnessExclusionRow` + `RehomedBinWetRow` + `SubstrateLongLaneRow` |",
        f"| **Brief grain** (excl. templates — already field-shaped) | "
        f"**{grains['brief_grain_excl_templates']}** | "
        f"{format_kib(partition['module_note']['bytes'] + partition['per_row_typed']['bytes'])} | "
        "24 notes + 113 per-row |",
        f"| **Brief claimed** (excl. templates + NoConsumer RED-control pair) | "
        f"**{grains['brief_claimed']}** | — | 137 − 2 inline fields on "
        "`synthetic_orphan_admission_witness_test.dag` |",
        f"| Template-ref sites (`reason: excl_*`, not prose) | {summary['template_refs']} | — | "
        "`is_ref=True` |",
        f"| **Total marker sites** (prose + refs) | **{summary['total_sites']}** | — | "
        "TSV row count |",
        "",
        f"Arithmetic check: {partition['module_note']['count']} + "
        f"{partition['shared_template']['count']} + {partition['per_row_typed']['count']} = "
        f"{inline_total} inline sites.",
        "",
        "---",
        "",
        "## 1. Structural groups (where prose lives)",
        "",
        "Five carriers hold all prose. Three are **already row-typed** with `reason` + "
        "`dissolve_on` fields; two are module-level notes/templates.",
        "",
        "| structural group | sites | inline prose | role |",
        "|---|---|---|---|",
    ]

    for key in (
        "witnessexclusionrow",
        "substratelonglanerow",
        "rehomedbinwetrow",
        "module_note",
        "shared_template",
    ):
        label = group_labels[key]
        sites = groups[key]
        inline = inline_by_group.get(key, 0)
        if key == "witnessexclusionrow":
            role = "PATH POLICY roster — pattern + `WitnessConsumerCadence` + reason/dissolve"
        elif key == "substratelonglanerow":
            role = "Falsifier batch 6 — Class C long-lane hermetic witnesses"
        elif key == "rehomedbinwetrow":
            role = "Falsifier batch 5 — over-budget bin-execution witnesses"
        elif key == "module_note":
            role = "Authority essays — lane policies, reconciliation receipts"
        else:
            role = "Classification-scoped reason/dissolve templates (§3 nicknaming half-fixed)"
        lines.append(f"| `{label}` | {sites} | {inline} | {role} |")

    lines.extend(
        [
            "",
            "---",
            "",
            "## 2. Semantic classes (inline prose, dag-note-prose-census §2)",
            "",
            "| class | markers | share | migrates to |",
            "|---|---|---|---|",
        ]
    )
    for cls, count in sorted(sem.items(), key=lambda kv: (-kv[1], kv[0])):
        share = round(100 * count / inline_total) if inline_total else 0
        target = {
            "SPEC_NORM": "`StandingIntent` row + type/lens material",
            "RECEIPT": "event-log row (ages out)",
            "RULING": "ruling-register row",
            "XREF": "citation edge",
            "EVENT": "event-log row",
        }.get(cls, "—")
        lines.append(f"| **{cls}** | {count} | {share}% | {target} |")

    lines.extend(
        [
            "",
            "**Dissolution census finding:** unlike the corpus-wide prose census (69% multi-class "
            "notes), `ci_layer_roots` prose is **already field-separated** — reason vs "
            "dissolve_on vs module note — so the anemic-serialization problem is structural "
            "(String fields on typed rows) not paragraph-level mixing. The payoff is typing the "
            "fields, not sentence-splitting.",
            "",
            "---",
            "",
            "## 3. Sibling censuses",
            "",
            "- [dag-note-prose-census.md](dag-note-prose-census.md) — corpus-wide annotation "
            "layer (864 KiB)",
            "- [live-read-witness-classification-design.md]("
            "live-read-witness-classification-design.md) — supersedes hand exclusion rows when "
            "G2+G3 wire (same dissolution trigger as "
            "`witness_exclusion_single_authority_reconciliation_note`)",
            "- [module-identity-storage-binding-design.md]("
            "module-identity-storage-binding-design.md) — Phase 0(b) admission invariant this "
            "carrier implements",
            "",
            "**Dissolve-on:** typed annotation carriers land (`StandingIntent`, event log, "
            "citation edges) and an annotation-budget lens counts rows — same trigger as "
            "dag-note-prose-census.",
            "",
        ]
    )
    return "\n".join(lines) + "\n"


def load_rows() -> list[dict[str, str | int | bool]]:
    return parse_tsv_rows(TSV_PATH.read_text(encoding="utf-8"))


def write_projections() -> None:
    rows = load_rows()
    summary = summarize(rows)
    SUMMARY_PATH.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    REPORT_PATH.write_text(render_markdown(summary, rows), encoding="utf-8")


def load_committed_summary() -> dict:
    return json.loads(SUMMARY_PATH.read_text(encoding="utf-8"))


def compare_summaries(expected: dict, actual: dict, label: str) -> list[str]:
    errors: list[str] = []

    def check_group(path: str, exp: dict, act: dict) -> None:
        for key, exp_val in exp.items():
            act_val = act.get(key)
            if act_val != exp_val:
                errors.append(f"{label}: {path}[{key!r}] expected {exp_val!r}, got {act_val!r}")

    for key in (
        "total_sites",
        "inline_prose",
        "template_refs",
        "bytes_inline",
        "per_row_inline_prose",
    ):
        if expected.get(key) != actual.get(key):
            errors.append(
                f"{label}: {key} expected {expected.get(key)!r}, got {actual.get(key)!r}"
            )

    check_group("structural_groups", expected["structural_groups"], actual["structural_groups"])
    check_group(
        "semantic_classes_inline",
        expected["semantic_classes_inline"],
        actual["semantic_classes_inline"],
    )

    for part in ("module_note", "shared_template", "per_row_typed"):
        exp_part = expected["byte_partition_inline"][part]
        act_part = actual["byte_partition_inline"][part]
        for field in ("count", "bytes"):
            if exp_part[field] != act_part[field]:
                errors.append(
                    f"{label}: byte_partition_inline.{part}.{field} "
                    f"expected {exp_part[field]!r}, got {act_part[field]!r}"
                )

    grains = expected["grain_reconciliation"]
    act_grains = actual["grain_reconciliation"]
    for field in (
        "all_inline_prose",
        "brief_grain_excl_templates",
        "brief_claimed_excl_templates_and_synthetic_red_control",
        "brief_claimed",
    ):
        if grains[field] != act_grains[field]:
            errors.append(
                f"{label}: grain_reconciliation.{field} "
                f"expected {grains[field]!r}, got {act_grains[field]!r}"
            )

    return errors


def verify() -> int:
    rows = load_rows()
    actual = summarize(rows)
    expected = load_committed_summary()
    errors = compare_summaries(expected, actual, "committed")
    if errors:
        for err in errors:
            print(f"ERROR: {err}", file=sys.stderr)
        return 1
    print("dissolution census A projection verify: OK")
    return 0


def verify_red() -> int:
    rows = load_rows()
    actual = summarize(rows)
    perturbed = json.loads(json.dumps(actual))
    groups = perturbed["structural_groups"]
    if groups.get("witnessexclusionrow", 0) < 2:
        print("ERROR: cannot build perturbed summary", file=sys.stderr)
        return 1
    groups["witnessexclusionrow"] -= 1
    groups["module_note"] = groups.get("module_note", 0) + 1
    errors = compare_summaries(actual, perturbed, "perturbed-red-control")
    if not errors:
        print("ERROR: perturbed summary incorrectly matched", file=sys.stderr)
        return 1
    print(f"dissolution census A projection verify-red: OK ({len(errors)} mismatches)")
    return 0


def main() -> int:
    if len(sys.argv) != 2 or sys.argv[1] not in {"write", "verify", "verify-red"}:
        print(__doc__, file=sys.stderr)
        return 2
    cmd = sys.argv[1]
    if cmd == "write":
        write_projections()
        print(f"wrote {SUMMARY_PATH.relative_to(REPO_ROOT)}")
        print(f"wrote {REPORT_PATH.relative_to(REPO_ROOT)}")
        return 0
    if cmd == "verify":
        return verify()
    return verify_red()


if __name__ == "__main__":
    raise SystemExit(main())
