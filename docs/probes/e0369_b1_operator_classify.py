#!/usr/bin/env python3
"""Classify B1-bucket E0369 operator-on-carrier sites (partition §18).

Unit of count: distinct (file, line, col, rustc code, diagnostic signature),
deduplicated across the M=11 entry modules — same grain as §11.1.

Classification: repr_fork | missing_trait_impl — decided per site from the
emitted line rustc cites, not from a type-name keyword alone.
"""
from __future__ import annotations

import argparse
import csv
import json
import re
import sys
from collections import Counter
from dataclasses import dataclass
from pathlib import Path

B1_KEYWORDS = (
    "CommutativeSemiring",
    "Magnitude",
    "Measure<",
    "Semiring",
)

MODULES_11 = (
    ("05_emit", "src/v2/compiler/05_emit.dag"),
    ("06_translate", "src/v2/compiler/06_translate.dag"),
    ("04_infer", "src/v2/compiler/04_infer.dag"),
    ("03_ingest", "src/v2/compiler/03_ingest.dag"),
    ("emit_host", "src/v2/compiler/emit_host.dag"),
    ("01_tokenize", "src/v2/compiler/01_tokenize.dag"),
    ("materialization_carriers", "src/v2/compiler/materialization_carriers.dag"),
    ("emit_module", "src/v2/compiler/emit_module.dag"),
    ("03_normalize", "src/v2/compiler/03_normalize.dag"),
    ("program_partition", "src/v2/compiler/program_partition.dag"),
    ("05_eval", "src/v2/compiler/05_eval.dag"),
)

E0369_RE = re.compile(
    r"^error\[E0369\]:\s*(.+)$",
    re.MULTILINE,
)
SPAN_RE = re.compile(
    r"^\s*-->\s+(?P<file>[^:]+):(?P<line>\d+):(?P<col>\d+)",
    re.MULTILINE,
)


@dataclass(frozen=True)
class Site:
    entry: str
    file: str
    line: int
    col: int
    code: str
    signature: str
    message: str
    code_line: str
    operand_types: str


def extract_operand_types(message: str) -> str:
    m = re.search(r"type `([^`]+)`", message)
    if m:
        return m.group(1)
    m = re.search(r"types? `([^`]+)` and `([^`]+)`", message)
    if m:
        return f"{m.group(1)} | {m.group(2)}"
    return ""


def b1_keyword_hit(signature: str, operand_types: str, message: str, file: str) -> bool:
    hay = f"{signature} {operand_types} {message}"
    return any(k in hay for k in B1_KEYWORDS)


def classify_site(site: Site) -> tuple[str, str]:
    code = site.code_line
    ops = site.operand_types
    sig = site.signature

    if "#[derive" in code or code.startswith("#[derive"):
        if "dyn Fn" in ops or "Fn(" in ops:
            return (
                "missing_trait_impl",
                "derive_expansion: PartialEq over function-value field",
            )
        if "im::Vector" in ops or "HashMap" in ops:
            return (
                "missing_trait_impl",
                "derive_expansion: PartialEq over upstream collection field",
            )
        if "Interpreter" in ops:
            return (
                "missing_trait_impl",
                "derive_expansion: PartialEq over interpreter carrier field",
            )
        if "EffectIoEvalBundle" in ops or "EffectIoEvalBundle" in code:
            return (
                "missing_trait_impl",
                "derive_expansion: PartialEq over effect-io bundle field",
            )
        if "CommutativeSemiring" in ops or "Measure<" in ops:
            return (
                "repr_fork",
                "derive_expansion: PartialEq on algebra-carrier record under FaithfulFreeMonoid",
            )

    if "dyn Fn" in ops or re.search(r"Fn\([^)]*\)", ops):
        return (
            "missing_trait_impl",
            "expr_binop: equality/compare on function-value carrier",
        )

    if "im::Vector" in ops or "HashMap" in ops or "Interpreter" in ops:
        return (
            "missing_trait_impl",
            "expr_binop: operator on collection/interpreter carrier without trait impl",
        )

    if any(k in ops or k in sig for k in ("CommutativeSemiring", "Measure<", "Semiring")):
        return (
            "repr_fork",
            "expr_binop: arithmetic/compare on modeled algebra carrier (FaithfulFreeMonoid)",
        )

    if site.file.endswith(("std_nat.rs", "v2_std_nat.rs", "v2_std_integer.rs")):
        if "CommutativeSemiring" in ops or "Magnitude" in ops or "Measure<" in ops:
            return (
                "repr_fork",
                "expr_binop: Nat/Int alias surfaces as algebra carrier under v2 closure",
            )

    return (
        "missing_trait_impl",
        f"unmatched_pattern: manual review required ({ops or sig[:80]})",
    )


