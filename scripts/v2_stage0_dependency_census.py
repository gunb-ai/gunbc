#!/usr/bin/env python3
"""Census generated v2 stage0 Rust module dependencies.

This is an evidence tool for stage0 crate-splitting design. It reads the
modules declared by src/v2/stage0/src/lib.rs, scans direct `crate::module`
references among those modules, and reports size, fan-in/fan-out, and strongly
connected components. It does not compile or modify stage0.
"""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_STAGE0_SRC = ROOT / "src/v2/stage0/src"

PUB_MOD_RE = re.compile(r"^\s*pub\s+mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;", re.MULTILINE)
CRATE_REF_RE = re.compile(r"\bcrate::([A-Za-z_][A-Za-z0-9_]*)\b")
RUST_IDENT_CHARS = frozenset("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_")


@dataclass(frozen=True)
class ModuleInfo:
    name: str
    path: Path
    bytes: int
    lines: int
    deps: tuple[str, ...]


@dataclass(frozen=True)
class Component:
    modules: tuple[str, ...]
    bytes: int
    lines: int
    internal_edges: tuple[tuple[str, str], ...]
    outgoing: tuple[str, ...]


def _read(path: Path) -> str:
    if not path.is_file():
        raise SystemExit(f"missing required path: {path}")
    return path.read_text(encoding="utf-8")


def strip_rust_non_code(text: str) -> str:
    """Replace comments and string/char literal contents with spaces.

    The generated stage0 emitter contains large Rust source templates as string
    literals. A raw `crate::foo` regex over those strings reports dependencies
    that rustc never resolves for the stage0 crate. This scanner is deliberately
    conservative: it preserves byte count/newlines enough for regex scanning,
    but erases normal strings, raw strings, chars, and comments.
    """

    out: list[str] = []
    i = 0
    n = len(text)
    state = "code"
    block_depth = 0
    raw_hashes = 0

    def blank(ch: str) -> str:
        return "\n" if ch == "\n" else " "

    def looks_like_lifetime_or_label(pos: int) -> bool:
        if pos + 1 >= n or text[pos] != "'":
            return False
        first = text[pos + 1]
        if first != "_" and not first.isalpha():
            return False
        j = pos + 2
        while j < n and text[j] in RUST_IDENT_CHARS:
            j += 1
        return j >= n or text[j] != "'"

    while i < n:
        ch = text[i]
        nxt = text[i + 1] if i + 1 < n else ""

        if state == "code":
            if ch == "/" and nxt == "/":
                out.extend("  ")
                i += 2
                state = "line_comment"
            elif ch == "/" and nxt == "*":
                out.extend("  ")
                i += 2
                state = "block_comment"
                block_depth = 1
            elif ch == "r":
                j = i + 1
                while j < n and text[j] == "#":
                    j += 1
                if j < n and text[j] == '"':
                    raw_hashes = j - i - 1
                    out.extend(" " * (raw_hashes + 2))
                    i = j + 1
                    state = "raw_string"
                else:
                    out.append(ch)
                    i += 1
            elif ch == '"':
                out.append(" ")
                i += 1
                state = "string"
            elif ch == "'" and not looks_like_lifetime_or_label(i):
                out.append(" ")
                i += 1
                state = "char"
            else:
                out.append(ch)
                i += 1
        elif state == "line_comment":
            out.append(blank(ch))
            i += 1
            if ch == "\n":
                state = "code"
        elif state == "block_comment":
            if ch == "/" and nxt == "*":
                out.extend("  ")
                i += 2
                block_depth += 1
            elif ch == "*" and nxt == "/":
                out.extend("  ")
                i += 2
                block_depth -= 1
                if block_depth == 0:
                    state = "code"
            else:
                out.append(blank(ch))
                i += 1
        elif state == "string":
            if ch == "\\" and nxt:
                out.extend(blank(ch) + blank(nxt))
                i += 2
            elif ch == '"':
                out.append(" ")
                i += 1
                state = "code"
            else:
                out.append(blank(ch))
                i += 1
        elif state == "char":
            if ch == "\\" and nxt:
                out.extend(blank(ch) + blank(nxt))
                i += 2
            elif ch == "'":
                out.append(" ")
                i += 1
                state = "code"
            else:
                out.append(blank(ch))
                i += 1
        elif state == "raw_string":
            if ch == '"':
                hashes = text[i + 1 : i + 1 + raw_hashes]
                if hashes == "#" * raw_hashes:
                    out.extend(" " * (1 + raw_hashes))
                    i += 1 + raw_hashes
                    state = "code"
                else:
                    out.append(" ")
                    i += 1
            else:
                out.append(blank(ch))
                i += 1
        else:
            raise AssertionError(f"unknown scanner state: {state}")

    return "".join(out)


