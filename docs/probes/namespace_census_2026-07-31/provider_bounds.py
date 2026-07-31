#!/usr/bin/env python3
"""Derive the two regex-catalogue bounds; neither is a semantic resolver."""

import argparse
import collections
import json
import pathlib
import re

TYPE = re.compile(r"^type\s+([A-Za-z_][A-Za-z0-9_]*)")
DATA = re.compile(r"^data\s+([A-Za-z_][A-Za-z0-9_]*)\s*:")
FUNCTION = re.compile(r"^(?:fn|test\s+fn)\s+([A-Za-z_][A-Za-z0-9_]*)")
MODULE = re.compile(r"^module\s+([A-Za-z0-9_.]+)", re.MULTILINE)


def catalogue(repo: pathlib.Path):
    declarations = collections.defaultdict(set)
    module_by_file = {}
    for root in (repo / "dag", repo / "src" / "v2"):
        for path in root.rglob("*.dag"):
            text = path.read_text(encoding="utf-8", errors="replace")
            module_match = MODULE.search(text)
            if module_match is None:
                continue
            module = module_match.group(1)
            module_by_file[str(path.relative_to(repo))] = module
            for line in text.splitlines():
                match = TYPE.match(line)
                if match:
                    declarations[(match.group(1), "type")].add(module)
                    rhs = line.split("=", 1)[1] if "=" in line else ""
                    for variant in re.findall(r"\b([A-Z][A-Za-z0-9_]*)\b", rhs):
                        declarations[(variant, "variant")].add(module)
                    continue
                match = DATA.match(line)
                if match:
                    declarations[(match.group(1), "variable")].add(module)
                    continue
                match = FUNCTION.match(line)
                if match:
                    declarations[(match.group(1), "function")].add(module)
            for variant in re.findall(r"^\s*([A-Z][A-Za-z0-9_]*)\s*(?:\{|,|$)", text, re.MULTILINE):
                declarations[(variant, "variant")].add(module)
    return declarations, module_by_file


def measure(rows, providers, module_by_file):
    counts = collections.Counter()
    edges = set()
    for row in rows:
        candidates = providers(row)
        if not candidates:
            counts["zero"] += 1
        elif len(candidates) == 1:
            counts["one"] += 1
            consumer = module_by_file.get(row["file"])
            if consumer:
                edges.add((consumer, next(iter(candidates))))
        else:
            counts["many"] += 1
    return {"zero": counts["zero"], "one": counts["one"], "many": counts["many"],
            "mechanical_share_percent": round(100 * counts["one"] / len(rows), 1),
            "unique_provider_edges": len(edges)}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("repo", type=pathlib.Path)
    parser.add_argument("population_json", type=pathlib.Path)
    args = parser.parse_args()
    declarations, modules = catalogue(args.repo.resolve())
    rows = json.loads(args.population_json.read_text())["rows"]
    by_name = collections.defaultdict(set)
    for (name, _category), providers in declarations.items():
        by_name[name].update(providers)
    strict = measure(rows, lambda row: declarations.get((row["symbol"], row["category"]), set()), modules)
    agnostic = measure(rows, lambda row: by_name.get(row["symbol"], set()), modules)
    expected = {"mechanical_share_percent": [60.7, 81.6], "unique_provider_edges": [2197, 2717]}
    actual = {"mechanical_share_percent": [agnostic["mechanical_share_percent"], strict["mechanical_share_percent"]],
              "unique_provider_edges": [agnostic["unique_provider_edges"], strict["unique_provider_edges"]]}
    if actual != expected:
        raise SystemExit(f"regex-bound drift: {actual}")
    print(json.dumps({"authority": "regex-bound", "category_agnostic": agnostic,
                      "category_strict": strict, "bracket": actual}, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
