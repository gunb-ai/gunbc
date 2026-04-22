# CI ratchet architecture audit `(M)`

## Problem

The 2s-per-test CI ratchet has been eroding by exemption widening rather than erosion detection. Recent commits (`37cd6128`, `4898983e`, `f84ed355`, `2d8396df`) together made the ratchet more robust against CI-log instability **and** widened the exemption list. Individual PRs looked fine locally; the aggregate ratchet is weakening.

The specific failure mode: exemptions accumulate one test at a time, each justified in its own PR, with no enforcement that the list ever shrinks. `scripts/slow-test-exemptions.txt` is 83 lines as of this writing with no monotonic-shrink ratchet. An exempt test can stay slow indefinitely; new exempt tests can be added indefinitely. The ratchet's job — driving test-suite latency toward a floor — is weakened whenever an exemption becomes effectively permanent.

This is a **feedback_ratchet_only_down** issue: the meta-ratchet (exemption count + exempt-test budget) isn't ratcheted itself, so the primary ratchet (2s per test) can drift upward through exemption growth.

## Read first

- `scripts/check-test-timeout.sh` — the primary ratchet script
- `scripts/slow-test-exemptions.txt` — current exemption list (83 lines as of 2026-04-21)
- `scripts/l1-ratchet.sh` — sibling ratchet for context on existing patterns
- `TESTING.md` § test layers — the 2s-per-test authority
- `feedback_test_timeout_2s` memory — tests >2s are broken/hanging
- `feedback_ratchet_only_down` memory — never increase a ratchet; if a fix needs a new violation, it belongs deeper
- Recent commits that widened exemptions vs tightened ratchet:
  - `37cd6128`, `4898983e`, `f84ed355` — CI-log instability robustness
  - `2d8396df` — exemption widening
  - The reflective-analysis diff range `53b3110..ae8825a` surfaced this as a systems-level concern

## Work

**Phase 1: Audit the current exemption list.**

1. For each line in `slow-test-exemptions.txt`, answer:
   - **Why was it exempt originally?** (grep the commit that added it; read the PR body)
   - **Is the exemption still needed?** Run the test locally, record current duration. If it's under 2s today, the exemption is stale — remove in this PR.
   - **Category**: (a) paydown backlog (slow-but-will-be-fixed), (b) structurally necessary (e.g., genuinely requires bootstrap+compile), (c) stale (under budget today), (d) unknown (needs owner assignment)
2. Produce `docs/debt/ci-ratchet-exemption-audit-2026-04-XX.md` listing every exemption with its category, current duration, and (for paydown-backlog entries) a specific fix path.

**Phase 2: Define monotonic-shrink rules.**

1. **Exemption count cap.** Add a meta-ratchet: `scripts/check-test-timeout.sh` fails if `slow-test-exemptions.txt` grows beyond its current count. New exemptions require paired removals (ratchet-only-down applied to the meta).
2. **Per-exempt budget ratchet.** For each exempt test, record its *current* duration in the exemption file (as a comment alongside the test name). Subsequent runs fail CI if the duration grows beyond the recorded budget + a small slack (say 10%). Exempt tests can't get slower indefinitely.
3. **Exemption-age reminder.** Print a warning if any exemption is older than N days (configurable, say 90). Not a hard fail; surfaces staleness for review.

**Phase 3: Apply findings.**

1. Delete stale exemptions from Phase 1.
2. For paydown-backlog entries, open one tracking debt row per entry in ROADMAP with the specific fix path surfaced in Phase 1.
3. Confirm the new meta-ratchet fires on drift (add a CI test or explicit regression case).

## Acceptance

- `docs/debt/ci-ratchet-exemption-audit-YYYY-MM-DD.md` exists with every exemption categorized
- Stale exemptions (category c) deleted in this PR
- Meta-ratchet in place: exemption count can only shrink without explicit authorization; per-exempt durations have a recorded budget that CI enforces
- ROADMAP debt rows exist for each paydown-backlog entry with a specific fix path
- CI passes on the current test suite
- PR body declares how many stale exemptions were removed and what the new exemption count floor is

## STOP-AND-ESCALATE

- **If the audit reveals most exemptions are structurally necessary** (category b dominates), STOP. That means the 2s-per-test budget may be wrong for the current test architecture, not that the exemptions are drift. Surface the mismatch rather than force monotonic shrink on genuinely-needed exemptions.
- **If adding the per-exempt budget to the file format breaks the parser in ways that are invasive**, STOP. The file format is simple and shouldn't need redesign for this lane — if it does, that's a separate refactor.
- **If any exemption's paydown path requires substrate changes** (not just a test refactor), STOP. That's a modeling lane, not a CI-hygiene lane. Surface and keep the exemption in place with the tracked debt pointer.
- **If the commits cited (`37cd6128`, `4898983e`, `f84ed355`, `2d8396df`) turn out to have already-tracked rationale** that was missed in the reflective analysis, STOP. Confirm the analysis is surfacing new drift, not re-surfacing old decisions.

## Non-goals

- Not rewriting the 2s-per-test authority from TESTING.md
- Not refactoring test architecture broadly — this is exemption-list hygiene, not test-suite redesign
- Not extending ratchet coverage to other CI surfaces (fmt, clippy) — just the per-test timeout ratchet and its exemption list
- Not auditing `l1-ratchet.sh` separately — that's its own lane if needed

## Size

M. Phase 1 audit is the bulk (~half a day of running each exempt test + categorization). Phase 2 meta-ratchet is a small script edit + a CI check. Phase 3 cleanup is mechanical. Total ~1 engineer-day.

## Dispatch note

This lane pays infrastructure principal. The primary artifact is the audit doc (Phase 1) — it names which exemptions are stale, which are paydown, which are structural. That answer informs every future test-performance decision. The meta-ratchet (Phase 2) is the enforcement that keeps the answer accurate going forward.

Director reviews the audit findings specifically — are stale exemptions really stale? Are structural exemptions really structural? If the split looks wrong, the meta-ratchet gets the wrong floor.

After this lands, the exemption list becomes trustworthy as a snapshot of real test-architecture debt rather than an accumulated blame-free TODO pile.