def stage0_modules(stage0_src: Path) -> list[str]:
    lib_rs = stage0_src / "lib.rs"
    modules = PUB_MOD_RE.findall(_read(lib_rs))
    if not modules:
        raise SystemExit(f"no `pub mod` declarations found in {lib_rs}")
    return modules


def module_info(stage0_src: Path, modules: Iterable[str]) -> dict[str, ModuleInfo]:
    module_set = set(modules)
    infos: dict[str, ModuleInfo] = {}
    for module in sorted(module_set):
        path = stage0_src / f"{module}.rs"
        text = _read(path)
        code_text = strip_rust_non_code(text)
        deps = sorted(
            dep
            for dep in set(CRATE_REF_RE.findall(code_text))
            if dep in module_set and dep != module
        )
        infos[module] = ModuleInfo(
            name=module,
            path=path,
            bytes=path.stat().st_size,
            lines=text.count("\n") + (0 if text.endswith("\n") else 1),
            deps=tuple(deps),
        )
    return infos


def strongly_connected_components(graph: dict[str, tuple[str, ...]]) -> list[tuple[str, ...]]:
    index = 0
    stack: list[str] = []
    on_stack: set[str] = set()
    indices: dict[str, int] = {}
    lowlinks: dict[str, int] = {}
    components: list[tuple[str, ...]] = []

    def visit(node: str) -> None:
        nonlocal index
        indices[node] = index
        lowlinks[node] = index
        index += 1
        stack.append(node)
        on_stack.add(node)

        for dep in graph[node]:
            if dep not in indices:
                visit(dep)
                lowlinks[node] = min(lowlinks[node], lowlinks[dep])
            elif dep in on_stack:
                lowlinks[node] = min(lowlinks[node], indices[dep])

        if lowlinks[node] == indices[node]:
            component: list[str] = []
            while True:
                dep = stack.pop()
                on_stack.remove(dep)
                component.append(dep)
                if dep == node:
                    break
            components.append(tuple(sorted(component)))

    for node in sorted(graph):
        if node not in indices:
            visit(node)
    return components


def component_census(infos: dict[str, ModuleInfo]) -> list[Component]:
    graph = {name: info.deps for name, info in infos.items()}
    components = []
    for modules in strongly_connected_components(graph):
        module_set = set(modules)
        internal_edges = sorted(
            (module, dep)
            for module in modules
            for dep in graph[module]
            if dep in module_set
        )
        outgoing = sorted(
            dep
            for module in modules
            for dep in graph[module]
            if dep not in module_set
        )
        components.append(
            Component(
                modules=modules,
                bytes=sum(infos[module].bytes for module in modules),
                lines=sum(infos[module].lines for module in modules),
                internal_edges=tuple(internal_edges),
                outgoing=tuple(dict.fromkeys(outgoing)),
            )
        )
    components.sort(key=lambda c: (len(c.modules), c.bytes, c.modules), reverse=True)
    return components


def reverse_deps(infos: dict[str, ModuleInfo]) -> dict[str, tuple[str, ...]]:
    incoming: dict[str, set[str]] = {name: set() for name in infos}
    for module, info in infos.items():
        for dep in info.deps:
            incoming[dep].add(module)
    return {name: tuple(sorted(deps)) for name, deps in incoming.items()}


def as_json(stage0_src: Path, infos: dict[str, ModuleInfo]) -> dict[str, object]:
    incoming = reverse_deps(infos)
    components = component_census(infos)
    return {
        "stage0_src": str(stage0_src),
        "summary": {
            "module_count": len(infos),
            "total_bytes": sum(info.bytes for info in infos.values()),
            "total_lines": sum(info.lines for info in infos.values()),
            "edge_count": sum(len(info.deps) for info in infos.values()),
            "component_count": len(components),
            "cyclic_component_count": sum(1 for c in components if len(c.modules) > 1),
        },
        "modules": [
            {
                "name": info.name,
                "path": str(info.path),
                "bytes": info.bytes,
                "lines": info.lines,
                "deps": list(info.deps),
                "incoming": list(incoming[info.name]),
                "fan_out": len(info.deps),
                "fan_in": len(incoming[info.name]),
            }
            for info in sorted(infos.values(), key=lambda i: i.name)
        ],
        "components": [
            {
                "modules": list(component.modules),
                "bytes": component.bytes,
                "lines": component.lines,
                "internal_edges": list(component.internal_edges),
                "outgoing": list(component.outgoing),
                "module_count": len(component.modules),
            }
            for component in components
        ],
    }


