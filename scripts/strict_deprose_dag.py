#!/usr/bin/env python3
"""Strict de-prose pass: keep only line-1 path + terse header + strip all other // lines.

Warning: every `//` line after `module` is deleted—non-idempotent if a target file gains
authored in-body commentary; scoped to the pinned allowlist for that reason.

Coproduct one-liners (`// 🟢|🟡|🔴 coproduct dissolution — DECISIONS.md Part 6 · …`) are
preserved and (re)injected from merge-base `92cb26402` Practice-4 / SL-3229 state
(operator directive 2026-05-17, PR #3234 modeling-discipline alignment).

`// Owns:` is a **manifest of top-level module symbols** in **file order**: every
`type`, `data`, and `fn` binding in the `.dag` body (deduped by name), not only headline
carriers—so regenerated headers stay aligned with actual exports.

Run with `--check` to verify allowlisted files already match merge-base-derived output
without writing (exit 1 on drift).
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

DECISIONS_REL = "src/v4/DECISIONS.md"

MERGE_BASE = "92cb26402eeb21471acb6ac47559cbae3b52afdb"

TYPE_RE = re.compile(r"^type\s+([A-Za-z_][A-Za-z0-9_]*)\b")
DATA_RE = re.compile(r"^data\s+([A-Za-z_][A-Za-z0-9_]*)\b")
FN_RE = re.compile(r"^fn\s+([A-Za-z_][A-Za-z0-9_]*)\b")
COPRODUCT_TAG_RE = re.compile(
    r"^\s*//\s*[🟢🟡🔴]\s+coproduct dissolution\b",
)

PART6_SLUG_HEAD_RE = re.compile(r"^### (SL-3229-[A-Z0-9-]+|CP-3229-[A-Z0-9-]+)\b")

# Non-coproduct merge-base receipts (strict de-prose strips their `//` bodies) that
# must still have a Part 6 row + appear on the live `// Ledger:` line for inventory.
EXTRA_PART6_SLUGS_BY_REL: dict[str, frozenset[str]] = {
    "src/v4/extdeps/languages/verilog.dag": frozenset(
        {
            "SL-3229-VERILOG-VECTOR-RANGE",
            "SL-3229-VERILOG-COST",
        }
    ),
    "src/v4/extdeps/languages/ptx.dag": frozenset(
        {
            "SL-3229-PTX-DIM3",
            "SL-3229-PTX-COST",
        }
    ),
    "src/v4/extdeps/languages/llvm_ir.dag": frozenset(
        {
            "SL-3229-LLVM-WIDTH",
            "SL-3229-LLVM-OPS",
        }
    ),
    "src/v4/std/integer.dag": frozenset(
        {
            "CP-3229-VERIFY",
            "SL-3229-INTEGER-GROUP-COMPLETION",
        }
    ),
    "src/v4/std/float.dag": frozenset({"SL-3229-FLOAT-NOMINAL"}),
}


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


def authority_block(lines: list[str], i: int) -> str:
    """Contiguous `//` lines above a `type` row plus `//` lines between `type` and `=`/`|`."""
    parts: list[str] = []
    k = i - 1
    while k >= 0 and lines[k].lstrip().startswith("//"):
        parts.insert(0, lines[k])
        k -= 1
    above = "\n".join(parts)
    j = i + 1
    between: list[str] = []
    while j < len(lines):
        t = lines[j].strip()
        if t.startswith("//"):
            between.append(lines[j])
            j += 1
            continue
        if not t:
            j += 1
            continue
        if t.startswith("=") or t.startswith("|"):
            break
        break
    mid = "\n".join(between)
    return above + ("\n" + mid if mid else "")


def practice4_face(block: str) -> str | None:
    """Return 'green', 'yellow', 'red', or None from the Practice-4 coproduct header."""
    key = "Coproduct dissolution (Practice 4 / modeling-discipline.md §4)"
    pos = block.rfind(key)
    if pos < 0:
        pos = block.rfind("Coproduct dissolution (Practice 4")
    tail = block[pos:] if pos >= 0 else block
    header = tail.split("// Terminal:")[0][:8000]
    if re.search(r"🟡\s*YELLOW", header):
        return "yellow"
    if "🔴" in header:
        return "red"
    if re.search(r"🟢\s*GREEN", header):
        return "green"
    return None


def verilog_yellow_ref(block: str) -> str:
    """Split Verilog 🟡 coproducts: #3200 first-consumer matrix vs Wave-A2 list-non-empty."""
    key = "Coproduct dissolution (Practice 4 / modeling-discipline.md §4)"
    pos = block.rfind(key)
    if pos < 0:
        pos = block.rfind("Coproduct dissolution (Practice 4")
    tail = block[pos:] if pos >= 0 else block
    ledger_header = tail.split("// Terminal:")[0]
    if (
        "#3200" in ledger_header
        or "first meaning-consumer" in ledger_header
        or "first consumer of" in ledger_header
    ):
        return "SL-3229-VERILOG-D3200"
    if (
        "TRACKED-SCAFFOLD: spec-non-empty" in block
        or "Twenty-six sites" in block
        or ("RQ-3" in block and "Wave-A2" in block)
    ):
        return "SL-3229-VERILOG-NONEMPTY"
    return "SL-3229-VERILOG-D3200"


