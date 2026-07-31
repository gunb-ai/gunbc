#!/usr/bin/env python3
"""Generate the uncommitted root that makes every declared module reachable."""

import argparse
import pathlib
import re

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
    parser.add_argument("--expected-modules", type=int, default=2746)
    args = parser.parse_args()

    modules = declared_modules(args.repo.resolve())
    if len(modules) != args.expected_modules:
        raise SystemExit(f"expected {args.expected_modules} modules, found {len(modules)}")
    body = ["module namespace_census.complete_population_root", ""]
    body.extend(f"import {module}" for module in modules)
    body.extend(("", 'data census_root: String = "complete"', ""))
    args.output.write_text("\n".join(body), encoding="utf-8")


if __name__ == "__main__":
    main()
