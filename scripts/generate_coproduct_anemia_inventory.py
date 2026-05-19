#!/usr/bin/env python3
"""Regenerate docs/audit/coproduct-anemia-inventory.md (operator census; deterministic heuristics)."""

from __future__ import annotations

import re
from pathlib import Path

OUT = Path("docs/audit/coproduct-anemia-inventory.md")


def corpus_paths() -> list[Path]:
    paths: list[Path] = []
    paths += sorted(Path("src/v4/extdeps").glob("*.dag"))
    for sub in ("languages", "formats"):
        paths += sorted((Path("src/v4/extdeps") / sub).rglob("*.dag"))
    for sub in ("std", "compiler", "lens"):
        paths += sorted((Path("src/v4") / sub).rglob("*.dag"))
    return paths


def file_anchor_header(text: str) -> bool:
    for line in text.splitlines()[:30]:
        if line.strip().startswith("// Anchor:"):
            return True
    return False


def preceding_comment_block(lines: list[str], idx: int) -> list[str]:
    j = idx - 1
    block: list[str] = []
    while j >= 0:
        t = lines[j].strip()
        if t == "":
            j -= 1
            continue
        if t.startswith("//"):
            block.append(t)
            j -= 1
            continue
        break
    return list(reversed(block))


def filing_class(comment_lines: list[str]) -> str:
    blob = " ".join(comment_lines)
    if "🟡" in blob:
        return "🟡-tracked-bind"
    if "🟢" in blob:
        return "🟢-substantiated"
    return "🔴-in-PR"


def arm_shape(arm: str) -> str:
    return "payload" if "{" in arm else "label"


def parse_sums(path: Path) -> list[tuple[int, str, str, list[str]]]:
    """(1-based line, name, shape label-only|coproduct, preceding // block)."""
    lines = path.read_text(encoding="utf-8").splitlines()
    n = len(lines)
    out: list[tuple[int, str, str, list[str]]] = []
    i = 0
    while i < n:
        line = lines[i].rstrip("\n")
        if line.strip().startswith("//"):
            i += 1
            continue
        m1 = re.match(r"^type\s+(\w+)(<[^>]+>)?\s*=\s*(.+)$", line)
        if m1:
            name, rhs = m1.group(1), m1.group(3).strip()
            if "|" in rhs and not rhs.startswith("{"):
                arms = re.split(r"\s*\|\s*", rhs)
                shapes = [arm_shape(a) for a in arms]
                kind = "label-only" if arms and all(s == "label" for s in shapes) else "coproduct"
                out.append((i + 1, name, kind, preceding_comment_block(lines, i)))
            i += 1
            continue
        m2 = re.match(r"^type\s+(\w+)(<[^>]+>)?\s*$", line)
        if not m2:
            i += 1
            continue
        name = m2.group(1)
        j = i + 1
        while j < n and (not lines[j].strip() or lines[j].strip().startswith("//")):
            j += 1
        if j >= n:
            i += 1
            continue
        nxt = lines[j].strip()
        if nxt.startswith("{"):
            i += 1
            continue
        if not nxt.startswith("="):
            i += 1
            continue
        parts = [nxt[1:].strip()]
        j += 1
        while j < n:
            L = lines[j]
            t = L.strip()
            if t.startswith("//"):
                j += 1
                continue
            if t == "":
                j += 1
                continue
            if re.match(r"^type\s+\w+", L) and not re.match(r"^\s+\|", L):
                break
            if re.match(r"^(data|import|module)\s", L):
                break
            if re.match(r"^fn\s", L):
                break
            if t.startswith("|") or t.startswith("="):
                parts.append(t.lstrip("|").strip())
            j += 1
        arms: list[str] = []
        for chunk in parts:
            for seg in re.split(r"\s*\|\s*", chunk):
                seg = seg.strip()
                if seg:
                    arms.append(seg)
        if not arms:
            i = j
            continue
        rhs_has_bar = any("|" in c for c in parts) or len(arms) > 1
        if not rhs_has_bar:
            i = j
            continue
        shapes = [arm_shape(a) for a in arms]
        kind = "label-only" if all(s == "label" for s in shapes) else "coproduct"
        out.append((i + 1, name, kind, preceding_comment_block(lines, i)))
        i = j
    return out


def extract_data_carriers(text: str) -> list[tuple[str, str]]:
    out: list[tuple[str, str]] = []
    for m in re.finditer(r"^data\s+(\w+)\s*:\s*([^=]+)=\s*", text, re.M):
        out.append((m.group(1), m.group(2).strip()))
    return out


def type_in_carrier(typename: str, carrier: str) -> bool:
    return re.search(r"\b" + re.escape(typename) + r"\b", carrier) is not None


