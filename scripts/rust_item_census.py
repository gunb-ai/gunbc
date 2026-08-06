#!/usr/bin/env python3
"""G0: exact Rust-item census for seed-growth admission lane.

Enumerates every top-level Rust item in git-tracked .rs files.
Primary denominator is item identity; LOC is secondary metadata.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from collections import defaultdict
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Iterator

# Generated artifact paths from gunbc.generated_artifact (subset verified at runtime).
GENERATED_PATHS = {
    "src/v1/stage0/src/bootstrap_stage0_crate_layout_generated.rs",
    "src/v1/stage0/src/v1_interpreter_dispatch_generated.rs",
}

GENERATED_STAGE0_BASENAMES = {
    "bootstrap_stage0_crate_layout_generated.rs",
    "v1_interpreter_dispatch_generated.rs",
}

# Heuristic: files matching stage0 emit model generated basenames.
STAGE0_GENERATED_PREFIXES = (
    "v1_compiler_",
    "v1_std_",
    "v1_tests_",
    "extdeps_",
    "gunbc_",
    "std_",
    "v2_",
)


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


def is_generated_path(repo_path: str) -> bool:
    if repo_path in GENERATED_PATHS:
        return True
    basename = Path(repo_path).name
    if basename in GENERATED_STAGE0_BASENAMES:
        return True
    if repo_path.startswith("src/v1/stage0/src/") and any(
        basename.startswith(p) for p in STAGE0_GENERATED_PREFIXES
    ):
        # Stage0 generated files follow v1_compiler_* etc. naming from emit model.
        # Exclude known hand-maintained exceptions.
        hand_exceptions = {
            "cli_run.rs",
            "main.rs",
            "memory_governor.rs",
            "resolved_graph_cache.rs",
            "coproduct_reflection.rs",
            "module_path_index",
            "test_module_hygiene_bridge.rs",
        }
        if basename in hand_exceptions or basename.startswith("bin/"):
            return False
        if basename.endswith("_generated.rs"):
            return True
        if any(basename.startswith(p) for p in STAGE0_GENERATED_PREFIXES):
            return True
    return False


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
    impl_stack: list[tuple[str, int]] = []  # (subject, start_line)
    brace_depth = 0
    in_impl = False
    current_impl_subject: str | None = None
    impl_start = 0

    for i, raw_line in enumerate(lines, start=1):
        line = strip_attrs_and_comments(raw_line)
        if not line:
            continue

        # Track impl blocks
        impl_m = IMPL_PATTERN.match(line)
        if impl_m and "{" in line:
            subject = impl_m.group(1).strip()
            # Normalize whitespace in impl subject
            subject = re.sub(r"\s+", " ", subject)
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
            impl_stack.append((subject, i))
            current_impl_subject = subject
            in_impl = True
            impl_start = i
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
                impl_stack.clear()
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

    # Approximate LOC per item (distance to next item or EOF)
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
    files = git_tracked_rs_files(repo_root)
    all_items: list[RustItem] = []
    file_loc: dict[str, int] = {}
    by_kind: dict[str, int] = defaultdict(int)
    hand_items: list[RustItem] = []
    gen_items: list[RustItem] = []

    for repo_path in files:
        full = repo_root / repo_path
        if not full.exists():
            continue
        content = full.read_text(encoding="utf-8", errors="replace")
        file_loc[repo_path] = len(content.splitlines())
        generated = is_generated_path(repo_path)
        items = parse_items(repo_path, content, generated)
        all_items.extend(items)
        for item in items:
            by_kind[item.kind] += 1
            if generated:
                gen_items.append(item)
            else:
                hand_items.append(item)

    hand_loc = sum(
        loc for path, loc in file_loc.items() if not is_generated_path(path)
    )
    gen_loc = sum(loc for path, loc in file_loc.items() if is_generated_path(path))

    # Top files by item count and LOC
    items_by_file: dict[str, list[RustItem]] = defaultdict(list)
    for item in hand_items:
        items_by_file[item.repo_path].append(item)

    top_by_items = sorted(
        ((p, len(v), file_loc.get(p, 0)) for p, v in items_by_file.items()),
        key=lambda x: -x[1],
    )[:20]

    top_by_loc = sorted(file_loc.items(), key=lambda x: -x[1])[:20]

    src_v1_hand = [
        p for p in files if p.startswith("src/v1/") and not is_generated_path(p)
    ]
    src_v1_hand_loc = sum(file_loc.get(p, 0) for p in src_v1_hand)

    return {
        "head": subprocess.check_output(
            ["git", "-C", str(repo_root), "rev-parse", "HEAD"], text=True
        ).strip(),
        "tracked_rs_files": len(files),
        "hand_files": len([p for p in files if not is_generated_path(p)]),
        "generated_files": len([p for p in files if is_generated_path(p)]),
        "total_items": len(all_items),
        "hand_items": len(hand_items),
        "generated_items": len(gen_items),
        "hand_loc": hand_loc,
        "generated_loc": gen_loc,
        "src_v1_hand_files": len(src_v1_hand),
        "src_v1_hand_loc": src_v1_hand_loc,
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
        "top_files_by_loc": [{"path": p, "loc": loc} for p, loc in top_by_loc],
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
    files = git_tracked_rs_files(repo_root)
    added: list[dict] = []
    removed: list[dict] = []
    modified_paths: list[str] = []

    for repo_path in files:
        if is_generated_path(repo_path):
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
            item_key(i): i for i in parse_items(repo_path, base_content, generated=False)
        }
        head_items = {
            item_key(i): i for i in parse_items(repo_path, head_content, generated=False)
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
        f"{result['hand_files']} files",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
