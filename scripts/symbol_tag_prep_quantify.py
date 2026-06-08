#!/usr/bin/env python3
"""Read-only PREP quantification for E2 standalone symbol-tag bulk sweep.

Precise signature only: `data NAME: Symbol = NAME` (tautological) or aliased.
Classifies tautological tags as ctor-tag (bridge arm target) vs standalone (^NAME).
"""
from __future__ import annotations

import csv
import json
import os
import re
import sys
from collections import defaultdict

ROOT = sys.argv[1] if len(sys.argv) > 1 else "src/v4"
OUT_DIR = sys.argv[2] if len(sys.argv) > 2 else "/tmp/symbol_tag_prep"

TAG_RE = re.compile(r"^\s*data\s+(\w+)\s*:\s*Symbol\s*=\s*(\w+)\s*$", re.M)
BRIDGE_HDR_RE = re.compile(r"^\s*fn\s+(\w+)\s*\([^)]*\)\s*->\s*Symbol\s*\{", re.M)
ARM_RE = re.compile(
    r"\b([A-Z]\w*)\s*(?:\{[^{}]*\}|\([^()]*\))?\s*=>\s*([a-z]\w*)\b", re.S
)
IDENT_RE = re.compile(r"\b[A-Za-z_][A-Za-z0-9_]*\b")


def brace_body(text: str, open_idx: int) -> str:
    depth = 0
    for i in range(open_idx, len(text)):
        c = text[i]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return text[open_idx : i + 1]
    return text[open_idx:]


def iter_dag_files(root: str):
    for dirpath, _, files in os.walk(root):
        for fn in files:
            if fn.endswith(".dag"):
                path = os.path.join(dirpath, fn)
                rel = os.path.relpath(path, root)
                with open(path, encoding="utf-8", errors="replace") as f:
                    yield rel, f.read()


def find_bridges(text: str) -> list[dict]:
    bridges = []
    for m in BRIDGE_HDR_RE.finditer(text):
        body = brace_body(text, m.end() - 1)
        if "match" not in body:
            continue
        arms = ARM_RE.findall(body)
        if len(arms) >= 2:
            bridges.append(
                {
                    "fn": m.group(1),
                    "targets": {r for _, r in arms},
                }
            )
    return bridges


