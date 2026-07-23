# The vacuity lens — tests that reaffirm the code instead of checking it

> A lens that classifies whether a test can **distinguish correct behavior from incorrect** or merely **re-affirms the code**. The output is fail-closed *evidence* (`ProvenDuplicate | ProvenIndependent | Unknown`), **not** a syntactic flag — only `ProvenDuplicate` gates; `Unknown` refuses or stays advisory, never green. DESIGN refs: §5 ("a check that re-states a constraint the model already carries is a second representation of it"; the "spec-without-execution" trap — a grep-passing test that runs green but discriminates nothing; fail-closed = `Unknown` never passes), §6 (lens as residue; coverage-by-illusion), §3 (single authority). **Two design corrections from review (2026-07-15), load-bearing:** (a) vacuity is **not** "the plurality axis at test grain" and does **not** reuse `coverage_defect_vacuous_arm` — that name already means an *exhaustive-but-empty match arm* (`coverage.dag:14-27`), a §3 name collision; test-oracle aliasing gets its **own** `VacuityEvidence`. (b) The classifier is **consumer-aware**: identical operands are *not* a wall (`rung_5` stores identical lhs/rhs and observes them through **emit vs eval** — a legit equivalence test), so the *observation/consumer relation* must be modeled before anything gates. Kin — but not identical — to the [inert-layer lens](inert-layer-lens.md) (reachability, a *distinct* invariant, per the [consolidation design](lens-consolidation-design.md)'s "consolidate mechanisms, not meanings"). Successor to the one-shot [mechanism-inventory audit](mechanism-inventory-red-controls.md) finding #5 (~28 synthetic-only lenses that execute yet catch no real violation).

## 1. The definition — a vacuous test is *redundant* with the code (the root)

The root is **redundancy**, not testing technique. A vacuous test is a *second representation* of a fact the code already carries — it moves 1:1 with the code (edit the code, edit the test, lockstep), so it is a **change-detector, not a check**. Stated once (operator framing, 2026-07-14):

> A test is **vacuous** iff its assertion is a deterministic 1:1 function of the code's own structure with **no referent independent of it**. It carries zero information about *correctness* — only a duplicate encoding of *state*.

This is DESIGN §2 (minimize redundancy) / §3 (single authority) at the test↔code seam, and DESIGN §5 verbatim: "a check that re-states a constraint the model already carries is a second representation of it."

**Vacuity shares a *mechanism* with `materialization_ladder`, not a *verdict* (review correction).** The tempting unification — "a vacuous test *is* `AuthoredDuplication`" — is wrong on the remedy: the ladder prescribes `AuthoredDuplication → Share` (`materialization_ladder.dag:518-532`), but *sharing* `f(x)` in `f(x)==f(x)` makes the test **more** tautological, not less. A vacuous test's remedy is an **independent oracle, construction, or deletion** — a different prescription, hence a different verdict. What the two legitimately share is only the counting *primitive* (a typed `Count<Relation>`), not the verdict lattice; per the [consolidation design](lens-consolidation-design.md) the rule is *consolidate mechanisms, not meanings*. So vacuity gets its own verdict authority (`VacuityEvidence`, §1.1), and grounds onto the **oracle/authority-provenance** kernel, not `LadderVerdict`.

**The three consequences (the §1 time lens — this is the *lesson*, not just the definition):**
- **complexity/cost** — a maintenance tax paid forever: every structural edit must touch two places (§2's "defers cost onto a later fixer").
- **safety** — zero harm reduction: it cannot catch a *behavioral* error, only a transcription typo, so it *looks* like coverage while providing none (§5 coverage-by-illusion).
- **the fix is not "delete the test" — it is §5 construction-over-validation.** A fact checkable by 1:1 restatement is *structural*, and structural facts should be **derived from one authority** (made unwritable-if-wrong), never validated after the fact by a mirror test. So the lens's *product* is the **located duplicated authority** (the "output is the root, not the symptom" discipline of the duplicate-work lens, §6 moat) — not a delete list.

### 1.1 The evidence — `VacuityEvidence`, fail-closed

A test is classified by **evidence, not syntax** (the review's core correction — syntactic shape decides nothing):

```
VacuityEvidence = ProvenDuplicate | ProvenIndependent | Unknown
```

- **`ProvenDuplicate`** — proof that *expected* and *actual* reduce to the same computation observed through one path with no independent oracle. **The only verdict that gates.**
- **`ProvenIndependent`** — a genuine independent oracle exists (behavioral, distinct-consumer, or temporal/external authority) → legit.
- **`Unknown`** — the evidence needed to decide is unavailable → **advisory or refuse; never green** (fail-closed, §5 — `Unknown` passing is the absorbing-fallback trap).

Evidence is a record over these dimensions; a missing dimension forces `Unknown`, never a guess:

- **subject** — what the claim is actually testing.
- **observation / consumer relation** (`TestClaimObservationRelation`) — *how the two sides are observed*: `SameEval` (one interpreter path — a duplicate risk), `EmitVsEval` (emitted-host output vs interpreter — independent by construction, the `rung_5` case, `rung_5.dag:31-45` / `emit_host.dag:412-433`), `SingleEvalOutput`, … **This is the dimension the review surfaced and the one that must be grounded first** — without it, identical operands cannot be judged.
- **provenance of actual & expected** — read-back-of-a-stored-field · independent-transcription · behavioral-computation · rendering-transform.
- **comparator semantics** — what `==` / structural-eq actually decides here.
- **effects** — either side running an effect can make identical operands observe different world states.
- **temporal / external authority** — a golden from a prior release, an ABI contract, a datasheet is a *legitimate* temporal/external oracle.

## 2. The classifier — only `ProvenDuplicate` gates; the adversarial suite

The classifier maps evidence → verdict and must be **total over all five `TestClaim` variants** (`CompilesClaim | DiagnosticClaim | EqualsClaim | StructuralEqualsClaim | RoundTripClaim`, `verification.dag:159-195`) — an Equals-only implementation is incomplete, and a variant it cannot classify returns `Unknown` (refuse), **never silently clean**. It classifies **per-assertion**, so one vacuous assertion beside a genuine one flags only the vacuous one. "No false positives/negatives" is a property of a *total classifier with `Unknown` refusing*, not of any finite corpus — a corpus can only falsify.

The falsification suite (the classifier is wrong if any row disagrees):

| case | observation / provenance | verdict |
| --- | --- | --- |
| identical operands, `EmitVsEval` consumer (`rung_5`) | two independent execution paths | `ProvenIndependent` |
| identical operands where reflexivity/determinism *is* the property under test | the equality is the subject | `ProvenIndependent` |
| identical **pure** operands under a single-eval output claim (subject + comparator proof) | one path, no oracle | `ProvenDuplicate` |
| stored field read back vs a copied literal (`x.module_path == "…"`) | read-back, no transform | `ProvenDuplicate` |
| rendering transform vs a literal oracle (`render(x) == "v2.std.text"`) | transform + independent expected | `ProvenIndependent` |
| golden from a prior release / ABI contract | temporal authority | `ProvenIndependent` |
| two **independent** datasheet transcriptions (code and test each transcribe) | independent oracle, but two authorities | `ProvenIndependent` for vacuity — **plus a separate `parallel-authority` debt** (a different lens) |
| expected helper *aliases* the production implementation | same computation, different syntax | `ProvenDuplicate` (syntax doesn't save it) |
| vacuous `CompilesClaim` / `DiagnosticClaim` | non-Equals variant | classify or `Unknown` — never silently clean |
| one vacuous assertion beside a genuine one | per-assertion | flag the vacuous assertion only |
| unknown consumer / provenance | evidence missing | `Unknown` — advisory or refuse, never green |

Two corrections the suite encodes. **The datasheet case is not simply "vacuous":** reading back the model field (`watt_count(cpu.tdp_watts) == 183`) is `ProvenDuplicate`, but two *independent* transcriptions are `ProvenIndependent` and belong to a distinct **parallel-authority** defect, not vacuity's wall. **Identical operand trees are never a wall on their own:** the reflexive shape is `ProvenDuplicate` only once the observation relation proves a single evaluation path with no oracle — `rung_5` is the standing counterexample to "reflexive ⇒ vacuous."

## 3. The coverage seam — parsed-tree native fold, NOT a v1 host bridge (v1 is being deleted)

The trap this lens must not fall into is *its own subject*. A **pure-`.dag`** vacuity lens run only over *synthetic* claims would be exactly a mechanism-inventory **Group C** lens: executes, passes the #5433 inert-lens backstop, and catches zero real corpus violations. A vacuity lens that is itself vacuous over the real tree is the worst possible outcome (§5, and §7 — the compiler's own dead check is a substrate fact). So the lens **must** read the live corpus.

**Correction (2026-07-14, after catching up to current main): the corpus read must NOT be a v1 host bridge.** `src/v1` is being deleted; adding functions to `cli_run.rs` builds on dead-code-walking. The tree is migrating every corpus-fact producer **off** v1 host builtins **onto** parsed-tree `.dag`-native folds — `v2.lens.reference_deps` already `import`s `v2.compiler.parse { parse_module }` / `v2.compiler.tokenize { tokenize }` and folds the corpus itself, and the surviving host twins (`build_import_adjacency` in `cli_run.rs`) are **isolated as a single swappable producer** under a standing operator constraint (2026-07-10): *"Do not add a second edge producer beside `dependency_resolution_facts_live`; the swap replaces its body."* A vacuity `*_project.rs` / `cli_run.rs` bridge would violate exactly that constraint and be deleted with v1.

So the vacuity lens rides the **same parsed-tree native producer** the reference/module-graph lenses do:

- **`src/v2/lens/vacuity.dag`** — a `.dag`-native reader that, for each corpus `*_test.dag`, folds over its parsed tree (via the shared `parse_module`/`tokenize` producer, or the `reference_*_facts_live(pool_roots)` census surface once it is live) to build a `VacuityEvidence` record per assertion (§1.1) and classify it (§2). It mints its **own** `VacuityEvidence` verdict authority — **not** `coverage_defect_vacuous_arm` (name collision, §3, see intro). Reuse the producer; do **not** add a second one.
  - The **observation relation** (`TestClaimObservationRelation`) is the first thing to ground — without it no operand-equality case can be judged (the `rung_5` false-positive). It is read from the claim's execution consumer (emit-vs-eval, single-eval, …).
  - **Provenance** (read-back vs transform vs independent transcription) needs the projection→decl def-use edge from `symbol_index_fill`'s reference projection; **comparator/subject** need the per-operand call-graph (`reference_deps`). Until those are live the evidence is incomplete → `Unknown` → advisory/refuse, never a wall.
- **`src/v2/lens/vacuity_test.dag`** — floor-discovered witness whose **RED controls are the §2 falsification suite** (each row a claim whose expected verdict is asserted; `rung_5`-shaped `EmitVsEval` must classify `ProvenIndependent`, a single-eval read-back must classify `ProvenDuplicate`, an evidence-missing case must classify `Unknown`), plus roster-diff teeth (`count_not_in_roster` / `count_stale_roster`, mirroring `inert_carrier_test.dag`). Ships green-by-execution on a **bounded witness corpus** first, full-corpus census gated.

**Gating + dissolution (same producer trigger as `reference_deps`):** the full-corpus parsed fold is *"scaffold-only until interpreter parse-per-member cost is bounded"* + the `symbol_index_fill` reference projection reaches steady state. **Only `ProvenDuplicate` ever gates**; `ProvenIndependent` is clean and `Unknown` stays advisory or refuses (§5). So the lens ships as a **consumer-aware candidate census** (advisory) and promotes an evidence class to fail-closed only once that class is *provably* `ProvenDuplicate` — never on operand syntax. The lens never dissolves; its roster of tolerated `ProvenDuplicate`/`Unknown` sites dissolves as they are fixed or grounded.

## 4. Frontier placement (evidence-gated, per [expressibility-frontier](expressibility-frontier.md))

The frontier is drawn by *evidence*, not by signal shape:

- **`ProvenDuplicate` is the ① wall** — but a case reaches it only when the observation relation + provenance + comparator are all grounded and prove a single-path duplicate with no oracle. Syntactic reflexivity does **not** reach it (`rung_5`).
- **`Unknown` is the fail-closed residue** — an evidence gap is a *typed, located, counted* `Unknown` (advisory, or a refusal), never a silent clean and never a widen (§5). Its frequency is the backlog of provenance/observation edges still to ground.
- **`ProvenIndependent` is clean** — including behavioral pins, distinct-consumer equivalence (`EmitVsEval`), and temporal/external-authority goldens.
- **Advisory-first, then wall per evidence class** — like inert-layer/doc-reachability, ship as a ranked `ProvenDuplicate`/`Unknown` census with a named shrinking roster; promote a *class* to fail-closed only once that class is provably `ProvenDuplicate` over the corpus. Each roster entry names its dissolve-on.

## 5. Reuse map (do not fork — §3)

| need | reuse | file |
| --- | --- | --- |
| structural equality of two operand subtrees (one evidence input, never the verdict) | `exact_structural_equality_zip_fold` | `src/v2/std/exact_structural_equality_zip_fold_predicate.dag` |
| the vacuity verdict — **mint `VacuityEvidence`; do NOT reuse `coverage_defect_vacuous_arm`** (name collision: that = exhaustive-empty match arm) | new `VacuityEvidence` authority | `src/v2/lens/coverage.dag:14-27` (the colliding name, for contrast) |
| roster + stale-roster ratchet + live-vs-declared count idiom | `inert_carrier` lens | `src/v2/lens/inert_carrier.dag` |
| parsed-tree native corpus fold (the single swappable producer — do NOT add a second) | `parse_module` / `tokenize` + `reference_*_facts_live(pool_roots)` census | `v2.compiler.parse` · `v2.compiler.tokenize` · `src/v2/lens/reference_deps.dag` · `src/v2/lens/module_graph.dag` |
| the observation/consumer relation (the crux dimension) | emit-vs-eval execution consumers | `rung_5.dag:31-45` · `rung_5_6_common.dag:40-48` · `emit_host.dag:412-433` |
| each test's own module path / containment position (provenance) | `SymbolIndex` lookup / containment | `src/v2/std/symbol_index.dag` |
| all five claim variants (classifier must be total) | `TestClaim` (`Compiles/Diagnostic/Equals/StructuralEquals/RoundTrip`) | `src/v2/std/verification.dag:159-195` |

## 6. Build order (the review's "strongest safe next step"; no v1 edit at any step)

1. **Ground `TestClaimObservationRelation` + `VacuityEvidence`, and fix the taxonomy contradictions** — model the observation/consumer relation (emit-vs-eval / single-eval) and the `ProvenDuplicate | ProvenIndependent | Unknown` verdict. This is the prerequisite: without it *nothing* is a wall. Mint `VacuityEvidence` (not `coverage_defect_vacuous_arm`).
2. **Land the classifier as a consumer-aware candidate census (advisory)** — total over all five `TestClaim` variants; emits `ProvenDuplicate`/`ProvenIndependent`/`Unknown`, gates on nothing yet. `vacuity_test.dag` proves it green-by-execution against the **§2 falsification suite** (the `rung_5` `EmitVsEval` row must classify `ProvenIndependent`; a single-eval read-back `ProvenDuplicate`; an evidence-missing case `Unknown`). Buildable over a bounded witness corpus now.
3. **Promote a class to fail-closed only when provably `ProvenDuplicate`** — as the `symbol_index_fill` provenance/reference edges go live, the read-back and helper-alias classes reach `ProvenDuplicate` and gate; everything else stays `Unknown`/advisory. Never gate on syntax.

The deliverable is the *displaced cost*: the operator's recurring manual deletion of vacuous witnesses (`bf3e33e15c` roster-count change-detectors, the read-back class) becomes a standing, counted, located refusal — while a `rung_5`-style equivalence test is *never* false-flagged.

## 7. Open

- **Observation-relation modeling (the prerequisite).** `TestClaimObservationRelation` must be grounded before any gate. Its variants (`SameEval`, `EmitVsEval`, `SingleEvalOutput`, …) are read from the claim's execution consumer; enumerate the real consumer kinds in the corpus (`rung_*` emit-vs-eval, plain interpreter claims) before fixing the enum.
- **Provenance edges gate the walls, not syntax.** The `ProvenDuplicate` classes (read-back, helper-alias) need the projection→decl def-use edge (`symbol_index_fill`) + per-operand call-graph (`reference_deps`), *scaffold-only until parse-per-member cost is bounded*. Until then those classes stay `Unknown`/advisory. Do **not** add a second corpus producer (operator constraint 2026-07-10).
- **Self-application (the §7 recursion).** `coverage/near_miss_vacuous_not_parallel.dag` and `coverage/sibling_degenerate_not_hollow.dag` assert `X != Y` on two declared constants — under this classifier they are **`ProvenDuplicate` candidates themselves** (a single-eval comparison of two literals, no oracle). The vacuity lens must classify its own taxonomy witnesses; that is the discriminating self-test.
- **Relationship to `parallel-authority`.** The two-independent-transcriptions case is *not* vacuity — it is a distinct parallel-authority defect. Confirm whether that lens exists or is a sibling to build; keep the two verdicts separate (the review's point).
- **v1-deletion timing.** Nothing here touches `src/v1`; the classifier ships over the parsed-tree producer regardless of v1's presence.

## Dissolution trigger (DESIGN §6)

Delete this doc when the classifier is fully built and self-describing in the carriers: `TestClaimObservationRelation` and `VacuityEvidence` are grounded, the classifier is total over the five `TestClaim` variants and green against the §2 falsification suite (including classifying its own `X != Y` taxonomy witnesses), `ProvenDuplicate` classes gate while `Unknown` refuses — at which point "a test needs an oracle independent of the code, proven by its observation relation" is enforced by execution and this prose is superseded by the lens (the lens never dissolves; this doc does, per DESIGN §6's mark-on-carrier-is-authority).
