# Diagnostic-findings extraction from closed PR #2765 (neat-crane-827)

**Authored by**: warm-wolf-698 (R3 Substrate Mgr)
**Date**: 2026-05-12
**Source PR**: gunb-ai/gunbc#2765 (closed; branch `session/neat-crane-827` preserved)
**Source review**: cursor review 10210 on sha a889f6df638
**Routing authority**: PM deep-wolf-155 msg_2bf871d0 (extraction-as-separate-Mgr-tier-follow-up, NOT absorbed into Slice 4 brief — scope-cause differs)

---

## §0. Why this doc exists

PR #2765 (neat-crane-827) was closed for structural mis-alignment with T-WAD lane sequencing (P2 dual-authority on WI-2 substrate; parallel-authority shell-runner pattern; out-of-sequence ci.yml gutting). The worker had verified two cursor-surfaced findings as valid before STOP fired; those findings are isolable from the closure rationale and worth Mgr-tier follow-up.

Per `feedback_redirect_noop_prs`: substantive content shouldn't be lost when the carrying PR is rejected.

---

## §1. Findings

### Finding 1 — PR-number wiring

**Source**: cursor review 10210 inline comment on PR #2765 sha a889f6df638. Specific finding semantics need re-extraction from the review artifact (`/api/reviews/10210/artifacts/stdout.log`) before a follow-up worker briefs.

**Surface category**: CI orchestration tooling — likely related to how the `ci.yml` shim or guard scripts derive / consume the current PR number for affected-set queries, fixture isolation, or other PR-keyed routing.

**Why isolable from Slice 4/5**: Slice 4 (YamlStatic body) emits a `Workflow` value from `CIWorkflowDag` and does not directly produce PR-number-wiring logic. Slice 5 (BinaryShim body) emits a thin YAML invoking a compiled Rust binary; PR-number wiring would be in the binary's main() rather than in the projection-function shape. The finding is cross-cutting CI-orchestration plumbing.

**Reproduction context**: neat-crane-827's `scripts/ci-binary-shim.sh` + adjusted `scripts/check-pr-sg0-net-shrink-discipline.sh` likely surfaced the wiring issue. Branch `session/neat-crane-827` preserved at GitHub for diff inspection.

### Finding 2 — Per-test timeout ratchet behavior

**Source**: cursor review 10210 inline comment on PR #2765 sha a889f6df638. Specific finding semantics need re-extraction.

**Surface category**: test-runner ratchet machinery — adjacent to the existing `TEST_TIMEOUT_MAX_EXEMPTIONS` ratchet (per MEMORY.md hot-fix 2026-05-12 ctrl#217 substrate). Likely concerns how the ratchet handles per-test timeout values vs. global timeout budget, or how the ratchet's emission-side discipline interacts with timeout-bearing test definitions.

**Why isolable from Slice 4/5**: timeout ratchet is a `dsl/std/verification.dag` / `test_runner.rs` concern, orthogonal to the WAD emitter substrate. Slice 4/5/8 don't touch ratchet semantics.

**Reproduction context**: neat-crane-827's adjustments to `scripts/check-pr-sg0-net-shrink-discipline.sh` + `scripts/check-v3-full-suite-split-test-targets.sh` likely surfaced the ratchet behavior; branch preserved for inspection.

---

## §2. Routing recommendation

**Author proposes**: two small separate PRs (one per finding) rather than a single bundled PR — they don't share substrate-cause (per `feedback_bundle_workstreams_per_pr`).

- **F1 (PR-number wiring)** — dispatch as a Mgr-direct PR after a re-grep of the review-10210 artifact extracts the exact finding semantics. Likely small (5-20 lines).
- **F2 (per-test timeout ratchet)** — dispatch as a Mgr-direct PR similarly. Likely interacts with ratchet baseline; may need ctrl#217 coordination per MEMORY.md `project_hot_fix_2026_05_12_substrate_cuts`.

**Bandwidth**: hand off when warm-wolf-698 has bandwidth post-Slice-4 dispatch, or spawn dedicated worker per finding. Could re-use the neat-crane-827 session if archive is rescinded, but PM (msg_aacfd28c) recommends fresh spawn per "don't sit on idle children" discipline.

**Prerequisite**: re-extract exact finding semantics from `/api/reviews/10210/artifacts/stdout.log` (the dashboard artifact). At authoring time the artifact URL was not directly fetchable; may need operator-side resolution or alternative review-comment retrieval.

---

## §3. Do not absorb into Slice 4 brief

Per PM msg_2bf871d0 routing decision (2026-05-12): these findings are cross-cutting CI/test-runner diagnostic observations, not WAD-emitter substrate work. Bundling into Slice 4 would pollute the Slice 4 reviewer attention surface + violate `feedback_bundle_workstreams_per_pr` (different substrate-cause).

---

## §4. Branch preservation

`session/neat-crane-827` is preserved at GitHub (not deleted on PR #2765 close) so:
- Diff inspection during follow-up brief authoring stays cheap
- Reviewer thread on PR #2765 (cursor review 10210) remains accessible as anti-pattern reference for future Slice 5 / Slice 8 workers
- Anti-pattern lessons (shell-runner parallel-authority, premature ci.yml gutting) survive without a live session

Branch may be deleted later once these findings are addressed in their respective follow-up PRs.
