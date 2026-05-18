#!/usr/bin/env python3
"""L1.1 discriminant-predicate dissolution checker (Practice 10 / design-dissolution-lens §5–§6).

Scans `.dag` text for two structural findings:

  (a) Direct: `fn ... -> Bool` where some parameter has a sum type declared in the
      same file, the body is a single `match` on that parameter, and every arm's
      RHS is exactly the literal `true` or `false`.

  (b) Fold-laundered: a `fold(` / `cata(` call using `f: fn(...) { ... }` where the
      lambda body is exactly `true` or `false` and every non-`_` parameter name does
      not appear in the body (constant Bool algebra on the fold step).

Default scan roots match the substrate profile in docs/design-dissolution-lens.md §7:
`src/v4/std/` and `src/v4/compiler/`.

Exit status: 0 when clean, 1 when any finding is reported (stderr). Intended to be
CI-gate-able once the live substrate is clean; until then, rely on
`scripts/test_l1_1_discriminant_predicate.py` in CI.
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

DEFAULT_SCAN_ROOTS = (
    ROOT / "src" / "v4" / "std",
    ROOT / "src" / "v4" / "compiler",
)


@dataclass(frozen=True)
class Finding:
    rel: str
    fn_name: str
    kind: str  # "direct" | "fold-laundered"
    detail: str


def strip_line_comments(text: str) -> str:
    out_lines: list[str] = []
    for line in text.splitlines():
        if "//" in line:
            line = line.split("//", 1)[0]
        out_lines.append(line.rstrip())
    return "\n".join(out_lines)


def skip_ws(s: str, i: int) -> int:
    while i < len(s) and s[i] in " \t\n\r":
        i += 1
    return i


def balanced_paren_content(s: str, open_idx: int) -> tuple[str, int]:
    """s[open_idx] == '('; returns (inside without outer parens, index after ')')."""
    assert s[open_idx] == "("
    depth = 0
    i = open_idx
    start_inside = open_idx + 1
    while i < len(s):
        c = s[i]
        if c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                return s[start_inside:i], i + 1
        i += 1
    raise ValueError("unbalanced '(' in signature")


def balanced_brace_block(s: str, open_idx: int) -> tuple[str, int]:
    """s[open_idx] == '{'; returns (inner without outer braces, index after '}')."""
    assert s[open_idx] == "{"
    depth = 0
    i = open_idx
    start_inside = open_idx + 1
    while i < len(s):
        c = s[i]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return s[start_inside:i], i + 1
        i += 1
    raise ValueError("unbalanced '{' in body")


def type_base_name(type_str: str) -> str:
    t = type_str.strip()
    if "<" in t:
        t = t.split("<", 1)[0].strip()
    return t


def coproduct_names_in_module(text: str) -> set[str]:
    """Names of N≥2 sum `type` carriers declared at module top-level (same heuristics as strict_deprose)."""
    lines = text.splitlines()
    names: set[str] = set()
    i = 0
    while i < len(lines):
        raw = lines[i]
        if not raw.startswith("type "):
            i += 1
            continue
        head = raw.strip()
        m = re.match(r"^type\s+([A-Za-z_][A-Za-z0-9_]*)\b", head)
        if not m:
            i += 1
            continue
        nm = m.group(1)
        j = i
        while j < len(lines):
            t = lines[j].strip()
            if j > i and t.startswith("type "):
                break
            if j > i and t.startswith("fn "):
                break
            if j > i and t.startswith("data "):
                break
            if j > i and t.startswith("import "):
                break
            if j > i and t.startswith("module "):
                break
            if "|" in lines[j] and "=" in lines[j]:
                names.add(nm)
                break
            j += 1
            if j < len(lines):
                s2 = lines[j].strip()
                if s2.startswith("|"):
                    names.add(nm)
                    break
        i = j + 1
    return names


def split_top_level_commas(s: str) -> list[str]:
    parts: list[str] = []
    depth = 0
    start = 0
    for idx, c in enumerate(s):
        if c in "({[":
            depth += 1
        elif c in ")}]":
            depth -= 1
        elif c == "," and depth == 0:
            parts.append(s[start:idx].strip())
            start = idx + 1
    tail = s[start:].strip()
    if tail:
        parts.append(tail)
    return parts


def parse_fn_params(param_inside: str) -> list[tuple[str, str]]:
    if not param_inside.strip():
        return []
    params: list[tuple[str, str]] = []
    for part in split_top_level_commas(param_inside):
        m = re.match(r"^([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(.+)$", part.strip())
        if not m:
            continue
        params.append((m.group(1), m.group(2).strip()))
    return params


def parse_return_type(s: str, i: int) -> tuple[str, int] | None:
    i = skip_ws(s, i)
    if not s.startswith("->", i):
        return None
    i += 2
    i = skip_ws(s, i)
    start = i
    depth_angle = 0
    while i < len(s):
        c = s[i]
        if c == "{" and depth_angle == 0:
            return s[start:i].strip(), i
        if c == "<":
            depth_angle += 1
        elif c == ">" and depth_angle > 0:
            depth_angle -= 1
        i += 1
    return None


def iter_fn_blocks(s: str) -> list[tuple[str, list[tuple[str, str]], str, str, int, int]]:
    """Each: name, params, return_type, body_inner, start_idx, end_idx (end after body })."""
    out: list[tuple[str, list[tuple[str, str]], str, str, int, int]] = []
    idx = 0
    while idx < len(s):
        j = s.find("fn ", idx)
        if j < 0:
            break
        if j > 0 and s[j - 1] not in "\n\r":
            idx = j + 3
            continue
        k = j + 3
        k = skip_ws(s, k)
        name_start = k
        while k < len(s) and (s[k].isalnum() or s[k] == "_"):
            k += 1
        name = s[name_start:k]
        k = skip_ws(s, k)
        if k < len(s) and s[k] == "<":
            depth = 1
            k += 1
            while k < len(s) and depth:
                if s[k] == "<":
                    depth += 1
                elif s[k] == ">":
                    depth -= 1
                k += 1
        k = skip_ws(s, k)
        if k >= len(s) or s[k] != "(":
            idx = j + 3
            continue
        try:
            param_inside, k2 = balanced_paren_content(s, k)
        except ValueError:
            idx = j + 3
            continue
        rt = parse_return_type(s, k2)
        if rt is None:
            idx = j + 3
            continue
        ret_ty, k3 = rt
        k3 = skip_ws(s, k3)
        if k3 >= len(s) or s[k3] != "{":
            idx = j + 3
            continue
        try:
            body_inner, k4 = balanced_brace_block(s, k3)
        except ValueError:
            idx = j + 3
            continue
        params = parse_fn_params(param_inside)
        out.append((name, params, ret_ty, body_inner, j, k4))
        idx = k4
    return out


def scan_match_arms_rhs(inner: str) -> list[str] | None:
    """Return each match arm's RHS text; None if the block does not parse as line-broken arms."""
    i = skip_ws(inner, 0)
    rhs_list: list[str] = []
    while i < len(inner):
        i = skip_ws(inner, i)
        if i >= len(inner):
            break
        depth = 0
        while i < len(inner):
            if inner.startswith("=>", i) and depth == 0:
                break
            c = inner[i]
            if c in "({[":
                depth += 1
            elif c in ")}]":
                depth -= 1
            i += 1
        if i >= len(inner) or not inner.startswith("=>", i):
            return None
        i += 2
        expr_start = skip_ws(inner, i)
        ed = 0
        i = expr_start
        while i < len(inner):
            c = inner[i]
            if c in "({[":
                ed += 1
            elif c in ")}]":
                ed -= 1
                if ed < 0:
                    return None
            if ed == 0 and c == "\n":
                rhs = inner[expr_start:i].strip()
                rhs_list.append(rhs)
                i += 1
                nxt = skip_ws(inner, i)
                if nxt < len(inner) and inner[nxt] == "}":
                    return rhs_list
                i = nxt
                break
            if ed == 0 and c == "}":
                rhs_list.append(inner[expr_start:i].strip())
                return rhs_list
            i += 1
        else:
            return None
    return rhs_list


