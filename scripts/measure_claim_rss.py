#!/usr/bin/env python3
"""Measure peak child RSS for claim_batch resolve+witness."""
import resource
import subprocess
import sys

REPO = "/home/briansrls/.worktrees/gunbc/fierce-raven-399"
CLAIM_BATCH = f"{REPO}/target/debug/claim_batch"


def measure(entry: str, functions: list[str]) -> tuple[float, int]:
    cmd = [
        CLAIM_BATCH,
        "--source-root",
        "dsl",
        "--source-root",
        f"{REPO}/src/v2",
        "--entry",
        entry,
        "--functions",
        *functions,
    ]
    proc = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True)
    rss_gib = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss / 1024 / 1024
    return rss_gib, proc.returncode


if __name__ == "__main__":
    entry = sys.argv[1]
    fns = sys.argv[2:]
    rss, rc = measure(entry, fns)
    oom = rc == -9 or rc == 137
    print(f"entry={entry}")
    print(f"functions={fns}")
    print(f"peak_child_rss_gib={rss:.3f}")
    print(f"returncode={rc} oom={oom}")
