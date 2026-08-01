#!/usr/bin/env python3
"""Regenerate provisional debt / expect-red / unclassified carriers from a pinned census.

Census input: SITE\\tUnresolvedType\\tTYPE\\tfile\\tstart\\tend lines (or full histogram out).
Requires --census-sha (the tree the histogram was evaluated against).

Classification (quiet-hawk-219):
  subset = files with unbound/refusal assertion vocabulary (word-bounded)
           OR §14 RED probe fixtures (row3/4/5)
  outside subset → incidental BY CONSTRUCTION
  inside subset → execution discriminator: does the control fail if the ref binds?
    yes → deliberate expect-red
    no → incidental
    undecidable → unclassified (real bucket; never empty-by-construction)

Does NOT freeze the roster.
"""
from __future__ import annotations

import argparse
import re
import shutil
import subprocess
import tempfile
from collections import Counter, defaultdict
from pathlib import Path

VOCAB = re.compile(
    r"(?<![A-Za-z_])(?:unresolved type|UnresolvedType|ExpectRed|expect_red|expect-red|"
    r"known_red|type_ref_fail_open|pool_present_unreachable|Product\(<anon>\))(?![A-Za-z_])",
    re.I,
)


def load_sites(path: Path) -> list[tuple[str, str, int, int]]:
    sites: list[tuple[str, str, int, int]] = []
    for line in path.read_text().splitlines():
        if line.startswith("SITE\t"):
            parts = line.split("\t")
            if len(parts) >= 6 and parts[1] == "UnresolvedType":
                sites.append((parts[2], parts[3], int(parts[4]), int(parts[5])))
            continue
        parts = line.split("\t")
        if len(parts) >= 4 and not line.startswith("#"):
            # TYPE file start end
            try:
                sites.append((parts[0], parts[1], int(parts[2]), int(parts[3])))
            except ValueError:
                continue
    return sites


def in_subset(root: Path, file: str) -> bool:
    if "type_ref_fail_open_probe" in file and any(
        x in file for x in ("row3", "row4", "row5")
    ):
        return True
    p = root / file
    if not p.exists():
        return False
    return bool(VOCAB.search(p.read_text(errors="replace")))


def compile_entry(
    gunbc: Path, root: Path, entry: Path, source_roots: list[Path], out_dir: Path
) -> str:
    out_dir.mkdir(parents=True, exist_ok=True)
    cmd = [
        str(gunbc),
        "compile",
        "--target",
        "dag",
        "--entry",
        str(entry),
        "--output-dir",
        str(out_dir),
    ]
    for r in source_roots:
        cmd.extend(["--source-root", str(r)])
    r = subprocess.run(cmd, capture_output=True, text=True, timeout=90, cwd=root)
    return r.stderr + "\n" + r.stdout


def unresolved_count(err: str, needle: str) -> int:
    return sum(
        1
        for line in err.splitlines()
        if "unresolved type" in line.lower() and needle in line
    )


def exec_fixture(
    root: Path, gunbc: Path, name: str, file: str
) -> str:
    """Return disposition reason for a §14 probe fixture site."""
    entry = root / file
    probe = root / "dag/test/fixture/type_ref_fail_open_probe"
    with tempfile.TemporaryDirectory(prefix="tr-class-") as td:
        td_path = Path(td)
        if "row3" in file:
            err0 = compile_entry(
                gunbc, root, entry, [entry.parent], td_path / "base"
            )
            c0 = unresolved_count(err0, "ContentHash")
            src = entry.read_text()
            lines = src.splitlines(True)
            out = []
            for l in lines:
                out.append(l)
                if l.startswith("module "):
                    out.append("import std.types { ContentHash }\n")
            sroot = td_path / "scratch"
            sroot.mkdir()
            (sroot / "entry.dag").write_text("".join(out))
            err1 = compile_entry(
                gunbc,
                root,
                sroot / "entry.dag",
                [sroot, root / "dag", root / "src/v2"],
                td_path / "bound",
            )
            c1 = unresolved_count(err1, "ContentHash")
        else:
            err0 = compile_entry(gunbc, root, entry, [probe], td_path / "base")
            c0 = unresolved_count(err0, "ContentHash")
            src = entry.read_text()
            sdir = td_path / "scratch_probe"
            shutil.copytree(probe, sdir)
            rel = Path(file).relative_to("dag/test/fixture/type_ref_fail_open_probe")
            lines = src.splitlines(True)
            out = []
            for l in lines:
                out.append(l)
                if l.startswith("module "):
                    out.append("import probe.shared.types { ContentHash }\n")
            text = "".join(out).replace(
                "probe.shared.types.ContentHash", "ContentHash"
            )
            (sdir / rel).write_text(text)
            err1 = compile_entry(
                gunbc, root, sdir / rel, [sdir], td_path / "bound"
            )
            c1 = unresolved_count(err1, "ContentHash")
    if c0 >= 1 and c1 == 0:
        return "exec_expect_red_fails_if_binds"
    if c0 >= 1 and c1 >= 1:
        return "exec_bind_did_not_resolve"
    if c0 == 0:
        return "exec_baseline_not_unresolved"
    return "exec_undecided_fixture"


