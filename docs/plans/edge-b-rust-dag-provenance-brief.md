# Edge-(b) scoped dispatch brief — rust-test ↔ consumed-.dag provenance (the coverage keystone)

**Status: SCOPING ONLY.** The multi-day build holds for the operator's explicit greenlight. This document is the decision artifact for that greenlight call.

## Goal

Make the rust↔.dag dependency a **declared structural fact**, recorded once and read in two directions. It is the single fact under three consumers (DESIGN §4, one grammar both directions), which is why this is the §1 *coverage keystone*, not a cost follow-up.

## Why — one fact, three consumers

1. **Coverage wall (soundness, fail-closed).** Today a pure-.dag PR fires *no* rust gate: `any_rust_monolith` (`dsl/tools/rust_stage0_gates.dag`) matches only `.rs`/Cargo/toolchain suffixes. But rust tests *read* `.dag` via opaque `filesystem_read`/`ws.join`, so a `.dag` change that breaks a rust test ships **silently green** — a §5 fail-OPEN. That file's own COMPLETENESS CLAIM ("every change that can alter a rust gate's verdict carries a `.rs`/Cargo suffix") is *false* for .dag-consuming tests, and its header names per-file rust→.dag dependence as the deliberately-skipped proper model. Edge-(b) FIRES the rust gate when a changed `.dag` is in a rust test's declared closure → makes the claim true. This is the operator's "lock fail-closed before you expand" priority.
2. **Affordability selector.** The *same* declared closure read the other way — skip rust tests whose closure the diff doesn't touch — is the affected-set shrink (the ROADMAP end-state: run-all baseline SHRUNK to affected). cargo has *zero* affected-test selection (every `#[test]` every run; nextest too) and can't see `.dag` deps, so a declared closure is the *only* path to it.
3. **Testgen reflection.** The rust-test→.dag closure is the same provenance testgen needs to regenerate the right tests when a `.dag` node changes.

FIRE-when-consumed and SKIP-when-unaffected are the single fact read in opposite directions — that unification is the moat (one carrier, three consumers).

## The asymmetry that decides what the build buys (construction, not validation)

The two directions have **asymmetric robustness to an incomplete declaration**, and this is the construction-vs-validation lesson made concrete:

- **Affordability (skip-unaffected) is fail-SAFE to over-declaration.** Declare too much → you just run more → never unsound. Robust to a sloppy/partial closure.
- **Coverage (fire-on-change) is fail-OPEN to UNDER-declaration.** If a test reads `.dag` A and B but its closure declares only A, a change to B does NOT fire the gate → the exact `.dag`→rust fail-open we are closing, silently re-introduced as **declaration drift**. The coverage wall is only as sound as the declaration is COMPLETE: **closure ⊇ actual-.dag-reads**.

Consequence: the coverage wall's soundness lives entirely in the *completeness* of the closure, which means a **hand-authored closure is the §3/§5 fork** — for the coverage consumer it silently re-opens the hole. So the closure MUST be structurally derived (drift-proof); any hand-authored interim must be marked **NON-SOUND** (delivers affordability only, not the coverage wall). The "shared blocker" below is therefore not the tail of the work — it is *where the soundness lives*.

## Bounded core

1. **Declare rust-test → consumed-.dag-paths on the EXISTING `NodeArtifactProvenance` / `EditLocusBinding` carrier** — no parallel mint (§3 single authority). Implementer must first verify the carrier shape fits `rust-test-identity → .dag-path list`; if not, add a *sibling entry on the same authority*, never a new module.
2. **Gate integration is small and precise:** extend `should_run_gates` / `any_rust_monolith` (`dsl/tools/rust_stage0_gates.dag`) so a changed `.dag` path in the union of declared rust-test closures ALSO returns true.
3. **Fail-closed default AT THE CONSUMER:** a rust test with no declared closure → *must-run*, never skip. Absent provenance = run. This keeps the soundness direction safe while declarations are added incrementally.
4. **Completeness-lens gated — on CORRECTNESS, not presence.** A lens that merely asserts "every rust test HAS a closure declaration" checks *presence*, which is exactly the §5 trap: it is satisfiable by editing the declaration while the realization diverges (the faked-cache-key / dead-field pattern). The lens must check **closure ⊇ actual-.dag-reads** — otherwise a partial or stale declaration passes the lens AND leaks coverage. Presence-only is not enough; correctness (the superset relation) is the soundness condition.

## Shared blocker (testgen-reflection) — this IS the soundness, not optional polish

Per the asymmetry above: the `.dag`-closure must be DISCOVERED structurally, not hand-listed (hand-listing is the §3/§5 fork that rots, and for the coverage consumer it *silently re-opens the hole*). The reflection infra exists *partially* — `v2.std.node_query` + `dependency_lens` + the `affected_set` node-frontier (compile-time, structural over Node/DependencyView). The missing piece: rust tests reach `.dag` through *opaque host paths* (`ws.join`/`filesystem_read`), so the closure isn't visible to the Node-tree lens. Surfacing each rust test's `.dag` reads as a declared, lens-visible dependency is the shared blocker all three consumers wait on — and because the coverage wall is only sound when `closure ⊇ actual-reads`, this structural discovery is the **load-bearing core**, not the tail. It is the hard part and the multi-day driver precisely because it is where the soundness is bought.

## First vertical slice (smallest end-to-end, green-by-execution)

- Pick ONE rust test with a clear `.dag` closure — `coproduct_reflection_conformance_test` (inputs entirely `.dag`) is the natural first.
- Declare its consumed-.dag closure on the carrier.
- Wire `should_run_gates` to fire when a changed path is in THAT test's declared closure.
- Prove by execution with a discriminating control: a diff to a `.dag` IN the closure FIRES the gate; a diff to a `.dag` OUTSIDE the closure does NOT. Green = both directions observed.
- This proves the structural fact end-to-end on one test before any corpus fan-out.
- **State explicitly which the slice proves.** For the slice to prove the *coverage direction soundly*, `coproduct_reflection_conformance_test`'s closure must be **structurally derived** (so a newly-added `.dag` read is captured automatically). If slice-1 hand-lists the closure as an interim, it proves the *wiring* (fire/skip plumbing) but NOT the *wall* (drift-proof completeness) — and must be labelled accordingly. The keystone's value is the wall, so the slice should target the derived closure, not the hand-listed one.

## Honest estimate

Multi-day.

- Slice 1 (one test: declare + gate-fire + discriminating control): ~1 day.
- Shared blocker (structural .dag-closure discovery through opaque host reads, lens-visible): the bulk — ~2–4 days, depending on whether the host-read seam can surface declarations cheaply or needs the reflection layer deepened.
- Corpus fan-out + completeness lens: after.

## Decoupling

Independent of #5427. #5427 is the cheap `.rs`-hole-closer (widen the rust gate's test command on `.rs`-touching PRs) — lands now, behind the two operator levers. Edge-(b) is the structural keystone that later makes the rust gate BOTH sound on `.dag` changes AND affordable by selection. Neither blocks the other.

## Dissolution trigger (DESIGN §6)

Delete this brief once edge-(b) is built and merged — the rust-test→consumed-.dag closure is a structurally-derived (drift-proof) declared fact on the NodeArtifactProvenance carrier, the coverage wall fires the rust gate on an in-closure .dag change (closure ⊇ actual-reads, lens-checked on correctness not presence), and the affordability selector skips unaffected rust tests — so the scoping decision this doc records is realized.
