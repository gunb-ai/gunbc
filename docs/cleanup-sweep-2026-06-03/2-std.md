# cleanup-sweep #2: `src/v4/std/` concerning-patterns catalog

**Session:** fierce-lynx-390  
**Date:** 2026-06-03  
**Scope:** 50 `.dag` files under `src/v4/std/` (~14.9k LOC; brief said 49 — `patterns.dag` MIRROR is the extra file)  
**Mode:** read-only audit during hard freeze — no code changes, diagnostic catalog only.

## Slice: `src/v4/std/` — overall scariness: 🔴

Two megatowers (`target_model.dag` 3442L, `node.dag` 1298L) plus repeated catalog-lookup, digest, and closed-vocab-table patterns across the slice. Many honest 🟡 dissolve-on gates, but substrate derived-op surfaces (coproduct Projection, `content_hash` export, Outcome-bearing fold, TotalMap ops) are mostly absent — towers hand-roll around them.

**Slice size flag:** larger than a typical single-concept slice. **`target_model.dag` alone could be its own sweep slice** if PM splits SG workstreams; `node.dag` hash tower ditto.

---

## Per-file (worst-first)

| Path | | Bridges | Towers | Note |
|------|---|---------|--------|------|
| `src/v4/std/target_model.dag` | 🔴 | SG-1 string/FreeMonoid raw-node projection bridge; SG-5 kernel-atom trait-eligibility 🟡; staged bundle/content_hash row matching | ~6× duplicated missing/unique/ambiguous catalog lookup folds; `content_hash`-as-key scans; `target_collection_repr_kind_from_atom` nested-if closed vocab; trait witness Bool matrix; `source_atom_*` projection folds | INVARIANTS P2 single authority — concept-sink (8 imports); worst hand-roll density |
| `src/v4/std/node.dag` | 🔴 | `loop_bound_edge` Symbol tag 🟡; Connective/Behavior variant-enumeration mirrors 🟡; `combine_hash` cross-module re-export stub; byte_offset 🟡 derived-op interim | ~600L byte_offset cache/digest ladder; `content_hash` merkle fold + Loop/LabeledEdges canonicalization; `canonical_hash_of_*` tables; `bag_hash_digest` | Load-bearing T-1 substrate root (0 imports) — hash authority trapped here |
| `src/v4/std/grammar.dag` | 🔴 | parser-derived production selection 🟡 (DESIGN-2); heavy `target_model` import | 5+ custom `fold_list` state machines; `grammar_symbol_fold` NodeFold; production/rule encoding walks | T-7 data model OK; folds should be grammar catamorphism |
| `src/v4/std/leaf_model_verification.dag` | 🔴 | 16× 🟡 host-transport fixtures (rust/python/go/typescript/runtime-exercise); Symbol scaffolds for toolchain fields | large fixture registry-as-functions; cross-runtime drift shell pointers | honest bridge markers — debt is registry-as-data |
| `src/v4/std/verification.dag` | 🔴 | ImpossibleBugClass variant mirror 🟡; manual-anchor bridge; T-19/T-21 CI policy 🟡 | hand-enumerated TestClaim→label / TestClaimCoproductVariant projections; `fold_list` + `fold_node` test indexing | waiting on coproduct-reflection (L1.1) |
| `src/v4/std/algebra.dag` | 🟡 | `free_monoid_tail` 🟡; `is_empty`/`non_empty` variant-discriminant 🟡; field.reciprocal partiality deferred | ~400L parallel `*_type_node` / `*_node` encoding ladder; FreeMonoid fixpoint placeholder | canonical `fold_list` authority — tower is Node-encoding boilerplate |
| `src/v4/std/datetime.dag` | 🟡 | none major | long RFC3339 validation/match chains (802L) | principled refinement carriers |
| `src/v4/std/integer.dag` | 🟡 | OnesComplement/SignMagnitude stubs 🟡 | width/overflow Node encodings; divide/modulo Outcome plumbing | SL-3229 stubs tracked |
| `src/v4/std/model_core.dag` | 🟡 | T-33 effect/partiality/law-expression scaffolds 🟡 | `model_core_wave1_bool_fact_lookup` match + `Map{lookup:fn}`; law_key→Symbol match | textbook TotalMap candidate (closed Symbol axis) |
| `src/v4/std/grounding.dag` | 🟡 | G0 scaffolds; `map_get` fact bundles | PerLanguageFactBundle Map plumbing; re-exports target_model lookup paths | concept-sink (8 imports) — aggregates others' debt |
| `src/v4/std/dependency.dag` | 🟡 | Wave-2-A staged classifier — dissolve-on T-9 | 3 bespoke NodeFold algebras (dependency/readiness/topological_layers) | should dissolve to graph facts + generic fold |
| `src/v4/std/qualified_name.dag` | 🟡 | QnEmpty/QnCons bootstrap-limitation (FreeMonoid alias blocked) 🟡 | QnFoldStatus `fold_node` algebra; hand-rolled `qn_eq`/`qn_for_all` | entire file dissolves when `FreeMonoid<Symbol>` alias lands |
| `src/v4/std/constraints.dag` | 🟡 | T-9-ground / closedness-bridge / candidate-singleton MVP 🟡 | `solve_constraints` dispatch; identity-MVP candidate enumeration | interim until find_witness + T-25 |
| `src/v4/std/coercion.dag` | 🟡 | T-9 closedness bridge in fold path 🟡 | `coercion_fold_exact_structural` / refinement_widening manual folds | overlaps exact_structural_equality_zip_fold_predicate |
| `src/v4/std/runtime.dag` | 🟡 | fn-ref Symbol pins pending content_hash body digest 🟡 | RuntimeValueNodeProjection match; interpretation registry helpers | T-22 eval cache hashes consumer of node.dag debt |
| `src/v4/std/refinement.dag` | 🟡 | vacuous_stub_pack scaffold | Validation/admits repetition (407L) | mostly principled Refined&lt;T&gt; |
| `src/v4/std/find_witness.dag` | 🟡 | T-25-tail predicate dispatch scaffold | routes to exact_structural / refinement_widening predicates | thin orchestrator |
| `src/v4/std/cardinality.dag` | 🟡 | loop-bound measure witness 🟡 → std.node | `termination_proof_fold_step` NodeFold | coupled to node Loop debt |
| `src/v4/std/diagnostic.dag` | 🟡 | Ratified-Q4 Outcome-bearing fold_node deferred | Outcome bind/merge helpers (359L) | foundational — moderate |
| `src/v4/std/effects.dag` | 🟡 | v3 effects structural mirror note | EffectShape → Node encoding helpers | clean coproducts; encoding tower only |
| `src/v4/std/float.dag` | 🟡 | 🟡 gates on representation | parallel to integer (194L) | numeric stack sibling |
| `src/v4/std/collection.dag` | 🟡 | `map_get` bootstrap-limitation 🟡; `optional_present_witness` 🟡; FiniteSet uniqueness deferred | `map_insert` closure tower; `list_nth`/`list_at_optional` fold accumulators | **TotalMap/TotalPolicy declared L156–160, zero ops** |
| `src/v4/std/lexing.dag` | 🟡 | 🟡 token walk interim | `fold_list` lexer helpers | small |
| `src/v4/std/nat.dag` | 🟡 | Peano encoding 🟡 markers | nat_add/compare folds | foundational — mostly terminal |
| `src/v4/std/node_query.dag` | 🟡 | cycle-break from import-free node.dag; positional/labeled child filters 🟡 | `find_named_child` fold+match; edge-label filters | should become derived Node projections |
| `src/v4/std/artifact.dag` | 🟡 | `artifact_is_generated` hand partition 🟡 | match ArtifactKind → Bool (TotalMap candidate) | gate names substrate Projection partition |
| `src/v4/std/change.dag` | 🟡 | affected-set frontier 🟡 | none significant | carrier-only |
| `src/v4/std/rust_leaf_model_claim.dag` | 🟡 | claim subject coproducts | none | data definitions |
| `src/v4/std/bounded_lattice_completeness.dag` | 🟡 | SG-6 cycle-breaker (algebra↔node_query) | nested `bl_named_child_lookup` match ladder | dissolves with node_query + algebra completeness |
| `src/v4/std/refinement_widening_predicate.dag` | 🟡 | leaf predicate (no extdeps) | triple nested find_named_child match | T-25-tail mechanical prover |
| `src/v4/std/report.dag` | 🟡 | none | `report_reason_to_diagnostic_reason` 2-arm match | TotalMap candidate |
| `src/v4/std/types.dag` | 🟡 | v4 compile-root MIRROR of dsl/std/types.dag 🟡 | none | delete-on dsl/std load |
| `src/v4/std/patterns.dag` | 🟡 | v4 compile-root MIRROR of dsl/std/patterns.dag 🟡 | none | delete-on dsl/std load |
| `src/v4/std/text.dag` | 🟡 | CharAbsent carrier for first-char 🟡 | `fold_list_right` char scan | awaiting FreeMonoid head projection |
| `src/v4/std/logic.dag` | 🟡 | bool fact nat/machine witness stubs 🟡 T-33 | bool algebra node witnesses | Wave-2 scaffold |
| `src/v4/std/patch.dag` | 🟢 | `config_patch_layer` syntactic stub (lowerer expands) | FieldPatch monoid — principled | terminal CP-T416 |
| `src/v4/std/platform.dag` | 🟢 | none | none | clean closed coproducts |
| `src/v4/std/network.dag` | 🟢 | none | none | RFC carriers + HttpMethod |
| `src/v4/std/pipeline.dag` | 🟢 | none | none | stage/carrier closed set |
| `src/v4/std/host_run.dag` | 🟢 | host-process receipt boundary | none | W2 execution evidence |
| `src/v4/std/target_triple.dag` | 🟢 | none | none | composes platform.dag |
| `src/v4/std/module_batch.dag` | 🟢 | none | none | small batch carrier |
| `src/v4/std/verdict.dag` | 🟢 | none | verdict_combine match (principled join) | terminal monoid |
| `src/v4/std/witness.dag` | 🟢 | none | none | terminal Witness coproduct |
| `src/v4/std/projection.dag` | 🟢 | none | none | **projection-as-data types only — NOT derived coproduct/field Projection ops** |
| `src/v4/std/determinism.dag` | 🟢 | none | none | PR-1 carrier-only |
| `src/v4/std/machine.dag` | 🟢 | none | none | width/word carriers |
| `src/v4/std/test_claim_falsification.dag` | 🟢 | 🟡 execution-evidence coproduct dissolution | none | receipt types for #3961 |
| `src/v4/std/exact_structural_equality_zip_fold_predicate.dag` | 🟢 | leaf T-25 prover | fold_node zip-fold (canonical) | model for derived fold |
| `src/v4/std/constraint_satisfaction_predicate.dag` | 🟢 | leaf T-25 prover | none (trivial equality) | smallest predicate file |

