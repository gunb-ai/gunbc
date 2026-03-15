from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
import re
import subprocess
import textwrap
from typing import Iterable, List


ROOT = Path(__file__).resolve().parents[2]
WORKBOARD_PATH = ROOT / "WORKBOARD.md"
INVARIANTS_PATH = ROOT / "INVARIANTS.md"
RUN_LOG_DIR = ROOT / "target" / "openclaw" / "runs"

SCOUT_PATHSPEC = "src/v1"
SCOUT_SUFFIXES = (".rs",)
TREE_PATHSPEC = "src"

MANUAL_START = "<!-- openclaw:manual:start -->"
MANUAL_END = "<!-- openclaw:manual:end -->"
SUMMARY_START = "<!-- openclaw:summary:start -->"
SUMMARY_END = "<!-- openclaw:summary:end -->"
SCOUT_START = "<!-- openclaw:scout:start -->"
SCOUT_END = "<!-- openclaw:scout:end -->"
FINDINGS_START = "<!-- openclaw:findings:start -->"
FINDINGS_END = "<!-- openclaw:findings:end -->"
TREE_START = "<!-- openclaw:tree:start -->"
TREE_END = "<!-- openclaw:tree:end -->"

CHECKBOX_RE = re.compile(r"^- \[(?P<done>[ x])\] (?P<body>.*)$")
BACKTICK_PATH_RE = re.compile(r"`([^`]+)`")


@dataclass
class CheckboxItem:
    done: bool
    body: str
    path: str | None = None


def now_iso() -> str:
    return datetime.now().astimezone().replace(microsecond=0).isoformat()


def ensure_workboard() -> str:
    if not WORKBOARD_PATH.exists():
        WORKBOARD_PATH.write_text(
            textwrap.dedent(
                """\
                # Repo Workboard

                <!-- openclaw:manual:start -->
                <!-- openclaw:manual:end -->

                <!-- openclaw:summary:start -->
                <!-- openclaw:summary:end -->

                <!-- openclaw:scout:start -->
                <!-- openclaw:scout:end -->

                <!-- openclaw:findings:start -->
                <!-- openclaw:findings:end -->

                <!-- openclaw:tree:start -->
                <!-- openclaw:tree:end -->
                """
            )
        )
    return WORKBOARD_PATH.read_text()


def run_git(args: List[str], cwd: Path | None = None, check: bool = True) -> subprocess.CompletedProcess:
    completed = subprocess.run(
        ["git"] + args,
        cwd=str(cwd or ROOT),
        capture_output=True,
        text=True,
    )
    if check and completed.returncode != 0:
        raise RuntimeError(
            "git command failed: git {args}\nstdout:\n{stdout}\nstderr:\n{stderr}".format(
                args=" ".join(args),
                stdout=completed.stdout,
                stderr=completed.stderr,
            )
        )
    return completed


def extract_section(text: str, start: str, end: str) -> str:
    pattern = re.compile(re.escape(start) + r"\n?(.*?)\n?" + re.escape(end), re.DOTALL)
    match = pattern.search(text)
    if not match:
        raise RuntimeError("missing managed section {start}..{end}".format(start=start, end=end))
    return match.group(1).strip("\n")


def replace_section(text: str, start: str, end: str, body: str) -> str:
    pattern = re.compile(re.escape(start) + r"\n?(.*?)\n?" + re.escape(end), re.DOTALL)
    replacement = "{start}\n{body}\n{end}".format(start=start, body=body.rstrip("\n"), end=end)
    updated, count = pattern.subn(replacement, text, count=1)
    if count != 1:
        raise RuntimeError("unable to replace managed section {start}..{end}".format(start=start, end=end))
    return updated


def parse_checkbox_items(section_text: str) -> list[CheckboxItem]:
    items: list[CheckboxItem] = []
    for raw_line in section_text.splitlines():
        line = raw_line.strip()
        match = CHECKBOX_RE.match(line)
        if not match:
            continue
        body = match.group("body")
        path_match = BACKTICK_PATH_RE.search(body)
        items.append(
            CheckboxItem(
                done=match.group("done") == "x",
                body=body,
                path=path_match.group(1) if path_match else None,
            )
        )
    return items


def render_checkbox_items(items: Iterable[CheckboxItem], empty_message: str) -> str:
    rendered = []
    for item in items:
        checkbox = "x" if item.done else " "
        rendered.append("- [{checkbox}] {body}".format(checkbox=checkbox, body=item.body))
    if not rendered:
        return empty_message
    return "\n".join(rendered)


def gather_repo_paths(pathspec: str, suffixes: tuple[str, ...] = ()) -> list[str]:
    tracked = run_git(["ls-files", "--", pathspec]).stdout.splitlines()
    untracked = run_git(["ls-files", "--others", "--exclude-standard", "--", pathspec]).stdout.splitlines()
    paths = sorted(set(path for path in tracked + untracked if path))
    if suffixes:
        paths = [path for path in paths if path.endswith(suffixes)]
    return paths


