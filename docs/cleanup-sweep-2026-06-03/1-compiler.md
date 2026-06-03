# Cleanup sweep #1: `src/v4/compiler/` — concerning-patterns catalog

**Session:** wise-ant-755 · **Date:** 2026-06-03 · **Mode:** read-only audit (hard freeze; no code changes).

**Frozen artifact (ledger standing):** This file is a **one-shot session snapshot** for PM
aggregation during the 2026-06-03 cleanup sweep. It summarizes patterns visible in the tree at
commit time; it is **not** a parallel ledger to keep in sync with inline `.dag` marks
(`dissolve-on`, `feature:`, scariness tags). **Do not update this doc** when marks change —
authoritative receipts stay in the substrate; open a new dated sweep doc if a fresh census is
needed.

**Scope:** 14 `.dag` modules under `src/v4/compiler/` (~11.2k lines). Pipeline stages:
`00_compile`, `01_tokenize`, `02_parse`, `03_{normalize,resolve,name_resolve}`, `04_infer`,
`05_{eval,emit}`, `06_translate`, `07_target_carriers`, plus adjunct `emit_host`, `self_host`,
`source_authority`.

**Vocabulary (from THESIS / INVARIANTS / MODELING):**

- **Bridges** — scaffold, interim, host-transport, shim, fallback, yellow-gated `dissolve-on`
  markers, Rust-side execution that bypasses the modeled path.
- **Hand-rolled towers** — private `combine_hash` / digest ladders, target-syntax-by-match,
  closed-vocab tables-as-functions, same-payload coproduct variants, concept-sink imports.

---

## Slice: `src/v4/compiler/` — overall scariness: 🟡

Load-bearing stages run real structural walks (tokenize → parse → infer → translate), but two
**RED** towers (`05_eval`, `06_translate`) and a **RED** host bridge (`emit_host`) concentrate
unmodeled repetitive work. Yellow-gated dissolve-on markers are pervasive (good receipts). **Zero
v3 imports** in this directory.

---

## Per-file (worst-first)

| Path | | Bridges | Towers | Note |
|------|---|---------|--------|------|
| `06_translate.dag` | 🔴 | SG-2 `ProjectionAbsent` / MVP-1 dual-path shim; T-11 grammar-relation row interim; `B-LOOKUP-1`; `authority_source_text` anchor; `grammar_inverse_not_realized` fail-closed | ~4.3k-line target-syntax tower: `translate_node`/`coerce`, grammar-inverse serialize, `trait_name==*` dispatch, type-expression projection | Load-bearing lower/emit path; largest file |
| `05_eval.dag` | 🔴 | RULING-1 `TestClaimTypedInput`/`EvalSubject` bridge; T22-EVAL-CACHE-HASHES gates; Wave-0 `TestClaimRun` | `combine_hash` + `test_claim_symbol_digest` ladder (~221 hash sites): interpretation slots, Diagnostic/Correction/Locus/Extent/TestClaim coproduct digests | Load-bearing eval; explicit dissolve → `content_hash(Node)` |
| `emit_host.dag` | 🔴 | `emit_host_bridge.rs` + `emit_host_eval.rs` intercept; fail-closed `run_emit_host_*`; `authority_source_text` pin match (3 MVP strings); python receipt asymmetry | 5-byte stdout parse; pin-routed target dispatch | Executable proof in Rust until T-PB-B/T-22 |
| `self_host.dag` | 🔴 | Full scaffold; `SelfHostRunnerNotRealized`; witness bodies return `Violates` | none | Contract shape only; not a pipeline stage |
| `02_parse.dag` | 🟡 | 8+ Practice-10 morphism gates (`adhoc-2145db6b-69a`); interim carriers (`ParseExprResult`, etc.); T-8 terminal staging | Hand-written `parse_expr` over `GrammarExpr` + `ParseTable` memo | Load-bearing parse |
| `01_tokenize.dag` | 🟡 | `LexMatchResult`/`LexRuleApply` interim; `char_in_class` + `lex_match_pattern` predicate gates | Hand-written `lex_match_pattern` match tower (~200 lines) | T-6 lex walk realized |
| `04_infer.dag` | 🟡 | `AlgebraRef` IR-1 bridge; `scaffold:T-9-ground`; `infer_algebra_ref_ungrounded` guard | Find-phase + bounded-lattice walks (structural) | Load-bearing infer |
| `03_name_resolve.dag` | 🟡 | T-28-B admission-state fold carrier; 4× `fold_list` bootstrap-limitation; `B-LOOKUP-1` | none | Cross-file admission modeled |
| `03_normalize.dag` | 🟡 | CP-1b surface-sugar gate (Atom identity vs LanguageModel) | `SurfaceSugarKind` + `classify_sugar` identity dispatch | Small |
| `00_compile.dag` | 🟡 | T-23 `CompileLens` stub; `compile_ingest_staging`; `compile_eval_not_realized` | none | Orchestrator |
| `source_authority.dag` | 🟡 | `tree==` witnesses until substrate equality + canonical `.dag` TargetModel row | Law composition via structural `==` | H.7.2 contract |
| `03_resolve.dag` | 🟢 | `B-LOOKUP-1` only | none | K-1; clean `Scope` coproduct |
| `05_emit.dag` | 🟢 | none | none | 42-line `translate` ∘ serialize delegate |
| `07_target_carriers.dag` | 🟢 | none (alias hub) | none | 18 lines |

