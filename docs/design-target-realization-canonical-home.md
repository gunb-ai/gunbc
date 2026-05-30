# Target-realization substrate — canonical home design

**Owner:** Target Realization Manager (keen-heron-687)
**Status:** Updated 2026-05-30 post PR #3938 merge + Modeling DFS worker-brief routing (msg_643a013d). §1 placement decision revised to match Modeling DFS addendum (extend `v4.std.target_model` rather than new sibling file); §5 consumer reference corrected from v3 `05_emit` to v4 `06_translate`. Original Option B reasoning preserved in §1 history note for the PR record.
**Scope:** Where `TargetAtomRealization` (SG-1), `TargetTypeExpressionProjection` (SG-2), and `TargetCollectionRealization` (SG-3) carrier type definitions live. Per-language rows remain in `extdeps/languages/<lang>.dag`.
**Source PR:** #3938 §10.1, §10.2, §10.3, §10.6, §11.1, §11.3.
**Modeling DFS ratification:** proud-pike-680 worksheet addendum 2026-05-30 (scaffold disposition, see §3).

---

## §1. The placement decision

The §10.1 brief states: *"per-language rows live in `extdeps/languages/<lang>.dag`; carrier type definition lives ONCE in the canonical home (target-realization substrate, sibling to SG-2's `TargetTypeExpressionProjection`)."*

Three candidate placements were evaluated:

| Option | Placement | Verdict |
| ------ | --------- | ------- |
| A | Extend `std/target_model.dag` in place | **Accepted (Modeling DFS ratified 2026-05-30 msg_643a013d).** That file's stated scope expands to host the three realization carriers alongside the existing TargetModel authority. The P2 single-authority claim is preserved by keeping `target_model.dag:3` ("single authority per INVARIANTS P2 — consumers import; do not redeclare") as the file-level invariant for everything in the file. The live `TargetModel` consumers — `src/v4/compiler/00_compile.dag:24`, `src/v4/compiler/05_emit.dag:12`, `src/v4/compiler/06_translate.dag:16,67` — import directly from `v4.std.target_model`; realization carriers added to the same module will be imported the same way. Note: `src/v4/compiler/07_target_carriers.dag` does NOT import `TargetModel` (it owns only `LanguageModel = Node` and `TargetSource = String` as carrier-shim aliases); the file's header refers to `v4.std.target_model` as the authority pointer but is not itself an importer. An earlier draft of this spec described `07_target_carriers.dag` as importing `TargetModel`; that was wrong and is corrected here. |
| B | One new file `std/target_realization.dag` hosting all three carriers | **Initially proposed (Option B) but superseded by Modeling DFS Option A.** Original rationale: concept-DAG-layer separation between "target model identity" and "target representation choice." Rationale rejected because concept-layer placement is a modeling decision (proud-pike-680 authority per §11.1), and they ratified absorption into target_model.dag. Co-location of the three realization carriers in one file still holds — they now co-locate inside target_model.dag. |
| C | Three files (one per carrier) | **Rejected.** Splits a single concept layer across three files for no win; cross-section invariant becomes cross-file. |

**Decision: Option A.** Extend `src/v4/std/target_model.dag`. Module remains `v4.std.target_model`. The three realization carriers (`TargetAtomRealization`, `TargetTypeExpressionProjection`, `TargetCollectionRealization`) plus supporting target-side vocabulary co-locate with the existing `TargetModelBundle` declarations.

## §2. Carrier sketches

Substrate sketch only — not authoritative content. Final field shapes are the worker's deliverable post §8 sign-off, constrained by the §10.1 / §10.2 / §10.3 worksheets and the §10.6 bidirectionality requirement.