def _size_kib(n: int) -> str:
    return f"{n / 1024:.1f} KiB"


def _table(headers: tuple[str, ...], rows: Iterable[tuple[object, ...]]) -> str:
    lines = [
        "| " + " | ".join(headers) + " |",
        "| " + " | ".join("---" for _ in headers) + " |",
    ]
    for row in rows:
        lines.append("| " + " | ".join(str(cell) for cell in row) + " |")
    return "\n".join(lines)


def as_markdown(stage0_src: Path, infos: dict[str, ModuleInfo], *, top: int) -> str:
    data = as_json(stage0_src, infos)
    summary = data["summary"]
    modules = data["modules"]
    components = data["components"]
    assert isinstance(summary, dict)
    assert isinstance(modules, list)
    assert isinstance(components, list)

    largest = sorted(modules, key=lambda m: int(m["bytes"]), reverse=True)[:top]
    fan_out = sorted(modules, key=lambda m: (int(m["fan_out"]), int(m["bytes"])), reverse=True)[:top]
    fan_in = sorted(modules, key=lambda m: (int(m["fan_in"]), int(m["bytes"])), reverse=True)[:top]
    largest_components = components[:top]

    lines = [
        "# v2 Stage0 Dependency Census",
        "",
        f"Stage0 source: `{stage0_src}`",
        "",
        "## Summary",
        "",
        _table(
            ("Metric", "Value"),
            (
                ("modules", summary["module_count"]),
                ("direct module edges", summary["edge_count"]),
                ("strongly connected components", summary["component_count"]),
                ("cyclic components", summary["cyclic_component_count"]),
                ("total size", _size_kib(int(summary["total_bytes"]))),
                ("total lines", summary["total_lines"]),
            ),
        ),
        "",
        "## Largest Modules",
        "",
        _table(
            ("Module", "Size", "Lines", "Fan-in", "Fan-out"),
            (
                (
                    f"`{m['name']}`",
                    _size_kib(int(m["bytes"])),
                    m["lines"],
                    m["fan_in"],
                    m["fan_out"],
                )
                for m in largest
            ),
        ),
        "",
        "## Top Fan-out",
        "",
        _table(
            ("Module", "Fan-out", "Size", "Deps"),
            (
                (
                    f"`{m['name']}`",
                    m["fan_out"],
                    _size_kib(int(m["bytes"])),
                    ", ".join(f"`{dep}`" for dep in m["deps"][:8])
                    + (" ..." if len(m["deps"]) > 8 else ""),
                )
                for m in fan_out
            ),
        ),
        "",
        "## Top Fan-in",
        "",
        _table(
            ("Module", "Fan-in", "Size", "Incoming"),
            (
                (
                    f"`{m['name']}`",
                    m["fan_in"],
                    _size_kib(int(m["bytes"])),
                    ", ".join(f"`{dep}`" for dep in m["incoming"][:8])
                    + (" ..." if len(m["incoming"]) > 8 else ""),
                )
                for m in fan_in
            ),
        ),
        "",
        "## Largest Strongly Connected Components",
        "",
        _table(
            ("Modules", "Size", "Lines", "Internal edges", "Outgoing deps"),
            (
                (
                    ", ".join(f"`{name}`" for name in c["modules"][:8])
                    + (" ..." if len(c["modules"]) > 8 else ""),
                    _size_kib(int(c["bytes"])),
                    c["lines"],
                    len(c["internal_edges"]),
                    len(c["outgoing"]),
                )
                for c in largest_components
            ),
        ),
        "",
        "Interpretation note: SCCs are the lower bound for direct crate splits. A cyclic component must stay together unless the `.dag` model or generated APIs are changed to break the cycle.",
    ]
    return "\n".join(lines)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--stage0-src",
        type=Path,
        default=DEFAULT_STAGE0_SRC,
        help="path to src/v2/stage0/src",
    )
    parser.add_argument(
        "--format",
        choices=("markdown", "json"),
        default="markdown",
        help="output format",
    )
    parser.add_argument("--top", type=int, default=12, help="rows per markdown section")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    stage0_src = args.stage0_src.resolve()
    modules = stage0_modules(stage0_src)
    infos = module_info(stage0_src, modules)
    if args.format == "json":
        print(json.dumps(as_json(stage0_src, infos), indent=2, sort_keys=True))
    else:
        print(as_markdown(stage0_src, infos, top=args.top))


if __name__ == "__main__":
    main()
