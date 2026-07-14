# The vacuity lens — tests that reaffirm the code instead of checking it

> A lens that flags **tautological tests**: a test whose expected side is *not independent* of the code under test, so it can only ever agree — it re-affirms *what the code literally is*, never *what it should do*. DESIGN refs: §5 ("a check that re-states a constraint the model already carries is a second representation of it… satisfied by editing the declaration while the realizer still lies"; the "spec-without-execution" trap — a grep-passing, type-checking test that runs green but discriminates nothing), §6 (lens as residue over the Node tree; coverage-by-illusion), §3 (single authority — the vacuity concept already has a home, `coverage_defect_vacuous_arm`; do not fork it). Sibling of the [inert-layer lens](inert-layer-lens.md) family — inertness is *"nothing reaches this concept"*; vacuity is *"this check reaches the concept but cannot fail on it."* Successor mechanism to the one-shot [mechanism-inventory audit](mechanism-inventory-red-controls.md), whose finding #5 (~28 floor-wired lenses are synthetic-only; can execute yet catch no real corpus violation) is exactly the class this makes standing and observable.

## 1. The definition — a vacuous test is *redundant* with the code (the root)

The root is **redundancy**, not testing technique. A vacuous test is a *second representation* of a fact the code already carries — it moves 1:1 with the code (edit the code, edit the test, lockstep), so it is a **change-detector, not a check**. Stated once (operator framing, 2026-07-14):

> A test is **vacuous** iff its assertion is a deterministic 1:1 function of the code's own structure with **no referent independent of it**. It carries zero information about *correctness* — only a duplicate encoding of *state*.

This is DESIGN §2 (minimize redundancy) / §3 (single authority) at the test↔code seam, and DESIGN §5 verbatim: "a check that re-states a constraint the model already carries is a second representation of it." Kin to the **duplicate-work** open thread — the same fact materialized twice.