def parse_instances_tsv(path: Path, entry: str) -> list[Site]:
  sites: list[Site] = []
  with path.open(encoding="utf-8", newline="") as fh:
    reader = csv.DictReader(fh, delimiter="\t")
    for row in reader:
      signature = row.get("message", "").strip()
      rel = row.get("path", "").strip()
      try:
        line = int(row.get("line", "0"))
        col = int(row.get("col", "0"))
      except ValueError:
        continue
      operand_types = row.get("single", "").strip() or extract_operand_types(signature)
      code_line = row.get("code", "").strip()
      if not b1_keyword_hit(signature, operand_types, signature, rel):
        continue
      sites.append(
        Site(
          entry=entry,
          file=rel,
          line=line,
          col=col,
          code="E0369",
          signature=signature,
          message=signature,
          code_line=code_line,
          operand_types=operand_types,
        )
      )
  return sites


def parse_cargo_log(path: Path, entry: str) -> list[Site]:
    text = path.read_text(encoding="utf-8", errors="replace")
    sites: list[Site] = []
    for m in E0369_RE.finditer(text):
        signature = " ".join(m.group(1).split())
        tail = text[m.end() : m.end() + 400]
        span = SPAN_RE.search(tail)
        if not span:
            continue
        rel = span.group("file")
        if rel.startswith("/"):
            idx = rel.find("/src/")
            rel = rel[idx + 1 :] if idx >= 0 else Path(rel).name
        line = int(span.group("line"))
        col = int(span.group("col"))
        operand_types = extract_operand_types(signature)
        if not b1_keyword_hit(signature, operand_types, signature, rel):
            continue
        code_line = ""
        cm = re.search(rf"^\s*{line}\s*\|\s*(.+)$", tail, re.MULTILINE)
        if cm:
            code_line = cm.group(1).strip()
        sites.append(
            Site(
                entry=entry,
                file=rel,
                line=line,
                col=col,
                code="E0369",
                signature=signature,
                message=signature,
                code_line=code_line,
                operand_types=operand_types,
            )
        )
    return sites


def parse_jsonl(path: Path, entry: str) -> list[Site]:
    sites: list[Site] = []
    for raw in path.read_text(encoding="utf-8", errors="replace").splitlines():
        if not raw.strip():
            continue
        try:
            row = json.loads(raw)
        except json.JSONDecodeError:
            continue
        if row.get("reason") != "compiler-message":
            continue
        msg = row.get("message") or {}
        if msg.get("level") != "error":
            continue
        code_obj = msg.get("code") or {}
        code = code_obj.get("code") if isinstance(code_obj, dict) else None
        if code != "E0369":
            continue
        spans = msg.get("spans") or []
        if not spans:
            continue
        span = spans[0]
        rel = span.get("file_name", "")
        if rel.startswith("/"):
            idx = rel.find("/src/")
            rel = rel[idx + 1 :] if idx >= 0 else Path(rel).name
        line = int(span.get("line_start", 0))
        col = int(span.get("column_start", 0))
        signature = " ".join(msg.get("message", "").split())
        operand_types = extract_operand_types(signature)
        if not b1_keyword_hit(signature, operand_types, signature, rel):
            continue
        sites.append(
            Site(
                entry=entry,
                file=rel,
                line=line,
                col=col,
                code=code,
                signature=signature,
                message=signature,
                code_line="",
                operand_types=operand_types,
            )
        )
    return sites


def dedupe_sites(sites: list[Site]) -> list[Site]:
    seen: dict[tuple, Site] = {}
    for s in sites:
        k = (s.file, s.line, s.col, s.code, s.signature)
        if k not in seen:
            seen[k] = s
    return list(seen.values())


