# R3 T-FixedPoint Worker Brief (PB)

**Status:** PROPOSAL (planning artifact, dispatch-gated). Authored 2026-04-29 by PB Manager continuation per the Pending entry in [`r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md) §"Sub-briefs (authored / pending)" and the lane row in [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure".

**Owning manager:** Pure Bootstrap Manager (R2 → R3 continuation per `r3-structure.md` §"Manager structure" Item 1).

**Lane size:** M (per `r3-structure.md` lane table).

**This is a planning artifact — not a dispatch order.** Worker dispatch is gated; see §"Dispatch preconditions" + §"STOP conditions". PB Manager re-reads this brief at gate-clear to issue worker dispatch.

## Scope

T-FixedPoint closes the **R3 thesis facet 2 horizon** of the `pb_self_compile_fixed_point` predicate: `compiler.dag` compiled by the v3 binary produces **bit-identical stage0 Rust + bit-identical emitted artifacts** under fixed-point semantics, with the in-tree hand-Rust floor at zero (per Director-locked decision 2026-04-28 in `r3-structure.md` §"Design challenge 4").

The lane delivers:
1. The strong-interpretation `.dag` `TestClaim` `pb_self_compile_fixed_point_strong` (per `r2-pure-bootstrap-manager.md` §"Acceptance" line 101) authored against the existing `FixedPointConverges` substrate variant at `src/v3/std/verification.dag:206` — same `FixedPointConverges` predicate variant, distinct strong claim name (`pb_self_compile_fixed_point_strong`, not the R1 `pb_self_compile_fixed_point`).
2. Verification that running the cycle a second time on the v3-emitted Rust produces byte-identical output (true fixed point, not just "compiles itself once").
3. Closure-ledger signal that R3 thesis facet 2 has landed.

## Two-horizon framing (load-bearing — do not collapse)

Per `r3-structure.md:59` and `r2-structure.md:296`, the predicate name `pb_self_compile_fixed_point` carries **two horizons**:

| Horizon | Acceptance | Where |
|---|---|---|
| **R1 lane gate** | Pass = current `verification.dag` + `test_runner` evaluation under R1 acceptance discipline. Made green at landing by #1050 + #1074. | `src/v3/compiler/tests/integration/r1_release_acceptance_test.rs:18` + `r1c_d_pb_census_gates_test.rs:43` |
| **R3 thesis facet 2** (this lane) | Closes under bit-identical fixed-point + SG-0 choreography per Director cascade 2026-04-28. Strong interpretation. | `pb_self_compile_fixed_point_strong` `.dag` claim authored under this lane |

**R1 close does not wait on R3.** R3's elevated bar is a separate release/thesis acceptance — **not a silent rename of the R1 predicate**. The R1 fixture remains green at its R1 acceptance; this lane authors the strong claim alongside.

**Worker discipline:** never edit the R1 fixture's predicate evaluation to incorporate the strong bar. Add the strong claim as a distinct `TestClaim` (see §"Acceptance gate"), so the R1 horizon stays untouched.

## Dependencies

Per `r3-structure.md` §"Lane structure" + §"Dependency DAG":

1. **R2-Evaluator landed** — runtime executes `compiler.dag`; without the Evaluator the fixed-point cycle has nothing to run. This is the dominant gate (7 of 10 R3 lanes share it).
2. **SG-0 non-test = 0 from T-LensProducer-Retirement** — per `r3-structure.md` §"Design challenge 4" Director-locked decision: T-FixedPoint closes under "SG-0 non-test = 0 + ≤1 first-time-bootstrap trampoline allowed per [`docs/design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md) §`First-time bootstrap`." The trampoline lives **outside** `src/v3/`; the in-tree floor stays 0. The three lens-producer files (`lens_apply.rs`, `lens_testgen.rs`, `regen_lens.rs`) must be retired before this lane closes — they are the load-bearing residual on the SG-0 non-test census.
3. **PB-1 generated bin-shim pattern** (sub-dependency of T-LensProducer-Retirement, transitively) — needed for `regen_lens.rs` retirement to land cleanly so SG-0 reaches zero.

These dependencies are cumulative: R2-Evaluator → T-LensProducer-Retirement (XL, 3 sub-gates) → T-FixedPoint (M).

## Acceptance gate (`.dag`)