---

## Recurring concerning patterns (unmodeled repetitive work)

These are the hand-rolled towers and bridges that recur across files and should collapse to a single modeled surface.

### 1. Catalog lookup fold clone army

- **Pattern:** `Missing | Unique { row } | Ambiguous` trichotomy + `fold_list` scan with per-row key match (`content_hash` or structural equality), duplicated step/init/outcome wiring.
- **Where it recurs:** `target_model.dag` (~6 variants: atom realization, signature realization, collection realization, use-site ownership catalog + node-keyed paths); echoed in `dependency.dag` readiness/topological folds.
- **One shared surface that dissolves it:** derived **`CatalogLookup<K, Row>`** catamorphism (or TotalMap row index + uniqueness witness) over typed catalog carriers — not six copy-pasted fold algebras.

### 2. Closed-vocab tables-as-functions (TotalMap gap)

- **Pattern:** `match` / nested-`if` over closed coproduct arms or Symbol atoms to produce a payload (Bool, Symbol, enum variant) — function-encoded tables instead of substrate data rows.
- **Where it recurs:** `model_core.dag` (`bool_fact_lookup`, `law_key→Symbol`); `artifact.dag` (`artifact_is_generated`); `report.dag` (`report_reason_to_diagnostic_reason`); `target_model.dag` (`target_collection_repr_kind_from_atom`); `verification.dag` (TestClaim arm projections); `node.dag` (Connective/Behavior mirrors).
- **One shared surface that dissolves it:** operational **`TotalMap<K, V>` / `TotalPolicy<K, Context, RowTemplate>`** in `collection.dag` (types declared L156–160 but **zero constructors/helpers**) + registry-as-data migration per `v4/lens/table_decision_tree.dag` L1.13.c.

