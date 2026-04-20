# Lane 1e — Single-walker emit collapse `(XXL)`

## Context

Stage 1c closed with the clean-emission contract proven on three targets (Rust, Go, Python — PRs #514, #520, #532). The structural claim — "emit = target spec + one walker" — is now falsifiable. But three parallel per-target emitter files still carry the actual implementation:

- `src/v3/compiler/src/emit.rs` (~2700 LOC)
- `src/v3/compiler/src/emit/rust_target.rs` (~5500 LOC)
- `src/v3/compiler/src/emit/python_target.rs` (~2000 LOC)

That's ~10K LOC of hand-authored Rust doing the same thing three times, differing only in target-language details that already live in `src/v3/spec/rust.dag`, `src/v3/spec/go.dag`, `src/v3/spec/python.dag`.

**The thesis claim requires dissolving them.** Adding a new target (TypeScript, Swift, etc.) should cost one `.dag` spec file, zero per-target Rust. Today it costs ~2K LOC of Rust.

## Read first

- `docs/single-emitter-design.md` — the canonical design for this lane. Read in full before proposing changes.
- `docs/emit-bridges.md` — catalog of named bridges (Rust-specific fold emission, Python integer dispatch, etc.) that the single walker must subsume or name as residual per-target extension points.
- `src/v3/spec/rust.dag`, `src/v3/spec/go.dag`, `src/v3/spec/python.dag` — target spec declarations as they exist today. Determines what spec rows already cover vs. what the walker would need new rows for.
- `src/v3/compiler/src/emit.rs` — the current shared-core emit entry point; its structure names the walk shape.
- `src/v3/compiler/src/emit/rust_target.rs` — the biggest per-target file; structural patterns here are the acid test for the generic walker's expressiveness.
- `src/v3/std/clean_emission.dag` — the structural contract that binds all three targets to the same shape.
- `src/v3/std/emit_model.dag` — the ParameterDisposition / shared carriers that already unify across targets.

## Work

Not one PR. This is an XXL multi-stage lane. Propose a staged plan in the first PR's body; each stage ships its own PR.

**Phase 1 — Audit + spec gap inventory (~1 PR).**
Walk `emit.rs` + `emit/rust_target.rs` + `emit/python_target.rs`. For every place the code branches on target or calls a target-specific helper, classify:
- Already covered by `spec/*.dag` (consumable via typed spec row) — good, walker can consume
- Should be a spec row but isn't — add to the gap inventory; each entry becomes a spec extension task
- Legitimately per-target (rare — e.g., rustc-invocation path, python import semantics) — name the narrow residual

Output: `docs/emit-target-spec-gaps.md` enumerating every gap with a proposed spec-row shape, owner, and sequence. This is the planning artifact for Phase 2+.

**Phase 2 — Extend `spec/*.dag` to close the gap (1-N PRs).**
Add the spec rows identified in Phase 1. Each PR: one coherent bridge cluster (e.g., "function rendering — arg list, return type, body — for all three targets"). Don't author the walker consumer yet; just land the data.

**Phase 3 — Single walker against extended spec (~1-2 PRs).**
Author `src/v3/compiler/src/emit/walker.rs` (or similar) consuming the extended specs. Walker reads the DAG + a `TargetSpec` handle, emits target text. No per-target branches in the walker body.

**Phase 4 — Cutover + deletion (1 PR).**
Replace all call sites to the old per-target files with the new walker + TargetSpec dispatch. Delete `emit/rust_target.rs`, `emit/python_target.rs`, and whatever portion of `emit.rs` is now subsumed (keep only the entry point + any genuinely per-target residuals from Phase 1). Update SG-0 census, `compiler.dag::hand_maintained_src`, and the fixed-point-snapshot guards.

**Phase 5 — Regression + roundtrip (~1 PR).**
Verify `m1_3_emit_rust_test`, `m1_3_emit_go_test`, `m1_4_emit_python_test` all pass against the new walker. Run the post-emit-verifier gate + the four-fixture determinism matrix. Golden outputs match bit-identically.

## Acceptance

- `src/v3/compiler/src/emit/rust_target.rs` **deleted** (or reduced to a re-export shim under 50 LOC)
- `src/v3/compiler/src/emit/python_target.rs` **deleted** (same criterion)
- A single walker module exists; `cargo test` proves all three target emitters work through it
- `spec/rust.dag`, `spec/go.dag`, `spec/python.dag` carry every bridge the walker needs — the walker body contains zero target-specific `if`/`match` branches beyond dispatching on the passed `TargetSpec`
- Adding a hypothetical TypeScript target is demonstrably one `.dag` file + zero walker changes (prove via a minimal `spec/typescript.dag` stub that at least renders a literal, even if incomplete)
- `m1_3_emit_rust_test`, `m1_3_emit_go_test`, `m1_4_emit_python_test`, four-fixture determinism, post-emit-verifier — all green
- SG-0 census drops by ≥ 2 files; `compiler.dag::hand_maintained_src` updated
- Lane 1e status in ROADMAP flipped from 🟡 to ✅

## STOP-AND-ESCALATE

- **If Phase 1 reveals > 30 distinct bridge gaps** — that's a sign the current spec vocabulary is too narrow. Surface the count; we may want an intermediate `emit_model.dag` extension PR before Phase 2 gets dispatched.
- **If Phase 3's walker needs a target-specific escape hatch** (an arm that checks "if target.name == 'rust' { ... }") — STOP. That's a spec gap that Phase 2 missed. Don't patch it in the walker; go back to Phase 2.
- **If deletion of a per-target file would require significant rustfmt / post-emit-verifier reshaping** — surface the coupled scope; likely wants its own PR.
- **If the bit-identical roundtrip fails on any target after cutover** — STOP. The walker's output must match the pre-Lane-1e bytes (or the golden files update is justified and documented). Drift is a regression, not a cutover cost.

## Non-goals

- **Not dissolving `emit.rs` entirely** — it's the entry point; the walker is called from it. Keep it as a thin dispatch shell if residual per-target logic survives.
- **Not adding new targets** — the acceptance is "adding a new target is mechanical," not "TypeScript shipped."
- **Not rewriting the clean-emission contract** — that's stable (Stage 1c closed). The walker consumes it.
- **Not touching the post-emit-verifier** — it's the gate, not in the cutover path.
- **Not changing `.dag` surface syntax** — this is a compiler-internal refactor.

## Size

XXL. Multiple PRs over multiple weeks. Phase 1 alone is probably 1-2 weeks of audit. Phase 2-3 is the bulk. Phase 4-5 lands the dissolution.

Expected LOC delta at close: **-8K to -10K** net. Single-worker lane; PRs can serialize or overlap if phases don't conflict.

## Dispatch note

Claude-review (director) reviews each phase PR. Phase 1 gets the most scrutiny (it defines the rest). STOP-AND-ESCALATE on Phase 1 scope surprises before the gap count explodes.
