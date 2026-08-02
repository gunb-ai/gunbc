#!/usr/bin/env python3
"""OFFLINE AUTHORING TOOL — HALF 2 host-effect + labeled→buckets + TSV serialize.

NEVER invoked from CI, claim_executor, compile-clean, or any enrolled floor
consumer. Checked artifacts are the OUTPUT rows (SHA-pinned TSVs +
type_ref_census_sha in authority.dag).

SINGLE AUTHORITY for census→bucket partition (review 47277): this script.
A prior unused roster_pure_regen.dag fold was deleted — it claimed HALF 1
while this file still partitioned. Dissolve-on:
  type_ref_roster_pure_regen_dissolve_trigger
(enrolled modeled fold becomes sole partition; this arm deletes or calls it).

HALF 2 HOST-EFFECT — deliberate/incidental execution discriminator
(compile as-is vs bind-patched scratch). Documented authoring-time measurement
PROCEDURE (type_ref_roster_host_effect_classify_procedure_note).

Authority: gunbc.type_ref_binding_authority_debt
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


def exec_fixture(root: Path, gunbc: Path, name: str, file: str) -> str:
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
    src_path = root / file
    if not src_path.exists():
        return "undecidable_missing_file"
    text = src_path.read_text()
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
        return "exec_no_control_depends_on_unbound"
    return "undecidable_asserting_control_harness"


def classify_site(
    root: Path, gunbc: Path, name: str, file: str, start: int, end: int
) -> tuple[str, tuple]:
    """HALF 2: return (kind, row) where kind is incidental|expect_red|unclassified."""
    if not in_subset(root, file):
        return ("incidental", (name, file, start, end))
    if "type_ref_fail_open_probe" in file:
        reason = exec_fixture(root, gunbc, name, file)
        if reason == "exec_expect_red_fails_if_binds":
            return ("expect_red", (name, file, start, end, reason))
        return ("unclassified", (name, file, start, end, reason))
    reason = exec_nonfixture(root, gunbc, name, file)
    if reason.startswith("exec_no_control"):
        return ("incidental", (name, file, start, end))
    if reason.startswith("exec_"):
        return ("expect_red", (name, file, start, end, reason))
    return ("unclassified", (name, file, start, end, reason))


def fold_labeled_sites_to_roster_buckets(
    labeled: list[tuple[str, tuple]],
) -> tuple[list[tuple], list[tuple], list[tuple]]:
    """Sole labeled→buckets partition (offline host; see dissolve trigger)."""
    incidental: list[tuple] = []
    deliberate: list[tuple] = []
    unclassified: list[tuple] = []
    for kind, row in labeled:
        if kind == "incidental":
            incidental.append(row)
        elif kind == "expect_red":
            deliberate.append(row)
        else:
            unclassified.append(row)
    return incidental, deliberate, unclassified


def host_serialize_buckets(
    path: Path,
    rows: list[tuple],
    census_sha: str,
    disposition: str,
    with_reason: bool,
) -> None:
    """Host TSV write of already-partitioned bucket rows."""
    by_type: dict[str, list] = defaultdict(list)
    for row in rows:
        by_type[row[0]].append(row)
    lines = [
        f"# census_sha\t{census_sha}",
        "# FROZEN — type_ref_debt_roster_freeze_state = Frozen; new-sites-refuse armed",
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
    # HALF 2 labels, then sole partition fold (no parallel .dag authority).
    labeled = [
        classify_site(root, args.gunbc, name, file, start, end)
        for name, file, start, end in sites
    ]
    incidental, deliberate, unclassified = fold_labeled_sites_to_roster_buckets(
        labeled
    )

    args.out_dir.mkdir(parents=True, exist_ok=True)
    host_serialize_buckets(
        args.out_dir / "provisional_debt_roster.tsv",
        incidental,
        args.census_sha,
        "incidental_debt",
        False,
    )
    host_serialize_buckets(
        args.out_dir / "expect_red_controls.tsv",
        deliberate,
        args.census_sha,
        "deliberate_expect_red",
        True,
    )
    host_serialize_buckets(
        args.out_dir / "unclassified_sites.tsv",
        unclassified,
        args.census_sha,
        "unclassified_refuses_both_buckets",
        True,
    )
    by_type = Counter(r[0] for r in incidental)
    rank = [
        f"# census_sha\t{args.census_sha}",
        "# FROZEN — type_ref_debt_roster_freeze_state = Frozen",
        "# type\tsite_count\tdisposition=incidental_debt",
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