---

## Concerning patterns (unmodeled repetitive work)

These are the patterns that **recur across files** and should become **one modeled surface**.
Ordered by dissolution leverage.

### P1 — Private digest-hash ladders (`combine_hash` + symbol-tag arms)

| Where | What is hand-rolled | One shared surface that dissolves it |
|-------|---------------------|--------------------------------------|
| `05_eval.dag` (~221 sites) | Per-coproduct-arm `*_digest` fns: `no_correction_reason_digest`, `correction_verdict_digest`, `locus_cache_digest`, `extent_cache_digest`, `test_claim_*_digest`, `interpretation_*_executed_digest`, `interpretation_structure_witness_*_digest` | **`content_hash(Node)`** on projected carriers (Diagnostic, Correction, Locus, Extent, TestClaim eval subject, InterpretationAlgebra fn bodies). Tags already name T22-EVAL-CACHE-HASHES / IRT-4. |

**Recurrence:** essentially single-file today, but the *shape* (match coproduct → tag digest → `combine_hash`) is the same anti-pattern INVARIANTS Practice 10 calls a derived operation.

---

### P2 — Target-syntax / grammar-inverse by structural match

| Where | What is hand-rolled | One shared surface that dissolves it |
|-------|---------------------|--------------------------------------|
| `06_translate.dag` (dominant) | `translate_node` / `translate_node_mvp1` dual path; `grammar_relation_row_*` lookup; grammar-inverse serialize walk; `trait_name == target_collection_trait_*` nested dispatch; type-expression projection match arms | **Grammar-inverse morphism** over declared `GrammarRelation` rows (T-11 typed row items); **TargetTypeExpression projection** on every `TargetModel` bundle (SG-2 — delete `ProjectionAbsent` shim); **catamorphism / fold over TargetModel edges** instead of atom-identity tables |
| `05_emit.dag` | Delegates only — tower lives in translate | (same as above) |
| `source_authority.dag` | Round-trip via `target_serialize_source_from_model` + `tree==` | Canonical **`.dag` TargetModel row** + structural equality witness substrate |

**Recurrence:** translate + source_authority + emit pipeline; MVP-1 TestClaims exercise the hand path for five targets.

---

### P3 — Coproduct structural morphisms (parse / lex walks)

| Where | What is hand-rolled | One shared surface that dissolves it |
|-------|---------------------|--------------------------------------|
| `02_parse.dag` | `parse_expr` match over `GrammarExpr`; nullable / left-corner / left-recursion / duplicate-name / undefined-ref as separate predicate fns | **Substrate-derived morphism over `GrammarExpr` / `GrammarRoot`** (single fold; queries as derived ops) — gates bind `node://adhoc-2145db6b-69a` |
| `01_tokenize.dag` | `lex_match_pattern` match over `LexPattern`; `char_in_class` per-arm nat ranges | **Substrate-derived morphism over `LexPattern`**; **variant-discriminant char-class projection** (same gate family) |

**Recurrence:** tokenize + parse share the “big `match` tower on grammar carrier” shape; both tagged for dissolution to one morphism substrate.

---

### P4 — Bootstrap generic / lookup workarounds

| Where | What is hand-rolled | One shared surface that dissolves it |
|-------|---------------------|--------------------------------------|
| `03_name_resolve.dag` (4×) | Hand-rolled `Empty`/`Cons` accumulation over `FreeMonoid<T>` via `fold_list` | **Generic `fold` / algebra over `FreeMonoid<T>`** when v2 bootstrap infers user-defined `T` |
| `03_resolve.dag`, `06_translate.dag` | `map_get` → `Option` / `Outcome` instead of `Map.lookup` with `Witness<V>` | **`Witness<V>` generic dispatch for `Map.lookup`** (`B-LOOKUP-1`; substrate at `v4.std.collection`) |

**Recurrence:** resolve, name_resolve, translate — same bootstrap limitation, same feature id.

---

### P5 — Dual-path shims (substrate row not landed)