```dag
// Additions to existing module v4.std.target_model
// (joins existing TargetModelBundle declarations)

import v4.std.node { Node, Symbol }       // already present for Symbol edge-key data
import v4.std.collection { List, Optional }
import v4.std.diagnostic { Diagnostic }

// SG-1 — canonical atom realization (Symbol, Bool, Char, ...)
type TargetAtomRealization {
  source_carrier: Node                   // canonical source-node identity; NOT raw spelling
  target_model: TargetModel
  type_form: TargetTypeExpression        // instance of SG-2 substrate below; do NOT re-coin
  value_form: TargetValueTemplate        // parametric over source atom value; NOT hardcoded
  constructor_form: Optional<TargetConstructorTemplate>
  display_name: Symbol                   // diagnostic only; never authority
}

// SG-2 — canonical type-expression projection (per-connective)
type TargetTypeExpressionProjection {
  target_model: TargetModel
  atom_form: TargetAtomTypeShape
  conj_form: TargetRecordTypeShape
  disj_form: TargetSumTypeShape
  arrow_form: TargetFunctionTypeShape
  cardinality_form: TargetGenericApply
  instantiation_form: TargetGenericApply
}

// SG-3 — canonical collection realization
type TargetCollectionRealization {
  source_carrier: Node                   // Set, Map, List, ...
  target_model: TargetModel
  primary_form: TargetRepresentation
  constraints: List<RequiredTraitWitness>
  alternative_forms: List<TargetRepresentation>
  fallback_diagnostic: Diagnostic
}

// Supporting target-side vocabulary — co-located; one authority.
type TargetTypeExpression { ... }
type TargetValueTemplate { ... }
type TargetConstructorTemplate { ... }
type TargetAtomTypeShape { ... }
type TargetRecordTypeShape { ... }
type TargetSumTypeShape { ... }
type TargetFunctionTypeShape { ... }
type TargetGenericApply { ... }
type TargetRepresentation { ... }
type RequiredTraitWitness { ... }
```

**Hard cross-section invariant (§10.1 verbatim):** `TargetAtomRealization.type_form` is typed as `TargetTypeExpression` — the SG-2 substrate — not a parallel atom-only type vocabulary. The in-file co-location of both carriers makes this enforceable by reading one file.

