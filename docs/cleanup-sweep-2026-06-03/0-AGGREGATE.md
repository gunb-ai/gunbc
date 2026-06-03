# v4 Cleanup-Sweep — Aggregate & Dissolution Plan (2026-06-03)

**Status:** read-only diagnostic, authored during the project freeze. Aggregates the six slice catalogs into one ranked picture, maps **every** concerning case onto a concrete **solution/deletion path**, and works out the dissolution **ordering** — i.e. what deleting one tower *uncovers* next.

## Source catalogs (child PRs — full per-file detail)

| # | Slice | PR | Doc | Overall | Headline |
|---|---|---|---|---|---|
| 1 | `src/v4/compiler/` (14 stages) | #4379 | `1-compiler.md` | 🟡 | 2 RED towers: `06_translate` (4.3k), `05_eval` (221 hash sites) |
| 2 | `src/v4/std/` (50) | #4380 | `2-std.md` | 🔴 | megatowers `target_model` (3442L), `node` (1298L); TotalMap declared-unbuilt |
| 3 | `src/v4/extdeps/` (45) | #4383 | `3-extdeps.md` | 🟡 | CP-1b dual-carrier across 11 languages; host pins leaked from compiler |
| 4 | `src/v4/lens/`+`workflow/`+ci | #4381 | `4-lens.md` | 🔴 | `workflow/ci.dag` 6.3k monolith, TRIPLE authority |
| 5 | `src/v4/test/` (253) | #4384 | `5-test.md` | 🟡 | volume not deception; `content_hash` used correctly here |
| 6 | v3 tests (262) | #4382 | `6-v3-tests.md` | 🟡 | v4 parse-surface smokes; one **live** bridge (`emit_host_bridge.rs`) |

## Root cause (one sentence)

Every concerning case is a file **hand-rolling a derived operation locally** because the shared substrate surface it needs is **missing, declared-but-unbuilt, or trapped in an import-isolated module** — and reviews validated each tower *locally*, so the tower became precedent. Confirmed across all six slices.

## The 6 missing substrate surfaces (the keystones)

Each: state · who hand-rolls it · solution · deletion path · **cascade (what landing/deleting it uncovers)**.

### S1 — `content_hash` / `Projection` / `combine_hash`, *exportable*
- **State:** the canonical `content_hash`/merkle fold + `combine_hash` live inside `std/node.dag`, which is **import-free by design** → **not consumable cross-module**. `std/projection.dag` models Projection *records for lenses* only — not derive-arm/field projection.
- **Hand-rolled by:** `05_eval` (221-site digest ladder: interpretation/Diagnostic/Locus/Extent/TestClaim digests) · `std/node.dag` itself (600L `byte_offset` cache/digest tower) · `std/target_model` (content_hash-as-key scans) · v3 `grounding_tests/stratum_a.rs` (`RowFingerprint`/`list_digest`) · v3 `grounding_engine` (pilot multiset mirror).
- **Solution:** a shared `node_hash` / projection module that **exports** `content_hash(Node)` + `combine_hash` + a generic `Projection<T>` (derive arm/field), consumable everywhere.
- **Deletion path:** each private digest ladder → `projection_hash(value, projection)`.
- **Cascade:** node.dag's import-freeness means the export must be a *new* module (or a cycle-free re-export). Landing it makes 05_eval's ladder, target_model's keys, and the v3 fingerprints **all simultaneously eligible** — delete consumers one-PR-each. node.dag's own 600L `byte_offset` tower is the same shape; dissolve in the same wave.

### S2 — TotalMap / registry-as-data ops
- **State:** `std/collection.dag` (L156–160) **declares `TotalMap<K,V>` + `TotalPolicy` but ships ZERO constructors / lookup / insert.**
- **Hand-rolled by:** `std/model_core` (bool/law axes as fn+match Maps) · `std/target_model` (**6× catalog-lookup folds** — atom/signature/collection/ownership) · `std/report` (reason→Symbol) · `std/artifact` (kind→Bool) · per-language fact bundles across `extdeps/languages/*` · `extdeps/coercion_widening` (single-pair table) · v3 `sg0_census` expected-set.
- **Solution:** build the `TotalMap` ops + a `Registry<K,Row>` derived lookup that returns the **missing/unique/ambiguous** trichotomy as ONE op (the shape `target_model` clones 6×).
- **Deletion path:** each closed-vocab `if sym == …` ladder + each `List<Row> + fold` catalog → `TotalMap` rows + `registry_lookup`.
- **Cascade:** dissolving target_model's 6 catalog folds shrinks it dramatically from 3442L — **but** the SG-1 string/`FreeMonoid` raw-node bridge *beneath* them needs **S3 (fold)** first. So target_model is a **two-layer** dissolve: lookup (S2) then fold (S3). Also uncovers `collection.dag`'s own `map_insert` closure tower + `list_nth/at_optional` accumulators — same gap, same wave.

