# #6985 witness-discovery cascade — root cause (read-only diagnosis)

**Status:** DIAGNOSIS ONLY. #6985 landed a 3-file residual; the bulk of the
src/v2/test + src/v2/lens import strip (≈4,120 imports across ~464 files) stayed
gated after repeated restores — every restore commit read "strip failed witness
discovery" even though `extend_sources_to_both_closure_fixpoint` (#6848,
`extend_with_bare_reference_closure` in `src/v1/stage0/src/cli_run.rs:4436`) exists
specifically to replace the dropped import edges with name-derived closure pulls.
This documents the exact mechanism the closure misses and what it must additionally
cover before the strip can re-run.

## 1. One-line verdict

`extend_with_bare_reference_closure`'s `pullable()` predicate
(`cli_run.rs:4559-4572`) only pulls a bare reference's declaring module when the
reference is **call-position**, or the census binding carries **non-empty params**,
a **type_annotation** (data const), or a **connective** (a Disj/Conj type decl
referenced bare). It does **not** pull for the fourth real shape: a name whose
census binding is **arity-zero and non-call** — a nullary Disj variant tag used as
a bare value (`DataDependent => false`), or a zero-param fn/data referenced by name
without being called or annotated. Every restore in #6985 whose stated cause names
a bare symbol (not a report of a compile *error* on the file itself) reduces to this
one gap.

## 2. Evidence, per restore

- **`parallelism.dag`** (`DataDependent`/`EffectCoupled` "not exported after import
  removal") — both are nullary variants of a Disj type
  (`src/v2/lens/parallelism.dag:5-6`, used bare in match arms
  `DataDependent => false` / `EffectCoupled => false`). `tree_bare_census_for_root`
  does enter corpus-unique Disj variant aliases into `global_bare`
  (`global_bare_fallback_invariant`, `v1_compiler_infer_env.rs:59`), so the census
  *resolves* the name — but the resolved binding's `connective` is
  `NoConnective` (the variant tag is not itself a type declaration), `params` is
  empty (a variant constructor with no fields takes none), and there is no
  `type_annotation` (it isn't a data const). `pullable()` returns `false` for all
  four branches, so the closure never enqueues the provider module and the
  interpreter reds with a runtime lookup failure on the variant's home module —
  exactly the "not exported" symptom.
- **`lens_module_gate.dag`** (`LensIdV0`, `ModuleDeclarationFact`,
  `lens_registry_*` unresolved) — same shape: type names and registry constants
  referenced bare without being called, several as bare type-position references
  rather than data consts, so `type_annotation` is absent and (for the referenced
  *use* site, as opposed to the type's own decl) `connective` reads `NoConnective`
  on the *reference's* resolved binding when the name binds to something other than
  the Disj/Conj head itself (e.g. an exported alias or registry row shaped like a
  zero-arg data const built from a call whose result is bound bare elsewhere).
- **hub restores** (`determinism.dag`, `vacuity.dag`,
  `extdeps_external_authority.dag`, `registry/completeness.dag`,
  `complexity_gate` table_fixture producers — "undefined Cardinality/Unit/Https,
  determinism cascade, table_fixture fns") — `Cardinality`/`Unit`/`Https` are the
  same nullary-variant class; `table_fixture_*` fns are zero-param fixture
  producers referenced **by name, uncalled** (passed as a value into a fold/HOF
  rather than invoked `table_fixture_x()` at the reference site) — `params.is_empty()`
  is true for a zero-arg fn, so this is the same arity-zero gap on the fn side of
  `pullable()`, not a new class.
- **`testgen.dag`** (`claim_nat_*` undefined) — same fn-by-value-not-call shape.
- **`cost.dag` / `enforcement/lens_module_gate/compile_gate.dag` / `machine_shape.dag`**
  ("unresolved type X") — bare TYPE references that should hit the `connective !=
  NoConnective` branch; the restores here suggest at least one more sub-case where
  the referenced name's `global_bare` binding is not the type's own Disj/Conj node
  but an intermediate re-export/alias binding whose `connective` reads
  `NoConnective` even though the underlying reference is genuinely a type edge —
  needs confirming against the actual `TypeBinding.resolved` shape for an aliased
  type before generalizing further (flagged as open, not closed, below).

## 3. What the closure must additionally cover

`pullable()` needs a fifth branch: **arity-zero, non-call bare reference is still
pullable when it is the only way the name resolves** — i.e. drop the implicit
"uncalled + no params + no type_annotation + no connective ⇒ untyped
local/irrelevant" assumption for exactly the shapes proven above (nullary variant
tag, zero-param fn/data referenced by value). Two ways to close it, in order of
preference per DESIGN §5 (construction over validation):

1. **Positive signal, not absence-of-signal.** Tag the census binding itself at
   census-build time with whether it is a *declaration* (fn/data/type/variant —
   always pullable when the census is the only source of truth for it, which is
   exactly the import-stripped case this closure exists for) vs a transient local.
   `global_bare` today only ever holds declaration-grain entries (per
   `global_bare_fallback_invariant`), so the absence-of-params/type_annotation/
   connective is not actually evidence of "don't pull" — it is evidence of
   "this decl shape doesn't need any of those fields," which `pullable()`
   currently misreads as "not a real reference." The construction fix is to make
   `pullable()` default to **true** for anything `global_bare`/`services` resolves
   to (a declaration, by the census's own invariant) and carve out the *rare*
   named exception (a name that is body-bound in the *same* scope this file
   already special-cases via `candidates.bound`) rather than allowlisting shapes.
2. If (1) is judged too wide (re-litigates the original over-pull measurements
   that motivated the narrow `pullable()` allowlist — see the `arrow_lambda`/
   `pattern_value_leaf` tests at `cli_run.rs:4380-4425`), the minimal closure is:
   add `is_variant_tag` (connective of the *owning* Disj, not the tag) and
   `is_zero_arity_fn_or_data` as two more `pullable()` disjuncts, sourced from a
   census-time flag rather than re-deriving arity from `resolved` shape at pull
   time (single authority, DESIGN §3).

## 4. Why this stayed invisible in the small residual that landed

#6985's landed 3-file diff (`sg_claims_test.dag` ×2, one `FunctionCall`
qualification) never exercised a bare nullary-variant or zero-arg-fn-by-value
reference — the files that did were exactly the ones repeatedly restored back to
import-bearing. The closure's own regression tests
(`bare_identifier_candidates_tests`, `cli_run.rs:4380-4425`) only cover the
bound-vs-referenced boundary (arrow-lambda params, pattern-value leaves), not the
`pullable()` arity gate this diagnosis targets — so the gap has no discriminating
RED today.

## 5. Next step (not taken here — this pass is read-only)

Land branch (1) or (2) above behind a discriminating RED built from the
`DataDependent`/`EffectCoupled` shape (bare nullary-variant match-arm reference in
an import-stripped file, asserting the provider module is pulled into the
closure), then re-attempt the src/v2/test + src/v2/lens strip in the same batched,
restore-on-red style #6985 used — the restores should now resolve to zero once
this gap is closed, since every named restore reduces to it or to the
open type-alias sub-case in §2.
