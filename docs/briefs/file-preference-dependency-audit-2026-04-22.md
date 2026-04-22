# File-preference scaffold — implicit-dependency audit

**Date:** 2026-04-22
**Lane:** A (audit only, no code changes)
**Scaffold under audit:** `Dag::declaration_name_preference_rank`
(`src/v3/compiler/src/dag.rs:2012-2020`) and its mirror
`collect_symbols` (`src/v3/compiler/src/lower.rs:1247-1286`, mirrored
in `lower_generated.rs`). Rank: `src/v3/` → 2, unknown → 1, `dsl/` → 0.

## Scaffold semantics (recap)

Ratified-parallel-authority bridge. When two top-level declarations
share a name, the `src/v3/`-rooted one wins. The scaffold is held
only because three concrete module pairs remain duplicated; every
other `declaration_by_name` consumer sees a single authority.

Duplicated authorities today (from ROADMAP "Post-merge debt" and
file inspection):

| module | dsl authority | v3 authority |
|---|---|---|
| `std.effects` | `dsl/std/effects.dag` | `src/v3/std/effects.dag` |
| `std.verification` | `dsl/std/verification.dag` | `src/v3/std/verification.dag` |
| `http_path` mirror | `dsl/std/http_path.dag` | embedded in `src/v3/std/effects.dag:138-260` |

The **overlap set** — names declared in both a `dsl/` authority and
a `src/v3/` authority — is the only set where rank changes behavior:

```
# std.effects (dsl/std/effects.dag ↔ src/v3/std/effects.dag)
check_modifier_vs_derivation, compose_effects, derive_effect_shape,
derive_op_effect, DeriveOpEffectResult, EffectShape,
generate_idempotency_obligations, IdempotencyEvidence,
IdempotencyTestObligation, is_idempotent_effect, KeySource,
ModifierAgreement, ModifierCheck, OperationEffect,
parse_http_method, WorkflowEffectConcern

# std.verification
TestClaim

# http_path mirror (dsl/std/http_path.dag ↔ embedded in src/v3/std/effects.dag)
last_path_param, parse_path_template, parse_segment_tokens,
PathSegmentTokensResult, PathTemplate, PathTemplateParseResult,
UrlPathToken
```

**Audit correction (v2, post-review).** The initial overlap set was
derived from a truncated `head -40` of each authority's declaration
list and undercounted `std.effects` entries. Full `comm -12` against
the complete declaration lists yields 24 overlap names, not the 18
originally listed. The expanded set drives the expanded (b) count
below.

Any `declaration_by_name("X")` whose `X` is outside this set is
rank-insensitive: it returns the sole existing declaration (or
None), regardless of scaffold state.

## Method

Enumerated every `declaration_by_name(…)` and every lowering path
that consumes `collect_symbols`'s symbol table. Static string
arguments were extracted and cross-checked against the overlap set.
Dynamic arguments were classified by semantic role.

Totals: **210 call sites total** (210 grep matches including doc
references). Actionable call sites (non-comment): **113 in
`src/v3/compiler/src/`**, **67 in `src/v3/compiler/tests/`**, plus
the two bulk consumers (`collect_symbols` + the stub-resolution
sweep at `lower.rs:2134` / `lower_generated.rs:2143` / `infer.rs:1619`).

## Classification

### (a) Incidental — rank-insensitive, safe

**Every static-name call site in `src/v3/compiler/src/` (113 sites,
55 distinct names)** looks up a singleton substrate/spec/stdlib
authority that does not appear in any `dsl/` duplicate. Rank cannot
affect which declaration is returned.

Unique names (all single-authority):
`BehaviorRealization, Bind, Bool, BooleanAlgebra, Branch,
CallableRealization, CallableStrategy, CleanEmissionContract, Dag,
dag_model, DeclarationId, DeclarationRef, fold, go_execution_model,
go_execution_requirement, go_language, go_source_filtering,
head_or_zero, id, Int, LanguageSpec, List, Loop, Main, MyInt,
NodeId, OperatorRealization, OrderedRing, parse, PatternBindingRule,
PatternRealization, PipelineSnapshotKind, PortId,
python_execution_requirement, python_language,
python_source_filtering, python_target, rust_clean_emission_binding,
rust_execution_model, rust_execution_requirement, rust_functions,
rust_language, rust_rendering, rust_source_filtering, Sign, String,
SubstrateAccessorBinding, TargetCleanEmissionBinding, Transform,
TypeInstantiationRealization, TypeRealization, use_callback, Value,
VariantPayloadFieldAccessRule, VerifierOutputPolicy`.

**Tests, rank-insensitive subset** — 49 of 67 test sites. Unique
names outside the overlap set include `answer, BinaryOpRow,
BracketRow, cfg, claim_obligation_resources, Classical,
CompilerHostRealization, DegreeAtLeastTwo, Dimension, div, f, first,
get_host, id, Int, KeywordTokenKind, LensRegistryEntry, List,
materialize_test_obligations, MyList, node, OrderedRing, pair, port,
PostEmitVerifier, PunctTokenKind, resolve_producer, ResourceHandle,
rust_clean_emission, rust_language, Secret, SoftKeywordIdentRow,
SubstrateAccessorBinding, TestObligation, TokenKind,
TopLevelItemKwRow, WorkflowEffect`. (`WorkflowEffect` and
`TestObligation` exist only under `src/v3/`, so they are also
rank-insensitive — but see dissolution note for **(c)** below.)

