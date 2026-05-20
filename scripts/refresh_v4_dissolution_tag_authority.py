#!/usr/bin/env python3
"""Post–DECISIONS.md-nuke: refresh v4 inline coproduct / gated marks to slug-only authority.

Removes stale `DECISIONS.md` and `dissolution-inventory` pointers from carrier comments
while preserving emoji classification and dissolution slugs. Coproduct tags prefer
merge-base Practice-4 mapping from `strict_deprose_dag`; files or carriers absent at
merge-base fall back to the live tag tail with ledger prefixes stripped.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

from strict_deprose_dag import (  # noqa: E402
    COPRODUCT_TAG_RE,
    ROOT as _ROOT,  # type: ignore[attr-defined]
    coproduct_tag_from_merge_base,
    coproduct_type_names_in_path,
    inject_coproduct_tags,
    is_coproduct,
)

assert ROOT == _ROOT

GATED_RE = re.compile(r"^(\s*//\s*🟡\s+gated —\s*)(.*)$")
COPRODUCT_TAIL_RE = re.compile(
    r"^(\s*//\s*)([🟢🟡🔴])(\s+coproduct dissolution —\s*)(.+?)\.?\s*$"
)

CLASSIFICATION_LEDGER_RE = re.compile(r"^classification ledger:\s*(.+)$", re.I)


def git_file_at_merge_base(rel: str) -> bool:
    try:
        subprocess.check_call(
            ["git", "cat-file", "-e", f"92cb26402eeb21471acb6ac47559cbae3b52afdb:{rel}"],
            cwd=ROOT,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        return True
    except subprocess.CalledProcessError:
        return False


def live_coproduct_tag_line(path: Path, type_name: str) -> str | None:
    lines = path.read_text().splitlines()
    mod = next(i for i, ln in enumerate(lines) if ln.startswith("module "))
    bl = lines[mod + 1 :]
    for i, _ln in enumerate(bl):
        if is_coproduct(bl, i) and TYPE_NAME(bl, i) == type_name:
            if i > 0 and COPRODUCT_TAG_RE.match(bl[i - 1]):
                return bl[i - 1]
            j = i + 1
            while j < len(bl):
                s = bl[j].strip()
                if COPRODUCT_TAG_RE.match(bl[j]):
                    return bl[j]
                if s.startswith("|") or s.startswith("="):
                    break
                if s and not s.startswith("//"):
                    break
                j += 1
    return None


def TYPE_NAME(bl: list[str], i: int) -> str | None:
    m = re.match(r"^type\s+([A-Za-z_][A-Za-z0-9_]*)\b", bl[i])
    return m.group(1) if m else None


def live_feature_or_anchor_tag(path: Path, type_name: str) -> str | None:
    lines = path.read_text().splitlines()
    mod = next(i for i, ln in enumerate(lines) if ln.startswith("module "))
    bl = lines[mod + 1 :]
    for i, ln in enumerate(bl):
        if ln.strip() == f"type {type_name}" or ln.startswith(f"type {type_name} "):
            j = i - 1
            while j >= 0 and bl[j].strip().startswith("//"):
                if "DECISIONS.md" in bl[j] or COPRODUCT_TAG_RE.match(bl[j]):
                    return bl[j]
                j -= 1
    return None


def slug_from_live_tag_line(line: str) -> tuple[str, str] | None:
    m = COPRODUCT_TAIL_RE.match(line)
    if m:
        emoji, tail = m.group(2), m.group(4).strip()
    elif "DECISIONS.md" in line:
        em_m = re.search(r"//\s*([🟢🟡🔴])", line)
        slug_m = re.search(r"DECISIONS\.md\s+(.+?)\.?\s*$", line)
        if not em_m or not slug_m:
            return None
        emoji, tail = em_m.group(1), slug_m.group(1).strip()
    else:
        return None
    tail = re.sub(r"^DECISIONS\.md\s*", "", tail)
    tail = re.sub(r"^Part 6\s*·\s*", "", tail)
    mcl = CLASSIFICATION_LEDGER_RE.match(tail)
    if mcl:
        return emoji, mcl.group(1).strip()
    return emoji, tail.rstrip(".")


def build_tag_map(rel: str, path: Path) -> dict[str, tuple[str, str]]:
    tag_map: dict[str, tuple[str, str]] = {}
    if git_file_at_merge_base(rel):
        try:
            tag_map = coproduct_tag_from_merge_base(rel)
        except SystemExit:
            tag_map = {}

    live = coproduct_type_names_in_path(path)
    for nm in live:
        if nm in tag_map:
            continue
        prev = live_coproduct_tag_line(path, nm)
        if not prev:
            prev = live_feature_or_anchor_tag(path, nm)
        if not prev:
            print(f"FAIL: {rel} coproduct {nm!r} has no tag line", file=sys.stderr)
            sys.exit(1)
        parsed = slug_from_live_tag_line(prev)
        if not parsed:
            print(f"FAIL: cannot parse tag for {rel} {nm!r}: {prev!r}", file=sys.stderr)
            sys.exit(1)
        em, slug = parsed
        if CLASSIFICATION_LEDGER_RE.match(
            re.sub(r"^DECISIONS\.md\s*", "", prev.split("—", 1)[-1].strip())
        ):
            # Type-name-only ledger pointer — use merge-base classification when possible.
            if git_file_at_merge_base(rel):
                try:
                    tag_map = {**tag_map, **coproduct_tag_from_merge_base(rel)}
                    if nm in tag_map:
                        continue
                except SystemExit:
                    pass
            slug = f"CP-3229-GREEN-TERMINAL" if em == "🟢" else slug
        tag_map[nm] = (em, slug)
    return tag_map


def refresh_gated_and_predicate_lines(text: str) -> str:
    out: list[str] = []
    for line in text.splitlines():
        gm = GATED_RE.match(line)
        if gm:
            body = gm.group(2)
            body = body.replace("dissolution-inventory §1.1 P3 — ", "")
            body = re.sub(r"DECISIONS\.md Part 6 · ", "", body)
            body = re.sub(r"DECISIONS\.md ", "", body)
            line = gm.group(1) + body
        if "dissolution-inventory.md" in line or "DECISIONS.md" in line:
            line = line.replace("docs/audit/dissolution-inventory.md §1.0 R1 (PR #3284); ", "")
            # Preserve slug + parenthetical when only the ledger prefix is stale (e.g. CP-1b anchors).
            line = re.sub(
                r"DECISIONS\.md (§[^;]+;\s*)",
                r"\1",
                line,
            )
            line = re.sub(r"DECISIONS\.md §[^;]+;\s*", "", line)
            line = re.sub(r"DECISIONS\.md/§\S+", "modeling-discipline.md", line)
            line = re.sub(r"DECISIONS\.md Part 6 · ", "", line)
            line = re.sub(r"per DECISIONS\.md Part 6", "per TASKS.md T-4.6", line)
            line = re.sub(r"DECISIONS\.md ", "", line)
        out.append(line)
    return "\n".join(out) + ("\n" if text.endswith("\n") else "")


def refresh_file(rel: str) -> bool:
    path = ROOT / rel
    text = path.read_text()
    tag_map = build_tag_map(rel, path)
    lines = text.splitlines(keepends=True)
    mod_idx = next(i for i, ln in enumerate(lines) if ln.startswith("module "))
    header = "".join(lines[: mod_idx + 1])
    body = "".join(lines[mod_idx + 1 :])
    new_body = inject_coproduct_tags(body, rel, tag_map)
    new_text = refresh_gated_and_predicate_lines(header + new_body)
    if new_text != text:
        path.write_text(new_text)
        return True
    return False


def main() -> None:
    proc = subprocess.run(
        [
            "rg",
            "-l",
            r"DECISIONS\.md|dissolution-inventory",
            "src/v4",
            "--glob",
            "*.dag",
        ],
        cwd=ROOT,
        text=True,
        capture_output=True,
    )
    targets = proc.stdout.strip()
    if proc.returncode not in (0, 1) or not targets:
        print("OK: no stale ledger pointers in src/v4/*.dag")
        return
    changed: list[str] = []
    for rel in sorted(targets.splitlines()):
        if refresh_file(rel):
            changed.append(rel)
    if changed:
        print("refreshed:", ", ".join(changed))
    else:
        print("OK: no file changes (already current)")


if __name__ == "__main__":
    main()
