# Plan — testgen as the bug-class oracle (coverage by construction)

**Status:** audit + direction · **DESIGN.md + carriers are authority** (§6). Linked from `ROADMAP.md` §4 *Testgen as the bug-class oracle* (and a §0 lock-down subject — its output is not floor-discovered).
The operator's frame: testgen is the answer to *"how do we prevent the next CLASS of bugs, and how do
they relate to structure?"* — cover a class by **generating witnesses from the declared structure**, not
one hand-written witness per bug. That is the construction principle (§0) applied to test coverage:
instance-by-instance witnesses are validation; a generator over structure is construction.

**Verified against the live tree 2026-06-21.** Receipts below; re-check before acting.

## 0. What testgen actually is today

A compile-time lens (`src/v2/lens/testgen.dag`, ~2078 lines) that emits conformance test modules into
`src/v2/compiler/generated/` (`module v2.test.generated.*`, all stamped one batch). Categories present:

| category | file | source |
|---|---|---|
| AlgebraLawConformance | `algebra_law_conformance.dag` | **structural** — `nat_declared_algebra_law_obligations()` (`std/nat.dag`) |
| CoproductExhaustiveness | `coproduct_exhaustiveness.dag` | **structural** — `enumerate_coproduct_decls()` × `coproduct_arm_keys` over EVERY declared coproduct (§4.2 landed) |
| WitnessValidity | `witness_validity.dag` | hand-anchored (RoundTrip Deferred) |
| IdempotentOperationConformance | `idempotent_operation_conformance.dag` | hand-anchored |
| LanguageBehaviorEquivalence | `language_behavior_equivalence.dag` (+ `lbe_anchor_manifest`) | hand-anchored |
| RefinementPreservation | `refinement_preservation.dag` (+ manifest) | hand-anchored |

Header: *"new TestgenConcept arms = operator STOP."* So testgen is **manual-anchor with one structural
category** (AlgebraLaw), not yet arbitrary structural generation.

## 1. The gaps — testgen is itself a §0 lock-down subject (three faces)

1. **Coverage by illusion — the generated output is not floor-discovered.** Every file in `generated/`
   has **zero `test fn`** and is **not `*_test.dag`** → the witness floor (which discovers `test fn` in
   `*_test.dag`) runs none of them. The only execution path is Rust `m1_5_testgen_test::*` (listed in
   `test_node_wall_clock_ratchet.dag`), and the CI rust gate runs only the narrow subset
   (`interp_recorded_fixture wet_hermetic resolve_expr_types_retraversal`, `ci_spec.dag:160`) — so the
   testgen Rust tests are **ungated** too. Testgen produces conformance modules that **nothing in CI
   runs**. (= the §0 CI-coverage-completeness audit, concretely.)
2. **Parallel representation — no drift gate.** The checked-in `generated/` artifacts (a *realization*)
   can fork from the generator (the *model*) — the `ci.yml` problem. A regen-without-drift smoke exists
   (`self_gen6_regen_lens_cli_smoke_regenerates_named_entry_without_drift`) but it's in the wall-clock
   ratchet = Rust, ungated. `testgen.dag` already carries `🟡` comments *forbidding* record-shape drift
   (`NatAlgebraLawObligation ⇄ AlgebraLawSubject`) — the risk is known and uncovered. Construction fix:
   the generated artifacts should not be checked in as an independent representation at all, **or** a
   floor-gated drift check (regen == committed) must make divergence unwritable (same shape as CiYamlGate).
3. **Manual-anchor, not structural.** Only AlgebraLaw derives from declared obligations; the rest are
   hand-lists. Arbitrary structural generation needs `.dag` structural reflection — and the primitive
   **now exists** (`v2.std.node_query.coproduct_arm_keys(type_name)`, `collect_nodes_where`); routing the
   other categories through it is the construction step (and per the header, an operator-gated decision).

## 2. The oracle method — map each bug class to a generator category

The retro the operator wants: for every bug class we hit, ask three questions and record the answer.
(1) Is there a generator **category** for this class? (2) Is it **structural** or hand-anchored?
(3) Is its **output gated** (run in the floor)? A "no" on any axis is the work.