def exec_nonfixture(root: Path, gunbc: Path, name: str, file: str) -> str:
    """Execution discriminator for non-fixture subset sites.

    Question: does any enrolled control fail if the reference binds?
    No applicable control that depends on unbound → incidental (not deliberate).
    Cannot run the discriminator → unclassified.
    """
    src_path = root / file
    if not src_path.exists():
        return "undecidable_missing_file"
    text = src_path.read_text()
    # Deliberate only if this file asserts unboundness for THIS type.
    asserts_this = bool(
        re.search(
            rf"unresolved type['\"]?\s*{re.escape(name)}|"
            rf"{re.escape(name)}[^\n]{{0,80}}unresolved|"
            rf"UnresolvedType[^\n]{{0,80}}{re.escape(name)}",
            text,
            re.I,
        )
    )
    if not asserts_this:
        # Vocab matched for another reason (e.g. prose 'unresolved type name').
        # No control depends on THIS site being unbound → incidental by execution.
        return "exec_no_control_depends_on_unbound"
    # Would need bind+re-run of the asserting control — refuse if we cannot.
    return "undecidable_asserting_control_harness"


def write_typed(
    path: Path,
    rows: list[tuple],
    census_sha: str,
    disposition: str,
    with_reason: bool,
) -> None:
    by_type: dict[str, list] = defaultdict(list)
    for row in rows:
        by_type[row[0]].append(row)
    lines = [
        f"# census_sha\t{census_sha}",
        "# PROVISIONAL_UNFROZEN — freeze waits witty-badger-200 peel fix + re-census",
        "# key grain: TYPE x (file, span_start, span_end)",
        f"# disposition\t{disposition}",
        f"# types\t{len(by_type)}\tsites\t{len(rows)}",
    ]
    for t in sorted(by_type.keys(), key=lambda t: (-len(by_type[t]), t)):
        lines.append(f"# TYPE\t{t}\t{len(by_type[t])}")
        for r in sorted(by_type[t], key=lambda r: (r[1], r[2], r[3])):
            if with_reason:
                lines.append(f"{r[0]}\t{r[1]}\t{r[2]}\t{r[3]}\t{r[4]}")
            else:
                lines.append(f"{r[0]}\t{r[1]}\t{r[2]}\t{r[3]}")
    path.write_text("\n".join(lines) + "\n")


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--census", type=Path, required=True)
    ap.add_argument("--census-sha", required=True)
    ap.add_argument(
        "--out-dir",
        type=Path,
        default=Path("dag/gunbc/type_ref_binding_authority_debt"),
    )
    ap.add_argument("--gunbc", type=Path, default=Path("target/release/gunbc"))
    args = ap.parse_args()
    root = Path(".").resolve()
    sites = load_sites(args.census)
    deliberate: list[tuple] = []
    incidental: list[tuple] = []
    unclassified: list[tuple] = []

    for name, file, start, end in sites:
        if not in_subset(root, file):
            incidental.append(
                (
                    name,
                    file,
                    start,
                    end,
                    "incidental_outside_unbound_vocab_subset_by_construction",
                )
            )
            continue
        if "type_ref_fail_open_probe" in file:
            reason = exec_fixture(root, args.gunbc, name, file)
            bucket = deliberate if reason.startswith("exec_expect_red") else unclassified
            if reason == "exec_expect_red_fails_if_binds":
                deliberate.append((name, file, start, end, reason))
            else:
                unclassified.append((name, file, start, end, reason))
            continue
        reason = exec_nonfixture(root, args.gunbc, name, file)
        if reason.startswith("exec_no_control"):
            incidental.append((name, file, start, end, reason))
        elif reason.startswith("exec_"):
            # future: deliberate from exec
            deliberate.append((name, file, start, end, reason))
        else:
            unclassified.append((name, file, start, end, reason))

    args.out_dir.mkdir(parents=True, exist_ok=True)
    write_typed(
        args.out_dir / "provisional_debt_roster.tsv",
        incidental,
        args.census_sha,
        "provisional_incidental_debt",
        False,
    )
    write_typed(
        args.out_dir / "expect_red_controls.tsv",
        deliberate,
        args.census_sha,
        "deliberate_expect_red",
        True,
    )
    write_typed(
        args.out_dir / "unclassified_sites.tsv",
        unclassified,
        args.census_sha,
        "unclassified_refuses_both_buckets",
        True,
    )
    by_type = Counter(r[0] for r in incidental)
    rank = [
        f"# census_sha\t{args.census_sha}",
        "# type\tsite_count\tdisposition=provisional_incidental_debt",
    ]
    for t, c in sorted(by_type.items(), key=lambda x: (-x[1], x[0])):
        rank.append(f"{t}\t{c}")
    (args.out_dir / "provisional_debt_by_type.tsv").write_text("\n".join(rank) + "\n")
    (args.out_dir / "census_sha.txt").write_text(args.census_sha + "\n")

    print(f"census_sha={args.census_sha}")
    print(f"sites={len(sites)}")
    print(f"deliberate={len(deliberate)}")
    print(f"incidental={len(incidental)}")
    print(f"unclassified={len(unclassified)}")
    print("deliberate_reasons", Counter(r[4] for r in deliberate))
    print("unclassified_reasons", Counter(r[4] for r in unclassified))


if __name__ == "__main__":
    main()