def build_tree(paths: list[str]) -> str:
    if not paths:
        return "```text\n(no files)\n```"
    tree: dict[str, dict] = {}
    for path in paths:
        node = tree
        for part in path.split("/"):
            node = node.setdefault(part, {})
    lines: list[str] = []

    def walk(node: dict[str, dict], prefix: str) -> None:
        items = sorted(node.items())
        for index, (name, child) in enumerate(items):
            last = index == len(items) - 1
            connector = "└── " if last else "├── "
            lines.append("{prefix}{connector}{name}".format(prefix=prefix, connector=connector, name=name))
            if child:
                child_prefix = prefix + ("    " if last else "│   ")
                walk(child, child_prefix)

    walk(tree, "")
    return "```text\n{body}\n```".format(body="\n".join(lines))


def get_manual_items(text: str | None = None) -> list[CheckboxItem]:
    workboard = text if text is not None else ensure_workboard()
    return parse_checkbox_items(extract_section(workboard, MANUAL_START, MANUAL_END))


def get_scout_items(text: str | None = None) -> list[CheckboxItem]:
    workboard = text if text is not None else ensure_workboard()
    return parse_checkbox_items(extract_section(workboard, SCOUT_START, SCOUT_END))


def get_findings_lines(text: str | None = None) -> list[str]:
    workboard = text if text is not None else ensure_workboard()
    section = extract_section(workboard, FINDINGS_START, FINDINGS_END)
    return [line.strip() for line in section.splitlines() if line.strip()]


def refresh_summary_section(text: str) -> str:
    manual_items = get_manual_items(text)
    scout_items = get_scout_items(text)
    findings = get_findings_lines(text)
    last_event = findings[0][2:] if findings else "never"
    summary = "\n".join(
        [
            "- Manual tasks open: {count}".format(count=sum(not item.done for item in manual_items)),
            "- Scout files remaining: {count}".format(count=sum(not item.done for item in scout_items)),
            "- Last event: {event}".format(event=last_event),
        ]
    )
    return replace_section(text, SUMMARY_START, SUMMARY_END, summary)


def sync_workboard() -> dict[str, int]:
    text = ensure_workboard()
    existing_scout = {item.path: item.done for item in get_scout_items(text) if item.path}
    scout_items = [
        CheckboxItem(done=existing_scout.get(path, False), body="`{path}`".format(path=path), path=path)
        for path in gather_repo_paths(SCOUT_PATHSPEC, SCOUT_SUFFIXES)
    ]
    findings = get_findings_lines(text)
    if not findings:
        findings = ["- {timestamp} initialized workboard scaffold".format(timestamp=now_iso())]
    tree_block = build_tree(gather_repo_paths(TREE_PATHSPEC))
    text = replace_section(
        text,
        SCOUT_START,
        SCOUT_END,
        render_checkbox_items(
            scout_items,
            "_No scout files found under `{pathspec}`._".format(pathspec=SCOUT_PATHSPEC),
        ),
    )
    text = replace_section(text, FINDINGS_START, FINDINGS_END, "\n".join(findings))
    text = replace_section(text, TREE_START, TREE_END, tree_block)
    text = refresh_summary_section(text)
    WORKBOARD_PATH.write_text(text)
    return {
        "manual_open": sum(not item.done for item in get_manual_items(text)),
        "scout_open": sum(not item.done for item in get_scout_items(text)),
    }


def append_finding(message: str) -> None:
    text = ensure_workboard()
    findings = get_findings_lines(text)
    findings = ["- {timestamp} {message}".format(timestamp=now_iso(), message=message)] + findings
    findings = findings[:25]
    text = replace_section(text, FINDINGS_START, FINDINGS_END, "\n".join(findings))
    text = refresh_summary_section(text)
    WORKBOARD_PATH.write_text(text)


def mark_manual_item_complete(body: str) -> bool:
    text = ensure_workboard()
    items = get_manual_items(text)
    updated = False
    for item in items:
        if not item.done and item.body == body:
            item.done = True
            updated = True
            break
    if not updated:
        return False
    text = replace_section(
        text,
        MANUAL_START,
        MANUAL_END,
        render_checkbox_items(
            items,
            "<!-- Add unchecked checkbox items here. -->",
        ),
    )
    text = refresh_summary_section(text)
    WORKBOARD_PATH.write_text(text)
    return True


def mark_scout_item_complete(path: str) -> bool:
    text = ensure_workboard()
    items = get_scout_items(text)
    updated = False
    for item in items:
        if item.path == path:
            item.done = True
            updated = True
            break
    if not updated:
        return False
    text = replace_section(
        text,
        SCOUT_START,
        SCOUT_END,
        render_checkbox_items(
            items,
            "_No scout files found under `{pathspec}`._".format(pathspec=SCOUT_PATHSPEC),
        ),
    )
    text = refresh_summary_section(text)
    WORKBOARD_PATH.write_text(text)
    return True


def first_non_empty_line(text: str) -> str:
    for line in text.splitlines():
        stripped = line.strip()
        if stripped:
            return stripped
    return ""


def truncate(text: str, limit: int = 160) -> str:
    if len(text) <= limit:
        return text
    return text[: limit - 3].rstrip() + "..."