**Dissolution impact:** zero. Drop the rank: same behavior.

### (b) Silent dependency — load-bearing on rank

**18 call sites** (direct + helper-mediated) look up a name in the
overlap set — 5 `TestClaim` lookups + 13 `std.effects`/`http_path`
lookups. The initial draft counted only the two direct-call sites
and missed helper-mediated uses; the corrected tally follows. All
18 live in `src/v3/compiler/tests/integration/`.

1. `src/v3/compiler/tests/integration/m1_5_testgen_test.rs:208`
   ```rust
   dag.declaration_by_name("TestClaim").map(|decl| decl.id),
   ```
   Compares a generated-claim decl's `meta_tag` to whatever
   `TestClaim` resolves to. The assertion is satisfied when
   **`meta_tag`** (set during testgen lowering) and this lookup
   agree — both are today driven by the rank preference selecting
   `src/v3/std/verification.dag`'s `TestClaim`. If the dsl-authored
   `TestClaim` won instead (different conjunctive shape, no
   `requires` field), `meta_tag` assignment upstream and the
   assertion downstream could still agree if both observe the same
   rule — but the test's *intent* is the v3 surface. **Silent.**

2. `src/v3/compiler/tests/integration/lane2_stage_2c_db15_test.rs:9`
   ```rust
   let decl = dag.declaration_by_name("TestClaim")
       .expect("TestClaim from std.verification");
   let TypeConnective::Conj { children } = &decl.connective else { ... };
   assert!(labels.contains(&"requires"), "{labels:?}");
   ```
   Only the v3 `TestClaim` has a `requires` field. The assertion
   passes *only because* rank routes this lookup to the v3
   authority. Under neutral lookup with both authorities present,
   the test would be nondeterministic or fail. **Silent — this is
   the archetype (b) case.**

3. **`m1_5_verification_test.rs:76, 203, 207`** — helper-mediated via
   `record_fields(&dag, "TestClaim")` (line 76) and
   `find_named(&dag, "TestClaim")` (lines 203, 207). `find_named`
   is a local helper that wraps `dag.declaration_by_name(name)` at
   `m1_5_verification_test.rs:13-14`; `record_fields` also reaches
   `declaration_by_name` through its `find_named` call. All three
   assertions expect the v3 `TestClaim` shape (with `requires`) and
   would be silently routed by rank. **Helper-mediated silent —
   archetype (b).**

4. **`lane2_stage_2a_effects_smoke.rs:62-100`** — 13 helper-mediated
   lookups via `arrow_body(dag, name)` (line 15, calls
   `declaration_by_name`) and `assert_record_type(dag, name)` (line
   25, same). Names exercised: `is_idempotent_effect, compose_effects,
   derive_effect_shape, check_modifier_vs_derivation,
   generate_idempotency_obligations, parse_path_template,
   last_path_param, EffectShape, KeySource, IdempotencyEvidence,
   OperationEffect, ModifierAgreement, ModifierCheck`. All 13 are
   overlap-set names; the test's assertions (arrow-body presence,
   record-type shape) depend on the v3 authority's extended surface
   — the dsl authorities have narrower shapes (e.g., v2's
   `EffectShape` lacks `IsIdempotent`/`IsBreaking` variants of the
   v3 form; `compose_effects` arrow body in dsl returns
   `ComposedEffect`, in v3 a richer `CompositionVerdict`-bearing
   type). **Helper-mediated silent — the largest cluster.**

**Dissolution path.** These sites are not a substrate problem;
they are symptoms of the two convergence blockers already tracked
in ROADMAP:
- **`std.verification` convergence** — governs the 5 `TestClaim`
  sites (items 1, 2, 3 above).
- **`std.effects` convergence** (plus embedded `http_path` mirror)
  — governs the 13 `lane2_stage_2a_effects_smoke.rs` sites (item
  4). Tracked in the same ROADMAP "Post-merge debt" row as the
  file-preference scaffold.

Per convergence, tests either become (a) incidental (v3 authority
survives), or migrate to the surviving shape (dsl wins / merged
surface emerges). No new substrate work needed — the convergences
*are* the dissolution. **The "no new modeling gap" conclusion from
the draft still holds — but the scale (18 sites, not 2)
materially changes the cost-of-dissolution estimate for the
`std.effects` convergence in particular, which the ROADMAP row
should reflect when that lane is scoped.**

### (c) Legitimate-looking — scaffold's intended consumers

Three classes of consumer look up names programmatically (not from
a static whitelist) and depend on rank at the systemic level rather
than at a single call site:

