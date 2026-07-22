# #6985 witness-discovery cascade — root cause (read-only diagnosis)

**Status:** SUPERSEDED BY EXECUTION — §1's headline mechanism is refuted; see §5.
Kept in place (not rewritten) so the refutation is traceable against the original
reasoning, per DESIGN §5 ("reflection evidence ≠ structural proof"). Do not act on
§1–§4 below; read §5–§6 first.

#6985 landed a 3-file residual; the bulk of the
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

## 5. Execution receipt — §1's central claim is REFUTED for the variant-tag case

Per DESIGN §5 ("reflection evidence ≠ structural proof — prove a read axis by
execution"), §1's headline claim was tested by execution, not just read. Result:
the claim as stated is **wrong**.

**Repro.** Scratch worktree at PR #7061's HEAD; fully stripped the `import` lines
(matching the real batch-strip shape, not a partial one) from all three files in
the chain that actually reference `DataDependent`/`EffectCoupled` bare:
`src/v2/lens/parallelism.dag`, `src/v2/lens/parallelism/data_dependency.dag`,
`src/v2/workflow/lens_parallelism_family_eval_test.dag`. Built `gunbc` locally
(`CTRL_BUILD_MODE=local cargo build -p v1-compiler --bin gunbc`) and ran the
witness through the real closure-loader path:

```
gunbc run --entry src/v2/workflow/lens_parallelism_family_eval_test.dag \
  --source-root src/v2 --source-root dag \
  --function witness_lens_parallelism_family_gate_closed --claim-run
```

**Predicted:** a missing-module/unresolved failure on `DataDependent`/
`EffectCoupled` (§1's arity-zero pullable() gap). **Observed:** clean compile,
`witness_lens_parallelism_family_gate_closed()` returns `true` — no error at all.

**Root cause of the mismatch, traced with `GUNBC_BARE_PULL_TRACE=1`:** the trace
shows `'DataDependent' -> v2.lens.common.algebraic_composition` firing directly —
`pullable()` DID pull it. Reading `merge_global_bare_variant_locals`
(`v1_compiler_infer.rs:15865-15899`) shows why: a variant name's `global_bare`
entry is a `GlobalBareUniqueBinding` whose `binding.resolved` is bound to the
**owning Disj type's node itself** (`owner.connective == Connective::Disj`,
checked at `:15879`), not to some connective-less binding for the variant tag in
isolation. So when `pullable()` reads `binding.resolved.connective`, it reads the
parent Disj's `connective`, which is `Disj` — satisfying the fourth branch
(`connective != NoConnective`) directly. §1's claim that a nullary variant tag's
resolved binding carries `connective == NoConnective` is **false**: the census
never binds a variant name to a connective-less standalone binding in the first
place; it binds it to its coproduct owner.

**What survives, what doesn't:**
- The `DataDependent`/`EffectCoupled` evidence in §2 (the `parallelism.dag`
  restore) is **refuted**. Nullary Disj-variant tags used bare in match arms are
  already pullable today; #6848's closure already covers this shape correctly.
- The zero-param-fn/data-referenced-by-value evidence in §2 (`table_fixture_*`,
  `claim_nat_*`) is **not yet re-tested by execution** — `params.is_empty()` on a
  genuine zero-arg fn's own binding does not obviously inherit a parent-node
  rescue the way a variant tag does (a fn has no "owning coproduct"), so this
  class needs its own discriminating repro before it can be trusted either way.
- The actual cause of the `parallelism.dag` restore is more likely the shape
  surfaced by an earlier, non-faithful (single-file) repro pass: stripping
  `parallelism.dag`'s imports while `data_dependency.dag` still carried
  `import v2.lens.parallelism { DataDependent, ... }` (treating
  `v2.lens.parallelism` as a re-export source for a name it never declares —
  the true home is `v2.lens.common.algebraic_composition`) produced a real
  failure, but of a different shape: `name 'DataDependent' not found in module
  'v2.lens.parallelism'` — a broken qualified-import/re-export reference, not a
  bare-closure miss. This is a distinct bug class from §1's `pullable()` claim
  and was likely an artifact of the batch strip running file-by-file with some
  files landing before their sibling consumers, not a closure predicate gap at
  all.

**Verdict:** §1–§4 of this document are **not confirmed** as the mechanism behind
the `parallelism.dag` restore and should not be used to scope a fix. The fn/data
zero-arity claim remains open pending its own execution proof. No fix should be
attempted from this document as currently written; the next pass needs a fresh,
execution-first investigation into what actually broke, starting from the
qualified-import/re-export shape above.

## 6. Next step

Before any fix: (a) execution-test the zero-arity fn/data-by-value claim the same
way (a discriminating repro, not a reading), since it is not yet known to survive
either; (b) investigate the qualified-import/re-export mismatch shape as a
candidate real cause, since it did reproduce a real failure by execution; (c) only
land a `pullable()` change once its motivating case has itself been proven by
execution, not derived from reading `cli_run.rs` alone.
