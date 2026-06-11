#!/usr/bin/env python3
"""T-38B lens_ownership family roster and run_test_claim wiring receipt."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

OWNERSHIP_CLAIM = ROOT / "src/v4/test/claim/lens_ownership/resource_dependency.dag"
SUBJECT_ROSTER = ROOT / "src/v4/test/claim/lens_ownership/subject_roster.dag"
FAMILY_EVAL = ROOT / "src/v4/test/claim/workflow/lens_ownership_family_eval.dag"
# Note: the former `src/v4/workflow/ci.dag` corpus-eval wiring assertions were dropped when that
# descriptive-only model was deleted (PR #4543); CI is hand-authored in .github/workflows/ci.yml.


def _read(path: Path) -> str:
    if not path.is_file():
        raise SystemExit(f"missing required path: {path}")
    return path.read_text(encoding="utf-8")


def _require_substrings(label: str, text: str, needles: tuple[str, ...]) -> None:
    missing = [needle for needle in needles if needle not in text]
    if missing:
        raise SystemExit(f"{label}: missing required substrings: {missing!r}")


def main() -> None:
    ownership_claim = _read(OWNERSHIP_CLAIM)
    subject_roster = _read(SUBJECT_ROSTER)
    family_eval = _read(FAMILY_EVAL)

    _require_substrings(
        "resource_dependency.dag",
        ownership_claim,
        (
            "data ownership_resource_dependency_claim_passes: Bool",
            "ResourceDependsOn",
            "RequiresAccessWitness",
            "ownership_witness",
        ),
    )

    _require_substrings(
        "subject_roster.dag",
        subject_roster,
        (
            "data claim_lens_ownership_resource_dependency_id: Symbol = claim_lens_ownership_resource_dependency",
            "ownership_resource_dependency_claim_passes",
            "fn ownership_claim_input(ok: Bool) -> Node",
            "lhs: ownership_claim_input(ok: ownership_resource_dependency_claim_passes)",
            "rhs: ownership_claim_pass_node()",
            "data subject_lens_ownership_resource_dependency: TestClaimEvalSubject<Node>",
            "eval_test_claim_subject(",
            "data lens_ownership_subject_rows: List<TestClaimEvalSubject<Node>>",
            "subject_lens_ownership_resource_dependency",
            "data lens_ownership_family_claim_ids: List<Symbol>",
            "claim_lens_ownership_resource_dependency_id",
            "data lens_ownership_node_subject_rows: List<TestClaimEvalSubject<Node>> = lens_ownership_subject_rows",
        ),
    )

    if "type LensOwnershipSubject" in subject_roster:
        raise SystemExit("subject_roster.dag: LensOwnershipSubject parallel-authority wrapper is forbidden")

    _require_substrings(
        "lens_ownership_family_eval.dag",
        family_eval,
        (
            "import v4.compiler.eval",
            "run_test_claim",
            "ownership_resource_dependency_claim_passes",
            "lens_ownership_subject_rows",
            "fn run_lens_ownership_subjects",
            "map(subjects, fn(subject) { run_test_claim(subject: subject) })",
            "runs: run_lens_ownership_subjects(subjects: lens_ownership_subject_rows)",
            "fn lens_ownership_family_report_tally",
            "fn lens_ownership_structural_witnesses_hold",
            "ownership_resource_dependency_claim_passes",
            "lens_ownership_structural_witnesses_hold() && lens_ownership_family_all_pass(report: report)",
            "witness_lens_ownership_family_gate_closed",
        ),
    )

    print("OK: T-38B lens_ownership subject roster + run_test_claim family CI receipt.")


if __name__ == "__main__":
    main()
