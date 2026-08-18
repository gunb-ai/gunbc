#!/usr/bin/env python3
"""Partition E0308 mismatched-type sites into mechanism roots (partition §11 grain).

Unit of count: distinct (file, line, col, rustc code, expected/found pair),
deduplicated across M=11 entry modules — same grain as §11.1.

Diagnostic signature for E0308 is the expected/found type pair extracted from
rustc span labels or the message tail, never from rendered text heuristics alone.
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

E0308_RE = re.compile(r"^error\[E0308\]:\s*(.+)$", re.MULTILINE)
SPAN_RE = re.compile(
    r"^\s*-->\s+(?P<file>[^:]+):(?P<line>\d+):(?P<col>\d+)",
    re.MULTILINE,
)
PAIR_RE = re.compile(
    r"expected `(?P<expected>[^`]+)`, found `(?P<found>[^`]+)`"
)


@dataclass(frozen=True)
class Site:
    entry: str
    file: str
    line: int
    col: int
    code: str
    expected: str
    found: str
    pair_signature: str
    message: str


def normalize_rel(path: str) -> str:
    if path.startswith("/"):
        idx = path.find("/src/")
        return path[idx + 1 :] if idx >= 0 else Path(path).name
    return path


def extract_pair(message: str, tail: str) -> tuple[str, str] | None:
    for hay in (tail, message):
        m = PAIR_RE.search(hay)
        if m:
            return m.group("expected"), m.group("found")
    return None


def classify_root(expected: str, found: str) -> tuple[str, str]:
    exp, fnd = expected, found
    pair = f"{exp} | {fnd}"

    if exp == "()" and fnd.startswith("Option"):
        return "C", "optional_tail_collapsed_to_unit"
    if exp.startswith("Rc<Correction>") and fnd.startswith("Option"):
        return "C", "correction_optional_collapse"
    if "Fnv1a64" in exp or "Fnv1a64" in fnd or "ContentHash" in pair:
        if "String" in exp or "String" in fnd:
            return "T7", "seed_prelude_hash_name_collision"
    if "CommutativeSemiring" in pair or "Measure<" in pair or "Magnitude" in pair:
        if "{integer}" in fnd or "{integer}" in exp:
            return "B1-repr", "algebra_carrier_vs_integer_literal"
        if "CommutativeSemiring" in pair or "Measure<" in pair:
            return "B1-repr", "algebra_carrier_vs_named_type"
    if re.search(r"\bNat\b|Rc<Nat>|Rc<v2_std_nat::Nat>", pair) and (
        "{integer}" in fnd or "i64" in fnd or "{integer}" in exp
    ):
        return "B3", "modeled_nat_vs_native_integer"
    if exp == "bool" and (fnd == "Bool" or fnd.startswith("True")):
        return "B2", "modeled_bool_enum_vs_native_bool"
    if fnd == "bool" and exp == "Bool":
        return "B2", "modeled_bool_enum_vs_native_bool"
    if exp.startswith("Rc<") and fnd == exp[3:-1]:
        return "R1", "under_wrapped_bare_leaf"
    if fnd.startswith("Rc<") and exp == fnd[3:-1]:
        return "R1", "over_wrapped_rc_leaf"
    if exp.startswith("Rc<") and fnd.startswith("Rc<") and exp != fnd:
        inner_exp, inner_fnd = exp[3:-1], fnd[3:-1]
        if inner_exp == inner_fnd or inner_exp.startswith("Rc<") or inner_fnd.startswith("Rc<"):
            return "R1", "nested_rc_wrap_depth"
    if "Present" in pair or "Absent" in pair and "Option" in pair:
        return "R2", "optional_variant_surface_vs_option"
    if any(
        k in pair
        for k in (
            "OrdSet",
            "PointwisePower",
            "PartialFunction",
            "HashMap<",
            "im::Vector",
        )
    ):
        return "T3", "collection_carrier_record_vs_native_im"
    if ("String" in exp and ("Vector" in fnd or "FreeMonoid" in fnd)) or (
        "String" in fnd and ("Vector" in exp or "FreeMonoid" in exp)
    ):
        return "T2", "text_carrier_string_vs_freemonoid_vec"
    if exp.startswith("(") and fnd.startswith("Rc<"):
        return "T4", "record_emitted_as_tuple"
    if "dyn Fn" in pair or "Rc<dyn Fn" in pair:
        return "R3", "function_value_carrier"
    if "std_occurrence_identity" in pair and "v2_std_node" in pair:
        return "R5", "duplicate_type_authority_across_modules"
    if "v2_std_" in pair and "std_" in pair and exp != fnd:
        if exp.split("::")[0] != fnd.split("::")[0]:
            return "R5", "duplicate_type_authority_across_modules"
    if "Diagnostics" in pair:
        return "RESIDUE-diagnostics", "diagnostics_carrier_tail"
    if "Witness" in pair:
        return "RESIDUE-witness", "witness_parametrization_tail"
    return "RESIDUE", f"unclassified_pair: {pair[:120]}"


def parse_cargo_log(path: Path, entry: str) -> list[Site]:
    text = path.read_text(encoding="utf-8", errors="replace")
    sites: list[Site] = []
    for m in E0308_RE.finditer(text):
        message = " ".join(m.group(1).split())
        tail = text[m.end() : m.end() + 800]
        span = SPAN_RE.search(tail)
        if not span:
            continue
        rel = normalize_rel(span.group("file"))
        line = int(span.group("line"))
        col = int(span.group("col"))
        pair = extract_pair(message, tail)
        if not pair:
            continue
        expected, found = pair
        sites.append(
            Site(
                entry=entry,
                file=rel,
                line=line,
                col=col,
                code="E0308",
                expected=expected,
                found=found,
                pair_signature=f"expected `{expected}`, found `{found}`",
                message=message,
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
        if code != "E0308":
            continue
        spans = msg.get("spans") or []
        if not spans:
            continue
        span = spans[0]
        rel = normalize_rel(span.get("file_name", ""))
        line = int(span.get("line_start", 0))
        col = int(span.get("column_start", 0))
        message = " ".join(msg.get("message", "").split())
        labels = [span.get("label") or ""]
        for child in span.get("children") or []:
            labels.append(child.get("label") or "")
        pair = extract_pair(message, " ".join(labels))
        if not pair:
            continue
        expected, found = pair
        sites.append(
            Site(
                entry=entry,
                file=rel,
                line=line,
                col=col,
                code="E0308",
                expected=expected,
                found=found,
                pair_signature=f"expected `{expected}`, found `{found}`",
                message=message,
            )
        )
    return sites


def dedupe_sites(sites: list[Site]) -> list[Site]:
    seen: dict[tuple, Site] = {}
    for s in sites:
        k = (s.file, s.line, s.col, s.code, s.pair_signature)
        if k not in seen:
            seen[k] = s
    return list(seen.values())


def write_outputs(
    classified: list[tuple[Site, str, str]],
    out_tsv: Path,
    summary_md: Path | None,
    git_sha: str,
    paired_e0308_errors: int,
    module_stats: list[tuple[str, int, int]],
) -> None:
    root_counts = Counter(root for _, root, _ in classified)
    pair_counts = Counter(s.pair_signature for s, _, _ in classified)
    total_sites = len(classified) or 1

    out_tsv.parent.mkdir(parents=True, exist_ok=True)
    with out_tsv.open("w", encoding="utf-8", newline="") as fh:
        w = csv.writer(fh, delimiter="\t")
        w.writerow(
            [
                "file",
                "line",
                "col",
                "expected",
                "found",
                "pair_signature",
                "root",
                "reason",
                "entry_example",
            ]
        )
        for s, root, reason in classified:
            w.writerow(
                [
                    s.file,
                    s.line,
                    s.col,
                    s.expected,
                    s.found,
                    s.pair_signature,
                    root,
                    reason,
                    s.entry,
                ]
            )

    lines = [
        "# E0308 root partition (mechanism grain)",
        "",
        "| field | value |",
        "|---|---|",
        f"| git_sha | `{git_sha}` |",
        "| modules | M=11 (partition §11.14) |",
        f"| distinct E0308 sites | **{len(classified)}** |",
        f"| paired rustc E0308 error blocks (summed) | {paired_e0308_errors} |",
        "",
        "## Per-module E0308 (diagnostic blocks vs distinct sites)",
        "",
        "| module | E0308 blocks | distinct sites | share of module errors |",
        "|---|---:|---:|---:|",
    ]
    for name, blocks, sites in module_stats:
        share = f"{blocks / max(paired_e0308_errors, 1) * 100:.1f}% of corpus blocks"
        lines.append(f"| {name} | {blocks} | {sites} | {share} |")

    lines += [
        "",
        "## Mechanism roots (site grain)",
        "",
        "| root | sites | % of E0308 sites | partition §11 owner |",
        "|---|---:|---:|---|",
    ]
    owner_map = {
        "C": "gentle-dove-833",
        "B1-repr": "eager-deer-389 / §18",
        "B3": "eager-deer-389",
        "B2": "eager-deer-389",
        "T7": "vivid-wren / checkpoint table",
        "T3": "unowned",
        "T2": "unowned",
        "T5": "— (usually E0277)",
        "R1": "bold-lark-722",
        "R2": "unowned",
        "T4": "unowned",
        "R3": "unowned",
        "R5": "unowned",
        "RESIDUE-diagnostics": "closed (July)",
        "RESIDUE-witness": "closed (July)",
        "RESIDUE": "misc",
    }
    for root, n in root_counts.most_common():
        pct = n / total_sites * 100
        owner = owner_map.get(root, "—")
        lines.append(f"| {root} | {n} | {pct:.1f}% | {owner} |")

    lines += [
        "",
        "## Top pair signatures",
        "",
    ]
    for sig, n in pair_counts.most_common(25):
        lines.append(f"- {n}× `{sig}`")

    lines += [
        "",
        "## Decision rules",
        "",
        "1. Signature = rustc expected/found pair from span label or message.",
        "2. Root assignment follows partition §11.3/§11.4 mechanism names.",
        "3. One site may map to one root; pair diversity within a root is expected.",
        "",
        "Repro: `docs/probes/run_e0308_partition.sh`",
    ]

    if summary_md:
        summary_md.write_text("\n".join(lines) + "\n", encoding="utf-8")


def count_e0308_blocks(log_dir: Path) -> tuple[int, list[tuple[str, int, int]]]:
    module_stats: list[tuple[str, int, int]] = []
    total = 0
    for entry, _ in MODULES_11:
        log = log_dir / f"{entry}.cargo.log"
        blocks = 0
        if log.is_file():
            text = log.read_text(encoding="utf-8", errors="replace")
            blocks = len(re.findall(r"^error\[E0308\]:", text, flags=re.MULTILINE))
        total += blocks
        module_stats.append((entry, blocks, 0))
    return total, module_stats


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--log-dir", type=Path, help="*.cargo.log from curated_cargo_probe_one.sh")
    ap.add_argument(
        "--require-all-logs",
        action="store_true",
        help="refuse if any M=11 module log is missing or empty",
    )
    ap.add_argument("--json-dir", type=Path, help="optional *.jsonl cargo check outputs")
    ap.add_argument("--out-tsv", type=Path, required=True)
    ap.add_argument("--summary-md", type=Path)
    ap.add_argument("--git-sha", default="unknown")
    args = ap.parse_args()

    if not args.log_dir and not args.json_dir:
        ap.error("one of --log-dir or --json-dir is required")

    all_sites: list[Site] = []
    if args.log_dir:
        for entry, _ in MODULES_11:
            log = args.log_dir / f"{entry}.cargo.log"
            if log.is_file():
                all_sites.extend(parse_cargo_log(log, entry))
            elif args.require_all_logs:
                print(f"REFUSED: missing log {log}", file=sys.stderr)
                return 2
            if args.require_all_logs and log.is_file() and log.stat().st_size == 0:
                print(f"REFUSED: empty log {log}", file=sys.stderr)
                return 2
    if args.json_dir:
        for entry, _ in MODULES_11:
            jpath = args.json_dir / f"{entry}.jsonl"
            if jpath.is_file():
                all_sites.extend(parse_jsonl(jpath, entry))

    unique = dedupe_sites(all_sites)
    classified: list[tuple[Site, str, str]] = []
    for s in sorted(
        unique, key=lambda x: (x.file, x.line, x.col, x.pair_signature)
    ):
        classified.append((s, *classify_root(s.expected, s.found)))

    paired_e0308 = 0
    module_stats: list[tuple[str, int, int]] = []
    if args.log_dir:
        paired_e0308, module_stats = count_e0308_blocks(args.log_dir)
        per_mod: Counter[str] = Counter()
        for entry, _ in MODULES_11:
            log = args.log_dir / f"{entry}.cargo.log"
            if log.is_file():
                per_mod[entry] = len(dedupe_sites(parse_cargo_log(log, entry)))
        module_stats = [
            (name, blocks, per_mod.get(name, 0))
            for name, blocks, _ in module_stats
        ]

    write_outputs(
        classified,
        args.out_tsv,
        args.summary_md,
        args.git_sha,
        paired_e0308,
        module_stats,
    )

    root_counts = Counter(root for _, root, _ in classified)
    print(
        f"sites={len(classified)} roots={len(root_counts)} "
        f"paired_e0308_blocks={paired_e0308}"
    )
    if args.log_dir and len(classified) == 0 and paired_e0308 == 0:
        print(
            "SUSPECT_ZERO: bare zero with no paired E0308 output from this log dir",
            file=sys.stderr,
        )
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
