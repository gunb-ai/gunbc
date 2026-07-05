#!/usr/bin/env python3
"""Reproducible census for docs/plans/witness-subject-execution-audit.md.

Counts only `test fn … -> Bool` rows under the documented witness roots and
classifies each by transitive body analysis. Run from repo root:

    python3 docs/plans/witness-subject-execution-audit.py
"""
from __future__ import annotations

import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

WITNESS_ROOTS = [
    Path("dag/test/claim"),
    Path("src/v2/test/claim"),
    Path("src/v2/lens"),
    Path("src/v2/test/manual"),
    Path("src/v2/workflow"),
]

FILE_CLASS: dict[str, str] = {
    "dag/test/claim/v1_source_audit_witness_test.dag": "c",
    "src/v2/test/claim/ci_spec_witness_test.dag": "c",
    "dag/test/claim/gitignore_gate_test.dag": "c",
    "src/v2/test/claim/self_host_realized_comparison_floor_test.dag": "c",
    "dag/test/claim/ci_yaml_serializer_witness_test.dag": "c",
    "dag/test/claim/ci_deploy_witness_test.dag": "c",
    "dag/test/claim/ci_failure_class_witness_test.dag": "c",
    "dag/test/claim/build_artifact_verification_witness_test.dag": "c",
    "dag/test/claim/roadmap_dashboard_emit_witness_test.dag": "c",
    "dag/test/claim/node_http_server_emit_test.dag": "c",
    "src/v2/test/claim/intent_linearity/lens_unit/import_closure_completeness_test.dag": "b-danger",
}

TRANSPORT_FILES = {
    "dag/test/claim/diagnostics_test.dag",
    "dag/test/claim/effects_rest_transport_parse_witness_test.dag",
    "dag/test/claim/floor_skip_discovery_witness_test.dag",
    "dag/test/claim/interp_recorded_fixture_witness_test.dag",
    "src/v2/test/claim/auth_declared_but_unwired_witness_test.dag",
    "src/v2/test/claim/bootstrap_test.dag",
    "src/v2/test/claim/infer_semantics_witness_test.dag",
}

DANGEROUS_HOST = {
    "import_closure_is_clean_live",
    "import_closure_live_paths",
    "realization_vocab_containment_clean_live",
    "realization_vocab_leak_count_live",
    "medium_structure_containment_clean_live",
    "medium_structure_leak_count_live",
    "enforcement_consistency_live",
    "complexity_repo_wide_verdict_live",
    "complexity_linearity_syntactic_finding_count_live",
    "complexity_linearity_syntactic_wildcard_finding_count_live",
    "complexity_linearity_syntactic_site_fired_live",
    "complexity_linearity_wildcard_total_live",
    "complexity_linearity_wildcard_migration_debt_live",
    "complexity_linearity_wildcard_open_domain_live",
    "transport_script_position_facts_live",
    "fn_arrow_decl_facts_live",
    "witness_three_way_conservation_live",
    "test_migration_delete_guard_holds_live",
    "realization_vocab_roster_sound_live",
    "realization_vocab_roster_stale_count_live",
}

TEST_FN_RE = re.compile(r"test\s+fn\s+(\w+)\s*\([^)]*\)\s*->\s*Bool\b")
FN_RE = re.compile(r"(?:test\s+)?fn\s+(\w+)\s*\([^)]*\)\s*->\s*(\w+)")


def extract_all_fns(content: str) -> dict[str, str]:
    fns: dict[str, str] = {}
    for m in FN_RE.finditer(content):
        name = m.group(1)
        start = content.find("{", m.end())
        if start < 0:
            continue
        depth = 0
        end = start
        for i, ch in enumerate(content[start:], start):
            if ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
                if depth == 0:
                    end = i + 1
                    break
        fns[name] = content[start:end]
    return fns


def body_uses(fns: dict[str, str], body: str, predicate) -> bool:
    seen: set[str] = set()

    def walk(b: str) -> bool:
        if predicate(b):
            return True
        for call in re.findall(r"\b(\w+)\s*\(", b):
            if call in fns and call not in seen:
                seen.add(call)
                if walk(fns[call]):
                    return True
        return False

    return walk(body)


def classify(rel: str, fn: str, body: str, fns: dict[str, str]) -> str:
    if rel in FILE_CLASS:
        return FILE_CLASS[rel]
    if rel in TRANSPORT_FILES and (
        ("run_" in body and "witness" in body) or fn.endswith("_keystone_holds")
    ):
        return "b"
    if "layering_imports_clean_holds(" in body:
        return "a"
    if body_uses(
        fns,
        body,
        lambda b: any(h + "(" in b for h in DANGEROUS_HOST),
    ):
        if "import_closure_live(" not in body and "import_closure(" not in body:
            return "b-danger"
    if body_uses(fns, body, lambda b: "filesystem_read(" in b and "string_contains" in b):
        return "c"
    if body_uses(fns, body, lambda b: b.count("string_contains") >= 3):
        return "c"
    if "shell.Exec.Run" in body:
        return "b"
    return "a"


def extract_test_witnesses() -> list[tuple[str, str, int, str]]:
    entries: list[tuple[str, str, int, str]] = []
    for root in WITNESS_ROOTS:
        if not root.exists():
            continue
        for fp in sorted(root.rglob("*_test.dag")):
            if "__" in fp.name:
                continue
            rel = str(fp)
            content = fp.read_text(errors="replace")
            fns = extract_all_fns(content)
            for m in TEST_FN_RE.finditer(content):
                fn = m.group(1)
                line = content[: m.start()].count("\n") + 1
                start = content.find("{", m.end())
                depth = 0
                end = start
                for i, ch in enumerate(content[start:], start):
                    if ch == "{":
                        depth += 1
                    elif ch == "}":
                        depth -= 1
                        if depth == 0:
                            end = i + 1
                            break
                body = content[start:end]
                entries.append((rel, fn, line, classify(rel, fn, body, fns)))
    return entries


def count_scope_files() -> int:
    n = 0
    for root in WITNESS_ROOTS:
        if not root.exists():
            continue
        for fp in root.rglob("*_test.dag"):
            if "__" not in fp.name:
                n += 1
    return n


def main() -> int:
    entries = extract_test_witnesses()
    counts = Counter(cls for *_, cls in entries)
    total = len(entries)
    files = count_scope_files()
    illusion = counts["b"] + counts["b-danger"] + counts["c"]

    print("witness-subject-execution census")
    print(f"scope files (*_test.dag): {files}")
    print(f"witness test fn -> Bool: {total}")
    for key in ("a", "b", "b-danger", "c"):
        n = counts[key]
        print(f"  ({key}): {n} ({100 * n / total:.1f}%)")
    print(f"illusion rate (b+b-danger+c): {illusion}/{total} = {100 * illusion / total:.1f}%")
    print(f"high-risk floor b-danger: {counts['b-danger']} (+ 12 Rust import_closure equivalence)")

    c_by_file = defaultdict(int)
    for rel, _, _, cls in entries:
        if cls == "c":
            c_by_file[rel] += 1
    print(f"class (c) files: {len(c_by_file)} test fns: {sum(c_by_file.values())}")

    expected_total = 1532
    expected_files = 523
    if total != expected_total or files != expected_files:
        print(
            f"ERROR: census drift — expected {expected_total} witnesses in {expected_files} files",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