Per `r2-pure-bootstrap-manager.md` §"Acceptance" line 101 + `r3-structure.md:60`:

**`pb_self_compile_fixed_point_strong`** — authored as a `.dag` `TestSuite` **composing two existing `TestPredicate` variants** at `src/v3/std/verification.dag`:

1. **`FixedPointConverges { compile_target, expected }`** (line 206-209) — one row per Shape-A artifact in the accepted artifact set. Each row asserts: the v3 binary compiles `compiler.dag` to that target → re-compiling from the emitted output produces **byte-identical** output (per [`docs/design-fixed-point-ratchet.md`](../design-fixed-point-ratchet.md) §Design — byte equality, not semantic diff). DB-8's `self_host_fixed_point` binary is the runtime mechanic; this row is the structural acceptance.
2. **`RatchetZero { authority, ratchet_kind }`** (line 210-213) — single row asserting SG-0 `EXPECTED_HAND_AUTHORED_NON_TEST` census = 0 at evaluation time. Cross-reads the SG-0 census authority (does not duplicate the list).

### Single grounding gate (artifact set derivation)

Per [`docs/r3-structure.md`](../r3-structure.md) §"R3 worker dispatch precondition" (Director-locked 2026-04-28), all 7 Evaluator-gated lanes (including T-FixedPoint) dispatch under the **joint precondition** "R2-Evaluator landed AND R2-Grounding-Rust+Python landed." That sets a hard floor: **{Rust, Python} are both required** before any T-FixedPoint dispatch can occur. The brief does not introduce a Rust-only carve-out.

The accepted artifact set for the `FixedPointConverges` rows is **derived from one grounding fact**: the R2 Release Manager closure ledger's report of "which `R2-Grounding-{Lang}` lanes are closed at the moment of dispatch." Worker reads this ledger once at dispatch — that single reading both (a) gates dispatch (must show Rust + Python both closed per the authority above) AND (b) derives the artifact set (one `FixedPointConverges` row per closed-grounding language at that reading). One fact, two consumers; no parallel lists.

Concretely, **at the earliest dispatch-eligible moment** (Rust+Python floor met):
- Rust + Python closed (no Go) → artifact set = {Rust, Python} → two `FixedPointConverges` rows.
- Rust + Python + Go closed → artifact set = {Rust, Python, Go} → three rows.
- Rust-only (Python pending) → **NOT dispatch-eligible** per `r3-structure.md` authority; PB Manager waits.

The lane closes when every artifact in the dispatch-time set produces the byte-identical cycle. Subsequent `R2-Grounding-Go` closure landing **after** T-FixedPoint closes is addressed by re-evaluating the `TestSuite` against the new ledger reading; if a row regresses, that's a Grounding-lane regression, not a re-opening of T-FixedPoint.

### Substrate readiness check

Both `FixedPointConverges` and `RatchetZero` variants exist on main today (verified at `src/v3/std/verification.dag:206-213`). The strong claim is **substrate-composition**, not substrate-introduction. STOP condition (see §"STOP conditions"): if `TestSuite`-level composition of these two predicates proves structurally insufficient at authoring time (e.g., the runtime cannot evaluate the AND-conjunction across the two predicate kinds, or the SG-0 census authority surface is not addressable from `RatchetZero.authority`), that's a substrate gap → escalate per `INVARIANTS.md` §P1 (signal Substrate Manager; do not extend variants from this lane).

### Relationship to DB-8 ratchet infrastructure

DB-8 already landed the **mechanical** fixed-point ratchet ([`docs/db-history/db-8.md`](../db-history/db-8.md)): `src/v3/compiler/src/bin/self_host_fixed_point.rs` runs the emit → rustc → run → byte-diff cycle, plus `tests/determinism_test.rs` 5× per-fixture determinism check, plus INVARIANTS D-1, plus CI job `self_host_ratchet` (currently `continue-on-error: true` until Lane 1e closes — see DB-8 acceptance checklist). The cycle is **staged on `default_fixed_point_source`**; the full cycle on `dsl/gunbc/compiler.dag` is gated on `compiler.dag` parsing under v3 and emitted output being a re-emittable CLI.

T-FixedPoint **does not** rebuild this infrastructure. The lane delivers two specific transitions on top of the existing ratchet:

