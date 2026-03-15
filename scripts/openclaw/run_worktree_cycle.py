#!/usr/bin/env python3

from __future__ import annotations

import argparse
from pathlib import Path
import shutil
import subprocess

from workboard_lib import (
    INVARIANTS_PATH,
    ROOT,
    RUN_LOG_DIR,
    WORKBOARD_PATH,
    append_finding,
    first_non_empty_line,
    get_manual_items,
    get_scout_items,
    mark_manual_item_complete,
    mark_scout_item_complete,
    now_iso,
    run_git,
    sync_workboard,
    truncate,
)


WORKTREE_BRANCH = "openclaw/queue"
WORKTREE_PATH = Path.home() / ".worktrees" / ROOT.name / "openclaw-queue"


def default_base_ref() -> str:
    symbolic = run_git(["symbolic-ref", "refs/remotes/origin/HEAD"]).stdout.strip()
    if symbolic.startswith("refs/remotes/"):
        return symbolic[len("refs/remotes/") :]
    return "origin/main"


def run_git_in_worktree(args: list[str], check: bool = True) -> subprocess.CompletedProcess:
    return run_git(args, cwd=WORKTREE_PATH, check=check)


def worktree_exists() -> bool:
    return WORKTREE_PATH.exists() and (WORKTREE_PATH / ".git").exists()


def branch_exists(branch: str) -> bool:
    result = run_git(["show-ref", "--verify", "--quiet", "refs/heads/{branch}".format(branch=branch)], check=False)
    return result.returncode == 0


def worktree_status() -> list[str]:
    raw = run_git_in_worktree(["status", "--porcelain"], check=True).stdout.splitlines()
    paths: list[str] = []
    for line in raw:
        path = line[3:]
        if " -> " in path:
            path = path.split(" -> ", 1)[1]
        if path.startswith("target/openclaw/context/"):
            continue
        paths.append(path)
    return paths


def ensure_worktree() -> str:
    base_ref = default_base_ref()
    remote_name, remote_branch = base_ref.split("/", 1)
    run_git(["fetch", remote_name, remote_branch])
    WORKTREE_PATH.parent.mkdir(parents=True, exist_ok=True)
    if not worktree_exists():
        if not branch_exists(WORKTREE_BRANCH):
            run_git(["branch", WORKTREE_BRANCH, base_ref])
        run_git(["worktree", "add", str(WORKTREE_PATH), WORKTREE_BRANCH])
    if worktree_status():
        raise RuntimeError(
            "automation worktree has uncommitted changes: {paths}".format(
                paths=", ".join(worktree_status()[:8])
            )
        )
    ahead = int(run_git_in_worktree(["rev-list", "--count", "{base}..HEAD".format(base=base_ref)]).stdout.strip() or "0")
    if ahead == 0:
        merge = run_git_in_worktree(["merge", "--ff-only", base_ref], check=False)
        combined = (merge.stdout + merge.stderr).strip()
        if merge.returncode != 0 and "Already up to date." not in combined:
            raise RuntimeError("unable to fast-forward automation worktree to {base}: {msg}".format(base=base_ref, msg=combined))
    return base_ref


def copy_context_files() -> None:
    context_dir = WORKTREE_PATH / "target" / "openclaw" / "context"
    context_dir.mkdir(parents=True, exist_ok=True)
    shutil.copy2(INVARIANTS_PATH, context_dir / "INVARIANTS.md")
    shutil.copy2(WORKBOARD_PATH, context_dir / "WORKBOARD.md")


def build_manual_prompt(task_body: str) -> str:
    return """Read `target/openclaw/context/INVARIANTS.md` and `target/openclaw/context/WORKBOARD.md` first.

Task:
{task}

Rules:
- Work only in this git worktree.
- Keep the change narrow: the named file plus directly necessary tests/docs.
- If you make a code change, run the smallest relevant verification you can identify.
- If the task turns out to be unnecessary, leave code unchanged and say so.

End with a short summary covering:
1. what issue you found or ruled out
2. which files changed
3. what verification you ran
""".format(task=task_body)


def build_scout_prompt(path: str) -> str:
    return """Read `target/openclaw/context/INVARIANTS.md` and `target/openclaw/context/WORKBOARD.md` first.

Scout `{path}` against the repo invariants.

Rules:
- Focus on this file first and only touch directly necessary tests/docs.
- If you find a concrete issue, fix it in the smallest defensible way.
- If you do not find a concrete issue, leave code unchanged and say so.
- Run the smallest relevant verification for any change you make.

End with a short summary covering:
1. what issue you found or ruled out
2. which files changed
3. what verification you ran
""".format(path=path)


