#!/usr/bin/env python3
"""Generate the uncommitted root that makes every declared module reachable."""

import argparse
import pathlib
import re

from receipt_common import load_summary, require_pinned_repo

MODULE = re.compile(r"^module\s+([A-Za-z0-9_.]+)\s*$", re.MULTILINE)


def declared_modules(repo: pathlib.Path) -> list[str]:
    modules: dict[str, pathlib.Path] = {}
    for source_root in (repo / "dag", repo / "src" / "v2"):
        for path in sorted(source_root.rglob("*.dag")):
            match = MODULE.search(path.read_text(encoding="utf-8", errors="replace"))
            if match is None:
                continue
            module = match.group(1)
            previous = modules.get(module)
            if previous is not None:
                raise SystemExit(f"duplicate module {module}: {previous} and {path}")
            modules[module] = path
    return sorted(modules)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("repo", type=pathlib.Path)
    parser.add_argument("output", type=pathlib.Path)
    parser.add_argument("--summary-json", type=pathlib.Path, required=True)
    args = parser.parse_args()

    summary = load_summary(args.summary_json)
    repo = require_pinned_repo(args.repo, summary)
    modules = declared_modules(repo)
    expected_modules = summary["inputs"]["declared_corpus_modules"]
    if len(modules) != expected_modules:
        raise SystemExit(f"expected {expected_modules} modules, found {len(modules)}")
    body = ["module namespace_census.complete_population_root", ""]
    body.extend(f"import {module}" for module in modules)
    body.extend(("", 'data census_root: String = "complete"', ""))
    args.output.write_text("\n".join(body), encoding="utf-8")


if __name__ == "__main__":
    main()