def has_behavioral_fn(text_lines: list[str], typename: str, decl_line_idx: int) -> bool:
    """True if some top-level `fn` lists `typename` as a *parameter* carrier and the tail uses `match `.

    Uses a tiny balanced-paren scan so `fn foo<T>(xs: FreeMonoid<T>, ...)` does not get confused by
    nested `)` inside `fn(A, T) -> ...` parameter types.
    """
    tail = "\n".join(text_lines[decl_line_idx:])
    if "match " not in tail:
        return False
    esc = re.escape(typename)
    param_pat = re.compile(r":\s*[^,)]*\b" + esc + r"\b|,\s*\w+\s*:\s*[^,)]*\b" + esc + r"\b")
    for m in re.finditer(r"(?m)^fn\s+\w+", tail):
        start = m.start()
        sub = tail[start:]
        i = m.end() - m.start()  # offset inside sub
        while i < len(sub) and sub[i] in " \t":
            i += 1
        if i < len(sub) and sub[i] == "<":
            depth = 1
            i += 1
            while i < len(sub) and depth:
                if sub[i] == "<":
                    depth += 1
                elif sub[i] == ">":
                    depth -= 1
                i += 1
        while i < len(sub) and sub[i] in " \t":
            i += 1
        if i >= len(sub) or sub[i] != "(":
            continue
        depth = 0
        j = i
        while j < len(sub):
            ch = sub[j]
            if ch == "(":
                depth += 1
            elif ch == ")":
                depth -= 1
                if depth == 0:
                    params = sub[i : j + 1]
                    if param_pat.search(params):
                        return True
                    break
            j += 1
    return False


def classify(
    path: str,
    name: str,
    text: str,
    lines: list[str],
    decl_idx: int,
    comments: list[str],
    file_carriers: dict[str, list[tuple[str, str]]],
) -> tuple[str, str, str, str, str]:
    """Returns (c4, verdict, col5, col6, c7)."""
    blob = " ".join(comments)
    filing = filing_class(comments)
    anchor_file = file_anchor_header(text)

    data_hits = []
    for dname, carrier in file_carriers[path]:
        if type_in_carrier(name, carrier):
            data_hits.append((dname, carrier))

    if data_hits:
        dname, carrier = data_hits[0]
        col5 = "GROUNDED — substrate `data` binds a named witness whose carrier mentions this sum."
        col6 = f"`data {dname}: {carrier}`"
        return ("green-machine-readable-edge", "GROUNDED", col5, col6, filing)

    if has_behavioral_fn(lines, name, decl_idx):
        col5 = "GROUNDED — a top-level `fn` lists this carrier as a parameter and the file uses `match ` (balanced-paren heuristic)."
        col6 = "See following `fn`/`match` in this module: variants are decomposed into behavioral primitives downstream of the declaration."
        return ("green-machine-readable-edge", "GROUNDED", col5, col6, filing)

    if "🟡" in blob:
        return (
            "yellow-tracked-scaffold",
            "ANEMIC",
            "ANEMIC — 🟡 scaffold header without a machine-readable witness on this row (heuristic).",
            "Close the named bind / gate in the 🟡 header, then land paired `data` or substrate `fn` laws per variant.",
            "🟡-tracked-bind",
        )

    if (
        anchor_file
        or "Anchor:" in blob
        or "http" in blob
        or "DECISIONS.md" in blob
        or "CP-" in blob
        or "SL-" in blob
        or "T-" in blob
    ):
        return (
            "spec-anchor-comment-only",
            "ANEMIC",
            "ANEMIC — prose anchor / ledger / 🟢 filing without `data` or parameter-taking `fn` witness on this carrier (per ruling frame).",
            "Add a `data` morphism/witness or a `fn` that takes this sum as a parameter and `match`es each arm to stated primitives.",
            filing,
        )

    return (
        "NONE-bare",
        "ANEMIC",
        "ANEMIC — no anchor text class hit and no substrate `data` / qualifying `fn` witness (heuristic).",
        "Add `data …: Algebra<ThisSum>` (or equivalent) or explicit `fn` decomposition with `match` on this carrier.",
        filing,
    )


def esc_cell(s: str) -> str:
    return s.replace("|", "\\|").replace("\n", " ")