def write_outputs(
    classified: list[tuple[Site, str, str]],
    out_tsv: Path,
    summary_md: Path | None,
    git_sha: str,
) -> None:
    counts = Counter(cls for _, cls, _ in classified)
    reason_counts = Counter((cls, reason) for _, cls, reason in classified)

    out_tsv.parent.mkdir(parents=True, exist_ok=True)
    with out_tsv.open("w", encoding="utf-8", newline="") as fh:
        w = csv.writer(fh, delimiter="\t")
        w.writerow(
            [
                "file",
                "line",
                "col",
                "signature",
                "operand_types",
                "classification",
                "reason",
                "entry_example",
                "code_line",
            ]
        )
        for s, cls, reason in classified:
            w.writerow(
                [
                    s.file,
                    s.line,
                    s.col,
                    s.signature,
                    s.operand_types,
                    cls,
                    reason,
                    s.entry,
                    s.code_line,
                ]
            )

    total = len(classified) or 1
    lines = [
        "# E0369 B1 operator-on-carrier classification",
        "",
        f"| field | value |",
        f"|---|---|",
        f"| git_sha | `{git_sha}` |",
        f"| modules | M=11 (partition §11.14) |",
        f"| distinct sites | **{len(classified)}** |",
        f"| partition §18 target | 191 |",
        "",
        "## By classification",
        "",
        "| classification | sites | share | moved by repr cut alone |",
        "|---|---:|---:|---|",
    ]
    for cls in ("repr_fork", "missing_trait_impl"):
        n = counts.get(cls, 0)
        moved = "yes" if cls == "repr_fork" else "no"
        lines.append(f"| {cls} | {n} | {n / total * 100:.1f}% | {moved} |")

    lines += ["", "## By reason", ""]
    for (cls, reason), n in reason_counts.most_common():
        lines.append(f"- **{cls}** ({n}): {reason}")

    lines += [
        "",
        "## Decision rules (per site)",
        "",
        "1. `#[derive(..., PartialEq, ...)]` on a line rustc cites → inspect operand:",
        "   - `dyn Fn` / `*Interpreter` / `im::Vector` / `HashMap` / `EffectIoEvalBundle` → **missing_trait_impl**",
        "   - `CommutativeSemiring` / `Measure<…>` record → **repr_fork** (carrier should not be emitted as algebra stub)",
        "2. Body `==`/`<`/`>`/`*`/`/` on `dyn Fn` or interpreter/collection carriers → **missing_trait_impl**",
        "3. Body arithmetic/compare on `CommutativeSemiring`/`Measure` carriers → **repr_fork**",
        "4. `std_nat.rs` / `v2_std_nat.rs` / `v2_std_integer.rs` binops → **repr_fork** (Nat/Int alias under FaithfulFreeMonoid)",
        "",
        "Falsifier for repr_fork class: `eager-deer-389` HostNative flip eliminates algebra-carrier",
        "E0369 on `06_translate` (74 sites → 0); missing_trait_impl sites (dyn Fn, Vector derive) are unchanged.",
    ]

    if summary_md:
        summary_md.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--log-dir", type=Path, help="*.cargo.log from curated_cargo_probe_one.sh")
    ap.add_argument("--instances-dir", type=Path, help="July-style shapes/*.instances.tsv directory")
    ap.add_argument("--json-dir", type=Path, help="optional *.jsonl cargo check outputs")
    ap.add_argument("--out-tsv", type=Path, required=True)
    ap.add_argument("--summary-md", type=Path)
    ap.add_argument("--git-sha", default="unknown")
    args = ap.parse_args()

    if not args.log_dir and not args.json_dir and not args.instances_dir:
        ap.error("one of --log-dir, --instances-dir, or --json-dir is required")

    all_sites: list[Site] = []
    if args.instances_dir:
        for entry, _ in MODULES_11:
            tsv = args.instances_dir / f"{entry}.instances.tsv"
            if tsv.is_file():
                all_sites.extend(parse_instances_tsv(tsv, entry))
            else:
                print(f"missing instances: {tsv}", file=sys.stderr)
    if args.log_dir:
        for entry, _ in MODULES_11:
            log = args.log_dir / f"{entry}.cargo.log"
            if log.is_file():
                all_sites.extend(parse_cargo_log(log, entry))
            else:
                print(f"missing log: {log}", file=sys.stderr)
    if args.json_dir:
        for entry, _ in MODULES_11:
            jpath = args.json_dir / f"{entry}.jsonl"
            if jpath.is_file():
                all_sites.extend(parse_jsonl(jpath, entry))

    unique = dedupe_sites(all_sites)
    classified: list[tuple[Site, str, str]] = []
    for s in sorted(unique, key=lambda x: (x.file, x.line, x.col, x.signature)):
        classified.append((s, *classify_site(s)))

    write_outputs(classified, args.out_tsv, args.summary_md, args.git_sha)
    counts = Counter(cls for _, cls, _ in classified)
    print(
        f"sites={len(classified)} repr_fork={counts.get('repr_fork', 0)} "
        f"missing_trait_impl={counts.get('missing_trait_impl', 0)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
