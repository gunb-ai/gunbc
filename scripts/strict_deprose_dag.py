#!/usr/bin/env python3
"""Strict de-prose pass: keep only line-1 path + terse header + strip all other // lines.

Warning: every `//` line after `module` is deleted—non-idempotent if a target file gains
authored in-body commentary; scoped to the pinned allowlist for that reason.

Coproduct one-liners (`// 🟢|🟡|🔴 coproduct dissolution · …`) are (re)written from merge-base
`92cb26402` Practice-4 / SL-3229 state (operator directive 2026-05-17, PR #3234
modeling-discipline alignment): an existing tag with the wrong slug is replaced so
`--check` cannot pass on stale pointers.

RULING-1 (operator 2026-05-19): each braced record `type` (not a sum coproduct) gets a
single `// 🟢 grounded.` or `// 🟡 grounded.` line (lexeme-shaped `String` slots in
the record body ⇒ 🟡). Coproduct rows already carry 🟢/🟡 dissolution state and do
not get a second grounded line. Grounded lines are **not** preserved across strip —
they are re-injected every run so spacing stays canonical.

`// Owns:` has been removed from all files, including those previously on this
script's `specs` allowlist. Practice 9 treats it as a parallel ledger everywhere;
the allowlist exception is dissolved.

`// Ledger:` lists **slug inventory for the live substrate**: one ledger ref per live sum
coproduct (from the merge-base tag map) plus **EXTRA** non-coproduct scaffolds
(`EXTRA_PART6_SLUGS_BY_REL`). A live coproduct name absent from the merge-base map is
a hard failure (Practice 4 fail-closed). There is **no** separate maintained ledger file;
slugs are merge-base-derived + script-enforced only (operator 2026-05-19).

Run with `--check` to verify allowlisted files already match merge-base-derived output
without writing (exit 1 on drift).
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

MERGE_BASE = "92cb26402eeb21471acb6ac47559cbae3b52afdb"
# Audit recovery: `coproduct_tag_from_merge_base` uses `git show MERGE_BASE:path`.
# CI and local dev assume a full object DB (not a shallow clone missing this commit).

# Lines of merge-base text above a `type` row to resolve Practice-4 face when the
# immediate authority tail is empty (e.g. sibling LLVM ordering carriers after a
# shared dissolution banner).
PRACTICE4_CONTEXT_LINES = 250

TYPE_RE = re.compile(r"^type\s+([A-Za-z_][A-Za-z0-9_]*)\b")
DATA_RE = re.compile(r"^data\s+([A-Za-z_][A-Za-z0-9_]*)\b")
FN_RE = re.compile(r"^fn\s+([A-Za-z_][A-Za-z0-9_]*)\b")
COPRODUCT_TAG_RE = re.compile(
    r"^\s*//\s*[🟢🟡🔴]\s+coproduct dissolution\b",
)

# Any `…lexeme`-shaped field typed `String` (partial lexical authority / D3200-style).
# Uses a name suffix rule so new `foo_lexeme: String` sites classify as 🟢→🟡 without
# extending a brittle per-field allowlist (composer-2 exploratory #3370).
LEXEME_STRING_FIELD_RE = re.compile(r"\b\w*lexeme\s*:\s*String\b")

# Non-coproduct merge-base receipts (strict de-prose strips their `//` bodies) that
# must still appear on the live `// Ledger:` line for inventory.
EXTRA_PART6_SLUGS_BY_REL: dict[str, frozenset[str]] = {
    "src/v4/extdeps/languages/verilog.dag": frozenset(
        {
            "SL-3229-VERILOG-NONEMPTY",
        }
    ),
    "src/v4/extdeps/languages/ptx.dag": frozenset(
        {
            "SL-3229-PTX-DIM3",
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
    try:
        out = subprocess.check_output(
            ["git", "show", f"{MERGE_BASE}:{rel}"],
            cwd=ROOT,
            text=True,
            stderr=subprocess.DEVNULL,
        )
        return out.splitlines()
    except subprocess.CalledProcessError:
        return []


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


def practice4_tail_for_face(lines: list[str], coproduct_line_idx: int) -> str:
    """Merge-base text used to classify a coproduct: immediate tail, else a short upward window."""
    block = authority_block(lines, coproduct_line_idx)
    if practice4_face(block) is not None:
        return block
    ctx = "\n".join(lines[max(0, coproduct_line_idx - PRACTICE4_CONTEXT_LINES) : coproduct_line_idx])
    if block and ctx:
        return block + "\n" + ctx
    return block or ctx


def verilog_yellow_ref(block: str) -> str:
    """Split Verilog 🟡 coproducts: #3200 first-consumer matrix vs Wave-A2 list-non-empty."""
    key = "Coproduct dissolution (Practice 4 / modeling-discipline.md §4)"
    pos = block.rfind(key)
    if pos < 0:
        pos = block.rfind("Coproduct dissolution (Practice 4")
    tail = block[pos:] if pos >= 0 else block
    ledger_header = tail.split("// Terminal:")[0]
    # `#3200` footers often live after `// Terminal:` in merge-base banners; scan the full
    # authority tail, not only the pre-Terminal slice (merge-base §SL-3229-VERILOG-D3200).
    if (
        "#3200" in block
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
    """Map coproduct `type` name -> (emoji, ledger ref slug) from merge-base authority."""
    lines = git_merge_base_lines(rel)
    out: dict[str, tuple[str, str]] = {}
    for i, _ln in enumerate(lines):
        nm = is_coproduct(lines, i)
        if not nm:
            continue
        block = authority_block(lines, i)
        if rel.endswith("llvm_ir.dag") and nm == "LlvmType" and "RAW-Int WIDTH RESIDUAL" in block:
            # Merge-base still carries the historical RAW-Int banner; live substrate
            # closed SL-3229-LLVM-WIDTH via std/cardinality.dag `NonZeroNat` + `Nat`
            # (PR #3310 P1 cardinality refinement — operator receipt 2026-05-18).
            out[nm] = ("🟢", "CP-3229-GREEN-TERMINAL")
            continue
        tail = practice4_tail_for_face(lines, i)
        face = practice4_face(tail)
        if face == "green":
            out[nm] = ("🟢", "CP-3229-GREEN-TERMINAL")
        elif face == "red":
            out[nm] = ("🔴", "CP-3229-RED-PRACTICE4")
        elif face == "yellow":
            if "verilog" in rel:
                out[nm] = ("🟡", verilog_yellow_ref(tail))
            elif "float" in rel:
                out[nm] = ("🟡", "SL-3229-FLOAT-NOMINAL")
            else:
                out[nm] = ("🟡", "CP-3229-GREEN-TERMINAL")
        else:
            print(
                f"FAIL: cannot classify Practice-4 face for {rel} coproduct {nm!r} "
                f"(merge-base {MERGE_BASE}); extend tail or add explicit mapping.",
                file=sys.stderr,
            )
            sys.exit(1)
    if rel == "src/v4/extdeps/languages/verilog.dag":
        for nm in (
            "ConstantUnaryOperator",
            "ConstantBinaryOperator",
            "ConstantRangeExpression",
            "ConstantSelect",
            "ConstantPrimary",
            "ConstantExpression",
        ):
            out[nm] = ("🟢", "CP-3229-VERILOG-CONSTEXPR-TERMINAL")
    if rel.endswith("llvm_ir.dag"):
        # Wave-1 fact-bundle coproducts (T-4 quiet-otter-381); absent at merge-base.
        for nm, slug in (
            ("LlvmWave1IntegerBits", "SL-3229-LLVM-WAVE1-INT-WIDTH"),
            ("LlvmWave1FloatKind", "SL-3229-LLVM-WAVE1-FLOAT-KIND"),
        ):
            if nm == "LlvmWave1IntegerBits":
                out.setdefault(nm, ("🟡", slug))
            else:
                out.setdefault(nm, ("🟢", slug))
    if rel == "src/v4/extdeps/languages/typescript.dag":
        # T-4 wave-1 catalog row tag (replaces merge-base TsEcma262NumericPrimitiveKind).
        out["TsEcma262NumericPrimitiveFactsUnion"] = ("🟢", "CP-3229-GREEN-TERMINAL")
        # T-11 MVP-1 grammar-relation token carrier (absent at merge-base).
        out["TsConcreteSyntaxToken"] = ("🟢", "CP-3229-GREEN-TERMINAL")
        out["TsGramBuild1State"] = ("🟡", "SL-T11-GRAMMAR-FROM-TOKEN-ROW")
        # T-4 Wave 2b law-kind coproduct (absent at merge-base; scaffold:T-4-wave-2b).
        out["TsModelCoreLawKey"] = ("🟡", "SL-T4-WAVE2B-TS-LAW-KEY")
    if rel == "src/v4/extdeps/languages/rust.dag":
        out["RustGramBuild1State"] = ("🟡", "SL-T11-GRAMMAR-FROM-TOKEN-ROW")
    if rel == "src/v4/extdeps/languages/python.dag":
        out["PythonConcreteSyntaxToken"] = ("🟢", "CP-3229-GREEN-TERMINAL")
        out["PythonGramBuild1State"] = ("🟡", "SL-T11-GRAMMAR-FROM-TOKEN-ROW")
    if rel == "src/v4/extdeps/languages/go.dag":
        out["GoConcreteSyntaxToken"] = ("🟢", "CP-3229-GREEN-TERMINAL")
        out["GoGramBuild1State"] = ("🟡", "SL-T11-GRAMMAR-FROM-TOKEN-ROW")
    if rel == "src/v4/extdeps/languages/cpp.dag":
        out["CppConcreteSyntaxToken"] = ("🟢", "CP-3229-GREEN-TERMINAL")
        out["CppGramBuild1State"] = ("🟡", "SL-T11-GRAMMAR-FROM-TOKEN-ROW")
    if rel == "src/v4/extdeps/languages/swift.dag":
        # New T-4/T-11 language slice (absent at merge-base).
        for nm in (
            "SwiftIntWidth",
            "SwiftIntSignedness",
            "SwiftFloatWidth",
            "SwiftScalar",
            "SwiftNonNumericPrimitiveFacts",
            "SwiftConcreteSyntaxToken",
        ):
            out[nm] = ("🟢", "CP-3229-GREEN-TERMINAL")
        # T-4 Wave 2b law-kind coproduct (absent at merge-base; scaffold:T-4-wave-2b).
        out["SwiftModelCoreLawKey"] = ("🟡", "SL-T4-WAVE2B-SWIFT-LAW-KEY")
    if rel == "src/v4/extdeps/languages/wasm.dag":
        # New T-4/T-11 language slice (absent at merge-base).
        for nm in (
            "WasmIntWidth",
            "WasmFloatWidth",
            "WasmScalar",
            "WasmConcreteSyntaxToken",
        ):
            out[nm] = ("🟢", "CP-3229-GREEN-TERMINAL")
        # T-4 Wave 2b law-kind coproduct (absent at merge-base; scaffold:T-4-wave-2b).
        out["WasmModelCoreLawKey"] = ("🟡", "SL-T4-WAVE2B-WASM-LAW-KEY")
    if rel == "src/v4/std/integer.dag":
        # T-3A shared-fact vocabulary for T-4 language primitive fact-bundles
        # (absent at merge-base).
        out["Signedness"] = ("🟢", "CP-3229-GREEN-TERMINAL")
        out["IntegerBoundSign"] = ("🟢", "CP-3229-GREEN-TERMINAL")
        out["IntegerRangeEndpoint"] = ("🟢", "CP-3229-GREEN-TERMINAL")
        out["DecimalDigit"] = ("🟢", "CP-3229-GREEN-TERMINAL")
        out["NonZeroDecimalDigit"] = ("🟢", "CP-3229-GREEN-TERMINAL")
        out["DecimalMagnitude"] = ("🟢", "CP-3229-GREEN-TERMINAL")
        out["IntegerIntervalBound"] = ("🟢", "CP-3229-GREEN-TERMINAL")
        out["Representation"] = ("🟡", "SL-3229-INTEGER-REPRESENTATION-STUBS")
        out["OverflowDisposition"] = ("🟡", "SL-3229-INTEGER-OVERFLOW-SEMANTICS")
    if rel == "src/v4/std/float.dag":
        # T-3B language-facing disposition carriers (absent at merge-base).
        for nm in ("NaNPropagation", "InfPropagation", "FloatExceptionDisposition"):
            out[nm] = ("🟡", "SL-3229-FLOAT-DISPOSITION")
    return out


def module_body_line_list(path: Path) -> list[str]:
    lines = path.read_text().splitlines()
    for i, ln in enumerate(lines):
        if ln.startswith("module "):
            return lines[i + 1 :]
    raise SystemExit(f"{path}: no `module` line found")


def coproduct_type_names_in_path(path: Path) -> set[str]:
    """Every live N≥2 sum `type` in the substrate body (Practice-4 coproduct set)."""
    bl = module_body_line_list(path)
    out: set[str] = set()
    for i in range(len(bl)):
        nm = is_coproduct(bl, i)
        if nm:
            out.add(nm)
    return out


def required_ledger_slugs(
    rel: str, tag_map: dict[str, tuple[str, str]], live_coproducts: set[str]
) -> set[str]:
    """Ledger slug inventory for the live `// Ledger:` line: current coproduct refs + EXTRA scaffolds."""
    unknown = live_coproducts - tag_map.keys()
    if unknown:
        print(
            f"FAIL: {rel} live sum coproduct(s) missing from merge-base {MERGE_BASE} tag map "
            f"(add Practice-4 authority to merge-base or extend coproduct_tag_from_merge_base): "
            + ", ".join(sorted(unknown)),
            file=sys.stderr,
        )
        sys.exit(1)
    refs = {tag_map[nm][1] for nm in live_coproducts}
    refs |= set(EXTRA_PART6_SLUGS_BY_REL.get(rel, frozenset()))
    return refs


def format_ledger_line(
    rel: str, tag_map: dict[str, tuple[str, str]], live_coproducts: set[str]
) -> str:
    slugs = ", ".join(sorted(required_ledger_slugs(rel, tag_map, live_coproducts)))
    return f"// Ledger: {slugs}.\n"


def format_grounded_r1_slice_marker(marker: str) -> str:
    """RULING-1 slice groundedness line + blank line before `module` (marker has no required trailing newline)."""
    stripped = marker.strip()
    if stripped not in ("// 🟡", "// 🟢"):
        raise SystemExit(f"RULING-1 marker must be '// 🟡' or '// 🟢', got {stripped!r}")
    return stripped + "\n\n"


def format_coproduct_tag(emoji: str, ref: str, type_name: str | None = None) -> str:
    if ref == "SL-T11-GRAMMAR-FROM-TOKEN-ROW":
        return (
            "// 🟡 coproduct dissolution — SL-T11-GRAMMAR-FROM-TOKEN-ROW — "
            "feature:t11-grammar-from-token-row — "
            "bind task: src/v4/TASKS.md#t-11-emit-per-target-specialization — "
            "owner:T-11 target-model grammar rows — "
            "dissolve-on-arrival: replace the fold1 accumulator with a std "
            "fold_non_empty/list-to-sequence helper once that helper lands and "
            "T-11 grammar rows consume it; forbidden: hand-authoring token-class "
            "sequence trees beside the *_mvp1_concrete_tokens source."
        )
    if type_name == "LlvmWave1IntegerBits":
        return (
            "// 🟡 coproduct dissolution — SL-3229-LLVM-WAVE1-INT-WIDTH — "
            "feature:llvm-wave1-int-bits-subset — "
            "dissolve-on-arrival: llvm_integer_facts_catalog aligns with "
            "LlvmType.IntegerType.bits NonZeroNat carrier (wave-2 · T-4 quiet-otter-381)."
        )
    if type_name == "Representation":
        return (
            "// 🟡 coproduct dissolution — SL-3229-INTEGER-REPRESENTATION-STUBS — "
            "feature:integer-representation-stub-semantics — "
            "dissolve-on-arrival: OnesComplement and SignMagnitude gain target-backed "
            "representation semantics or are proven unused by the T-4 fact-bundle catalog."
        )
    if type_name == "OverflowDisposition":
        return (
            "// 🟡 coproduct dissolution — SL-3229-INTEGER-OVERFLOW-SEMANTICS — "
            "feature:integer-overflow-disposition-semantics — "
            "dissolve-on-arrival: wrapping/saturating/trapping/undefined overflow are "
            "backed by width, bounds, and Outcome semantics consumed by T-4 fact-bundles."
        )
    if type_name == "NaNPropagation":
        return (
            "// 🟡 coproduct dissolution — SL-3229-FLOAT-DISPOSITION — "
            "feature:float-nan-propagation-semantics — "
            "dissolve-on-arrival: SignalingNaNEnabled gains a target-backed trap handler "
            "path consumed by a T-4 language fact-bundle."
        )
    if type_name == "InfPropagation":
        return (
            "// 🟡 coproduct dissolution — SL-3229-FLOAT-DISPOSITION — "
            "feature:float-inf-propagation-semantics — "
            "dissolve-on-arrival: SaturatingInf gains a target-backed clamp semantic "
            "consumed by a T-4 fact-bundle (e.g. DSP target)."
        )
    if type_name == "FloatExceptionDisposition":
        return (
            "// 🟡 coproduct dissolution — SL-3229-FLOAT-DISPOSITION — "
            "feature:float-exception-disposition-semantics — "
            "dissolve-on-arrival: TrapEnabled or FastMathUndefined gain target-backed "
            "exception handler paths consumed by T-4 fact-bundles."
        )
    if type_name == "TsModelCoreLawKey":
        return (
            "// 🟡 coproduct dissolution — SL-T4-WAVE2B-TS-LAW-KEY — "
            "feature:model-core-law-expression-carrier — "
            "scaffold:T-4-wave-2b · law-kind coproduct for ts_model_core_law_node kind edge — "
            "owner:T-4 typescript language model lane — "
            "dissolve-on-arrival: std/algebra exports typed law/equation Nodes replacing "
            "kind atoms (same gate as model_core ModelCoreWave1BoolLawKey)."
        )
    if type_name == "WasmModelCoreLawKey":
        return (
            "// 🟡 coproduct dissolution — SL-T4-WAVE2B-WASM-LAW-KEY — "
            "feature:model-core-law-expression-carrier — "
            "scaffold:T-4-wave-2b · law-kind coproduct for wasm_model_core_law_node kind edge — "
            "owner:T-4 wasm language model lane — "
            "dissolve-on-arrival: std/algebra exports typed law/equation Nodes replacing "
            "kind atoms (same gate as model_core ModelCoreWave1BoolLawKey)."
        )
    if type_name == "SwiftModelCoreLawKey":
        return (
            "// 🟡 coproduct dissolution — SL-T4-WAVE2B-SWIFT-LAW-KEY — "
            "feature:model-core-law-expression-carrier — "
            "scaffold:T-4-wave-2b · law-kind coproduct for swift_model_core_law_node kind edge — "
            "owner:T-4 swift language model lane — "
            "dissolve-on-arrival: std/algebra exports typed law/equation Nodes replacing "
            "kind atoms (same gate as model_core ModelCoreWave1BoolLawKey)."
        )
    return f"// {emoji} coproduct dissolution · {ref}."


def grounded_tag_for_record_body(body_lines: list[str]) -> str:
    body = "\n".join(body_lines)
    em = "🟡" if LEXEME_STRING_FIELD_RE.search(body) else "🟢"
    return f"// {em} grounded."


def inject_grounded_tags(bl: list[str]) -> list[str]:
    """Insert RULING-1 `// 🟢|🟡 grounded.` before each braced record `type`.

    Tags are **derived only** from the live record body (`LEXEME_STRING_FIELD_RE` → 🟡).
    Callers must run `strip_body_comments` first so disk-authored grounded lines cannot
    make `--check` pass stale classifications (codex BLOCKING #3370 / INVARIANTS P2).
    """
    out: list[str] = []
    j = 0
    while j < len(bl):
        ln = bl[j]
        st = ln.strip()
        if TYPE_RE.match(st) and st.rstrip().endswith("{"):
            depth = ln.count("{") - ln.count("}")
            chunk = [ln]
            j += 1
            while j < len(bl) and depth > 0:
                chunk.append(bl[j])
                depth += bl[j].count("{") - bl[j].count("}")
                j += 1
            # Blank line before grounded tags when the previous emitted line is
            # non-empty (matches verilog/llvm_ir spacing; fixes float.dag after
            # coproduct variant rows — claude-opus #3370 non-blocking).
            if out and out[-1].strip():
                out.append("")
            out.append(grounded_tag_for_record_body(chunk))
            out.extend(chunk)
            continue
        out.append(ln)
        j += 1
    return out


def inject_coproduct_tags(body: str, rel: str, tag_map: dict[str, tuple[str, str]]) -> str:
    bl = body.splitlines()
    out_lines: list[str] = []
    j = 0
    while j < len(bl):
        nm = is_coproduct(bl, j)
        if nm:
            if nm not in tag_map:
                print(
                    f"FAIL: {rel} sum coproduct {nm!r} missing from merge-base {MERGE_BASE} tag map.",
                    file=sys.stderr,
                )
                sys.exit(1)
            em, ref = tag_map[nm]
            expected = format_coproduct_tag(em, ref, type_name=nm)
            prev = bl[j - 1] if j > 0 else ""
            if COPRODUCT_TAG_RE.match(prev):
                if prev != expected:
                    if not out_lines:
                        raise SystemExit(
                            "internal error: coproduct tag mismatch but out_lines empty "
                            f"({nm!r})"
                        )
                    out_lines[-1] = expected
            else:
                out_lines.append(expected)
            out_lines.append(bl[j])
            j += 1
            continue
        out_lines.append(bl[j])
        j += 1
    trailing = "\n" if body.endswith("\n") or not body else ""
    return "\n".join(out_lines) + trailing



def comment_ratio(text: str) -> tuple[int, int, float]:
    lines = text.splitlines()
    total = len(lines) if lines else 1
    c = sum(1 for ln in lines if ln.lstrip().startswith("//"))
    return c, total, 100.0 * c / total


def strip_body_comments(after_module: str) -> str:
    out_lines: list[str] = []
    for line in after_module.splitlines(True):
        sl = line.rstrip("\r\n")
        # Only coproduct one-liners survive the strip; RULING-1 grounded lines are
        # always re-materialized by inject_grounded_tags (keeps one code path and
        # spacing normalization — e.g. float.dag after coproduct variants).
        if (
            sl.lstrip().startswith("//")
            and not COPRODUCT_TAG_RE.match(sl)
        ):
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
    ends_nl = new_body.endswith("\n")
    core = new_body[:-1] if ends_nl else new_body
    grounded_lines = inject_grounded_tags(core.splitlines())
    new_body = "\n".join(grounded_lines) + ("\n" if ends_nl else "")
    return header + module_line + new_body


def rewrite(path: Path, header: str, rel: str, tag_map: dict[str, tuple[str, str]]) -> None:
    path.write_text(materialize_deprose_text(path, rel, tag_map, header))


def run_check(specs: list[tuple[str, str, str, str, str, str]]) -> None:
    """Exit 0 only if each allowlisted file already matches merge-base-derived output."""
    drift: list[str] = []
    for rel, scope, anchor, consumes, status, grounded_r1 in specs:
        path = ROOT / rel
        tag_map = coproduct_tag_from_merge_base(rel)
        live = coproduct_type_names_in_path(path)
        first = path.read_text().splitlines()[0]
        if not first.startswith("// src/"):
            raise SystemExit(f"{rel}: expected line 1 // src/…, got {first!r}")
        header = (
            f"{first}\n"
            f"{scope}\n"
            f"{consumes}\n"
            f"{status}\n"
            f"{anchor}\n"
            f"{format_ledger_line(rel, tag_map, live)}"
            f"{format_grounded_r1_slice_marker(grounded_r1)}"
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

    # Sixth field: operator RULING-1 slice groundedness (emoji-only; ratified in
    # `docs/modeling-discipline.md` Practice 9; ledger doc retired 2026-05-19).
    # `// Owns:` is no longer generated; the allowlist exception is dissolved.
    # Extdeps language slices 🟡 (Shape A emit/L5/L6 still open per v4-close-interrogation §14); std 🟢.
    specs: list[tuple[str, str, str, str, str, str]] = [
        (
            "src/v4/extdeps/languages/verilog.dag",
            "// Scope: IEEE 1364-2005 Verilog structural carriers.",
            "// Anchor: https://standards.ieee.org/ieee/1364/3641/",
            "// Consumes: std/node.dag; std/nat.dag (Nat).",
            "// Status: import v4.std.node Symbol; v4.std.nat Nat.",
            "// 🟡",
        ),
        (
            "src/v4/extdeps/languages/llvm_ir.dag",
            "// Scope: LLVM 18 LangRef IR structural vocabulary (SSA-oriented types, instructions, terminators, and related carriers).",
            "// Anchor: https://releases.llvm.org/18.1.8/docs/LangRef.html",
            "// Consumes: std/node.dag (Symbol); std/nat.dag (Nat); std/cardinality.dag (NonZeroNat); Int kernel-ambient.",
            "// Status: import v4.std.node Symbol; v4.std.nat Nat; v4.std.cardinality NonZeroNat.",
            "// 🟡",
        ),
        (
            "src/v4/extdeps/languages/ptx.dag",
            "// Scope: NVIDIA PTX ISA 8.5 SIMT structural classifiers (param/shared state, registers, thread hierarchy).",
            "// Anchor: https://docs.nvidia.com/cuda/pdf/ptx_isa_8.5.pdf — TOC https://docs.nvidia.com/cuda/parallel-thread-execution/index.html",
            "// Consumes: std/nat.dag (Nat); std/cardinality.dag (PositiveUpperBoundedNat).",
            "// Status: import v4.std.nat Nat; v4.std.cardinality PositiveUpperBoundedNat.",
            "// 🟡",
        ),
        (
            "src/v4/extdeps/languages/typescript.dag",
            "// Scope: TypeScript 5.9 + ECMA-262 ES2025 numeric+string primitive fact-bundles, ModelCore wave-1 (primitives+inhabitance), wave-2b (laws+effects+partiality+string).",
            "// Anchor: https://www.typescriptlang.org/docs/handbook/2/everyday-types.html — ECMA-262 https://tc39.es/ecma262/2025/multipage/",
            "// Consumes: v4.std.grammar, v4.std.lexing, v4.std.collection, v4.std.node, v4.std.logic, v4.std.algebra, v4.std.model_core, v4.std.target_model, v4.std.text.",
            "// Status: T-4 typescript slice — ECMA `number` (IEEE-754 binary64) + `bigint` (exact unbounded ℤ) + `string` (UTF-16 code unit sequence, §6.1.4) fact-bundles; wave-2b: inhabitance+laws+effects+partiality; `core: ModelCore` primitives from `ts_numeric_facts_catalog` via fold; canonical_symbols = catalog surface spellings + wave-1 lex/grammar/MVP; target_model edge keys from std/target_model.dag; MVP-1 grammar/token substrate for T-11; bool canonical-B decl-ref — 🟡 E-6(b) staging.",
            "// 🟡",
        ),
        (
            "src/v4/extdeps/languages/swift.dag",
            "// Scope: Swift language/standard-library scalar fact-bundles, ModelCore wave-1 (primitives), wave-2b (inhabitance+laws+effects+partiality).",
            "// Anchor: https://docs.swift.org/swift-book/documentation/the-swift-programming-language/thebasics/",
            "// Consumes: v4.std.grammar, v4.std.lexing, v4.std.collection, v4.std.model_core, v4.std.target_model, v4.std.node, v4.std.logic, v4.std.algebra, v4.std.text.",
            "// Status: T-4 Swift wave-1+2b; fixed-width integer spellings explicit, Int/UInt platform-word-width, Float/Double IEEE-754; wave-2b: signed int OrderedRing + unsigned int CommutativeSemiring + float ApproximateField + bool BooleanAlgebra + string FreeMonoid inhabitance; laws+effects+partiality; canonical_symbols = catalog surface spellings + wave-1 lex/grammar/MVP; target_model edge keys from std/target_model.dag; MVP-1 grammar/token substrate for T-11; bool canonical-B decl-ref is E-6(b) staging.",
            "// 🟡",
        ),
        (
            "src/v4/extdeps/languages/wasm.dag",
            "// Scope: WebAssembly Core numeric value types — LanguageModel fact-bundles (Shape A).",
            "// Anchor: https://webassembly.github.io/spec/core/types.html#number-types",
            "// Consumes: v4.std.grammar, v4.std.lexing, v4.std.collection, v4.std.model_core, v4.std.target_model, v4.std.node, v4.std.logic, v4.std.algebra, v4.std.text.",
            "// Status: T-4 wasm slice — Core §2.3.1 number types (i32/i64/f32/f64 wave-1); wave-2b: inhabitance+laws+effects+partiality; `core: ModelCore` (#3474); canonical_symbols = catalog surface spellings + wave-1 lex/grammar/MVP; target_model edge keys from std/target_model.dag; MVP-1 WAT grammar/token substrate for T-11; v128/funcref/externref 🟡 wave-2; integer sign-agnostic per spec (width + modular wrap, not signed/unsigned partition).",
            "// 🟡",
        ),
        (
            "src/v4/std/integer.dag",
            "// Scope: Abstract Int/UInt, fixed-width projections, divide/modulo carriers and diagnostics.",
            "// Anchor: https://en.wikipedia.org/wiki/Integer",
            "// Consumes: std/node.dag (Instantiation, Symbol); std/nat.dag (Nat); std/machine.dag (MachineWidth, PointerWidth, Word8, Word16, Word32, Word64, Word128); std/algebra.dag (Magma, Semigroup, Monoid, Group, AbelianGroup, Ring, OrderedRing, Ordering); std/diagnostic.dag (Diagnostic, Locus, Correction, NoCorrectionReason, Outcome<T> Tier-2 divide/modulo).",
            "// Status: std integer vocabulary (widths, group completion, divide/modulo outcomes).",
            "// 🟢",
        ),
        (
            "src/v4/std/float.dag",
            "// Scope: IEEE-754 Float32/Float64 interchange, specials, body carriers, and ordered compare semantics.",
            "// Anchor: https://en.wikipedia.org/wiki/IEEE_754",
            "// Consumes: std/node.dag (Symbol); std/machine.dag (Bit, Word32, Word64); std/algebra.dag (Ordering, Less, Equal, Greater); std/diagnostic.dag (Diagnostic, Outcome, PortLocus, Produced, Rejected, Unavailable, UserInputBoundary); std/logic.dag (Bool); std/nat.dag (Nat, is_zero, nat_compare).",
            "// Status: std float vocabulary (32/64, specials, IEEE-ordered compare).",
            "// 🟢",
        ),
    ]

    if check_only:
        run_check(specs)
        return

    report: list[tuple[str, float]] = []
    for rel, scope, anchor, consumes, status, grounded_r1 in specs:
        path = ROOT / rel
        tag_map = coproduct_tag_from_merge_base(rel)
        live = coproduct_type_names_in_path(path)
        first = path.read_text().splitlines()[0]
        if not first.startswith("// src/"):
            raise SystemExit(f"{rel}: expected line 1 // src/…, got {first!r}")
        header = (
            f"{first}\n"
            f"{scope}\n"
            f"{consumes}\n"
            f"{status}\n"
            f"{anchor}\n"
            f"{format_ledger_line(rel, tag_map, live)}"
            f"{format_grounded_r1_slice_marker(grounded_r1)}"
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