| Where | What is hand-rolled | One shared surface that dissolves it |
|-------|---------------------|--------------------------------------|
| `06_translate.dag` | `TypeExprProjectionPresence`: `ProjectionPresent` vs `ProjectionAbsent` → `translate_node_with_projection` vs `translate_node_mvp1` | **`target_model_edge_type_expression_projection` on every extdeps `TargetModel`** (SG-2 worksheet) |
| `emit_host.dag` + Rust | `.dag` `run_emit_host_*` fail-closed; real run in `emit_host_bridge.rs` / `emit_host_eval.rs` | **`runtime_row` on TargetModel** + modeled `v4.std.host_run` dispatch (T-22 / T-PB-B) |
| `00_compile.dag` | `compile_eval_not_realized`; T-23 local `CompileLens` | **`v4.lens.application`** canonical lens adapter; eval mode substrate |
| `03_normalize.dag` | `classify_sugar` compares Atom ids to `dag_c3_surface_sugar_*` | **LanguageModel surface-sugar facts** in extdeps (CP-1b / T-8) |

**Recurrence:** pattern is “interim arm + permanent path”; yellow gates name the owning feature.

---

### P6 — Host-transport and authority pins (bypass modeled path)

| Where | What is hand-rolled | One shared surface that dissolves it |
|-------|---------------------|--------------------------------------|
| `emit_host.dag` | String match on `authority_source_text` (rust/python/go MVP pins); 5-byte stdout parse; python receipt hand-reify in Rust | **Structural `runtime_row` + host_run receipt assembly in substrate**; **value parse projection** from TargetModel |
| Rust (out of slice) | `emit_host_bridge.rs`, `emit_host_eval.rs`, `tools/emit_host_runner` | Same — not in `src/v4/compiler/` but is the live bridge for W3 harness |

**Recurrence:** only `emit_host.dag` in-tree; recommend separate sweep slice for Rust bridge crates if PM wants full census.

---

### P7 — IR / grounding bridges (smaller but load-bearing)

| Where | What is hand-rolled | One shared surface that dissolves it |
|-------|---------------------|--------------------------------------|
| `04_infer.dag` | `AlgebraRef` bridge; infer-side admission for `CanonicalGrounding` (`scaffold:T-9-ground`) | **Grounded algebra `Node` constructors** (T-2/T-4); proof-by-construction `CanonicalGrounding` in `std/constraints` |
| `05_eval.dag` | `TestClaimTypedInput` / `TestClaimEvalSubject` (RULING-1) | **TestClaim input projection** as substrate fact (typed eval subject) |

---

## Recurring patterns (top 5 in slice)

1. **Yellow-gated dissolve-on** — Practice 4/9/10 markers on nearly every interim carrier (`feature:` + `dissolve-on-arrival` + `forbidden:`). Good discipline; high count is expected during migration.
2. **Bootstrap `B-LOOKUP-1` + `fold_list` hacks** — `map_get(Option)` and hand `FreeMonoid` folds in resolve / name_resolve / translate.
3. **Dual-path shims** — translate MVP-1 vs projection; emit_host `.dag` vs Rust; compile eval fail-closed.
4. **Digest-hash ladders** — concentrated in `05_eval` (P1); all tagged → `content_hash(Node)`.
5. **Grammar/target match towers** — `01_tokenize` + `02_parse` + `06_translate` (P2 + P3).

---

## Missing-substrate map (rollup)

| Hand-rolled tower / bridge | Shared derived-op surface |
|----------------------------|---------------------------|
| Eval cache / interpretation digests | `content_hash(Node)`; registry-as-data for interpretation slots |
| Translate / serialize / type expressions | Grammar-inverse morphism; TargetTypeExpression projection; TargetModel edge catamorphism |
| Lex + parse walks | `LexPattern` / `GrammarExpr` structural morphisms (one fold each) |
| Sugar classification | LanguageModel facts (not Atom identity compare) |
| Map / monoid accumulation | `Witness<V>` lookup; generic `FreeMonoid` fold |
| Host emit-vs-eval | `host_run` + `runtime_row` on TargetModel; modeled stdout parse |
| Source round-trip laws | Structural equality witness on ParseTree / ResolvedTree; canonical `.dag` TargetModel row |
| Infer grounding | Grounded algebra constructors; constraints proof-by-construction |

---

## Slice size note

~11.2k lines / 14 files — **expected** for compiler spine. **06_translate** (4.3k) + **05_eval** (2.0k) dominate.
No misplaced files; `emit_host`, `self_host`, `source_authority` are adjunct modules, not extra numbered stages.

**Out-of-slice recommendation:** Rust bridge crates (`src/v3/compiler/emit_host_bridge.rs`, `emit_host_eval.rs`, `tools/emit_host_runner`) if the program wants a full host-transport census — not under `src/v4/compiler/`.
