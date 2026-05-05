# T-Ground-Tests — L4 routing-correctness verification

**Status:** PROPOSAL — dispatchable now (no design-cadence gate; consumer wiring is downstream and lands separately). Authored 2026-04-29.

**Lane:** T-Ground-Tests (S) — item **10** of 11 in [`r2-grounding-manager.md`](r2-grounding-manager.md) (lane description line 38, lane row line 69, acceptance gate line 129 `routing_correctness_l4_verified`, pending list line 147).

**Manager:** R2 Grounding Manager ([`r2-grounding-manager.md`](r2-grounding-manager.md)).

**Lineage / authorities consumed (no re-litigation):**
- R2 manager lane row + acceptance gate: [`r2-grounding-manager.md`](r2-grounding-manager.md) lines 38, 69, 129, 147; pilot precedent at line 60 + receipt at line 120 (`pilot_inhabitance_routing_stability_landed` — DONE PR #765).
- Engine-reframe spec: [`docs/design-emission-model.md`](../design-emission-model.md) — Examples 1-7 are lifted as test cases (per `r2-grounding-manager.md:11` structural-acceptance-per-lane-close discipline); Q4 universal four-property gate (Faithful / Correct / Minimal / Performant; locked PR-I); R3-vs-R2 split for L4-L7 (lines 404-411 — R3 `T-Verification-L4-L7-Direct` is the runtime-equivalence harness; this R2 lane is the **routing-stability** structural sibling that runs purely off substrate facts).
- Lens framework: [`docs/design-lens-framework.md`](../design-lens-framework.md) Q4 four-property entries and the Faithful/Performant/structural-Minimal lens-instance shape; `Lens<C>.read: (Dag, Behavior) → Witness<C>` substrate input (per design-emission-model.md:1107-1111 distinction: structural-fold properties are `Lens<C>` instances; runtime-equivalence properties live in R3 harness — this lane covers the structural side).
- Pilot routing-stability test stratum: `src/v3/grounding_pilot/src/lib.rs:408-460` — Stratum A (name-keyed parity) + Stratum B (algebra-homomorphism extension); precedent shape this lane generalizes.
- Substrate single-authority: row authorities at `src/v3/std/{rust,python,go}_method_template_contracts.dag` (Phase 1 / Phase 2 landed); LanguageSpec axes at `src/v3/std/emit_model.dag:303`; `MethodRef`/`MethodDeclaration` registry at `dsl/std/methods.dag` + `src/v3/std/methods.dag`.
- Fail-closed: [`INVARIANTS.md`](../../INVARIANTS.md) C-8 (P3) + C-series — every detectable mis-routing is a typed `Diagnostic`; no silent passes.
- Test discipline: [`TESTING.md`](../../TESTING.md) — hermetic, behavior-driven, unit-first; sub-second per `feedback_test_timeout_2s.md`.
- Brief shape templates: [`t-ground-languagespec.md`](t-ground-languagespec.md), [`t-ground-lifetime-analyzer.md`](t-ground-lifetime-analyzer.md), [`t-ground-diagnostic.md`](t-ground-diagnostic.md).

---

## Framing question this lane answers

For every (program × substrate × target) triple, can the compiler **structurally certify** that:

1. **Routing is stable** — the program's target inhabitance, as selected by the structural fold, is a deterministic function of (program, substrate). It does not depend on file-load order, declaration-id assignment, dictionary iteration order, or any other non-substrate-derived state. Re-compiling the same program against the same substrate yields the same per-binding inhabitance every time.
2. **Algebra satisfaction holds** — for every emitted target, the four properties of the Q4 gate (Faithful / Correct-structural / Minimal-structural / Performant) certify against the candidate's substrate-declared facts via `Lens<C>` instances. The fold's emission predicate is exact-match; the lens-instance reads close the structural verification loop.

A "yes" closes the L4 surface for **routing correctness on substrate facts** (the structural sibling to R3's `T-Verification-L4-L7-Direct` runtime-equivalence harness). A "no" — i.e., a routing decision that varies with non-substrate state, or a candidate that emits without satisfying its own declared algebra — is a **structural correctness fault** in the upstream substrate or fold; this lane surfaces it as a typed diagnostic, not a heuristic gate.

---

## Scope distinction — R2 structural vs R3 runtime (load-bearing)

Per `design-emission-model.md:404-411` and §"R3 lanes consume `Lens<C>` differently" (line 958):

- **R3 `T-Verification-L4-L7-Direct`** is the **runtime-equivalence harness**: corpus-driven; runs the emit target alongside `.dag` evaluation; checks output equality. It consumes Evaluator outputs and `Lens<C>` instances as inputs but is **not** itself a `Lens<C>` instance.
- **R2 T-Ground-Tests (this lane)** is the **structural certification**: walks substrate + lens-instance results purely; no runtime execution; no Evaluator dependency. It certifies that the emission decision is structurally well-formed *before* R3 runs the artifact.

The two lanes are complementary. R2 covers "routing is structurally correct"; R3 covers "structurally-correct routing produces runtime-equivalent output." A program that fails R2 cannot reach R3 (the fold rejects it); a program that passes R2 may still fail R3 if substrate facts under-determine runtime behavior — that gap surfaces as a substrate-completion task, not a routing fault.

---

## Scope (per acceptance gate `routing_correctness_l4_verified`)

### A. Routing-stability test framework

Per the pilot precedent at `src/v3/grounding_pilot/src/lib.rs:408-460`, generalize the two strata to the production fold (T-Ground-Coercion-Fold consumer):

- **Stratum A — Name-keyed parity.** For programs whose target inhabitance is determined by an exact-match registry entry (e.g., kernel-aliased `Int = Int64 → "i64"` per `dsl/extdeps/languages/rust/types.dag`), the fold's selection MUST equal the registry's declared target. This proves the fold consumes the substrate without re-deriving.
- **Stratum B — Algebra-homomorphism extension.** For programs whose target inhabitance has **no** name-keyed entry (e.g., `Int8 / Int16 / UInt32` etc.), the fold's selection MUST match the unique `(algebra × refinement)` candidate. This proves the structural emission predicate works where the registry doesn't.

Both strata test the **deterministic** property: shuffle non-substrate state (file load order, declaration-id allocation order, hash-map iteration) and assert routing output is bit-identical. Stratum-A failure = registry/fold drift; Stratum-B failure = upstream substrate gap or fold over-resolution.

### B. Algebra-satisfaction certification

For each per-target inhabitance, run the Q4 four-property gate via `Lens<C>` instances (per `design-emission-model.md:1107-1111`):

- **Faithful** — `Lens<FaithfulnessVerdict>`: substrate predicate that the inhabitance's declared algebra matches the program's algebra. Failure = inhabitance lies about which algebra it inhabits.
- **Correct (structural)** — for the structural component: `Lens<CorrectnessVerdict>` reading substrate facts (e.g., bound carriers, encoding axes). The runtime-equivalence component lives in R3.
- **Minimal (structural)** — `Lens<MinimalityVerdict>` where minimality is structurally definable (per `design-emission-model.md:1108`); the runtime-comparison fallback lives in R3.
- **Performant** — `Lens<PerformanceVerdict>`: reads `RealizationCost { storage, access }` (Q3 lock per PR-I); checks for pathological complexity classes.

Each lens-instance produces `Witness<C>` per per-target inhabitance per program. The lane lifts these witnesses into per-program × per-target × per-axis `TestClaim` outcomes.

### C. Cross-stratum consistency

The two strata MUST agree where they overlap (Stratum-A registry entries that also have a structural homomorphism). Disagreement is a substrate-completion fault: either the registry has stale data or the algebra-homomorphism extension is wrong; either way, surface a typed `Diagnostic` per Q6.5 Layer-1 `CompilerDiagnosticKind`.

### D. Fail-closed surface

Per [`INVARIANTS.md`](../../INVARIANTS.md) C-8: every per-(program × target × axis) outcome is `TestClaim::Pass` or `TestClaim::Fail(Diagnostic)`. Lens-instance under-determination (e.g., Q4 `Performant` lens encounters an algebra op with no declared `RealizationCost`) surfaces as `Diagnostic` per the substrate-load-time fail-closed discipline (`feedback_fail_closed_discipline.md`). No silent skips.

### E. Test corpus shape

Per `design-emission-model.md:1232` recommendation: hybrid (c)+(d) — generated cross-product corpus (every combination of substrate axes × Shape-A targets) for completeness coverage + user-program corpus (`dsl/std/` + `src/v3/std/` + `dsl/examples/`) for self-hosting / realism coverage. The L6 cross-target meta-spec at `T-Ground-CrossTarget-Meta` is the structural completeness companion (verifies emission paths exist); this lane verifies emission **routing is stable** on those paths.

Examples 1-7 from `design-emission-model.md` (lines 415-792) are lifted as the seed corpus per `r2-grounding-manager.md:11` structural-acceptance-per-lane-close discipline. Each example's "expected target" + "expected diagnostic" become `TestClaim` rows.

### F. P1 substrate-fact-introduction receipts

Worker MUST run the 3-step procedure ([`INVARIANTS.md#p1-modeling-faithfulness`](../../INVARIANTS.md#p1-modeling-faithfulness) (Procedure)) for any new substrate type / variant / field introduced under this lane:

- **Step 1 (DAG-ancestor):** the per-axis `Witness<C>` and `TestClaim` carriers — does an ancestor exist in `dsl/std/` or `src/v3/std/`? (`Witness<C>` is from `src/v3/std/dimensions.dag`; `TestClaim` is from `src/v3/std/test_claims.dag` if present, else escalate.)
- **Step 2 (Coproduct-vs-coordinate):** the test outcome shape — `TestClaim::Pass | Fail(Diagnostic)` is a proper alternative; per-axis verdicts on a single (program, target) inhabitance are coordinates of a record.
- **Step 3 (Primitive-vs-lens-extensible):** new lens-instance kinds (e.g., `RoutingStabilityVerdict`) are **lens-extensible**, not substrate-primitives; they ride the Q6.5 Layer-2 path (per `t-ground-diagnostic.md` framing — Layer 1 is closed-substrate-owned; Layer 2 is open-lens-extensible). This lane MUST NOT add to Layer 1 `CompilerDiagnosticKind`.

Per `feedback_substrate_principle_audit.md` and `r2-grounding-manager.md:106`, this is non-optional.

---

## Out of scope (do NOT do)

- **Runtime-equivalence checks** (`L4 emit/eval match` per `design-emission-model.md:406`) — that's R3 `T-Verification-L4-L7-Direct`. This lane is the structural sibling, not the runtime harness.
- **Authoring `Lens<C>` instance kinds.** This lane *consumes* lens instances (per Q6.5 Layer-2 framework); does NOT widen the closed `CompilerDiagnosticKind` Layer-1 sum.
- **Per-target benchmark performance tests.** Performance certification here is the **structural** Performant lens (reads declared `RealizationCost`); benchmark-driven measurement lives in a different lane (likely R3 / post-R3).
- **Emit-pipeline tests** (codegen correctness) — that's the emit lane / language-specific tests, not this lane.
- **Coercion-Fold body authoring.** This lane consumes the fold's outputs; it does NOT implement the fold.
- **L5 cross-target consistency** — separate R3 lane (`T-Verification-L5-Corpus`).
- **L6 substrate-load completeness** — `T-Ground-CrossTarget-Meta` (R2 sibling lane, per `design-emission-model.md:408`).
- **Touching `src/v3/compiler/`** — SG-0 ratchet.
- **Re-litigating Q4 four-property gate**, the R2-vs-R3 split at line 404, or the lens-instance vs runtime-equivalence distinction at lines 958-959 / 1107-1111.

---

## Dependencies / gates

| Gate | Status | Lane impact |
|---|---|---|
| **Coercion-Fold output surface** (T-Ground-Coercion-Fold body) | held per `design-emission-model.md:900-910` option (c) until LanguageSpec lands | Required for Stratum A/B to have a real fold to certify; until then, this lane runs against the pilot crate's `find_inhabitant` / `ground` (precedent at `src/v3/grounding_pilot/src/lib.rs:402-405`) as a stand-in |
| **LanguageSpec axes populated** (T-Ground-LanguageSpec Phase 1) | LANDED #1195 + #1210 | Consumed for routing-stability inputs (per-target candidate set) |
| **PR-I (Q4 four-property gate)** | per `r2-grounding-manager.md:51` | Required for B (algebra-satisfaction certification) — Q4 lens-instance shapes land here |
| **`MethodTemplateContract` rows + `MethodRef` registry** | LANDED #1175 + #1193 + #1195 + #1210 | Stratum-A registry entries lift from these per (target, dag_method) |
| **PR-PreF / PR-F / PR-G / PR-H (per-target axes)** | per cadence | Required for B's per-target structural axis lens-instances |
| **T-Ground-Lifetime-Analyzer** outputs | per `r2-grounding-manager.md:66` | Required for programs with non-trivial scoping (Examples 3-4 in `design-emission-model.md`) |
| **T-Ground-Diagnostic** `EmissionDiagnostic` carrier | per `r2-grounding-manager.md:67` | Consumed for fail-closed `TestClaim::Fail(Diagnostic)` shape |
| **T-Ground-CrossTarget-Meta** L6 completeness | per `r2-grounding-manager.md:68` | Companion (not gate) — completeness vs stability are independent properties; both must hold |

**Cross-program signals:**
- **R3 Verification Manager:** signal lane-close to coordinate with `T-Verification-L4-L7-Direct` runtime harness — R2 routing-stability + R3 runtime-equivalence are dependent (R3 corpus is L4-corpus per Q4; this lane's stability claim is precondition for R3's equivalence claim).
- **Cost-lens consumer (R3 `T-CostLens-Composition`):** Performant lens-instance shape co-developed.

---

## Sizing

**S** per `r2-grounding-manager.md:69` and `:38`. Distribution (informal):
- Routing-stability test framework (A): S — generalizes pilot-crate Stratum A/B pattern; no novel substrate.
- Algebra-satisfaction certification (B): S — wires Q4 lens-instances into per-(program × target × axis) walker; lens instances are PR-I deliverables, not this lane.
- Cross-stratum consistency check (C): trivial — set comparison.
- Fail-closed surface (D): S — `TestClaim` shape lifted from sibling diagnostic carrier.
- Corpus authoring (E): S — Examples 1-7 lift mechanically; cross-product generation is small per `design-emission-model.md:1232` recommendation.

Bundle into one PR per `feedback_bundle_workstreams_per_pr.md` unless scope balloons. If (E) corpus generation surfaces a substrate-completeness gap (i.e., a `(connective × behavior × target)` cell with no emission path), STOP — that's L6 (`T-Ground-CrossTarget-Meta`) territory, not this lane.

---

## Test plan

Per `TESTING.md` — hermetic, behavior-driven, unit-first; sub-second per `feedback_test_timeout_2s.md`.

Acceptance lifted to a `.dag` `TestClaim` (gate: `routing_correctness_l4_verified` per `r2-grounding-manager.md:129`):

1. **Stratum-A name-keyed parity** — for every `(target, dag_method)` row in `{rust,python,go}_method_template_contracts`, the fold's chosen target equals the row's declared target. (Existing pilot-crate Stratum A test pattern generalized.)
2. **Stratum-B algebra-homomorphism extension** — for every program whose `(algebra × refinement)` matches a single substrate inhabitance with no name-keyed entry, the fold's chosen target equals that unique inhabitance. (Existing pilot-crate Stratum B test pattern generalized.)
3. **Determinism / non-substrate-state-free** — re-run Stratum A/B with shuffled file-load order and shuffled hash-map iteration; results bit-identical. Failure = the fold reads non-substrate state.
4. **Q4 Faithful per inhabitance** — every per-target inhabitance's declared algebra matches the program's algebra in every example program.
5. **Q4 Correct (structural) per inhabitance** — substrate-fact predicates (bound carriers, encoding axes) hold per inhabitance.
6. **Q4 Minimal (structural) per inhabitance** — where structurally definable; otherwise marked "deferred to R3 runtime check."
7. **Q4 Performant per inhabitance** — `RealizationCost { storage, access }` lens-instance fires per inhabitance; missing access-map entry produces `Witness::Violates` per `design-emission-model.md:1206`.
8. **Cross-stratum consistency** — overlapping (target, dag_method) entries produce identical inhabitance choices in both strata.
9. **Fail-closed under-determination** — a program with under-refined axis surfaces `TestClaim::Fail(EmissionDiagnostic::UnderRefined { axis })` per `t-ground-diagnostic.md` shape; not silently skipped.
10. **Corpus coverage** — Examples 1-7 from `design-emission-model.md` (lines 415-792) lift as TestClaim rows; each pass-or-fail outcome matches the example's documented expectation.

`cargo test --workspace --exclude v2-compiler-tests`, `cargo test -p v2-compiler-tests`, `cargo test -p v3-compiler --test integration lane2_stage_2d_symbolic_cost`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check` all clean (per the post-#1195 lesson: full-bootstrap + lane2 cost-test paths catch what workspace-exclude misses).

---

## Dissolution claim

When this lane lands:
- The `routing_correctness_l4_verified` acceptance gate at `r2-grounding-manager.md:129` closes (DONE).
- Routing-stability is a **structural fact** (test pass) for every program in the corpus, not an asserted-not-verified claim. The pilot crate's Stratum-A/B receipt at `src/v3/grounding_pilot/src/lib.rs:408-460` generalizes to the production fold; the pilot crate becomes deletable per `T-Ground-Dissolve` once consumers route through the production fold (lane 11 territory; not this lane).
- Q4 four-property gate is enforced **per inhabitance** (not just per program), via lens-instance walker on the per-target row authorities.
- R3 `T-Verification-L4-L7-Direct` consumes routing-stability as precondition (R2 → R3 hand-off receipt named in cross-program signal).
- Substrate-completeness gaps that this lane surfaces (cross-stratum disagreement, missing `RealizationCost` entries, etc.) get typed `Diagnostic` channels — caught by construction rather than discovered as production faults.

The dissolution claim is verifiable: every TestClaim row in the corpus produces a structural `Pass`/`Fail(Diagnostic)` outcome; failure modes are enumerable and named.

---

## Hand-off discipline

Escalate to manager (post on #1133, do **not** absorb in lane) if:

- **Cross-stratum disagreement** surfaces — Stratum A and Stratum B return different inhabitances for the same (algebra × refinement) overlap. That's a substrate-completion fault upstream (LanguageSpec axis disagreement with registry); needs R2 Substrate / Grounding Manager triage.
- **Routing is non-deterministic** — shuffled state changes the fold's output. That's a fold-implementation bug; escalate to T-Ground-Coercion-Fold owner.
- **Q4 lens-instance under-determination** — a program's required axis isn't declared on any candidate inhabitance; that's a substrate-completion gap (PR-I follow-up or per-target axis lane work).
- **Corpus generation surfaces an L6 gap** — a `(connective × behavior × target)` cell with no emission path. That's `T-Ground-CrossTarget-Meta` territory; surface and stop.
- **A new lens-instance kind** is needed beyond the existing Q4 four-property set — escalate before authoring; the lens framework is closed-by-construction (Layer-1) for compiler-internal kinds.
- **Implementation requires touching `src/v3/compiler/`** — SG-0 ratchet violation.

Per `feedback_root_causes_over_quick_fixes.md`: no quick fixes. Per `feedback_no_textual_enforcement_bridges.md`: no grep/regex bridges to "be structural."

---

## Acceptance — `.dag` gate

Lane closes under the `r2-grounding-manager.md:129` gate:

> `routing_correctness_l4_verified` — Tests lane closes.

Authored as a `.dag` `TestClaim`. Per the **structural-acceptance-per-lane-close discipline** (`r2-grounding-manager.md:11`), the gate IS the demo — no separate artifact.

PR body covers: scope (A-F); routing-stability strata (Stratum A name-keyed parity + Stratum B algebra-homomorphism extension); Q4 four-property certification per inhabitance; corpus shape (Examples 1-7 lift + cross-product + user-program); fail-closed receipts (every TestClaim is Pass or Fail(Diagnostic)); cross-stratum consistency receipts; P1 substrate-introduction Step 1+2+3 receipts per new carrier (if any).

---

## What unblocks on merge

- **R3 `T-Verification-L4-L7-Direct`** has its R2 routing-stability precondition; runtime-equivalence harness can dispatch with confidence the fold's structural output is well-formed.
- **R3 `T-CostLens-Composition`** Performant lens-instance shape co-validated by this lane's corpus-coverage runs.
- **`T-Ground-Dissolve`** has the structural receipt to begin retiring `TypeCheckpoint` / `InhabitantDecl` / `carrier: String` (per `r2-grounding-manager.md:130`).
- **Manager** updates lane row at `r2-grounding-manager.md:69` to LANDED; signals R2 Release Manager (closure ledger).

---

## Cross-refs

- Parent: [`r2-grounding-manager.md`](r2-grounding-manager.md) (lane 10 of 11; row line 69; gate line 129)
- Engine-reframe spec: [`docs/design-emission-model.md`](../design-emission-model.md) — Examples 1-7 (corpus seed); Q4 (lines 1107-1111); R2-vs-R3 split (lines 404-411 + 958-959)
- Lens framework: [`docs/design-lens-framework.md`](../design-lens-framework.md) — Q4 four-property entries; `Lens<C>` substrate input shape
- Sibling lanes: [`t-ground-languagespec.md`](t-ground-languagespec.md) (axis vocabulary), [`t-ground-lifetime-analyzer.md`](t-ground-lifetime-analyzer.md) (structural intent inputs), [`t-ground-diagnostic.md`](t-ground-diagnostic.md) (`EmissionDiagnostic` shape consumed)
- R3 successor: `T-Verification-L4-L7-Direct` (per `design-emission-model.md:404` — runtime-equivalence companion)
- Substrate-fact-introduction: [`INVARIANTS.md`](../../INVARIANTS.md) §P1
- Brief shape templates: [`t-ground-languagespec.md`](t-ground-languagespec.md), [`t-ground-lifetime-analyzer.md`](t-ground-lifetime-analyzer.md), [`t-ground-diagnostic.md`](t-ground-diagnostic.md)
- Pilot precedent: `src/v3/grounding_pilot/src/lib.rs:408-460` (Stratum A/B routing-stability seed); receipt at `r2-grounding-manager.md:120` (`pilot_inhabitance_routing_stability_landed` — DONE PR #765)