**Bidirectional readability (§10.6):** every `*_form` field above must admit reading in both emission (`Node → target syntax`) and ingestion (`target syntax → Node`) directions. Forms encoding only one direction (e.g., a free-text emit template that can't be pattern-matched in reverse) are rejected at worker-brief level. The single-fixture round-trip falsification probe of §10.6 is the gate.

## §3. Scaffold-reconciliation disposition (Modeling DFS ratified)

`extdeps/languages/rust.dag:263-364` carries 94 Symbol-tagged catalog entries across five families: `rust_std_projection_*`, `rust_surface_spelling_*`, `rust_repr_*`, `rust_inhabitant_*`, `rust_coercion_field_*`. They are used inside `rust_facts_*` Conj bundles but have **zero compiler/translate consumers** — inert sentinels for the emit path, real as parallel name-keyed scaffold for the P2-authority question.

Per Modeling DFS Mgr (proud-pike-680) addendum ratification 2026-05-30, the SG-1 worker brief's scaffold-reconciliation disposition is:

| Family member | Disposition | SG-1 worker action |
| ------------- | ----------- | ------------------ |
| `rust_std_projection_bool`, `rust_std_projection_char` | **ABSORB** into `TargetAtomRealization` Rust rows as `type_form`/`value_form` slots (Node-keyed, not spelling-keyed) | Land Rust rows for Bool + Char that subsume these sentinels; delete the sentinel `data` lines in the same PR |
| `rust_surface_spelling_bool`, `rust_surface_spelling_char` | **ABSORB** as above | Same |
| `rust_inhabitant_bool`, `rust_inhabitant_char`, `rust_inhabitant_field_bool`, `rust_inhabitant_field_char` | **ABSORB** to the extent the Conj-bundle fields they tag become `value_form` slots; otherwise leave for SG-3-adjacent work | Worker brief enumerates which absorb, which stay |
| `rust_repr_bool` | **ABSORB** as `value_form` representation discriminator | Same |
| Symbol (the SG-1 carrier itself) | **GREENFIELD** — no rust.dag sentinel exists | Row is net-new |
| All `rust_std_projection_int*`, `_uint*`, `_float*`, `_str`, `_unit`, `_never` and parallel `_surface_spelling_*` / `_repr_*` / `_inhabitant_*` / `_coercion_field_*` | **OUT OF SG-1 SCOPE** — dissolution-on-arrival when numeric / alias realization substrate exists | Worker brief explicitly forbids touching; flagged as separate follow-on work item under TR Mgr |

**Forbidden by brief:** leaving bool/char sentinels live alongside the new `TargetAtomRealization` Rust rows for the same atoms. That is exactly the third-authority outcome the canonical home is supposed to eliminate.

**Forbidden by brief:** SG-1 worker migrating numeric/str/unit/never families. That widens scope past the SG-1 worksheet and pre-empts a future modeling decision (numeric atoms may want a different carrier shape — `TargetNumericRealization` with width/signedness/overflow-policy fields).

## §4. Dispatch boundary

This spec is **substrate placement + carrier sketch + scaffold disposition**. It is NOT a worker brief.

The worker briefs (one per SG class — SG-2 first per Modeling DFS dispatch order) are the §10.1 / §10.2 / §10.3 *Tightened worker briefs* verbatim, **augmented** with:
- the canonical-home placement `src/v4/std/target_model.dag` (module `v4.std.target_model`) from §1 above
- the v4 consumer path `src/v4/compiler/06_translate.dag` from §5 above
- the scaffold-reconciliation table from §3 above (SG-1 only)
- the §10.6 bidirectional-readability falsification probe as acceptance criterion

The worker brief will NOT decide field-level shape of the supporting target-side vocabulary (`TargetTypeExpression`, `TargetValueTemplate`, etc.) — those are co-substrate authoring within the same PR, but the *cross-carrier invariants* (§10.1 cross-section, §10.6 bidirectionality) are non-negotiable.

## §5. Coordination

| Sibling manager | Touchpoint | Status |
| --------------- | ---------- | ------ |
| Modeling DFS (proud-pike-680) | §10.0 worksheet gate; scaffold-reconciliation disposition | **Gate cleared 2026-05-30** for first-artifact spec |
| Compiler Spine (smart-stag-871) | `src/v4/compiler/06_translate.dag` type + value translate paths consume the realization rows; `Instantiation` consumer added at the type-expression projection. (v4 emit = translate + grammar-inverse serialize, NOT v3 `05_emit` templates.) | **Notify on each SG dispatch** — Compiler Spine owns the consumer-side refactor; TR owns the substrate carrier + rows |
| Runtime/TestClaim (pending) | §10.6 bidirectional-readability falsification probe — Rust fixture round-trip | **Notify on spawn** |
| Ladder/Fixture (keen-crab-361) | Phase 1 fixture (`nat_semiring.dag`) exercises rungs 0-2; SG-1 row landing affects rung-2 emit-clean gate | **No coordination needed pre-dispatch** — fixture choice is theirs; SG-1 row lands regardless |
| Close/Receipt (sharp-otter-407) | Disposition vocabulary for the falsification-probe verdict (SUBSTRATE / GATED / DONE) | **No coordination needed pre-dispatch** |
| Self-host/Release (nimble-crane-490) | T-15, T-36 — SG-1 row absence currently masks Symbol realization in self-emit | **No coordination needed pre-dispatch** |

## §6. Open questions for operator at §8 sign-off

None blocking. The Modeling DFS scaffold disposition resolved the only TR-side modeling decision. Field-shape of supporting target-side vocabulary (§2 placeholders) is worker authoring inside the brief — not an operator decision.

---

**End of spec.** No commits to substrate. No worker dispatch. Awaiting §8 D1-D7 operator sign-off per PR #3938.