def is_literal_bool_atom(expr: str) -> bool:
    e = expr.strip()
    return e == "true" or e == "false"


def direct_l1_1_finding(
    _fn_name: str,
    params: list[tuple[str, str]],
    ret_ty: str,
    body_inner: str,
    coproducts: set[str],
) -> str | None:
    if ret_ty.strip() != "Bool":
        return None
    m = re.match(r"^\s*match\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{", body_inner)
    if not m:
        return None
    scrutinee = m.group(1)
    rest = body_inner[m.start() :]
    open_brace = rest.find("{")
    if open_brace < 0:
        return None
    try:
        inner, after = balanced_brace_block(rest, open_brace)
    except ValueError:
        return None
    tail = rest[after:].strip()
    if tail:
        return None
    coproduct_param = False
    for pname, pty in params:
        if pname != scrutinee:
            continue
        if type_base_name(pty) in coproducts:
            coproduct_param = True
            break
    if not coproduct_param:
        return None
    arms = scan_match_arms_rhs(inner)
    if arms is None or not arms:
        return None
    if not all(is_literal_bool_atom(a) for a in arms):
        return None
    return "Bool `fn` matches a same-file coproduct parameter with all-`true`/`false` arms"


def fold_laundered_in_body(body_inner: str) -> str | None:
    """Return detail string if any fold/cata in body uses constant Bool algebra."""

    def scan_calls(keyword: str, src: str) -> str | None:
        pos = 0
        while True:
            hit = src.find(keyword + "(", pos)
            if hit < 0:
                return None
            if hit > 0 and (src[hit - 1].isalnum() or src[hit - 1] == "_"):
                pos = hit + len(keyword) + 1
                continue
            open_paren = hit + len(keyword)
            if open_paren >= len(src) or src[open_paren] != "(":
                pos = hit + 1
                continue
            try:
                inside, after = balanced_paren_content(src, open_paren)
            except ValueError:
                pos = hit + len(keyword) + 1
                continue
            args = split_top_level_commas(inside)
            found: str | None = None
            for arg in args:
                a = arg.strip()
                if not a.startswith("f:"):
                    continue
                rest = a[skip_ws(a, len("f:")) :]
                if not rest.startswith("fn"):
                    continue
                brace_pos = rest.find("{")
                if brace_pos < 0:
                    continue
                if not re.match(r"^fn\s*\([^)]*\)\s*\{", rest[: brace_pos + 1]):
                    continue
                param_m = re.match(r"^fn\s*\(([^)]*)\)\s*\{", rest)
                if not param_m:
                    continue
                param_blob = param_m.group(1)
                try:
                    lam_inner, _ = balanced_brace_block(rest, brace_pos)
                except ValueError:
                    continue
                lam_body = lam_inner.strip()
                if not is_literal_bool_atom(lam_body):
                    continue
                names = [p.strip() for p in param_blob.split(",") if p.strip()]
                usable: list[str] = []
                for n in names:
                    if n == "_":
                        continue
                    if re.fullmatch(r"_\w+", n):
                        continue
                    usable.append(n)
                if any(re.search(rf"\b{re.escape(n)}\b", lam_body) for n in usable):
                    continue
                found = (
                    f"`{keyword}(…, f: fn … {{ {lam_body} }})` constant Bool step "
                    "(fold-laundered discriminant shape)"
                )
                break
            if found:
                return found
            pos = max(after, pos + 1)

    for kw in ("fold", "cata"):
        d = scan_calls(kw, body_inner)
        if d:
            return d
    return None


