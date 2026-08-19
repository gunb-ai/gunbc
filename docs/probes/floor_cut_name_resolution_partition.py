#!/usr/bin/env python3
"""Derive identity-grain partition TSV from floor_cut_name_resolution_census.tsv."""

from __future__ import annotations

import argparse
import re
import sys
from collections import defaultdict
from pathlib import Path

IMPORT_RE = re.compile(
    r"^import\s+([\w.]+)(?:\s*\{([^}]*)\})?",
    re.MULTILINE,
)
IMPORT_FROM_RE = re.compile(
    r"^import\s+([\w.]+)\s*\{([^}]*)\}",
    re.MULTILINE,
)


def module_path_to_file(module_path: str, roots: list[Path]) -> Path | None:
    rel = module_path.replace(".", "/") + ".dag"
    for root in roots:
        candidate = root / rel
        if candidate.is_file():
            return candidate
    return None


def parse_import_bindings(source: str) -> dict[str, str]:
    bindings: dict[str, str] = {}
    for match in IMPORT_FROM_RE.finditer(source):
        module = match.group(1)
        symbols = match.group(2)
        for part in symbols.split(","):
            part = part.strip()
            if not part:
                continue
            if " as " in part:
                bare, _alias = part.split(" as ", 1)
                bindings[bare.strip()] = module
            else:
                bindings[part.strip()] = module
    for match in IMPORT_RE.finditer(source):
        module = match.group(1)
        if match.group(2):
            continue
        leaf = module.rsplit(".", 1)[-1]
        bindings.setdefault(leaf, module)
    return bindings


def infer_intended(
    witness_module: str,
    bare_name: str | None,
    candidate_modules: list[str],
    roots: list[Path],
) -> str:
    if not bare_name:
        return ""
    path = module_path_to_file(witness_module, roots)
    if path is None:
        return ""
    source = path.read_text(encoding="utf-8")
    imports = parse_import_bindings(source)
    if bare_name in imports:
        return f"{imports[bare_name]}.{bare_name}"
    if f"{bare_name}(" in source or f".{bare_name}" in source:
        for module in candidate_modules:
            if module in source:
                return f"{module}.{bare_name}"
    if len(candidate_modules) == 1:
        return f"{candidate_modules[0]}.{bare_name}"
    return ""


def classify_reach(failure_class: str, candidate_modules: list[str], selected: str) -> str:
    if failure_class == "call_contract_mismatch":
        return "bare_name_binding"
    if failure_class in {"no_such_function", "undefined_variable"}:
        if not candidate_modules:
            return "reach_gap"
        if selected:
            return "bare_name_binding"
        return "reach_or_binding_unresolved"
    return "other"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("census_tsv", type=Path)
    parser.add_argument(
        "-o",
        "--output",
        type=Path,
        default=Path("docs/probes/floor_cut_name_resolution_partition.tsv"),
    )
    parser.add_argument("--dag-root", type=Path, default=Path("dag"))
    parser.add_argument("--v2-root", type=Path, default=Path("src/v2"))
    args = parser.parse_args()

    roots = [args.dag_root, args.v2_root]
    lines = args.census_tsv.read_text(encoding="utf-8").splitlines()
    if not lines:
        print("empty census", file=sys.stderr)
        return 1
    header = lines[0].split("\t")
    rows = [dict(zip(header, line.split("\t"))) for line in lines[1:] if line.strip()]

    out_header = [
        "reference_site",
        "intended_declaration_identity",
        "actually_selected_declaration_identity",
        "visible_candidate_set",
        "failure_class",
        "reach_vs_binding",
        "witness_qualified",
        "error_message",
    ]
    out_lines = ["\t".join(out_header)]
    counts: dict[str, int] = defaultdict(int)

    for row in rows:
        bare = row.get("reference_bare_name", "") or ""
        witness_module = row.get("witness_module", "")
        selected_module = row.get("selected_decl_module", "")
        candidates = [c for c in row.get("candidate_decl_modules", "").split("|") if c]
        failure_class = row.get("failure_class", "other")
        counts[failure_class] += 1

        intended = infer_intended(witness_module, bare or None, candidates, roots)
        selected = f"{selected_module}.{bare}" if selected_module and bare else ""
        reference_site = (
            f"{witness_module} (witness); bare={bare}"
            if bare
            else row.get("witness_qualified", "")
        )
        reach = classify_reach(failure_class, candidates, selected_module)

        out_lines.append(
            "\t".join(
                [
                    reference_site,
                    intended,
                    selected,
                    "|".join(candidates),
                    failure_class,
                    reach,
                    row.get("witness_qualified", ""),
                    row.get("error_message", "").replace("\t", " "),
                ]
            )
        )

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text("\n".join(out_lines) + "\n", encoding="utf-8")

    print(f"PARTITION_STATUS ok rows={len(rows)} output={args.output}")
    for cls, count in sorted(counts.items()):
        print(f"PARTITION_CLASS {cls}={count}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
