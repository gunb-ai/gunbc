#!/usr/bin/env python3
"""Local syntax gate for a patched .dag -- so a malformed patch never costs a dispatch again.

The regeneration dispatch died with `parse-failed at src/v1/04_infer.dag:9287:13: expected
FatArrow, found Ident` after 45 minutes, because my patch re-emitted a `let` INSIDE
`match sig { ... }` (a `let` is not a match arm). That is checkable in seconds: a locally
runnable gunbc (arm64) accepts `compile --source-dir <dir>` on a single file, and parse
diagnostics are emitted BEFORE any import/type resolution, so unresolved-type noise from the
missing corpus is expected and must be ignored.

VERIFIED TO DISCRIMINATE: on the exact buggy form it prints
`expected FatArrow, found keyword 'let'`; on the repaired file it prints 0 parse diagnostics.

Exit codes: 0 parse-clean, 1 parse errors found, 2 gate could not run (NEVER treat as clean).

Usage: parsecheck.py FILE.dag
"""
import os
import re
import shutil
import subprocess
import sys
import tempfile

CANDIDATES = [
    "/home/briansrls/gunbc/target-main/release/gunbc",
    "/home/briansrls/gunbc/target/debug/gunbc",
    "/home/briansrls/.worktrees/gunbc/_pr5680_verify/target/release/gunbc",
    "/home/briansrls/.worktrees/gunbc/neat-otter-loss2/target/release/gunbc",
]

# Parse-stage diagnostics only. Everything else the compiler reports (unresolved type from a
# deliberately incomplete corpus) is noise here, and calling it clean or broken would both be
# wrong -- so the gate's answer comes from this set or not at all.
PARSE_PATTERNS = [
    r"parse-failed",
    r"expected [A-Za-z]+, found",
    r"unclosed (delimiter|brace|paren|bracket)",
    r"unexpected token",
    r"syntax error",
]


def find_binary():
    for c in CANDIDATES:
        if os.path.exists(c) and os.access(c, os.X_OK):
            return c
    w = shutil.which("gunbc")
    return w if w and os.path.realpath(w) != "/usr/local/ctrl-build-shims/gunbc" else None


def main():
    path = sys.argv[1]
    if not os.path.exists(path):
        print("PARSECHECK_UNAVAILABLE: no such file %s" % path)
        return 2
    binary = find_binary()
    if not binary:
        print("PARSECHECK_UNAVAILABLE: no locally runnable gunbc (a runner-built binary is "
              "x86-64 and cannot run here) -- do NOT read this as clean")
        return 2

    tmp = tempfile.mkdtemp(prefix="parsecheck_")
    shutil.copy(path, os.path.join(tmp, os.path.basename(path)))
    outdir = os.path.join(tmp, "out")
    try:
        proc = subprocess.run(
            [binary, "compile", "--source-dir", tmp, "--output-dir", outdir, "--target", "rust"],
            capture_output=True, text=True, timeout=int(os.environ.get("PARSECHECK_TIMEOUT", "300")),
        )
    except subprocess.TimeoutExpired:
        print("PARSECHECK_UNAVAILABLE: compile timed out -- do NOT read this as clean")
        return 2
    except OSError as e:
        print("PARSECHECK_UNAVAILABLE: %s" % e)
        return 2

    text = (proc.stdout or "") + (proc.stderr or "")
    hits = []
    for pat in PARSE_PATTERNS:
        for line in text.splitlines():
            if re.search(pat, line, re.I):
                hits.append(line.strip()[:240])
    uniq = sorted(set(hits))
    if uniq:
        print("PARSECHECK FAIL %s" % os.path.basename(path))
        for h in uniq[:12]:
            print("  PARSE %s" % h)
        return 1
    # Non-zero rc with no parse diagnostic is normal here (unresolved types from the
    # deliberately incomplete single-file corpus). Say so, so nobody reads rc as the verdict.
    print("PARSECHECK OK %s (rc=%s, %d non-parse diagnostics ignored: e.g. %s)"
          % (os.path.basename(path), proc.returncode, len(text.splitlines()),
             (text.splitlines() or ["<none>"])[0][:80]))
    return 0


sys.exit(main())
