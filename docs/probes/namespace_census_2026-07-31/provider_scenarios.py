#!/usr/bin/env python3
"""Derive two regex sensitivity scenarios; neither bounds semantic resolution."""

import argparse
import collections
import json
import pathlib
import re

from receipt_common import load_summary, require_pinned_repo

TYPE = re.compile(r"^type\s+([A-Za-z_][A-Za-z0-9_]*)")
DATA = re.compile(r"^data\s+([A-Za-z_][A-Za-z0-9_]*)\s*:")
FUNCTION = re.compile(r"^(?:fn|test\s+fn)\s+([A-Za-z_][A-Za-z0-9_]*)")
MODULE = re.compile(r"^module\s+([A-Za-z0-9_.]+)", re.MULTILINE)


def catalogue(repo: pathlib.Path):
    declarations = collections.defaultdict(set)
    modules_by_file = collections.defaultdict(set)
    for root in (repo / "dag", repo / "src" / "v2"):
        for path in root.rglob("*.dag"):
            text = path.read_text(encoding="utf-8", errors="replace")
            module_match = MODULE.search(text)
            if module_match is None:
                continue
            module = module_match.group(1)
            modules_by_file[str(path.relative_to(repo))].add(module)
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
    return declarations, modules_by_file


def measure(rows, providers, modules_by_file):
    counts = collections.Counter()
    edges = set()
    unmapped_consumer_rows = 0
    duplicate_consumer_module_mappings = 0
    for row in rows:
        candidates = providers(row)
        if not candidates:
            counts["zero"] += 1
        elif len(candidates) == 1:
            counts["one"] += 1
            consumers = modules_by_file.get(row["file"], set())
            if not consumers:
                unmapped_consumer_rows += 1
            elif len(consumers) > 1:
                duplicate_consumer_module_mappings += 1
            else:
                edges.add((next(iter(consumers)), next(iter(candidates))))
        else:
            counts["many"] += 1
    if unmapped_consumer_rows or duplicate_consumer_module_mappings:
        raise SystemExit(
            "consumer mapping refused: "
            f"unmapped_consumer_rows={unmapped_consumer_rows}, "
            f"duplicate_consumer_module_mappings={duplicate_consumer_module_mappings}"
        )
    return {"zero": counts["zero"], "one": counts["one"], "many": counts["many"],
            "apparent_single_provider_share_percent": round(100 * counts["one"] / len(rows), 1),
            "unique_apparent_single_provider_edges": len(edges),
            "unmapped_consumer_rows": unmapped_consumer_rows,
            "duplicate_consumer_module_mappings": duplicate_consumer_module_mappings}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("repo", type=pathlib.Path)
    parser.add_argument("population_json", type=pathlib.Path)
    parser.add_argument("output", type=pathlib.Path)
    parser.add_argument("--summary-json", type=pathlib.Path, required=True)
    args = parser.parse_args()
    summary = load_summary(args.summary_json)
    repo = require_pinned_repo(args.repo, summary)
    declarations, modules_by_file = catalogue(repo)
    rows = json.loads(args.population_json.read_text())["rows"]
    by_name = collections.defaultdict(set)
    for (name, _category), providers in declarations.items():
        by_name[name].update(providers)
    strict = measure(rows, lambda row: declarations.get((row["symbol"], row["category"]), set()), modules_by_file)
    agnostic = measure(rows, lambda row: by_name.get(row["symbol"], set()), modules_by_file)
    result = {"authority": "regex-sensitivity-scenario",
              "category_agnostic_regex_scenario": agnostic,
              "category_strict_regex_scenario": strict}
    args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
