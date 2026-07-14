# The vacuity lens — tests that reaffirm the code instead of checking it

> A lens that flags **tautological tests**: a test whose expected side is *not independent* of the code under test, so it can only ever agree — it re-affirms *what the code literally is*, never *what it should do*. DESIGN refs: §5 ("a check that re-states a constraint the model already carries is a second representation of it… satisfied by editing the declaration while the realizer still lies"; the "spec-without-execution" trap — a grep-passing, type-checking test that runs green but discriminates nothing), §6 (lens as residue over the Node tree; coverage-by-illusion), §3 (single authority — the vacuity concept already has a home, `coverage_defect_vacuous_arm`; do not fork it). Sibling of the [inert-layer lens](inert-layer-lens.md) family — inertness is *"nothing reaches this concept"*; vacuity is *"this check reaches the concept but cannot fail on it."* Successor mechanism to the one-shot [mechanism-inventory audit](mechanism-inventory-red-controls.md), whose finding #5 (~28 floor-wired lenses are synthetic-only; can execute yet catch no real corpus violation) is exactly the class this makes standing and observable.

## 1. The definition — "2FA for testing" (the independent-oracle rule)

A real test needs an **oracle independent of the code under test**. A tautological test authenticates with the *same factor twice*: its expected value is derived from the same source-of-truth as its actual value, so the assertion is structurally incapable of going RED. Stated once:

> A test is **vacuous** iff its expected side is not independent of its actual side — no second factor. Independence is the property; vacuity is its absence.

This is the operator's framing (2026-07-14) and it is DESIGN §5 verbatim: a check that re-states a constraint the model already carries is a *second representation of it*, and a second representation of a fact is not a check of it — it is redundant work (§2) wearing a test's clothes.

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

## 3. The coverage seam (why a pure-`.dag` lens would be self-vacuous)

The trap this lens must not fall into is *its own subject*. Every live-corpus enumeration in this tree is host-fed (`concept_index` declarations, floor witness discovery by marker scan, the `inert_carrier` / `doc_reachability` `*_project.rs` bridges). A **pure-`.dag`** vacuity lens could only run over *synthetic* claims — it would be exactly a mechanism-inventory **Group C** lens: executes, passes the #5433 inert-lens backstop, and catches zero real corpus violations. A vacuity lens that is itself vacuous over the real tree is the worst possible outcome (§5, and §7 — the compiler's own dead check is a substrate fact).

So the lens **must** read the live corpus, which means a small additive host bridge, exactly the established non-load-bearing pattern:

- **`src/v1/stage0/src/vacuity_project.rs`** — walks `*_test.dag`, enumerates each `test fn` body / `TestClaim` assertion, and exposes scalar/list verdicts through the **same additive corpus-gate builtin seam** as `inert_carrier_names_live` / `doc_graph_orphan_count` (it does **not** touch `cli_run.rs`'s #5433 closure). Signals #1/#2 need each assertion's operand ASTs + a per-operand call-graph; #3 needs the projection→data-decl def-use edge.
- **`src/v2/lens/vacuity.dag`** — the `.dag` reader: folds the host-provided rows, diffs the vacuous set against a **named shrinking roster** (deliberately-tolerated cases with a dissolve-on), tags its concern with the existing `coverage_defect_vacuous_arm` (§3 — no new vacuity authority).
- **`src/v2/lens/vacuity_test.dag`** — floor-discovered witness with a **discriminating RED control**: a synthetic reflexive claim flagged vacuous=true (RED on revert), a synthetic genuine claim flagged vacuous=false, and roster-diff teeth (`count_not_in_roster` / `count_stale_roster`, mirroring `inert_carrier_test.dag`).

**Dissolution trigger:** when `.dag` gains compile-graph / def-use access (gunbc#5364, the same trigger the inert-carrier Tier-2 and doc-graph orphan-half wait on), the host walk folds into a pure `.dag` reader and `vacuity_project.rs` deletes. The lens itself never dissolves (vacuity is a standing property); its roster dissolves to empty as the tautological tests are fixed or deleted.

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
| host-fed corpus enumeration through the additive builtin seam | `inert_carrier_project.rs` / `doc_reachability_project.rs` | `src/v1/stage0/src/*_project.rs` |
| floor-discovered witness + RED control idiom | `inert_carrier_test.dag` | `src/v2/lens/inert_carrier_test.dag` |
| test-claim AST shapes (operands to inspect) | `TestClaim` (`EqualsClaim`/`StructuralEqualsClaim` `lhs`/`rhs`) | `src/v2/std/verification.dag:159-203` |

## 6. Build order (displaced cost first — §6)

1. **Signal #1 reflexive wall + host bridge scaffold** — smallest hard wall; proves the bridge + witness + RED control end-to-end (green-by-execution, not grep). Advisory roster empty-or-tiny.
2. **Signal #3 advisory census** — the high-volume operator pain (hardcoded filenames/module-names). Ranked report + roster; needs the projection→decl def-use edge in the bridge. No gate on syntax.
3. **Signal #2 shared-authority wall** — needs the per-operand call-graph; folds in once #1's bridge carries call-graph facts.

The deliverable is the *displaced cost*: the operator's recurring manual deletion of vacuous witnesses (`007468e673`, `6a22be97c7`, `bf3e33e15c`, the `ManualTcConjEqualsRefl` reflexive anchor) becomes a standing, counted, located refusal instead of hand-diligence re-paid each conversation.

## 7. Open

- **The bridge's def-use depth.** Signal #1 needs only operand-subtree equality (cheap). #2/#3 need call-graph / projection→decl edges — confirm what `stage0` already exposes vs what the bridge must compute, before committing #2/#3 to the same PR as #1.
- **Roster seed.** The census's ~24 module-string + ~62 path-string hits must be triaged (vacuous read-back vs legitimate transform) to seed the #3 roster; #1 reflexive cases enumerated separately. Do the triage as the census step, not by hand-guessing.
- **Interaction with `near_miss_vacuous_not_parallel`.** The existing `coverage/near_miss_vacuous_not_parallel.dag` witness (`VacuousArm != ParallelAuthority`) is a *near-miss* unit for the coverage taxonomy, not a corpus reader — keep it; the vacuity lens is the corpus-reading mechanism it gestures at.

## Dissolution trigger (DESIGN §6)

Delete this doc when the vacuity lens family is fully built and self-describing in the carriers: signals #1/#2 live as fail-closed floor witnesses with drained rosters, #3 live as a ranked advisory, and the host bridge folded into a pure `.dag` reader at gunbc#5364 — at which point "a test must have an oracle independent of the code under test" is enforced by execution and this prose is superseded by the lens (the lens never dissolves; this doc does, per DESIGN §6's mark-on-carrier-is-authority).