### 3. Hash/digest ladder trapped in `node.dag`

- **Pattern:** private `hash_combine` / peano-limb / overflow-peel recursion towers; merkle `content_hash` fold with Loop/LabeledEdges canonicalization; `combine_hash` re-export stub because v2 cannot import cross-module.
- **Where it recurs:** authority in `node.dag` (~319 hash/digest refs); consumers in `target_model.dag` (26 `content_hash` uses for row keys), `runtime.dag` (fn-ref pins), `bootstrap.dag` / `05_eval.dag` (hand digest folds — outside slice but downstream).
- **One shared surface that dissolves it:** exported **`content_hash` + `hash_combine`** derived ops (B1 merkle fold as declared primitive) and T-22 **`ByteOffset` ranked offset projection** replacing the fixed-limb byte_offset tower.

### 4. Coproduct Projection / variant reflection (L1.1) absent

- **Pattern:** hand-enumerated arm match for coproduct→field, coproduct→label, coproduct→variant-key, generated-kind partition, repr-kind atom decode.
- **Where it recurs:** `verification.dag`, `node.dag`, `artifact.dag`, `report.dag`, `target_model.dag`, `algebra.dag` (`is_empty`/`non_empty`).
- **One shared surface that dissolves it:** substrate **coproduct reflection / variant-enumeration / arm-discriminant Projection** (docs/design-dissolution-lens.md L1.1). Note: `projection.dag` models Projection *records for lenses* — not these derived ops.

