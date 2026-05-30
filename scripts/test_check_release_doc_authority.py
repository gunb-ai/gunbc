#!/usr/bin/env python3
"""Self-test for scripts/check_release_doc_authority.py."""

from __future__ import annotations

import contextlib
import io
import tempfile
from pathlib import Path

from check_release_doc_authority import check


def write_seed_docs(root: Path) -> None:
    (root / "docs/thesis").mkdir(parents=True, exist_ok=True)
    (root / "docs/r3-structure.md").write_text("", encoding="utf-8")
    (root / "docs/thesis/r2-r3-thesis-mapping.md").write_text("", encoding="utf-8")


def run_check(root: Path) -> tuple[int, str]:
    out = io.StringIO()
    with contextlib.redirect_stdout(out):
        code = check(root)
    return code, out.getvalue()


def write_r2(root: Path, content: str) -> None:
    (root / "docs").mkdir(parents=True, exist_ok=True)
    (root / "docs/r2-structure.md").write_text(content, encoding="utf-8")


def expect_fail(root: Path, label: str, content: str) -> bool:
    write_r2(root, f"# R2 Structure (negative-test fixture)\n\n{content}\n")
    code, _ = run_check(root)
    if code != 0:
        return True
    print(f"FAIL [negative/{label}]: consumer passed on a fixture with live '{label}'")
    print(f"  Expected: consumer should detect '{label}' in non-retraction context")
    return False


def main() -> int:
    failures = 0
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        write_seed_docs(root)

        cases = [
            ("T-Ground-Engine", "T-Ground-Engine is a live lane in this fixture."),
            ("T-Ground-Annotation", "T-Ground-Annotation is a live program-side substrate lane."),
            ("canonical choice", "When multiple inhabitants exist, the canonical choice is declared at the language level."),
            ("@target", "Users annotate program-side intent via @target syntax."),
            ("DECISIONS LOCKED", "DECISIONS LOCKED 2026-04-28: Director ratified all 8 challenges as final decisions."),
            ("T-Verification-L4L7", "T-Verification-L4L7 verifies the no-engine claim via runtime evaluation."),
        ]
        for label, content in cases:
            print(f"Test (negative/{label}): live string should fail consumer...")
            if expect_fail(root, label, content):
                print("  PASS")
            else:
                failures += 1

        print()
        print("=== Pinned v1 limitations (NOT contract assertions; documented foot-guns) ===")
        write_r2(
            root,
            "# R2 Structure (foot-gun fixture per gpt-5-5-pro review)\n\n"
            "T-Ground-Engine is a live lane in this fixture; an unrelated prior plan was retracted last quarter.\n",
        )
        code, _ = run_check(root)
        if code == 0:
            print("  DOCUMENTED-LIMITATION (v1 retraction-pattern foot-gun is exempt by design; v2 narrowing will flip this)")
        else:
            print("  ASSERTION FLIPPED (consumer now rejects the foot-gun fixture; update this self-test)")
            failures += 1

        print()
        print("=== Contract assertions (resume) ===")
        print("Test (missing-doc): consumer must fail-closed when a configured doc is missing...")
        write_r2(root, "# R2 Structure (test fixture - clean; no forbidden strings)\n")
        (root / "docs/r3-structure.md").unlink()
        code, output = run_check(root)
        if code != 0 and "MISSING: docs/r3-structure.md" in output:
            print("  PASS")
        else:
            print("FAIL [missing-doc]: consumer did not fail closed naming r3-structure.md")
            failures += 1
        (root / "docs/r3-structure.md").write_text("", encoding="utf-8")

        print("Test (positive): retraction-context forbidden strings should pass consumer...")
        write_r2(
            root,
            "# R2 Structure (test fixture - retraction context)\n\n"
            "~~T-Ground-Engine~~ RETRACTED 2026-04-28 - replaced by substrate-completion lanes.\n"
            "The retracted T-Ground-Engine framing is described here for audit only.\n"
            "SUPERSEDED 2026-04-28: prior \"DECISIONS LOCKED\" [retraction-context] framing was retracted.\n"
            "The retracted T-Ground-Annotation lane is replaced. @target [retraction-context: annotation supersession] no longer used.\n"
            "The \"canonical choice\" [retraction-context: documenting supersession] framing was retracted.\n",
        )
        code, _ = run_check(root)
        if code == 0:
            print("  PASS")
        else:
            print("FAIL: consumer rejected a fixture with all forbidden strings in retraction context")
            failures += 1

    if failures:
        print(f"test-check-release-doc-authority: FAILED ({failures} failure(s))")
        return 1
    print("test-check-release-doc-authority: OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