1. **Promote target from `default_fixed_point_source` → `dsl/gunbc/compiler.dag`** in `self_host_fixed_point` (the binary already probes `compiler.dag`; promote it from probe to required gate).
2. **Graduate CI `self_host_ratchet` from `continue-on-error: true` → merge-blocking** (DB-8 acceptance checklist names this graduation as gated on Lane 1e clearing iteration debt; T-FixedPoint dispatch precondition #3 — T-LensProducer-Retirement closure — subsumes that prerequisite).
3. Author the `pb_self_compile_fixed_point_strong` `.dag` `TestClaim` so the structural gate lives in `verification.dag` alongside the binary's mechanical check (single authority for the strong horizon's acceptance).

The non-determinism elimination work (HashMap/HashSet/timestamps/paths per `design-fixed-point-ratchet.md` §"Sources of non-determinism") is **not** in T-FixedPoint scope — it's continuing Lane 1e / emit.rs work that surfaces as ratchet failures and routes to whichever lane owns the offending code path.

## Cross-lane sequencing (Shape-A target coverage)

`r3-structure.md` lane table specifies "bit-identical stage0 Rust + bit-identical emitted artifacts" — Rust is the load-bearing target (stage0 is Rust). Python + Go coverage is **derived from the single grounding gate** (see §"Single grounding gate (artifact set derivation)"), not maintained as a parallel target list here. The artifact set IS exactly the set of `R2-Grounding-{Lang}` lanes the closure ledger reports closed at dispatch.