### 5. Bespoke NodeFold / fold_list state machines

- **Pattern:** per-use-case fold init/step types (`QnFoldStatus`, `DependencyFoldState`, `GrammarSymbolFoldState`, catalog lookup states) re-implementing the same traverse-and-accumulate shape.
- **Where it recurs:** `qualified_name.dag`, `dependency.dag` (3×), `grammar.dag`, `target_model.dag`, `verification.dag`, `node_query.dag`, `cardinality.dag`, `exact_structural_equality_zip_fold_predicate.dag` (canonical example).
- **One shared surface that dissolves it:** generic **`fold_node` / `fold_list` derived algebras** including **Outcome-bearing fold (Ratified Q4)** in `diagnostic.dag` — so folds carry Accepted/Rejected natively instead of nested match ladders.

### 6. Bootstrap bridges blocking algebra reuse

- **Pattern:** yellow-gated interim shims because v2 bootstrap cannot express a substrate fact (FreeMonoid generic alias, Witness→Optional map_get, syntactic patch-layer stub).
- **Where it recurs:** `qualified_name.dag` (parallel QnEmpty/QnCons type); `collection.dag` (`map_get` bootstrap); `patch.dag` (`config_patch_layer` lowerer stub); `text.dag` (CharAbsent for head projection).
- **One shared surface that dissolves it:** v2 bootstrap fixes for **FreeMonoid&lt;T&gt; type alias**, **Witness dispatch at Map.lookup**, and **substrate-derived Optional/head/tail projections** — each gate already names its dissolve-on.