def coproduct_tag_from_merge_base(rel: str) -> dict[str, tuple[str, str]]:
    """Map coproduct `type` name -> (emoji, Part-6 ref slug) from merge-base authority."""
    lines = git_merge_base_lines(rel)
    out: dict[str, tuple[str, str]] = {}
    for i, _ln in enumerate(lines):
        nm = is_coproduct(lines, i)
        if not nm:
            continue
        block = authority_block(lines, i)
        if rel.endswith("llvm_ir.dag") and nm == "LlvmType" and "RAW-Int WIDTH RESIDUAL" in block:
            out[nm] = ("🟡", "SL-3229-LLVM-WIDTH")
            continue
        face = practice4_face(block)
        if face == "green":
            out[nm] = ("🟢", "CP-3229-GREEN-TERMINAL")
        elif face == "red":
            out[nm] = ("🔴", "CP-3229-RED-PRACTICE4")
        elif face == "yellow":
            if "verilog" in rel:
                out[nm] = ("🟡", verilog_yellow_ref(block))
            elif "ptx" in rel:
                out[nm] = ("🟡", "SL-3229-PTX-DIM3")
            elif "float" in rel:
                out[nm] = ("🟡", "SL-3229-FLOAT-NOMINAL")
            else:
                out[nm] = ("🟡", "CP-3229-GREEN-TERMINAL")
        else:
            out[nm] = ("🟢", "CP-3229-GREEN-TERMINAL")
    return out


def part6_slugs_in_decisions(decisions_text: str) -> set[str]:
    if "## Part 6" not in decisions_text:
        return set()
    chunk = decisions_text.split("## Part 6", 1)[1]
    out: set[str] = set()
    for ln in chunk.splitlines():
        if ln.startswith("## ") and "Part 6" not in ln:
            break
        m = PART6_SLUG_HEAD_RE.match(ln)
        if m:
            out.add(m.group(1))
    return out


def required_ledger_slugs(rel: str, tag_map: dict[str, tuple[str, str]]) -> set[str]:
    refs = {ref for _em, ref in tag_map.values()}
    refs |= set(EXTRA_PART6_SLUGS_BY_REL.get(rel, frozenset()))
    return refs


def format_ledger_line(rel: str, tag_map: dict[str, tuple[str, str]]) -> str:
    slugs = ", ".join(sorted(required_ledger_slugs(rel, tag_map)))
    return f"// Ledger: DECISIONS.md Part 6 (PR #3229): {slugs}.\n"


def assert_part6_inventory(decisions_text: str, all_required: set[str]) -> None:
    present = part6_slugs_in_decisions(decisions_text)
    missing = sorted(all_required - present)
    if missing:
        print(
            "FAIL: Part 6 missing authoritative rows for: " + ", ".join(missing),
            file=sys.stderr,
        )
        sys.exit(1)


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
    """Top-level `type`, `data`, and `fn` names in file order (deduped)."""
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
            continue
        m = FN_RE.match(s)
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


def materialize_deprose_text(path: Path, rel: str, tag_map: dict[str, tuple[str, str]], header: str) -> str:
    """Return full file text after strip + coproduct reinjection (no disk write)."""
    text = path.read_text()
    lines = text.splitlines(keepends=True)
    idx = next(i for i, ln in enumerate(lines) if ln.startswith("module "))
    module_line = lines[idx]
    body = "".join(lines[idx + 1 :])
    new_body = strip_body_comments(body)
    new_body = inject_coproduct_tags(new_body, rel, tag_map)
    return header + module_line + new_body


def rewrite(path: Path, header: str, rel: str, tag_map: dict[str, tuple[str, str]]) -> None:
    path.write_text(materialize_deprose_text(path, rel, tag_map, header))


def run_check(specs: list[tuple[str, str, str, str, str]]) -> None:
    """Exit 0 only if each allowlisted file already matches merge-base-derived output."""
    decisions_path = ROOT / DECISIONS_REL
    decisions_text = decisions_path.read_text()
    union_required: set[str] = set()
    for rel, *_rest in specs:
        union_required |= required_ledger_slugs(rel, coproduct_tag_from_merge_base(rel))
    assert_part6_inventory(decisions_text, union_required)

    drift: list[str] = []
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
            f"{format_ledger_line(rel, tag_map)}"
            f"\n"
        )
        expected = materialize_deprose_text(path, rel, tag_map, header)
        actual = path.read_text()
        if expected != actual:
            drift.append(rel)

    if drift:
        print("FAIL: --check drift on: " + ", ".join(drift), file=sys.stderr)
        sys.exit(1)
    print("OK: strict_deprose_dag --check (all allowlisted files match).")


def main() -> None:
    argv = sys.argv[1:]
    if argv not in ([], ["--check"]):
        raise SystemExit("usage: strict_deprose_dag.py [--check]")
    check_only = argv == ["--check"]

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

    decisions_path = ROOT / DECISIONS_REL
    decisions_text = decisions_path.read_text()
    union_required: set[str] = set()
    for rel, *_rest in specs:
        union_required |= required_ledger_slugs(rel, coproduct_tag_from_merge_base(rel))
    assert_part6_inventory(decisions_text, union_required)

    if check_only:
        run_check(specs)
        return

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
            f"{format_ledger_line(rel, tag_map)}"
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