def run_codex(prompt: str, mode: str) -> tuple[int, str, Path, Path]:
    RUN_LOG_DIR.mkdir(parents=True, exist_ok=True)
    stamp = now_iso().replace(":", "").replace("+", "_plus_")
    summary_file = RUN_LOG_DIR / "{stamp}-{mode}-summary.md".format(stamp=stamp, mode=mode)
    console_file = RUN_LOG_DIR / "{stamp}-{mode}-console.log".format(stamp=stamp, mode=mode)
    completed = subprocess.run(
        [
            "codex",
            "exec",
            "--full-auto",
            "-C",
            str(WORKTREE_PATH),
            "-o",
            str(summary_file),
            "-",
        ],
        input=prompt,
        text=True,
        capture_output=True,
    )
    console_file.write_text(completed.stdout + ("\n" if completed.stdout and completed.stderr else "") + completed.stderr)
    summary = summary_file.read_text().strip() if summary_file.exists() else ""
    return completed.returncode, summary, summary_file, console_file


def commit_worktree_changes(commit_message: str) -> str:
    run_git_in_worktree(["add", "-A"])
    diff_check = run_git_in_worktree(["diff", "--cached", "--quiet"], check=False)
    if diff_check.returncode == 0:
        return ""
    run_git_in_worktree(["commit", "-m", commit_message])
    return run_git_in_worktree(["rev-parse", "--short", "HEAD"]).stdout.strip()


def describe_change_set() -> list[str]:
    return worktree_status()


def next_manual_task() -> str | None:
    for item in get_manual_items():
        if not item.done:
            return item.body
    return None


def next_scout_path() -> str | None:
    for item in get_scout_items():
        if not item.done and item.path:
            return item.path
    return None


def handle_success(mode: str, identifier: str, summary: str, changed_files: list[str]) -> None:
    summary_line = truncate(first_non_empty_line(summary) or "Codex returned no final summary.")
    commit_sha = ""
    if changed_files:
        if mode == "manual":
            commit_message = "openclaw: {text}".format(text=truncate(identifier, 72))
        else:
            commit_message = "openclaw: scout {path}".format(path=identifier)
        commit_sha = commit_worktree_changes(commit_message)
    if mode == "manual":
        mark_manual_item_complete(identifier)
    else:
        mark_scout_item_complete(identifier)
    if commit_sha:
        append_finding(
            "{mode} completed {identifier}; commit `{sha}`; changed {count} files; {summary}".format(
                mode=mode,
                identifier=identifier,
                sha=commit_sha,
                count=len(changed_files),
                summary=summary_line,
            )
        )
        print("COMPLETED {mode} {identifier} commit={sha}".format(mode=mode, identifier=identifier, sha=commit_sha))
    else:
        append_finding(
            "{mode} completed {identifier}; no code changes; {summary}".format(
                mode=mode,
                identifier=identifier,
                summary=summary_line,
            )
        )
        print("COMPLETED {mode} {identifier} no_code_changes".format(mode=mode, identifier=identifier))


def main() -> int:
    parser = argparse.ArgumentParser(description="Run one OpenClaw -> Codex worktree cycle.")
    parser.add_argument("--dry-run", action="store_true", help="Select the next task and prepare the worktree, but do not invoke Codex.")
    args = parser.parse_args()

    stats = sync_workboard()
    try:
        base_ref = ensure_worktree()
    except Exception as exc:  # pragma: no cover - exercised by runtime conditions
        append_finding("blocked: {message}".format(message=truncate(str(exc), 200)))
        print("BLOCKED {message}".format(message=str(exc)))
        return 0

    manual = next_manual_task()
    mode = "manual" if manual else "scout"
    identifier = manual or next_scout_path()
    if not identifier:
        append_finding("idle: no manual tasks and no scout files remain")
        print(
            "IDLE manual_open={manual_open} scout_open={scout_open} base={base}".format(
                manual_open=stats["manual_open"],
                scout_open=stats["scout_open"],
                base=base_ref,
            )
        )
        return 0

    copy_context_files()
    if args.dry_run:
        print(
            "DRY_RUN mode={mode} target={target} worktree={worktree} base={base}".format(
                mode=mode,
                target=identifier,
                worktree=WORKTREE_PATH,
                base=base_ref,
            )
        )
        return 0

    prompt = build_manual_prompt(identifier) if mode == "manual" else build_scout_prompt(identifier)
    returncode, summary, summary_file, console_file = run_codex(prompt, mode)
    changed_files = describe_change_set()
    if returncode != 0:
        append_finding(
            "blocked: codex {mode} run failed for {identifier}; summary `{summary}`; logs `{console}`".format(
                mode=mode,
                identifier=identifier,
                summary=truncate(first_non_empty_line(summary) or "no summary", 120),
                console=console_file.relative_to(ROOT).as_posix(),
            )
        )
        print(
            "BLOCKED codex_failed mode={mode} target={target} summary={summary_file}".format(
                mode=mode,
                target=identifier,
                summary_file=summary_file,
            )
        )
        return 0

    handle_success(mode, identifier, summary, changed_files)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