### S3 — Generic `Outcome`-bearing fold + coproduct-variant reflection
- **State:** `algebra.dag` has `fold_list`, `node.dag` has `fold_node`, but there is **no generic Outcome-carrying NodeFold (Ratified Q4, deferred)** and **no coproduct-variant projection/reflection (L1.1)**.
- **Hand-rolled by:** `std/qualified_name` (`QnFoldStatus` — whole file dissolves with the FreeMonoid alias) · `std/dependency` (3× bespoke NodeFold) · `std/grammar` (5 fold state machines) · `std/target_model` catalog folds · `std/verification` (TestClaim→label/variant projections) · `06_translate` child-reattachment zip-fold · `extdeps` catalog construction folds.
- **Solution:** a generic `Outcome`-bearing fold/algebra + coproduct-variant projection (reflect a coproduct's arms/fields as data).
- **Deletion path:** replace bespoke folds with the generic algebra; replace hand-enumerated variant→X tables with variant-projection.
- **Cascade:** unblocks S2 (catalog folds become registry lookups once the fold is generic) **and** S4 (a grammar morphism *is* a fold). Uncovers the **bootstrap FreeMonoid alias failure** — a v2 generic-inference limit that `qualified_name`, `name_resolve`'s `fold_list` hacks, and `map_get` Witness→Optional all work around. That is a **deeper bootstrap-compiler fix**, not just a substrate addition — flag it as a separate prerequisite (see §Cascade-2 below).

### S4 — Grammar morphism + `TargetSurfaceNode`/serializer (+ CP-1b convergence)
- **State:** `06_translate` builds target syntax by **local structure-match logic** (4.3k-line tower: grammar-inverse walk, `trait_name == *` dispatch, type-expression projection). `extdeps` carries the **CP-1b dual-carrier** — every `languages/*.dag` keeps `FormalProduction` **and** a parallel operational `GrammarExpr`/`LexRules` (**duplicated grammar authority across 11 languages**). `01_tokenize`/`02_parse` hand-roll `lex_match_pattern`/`parse_expr` over the same un-converged grammar.
- **Hand-rolled by:** `06_translate` · `01_tokenize` (~200L lex match) · `02_parse` (`parse_expr` + `ParseTable`) · `std/grammar` (fold state machines) · every `extdeps/languages/*` (the second, operational grammar).
- **Solution:** a `TargetSurfaceNode` carrier + a **grammar-driven serializer** (grammar-as-data); converge CP-1b so `FormalProduction` is the **sole** authority and `GrammarExpr`/`LexRules` are *derived* (or deleted).
- **Deletion path:** translate serializer → `serialize_with_grammar(grammar, surface)`; tokenize/parse → one grammar catamorphism (S3); delete the per-language operational grammar.
- **Cascade:** S4 **depends on S3** (the morphism is a fold). Dissolving translate's tower uncovers the layer *below* = **S5** (emit_host runtime_row): translate stops at the surface, emitting/running is S5. Also uncovers `source_authority`'s dag round-trip law — it dissolves once translate is grammar-driven.

### S5 — Host-transport as a modeled `TargetModel.runtime_row` (**the live bridge**)
- **State:** `emit_host` dispatches via **string `authority_source_text` pins** + the **one genuinely live bridge**: `emit_host_bridge.rs` + `emit_host_eval.rs` (real cargo/python/go spawn) while the substrate row stays `transport_not_wired`. v3 `v4_emit_host_harness_test.rs` exercises it. `extdeps/languages/{rust,python,go}` duplicate the MVP source-text pins.
- **Hand-rolled by:** `compiler/emit_host.dag` (pin-routed dispatch + 5-byte stdout heuristic) · `extdeps/languages/{rust,python,go}` (`*_mvp1_source_text` pins) · v3 `emit_host_harness` + `emit_host_bridge.rs`.
- **Solution:** model host transport as a structural `TargetModel.runtime_row`; route eval through the substrate eval dispatch (T-22 `run_target_verification`).
- **Deletion path:** delete string pins → `runtime_row` lookup; delete `emit_host_bridge.rs` → substrate `run_emit_host_*` via the eval path.
- **Cascade — DEEPEST:** S5 needs the eval substrate to **actually run** — i.e. the Rung-1 `gunbc test` execution handler we were mid-building. The bridge exists *because* claims don't execute at runtime. So **the runtime-execution path is a prerequisite for the deepest bridge deletion**, not optional. Order S5 **last**.

### S6 — CI-as-data single authority (`ci.dag` triple-authority collapse)
- **State:** `src/v4/workflow/ci.dag` is a **~6.3k-line monolith with TRIPLE authority**: the `.dag` substrate + hand-synced `.github/workflows/ci.yml` + the `tools/ci_affected_components` Rust mirror. `lens/testgen` + `lens/coverage` towers feed CI selection.
- **Hand-rolled by:** `ci.dag` (workflow policy + upserts + affected-set + testclaim rosters + receipt persistence + cache hashing + shell-exception ledger) · `lens/testgen` + `lens/coverage` (selection towers).
- **Solution:** ci.dag becomes the **sole** authority (emits `ci.yml`; delete the Rust mirror); extract selection-receipt / upsert / shell-exceptions / testclaim-selection / gh-bridge into their own modules; selection towers consume S2 + S3.
- **Deletion path:** the bankruptcy already chartered (§11.7) — extract concerns, ban private digests + concrete-testclaim imports in `ci-core`, collapse 3 authorities → 1.
- **Cascade:** depends on S2 + S3 (selection/roster towers are registry + fold). Collapsing the triple authority uncovers whether v3 `v4_workflow_ci_runner_dag_smoke_test.rs` (2734L hand-Rust CI ratchet) can dissolve — it is the Rust mirror's test twin.

## Think-ahead — dissolution ordering (deleting in the wrong order just *moves* the tower)

```
S1 content_hash export ─┐
S2 TotalMap/registry  ──┼──► foundation: most towers become deletable once these land
S3 generic Outcome-fold ┘──► dissolves S2's catalog-fold layer; enables S4
                            │
S4 grammar/TargetSurface ───┴──► translate tower, tokenize/parse, CP-1b convergence
                                 │
S6 ci single-authority ──────────┴──► needs S2+S3 (selection/roster)  [mid]
S5 runtime_row (live bridge) ─────────► needs the Rung-1 EXECUTION path  [deepest, LAST]
```

### Second-order issues uncovered (the "deleting reveals more")
1. **node.dag is import-free** → S1's export cannot simply `import content_hash`; it needs a *new* shared module or a cycle-free re-export. *(Uncovered the moment you try to delete 05_eval's ladder.)*
2. **Bootstrap FreeMonoid / generic-inference limit** → several folds (`qualified_name`, `name_resolve` `fold_list` hacks, `map_get` Witness→Optional) work around a v2 inference limit. Dissolving S3 reveals this is a **deeper bootstrap-compiler fix**, a *prerequisite* to several S3 deletions — not a substrate addition. **Track separately.**
3. **S5 needs execution** → the emit_host bridge exists *because* claims don't execute at runtime (the Rung-1 gap). The runtime-execution work is therefore a hard prerequisite for the deepest bridge deletion.
4. **target_model is a two-layer dissolve** → S2 removes the *lookup* clones; the SG-1 string/FreeMonoid bridge underneath needs S3. Don't expect one PR to flatten 3442L.
5. **test/ self-contradictions** → a few test files *forbid the shape they ship* / assert-against-absence; dissolving the shipped tower may break the self-referential test — check those before deleting.