1. **`collect_symbols`** (`lower.rs:1269-1286`, mirrored in
   `lower_generated.rs:1259-1286`). Seeds the per-Dag symbol table
   used to resolve every identifier in every lowered `.dag` source
   file. When a surface identifier matches a name in the overlap
   set, the v3 authority wins. This is the scaffold's *reason for
   existing*: it lets v3-authored `.dag` code reference the v3
   surfaces of `EffectShape`, `TestClaim`, `PathTemplate`, etc.
   even while legacy `dsl/` duplicates remain ingested.

2. **Stub-resolution sweep** — `lower.rs:2134`,
   `lower_generated.rs:2143`:
   ```rust
   if let Some(target) = dag.declaration_by_name(&name).map(|d| d.id)
   ```
   Repairs `UnresolvedIdentifier` stubs after bodies are lowered.
   Same rank-consumer pattern as `collect_symbols`.

3. **`infer.rs:1619`** (`dag.declaration_by_name(name)`) and
   **`regen_tokenize.rs:159`** — general name-lookup helpers used
   by inference and codegen regen paths. Same pattern.

These are the call sites the scaffold exists to serve; they are
*not* (b) because they are not load-bearing on rank for their own
correctness — they are load-bearing on rank for the correctness of
any v3 `.dag` source that references an overlap name. Dropping
rank without converging authorities would make **every** v3-side
reference to `EffectShape` / `TestClaim` / `PathTemplate` (and the
sibling names) ambiguous at the same moment.

**Subtle systemic risk — dsl-resident references rebind.** `collect_symbols`
is global, not per-file. Internal references inside
`dsl/std/effects.dag` (e.g., `shape: EffectShape` at line 144, and
all sibling intra-dsl references enumerated by
`grep -rnE '\b(EffectShape|OperationEffect|IdempotencyEvidence|
TestClaim|PathTemplate)\b' dsl/std/`) are silently rebound to v3
authorities when v3 lowers them. The dsl-authored bodies run
against v3 type shapes. No call site can flag this; it is a
property of the bridge. Classification: (c) — this is what the
scaffold *means*, not an accidental dependency — but worth
documenting explicitly because reasoning about `dsl/std/effects.dag`
semantics inside v3 compilation requires knowing this rebind
happens.

**Dissolution path for (c).** Same as ROADMAP: converge the three
duplicated authorities. Once converged, these consumers revert to
vanilla single-authority lookup; the rank function and its mirror
delete together.

## Summary table

| Category | Count | Sites |
|---|---:|---|
| (a) Incidental | 162 | all 113 `src/` static-name sites + 49 test sites outside the overlap set |
| (b) Silent dependency | 18 | 5 TestClaim sites (`m1_5_testgen_test.rs:208`, `lane2_stage_2c_db15_test.rs:9`, `m1_5_verification_test.rs:{76,203,207}`); 13 in `lane2_stage_2a_effects_smoke.rs:{62,63,64,65,66,76×2,90,94,95,97,98,100}` |
| (c) Legitimate-looking | 3 classes | `collect_symbols`, stub-resolution sweep, dynamic name helpers (`infer.rs:1619`, `regen_tokenize.rs:159`) |

PR-body framing: **162 safe, 18 silent, 3 class-level legitimate.**
(Site totals: 113 static `src/` + 67 tests = 180 actionable call
sites; 180 − 18 (b) = 162 (a). The three (c) classes are consumer
patterns, not individual sites, and are listed separately.)

## Recommendations

1. **No lane needed to repair the (b) sites independently.** All 18
   sites dissolve automatically when their governing convergence
   lands (5 with `std.verification`, 13 with `std.effects` /
   http_path mirror). The existing ROADMAP rows already own the
   work.

2. **Consider a tightening helper** *after* convergence: a
   fail-closed `declaration_by_name_unique` that returns the single
   declaration or emits a diagnostic on multiple matches, used as
   the post-scaffold replacement. Not needed now; mentioned here so
   the dissolution PR has a named target.

3. **No modeling gap discovered.** The audit confirms the
   scaffold's dissolution blocker is the three known duplicated
   modules, nothing more. But the reflective-analysis concern
   that "new lanes may silently depend on rank preference" *is*
   realized — the lane2 Stage 2a effects-smoke lane added 13
   silent-dependency sites (via `arrow_body` / `assert_record_type`
   helpers) after the scaffold went in. Every site is still
   governed by an already-tracked convergence row, but the cost
   of the `std.effects` convergence when scoped is meaningfully
   higher than the ROADMAP row suggests.

4. **Watchpoint for future lanes.** Adding any new `declaration_by_name("X")`
   where `X` is in the overlap set above reintroduces a (b)
   dependency. A lightweight CI check (grep the overlap-name list
   against `declaration_by_name` call sites outside a known
   allowlist) would catch this at review time without modeling
   work. This is mentioned as a watchpoint; not proposed as a
   deliverable of this audit per the "no ratchets for textual
   enforcement" feedback principle.

## Non-goals respected

- No change to `dag.rs:2012-2020` or `lower.rs:1247-1286`.
- No change to any consumer.
- No attempt to resolve `std.verification` convergence.
