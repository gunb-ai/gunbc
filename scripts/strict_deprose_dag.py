#!/usr/bin/env python3
"""Strict de-prose pass: keep only line-1 path + terse header + strip all other // lines.

Warning: every `//` line after `module` is deleted—non-idempotent if a target file gains
authored in-body commentary; scoped to the pinned allowlist for that reason.

Coproduct one-liners (`// 🟢|🟡|🔴 coproduct dissolution — DECISIONS.md Part 6 · …`) are
preserved and (re)injected from merge-base `92cb26402` Practice-4 / SL-3229 state
(operator directive 2026-05-17, PR #3234 modeling-discipline alignment).
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

MERGE_BASE = "92cb26402eeb21471acb6ac47559cbae3b52afdb"

TYPE_RE = re.compile(r"^type\s+([A-Za-z_][A-Za-z0-9_]*)\b")
DATA_RE = re.compile(r"^data\s+([A-Za-z_][A-Za-z0-9_]*)\b")
COPRODUCT_TAG_RE = re.compile(
    r"^\s*//\s*[🟢🟡🔴]\s+coproduct dissolution\b",
)


def git_merge_base_lines(rel: str) -> list[str]:
    out = subprocess.check_output(
        ["git", "show", f"{MERGE_BASE}:{rel}"],
        cwd=ROOT,
        text=True,
    )
    return out.splitlines()


def is_coproduct(lines: list[str], i: int) -> str | None:
    """Return carrier name if `lines[i]` starts a sum type (N>=2 variants, `|`)."""
    m = TYPE_RE.match(lines[i])
    if not m:
        return None
    name = m.group(1)
    if "|" in lines[i] and "=" in lines[i]:
        return name
    j = i + 1
    pipes = 0
    while j < len(lines):
        s = lines[j].strip()
        if not s or s.startswith("//"):
            j += 1
            continue
        if s.startswith("type ") or s.startswith("data ") or s.startswith("module ") or s.startswith("fn "):
            break
        if s.startswith("{") and pipes == 0 and "=" not in lines[i]:
            return None
        if s.startswith("="):
            j += 1
            continue
        if s.startswith("|"):
            pipes += 1
        j += 1
        if pipes >= 1:
            return name
    return None


def coproduct_tag_from_merge_base(rel: str) -> dict[str, tuple[str, str]]:
    """Map coproduct `type` name -> (emoji, Part-6 ref slug) from merge-base authority."""
    lines = git_merge_base_lines(rel)
    out: dict[str, tuple[str, str]] = {}
    for i, _ln in enumerate(lines):
        nm = is_coproduct(lines, i)
        if not nm:
            continue
        lo = max(0, i - 160)
        window = "\n".join(lines[lo:i])
        if rel.endswith("llvm_ir.dag") and nm == "LlvmType" and "RAW-Int WIDTH RESIDUAL" in window:
            out[nm] = ("🟡", "SL-3229-LLVM-WIDTH")
            continue
        last_class: str | None = None
        for k in range(i - 1, lo - 1, -1):
            ln = lines[k]
            if not ln.lstrip().startswith("//"):
                continue
            if "classification:" in ln:
                last_class = ln
                break
            if "🟡 YELLOW" in ln and (
                "deferred-on-consumer" in ln
                or "re-scoped" in ln
                or "namable richer" in ln
            ):
                last_class = ln
                break
        if last_class:
            if "🟢 GREEN" in last_class:
                out[nm] = ("🟢", "CP-3229-GREEN-TERMINAL")
            elif "🔴" in last_class:
                out[nm] = ("🔴", "CP-3229-GREEN-TERMINAL")
            elif "🟡" in last_class or "YELLOW" in last_class:
                if "verilog" in rel:
                    out[nm] = ("🟡", "SL-3229-VERILOG-NONEMPTY")
                elif "ptx" in rel:
                    out[nm] = ("🟡", "SL-3229-PTX-DIM3")
                elif "float" in rel:
                    out[nm] = ("🟡", "SL-3229-FLOAT-NOMINAL")
                else:
                    out[nm] = ("🟡", "CP-3229-GREEN-TERMINAL")
            else:
                out[nm] = ("🟢", "CP-3229-GREEN-TERMINAL")
        else:
            out[nm] = ("🟢", "CP-3229-GREEN-TERMINAL")
    return out


def format_coproduct_tag(emoji: str, ref: str) -> str:
    return f"// {emoji} coproduct dissolution — DECISIONS.md Part 6 · {ref}."


def inject_coproduct_tags(body: str, _rel: str, tag_map: dict[str, tuple[str, str]]) -> str:
    bl = body.splitlines()
    out_lines: list[str] = []
    j = 0
    while j < len(bl):
        nm = is_coproduct(bl, j)
        if nm and nm in tag_map:
            prev = bl[j - 1] if j > 0 else ""
            if not COPRODUCT_TAG_RE.match(prev):
                em, ref = tag_map[nm]
                out_lines.append(format_coproduct_tag(em, ref))
            out_lines.append(bl[j])
            j += 1
            continue
        out_lines.append(bl[j])
        j += 1
    trailing = "\n" if body.endswith("\n") or not body else ""
    return "\n".join(out_lines) + trailing


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
        if line.lstrip().startswith("//") and not COPRODUCT_TAG_RE.match(line):
            continue
        out_lines.append(line)
    return "".join(out_lines)


def rewrite(path: Path, header: str, rel: str, tag_map: dict[str, tuple[str, str]]) -> None:
    text = path.read_text()
    lines = text.splitlines(keepends=True)
    idx = next(i for i, ln in enumerate(lines) if ln.startswith("module "))
    module_line = lines[idx]
    body = "".join(lines[idx + 1 :])
    new_body = strip_body_comments(body)
    new_body = inject_coproduct_tags(new_body, rel, tag_map)
    path.write_text(header + module_line + new_body)


def main() -> None:
    specs: list[tuple[str, str, str, str, str]] = [
        (
            "src/v4/extdeps/languages/verilog.dag",
            "// Scope: IEEE 1364-2005 Verilog structural carriers (T-4.9).",
            "// Anchor: https://standards.ieee.org/ieee/1364/3641/",
            "// Consumes: std/node.dag; Int kernel-ambient.",
            "// Status: T-4.9 PASS (IN-B); import v4.std.node Symbol only.",
        ),
        (
            "src/v4/extdeps/languages/llvm_ir.dag",
            "// Scope: LLVM 18 LangRef IR structural vocabulary (T-4.12).",
            "// Anchor: https://releases.llvm.org/18.1.8/docs/LangRef.html",
            "// Consumes: std/node.dag; Int kernel-ambient.",
            "// Status: T-4.12 PASS (B2-OMNI); import v4.std.node Symbol only.",
        ),
        (
            "src/v4/extdeps/languages/ptx.dag",
            "// Scope: NVIDIA PTX ISA 8.5 SIMT structural classifiers (T-4.14).",
            "// Anchor: https://docs.nvidia.com/cuda/pdf/ptx_isa_8.5.pdf — TOC https://docs.nvidia.com/cuda/parallel-thread-execution/index.html",
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
        tag_map = coproduct_tag_from_merge_base(rel)
        names = carrier_names(path)
        owns_line = "// Owns: " + ", ".join(names)
        first = path.read_text().splitlines()[0]
        if not first.startswith("// src/"):
            raise SystemExit(f"{rel}: expected line 1 // src/…, got {first!r}")
        header = (
            f"{first}\n"
            f"{scope}\n"
            f"{anchor}\n"
            f"{owns_line}\n"
            f"{consumes}\n"
            f"{status}\n"
            f"// Ledger: DECISIONS.md Part 6 (PR #3229).\n"
            f"\n"
        )
        rewrite(path, header, rel, tag_map)
        c, t, pct = comment_ratio(path.read_text())
        report.append((rel, pct))
        print(f"{rel}: {c}/{t} comments = {pct:.1f}%")

    worst = max(pct for _, pct in report)
    if worst >= 20.0:
        print(f"FAIL: worst comment% {worst:.1f} >= 20", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