This avoids artificially gating T-FixedPoint on T-Verification-L5-Corpus (cross-target *equivalence* is L5's lane; T-FixedPoint is per-target byte-identity across the self-host cycle).

## Non-goals

T-FixedPoint **does not** own:

1. **The R1 horizon of `pb_self_compile_fixed_point`** — that's R1's; closed at R1 close. Worker must not modify the R1 fixture's predicate evaluation.
2. **Lens-producer retirement work** — that's T-LensProducer-Retirement (PB Manager R3 lane, separate brief). T-FixedPoint **consumes** the SG-0=0 signal; it does not produce it.
3. **Cross-target algebraic equivalence (L5)** — that's T-Verification-L5-Corpus (Verification Manager R3 lane). Different acceptance: byte-identity (this lane) vs algebraic-equivalence over a corpus (L5).
4. **Tier 3 mirror dissolution** — that's T-Tier3-Dissolution (PB Manager R3 lane).
5. **Bridge retirement** — that's T-Bridge-Retirement distribution (PB owns 3 of 5; tracked separately under `bridge_retirement_ledger_zero`).
6. **Performance budgets** — `r3-structure.md` §"Design challenge 7" Director decision: perf is post-R3 unless someone authors a budget claim with concrete numbers. T-FixedPoint deliverable is structural close; do not author perf gates here.

## Dispatch preconditions

Per `r3-structure.md` §"R3 worker dispatch precondition": the **joint precondition** is "R2-Evaluator landed AND R2-Grounding-Rust+Python landed" — Director-discretionary brief authoring may begin during R2 final week (this is what authorizes this planning artifact today), but **worker dispatch waits**.

PB Manager dispatches when **a single ledger reading** of the R2 Release Manager closure ledger shows all of the following simultaneously:

1. R2 close signal (R2 Release Manager closure ledger).
2. R2-Evaluator landed and stable (R2 lane closed).
3. T-LensProducer-Retirement (XL) closed — all three sub-gates (`lens_apply_dot_rs_retired`, `lens_testgen_dot_rs_retired`, `regen_lens_dot_rs_retired`) green; SG-0 non-test census = 0.
4. **R2-Grounding-Rust AND R2-Grounding-Python both closed** per `r3-structure.md` §"R3 worker dispatch precondition" Director-locked floor for the 7 Evaluator-gated lanes. R2-Grounding-Go closure is not required for dispatch; if Go is closed at the dispatch reading it expands the artifact set, otherwise the lane targets {Rust, Python} (see §"Single grounding gate").

The same ledger reading that satisfies (1)-(4) **derives** the accepted artifact set for the `pb_self_compile_fixed_point_strong` `TestSuite` (§"Acceptance gate"). One ledger reading → both dispatch readiness and artifact-set composition; the brief does not maintain a parallel hand-curated artifact list.

If any of (1)-(4) is not met, this brief stays in PROPOSAL state; PB Manager does not dispatch.

## STOP conditions

Worker MUST STOP and escalate to PB Manager (which escalates to Director if cross-program) when:

- **R1 fixture pressure:** any change required to `r1_release_acceptance_test.rs` predicate evaluation or `verification.dag` `FixedPointConverges` variant shape — that's a substrate or R1-horizon edit; not in this lane's scope. Surface as a substrate gap.
- **SG-0 census drift:** the SG-0 `EXPECTED_HAND_AUTHORED_NON_TEST` count is non-zero at evaluation time — T-LensProducer-Retirement is incomplete; this lane is not yet dispatchable.
- **Bit-identity fails for a structural reason** (e.g., emitter non-determinism: HashMap iteration order, timestamps, absolute paths in emitted output — full enumeration in [`docs/design-fixed-point-ratchet.md`](../design-fixed-point-ratchet.md) §"Sources of non-determinism") — that's an emitter dissolution, not a fixed-point-acceptance edit. Surface to PB Manager; do not paper over with normalization in the gate. The DB-8 grep gate + `determinism_test.rs` 5× check should catch most of these before the strong gate evaluates.
- **Trampoline expansion:** the "≤1 first-time-bootstrap trampoline" boundary tightens or expands — that's a Director-level cascade-decision change, not a worker call.
- **Substrate gap:** any need to introduce a new `TestPredicate` variant or extend `FixedPointConverges` — follow `INVARIANTS.md` §P1; do not author the variant in this lane.

## Cross-program signals

- **Lane open:** PB Manager → R2 Release Manager closure ledger (R3 continuation readiness signal).
- **Lane close:** PB Manager → R3 Release Manager (when authored) for R3 closure ledger; → Director for R3 thesis facet 2 closure announcement; updates `docs/thesis/r2-r3-thesis-mapping.md` row 136 status from ⏳ to ✅.
- **No upstream production:** T-FixedPoint consumes; it does not produce carriers other managers consume.

## TC3 — Strong-normalization TestClaim (author-now-fire-later, PB → R3 Verification transition)

**Status:** PROPOSAL — author-now-fire-later. Folded into this brief per add-on dispatch from PB Manager (#1149, 2026-04-29) because TC3 is structurally adjacent to T-FixedPoint and shares Evaluator termination semantics. **Currently PB territory; transitions to R3 Verification Manager when spawned.**

### Claim shape

```dag
test_claim every_typed_dag_program_terminates_in_bounded_steps {
  // For any well-typed .dag program with declared LoopBound,
  // evaluation terminates in O(loop_bound) reduction steps.
}
```

This is the **strong-normalization theorem** for the typed `.dag` fragment — the formal correlate of the totality choice that P4 Decidability rests on (`INVARIANTS.md:236` §P4; "bounded forward execution" foundational premise). Sufficient proof obligation per the add-on dispatch: structural induction on `Behavior` × `LoopBound BoundedLattice`.

### Dependencies (gates)

TC3 fires only when **both** land:

1. **B5 Loop construction-closure audit** — per [`docs/briefs/r2-release-b5-loop-construction-closure-audit-worker.md`](r2-release-b5-loop-construction-closure-audit-worker.md). The audit is load-bearing: strong-normalization quantifies over Loop constructions, so the closure of Loop's construction surface is a prerequisite.
2. **T-Substrate-Lens-Primitive** — provides the `Lens<C>` shape that TC3's evaluation-step witness presumably consumes. Without it, the claim has no way to express "evaluation terminates" as a structural fold.

(Source for the proof-obligation framing: PR #1178 `docs/design-substrate-lambda-calculus-grounding.md` §"Strong normalization for the typed fragment" — not on main at the time of authoring.)

### Substrate gap (load-bearing — STOP condition)

**The current `TestPredicate` substrate (`src/v3/std/verification.dag:108-235`) cannot encode TC3 without substrate introduction.** Every existing variant — `Compiles`, `OutputEquals`, `BehavioralObservation`, `AlgebraicLaw`, `FixedPointConverges`, `RatchetZero`, etc. — is **per-program**: it evaluates a property of the single `TestClaim.source` program. TC3 is a **meta-theorem over the entire well-typed fragment** — universally quantified across all programs the typing rules admit. No existing variant carries that quantifier shape.

Per the add-on dispatch guardrails:

- **Do not invent a new `TestPredicate` variant from this lane.** PB territory does not introduce verification substrate; that authority lives with Substrate Manager (per `INVARIANTS.md` §P1 substrate-fact-introduction procedure at line 86) or with the future R3 Verification Manager once spawned.
- **Do not fabricate a runner path.** The claim text above is the *declarative shape*; the runner-side encoding (e.g., is this a structural-induction proof checked by the Evaluator? a corpus-driven termination harness? a `Lens<TerminationWitness>` instance?) depends on which substrate variant carries it, which is precisely the gap.
- **Leave the declaration as a dispatch-gated proposal.** TC3 sits as text-form in this brief until B5 + T-Substrate-Lens-Primitive land and the R3 Verification Manager (spawned at R2 close) authors the substrate path.

### Transition to R3 Verification

When the R3 Verification Manager spawns (per `r3-structure.md` §"Manager structure" Item 2), TC3 ownership moves from PB to Verification. Verification then:

1. Picks up the substrate-gap question — either composes from existing variants (if a path emerges from B5 + T-Substrate-Lens-Primitive landing) OR escalates substrate introduction to Substrate Manager per §P1.
2. Authors the runner-side encoding once the substrate path is named.
3. Cross-references this brief's TC3 section as the upstream PB-authored declarative shape; PB does not re-author after transition.

### Non-goals

- Not a replacement for T-FixedPoint's two-horizon scope (#1169 main subject); TC3 is adjacent.
- Not in T-LensProducer-Retirement scope (Items 4+5 work; gated separately).
- Not a runtime termination check on a single program — that's already covered by ordinary timeout / decidability machinery. TC3 is the *meta-theorem* about the fragment.

## Cross-refs

- Parent manager: [`docs/briefs/r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md) §"Owns (R3 continuation)" + §"Acceptance" `pb_self_compile_fixed_point_strong`
- Lane authority: [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure" T-FixedPoint row + §"Design challenge 4" Director-locked SG-0 decision
- Two-horizon authority: [`docs/r2-structure.md`](../r2-structure.md) §"R1 closure criteria" + `r3-structure.md:60` two-horizon clarification
- Thesis-facet mapping: [`docs/thesis/r2-r3-thesis-mapping.md`](../thesis/r2-r3-thesis-mapping.md) row 136 (Facet 2)
- SG-0 floor authority: [`docs/design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md) §`First-time bootstrap` (≤1 trampoline rule)
- DB-8 ratchet design (mechanical authority): [`docs/design-fixed-point-ratchet.md`](../design-fixed-point-ratchet.md)
- DB-8 history (landed infrastructure): [`docs/db-history/db-8.md`](../db-history/db-8.md)
- Substrate variant: `src/v3/std/verification.dag:206-209` (`FixedPointConverges`)
- Ratchet binary (extend, do not rebuild): `src/v3/compiler/src/bin/self_host_fixed_point.rs`
- R1 fixture (do not edit): `src/v3/compiler/tests/integration/r1_release_acceptance_test.rs:18`
- Existing test scaffolding (reference, not the strong gate): `src/v3/compiler/tests/integration/l1_5_fixed_point_test.rs`, `src/v3/compiler/tests/integration/r1c_d_pb_census_gates_test.rs`
- Sibling R3-PB lane brief (gating dependency): T-LensProducer-Retirement worker briefs (pending — see `r2-pure-bootstrap-manager.md` §"Sub-briefs … Pending")

### TC3-specific cross-refs

- B5 audit (TC3 dependency): [`docs/briefs/r2-release-b5-loop-construction-closure-audit-worker.md`](r2-release-b5-loop-construction-closure-audit-worker.md)
- P4 Decidability (TC3 formal home): `INVARIANTS.md:236` §P4
- Termination carrier: `dsl/std/termination.dag` (`DescentEvidence` / BoundedLattice)
- Substrate-fact-introduction procedure (TC3 escalation path): `INVARIANTS.md:86` §"Procedure: substrate-fact introduction"
- Strong-normalization theorem source (off-main at authoring): PR #1178 `docs/design-substrate-lambda-calculus-grounding.md` §"Strong normalization for the typed fragment"
- TC3 transition target: R3 Verification Manager (per `docs/r3-structure.md` §"Manager structure" Item 2)
