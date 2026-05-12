# T-WAD Slice 7 — Implementation pre-queue (harness contract + Layer 2 inventory)

**Status:** Verification-lane pre-implementation scaffolding. **Does NOT** satisfy
gate `ci_uses_affected_set_selection` (`docs/r3-structure.md` / program-plan row
103). The gate stays as-was until the **post–Slice 5** integration PR lands.

**Companion canvas:** `docs/design-t-wad-slice-7-binary-shim-affected-set-selection-canvas.md` (PR #2760).
This document is the **queued-behind** scaffolding for the canvas: it captures
the harness contract from the canvas §9 in a form reviewers can grep against
when the implementation PR arrives, and it inventories the **current**
authoritative path-regex selection so the eventual dissolution is mechanically
checkable.

**Authority for scope:** Verification Mgr (clever-tern-670) directive
2026-05-12: pre-author §9 harness skeleton + add **observational** Layer 2
path-regex inventory/ratchet; do **not** implement BinaryShim runtime/substrate;
do **not** alter workflow behavior. If the current tree cannot support a
hermetic planner-stub test without inventing public API, stop at docs/ratchet
inventory rather than fabricating substrate.

**Hard block on Slice 5 (warm-wolf-698):** The canvas §6 names the artifacts
this lane consumes — `project_github_actions(_, BinaryShim)`, the compiled
runner crate, the thin shim YAML, and the public hook where lens evaluation +
selection wire in. None of these exist on `main` at this commit. Per parent
directive **no fabricated substrate** is introduced here.

---

## §1. What this PR ships

1. **Inventory** of `.github/workflows/*.yml` sites that today encode
   **authoritative path-regex selection** for heavy CI compute (§3).
2. **Observational ratchet script** `scripts/check-workflow-path-regex-inventory.sh`
   that pins those sites and fails if either (a) a **new** path-regex
   authoritative selection appears outside the inventory, or (b) an **existing**
   inventoried site is removed **before** a BinaryShim replacement is wired
   (fail-closed — see §4).
3. **Harness contract** (§5) — a written specification of the hermetic test
   shape the implementation PR will land. No Rust types are introduced here
   because the planner's input types (`Receipt`, `RunPlan`, `obligation_metadata`)
   are Slice 5 / lens-substrate authorities; declaring them in this PR would
   violate INVARIANTS P2 single-authority.

## §2. What this PR explicitly does NOT ship

- No `gunbc-ci` binary, no Rust planner module, no public-API types.
- No edits to `.github/workflows/*.yml` content or job graph.
- No invocation of the ratchet script from CI (the script exists as an
  out-of-band tool until Slice 5 is in place to mean anything by passing).
- No claim that gate `ci_uses_affected_set_selection` is closer to PASSING;
  row 103 is unchanged. The §1.8 state-check gate flips only when the
  BinaryShim runner is **wired and consuming** PR #2713 lens output, per
  canvas §1 + §7.

## §3. Layer 2 inventory — authoritative path-regex selection sites

As of this commit (`.github/workflows/ci.yml` content hash captured in the
ratchet script via grep-anchored fingerprints, not file hash, so unrelated
edits don't false-trip):

| # | File | Lines | Selection role | Replacement owner |
|---|------|-------|----------------|-------------------|
| 1 | `.github/workflows/ci.yml` | `changes:` job (around L201–L247) | Computes `outputs.code = true \| false` via `git diff --name-only origin/main...HEAD` filtered by `grep -vE '^(docs/.*\|[^/]+\.md)$'` — i.e., **path-regex** docs-only allowlist. **Layer 1** mitigation per inline comment ("dissolution trigger = affected-set lens CI integration"). | Slice 7 implementation PR (post–Slice 5) deletes the `changes` job and routes selection through the BinaryShim runner consuming PR #2713 lens output. |
| 2 | `.github/workflows/ci.yml` | `v3:` job `if:` (around L253) | `needs.changes.outputs.code == 'true' \|\| github.event_name == 'push'` — the **`if:` gate** that turns inventory #1 from a probe into authoritative selection (skip-when-false on PR events). | Same as #1: the `if:` reduces to draft-PR / event-orthogonal mechanics only; affected-set selection lives in the binary. |

**`if:` not inventoried (event-orthogonal, per canvas §5):**

- `if: github.event.pull_request.draft != true` on jobs `changes`, `cache_warm` (L84), `lint_short` (L54), `v3` (combined with #2 above), and the always-run aggregator at L616.
- These remain valid YAML mechanics after Slice 7. They are **not** authoritative
  for "which tests/gates prove merge safety" — they encode **event** policy
  (draft PR skip) rather than affected-set narrowing. The ratchet script does
  not flag them.

## §4. Ratchet semantics (fail-closed, both directions)

The script `scripts/check-workflow-path-regex-inventory.sh` is a structural
test that exits non-zero in any of:

- **New authoritative selection** appears outside the inventoried sites. The
  script's heuristic is anchored on (a) `git diff --name-only` invocations
  inside any `.github/workflows/*.yml`, and (b) `if:` gates referring to
  `outputs.code`-style booleans derived from path-regex jobs. New occurrences
  → fail with a pointer to this document so the operator can either add to
  inventory + canvas or remove the path.
- **Inventory drift before replacement**: if either fingerprint listed in §3
  vanishes before Slice 7 implementation lands a structurally documented
  BinaryShim runner consuming PR #2713, the script fails. Rationale: silently
  removing the docs-only skip without a replacement would either widen
  unnecessary compute (best case) or, if a hand-edit removes the `if:` while
  keeping the probe, leave authoritative selection in an inconsistent state.

This script is **not** wired into `.github/workflows/ci.yml` by this PR (that
would itself be a CI behavior change; parent directive forbids). It is intended
to be invoked manually by reviewers and by the Slice 7 implementation PR's
test plan as part of the dissolution receipt.

## §5. Hermetic verification harness contract (canvas §9 specialization)

When Slice 5 lands, the implementation PR will own a module (working name
`gunbc-ci-planner`, exact crate path Slice-5-owned) exposing a pure function of
the conceptual shape

    plan_dispatch(receipt: AffectedSetReceipt,
                  obligations: ObligationMetadata) -> RunPlan

where the types are **all** owned by Slice 5 / the PR #2713 lens substrate.
This document does **not** declare them; doing so would create parallel
authority and violate `feedback_import_not_redeclare_carriers`.

Each row below is the contract the implementation PR's test file must satisfy.
Tests are hermetic (each declares its own fixture), behavior-driven (one claim
per test), and unit-first per `TESTING.md`.

### §5.1 Required unit tests (planner)

| Test ID | Behavior under test | Fixture | Expected result |
|---------|---------------------|---------|-----------------|
| `dispatch_unknown_dim_forces_superset` | Receipt has `Unknown` delta on a narrowing edge for at least one asserted dimension of `t ∈ B`. | Synthetic receipt with a single `Unknown` row; obligation `t` with `D(t) = {Cost}`; `B = {t, t2}`. | `RunPlan` includes every member of `B` (superset). |
| `dispatch_missing_exclusion_receipt_includes` | Per-dimension exclusion proof is absent on a candidate-skip obligation. | Receipt that proves narrow for dim D but lacks the exclusion receipt for `t`. | `t` ∈ `RunPlan`. |
| `dispatch_narrow_path_excludes_unaffected` | Receipt proves only subgraph `S` changed; obligation `t` references only nodes outside `S` and declares `D(t) ⊆ {dims with empty proven delta}`. | Receipt with proven empty delta for `D(t)` on `t`'s structural closure. | `t` ∉ `RunPlan`. |
| `dispatch_undeclared_dimensions_force_superset` | `t ∈ B` carries no `D(t)` metadata. | Obligation list with one entry having `D(t) = ∅` / absent. | `t` ∈ `RunPlan` (cannot prove skip-safe). |
| `dispatch_unmapped_job_forces_superset` | Workflow job step is referenced in `B` but lacks a `CIWorkflowDag` / NodeRef mapping. | Synthetic obligation with `mapping = None`. | That obligation ∈ `RunPlan`. |

### §5.2 Required ratchet / static tests

| Test ID | Subject | Behavior |
|---------|---------|----------|
| `workflow_no_path_regex_policy` | Emitted `.github/workflows/ci.yml` (or thin shim once BinaryShim lands) | Asserts no `git diff --name-only`-driven `outputs.code` boolean is consumed by a job `if:`. (Inventory in §3 is the seed; implementation PR's ratchet supersedes the §3 script.) |
| `workflow_affected_set_fail_closed` | Runner entry point | Asserts that when the lens evaluation errors or returns unknown for a dimension required by any `t ∈ B`, the runner emits a structured reason code AND selects superset. Test substitutes a mock lens; no real DAG compile. |

### §5.3 Optional integration test (cost-gated; canvas §9.2 row 4)

Real affected-set lens evaluated on a tiny paired `.dag` fixture; planner output
matches expected run list. Skipped by default until the lens API is stable post
PR #2713 follow-ups. This row is reserved here so the implementation PR cannot
silently drop it.

## §6. Cross-references

- Canvas: `docs/design-t-wad-slice-7-binary-shim-affected-set-selection-canvas.md` (PR #2760)
- Emitter-dispatch parent canvas: `docs/design-ci-workflow-emitter-dispatch.md`
- Affected-set lens spec: `docs/design-affected-set-lens.md`
- FULL R3-close scope: `docs/r3-t-workflow-as-data-full-r3-close-scope.md`
- Program plan row 103 + gate text: `docs/r3-program-plan.md`, `docs/r3-structure.md`
- Inventory ratchet: `scripts/check-workflow-path-regex-inventory.sh`