def findings_in_text(rel: str, text: str) -> list[Finding]:
    clean = strip_line_comments(text)
    try:
        mod_idx = next(i for i, ln in enumerate(clean.splitlines()) if ln.startswith("module "))
    except StopIteration:
        return []
    body = "\n".join(clean.splitlines()[mod_idx + 1 :])
    cops = coproduct_names_in_module(body)
    out: list[Finding] = []
    for name, params, ret_ty, body_inner, _s, _e in iter_fn_blocks(body):
        d = direct_l1_1_finding(name, params, ret_ty, body_inner, cops)
        if d:
            out.append(Finding(rel, name, "direct", d))
        fl = fold_laundered_in_body(body_inner)
        if fl:
            out.append(Finding(rel, name, "fold-laundered", fl))
    return out


def collect_dag_files(roots: list[Path]) -> list[Path]:
    files: list[Path] = []
    for r in roots:
        if r.is_file() and r.suffix == ".dag":
            files.append(r)
        elif r.is_dir():
            files.extend(sorted(p for p in r.rglob("*.dag") if p.is_file()))
    return sorted(set(files))


def scan_files(paths: list[Path]) -> list[Finding]:
    all_findings: list[Finding] = []
    for p in paths:
        rel = str(p.relative_to(ROOT)) if p.is_relative_to(ROOT) else str(p)
        text = p.read_text(encoding="utf-8")
        all_findings.extend(findings_in_text(rel, text))
    return all_findings


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "paths",
        nargs="*",
        type=Path,
        help="Optional `.dag` files or directories to scan (default: substrate roots).",
    )
    args = parser.parse_args(argv)
    if args.paths:
        scan_paths = [ROOT / p if not p.is_absolute() else p for p in args.paths]
    else:
        scan_paths = list(DEFAULT_SCAN_ROOTS)
    files = collect_dag_files(scan_paths)
    findings = scan_files(files)
    for f in findings:
        print(f"{f.rel}: fn {f.fn_name}: L1.1 {f.kind}: {f.detail}", file=sys.stderr)
    return 1 if findings else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