## Proposed cleanup-wave shape (falls directly out of the ordering)

- **WC0 — stop the bleeding:** land the dissolution **lenses fail-closed FIRST** (no new digest ladder / target-syntax-by-concat / closed-vocab policy fn / same-payload variant / concept-sink growth / **no net-new bridge**). Nothing regrows while we dig out.
- **WC1 — foundation:** model **S1** (content_hash export) + **S2** (TotalMap/registry) → dissolve the digest ladders + closed-vocab tables (05_eval, node byte_offset, model_core, report, artifact, target_model lookups, per-language bundles).
- **WC2 — fold + grammar:** model **S3** (generic Outcome-fold + variant projection) + **S4** (TargetSurfaceNode/serializer + CP-1b convergence) → dissolve translate, tokenize/parse, the catalog folds, the dual-carrier.
- **WC3 — execution + transport + ci:** land the **Rung-1 execution path**, then dissolve **S5** (runtime_row, the live bridge) + **S6** (ci triple-authority).
- **Prerequisite (separate track):** the bootstrap **FreeMonoid / generic-inference** fix — blocks several S3 dissolutions.

## NO-BRIDGES rule (going forward)

Adding a bridge = **hard stop + escalate** (C1-class, like a substrate extension). Inventory today: the **one** genuinely live bridge is `emit_host_bridge.rs` (+ its harness); everything else is either a yellow-gated interim *with* a named `dissolve-on` (acceptable, shrinking) or a *tower* (a missing surface, not a bridge). **Net-new bridges target: zero.**

---
*Aggregated by PM nimble-dove-733 from sweeps #1–#6. Per-file detail lives in the six child catalog PRs above; this doc is the synthesis + dissolution plan.*
