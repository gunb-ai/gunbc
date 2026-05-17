#!/usr/bin/env python3
"""Strict de-prose pass: keep only line-1 path + terse header + strip all other // lines."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

TYPE_RE = re.compile(r"^type\s+([A-Za-z_][A-Za-z0-9_]*)\b")
DATA_RE = re.compile(r"^data\s+([A-Za-z_][A-Za-z0-9_]*)\b")


def carrier_names(path: Path) -> list[str]:
    """Declaration-order carrier names: `type` rows first, then `data` rows (deduped)."""
    seen: set[str] = set()
    out: list[str] = []
    for line in path.read_text().splitlines():
        s = line.strip()
        m = TYPE_RE.match(s)
        if m:
            name = m.group(1)
            if name not in seen:
                seen.add(name)
                out.append(name)
            continue
        m = DATA_RE.match(s)
        if m:
            name = m.group(1)
            if name not in seen:
                seen.add(name)
                out.append(name)
    return out


def comment_ratio(text: str) -> tuple[int, int, float]:
    lines = text.splitlines()
    total = len(lines) if lines else 1
    c = sum(1 for ln in lines if ln.lstrip().startswith("//"))
    return c, total, 100.0 * c / total


def strip_body_comments(after_module: str) -> str:
    out_lines: list[str] = []
    for line in after_module.splitlines(True):
        if line.lstrip().startswith("//"):
            continue
        out_lines.append(line)
    return "".join(out_lines)


def rewrite(path: Path, header: str) -> None:
    text = path.read_text()
    lines = text.splitlines(keepends=True)
    idx = next(i for i, ln in enumerate(lines) if ln.startswith("module "))
    module_line = lines[idx]
    body = "".join(lines[idx + 1 :])
    new_body = strip_body_comments(body)
    path.write_text(header + module_line + new_body)


def main() -> None:
    specs: list[tuple[str, str, str, str]] = [
        (
            "src/v4/extdeps/languages/verilog.dag",
            "// Scope: IEEE 1364-2005 Verilog structural carriers (T-4.9).",
            "// Anchor: https://ieeexplore.ieee.org/document/9576818",
            "// Consumes: std/node.dag, std/algebra.dag, std/primitive.dag, extdeps/coordination.dag",
            "// Status: T-4.9 PASS (IN-B); import v4.std.node Symbol only.",
        ),
        (
            "src/v4/extdeps/languages/llvm_ir.dag",
            "// Scope: LLVM 18 LangRef IR structural vocabulary (T-4.12).",
            "// Anchor: https://releases.llvm.org/18.1.8/docs/LangRef.html",
            "// Consumes: std/node.dag, std/algebra.dag, std/primitive.dag",
            "// Status: T-4.12 PASS (B2-OMNI); import v4.std.node Symbol only.",
        ),
        (
            "src/v4/extdeps/languages/ptx.dag",
            "// Scope: NVIDIA PTX ISA 8.5 SIMT structural classifiers (T-4.14).",
            "// Anchor: https://docs.nvidia.com/cuda/parallel-thread-execution/index.html",
            "// Consumes: (none; Int kernel-ambient).",
            "// Status: T-4.14 PASS (IN-B).",
        ),
        (
            "src/v4/std/integer.dag",
            "// Scope: Abstract Int/UInt, fixed-width projections, Tier-2 divide/modulo (T-3).",
            "// Anchor: https://en.wikipedia.org/wiki/Integer",
            "// Consumes: std/node.dag (Instantiation, Symbol); std/nat.dag (Nat); std/machine.dag (MachineWidth, PointerWidth, Word8, Word16, Word32, Word64, Word128); std/algebra.dag (Magma, Semigroup, Monoid, Group, AbelianGroup, Ring, OrderedRing, Ordering); std/diagnostic.dag (Diagnostic, Locus, Correction, NoCorrectionReason, Outcome<T> Tier-2 divide/modulo).",
            "// Status: T-3 modeled.",
        ),
        (
            "src/v4/std/float.dag",
            "// Scope: IEEE-754 Float32/Float64 semantic carrier + Tier-2 compare (T-3).",
            "// Anchor: https://en.wikipedia.org/wiki/IEEE_754",
            "// Consumes: std/node.dag (Symbol); std/machine.dag (Bit, Word32, Word64); std/algebra.dag (Ordering, Less, Equal, Greater); std/diagnostic.dag (Diagnostic, Outcome, PortLocus, Produced, Rejected, Unavailable, UserInputBoundary); std/logic.dag (Bool); std/nat.dag (Nat).",
            "// Status: T-3 modeled.",
        ),
    ]

    report: list[tuple[str, float]] = []
    for rel, scope, anchor, consumes, status in specs:
        path = ROOT / rel
        names = carrier_names(path)
        owns_lines = "\n".join(f"//   {n}" for n in names)
        first = path.read_text().splitlines()[0]
        if not first.startswith("// src/"):
            raise SystemExit(f"{rel}: expected line 1 // src/…, got {first!r}")
        header = (
            f"{first}\n"
            f"{scope}\n"
            f"{anchor}\n"
            f"// Owns:\n"
            f"{owns_lines}\n"
            f"{consumes}\n"
            f"{status}\n"
            f"\n"
        )
        rewrite(path, header)
        c, t, pct = comment_ratio(path.read_text())
        report.append((rel, pct))
        print(f"{rel}: {c}/{t} comments = {pct:.1f}%")

    worst = max(pct for _, pct in report)
    if worst >= 20.0:
        print(f"FAIL: worst comment% {worst:.1f} >= 20", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
