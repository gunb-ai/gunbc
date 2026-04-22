# File-preference scaffold — implicit-dependency audit

**Date:** 2026-04-22
**Lane:** A (audit only, no code changes)
**Scaffold under audit:** `Dag::declaration_name_preference_rank`
(`src/v3/compiler/src/dag.rs:2012-2020`) and its three mirrors:
`collect_symbols` (`src/v3/compiler/src/lower.rs:1247-1286`,
mirrored in `lower_generated.rs`); a `shared_symbols` rebuild in
`src/v3/compiler/src/bootstrap.rs:81` (rank fn) + `:165-185`
(application) used by the legacy PB-1-equivalence bootstrap path.
Rank: `src/v3/` → 2, unknown → 1, `dsl/` → 0.

## Scaffold semantics (recap)

Ratified-parallel-authority bridge. When two top-level declarations
share a name, the `src/v3/`-rooted one wins. The scaffold is held
only because four concrete module pairs remain duplicated; every
other `declaration_by_name` consumer sees a single authority.

Duplicated authorities today (from ROADMAP "Post-merge debt" and
file inspection):

| module | dsl authority | v3 authority |
|---|---|---|
| `std.effects` | `dsl/std/effects.dag` | `src/v3/std/effects.dag` |
| `std.verification` | `dsl/std/verification.dag` | `src/v3/std/verification.dag` |
| `http_path` mirror | `dsl/std/http_path.dag` | embedded in `src/v3/std/effects.dag:138-260` |
| language specs | `dsl/std/languages.dag` | `src/v3/spec/{rust,go,python}.dag` |