def main() -> None:
    file_carriers: dict[str, list[tuple[str, str]]] = {}
    text_by_path: dict[str, str] = {}
    for path in corpus_paths():
        t = path.read_text(encoding="utf-8")
        p = str(path).replace("\\", "/")
        text_by_path[p] = t
        file_carriers[p] = extract_data_carriers(t)

    rows: list[dict[str, str]] = []
    for path in corpus_paths():
        p = str(path).replace("\\", "/")
        text = text_by_path[p]
        lines = text.splitlines()
        for line_no, name, shape, comments in parse_sums(path):
            decl_idx = line_no - 1
            c4, verdict, col5, col6, c7 = classify(p, name, text, lines, decl_idx, comments, file_carriers)
            rows.append(
                {
                    "path": p,
                    "line": str(line_no),
                    "name": name,
                    "shape": shape,
                    "c4": c4,
                    "verdict": verdict,
                    "col5": col5,
                    "col6": col6,
                    "c7": c7,
                }
            )

    anemic = [r for r in rows if r["verdict"] == "ANEMIC"]
    grounded = [r for r in rows if r["verdict"] == "GROUNDED"]

    def worst_key(r: dict[str, str]) -> tuple[int, int, int, str, str]:
        c4 = {"NONE-bare": 0, "spec-anchor-comment-only": 1, "yellow-tracked-scaffold": 2, "green-machine-readable-edge": 3}[
            r["c4"]
        ]
        shape = 0 if r["shape"] == "label-only" else 1
        c7 = {"🔴-in-PR": 0, "🟡-tracked-bind": 1, "🟢-substantiated": 2}[r["c7"]]
        return (c4, shape, c7, r["path"], r["name"])

    worst10 = sorted(anemic, key=worst_key)[:10]

    lines_out: list[str] = []
    lines_out.append("# Coproduct anemia inventory — v4 corpus (extdeps + std + compiler + lens)")
    lines_out.append("")
    lines_out.append("> **Ruling frame (operator VERBATIM; do not re-litigate):** coproduct/label *shape* is valid; the violation is *anemic modeling* — a label meaning nothing beyond its name unless grounded by a machine-readable grounding edge or behavioral-primitive decomposition. Bare label + spec-anchor comment + 🟢/🟡 tag alone is **ANEMIC** (filing posture ≠ semantic grounding).")
    lines_out.append("")
    lines_out.append("**Corpus:** every `type … = … | …` sum in `src/v4/extdeps/languages/*`, `src/v4/extdeps/formats/*`, `src/v4/extdeps/*.dag`, plus every sum in `src/v4/std/**/*.dag`, `src/v4/compiler/**/*.dag`, `src/v4/lens/**/*.dag`.")
    lines_out.append("")
    lines_out.append(
        "**Method:** mechanical parse of `.dag` sum declarations; column 4–7 use deterministic substrate heuristics: "
        "(a) `data` RHS carrier mentions the sum name; else (b) a top-level `fn` lists the sum as a *parameter* type "
        "(balanced-paren scan, including `fn name<T>(…)`) and the remainder of the file uses `match `. "
        "Reviewer spot-check: open the cited file and confirm the `data` / `fn` binding by eye."
    )
    lines_out.append("")
    lines_out.append("## Summary")
    lines_out.append("")
    lines_out.append(f"- **Row count (corpus sums):** {len(rows)}")
    lines_out.append(f"- **GROUNDED:** {len(grounded)}")
    lines_out.append(f"- **ANEMIC:** {len(anemic)}")
    lines_out.append("- **Ten worst (anemic-worst-first heuristic: col4 NONE > spec > yellow; then label-only; then 🔴 filing > 🟡 > 🟢):**")
    for i, r in enumerate(worst10, 1):
        lines_out.append(
            f"  {i}. `{r['path']}` **`{r['name']}`** — {r['c4']} / {r['shape']} / {r['c7']}"
        )
    lines_out.append("")
    lines_out.append("## Operator exemplars (verbatim)")
    lines_out.append("")
    lines_out.append(
        "ANEMIC = src/v4/extdeps/languages/rust.dag OverflowAction (PanicOnOverflow | TwoComplementWrap) + OverflowDisposition — bare labels, no grounding edge, code cannot know what either arm DOES."
    )
    lines_out.append("")
    lines_out.append(
        "GROUNDED = same file, data rust_bool_grounding BooleanAlgebra of Bool = bool_boolean_algebra."
    )
    lines_out.append("")
    lines_out.append("## Inventory (worst-first within ANEMIC, then GROUNDED alphabetically by file)")
    lines_out.append("")
    lines_out.append(
        "| file | type/concept | shape (coproduct vs label-only) | grounding class | ANEMIC vs GROUNDED + reason | what GROUNDED requires here | class (filing) |"
    )
    lines_out.append("| --- | --- | --- | --- | --- | --- | --- |")

    sorted_anemic = sorted(anemic, key=worst_key)
    sorted_grounded = sorted(grounded, key=lambda r: (r["path"], r["name"]))

    for r in sorted_anemic + sorted_grounded:
        lines_out.append(
            "| "
            + " | ".join(
                esc_cell(x)
                for x in (
                    f"`{r['path']}:{r['line']}`",
                    f"`{r['name']}`",
                    r["shape"],
                    r["c4"],
                    r["col5"],
                    r["col6"],
                    r["c7"],
                )
            )
            + " |"
        )

    OUT.write_text("\n".join(lines_out) + "\n", encoding="utf-8")
    print(f"Wrote {OUT} ({len(rows)} rows)")


if __name__ == "__main__":
    main()
