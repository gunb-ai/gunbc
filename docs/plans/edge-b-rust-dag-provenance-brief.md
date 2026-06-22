# Edge-(b) scoped dispatch brief — rust-test ↔ consumed-.dag provenance (the coverage keystone)

**Status: SCOPING ONLY.** The multi-day build holds for the operator's explicit greenlight.
This document is the decision artifact for that greenlight call.

## Goal

Make the rust↔.dag dependency a **declared structural fact**, recorded once and read in two
directions. It is the single fact under three consumers (DESIGN §4, one grammar both directions),
which is why this is the §1 *coverage keystone*, not a cost follow-up.

## Why — one fact, three consumers

1. **Coverage wall (soundness, fail-closed).** Today a pure-.dag PR fires *no* rust gate:
   `any_rust_monolith` (`dsl/tools/rust_stage0_gates.dag`) matches only `.rs`/Cargo/toolchain
   suffixes. But rust tests *read* `.dag` via opaque `filesystem_read`/`ws.join`, so a `.dag`
   change that breaks a rust test ships **silently green** — a §5 fail-OPEN. That file's own
   COMPLETENESS CLAIM ("every change that can alter a rust gate's verdict carries a `.rs`/Cargo
   suffix") is *false* for .dag-consuming tests, and its header names per-file rust→.dag
   dependence as the deliberately-skipped proper model. Edge-(b) FIRES the rust gate when a
   changed `.dag` is in a rust test's declared closure → makes the claim true. This is the
   operator's "lock fail-closed before you expand" priority.

2. **Affordability selector.** The *same* declared closure read the other way — skip rust tests
   whose closure the diff doesn't touch — is the affected-set shrink (the ROADMAP end-state:
   run-all baseline SHRUNK to affected). cargo has *zero* affected-test selection (every `#[test]`
   every run; nextest too) and can't see `.dag` deps, so a declared closure is the *only* path to it.

3. **Testgen reflection.** The rust-test→.dag closure is the same provenance testgen needs to
   regenerate the right tests when a `.dag` node changes.

FIRE-when-consumed and SKIP-when-unaffected are the single fact read in opposite directions —
that unification is the moat (one carrier, three consumers).

## Bounded core

1. **Declare rust-test → consumed-.dag-paths on the EXISTING `NodeArtifactProvenance` /
   `EditLocusBinding` carrier** — no parallel mint (§3 single authority). Implementer must first
   verify the carrier shape fits `rust-test-identity → .dag-path list`; if not, add a *sibling
   entry on the same authority*, never a new module.
2. **Gate integration is small and precise:** extend `should_run_gates` / `any_rust_monolith`
   (`dsl/tools/rust_stage0_gates.dag`) so a changed `.dag` path in the union of declared rust-test
   closures ALSO returns true.
3. **Fail-closed default AT THE CONSUMER:** a rust test with no declared closure → *must-run*,
   never skip. Absent provenance = run. This keeps the soundness direction safe while declarations
   are added incrementally.
4. **Completeness-lens gated:** a lens asserts every rust test has a closure declaration (or sits
   on an explicit must-run list), so a new undeclared test can neither silently fall into the
   always-run bucket (eroding the affordability win) nor be silently skipped.

## Shared blocker (testgen-reflection)

The `.dag`-closure must be DISCOVERED structurally, not hand-listed (hand-listing is the §3/§5
fork that rots). The reflection infra exists *partially* — `v2.std.node_query` + `dependency_lens`
+ the `affected_set` node-frontier (compile-time, structural over Node/DependencyView). The missing
piece: rust tests reach `.dag` through *opaque host paths* (`ws.join`/`filesystem_read`), so the
closure isn't visible to the Node-tree lens. Surfacing each rust test's `.dag` reads as a declared,
lens-visible dependency is the shared blocker all three consumers wait on — the hard part and the
multi-day driver.

## First vertical slice (smallest end-to-end, green-by-execution)

- Pick ONE rust test with a clear `.dag` closure — `coproduct_reflection_conformance_test`
  (inputs entirely `.dag`) is the natural first.
- Declare its consumed-.dag closure on the carrier.
- Wire `should_run_gates` to fire when a changed path is in THAT test's declared closure.
- Prove by execution with a discriminating control: a diff to a `.dag` IN the closure FIRES the
  gate; a diff to a `.dag` OUTSIDE the closure does NOT. Green = both directions observed.
- This proves the structural fact end-to-end on one test before any corpus fan-out.

## Honest estimate

Multi-day.
- Slice 1 (one test: declare + gate-fire + discriminating control): ~1 day.
- Shared blocker (structural .dag-closure discovery through opaque host reads, lens-visible):
  the bulk — ~2–4 days, depending on whether the host-read seam can surface declarations cheaply
  or needs the reflection layer deepened.
- Corpus fan-out + completeness lens: after.

## Decoupling

Independent of #5427. #5427 is the cheap `.rs`-hole-closer (widen the rust gate's test command on
`.rs`-touching PRs) — lands now, behind the two operator levers. Edge-(b) is the structural
keystone that later makes the rust gate BOTH sound on `.dag` changes AND affordable by selection.
Neither blocks the other.