**Audit correction (v4, post-review #2).** The ROADMAP "Post-merge
debt" row names only the first three module pairs as blockers, but
`dsl/std/languages.dag` (1419 lines) overlaps with the three v3
spec files on 28 names (`comm -12`): `{go,python,rust}_language`
and the per-language fan-out `{rust,go}_{collection_ops,
control_flow,expressions,literals,modules,patterns,statements,
type_defs}`, `{python,rust}_functions`, etc. Four of those names
are consumed through `declaration_by_name`, making this a fourth
duplicated-authority group the scaffold silently resolves. The
ROADMAP row should be extended to include it; called out in
Recommendations.

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

# language specs (dsl/std/languages.dag ↔ src/v3/spec/{rust,go,python}.dag)
# 28 names; only the four consumed through declaration_by_name
# are load-bearing today. Full set:
#   go_collection_ops, go_control_flow, go_expressions, go_functions,
#   go_language, go_literals, go_modules, go_patterns, go_statements,
#   go_type_defs, python_control_flow, python_expressions,
#   python_functions, python_language, python_literals, python_modules,
#   python_patterns, python_statements, rust_collection_ops,
#   rust_control_flow, rust_expressions, rust_functions, rust_language,
#   rust_literals, rust_modules, rust_patterns, rust_statements,
#   rust_type_defs
```

**Audit correction (v2, post-review).** The initial overlap set
was derived from a truncated `head -40` of each authority's
declaration list and undercounted `std.effects` entries. **Audit
correction (v5, post-review #3)**: the original audit also missed
the `dsl/std/languages.dag` ↔ `src/v3/spec/*.dag` duplication
entirely. Full `comm -12` across all four duplicated pairs yields
**52 overlap names** — 24 across effects/verification/http_path
plus 28 language-spec names — not the 18 originally listed. The
expanded set drives the expanded (b) count below.

Any `declaration_by_name("X")` whose `X` is outside this set is
rank-insensitive: it returns the sole existing declaration (or
None), regardless of scaffold state.

## Method

Enumerated every `declaration_by_name(…)` and every lowering path
that consumes `collect_symbols`'s symbol table. Static string
arguments were extracted and cross-checked against the overlap set.
Dynamic arguments were classified by semantic role.

Totals: **210 call sites total** (210 grep matches including doc
references). Actionable call sites (non-comment): **113
literal-string + 3 named-const `declaration_by_name("...")` /
`declaration_by_name(CONST)` sites in `src/v3/compiler/src/`**
(116 total static-valued), **67 in `src/v3/compiler/tests/`**,
plus **6 dynamic-form sites in `src/v3/compiler/src/`**:
- 2 bulk consumers (`collect_symbols` + the stub-resolution sweep
  at `lower.rs:2134` / `lower_generated.rs:2143`) — (c)
- 2 singletons-only helpers (`infer.rs:1619` →
  `{Int,Bool,String}`, `regen_tokenize.rs:159` →
  `{dag_keyword_set,dag_operators}`) — (a) despite dynamic form
- 2 user-name dispatch helpers that range over arbitrary
  declaration names (emit `parent_name` path at `emit.rs:2909`,
  `emit/rust_target.rs:2391`, `emit/python_target.rs:1837`; lens
  `sum_name` at `lens_testgen.rs:330`) — (c), same class as
  stub-resolution

**Audit correction (v10, post-review #4).** Earlier drafts
grepped for the literal-quote form `declaration_by_name("` and so
under-counted three named-const call sites
(`PIPELINE_STAGE_BINDING_TYPE = "PipelineStageBinding"` at
`pipeline_authority.rs:31`; `PIPELINE_REALIZATION_META =
"CompilerHostRealization"` at `bootstrap.rs:303`;
`LENS_REGISTRY_ENTRY_TYPE = "LensRegistryEntry"` at
`bin/regen_lens.rs:111`) and four dynamic-form (c) sites
(emit-parent + lens-testgen, above). All three const values
resolve to non-overlap names, so the const sites are (a); the
four dynamic-form sites join the (c) bucket as
user-name-dispatch helpers.

## Classification

### (a) Incidental — rank-insensitive, safe

**Of the 113 static-name call sites in `src/v3/compiler/src/`, 109
are (a) and 4 are (b).** Four static names are in the language-spec
overlap set and move to (b): `rust_language` (dag.rs:2269),
`go_language` (dag.rs:2282), `python_language` (dag.rs:2292),
`rust_functions` (dag.rs:2309). The remaining 109 sites look up
singleton substrate/spec/stdlib authorities absent from any `dsl/`
duplicate.

Unique rank-insensitive names (single-authority):
`BehaviorRealization, Bind, Bool, BooleanAlgebra, Branch,
CallableRealization, CallableStrategy, CleanEmissionContract, Dag,
dag_model, DeclarationId, DeclarationRef, fold, go_execution_model,
go_execution_requirement, go_source_filtering,
head_or_zero, id, Int, LanguageSpec, List, Loop, Main, MyInt,
NodeId, OperatorRealization, OrderedRing, parse, PatternBindingRule,
PatternRealization, PipelineSnapshotKind, PortId,
python_execution_requirement,
python_source_filtering, python_target, rust_clean_emission_binding,
rust_execution_model, rust_execution_requirement,
rust_rendering, rust_source_filtering, Sign, String,
SubstrateAccessorBinding, TargetCleanEmissionBinding, Transform,
TypeInstantiationRealization, TypeRealization, use_callback, Value,
VariantPayloadFieldAccessRule, VerifierOutputPolicy`. (Removed from
this list post-review: `rust_language, go_language, python_language,
rust_functions` — all four have `dsl/std/languages.dag` duplicates
and are reclassified (b).)

**Tests, rank-insensitive subset** — 44 of 67 test sites. Unique
names outside the overlap set include `answer, BinaryOpRow,
BracketRow, cfg, claim_obligation_resources, Classical,
CompilerHostRealization, DegreeAtLeastTwo, Dimension, div, f, first,
get_host, id, Int, KeywordTokenKind, LensRegistryEntry, List,
materialize_test_obligations, MyList, node, OrderedRing, pair, port,
PostEmitVerifier, PunctTokenKind, resolve_producer, ResourceHandle,
rust_clean_emission, Secret, SoftKeywordIdentRow,
SubstrateAccessorBinding, TestObligation, TokenKind,
TopLevelItemKwRow, WorkflowEffect`. (`WorkflowEffect` and
`TestObligation` exist only under `src/v3/`, so they are also
rank-insensitive — but see dissolution note for **(c)** below.)

**Dissolution impact:** zero. Drop the rank: same behavior.

### (b) Silent dependency — load-bearing on rank

**27 call sites** (direct + helper-mediated) look up a name in the
overlap set: 5 `TestClaim`, 13 `std.effects`/`http_path`, 9 language-
spec. Revision history: v2 added helper-mediated TestClaim sites;
v5 added the language-spec group with 5 direct-call sites; v7
found 4 more helper-mediated language-spec sites
(`find_named(...)` in `m1_substrate_test.rs:679` and
`m2_substrate_inhabitance_test.rs:{665,666,667}`) that the earlier
grep missed. 23 sites live in `src/v3/compiler/tests/integration/`;
4 live in `src/v3/compiler/src/dag.rs` (the init-pass surface
lookups).

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

5. **Language-spec lookups** — 9 sites total. Direct:
   `dag.rs:2269` (`rust_language`), `dag.rs:2282` (`go_language`),
   `dag.rs:2292` (`python_language`), `dag.rs:2309`
   (`rust_functions`); plus `m1_substrate_test.rs:2851`
   (`rust_language`). Helper-mediated via `find_named`:
   `m1_substrate_test.rs:679` (`rust_language`) and
   `m2_substrate_inhabitance_test.rs:{665,666,667}`
   (`rust_language`/`go_language`/`python_language`). All 9 names
   exist in both `dsl/std/languages.dag` and
   `src/v3/spec/{rust,go,python}.dag`. The init-pass assigns
   `target_syntax.{rust,go,python}_language` and
   `emit_anchors.rust_functions` from whatever rank returns — i.e.
   the v3 spec's declaration id; every downstream emission path
   that consumes these anchor ids is silently gated on the v3
   authority; test assertions reading these same names via
   `find_named` likewise see the v3 spec. **Systemic silent — the
   emission pipeline's anchor binding.**

**Dissolution path.** These sites are not a substrate problem;
they are symptoms of three convergence blockers — two already
tracked in ROADMAP, one newly surfaced by this audit:
- **`std.verification` convergence** — governs the 5 `TestClaim`
  sites (items 1, 2, 3 above). Tracked.
- **`std.effects` convergence** (plus embedded `http_path` mirror)
  — governs the 13 `lane2_stage_2a_effects_smoke.rs` sites (item
  4). Tracked.
- **Language-spec convergence** — governs the 9 init-pass +
  test-side sites (item 5). `dsl/std/languages.dag` vs
  `src/v3/spec/{rust,go,python}.dag`. **Not yet tracked as a
  file-preference-scaffold blocker** — the ROADMAP "Post-merge
  debt" row names only the first three pairs. Adding this fourth
  pair is a recommendation below.

Per convergence, tests either become (a) incidental (v3 authority
survives), or migrate to the surviving shape (dsl wins / merged
surface emerges). No new substrate work needed — the convergences
*are* the dissolution for the two tracked groups. The
language-spec group needs a ROADMAP entry before its convergence
can be scoped. **The "no new modeling gap" conclusion from the
draft still holds — but the scale (27 sites, not 2) materially
changes the cost-of-dissolution estimate for the `std.effects`
convergence, and surfaces a fourth duplicated-authority group
(languages vs spec) that the ROADMAP row must also cover.**

### (c) Legitimate-looking — scaffold's intended consumers

**Two classes** of consumer look up names programmatically from
inputs that can range over the overlap set, and so depend on rank
at the systemic level rather than at a single call site:

1. **`collect_symbols`** (`lower.rs:1269-1286`, mirrored in
   `lower_generated.rs:1259-1286`; also mirrored in
   `bootstrap.rs:165-185` as the `shared_symbols` rebuild used by
   the legacy PB-1-equivalence bootstrap path, with its own local
   copy of `declaration_name_preference_rank` at
   `bootstrap.rs:81`). Seeds the per-Dag symbol table used to
   resolve every identifier in every lowered `.dag` source file.
   When a surface identifier matches a name in the overlap set,
   the v3 authority wins. This is the scaffold's *reason for
   existing*: it lets v3-authored `.dag` code reference the v3
   surfaces of `EffectShape`, `TestClaim`, `PathTemplate`, etc.
   even while legacy `dsl/` duplicates remain ingested. Dissolution
   therefore deletes *three* rank-function copies (`dag.rs`,
   `lower.rs` import, `bootstrap.rs`) plus their application
   sites, not two.

2. **User-name dispatch helpers** — six sites that look up
   declarations by a variable name taken from user-authored
   sources. Same rank-consumer pattern as `collect_symbols`.
   - Stub-resolution sweep at `lower.rs:2134` /
     `lower_generated.rs:2143`:
     ```rust
     if let Some(target) = dag.declaration_by_name(&name).map(|d| d.id)
     ```
     Repairs `UnresolvedIdentifier` stubs after bodies are lowered.
   - Emit-time parent resolution at `emit.rs:2909`,
     `emit/rust_target.rs:2391`, `emit/python_target.rs:1837`
     (`let parent = dag.declaration_by_name(parent_name)?;`). When
     `parent_name` is an overlap-set name, the emit pipeline binds
     to the v3 authority.
   - Lens-testgen sum dispatch at `lens_testgen.rs:330`
     (`.declaration_by_name(sum_name)`). Looks up user-authored
     sum types by name.

**Dynamic-form call sites that are nevertheless (a) incidental.**
Two sites invoke `declaration_by_name(name)` with a variable
argument but only range over singleton names and so are
rank-insensitive despite the dynamic form:

- `infer.rs:1619` — `literal_decl_id` helper, always one of
  `{"Int", "Bool", "String"}`.
- `regen_tokenize.rs:159` — iterates literal array
  `["dag_keyword_set", "dag_operators"]`.

An earlier draft of this audit classified both as (c); both are
(a). Reclassified after review.

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

**Dissolution path for (c).** Same as ROADMAP (extended per
Recommendation 2): converge all four duplicated authorities —
`std.effects`, `std.verification`, the `http_path` mirror, and
`dsl/std/languages.dag` ↔ `src/v3/spec/{rust,go,python}.dag`. Once
converged, these consumers revert to vanilla single-authority
lookup; the rank function (three copies — `dag.rs`, `lower.rs`,
`bootstrap.rs`) and their application sites delete together.

## Summary table

| Category | Count | Sites |
|---|---:|---|
| (a) Incidental | 158 | 112 `src/` static-valued sites (116 literal+const minus 4 language-spec overlap) + 44 test sites + 2 dynamic-form helpers ranging over singletons only (`infer.rs:1619`, `regen_tokenize.rs:159`) |
| (b) Silent dependency | 27 | 5 TestClaim (`m1_5_testgen_test.rs:208`, `lane2_stage_2c_db15_test.rs:9`, `m1_5_verification_test.rs:{76,203,207}`); 13 in `lane2_stage_2a_effects_smoke.rs:{62,63,64,65,66,76×2,90,94,95,97,98,100}`; 9 language-spec (4 in `dag.rs:{2269,2282,2292,2309}` + `m1_substrate_test.rs:{679,2851}` + `m2_substrate_inhabitance_test.rs:{665,666,667}`) |
| (c) Legitimate-looking | 2 classes | (i) `collect_symbols` + stub-resolution sweep; (ii) user-name dispatch helpers (`emit.rs:2909`, `emit/rust_target.rs:2391`, `emit/python_target.rs:1837`, `lens_testgen.rs:330`) |

PR-body framing: **158 safe, 27 silent, 2 class-level legitimate.**
(Site totals: 116 static-valued `src/` + 67 tests + 2
dynamic-singleton helpers = 185 actionable call sites; 185 − 27
(b) = 158 (a). Four additional dynamic-form `src/` sites dispatch
on user-authored names and are classified (c) rather than counted
as individual (a) sites — see §(c).)

## Recommendations

1. **No lane needed to repair the (b) sites independently.** All 27
   sites dissolve automatically when their governing convergence
   lands (5 with `std.verification`, 13 with `std.effects` /
   http_path mirror, 9 with language-spec convergence).

2. **Extend the ROADMAP "Post-merge debt" file-preference-scaffold
   row to include a fourth duplicated-authority pair**:
   `dsl/std/languages.dag` ↔ `src/v3/spec/{rust,go,python}.dag`. 28
   overlapping names (full list in overlap-set block above); 4
   currently consumed through `declaration_by_name`. Convergence
   direction TBD (v3 spec splits languages.dag's surface into
   per-target files; a design call is needed on whether dsl's
   monolithic `languages.dag` survives, the v3 per-language split
   survives, or a merged form emerges).

3. **Consider a tightening helper** *after* convergence: a
   fail-closed `declaration_by_name_unique` that returns the single
   declaration or emits a diagnostic on multiple matches, used as
   the post-scaffold replacement. Not needed now; mentioned here so
   the dissolution PR has a named target.

4. **One modeling-surface gap surfaced.** The audit confirms the
   scaffold's dissolution blockers are *four* duplicated module
   pairs, not the three named in the ROADMAP row. The
   language-spec pair (`dsl/std/languages.dag` ↔
   `src/v3/spec/{rust,go,python}.dag`) is newly surfaced here and
   needs its own convergence decision — see recommendation 2. The
   reflective-analysis concern that "new lanes may silently depend
   on rank preference" *is* realized — the lane2 Stage 2a
   effects-smoke lane added 13 silent-dependency sites (via
   `arrow_body` / `assert_record_type` helpers) after the scaffold
   went in. Cost of the `std.effects` convergence when scoped is
   meaningfully higher than the ROADMAP row suggests.

5. **Watchpoint for future lanes.** Adding any new `declaration_by_name("X")`
   where `X` is in the overlap set above reintroduces a (b)
   dependency. A lightweight CI check (grep the overlap-name list
   against `declaration_by_name` call sites outside a known
   allowlist) would catch this at review time without modeling
   work. This is mentioned as a watchpoint; not proposed as a
   deliverable of this audit per the "no ratchets for textual
   enforcement" feedback principle.

## Non-goals respected

- No change to `dag.rs:2012-2020`, `lower.rs:1247-1286`, or
  `bootstrap.rs:{81,165-185}`.
- No change to any consumer.
- No attempt to resolve `std.verification` convergence.
