# B2 — `lower_fn_body_into_existing_decl` defensive Arrow re-derive → diagnostic + seed-phase root cause `(S, Tier 0)`

> **Worker brief.** Reports through Director (`zesty-bear-812`).
> Tier 0 fail-closed P3 fix per
> [`docs/briefs/debt-paydown-synthesis-2026-04-25.md`](debt-paydown-synthesis-2026-04-25.md) §3.
> Independent of B1/B3; dispatch in parallel.

## Read first

- **[`src/v3/compiler/src/lower.rs:3585-3611`](../../src/v3/compiler/src/lower.rs)** — the site. The `match &dag.declaration(fn_decl_id).connective` arm reads the seeded `Arrow`; the `_ =>` defensive fallback re-derives parameter declarations + refinements when the connective is *not* an `Arrow`. Inline comment: *"Defensive fallback — should not occur because `seed_function_signatures_phase` runs unconditionally on every `Fn` item before bodies are lowered. If the connective is not an Arrow here, the seed phase was skipped or the declaration was clobbered; re-derive to keep the body-lowering pass going and surface the real issue upstream."*
- **[`src/v3/compiler/src/lower.rs:214`](../../src/v3/compiler/src/lower.rs)** — `seed_function_signatures_phase` invocation; runs unconditionally on every module pass.
- **[`src/v3/compiler/src/lower.rs:429`](../../src/v3/compiler/src/lower.rs)** — `seed_function_signatures_phase` definition. Verify: every `Fn` item gets an `Arrow` connective seeded. If a path skips it, that's the root cause.
- **[`src/v3/compiler/src/lower.rs:1697`](../../src/v3/compiler/src/lower.rs)** + **[`src/v3/compiler/src/lower.rs:2358`](../../src/v3/compiler/src/lower.rs)** — sibling sites that reference the seed phase as authority. Confirms the structural invariant the defensive fallback hedges against.
- **`feedback_construction_over_ratchets`** — *"Ratchets are an obtuse form of validation — the system already produced the wrong output and the ratchet notices retroactively."* The defensive fallback is exactly that shape.
- **`feedback_fail_closed_discipline`** (C-8) + **`feedback_compile_time_errors`**.

## Frame

The current code admits two states for a function declaration at body-lowering time:
1. The seeded `Arrow` connective is present (intended path).
2. The connective is something else; re-derive parameters from scratch.

State (2) is documented as *"should not occur"* — it's a hedge against a bug elsewhere (seed phase skipped or declaration clobbered). The hedge masks the bug: when state (2) fires, lowering proceeds with re-derived parameters that **do not match** the seeded ones used by other phases (DB-11 refinement carriers, callsite resolution, etc.), producing structurally inconsistent DAGs that pass downstream checks until a much later phase blows up opaquely.

Per `feedback_state_space_vs_behavioral_invariants` + `feedback_construction_over_ratchets`: the right shape is to **delete state (2) and fail closed** with a `Diagnostic`, then **fix the root cause** in seed phase. If seed phase already runs unconditionally on every `Fn`, the only way to reach state (2) is a bug — the diagnostic surfaces it; the seed-phase fix prevents it.

## Slice — two coupled deliverables

### Deliverable A — replace the defensive fallback with a fail-closed diagnostic

- At `lower.rs:3585-3611`, replace the `_ =>` arm with a `Diagnostic` (per C-8 / `feedback_fail_closed_discipline`). The diagnostic should name the fn declaration, its observed connective, and the invariant violated (*"seed_function_signatures_phase did not produce an Arrow connective for this Fn"*).
- Use the existing diagnostic-emission path; don't introduce parallel error machinery.

### Deliverable B — root-cause the seed-phase gap (if any)

- Audit `seed_function_signatures_phase` (`lower.rs:429`) and every `Fn` item enumeration site. Verify: every `Fn` item that reaches `lower_fn_body_into_existing_decl` has had its declaration seeded with an `Arrow`.
- If the audit finds **no gap** (i.e., seed phase is already total over `Fn` items), Deliverable A alone is sufficient. The diagnostic becomes a structural assertion that converts the unreachable hedge into a fail-closed contract.
- If the audit finds **a gap** (a path where `Fn` items reach body-lowering without seeding), fix the seed-phase invocation to cover it. The fix must be at the seed phase, **not** at the body-lowering site (per `feedback_construction_over_ratchets`).

## Acceptance

- [ ] `lower.rs:3585-3611` defensive fallback removed; diagnostic-emit path in place.
- [ ] Seed-phase audit recorded in PR description: every `Fn`-item path verified to seed before body-lowering, OR root-cause fix landed.
- [ ] Existing tests pass; no test currently relies on the silent-rederive path (if any do, that's a test-discipline fix, not a brief expansion — surface in PR).
- [ ] `cargo test --workspace --exclude v2-compiler-tests` passes.
- [ ] `cargo clippy --all-targets -- -D warnings` clean.
- [ ] `cargo fmt --all --check` clean.
- [ ] DB-8 fixed-point converges bit-identically.

## STOP-AND-ESCALATE

- **Audit reveals a legitimate runtime path that produces non-`Arrow` connectives at this site** (e.g., a refinement layer or builtin `Fn`-but-not-`Arrow` case) — STOP. The framing of "should not occur" is wrong; brief needs revision.
- **Diagnostic-emission requires substrate work beyond C-8's existing shape** (e.g., a new diagnostic class) — STOP. Coordinate diagnostic taxonomy with Surface Manager.
- **Removing the fallback breaks v2-compiler-tests** (rather than v3 tests) — STOP. `v2-compiler-tests` may exercise the legacy hedge intentionally; surface for design call.
- **DB-8 drifts** — STOP immediately.

## Non-goals

- Not refactoring `seed_function_signatures_phase` beyond the minimum needed to close the gap (if any).
- Not touching DB-11 refinement carriers (`lower_parameter_refinement`) — they're consumed correctly by both branches; the issue is which branch fires.
- Not part of the §0 identity-carrier class (B4).

## Reporting

Single PR. Title: `fix(v3): B2 lower_fn_body Arrow re-derive → fail-closed diagnostic + seed-phase root cause (Tier 0)`. Body cites this brief + records the seed-phase audit outcome.

On merge: signal Director.
