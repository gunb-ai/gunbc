#!/usr/bin/env python3
"""G0: exact Rust-item census for seed-growth admission lane.

SCAFFOLD — see gunbc.seed_growth_admission seed_growth_g0_census_host_scaffold.
Dissolves when rust_item_host_observation modeled producer lands.

Enumerates every top-level Rust item in git-tracked .rs files.
Primary denominator is item identity; LOC is secondary metadata.

Generated-vs-hand classification is derived from corpus authorities only:
  - gunbc.stage0_emit_plan_generated generated_stage0_files (basenames)
  - gunbc.generated_artifact artifact_path rows (repo paths)
Unclassified stage0/src paths refuse rather than defaulting to hand.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from collections import defaultdict
from dataclasses import asdict, dataclass
from pathlib import Path

STAGE0_SRC_PREFIX = "src/v1/stage0/src/"
GENERATED_STAGE0_DAG = "dag/gunbc/stage0_emit_plan_generated.dag"
GENERATED_ARTIFACT_DAG = "dag/gunbc/generated_artifact.dag"


@dataclass
class RustItem:
    repo_path: str
    kind: str
    name: str
    impl_subject: str | None = None
    start_line: int = 0
    end_line: int = 0
    loc: int = 0
    generated: bool = False

    @property
    def identity_key(self) -> str:
        if self.impl_subject:
            return f"{self.repo_path}::{self.kind}::{self.impl_subject}::{self.name}"
        return f"{self.repo_path}::{self.kind}::{self.name}"


ITEM_PATTERNS: list[tuple[str, re.Pattern[str]]] = [
    ("fn", re.compile(r"^(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+(\w+)")),
    ("struct", re.compile(r"^(?:pub(?:\([^)]*\))?\s+)?struct\s+(\w+)")),
    ("enum", re.compile(r"^(?:pub(?:\([^)]*\))?\s+)?enum\s+(\w+)")),
    ("type", re.compile(r"^(?:pub(?:\([^)]*\))?\s+)?type\s+(\w+)")),
    ("trait", re.compile(r"^(?:pub(?:\([^)]*\))?\s+)?trait\s+(\w+)")),
    ("const", re.compile(r"^(?:pub(?:\([^)]*\))?\s+)?const\s+(\w+)")),
    ("static", re.compile(r"^(?:pub(?:\([^)]*\))?\s+)?static\s+(\w+)")),
    ("mod", re.compile(r"^(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)")),
    ("macro_rules", re.compile(r"^macro_rules!\s+(\w+)")),
    ("use", re.compile(r"^use\s+")),
]

IMPL_PATTERN = re.compile(
    r"^(?:pub(?:\([^)]*\))?\s+)?impl(?:<[^>]*>)?\s+(.+?)\s*\{"
)
IMPL_FN_PATTERN = re.compile(
    r"^(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+(\w+)"
)


def git_tracked_rs_files(repo_root: Path) -> list[str]:
    out = subprocess.check_output(
        ["git", "-C", str(repo_root), "ls-files", "*.rs"],
        text=True,
    )
    return sorted(line.strip() for line in out.splitlines() if line.strip())


def _parse_dag_string_list(dag_text: str, data_name: str) -> list[str]:
    pattern = re.compile(
        rf"data {re.escape(data_name)}: List<String> = \[(.*?)\]",
        re.DOTALL,
    )
    match = pattern.search(dag_text)
    if not match:
        raise ValueError(f"could not parse {data_name} from authority dag")
    return re.findall(r'"([^"]+)"', match.group(1))


def _parse_artifact_paths(dag_text: str) -> set[str]:
  # artifact_path match arms that are string literals (not concat calls).
    return set(re.findall(r'=>\s*"([^"]+)"', dag_text))


def load_generation_authorities(repo_root: Path) -> tuple[set[str], set[str]]:
    stage0_dag = (repo_root / GENERATED_STAGE0_DAG).read_text(encoding="utf-8")
    artifact_dag = (repo_root / GENERATED_ARTIFACT_DAG).read_text(encoding="utf-8")
    basenames = set(_parse_dag_string_list(stage0_dag, "generated_stage0_files"))
    artifact_paths = _parse_artifact_paths(artifact_dag)
    return basenames, artifact_paths


def classify_repo_path(
    repo_path: str,
    generated_basenames: set[str],
    artifact_paths: set[str],
) -> str:
    """Return generated | hand | unclassified."""
    if repo_path in artifact_paths:
        return "generated"
    if repo_path.startswith(STAGE0_SRC_PREFIX):
        basename = Path(repo_path).name
        if basename in generated_basenames:
            return "generated"
        return "hand"
    if repo_path.endswith(".rs"):
        return "hand"
    return "unclassified"


def strip_attrs_and_comments(line: str) -> str:
    s = line.strip()
    if s.startswith("//"):
        return ""
    if s.startswith("#"):
        return ""
    return s


def parse_items(repo_path: str, content: str, generated: bool) -> list[RustItem]:
    lines = content.splitlines()
    items: list[RustItem] = []
    in_impl = False
    current_impl_subject: str | None = None
    brace_depth = 0

    for i, raw_line in enumerate(lines, start=1):
        line = strip_attrs_and_comments(raw_line)
        if not line:
            continue

        impl_m = IMPL_PATTERN.match(line)
        if impl_m and "{" in line:
            subject = re.sub(r"\s+", " ", impl_m.group(1).strip())
            items.append(
                RustItem(
                    repo_path=repo_path,
                    kind="impl_block",
                    name=subject[:80],
                    impl_subject=subject,
                    start_line=i,
                    generated=generated,
                )
            )
            current_impl_subject = subject
            in_impl = True
            brace_depth = line.count("{") - line.count("}")
            continue

        if in_impl:
            brace_depth += line.count("{") - line.count("}")
            fn_m = IMPL_FN_PATTERN.match(line)
            if fn_m and current_impl_subject:
                items.append(
                    RustItem(
                        repo_path=repo_path,
                        kind="impl_method",
                        name=fn_m.group(1),
                        impl_subject=current_impl_subject,
                        start_line=i,
                        generated=generated,
                    )
                )
            if brace_depth <= 0:
                in_impl = False
                current_impl_subject = None
            continue

        for kind, pat in ITEM_PATTERNS:
            if kind == "use":
                continue
            m = pat.match(line)
            if m:
                name = m.group(1) if m.lastindex else line[:40]
                items.append(
                    RustItem(
                        repo_path=repo_path,
                        kind=kind,
                        name=name,
                        start_line=i,
                        generated=generated,
                    )
                )
                break

    for idx, item in enumerate(items):
        end = len(lines)
        for j in range(idx + 1, len(items)):
            if items[j].start_line > item.start_line:
                end = items[j].start_line - 1
                break
        item.end_line = end
        item.loc = max(1, end - item.start_line + 1)

    return items


def census(repo_root: Path) -> dict:
    generated_basenames, artifact_paths = load_generation_authorities(repo_root)
    files = git_tracked_rs_files(repo_root)
    all_items: list[RustItem] = []
    file_loc: dict[str, int] = {}
    by_kind: dict[str, int] = defaultdict(int)
    hand_items: list[RustItem] = []
    gen_items: list[RustItem] = []
    unclassified_paths: list[str] = []
    path_disposition: dict[str, str] = {}

    for repo_path in files:
        disposition = classify_repo_path(
            repo_path, generated_basenames, artifact_paths
        )
        path_disposition[repo_path] = disposition
        if disposition == "unclassified":
            unclassified_paths.append(repo_path)
            continue
        full = repo_root / repo_path
        if not full.exists():
            continue
        content = full.read_text(encoding="utf-8", errors="replace")
        file_loc[repo_path] = len(content.splitlines())
        generated = disposition == "generated"
        items = parse_items(repo_path, content, generated)
        all_items.extend(items)
        for item in items:
            by_kind[item.kind] += 1
            if generated:
                gen_items.append(item)
            else:
                hand_items.append(item)

    if unclassified_paths:
        print(
            f"REFUSED: {len(unclassified_paths)} unclassified .rs path(s) "
            f"(not in generated_stage0_files or generated_artifact artifact_path)",
            file=sys.stderr,
        )
        for path in unclassified_paths[:20]:
            print(f"  unclassified: {path}", file=sys.stderr)
        if len(unclassified_paths) > 20:
            print(f"  ... and {len(unclassified_paths) - 20} more", file=sys.stderr)
        sys.exit(1)

    hand_loc = sum(
        loc for path, loc in file_loc.items() if path_disposition.get(path) == "hand"
    )
    gen_loc = sum(
        loc
        for path, loc in file_loc.items()
        if path_disposition.get(path) == "generated"
    )

    items_by_file: dict[str, list[RustItem]] = defaultdict(list)
    for item in hand_items:
        items_by_file[item.repo_path].append(item)

    top_by_items = sorted(
        ((p, len(v), file_loc.get(p, 0)) for p, v in items_by_file.items()),
        key=lambda x: -x[1],
    )[:20]

    src_v1_hand = [
        p for p in files if path_disposition.get(p) == "hand" and p.startswith("src/v1/")
    ]

    return {
        "head": subprocess.check_output(
            ["git", "-C", str(repo_root), "rev-parse", "HEAD"], text=True
        ).strip(),
        "authority": {
            "generated_stage0_files_dag": GENERATED_STAGE0_DAG,
            "generated_artifact_dag": GENERATED_ARTIFACT_DAG,
            "generated_stage0_basenames": len(generated_basenames),
            "generated_artifact_paths": len(artifact_paths),
        },
        "tracked_rs_files": len(files),
        "hand_files": len([p for p in files if path_disposition.get(p) == "hand"]),
        "generated_files": len(
            [p for p in files if path_disposition.get(p) == "generated"]
        ),
        "total_items": len(all_items),
        "hand_items": len(hand_items),
        "generated_items": len(gen_items),
        "hand_loc": hand_loc,
        "generated_loc": gen_loc,
        "src_v1_hand_files": len(src_v1_hand),
        "src_v1_hand_loc": sum(file_loc.get(p, 0) for p in src_v1_hand),
        "by_kind": dict(by_kind),
        "hand_by_kind": dict(
            defaultdict(
                int,
                {k: sum(1 for i in hand_items if i.kind == k) for k in by_kind},
            )
        ),
        "top_files_by_item_count": [
            {"path": p, "items": n, "loc": loc} for p, n, loc in top_by_items
        ],
        "cli_run": {
            "path": "src/v1/stage0/src/cli_run.rs",
            "loc": file_loc.get("src/v1/stage0/src/cli_run.rs", 0),
            "items": len(items_by_file.get("src/v1/stage0/src/cli_run.rs", [])),
        },
        "claim_executor": {
            "path": "src/v1/stage0/src/bin/claim_executor.rs",
            "loc": file_loc.get("src/v1/stage0/src/bin/claim_executor.rs", 0),
            "items": len(
                items_by_file.get("src/v1/stage0/src/bin/claim_executor.rs", [])
            ),
        },
    }


def item_key(item: RustItem) -> str:
    return item.identity_key


def diff_census(repo_root: Path, base_ref: str) -> dict:
    """G0 diff: items added/removed between base_ref and HEAD (hand only)."""
    generated_basenames, artifact_paths = load_generation_authorities(repo_root)
    files = git_tracked_rs_files(repo_root)
    added: list[dict] = []
    removed: list[dict] = []
    modified_paths: list[str] = []

    for repo_path in files:
        if classify_repo_path(repo_path, generated_basenames, artifact_paths) != "hand":
            continue
        full = repo_root / repo_path
        if not full.exists():
            continue
        try:
            base_content = subprocess.check_output(
                ["git", "-C", str(repo_root), "show", f"{base_ref}:{repo_path}"],
                text=True,
                stderr=subprocess.DEVNULL,
            )
        except subprocess.CalledProcessError:
            base_content = ""
        head_content = full.read_text(encoding="utf-8", errors="replace")
        if base_content == head_content:
            continue
        modified_paths.append(repo_path)
        base_items = {
            item_key(i): i
            for i in parse_items(repo_path, base_content, generated=False)
        }
        head_items = {
            item_key(i): i
            for i in parse_items(repo_path, head_content, generated=False)
        }
        for key, item in head_items.items():
            if key not in base_items:
                added.append(asdict(item))
        for key, item in base_items.items():
            if key not in head_items:
                removed.append(asdict(item))

    return {
        "base_ref": base_ref,
        "head": subprocess.check_output(
            ["git", "-C", str(repo_root), "rev-parse", "HEAD"], text=True
        ).strip(),
        "modified_hand_files": len(modified_paths),
        "added_hand_items": len(added),
        "removed_hand_items": len(removed),
        "added": added,
        "removed": removed,
    }


def main() -> None:
    import argparse

    parser = argparse.ArgumentParser(description="G0 exact Rust-item census")
    parser.add_argument(
        "--diff",
        metavar="BASE_REF",
        help="Diff hand items between BASE_REF and HEAD (e.g. merge-base)",
    )
    args = parser.parse_args()
    repo_root = Path(__file__).resolve().parents[1]

    if args.diff:
        result = diff_census(repo_root, args.diff)
        json.dump(result, sys.stdout, indent=2)
        print(file=sys.stderr)
        print(
            f"G0 diff {result['base_ref'][:12]}..{result['head'][:12]}: "
            f"+{result['added_hand_items']} / -{result['removed_hand_items']} hand items "
            f"across {result['modified_hand_files']} files",
            file=sys.stderr,
        )
        return

    result = census(repo_root)
    json.dump(result, sys.stdout, indent=2)
    print(file=sys.stderr)
    print(
        f"G0 census: {result['hand_items']} hand items / "
        f"{result['hand_loc']} hand LOC across "
        f"{result['hand_files']} files "
        f"(authorities: {GENERATED_STAGE0_DAG}, {GENERATED_ARTIFACT_DAG})",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
