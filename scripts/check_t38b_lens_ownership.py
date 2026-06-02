#!/usr/bin/env python3
"""T-38B lens_ownership family roster and run_test_claim wiring receipt."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

OWNERSHIP_CLAIM = ROOT / "src/v4/test/claim/lens_ownership/resource_dependency.dag"
SUBJECT_ROSTER = ROOT / "src/v4/test/claim/lens_ownership/subject_roster.dag"
FAMILY_EVAL = ROOT / "src/v4/test/claim/workflow/lens_ownership_family_eval.dag"
CI = ROOT / "src/v4/workflow/ci.dag"


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
    ci = _read(CI)

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
            "type LensOwnershipSubject",
            "structural_witness: Bool",
            "ownership_resource_dependency_claim_passes",
            "fn ownership_claim_input(ok: Bool) -> Node",
            "lhs: ownership_claim_input(ok: ownership_resource_dependency_claim_passes)",
            "rhs: ownership_claim_pass_node()",
            "data subject_lens_ownership_resource_dependency: TestClaimEvalSubject<Node>",
            "eval_test_claim_subject(",
            "data lens_ownership_subject_roster: List<LensOwnershipSubject>",
            "data lens_ownership_node_subject_rows: List<TestClaimEvalSubject<Node>>",
            "lens_ownership_subject_roster",
            "subject.eval_subject",
        ),
    )

    roster_body = subject_roster.split("data lens_ownership_node_subject_rows:")[1].split("\n\n", 1)[0]
    if "lens_ownership_subject_roster" not in roster_body or "subject.eval_subject" not in roster_body:
        raise SystemExit("lens_ownership_node_subject_rows must project from LensOwnershipSubject rows")

    _require_substrings(
        "lens_ownership_family_eval.dag",
        family_eval,
        (
            "import v4.compiler.eval",
            "run_test_claim",
            "LensOwnershipSubject",
            "fn run_lens_ownership_subjects",
            "map(subjects, fn(subject) { run_test_claim(subject: subject.eval_subject) })",
            "runs: run_lens_ownership_subjects(subjects: lens_ownership_subject_roster)",
            "fn lens_ownership_family_report_tally",
            "fn lens_ownership_structural_witnesses_hold",
            "acc && subject.structural_witness",
            "lens_ownership_structural_witnesses_hold() && lens_ownership_family_all_pass(report: report)",
            "witness_lens_ownership_family_gate_closed",
        ),
    )

    _require_substrings(
        "ci.dag",
        ci,
        (
            "lens_ownership_family_eval_execution",
            "LensOwnershipFamilyEvalCommand",
            "LensOwnershipFamilyVerdictSurfaceAuthority",
            "ci_lens_ownership_family_verdict_surface_authority",
            "surface == ci_lens_ownership_family_verdict_surface_authority()",
            "ci_lens_ownership_family_verdict_surface_projection_node",
            "ci_projection_corpus_surface_structural_witness_edge",
            "lens_ownership_structural_witnesses_hold",
            "ci_lens_ownership_family_eval_command",
            "ci_upsert_lens_ownership_family_eval_execution_mk",
            "ci_upsert_lens_ownership_family_eval_signal_mk",
            "ci_upsert_lens_ownership_family_eval_execution",
            "ci_upsert_lens_ownership_family_eval_signal",
            "ci_upsert_steps_full_in_scope_step_ids",
            "src/v4/test/claim/lens_ownership/subject_roster.dag",
            "src/v4/test/claim/workflow/lens_ownership_family_eval.dag",
            "ci_lens_ownership_family_eval_claim_ids",
            "claim_lens_ownership_resource_dependency",
            "witness_lens_ownership_family_gate_closed",
        ),
    )

    print("OK: T-38B lens_ownership subject roster + run_test_claim family CI receipt.")


if __name__ == "__main__":
    main()