### 7. Registry-as-data vs fixture-as-functions

- **Pattern:** large closed fixture sets encoded as function bodies + host-transport shell pointers instead of typed substrate rows keyed by claim id / target / generator.
- **Where it recurs:** `leaf_model_verification.dag` (16× host-bridge fixtures); `target_model.dag` (catalog data + separate lookup fn modules); `verification.dag` (manual-anchor bridge).
- **One shared surface that dissolves it:** **TotalMap registry rows** + `GeneratorProvenance` / `LeafModelFixture<C>` substrate carriers (partially modeled in `artifact.dag` / `leaf_model_verification.dag` but not yet the authoritative lookup surface).

### 8. Concept-sink orchestration layers

- **Pattern:** files importing 6–9 domains and accumulating interim towers from dependencies rather than owning a single concept.
- **Where it recurs:** `target_model.dag` (8 imports), `grounding.dag` (8), `grammar.dag` (9), `leaf_model_verification.dag` (6).
- **One shared surface that dissolves it:** not one op — **decomposition** once derived ops land (catalog lookups, projections, TotalMap) so orchestrators shrink to thin composition.

---

## Missing-substrate map

| Missing shared surface | Towers hand-rolling around it (in this slice) | Dissolve-on / owner |
|------------------------|-----------------------------------------------|---------------------|
| **Coproduct Projection / variant reflection (L1.1)** | TestClaim→label/variant (`verification`); Connective/Behavior mirrors (`node`); generated-kind partition (`artifact`); reason→Symbol (`report`); repr_kind atom decode (`target_model`) | T-19 testgen / coproduct-reflection; node://adhoc-2145db6b-69a |
| **`content_hash` + `hash_combine` export** | byte_offset ladder + merkle fold (`node` authority); row-key scans (`target_model`); fn-ref pins (`runtime`) | B1-CANON; T-22 IRT-4 |
| **Outcome-bearing NodeFold (Q4)** | QnFoldStatus (`qualified_name`); 3× graph folds (`dependency`); grammar_symbol_fold (`grammar`); catalog folds (`target_model`); edge filters (`node_query`) | Ratified Q4 in `diagnostic.dag` |
| **TotalMap / TotalPolicy ops** | bool/law Maps as fn+match (`model_core`); all target_model catalogs as List+fold; fixture registry (`leaf_model_verification`); 2-arm tables (`report`, `artifact`) | `collection.dag` L156–160 — **types only, no API**; L1.13.c table_decision_tree lens |
| **CatalogLookup derived op** | 6× missing/unique/ambiguous in `target_model`; readiness layers in `dependency` | SG catalog modeling (single authority P2) |
| **Grammar catamorphism / serializer** | 5+ fold_list machines; production selection (`grammar`) | DESIGN-2 parser-derived production selection |
| **FreeMonoid&lt;Symbol&gt; alias + head/tail projections** | QnEmpty/QnCons parallel type (`qualified_name`); CharAbsent (`text`); list_at carriers (`collection`) | free-monoid-entry-generic-inference bootstrap fix |
| **Node child projections (positional/labeled/named)** | manual edge-label filters (`node_query`); SG-2 positional reads (`target_model`, `grammar`) | node://adhoc-sg2-* ; lives in node_query because node.dag is import-free |

---

## Clean / low-concern files (🟢)

`witness`, `verdict`, `platform`, `network`, `pipeline`, `host_run`, `target_triple`, `module_batch`, `determinism`, `machine`, `patch` (FieldPatch terminal), `exact_structural_equality_zip_fold_predicate`, `constraint_satisfaction_predicate`, `test_claim_falsification` — principled carriers or canonical fold examples with no recurring unmodeled repetition.

**Staged mirrors (delete-on-arrival, not towers):** `types.dag`, `patterns.dag`.