def main() -> None:
    os.makedirs(OUT_DIR, exist_ok=True)

    tautological: list[dict] = []
    aliased: list[dict] = []
    file_text: dict[str, str] = {}
    decl_lines: dict[str, set[int]] = defaultdict(set)

    for rel, text in iter_dag_files(ROOT):
        file_text[rel] = text
        for m in TAG_RE.finditer(text):
            name, rhs = m.group(1), m.group(2)
            line = text[: m.start()].count("\n") + 1
            entry = {
                "file": rel,
                "name": name,
                "line": line,
                "rhs": rhs,
                "compiler": rel.startswith("compiler/"),
            }
            decl_lines[f"{rel}:{name}"].add(line)
            if name == rhs:
                tautological.append(entry)
            else:
                aliased.append(entry)

    bridge_targets: dict[str, list[tuple[str, str]]] = defaultdict(list)
    census_bridge_files: list[str] = []
    for rel, text in file_text.items():
        bridges = find_bridges(text)
        if bridges:
            census_bridge_files.append(rel)
        for b in bridges:
            for t in b["targets"]:
                bridge_targets[t].append((rel, b["fn"]))

    # Single-pass ref index: scan each file once, count declared tag idents.
    declared_names = {t["name"] for t in tautological} | {t["name"] for t in aliased}
    ref_counts: dict[str, int] = defaultdict(int)
    for rel, text in file_text.items():
        for m in IDENT_RE.finditer(text):
            name = m.group(0)
            if name not in declared_names:
                continue
            line = text[: m.start()].count("\n") + 1
            if line in decl_lines.get(f"{rel}:{name}", set()):
                continue
            ref_counts[name] += 1

    rows: list[dict] = []
    ctor_tag = 0
    standalone = 0
    for t in tautological:
        name = t["name"]
        if name in bridge_targets:
            kind = "ctor_tag"
            action = "discriminant"
            ctor_tag += 1
            bridges = ";".join(f"{f}:{fn}" for f, fn in bridge_targets[name])
        else:
            kind = "standalone"
            action = f"^{name}"
            standalone += 1
            bridges = ""
        rows.append(
            {
                "file": t["file"],
                "name": name,
                "line": t["line"],
                "compiler": t["compiler"],
                "kind": kind,
                "action": action,
                "ref_count": ref_counts.get(name, 0),
                "bridge_hits": bridges,
            }
        )

    rows.sort(key=lambda r: (r["file"], r["line"]))

    taut_csv = os.path.join(OUT_DIR, "tautological_targets.csv")
    with open(taut_csv, "w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(
            f,
            fieldnames=[
                "file",
                "name",
                "line",
                "kind",
                "action",
                "ref_count",
                "compiler",
                "bridge_hits",
            ],
        )
        w.writeheader()
        w.writerows(rows)

    alias_rows = sorted(aliased, key=lambda r: (r["file"], r["line"]))
    alias_csv = os.path.join(OUT_DIR, "aliased_targets.csv")
    with open(alias_csv, "w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(
            f,
            fieldnames=["file", "name", "line", "rhs", "action", "compiler"],
        )
        w.writeheader()
        for t in alias_rows:
            w.writerow({**t, "action": f"^{t['rhs']}"})

    compiler_taut = [t for t in tautological if t["compiler"]]
    compiler_alias = [t for t in aliased if t["compiler"]]

    ex_compiler_standalone = [
        r for r in rows if r["kind"] == "standalone" and not r["compiler"]
    ]
    by_file = defaultdict(int)
    for r in ex_compiler_standalone:
        by_file[r["file"]] += 1

    std_order = sorted(
        ((f, c) for f, c in by_file.items() if f.startswith("std/")),
        key=lambda x: x[1],
    )

  # Batch grouping: std/, lens/, extdeps/languages/, test/, workflow/, rest
    def batch_bucket(file: str) -> str:
        if file.startswith("std/"):
            return "batch_std"
        if file.startswith("lens/"):
            return "batch_lens"
        if file.startswith("extdeps/languages/"):
            return "batch_extdeps_languages"
        if file.startswith("extdeps/"):
            return "batch_extdeps_other"
        if file.startswith("test/"):
            return "batch_test"
        if file.startswith("workflow/"):
            return "batch_workflow"
        if file.startswith("install/"):
            return "batch_install"
        return "batch_other"

    batch_counts = defaultdict(int)
    for r in ex_compiler_standalone:
        batch_counts[batch_bucket(r["file"])] += 1

    summary = {
        "tautological_total": len(tautological),
        "tautological_ex_compiler": len(tautological) - len(compiler_taut),
        "tautological_compiler": len(compiler_taut),
        "aliased_total": len(aliased),
        "aliased_ex_compiler": len(aliased) - len(compiler_alias),
        "aliased_compiler": len(compiler_alias),
        "classification": {
            "ctor_tag": ctor_tag,
            "standalone": standalone,
            "standalone_ex_compiler": len(ex_compiler_standalone),
        },
        "census_bridge_files_without_tags": len(
            [f for f in census_bridge_files if not any(t["file"] == f for t in tautological)]
        ),
        "standalone_zero_refs": len(
            [r for r in ex_compiler_standalone if r["ref_count"] == 0]
        ),
        "batch_standalone_ex_compiler": dict(sorted(batch_counts.items())),
        "std_exemplar_order_smallest_first": [
            {"file": f, "count": c} for f, c in std_order
        ],
        "top_files_ex_compiler_standalone": sorted(
            by_file.items(), key=lambda x: -x[1]
        )[:20],
        "ctor_tag_remnants": [
            {
                "file": r["file"],
                "line": r["line"],
                "name": r["name"],
                "bridge_hits": r["bridge_hits"],
            }
            for r in rows
            if r["kind"] == "ctor_tag"
        ],
        "compiler_taut_files": sorted({t["file"] for t in compiler_taut}),
        "outputs": {"tautological_csv": taut_csv, "aliased_csv": alias_csv},
    }

    summary_path = os.path.join(OUT_DIR, "summary.json")
    with open(summary_path, "w", encoding="utf-8") as f:
        json.dump(summary, f, indent=2)

    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
