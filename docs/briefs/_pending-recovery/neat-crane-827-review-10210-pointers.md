# PENDING RECOVERY — Review-10210 comment pointers from closed PR #2765 (neat-crane-827)

> **⚠ NOT A DISPATCHABLE BRIEF.** This file lives in `docs/briefs/_pending-recovery/` because its content is unverified review-comment pointers awaiting source-evidence recovery (per `INVARIANTS.md` P1 "Documentation Describes Live State" — addressed by codex APPROVE_WITH_COMMENTS review 10266 on PR #2768). Do not author worker PRs against this file in its current state. See §2 for the recovery prerequisite that must complete before this content becomes dispatchable; once recovered + verified, the file may be promoted out of `_pending-recovery/` with explicit findings.

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

## §1. Pending diagnostic pointers (NOT verified findings)

This doc records **pointers to reviewer comments whose exact semantics have not yet been recovered**. Per `INVARIANTS.md` P1 (Documentation Describes Live State) + codex BLOCKING review (sha 2098603f5): until the source evidence is reconstructed from the review artifact, these are hypotheses, not authoritative findings. Treat them as TODO breadcrumbs for a future extraction step, not as dispatch-ready work items.

### Pointer 1 — "PR-number wiring"

**Source**: cursor review 10210 inline comment on PR #2765 sha a889f6df638. Comment topic header per worker neat-crane-827's status message msg_01dcded6: "REQUEST_CHANGES on PR-number wiring and per-test timeout ratchet behavior".

**Status**: **unverified**. Exact comment text + line citations + concrete change request have **not** been re-extracted from `/api/reviews/10210/artifacts/stdout.log`. The artifact URL was not directly fetchable from this session at authoring time.

**What is known**:
- neat-crane-827 verified the finding as "valid" before STOP fired (per msg_01dcded6), so SOMETHING substantive surfaced
- The topic header references CI orchestration tooling but the actual semantics — which file, which line, which behavior change — are unknown
- Branch `session/neat-crane-827` is preserved at GitHub and may carry the diff that triggered the review comment

**What is NOT known** (do not assert these — they need recovery):
- The specific file / function / line the review identified
- The exact concrete fix the reviewer proposed (if any)
- Whether the issue is in neat-crane-827's added scripts or in existing code they touched

### Pointer 2 — "Per-test timeout ratchet behavior"

**Source**: same cursor review 10210, same status-message topic header.

**Status**: **unverified**. Same recovery prerequisite as Pointer 1.

**What is known**:
- Topic header references test-runner ratchet machinery (adjacent to `TEST_TIMEOUT_MAX_EXEMPTIONS` per MEMORY.md hot-fix 2026-05-12 ctrl#217 substrate)
- Worker verified as "valid" before STOP

**What is NOT known**:
- Whether the comment concerns ratchet emission-side discipline, ratchet consumption-side, ratchet baseline drift, or some other axis
- The specific change the reviewer wanted
- Whether the concern is orthogonal to T-WAD scope or load-bearing on it

---

## §2. Required first step before any routing

**Recover the exact comment text** from `/api/reviews/10210/artifacts/stdout.log` (or via direct PR-review enumeration on PR #2765's reviews list). Until that step completes, **no follow-up PR can be authored against these pointers** — there is no grounded source to write acceptance criteria from.

Options for recovery:
- Operator-side resolution of the artifact URL fetch (out-of-band)
- Direct GitHub API enumeration of PR #2765's review comments
- Re-spawning a session with explicit dashboard-artifact read access

**Do not pre-size or pre-route** the resulting work until the source evidence is in hand. The earlier draft of this doc speculated on size ("likely 5-20 lines") and dispatch shape — those guesses are removed per `INVARIANTS.md` P1.

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