| bug class (from memory / recent PRs) | category | structural? | gated? |
|---|---|---|---|
| coproduct non-exhaustive match (`Value::eq` `_ => false`) | CoproductExhaustiveness ✓ | **yes** (§4.2 — every declared coproduct) | **yes** (floor-discovered) |
| cross-representation `==` straddle (model↔realization fork) | CrossRepresentationEquality ✓ | partial (modeled side structural; native-realization roster curated — §4.3) | **yes** (floor-discovered) |
| algebra-law violation | AlgebraLawConformance ✓ | **yes** (the exemplar) | no |
| flat-namespace fn collision (#5185) | — | — | — |
| heterogeneous `==` module resolve-fail | — | — | — |
| cache key drift / warm≠cold | — (runtime — needs the warm==cold *construction*, not testgen) | — | — |

The map is mostly "no" today — that **is** the finding. The valuable categories to add are the ones
whose class is **structural** (enumerable from the Node tree): cross-representation equality (every
modeled coproduct × its native realization), exhaustiveness over *every* declared coproduct (not a
roster). Runtime/realization classes (cache) are not testgen's job — they're the §0 construction items.

## 3. affected-set — the completeness half, not the minimality half

`affected_set` (`src/v2/lens/affected_set.dag`) is **not wired** in `ci_spec`/`ci_floor_plan` as a
selection mechanism — confirmed (zero refs). It was retired as a **0-min shadow** (node-precision
selection saved nothing; the v2 claim corpus is cheap). The loop witnesses now **exist and are
un-dangled** (`affected_testgen_ci_runner.dag` + `…_gate_test.dag` under `src/v2/test/`,
single-representation — the #5023 straggler is fixed), but selection-as-CI-gate stays shelved correctly.

The operator's tie to "CI running / v1 tests running" is real but it's the **coverage** half, not
minimality: the binding constraint is the §0 hole (v1 test set ungated + narrow rust subset), i.e. the
"widen/retire the rust gate" item. affected-set's *relevant* contribution is **completeness of the
universe** (model every repo process — incl. meta-processes — so nothing is a blind spot; an incomplete
universe = false confidence). That needs the same `.dag` structural reflection testgen needs.

**Shared blocker (the unifying insight, still true):** arbitrary-structural testgen AND
universe-complete affected-set both need the substrate to enumerate its own declared structure. The
primitive now exists (`node_query`), so the blocker is partly lifted — the work is routing generators +
universe-enumeration through it.

## 4. Direction (dependency-ordered → ROADMAP §4 *Testgen as the bug-class oracle*)

1. **Gate the existing generated output** — make `generated/` floor-discovered (single-representation
   `test fn` in `*_test.dag`, or a drift gate regen==committed). Closes face #1+#2 with no new logic.
2. **Make CoproductExhaustiveness structural** — ✅ LANDED. Routed through
   `v2.std.concept_index.enumerate_coproduct_decls()` × `coproduct_arm_keys` over *every* declared
   coproduct (the anchor is now `GeneratedCoproductExhaustiveness { coproduct_type: Symbol,
   omitted_variant: Symbol }`, not the closed `TestClaimCoproductVariant` roster). Floor-discovered in
   `generated_conformance_floor_test` with a non-empty-roster guard (fail-closed on a dormant
   enumeration) and independent arm recomputation for TestClaim + Connective; perturbing the roster to
   empty goes RED (verified).
3. **Add the cross-representation-equality category** — ✅ LANDED (SCOPE TIGHT). New `TestgenConcept`
   arm `CrossRepresentationEquality { modeled_coproduct, native_realization }` + generated anchor
   `GeneratedCrossRepresentationEquality`; the generator emits the `EqualsClaim` that the modeled form
   reconciles with its native realization (the grounding TARGET — emitted, not executed: grounding each
   straddle is its own runway). Floor-discovered in `generated_conformance_floor_test`
   (`cross_representation_equality.dag`), checking emit shape + curated straddle-roster coverage with
   each modeled coproduct's arm keys recomputed independently (`coproduct_arm_keys`) to exclude a
   vacuous pass (perturbing the roster or pointing it at a non-coproduct goes RED — verified). SCOPE:
   ONLY the straddles that remain UNSTRUCTURED — Bool over Value::Bool, and the Value::Null sentinel
   (Optional/Witness, fenced to its own runway). The numeric tower is GROUNDED (#5428) and excluded.
   The modeled side is structural; the native-realization side is curated (no Node-tree reflection for
   "realized as Value::Bool"). **Remaining (own runway):** ground each straddle so the `EqualsClaim`
   actually executes green (Bool, then the deeper Value::Null None/Absent/miss split, ~131 sites).
4. **affected-set completeness** — model the full repo-process universe (incl. meta) under the lens, on
   the reflection primitive; selection stays shelved (0-min) until a corpus makes it pay.

## 5. Dissolution trigger (DESIGN §6)

Delete this doc when testgen's generated output is floor-gated (drift-closed), the structural categories
are derived from `node_query` (not hand-rosters), and the bug-class→category map has no structural-class
"no" — at which point a new bug in a structural class is caught by the generator that already enumerates
it, and this tracker is redundant.