**The three consequences (the §1 time lens — this is the *lesson*, not just the definition):**
- **complexity/cost** — a maintenance tax paid forever: every structural edit must touch two places (§2's "defers cost onto a later fixer").
- **safety** — zero harm reduction: it cannot catch a *behavioral* error, only a transcription typo, so it *looks* like coverage while providing none (§5 coverage-by-illusion).
- **the fix is not "delete the test" — it is §5 construction-over-validation.** A fact checkable by 1:1 restatement is *structural*, and structural facts should be **derived from one authority** (made unwritable-if-wrong), never validated after the fact by a mirror test. So the lens's *product* is the **located duplicated authority** (the "output is the root, not the symptom" discipline of the duplicate-work lens, §6 moat) — not a delete list.

### 1.1 The discriminator — "independent referent" (a spectrum, not syntax)

Vacuity turns entirely on whether the expected value has a **referent independent of the code's own structure**. This is *not* syntactic (`count == N` is legit or vacuous depending on the referent):

| referent of the expected value | example | verdict |
| --- | --- | --- |
| **behavioral** — the correct output for a *specific input* | `parse(s).tokens.length() == 2` · `entries.length() == 1` (parse *this* diff) | legit |
| **external authority** — a datasheet / spec the test independently transcribes | `watt_count(cpu.tdp_watts) == 183` (the datasheet) | legit (fragile 2FA — both sides hand-transcribe) |
| **absent** — the code's own structure, nothing else | `length(perturb_receipt_rows) == 5` · `num_tests_in_file == 78` · a roster-size pin · `f(x) == f(x)` | **vacuous** |

The "2FA / independent oracle" framing is the *symptom*: an absent referent is exactly "no second factor." Reflexive (`f(x)==f(x)`), shared-authority (`emit(n)==emit(n)`), and literal-restatement (`module_path == "..."`, `count == 78`) are all the **absent-referent** class — one root, not three signals.

Census (2026-07-14): 386 `count/length/size == int` assertions in the corpus; 88 against 2+-digit literals — the *shape* is common, but most carry a behavioral referent (parse-output pins) and are legit. The vacuous residue is the **self-referential cardinality pin / read-back** (live instance: `coverage_domain_equivalence_test.dag:110`; the hand-deleted class: `bf3e33e15c`). The discriminator "absent vs behavioral/external referent" is the def-use / provenance question — see §2.1 for why the wall requires it.

## 2. The decidable signal tiers (what is a wall, what only ranks)

Vacuity is decidable on a confidence spectrum. Three signals; the first two are hard walls (①, decidable and grounded), the third is a permanent advisory ratchet (②, needs a semantic link that is undecidable from the test alone — by Rice). Keeping them apart is the whole discipline (the §5 "never"-is-a-trap rule: a ratchet must not masquerade as a wall).

| # | pattern | example | verdict |
|---|---|---|---|
| 1 | **reflexive** — expected and actual are the *same expression* | `f(x) == f(x)` · `xs == xs` | ① **wall** — trivially decidable (structural equality of the two operand subtrees) |
| 2 | **shared-authority** — both sides invoke the same fn / read the same `data` under test, so after inlining the assertion is `g(..) == g(..)` | `emit(n).len == emit(n).len` · `render(x) == render(x)` | ① **wall** — decidable from the call-graph of each operand |
| 3 | **literal-restatement** — expected is a hand-copy of a value the code *literally declares*, with **no transform** between them (the read-back class; the operator's `list.size == 10` and hardcoded filenames/module-names) | in module `gunbc.foo`: `x.module_path == "gunbc.foo"` · `data xs = [..10 items..]; length(xs) == 10` | ② **advisory** — see §2.1 |

### 2.1 Why #3 is advisory, not a wall (the census-proven nuance)

Signal #3 is the highest-*volume* case but the *hardest to wall*, because syntactic form does not decide it. Measured over the live tree (2026-07-14): 692 `*_test.dag` files; ~24 hardcode a module-path string literal, ~62 hardcode a `.dag` file-path string literal — and **the same syntax splits both ways**:

- **vacuous** — `qualified_name.module_path == "gunbc.fleet_converge_emit"` where the subject *stores* that string and the test reads it straight back: a read-back of a stored literal, no transform, pure identity restatement.
- **legitimate** — `qualified_name_to_dotted_string(qn) == "v2.std.text"` (`path_algebra_test.dag:36`): the actual side runs a real *rendering transform*; the string is the independent expected output, a genuine oracle.

The decidable discriminator is therefore **not** "is a filename/module-name hardcoded?" but **"does the actual side perform a transform, or is it a stored-literal read-back?"** — a read-back is `projection(data_decl)` where the projected field is set to exactly the expected literal, with no intervening fn. That is a def-use fact. Where the def-use chain proves *read-back with no transform*, #3 sharpens to a ① wall; where it cannot (the literal may encode an external requirement — "this roster MUST be exactly 10 per spec X" — an independent oracle that happens to share syntax), it stays a ranked ② report. **Under no circumstance does #3 gate on syntax alone** — that would false-red every legitimate rendering test.

## 3. The coverage seam — parsed-tree native fold, NOT a v1 host bridge (v1 is being deleted)

The trap this lens must not fall into is *its own subject*. A **pure-`.dag`** vacuity lens run only over *synthetic* claims would be exactly a mechanism-inventory **Group C** lens: executes, passes the #5433 inert-lens backstop, and catches zero real corpus violations. A vacuity lens that is itself vacuous over the real tree is the worst possible outcome (§5, and §7 — the compiler's own dead check is a substrate fact). So the lens **must** read the live corpus.

**Correction (2026-07-14, after catching up to current main): the corpus read must NOT be a v1 host bridge.** `src/v1` is being deleted; adding functions to `cli_run.rs` builds on dead-code-walking. The tree is migrating every corpus-fact producer **off** v1 host builtins **onto** parsed-tree `.dag`-native folds — `v2.lens.reference_deps` already `import`s `v2.compiler.parse { parse_module }` / `v2.compiler.tokenize { tokenize }` and folds the corpus itself, and the surviving host twins (`build_import_adjacency` in `cli_run.rs`) are **isolated as a single swappable producer** under a standing operator constraint (2026-07-10): *"Do not add a second edge producer beside `dependency_resolution_facts_live`; the swap replaces its body."* A vacuity `*_project.rs` / `cli_run.rs` bridge would violate exactly that constraint and be deleted with v1.

So the vacuity lens rides the **same parsed-tree native producer** the reference/module-graph lenses do:

- **`src/v2/lens/vacuity.dag`** — a `.dag`-native reader that, for each corpus `*_test.dag`, folds over its parsed tree (via the shared `parse_module`/`tokenize` producer, or the `reference_*_facts_live(pool_roots)` census surface once it is live) to extract each `test fn` / `TestClaim` assertion and inspect its operands. Diffs the vacuous set against a **named shrinking roster**, tags its concern with the existing `coverage_defect_vacuous_arm` (§3 — no new vacuity authority). Reuse the producer; do **not** add a second one.
  - Signal #1 (reflexive) needs only operand-subtree equality over the parsed assertion — cheap, buildable on the parsed fold directly.
  - Signal #3 (self-identity read-back) needs each test's own module path (from the containment position / `SymbolIndex`) + its string-literal reference sites — both available on the parsed tree; the discriminator "transform vs read-back" is the projection→decl def-use edge, which arrives with `symbol_index_fill`'s reference projection.
  - Signal #2 (shared-authority) needs the per-operand call-graph — the `reference_deps` reference-edge surface, live at the same steady state.
- **`src/v2/lens/vacuity_test.dag`** — floor-discovered witness with a **discriminating RED control**: a synthetic reflexive claim flagged vacuous=true (RED on revert), a synthetic genuine claim flagged vacuous=false, and roster-diff teeth (`count_not_in_roster` / `count_stale_roster`, mirroring `inert_carrier_test.dag`). Ships green-by-execution on a **bounded witness corpus** first (the parse-per-member cost caveat below), full-corpus census gated.

**Gating + dissolution (same as `reference_deps`, not #5364-host):** the full-corpus parsed fold is *"scaffold-only until interpreter parse-per-member cost is bounded"* + the `symbol_index_fill` reference projection reaches steady state — the identical trigger the `reference_resolution_facts_live` census waits on. Until then: signal #1 is buildable now over a bounded witness corpus (advisory, RED-controlled); the whole-corpus census + signals #2/#3 land as the producer goes live. The lens itself never dissolves (vacuity is a standing property); its roster dissolves to empty as the tautological tests are fixed or deleted, at which point #1/#2 flip advisory → fail-closed wall.

## 4. Frontier placement (advisory-first, per [expressibility-frontier](expressibility-frontier.md))

- **Signals #1/#2 are ① walls** — reflexive and shared-authority vacuity are decidable and grounded; a new one is a genuine defect. Ship as a fail-closed floor witness once the roster of pre-existing cases is drained.
- **Signal #3 is the ② residue** — permanently advisory (the "is this literal an independent requirement or a transcription?" judgment needs domain knowledge). Ranked report: rank a #3 candidate by how structural the restated fact is (module-path / file-path / declared-cardinality literals rank highest — the operator's named pain).
- **Advisory-first, then wall** — like inert-layer and doc-reachability, ship as a ranked report with a named shrinking roster; promote #1/#2 to fail-closed once the corpus is clean enough that a new reflexive/shared-authority test is a real defect rather than expected legacy. Each roster entry names its dissolve-on (the PR that fixes or deletes the tautological test).

## 5. Reuse map (do not fork — §3)

| need | reuse | file |
| --- | --- | --- |
| structural equality of two operand subtrees (signal #1) | `exact_structural_equality_zip_fold` | `src/v2/std/exact_structural_equality_zip_fold_predicate.dag` |
| the vacuity concern key (do not mint a new one) | `coverage_defect_vacuous_arm` / `VacuousArm` | `src/v2/lens/coverage.dag:23,58` |
| roster + stale-roster ratchet + live-vs-declared count idiom | `inert_carrier` lens | `src/v2/lens/inert_carrier.dag` |
| parsed-tree native corpus fold (the single swappable producer — do NOT add a second) | `parse_module` / `tokenize` + `reference_*_facts_live(pool_roots)` census | `v2.compiler.parse` · `v2.compiler.tokenize` · `src/v2/lens/reference_deps.dag` · `src/v2/lens/module_graph.dag` |
| each test's own module path / containment position (signal #3) | `SymbolIndex` lookup / containment | `src/v2/std/symbol_index.dag` |
| floor-discovered witness + RED control idiom | `inert_carrier_test.dag` | `src/v2/lens/inert_carrier_test.dag` |
| test-claim AST shapes (operands to inspect) | `TestClaim` (`EqualsClaim`/`StructuralEqualsClaim` `lhs`/`rhs`) | `src/v2/std/verification.dag:159-203` |

## 6. Build order (displaced cost first — §6; no v1 edit at any step)

1. **Signal #1 reflexive, over a bounded witness corpus** — the smallest hard wall, buildable *now* on the parsed fold without waiting for the full-corpus producer: `v2.lens.vacuity` folds the parsed assertion, flags `lhs ≡ rhs` via `exact_structural_equality_zip_fold`; `vacuity_test.dag` proves it green-by-execution with a RED control. Advisory, roster empty-or-tiny. This is the end-to-end proof and the only step landable ahead of the producer trigger.
2. **Signal #3 advisory census** — the high-volume operator pain (hardcoded filenames/module-names). Ranked report + roster; needs each test's module path (`SymbolIndex`) + the projection→decl def-use edge (`symbol_index_fill` reference projection). No gate on syntax. Lands as the producer/reference-projection goes live.
3. **Signal #2 shared-authority wall** — needs the per-operand call-graph (`reference_deps` reference edges); folds in at the same steady state.

The deliverable is the *displaced cost*: the operator's recurring manual deletion of vacuous witnesses (`007468e673`, `6a22be97c7`, `bf3e33e15c`, the `ManualTcConjEqualsRefl` reflexive anchor) becomes a standing, counted, located refusal instead of hand-diligence re-paid each conversation.

## 7. Open

- **Producer readiness (the gating question).** Signal #1 is buildable now over a bounded witness corpus (operand-subtree equality on the parsed fold). #2/#3 ride the `reference_*_facts_live` census + `symbol_index_fill` reference projection, which are *scaffold-only until parse-per-member cost is bounded* — so they land when that trigger fires, not before. Confirm the current cost bound before committing #2/#3 to a PR; do **not** add a second corpus producer to unblock them (operator constraint 2026-07-10).
- **Roster seed.** The census's ~24 module-string + ~62 path-string hits must be triaged (vacuous read-back vs legitimate transform, e.g. `path_algebra_test.dag`'s render oracle) to seed the #3 roster; #1 reflexive cases enumerated separately. Do the triage as the census step, not by hand-guessing.
- **Interaction with `near_miss_vacuous_not_parallel`.** The existing `coverage/near_miss_vacuous_not_parallel.dag` witness (`VacuousArm != ParallelAuthority`) is a *near-miss* unit for the coverage taxonomy, not a corpus reader — keep it; the vacuity lens is the corpus-reading mechanism it gestures at.
- **v1-deletion timing.** v1 is being deleted soon; nothing in this design touches `src/v1`. If the parsed-tree producer is not yet live-cheap when v1 goes, signal #1 (bounded-corpus, producer-independent) still stands; #2/#3 wait on the producer regardless of v1's presence.

## Dissolution trigger (DESIGN §6)

Delete this doc when the vacuity lens family is fully built and self-describing in the carriers: signals #1/#2 live as fail-closed floor witnesses with drained rosters, #3 live as a ranked advisory, and the host bridge folded into a pure `.dag` reader at gunbc#5364 — at which point "a test must have an oracle independent of the code under test" is enforced by execution and this prose is superseded by the lens (the lens never dissolves; this doc does, per DESIGN §6's mark-on-carrier-is-authority).
