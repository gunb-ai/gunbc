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

## 6. Controlled-pair + retrodiction (namespace-resolution-design.md §8 PR-4 lead)

Per nimble-owl-658's follow-up, the re-export lead from §5 has a name in the
design: §8 "import-from-definer migration" (`namespace-resolution-design.md`,
PR-4) — the corpus deliberately relies on re-export transitivity today, and
PR-4 is the staged step that migrates every import to name its true definer.
Two discriminating predictions were run by execution, plus one retrodiction
against a real #6985 restore commit:

**(1) Controlled pair — CONFIRMED.** Same scratch worktree, same 3-file chain.
Strip *only* `parallelism.dag` (the re-exporter), leaving `data_dependency.dag`
and `lens_parallelism_family_eval_test.dag` import-bearing:
```
error: name 'DataDependent' not found in module 'v2.lens.parallelism' (imported by 'v2.test.lens_parallelism.data_dependency')
error: name 'DataDependent' not found in module 'v2.lens.parallelism' (imported by 'v2.test.workflow.lens_parallelism_family_eval')
error: name 'EffectCoupled' not found in module 'v2.lens.parallelism' (imported by 'v2.test.workflow.lens_parallelism_family_eval')
```
Then stripping the two importers' `import v2.lens.parallelism { DataDependent,
EffectCoupled, ... }` lines as well (making the references bare, matching the
real batch-strip's simultaneous-file shape) returns to green. This confirms
partial-strip-through-a-re-export-chain as a real, reproducible failure mode,
independent of `pullable()`.

**(2) Retrodiction — PARTIALLY CONFIRMED, one open residual.** Checked
`25a751d712` ("Restore imports on 9 hub files"). `determinism.dag`,
`vacuity.dag`, `extdeps_external_authority.dag`, `registry/completeness.dag`
were restored importing **directly from std/extdeps** (`std.determinism`,
`extdeps.uri`, `std.disposition`, ...), not from another local hub — so this
does **not** fit the re-export-mismatch shape from (1) as-is; these files'
*own* bare references were the ones failing, on their own imports. Repro:
stripped only `determinism.dag`'s imports, left the consumer test file
(`reach_witness_test.dag`) import-bearing, ran
`determinism_flags_map_keys_reach` via `gunbc run --claim-run`. Result: a real
failure, but a **different shape** than (1) — `runtime error: type error:
error type cascade at src/v2/lens/determinism.dag:1983-1984` (the byte range
lands on a `NonDeterministic { source: root.source }` construction site), not
a "name not found" resolution error. `GUNBC_BARE_PULL_TRACE` shows
`Determinism`/`Deterministic`/`NonDeterministic` never appear as pulled names
at all — they resolve some other way (or fail silently upstream and only
surface as a downstream type cascade). This is a **second, distinct failure
class**, not yet root-caused, and not explained by either the refuted §1
`pullable()` claim or the (1) re-export-mismatch shape.

## 7. Verdict + wave rule

Two real, execution-confirmed failure classes exist behind #6985's restores,
not one:
- **Class A — partial strip through a re-export chain** (parallelism.dag):
  confirmed by a clean discriminating pair (§6.1). Fix direction: PR-4
  (import-from-definer migration) closes this class structurally — once every
  import names its true definer, there is no re-export chain left to break
  when a file is stripped independently of its "hub."
- **Class B — hub file's own bare references, non-"not found" failure shape**
  (determinism.dag and, per the same commit, vacuity.dag /
  extdeps_external_authority.dag / registry/completeness.dag): reproduced by
  execution but **not yet root-caused** — the failing names are never pulled
  nor explicitly refused, they cascade into a generic type-error. This needs
  its own investigation (start from where `Determinism`/`Deterministic`
  actually resolve today with the import present, and what differs with it
  absent) before a wave rule can claim to cover it.

**Wave rule (partial — covers Class A only):** a strip wave must either (a) be
closed under the imports-from-hub relation (strip a re-exporter and every file
that imports the re-exported name *from* it in the same commit — mechanically
derivable from the qualified-import graph), or (b) wait on PR-4 landing first,
which makes wave ordering irrelevant for Class A. **Class B is not yet
covered by any rule** — its mechanism is still open, so a wave rule scoped
only to Class A would still be un-derived-from-execution risk for any wave
touching `determinism.dag`-shaped hub files. The zero-arity fn/data-by-value
claim from the original (refuted) diagnosis also remains untested.

## 7.5 Class B discrimination (B1 vs B2) — B1 refuted by grep, lead redirected to B2

`Determinism`/`Deterministic`/`NonDeterministic` are declared exactly **once**
in the corpus (`dag/std/determinism.dag:4`, verified by
`grep -rn '^type Determinism\b'`) — an earlier hypothesis that they exist as a
homonym pair (a std copy and a `src/v2/lens/determinism.dag` copy, silently
mis-picked by `global_bare`'s nearest-ancestor LCP arm) is **refuted**:
`src/v2/lens/determinism.dag` declares no local copy, and every use inside
that file of these four names is fully **qualified** (`std.determinism.X`),
never bare — so no bare-name lookup on these four names occurs at all, and the
LCP-homonym mechanism has nothing to fire on here.

The trace redirects the lead instead to `DeclarationRef` (`std.decl_ref`,
imported alongside on the same stripped line, referenced **bare** in
type-annotation/field-type position — `DeterminismRoot { source: DeclarationRef }`
— and as a constructor in `map_keys_leak_source`). `DeclarationRef` also has
exactly one declaration site (no homonym), yet it never appears in
`GUNBC_BARE_PULL_TRACE`'s output at all — not attempted, not refused, silently
absent. This is consistent with type-annotation/field-type references
resolving through the typecheck/infer env path rather than
`extend_with_bare_reference_closure`'s loader-side pull loop, so that path
never triggers a module pull for `DeclarationRef` and the ungrounded field type
cascades into the `NonDeterministic { source: ... }` construction site as a
generic type-error rather than a loud refusal — not yet traced to a specific
line in the infer/typecheck code, so this remains a lead, not a confirmed
mechanism.

## 9. Class B root cause — CONFIRMED by execution (B2a and B2b both refuted; the real split is loader vs. env)

§7.5's premise ("`DeclarationRef` never appears in `GUNBC_BARE_PULL_TRACE`'s
output — not attempted, not refused, silently absent") is itself **refuted**
by a finer-grained trace. Two temporary `eprintln!` probes were added in a
throwaway scratch build (never committed) at the two decision points inside
`extend_with_bare_reference_closure`: (1) every name entering the `for (name,
service_head) in all_names` loop (`[bare-attempt]`), and (2) the
`target_module` value immediately after `resolve_in` runs, before the
`known_paths` short-circuit (`[bare-resolve]`). Running
`GUNBC_BARE_ATTEMPT_TRACE=1 gunbc run --entry
src/v2/lens/determinism/reach_witness_test.dag ... --claim-run --dry-run`
against the same import-stripped `src/v2/lens/determinism.dag` scratch worktree
used in §7.5 shows:

```
[bare-attempt] src/v2/lens/determinism.dag -> 'DeclarationRef'
[bare-resolve] src/v2/lens/determinism.dag -> 'DeclarationRef' -> Some("std.decl_ref")
```

So **B2a (producer gap) is refuted**: `DeclarationRef` IS produced by
`bare_identifier_candidates` (it is not caught by the KEY-POSITION,
binder-keyword, fn-param, or pattern-depth BOUND rules — none apply to a bare
type in field-type position), it IS in `all_names`, and `resolve_in` DOES
succeed against it (`pullable()` correctly fires on the `Conj` connective of
the `type DeclarationRef { ... }` record declaration, confirmed by reading
`Connective`'s definition at `v1_std_core.rs:110-115` and its Conj/Disj
assignment sites in `v1_compiler_infer.rs`). The reason no `[bare-pull]` line
ever printed in §7.5's trace is the `known_paths.contains(&dep_rel)` check
(`cli_run.rs:4668-4670`) firing *before* the trace print — `dag/std/decl_ref.dag`
was **already** in `known_paths` every time, so the closure's pull is a
silent no-op, not a silent failure.

The reason it is already known: `dag/std/determinism.dag` (never stripped —
only the *consumer* `src/v2/lens/determinism.dag` and its test were) itself
contains `import std.decl_ref { DeclarationRef }` as a genuine, unstripped
import edge, and the entry test file directly imports `std.determinism` (also
unstripped). So `dag/std/decl_ref.dag` reaches the corpus's LOADER-tier file
set through an ordinary, wholly unrelated transitive import chain — the
bare-reference closure never has to do any work for this name, and its
apparent "success" (`resolve_in` returning `Some`) is coincidental, not
load-bearing.

**This is the actual defect, and it is a different mechanism from both of
nimble-owl-658's B2 sub-hypotheses**: `extend_with_bare_reference_closure`
only ever governs the **LOADER tier** — which files get parsed and compiled
into the corpus (`cli_run.rs:3402-3419`'s own comment: "for the LOADER an
over-connected edge is harmless... The loader
(`extend_with_bare_reference_closure`) is deliberately left alone"). It says
nothing about, and never touches, the **per-file typecheck/infer
environment** that resolves a bare name used *inside* `src/v2/lens/
determinism.dag`'s own body — that environment is built from *that file's own
`import` statements*, which the strip deleted. Loading `dag/std/decl_ref.dag`
into the corpus (so it compiles standalone, and so any file that legitimately
imports it can resolve names against it) is orthogonal to whether
`src/v2/lens/determinism.dag` itself, with its imports gone, has a local
binding for the bare name `DeclarationRef` in its own scope. It does not, so
the field type `source: DeclarationRef` on `DeterminismRoot` is ungrounded at
typecheck/infer time regardless of `dag/std/decl_ref.dag`'s presence
elsewhere in the loaded corpus — producing the observed generic "error type
cascade" at construction time (`determinism_root_map_keys`'s use of
`map_keys_leak_source()`), never a located "unresolved type: DeclarationRef"
refusal on the field itself.

**Wave-rule consequence for Class B**: the closure change nimble-owl-658's
original diagnosis and nimble-owl-658's B2a fix location both assumed (patch
`bare_identifier_candidates`/`pullable()`, i.e. the LOADER) would do
**nothing** for this failure — the loader already succeeds. The actual gap is
that `extend_with_bare_reference_closure` has no counterpart that also
extends the referencing file's own typecheck-time import/name-binding scope
to cover bare references it resolved — i.e. resolving a name to a module at
load time and admitting that name into the referencing file's own local
env are two separate, currently disconnected mechanisms, and only the first
is patched. Any fix must add or extend the second (env-side) mechanism, not
the loader; this is the `src/v2/lens/determinism.dag` shape's specific
manifestation of the broader import-from-definer gap PR-4 targets, but PR-4 as
scoped (a loader/edge-naming fix) does not by itself cover it — env
construction is a distinct consumer that must also be re-pointed.

## 10. LOUDNESS defect (recorded separately, per instruction)

The failure surfaces at runtime as `type error: error type cascade at
src/v2/lens/determinism.dag:1983-1984` — a line number far outside the
141-line source file, evidence the number is an offset into a merged/compiled
representation, not a located position in the authored source. This is the
**inert `ExprError` arm**: once `DeclarationRef` fails to ground in
`src/v2/lens/determinism.dag`'s own typecheck env, the resulting field-type
error is not raised as a located, typed refusal on `DeterminismRoot.source`
(DESIGN §5 — "every path succeeds fully or fails with a typed, located
diagnostic") but instead propagates as an opaque `ExprError` that keeps
compounding through every downstream construction site that touches the
malformed type, arriving at the runtime boundary as a generic, unlocatable
"error type cascade." This absorption is *why* Class B stayed invisible in
the original small-residual landing (§4) and why B1/B2's early framing
("never appears in the trace... silently absent," §7.5) was itself
plausible but wrong — the loudness gap hides the true failure site well
enough that even code-reading the closure logic pointed at the wrong
mechanism. A located fix (raising a typed "unresolved type: `DeclarationRef`
in `src/v2/lens/determinism.dag`, field `DeterminismRoot.source`" refusal
at the point the env lookup fails, instead of deferring to a generic
downstream `ExprError`) is a prerequisite for diagnosing any *future*
instance of this env-side gap by inspection rather than by adding scratch
`eprintln!` probes, as had to be done here.

## 12. Reconciliation probe — §9's "loader vs. env split" theory REFUTED; the
real mechanism is pool-membership coincidence (execution-confirmed)

nimble-owl-658 accepted §9's finding but flagged a direct empirical
contradiction before wrap-up: ~74 `dag/extdeps` files (batch-1, #6938) are
*already* import-stripped on `main` today and typecheck green, despite
having bare cross-module references in field-type position exactly like
`DeterminismRoot.source: DeclarationRef`. Under §9's "the referencing file's
own typecheck env is built strictly from that file's own imports, and the
loader closure never feeds it" theory, they should fail the same way. They
don't. This required a second execution-tested probe before the diagnosis
could close.

**Probe.** Picked `dag/extdeps/bmc/types.dag` (already stripped by
`4ca7d52c30`, bare-references `ExternalAuthority`/`Uri`/`Https`/`NonEmptyStr`/
`Int`/`List`/`Secret` in construction position and `Watt`/`ByteSize`/
`HardwareThreadCount`/`Celsius`/`RevolutionsPerMinute` in field-type
position). Built a fresh scratch worktree at `main` tip with the same two
`[bare-attempt]`/`[bare-resolve]` trace probes used for §9, and a new,
narrowly-scoped entry file `test.claim.scratch_bmc_types_probe` that imports
only `RedfishProcessorSummary` from `extdeps.bmc.types` — deliberately
structured to mirror `DeterminismRoot`'s field-type bare-reference shape as
closely as possible, then run via the same scoped `gunbc run --entry ...
--claim-run --dry-run` path (not the confounded whole-tree `gunbc compile`).

**Result — the probe reproduces the exact same failure shape as
`determinism.dag`, on `main`, today:**

```
dag/extdeps/bmc/types.dag:16:15: error: unresolved type 'HardwareThreadCount'
dag/extdeps/bmc/types.dag:21:23: error: unresolved type 'ByteSize'
dag/extdeps/bmc/types.dag:36:25: error: unresolved type 'Watt'
dag/extdeps/bmc/types.dag:47:20: error: unresolved type 'Celsius'
dag/extdeps/bmc/types.dag:57:45: error: unresolved type 'RevolutionsPerMinute'
```

while in the *same run*, `ExternalAuthority`, `Uri`, `Https`, `List`,
`NonEmptyStr`, and `Int` all resolved and typechecked cleanly. This is the
discriminating data nimble-owl-658 asked for, and it answers both questions
at once — **the gap is neither positional nor shape-dependent; it is a third
factor, pool-membership coincidence, the same mechanism §9 already found for
`std.decl_ref`, just narrower here because the probe's closure is smaller
than the full repo:**

- `resolve_in` found `ExternalAuthority` → `Some("extdeps.external_authority")`
  and issued a genuine fresh `[bare-pull]` for it — succeeded outright, no
  coincidence needed.
- `resolve_in` found `Uri`/`Https` → `Some("extdeps.uri")` with **no** fresh
  pull logged — already known, because `extdeps/external_authority.dag`
  (itself freshly pulled) carries its own *unstripped* `import std.types {
  NonEmptyStr }`, which transitively loads `std/types.dag` into the pool —
  and `std/types.dag` is also where `List`, `NonEmptyStr`, and `Int` live, so
  all three resolve for free via that same accidental transitive chain
  (`grep` confirms both `extdeps/external_authority.dag` and `extdeps/uri.dag`
  carry `import std.types { NonEmptyStr }` on `main` today, unrelated to this
  probe).
- `resolve_in` returned **`None`** — not "resolved but not pulled," genuinely
  not found — for every `std.measure` name (`Watt`, `ByteSize`,
  `HardwareThreadCount`, `Celsius`, `RevolutionsPerMinute`). `grep -rl "import
  std.measure" dag/` shows real consumers (`product/budget_tree.dag`,
  `std/machine_shape.dag`, `std/cache_interface.dag`, etc.) exist tree-wide,
  but **none of them are reachable from this probe's closure**, so
  `std/measure.dag` never enters the pool at all, and `resolve_in`'s census
  has nothing to find.

**This retracts §9's "loader tier vs. per-file typecheck env are two
disconnected mechanisms" framing.** There is one mechanism, not two: a
module's own bare cross-references resolve exactly when the *target* module
has, by that point in the fixpoint, been loaded into the shared compilation
pool via *some* import edge — stripped or not, related or not, anywhere in
the transitive closure that's currently assembled. `std.decl_ref` in the §9
determinism.dag repro and `extdeps.uri`/`std.types` here both succeed for the
identical reason: an unrelated unstripped import happens to drag the target
module in first. `std.measure` fails here for the identical *absence* of
that same coincidence. **Batch-1's ~74 files typecheck green on `main` not
because of a different, working mechanism — they get lucky at the scale of
the whole repo**: enough files elsewhere still carry unstripped imports of
`std.measure`/`std.types`/etc. that those modules are reliably in the pool
by the time any given batch-1 file's own bare references are resolved. A
synthetic probe closure this narrow lacks that ambient coverage and exposes
the gap directly. This is exactly the "absorbing fallback [that] destroys
the only signal" pattern named in DESIGN §5, just realized structurally
rather than as a designed fallback: the corpus's own redundant import
density is silently doing load-bearing work, and nothing in the mechanism
is tracking that it's happening. **The wave rule is therefore corpus-wide
and load-bearing, not per-file:** any further import-strip batch's safety
depends on whether the *specific modules* the batch's bare references target
remain reachable via *some other* unstripped import somewhere in the
compiled closure at strip time — a property that can silently break as
*later, unrelated* strips remove the last such incidental import, with no
local signal at the newly-broken file.

**Side finding — the LOUDNESS defect (§10) is narrower than described.**
This probe's failures surfaced as **located, typed diagnostics**
(`unresolved type 'X' (file:line:col)`) via the same scoped `gunbc run
--claim-run` path used for the original determinism.dag repro, with exit
code 0 (not the opaque `error type cascade at ...:1983-1984` seen there).
So the opaque-cascade behavior is not a general property of the
`--claim-run` execution path, as §10 implied — it is specific to whatever
`determinism.dag`'s actual failing construction shape triggers (most likely
still the inert `ExprError` arm noted in §10, but only for some field-type
failure shapes, not all). §10's fix is still worth landing, but its scope
should be re-verified against both failure shapes before being called
complete, not assumed general from the single determinism.dag repro.

**Zero-arity fn/data-by-value claim**: not folded into this probe run;
still untested. Left as future work, unchanged from §11(c).

## 13. Consolidated wave-rule statement (diagnosis-complete)

- **Class A** (re-export-through-partial-strip, e.g. `parallelism.dag`) —
  CONFIRMED by execution (§7). Closed by either importing only from a
  module's original *definer* in wave order, or by landing PR-4
  (namespace-resolution-design.md §8, import-from-definer migration) first.
- **Class B** (hub-file own bare references, e.g. `determinism.dag`, and
  general bare-reference-in-stripped-file breakage, e.g. `bmc/types.dag`
  under a narrow closure) — root cause CONFIRMED by execution (§9, §12):
  **pool-membership coincidence**, not a positional/shape gap and not a
  clean loader/env split. A stripped file's bare cross-module references
  resolve only if the target module happens to already be reachable via
  some unrelated unstripped import in the currently-assembled closure.
  **This blocks ALL further import-stripping** (of `src/v2/test`,
  `src/v2/lens`, and any remaining `dag/**` batches) until a real fix lands:
  either (i) an explicit, closure-independent binding mechanism so a
  bare-resolved reference is guaranteed available regardless of incidental
  pool coverage, or (ii) a construction-time check that refuses a strip
  whose target module's bare references are not *provably* covered by a
  surviving import elsewhere in every closure that reaches the stripped
  file — never assumed safe from "it typechecks today," since today's
  safety is itself an accidental, silently-erodable property of the rest of
  the corpus.
- **LOUDNESS defect** — named side finding, narrower than first described
  (§12): the opaque unlocatable "error type cascade" is not general to the
  `--claim-run` path (it produces located diagnostics for this probe's
  failure shape); still worth a located-diagnostic fix for whatever
  triggers it in `determinism.dag` specifically, but re-scope before
  building it.
- **Zero-arity fn/data-by-value claim** — named side finding, still
  untested by execution; not folded into either probe run.

This work item closes as diagnosis-complete. Fixes (the env/pool-membership
mechanism, the LOUDNESS diagnostic, PR-4, and the zero-arity test) go to
fresh work items for the operator to sequence — none are authorized to land
from this investigation.

## 14. Post-flip re-observation — Class B is *type*-only, and it fails OPEN (CONFIRMED by execution, 2026-07-25)

§13 closed this diagnosis before the namespace flip landed. #7178 (Dispatch 1,
namespace-only resolution default ON) changed the substrate underneath it, so
Class B was re-run against the flipped tree. Two things changed, and the second
is worse than anything §13 records.

Trigger: #7200 (Dispatch 2, global import deletion) went red on exactly six
sites in `src/v2/std/node.dag`, all of the shape
`if branches resolve to incompatible types: Primitive(std.types.ContentHash)
vs Product(<anon>)`.

### 14.1 What was run

`gunbc compile --target dag`, release build at main+#7178, source roots
`dag` + a scratch dir, one entry per probe. Eight probes, each isolating one
variable. Subject throughout: a cross-module **type** name
(`std.types.ContentHash`) referenced from a module that does not import
`std.types`.

| # | spelling in type position | reachable on an import chain? | result |
|---|---|---|---|
| 1 | bare `ContentHash` | yes — own `import std.types { ContentHash }` | **GREEN**, 0 diagnostics |
| 2 | bare `ContentHash` | yes — transitively, via an imported module that imports `std.types` | **GREEN**, 4 *advisory* `UnlistedImportUse` |
| 3 | bare `ContentHash` | no — and absent from the pool entirely | **RED, correctly**: `unresolved type 'ContentHash'`, located |
| 4 | bare `ContentHash` | no — but *present* in the pool | `Product(<anon>)`, red only where an `if` juxtaposes it |
| 5 | qualified `std.types.ContentHash` | no — but *present* in the pool | `Product(<anon>)`, identical to row 4 |

Control, in the same file as rows 4/5: `fn local_int(x: Int) -> Int` called
bare in an `if` branch is **clean**. Same-module bare *callee* resolution
works. The failure is specific to the cross-module **type** name in the
signature. A second control: adding one *irrelevant* import (a sibling module
that does not reach `std.types`) does not help — so it is not a
"module has zero imports" degeneracy, it is import-chain reachability of the
name itself.

### 14.2 Finding 1 — the qualified spelling buys no binding

Row 5 is the load-bearing one. `std.types.ContentHash` written out in full, in
type position, resolves no better than the bare name. **There is no
containment-based binding path for cross-module type references today.** The
flip gave containment binding to values/functions (row-4 control) and not to
types.

This falsifies the premise Dispatch 2 was cleared on — that with binding by
containment, deletion no longer risks losing a resolution. That holds for
values; it does not hold for types. `namespace-resolution-design.md`'s Rule-1
end-state (delete the `import` grammar, derive deps from `container.member`
references) is therefore **not reachable for type references** on the current
resolver, in either spelling.

### 14.3 Finding 2 — the failure arm widens instead of refusing (§5 fail-open)

Compare rows 3 and 4. When the name is **absent** from the pool, the resolver
does the right thing: a typed, located `unresolved type 'ContentHash'`. When
the name is **present in the pool but not import-reachable**, it does not
refuse — it fabricates an anonymous product, `Product(<anon>)`, which then
unifies with everything downstream.

Two probes show it passing silently, with no `if` to catch it:

- fabricated `ContentHash` flowing through two functions →
  `0 blocking error(s), 6 advisory`.
- fabricated value fed directly into
  `std.content_hash.content_hash_combine(left: ContentHash, right: ContentHash)`
  — a real cross-module consumer carrying the real type →
  `0 blocking error(s), 4 advisory`.

**Argument-position checking does not catch it.** The only thing that caught it
in `node.dag` is the accident that six `if`-expressions juxtaposed a fabricated
branch against a correctly-typed one. The two `if`s in that same file whose
branches are *both* fabricated
(`byte_offset_residual_quotient_limb_pair_digest`,
`byte_offset_residual_quotient_magnitude_window_digest`) are **not** flagged —
fabricated-vs-fabricated agrees.

This is DESIGN §5's named class — *fabricated plausible output* — sitting
directly under a 1658-file mechanical strip. A corpus-wide strip could compile
GREEN and be silently mis-typed throughout, with only incidental `if`
juxtapositions reding. **Green would not have meant anything.** It is also why
§13's phrase "never assumed safe from 'it typechecks today'" understates the
risk for types: for types, typechecking today is not merely an accidental
property of the rest of the corpus, it is not even a signal.

### 14.4 Consequence for the wave rule

§13's blocker stands and **extends to Dispatch 2**, which was not exempt from
it. Restated for the flipped substrate:

- Import deletion is sound for **value/function** references (containment
  binding, row-4 control).
- Import deletion is **unsound for type references**, in both the bare and the
  qualified spelling, and its unsoundness is **not observable** from a green
  compile.
- Fixing the six `node.dag` sites would only restore the accident that made the
  defect visible. It is not a fix.

Ordering follows §5 (construction before validation, and a failure arm must
refuse rather than widen): the **fail-open is the first fix**, independent of
whether the strip ever resumes — pool-present-but-not-import-reachable must
become a typed, located, counted refusal. It is also the precondition that
makes a binding fix *verifiable*: until the arm refuses, no strip's green is
evidence. A containment binding path for type references is the second, and
only then is Dispatch 2 mechanical again.

Rows 3 and 4 are the discriminating pair any fix must turn: row 4 must become
row 3's shape (a located refusal), while rows 1 and 2 must stay green.

Nothing lands from this section either — it is a receipt. Dispatch 2's branch
is left in place as evidence.

**N2 design (quiet-hawk-219):** the follow-on design for the containment binding
path is [containment binding for cross-module type references](containment-binding-cross-module-type-references-design.md)
(import-deletion graph node N2; depends on N1 refusal floor).


## 15. Whole-corpus re-measurement (2026-08-09) — a ranked residual, not a countdown

§14 measured Class B on eight scoped probes. This section measures the **whole
corpus at once**, so the classes are denominated against each other rather than
each against itself.

**It establishes nothing about whether `import` can be deleted, and it is not an
exact worklist.** Of its 3,050 residual rows, 1,656 carry a probable but
*unobserved* mechanism and a further 1,109 are explicitly unclassified or
downstream; only 152 are corpus hygiene today. What follows is a ranking and a
set of obligations.

### 15.0 Maturity — three levels, not one frontier

An earlier revision of this section said "the closing frontier is 2 — A–D
establish, E and F remain unavailable." **That is withdrawn.** It was taken from
roadmap prose that is ahead of its own executable closing authority. Read the
authority instead: `gunbc.namespace_reference_derived_closure_contract`
`namespace_reference_derived_closure_acceptance_admissions` returns **six**
`ReferenceDerivedClosureUnavailable` rows — all six capabilities, including the
four A–D ones, whose trigger is `P2aStructuralCandidateProducer7515` — and
`reference_derived_closure_closing_contract_holds` is false for anything but a
frontier of zero.

The honest description separates three maturity levels the single-frontier
phrasing collapsed:

| level | subject | standing |
| --- | --- | --- |
| candidate/binding mechanics on authored parser fixtures | A–D | evidence exists |
| dependency projection and pool comparison on constructed carriers | E/F machinery | fixture evidence exists |
| the permanent ordinary-compiler closing contract | A–F | **all six unavailable** |

The carriers say this themselves, and cite-the-symbol beats paraphrase:
`gunbc.namespace_clause_e_projection_law` `namespace_clause_e_projection_law_note`
records that clause E is proven "on hand-built `ReferenceDerivedClauseEProduction`
carriers" and that "row closure over `OrdinaryLoadedCompilationClosure` is B2's
axis and is excluded from this module";
`std.occurrence_binding_candidates` `module_path_file_row_dissolution_note`
carries `ModulePathFileRow` as a scaffold "until B2 wires
`OrdinaryLoadedCompilationClosure` through `source_authority` directly"; and
`gunbc.roadmap_authority` `namespace_cross_file_provenance_lane_integration_note`
states the progress standard outright — "B0 exposure/assembly and B1
fixture-proven projection carriers do not close
namespace-cross-file-provenance; only the bound closing witness greens when
`OrdinaryLoadedCompilationClosure` production lands (B2)."

So the missing work is **an integration vertical**, not two remaining algebra
arms: the ordinary front-end must emit the same occurrence/declaration
identities it currently produces from hand-authored fixture strings, the
production type-reference changeover must thread the binding context into the
real resolve path, the loader must consume the dependency projection instead of
scanning imports and bare-name census guesses, and pool independence must be
proven against the loader's own output rather than against independently
assembled fixture closures.

### 15.1 What was run

Release `gunbc` built from `1eadad4af25`; no `.rs` changed between that commit
and the measured base, which is why the same binary is current for the measured
tree. (It was rebuilt for this lane because #7924 changed explicit-import
resolution after the previous strip attempt — measuring with a binary older than
the substrate it judges reports a compiler that no longer exists.) Two scratch
copies of `dag` + `src`; one left intact as the known-positive control, one
stripped with a **brace-depth-aware** pass (`sed '/^import /d'` orphans the
multi-line `import x { … }` blocks and inflates the reading roughly tenfold —
the trap recorded in §5's repro notes).

Strip completeness is denominated, not assumed: **16,315 import declarations
across 2,574 files, zero `import` residue**, with a per-file manifest.

The full reproducer — scripts, raw diagnostic logs, manifest, declaration
census, corpus hash and measured commit — is committed at
[`import-strip-measurement/`](import-strip-measurement/README.md), including the
deterministic command sequence that regenerates every number below. It is a
registered scaffold carrying its own dissolution trigger: when B2 lands, the
ledger becomes a projection of the loader's accepted-binding output and the
scripts delete.

**Sections 15.1–15.5 record the measurement as it stood BEFORE the corpus repair
(control 12, attributable 5,815 at `1eadad4af25`, then 3,047 after #8056). They
are kept as the reasoning that ranked the work. The current numbers, measured on
the head that carries the repair, are in §15.6 and §15.7 and supersede them.**

### 15.2 The two readings

| tree | hard diagnostics | exit |
| --- | --- | --- |
| control (unstripped) | 12 | 1 |
| stripped | 3,062 | 1 |

Both exit 1 — the control's 12 pre-existing annotation-grain diagnostics are
themselves hard, so exit status does not discriminate; only the count and the
per-name join do. The control is the operand, not a formality: an unstripped
compile reporting zero would have made those 12 read as strip damage. ("Known
positive" is this document's term for the unstripped control, not DESIGN's;
DESIGN §5's requirement is a real consumer **green by execution** plus a
discriminating input that goes *red* when the behavior is wrong, and the
stripped/unstripped pair is what supplies both halves here.)

The strip-attributable population is **3,050 diagnostics**, and the ledger
identity is checked by the classifier rather than asserted:

```
stripped_hard (3,062) = control_hard (12) + attributable (3,050)
attributable (3,050)  = Σ ledger rows (3,050)      [reconciliation: OK]
```

**These numbers are base-specific and supersede this section's first revision.**
That revision measured 5,815 attributable at `1eadad4af25`, before #8056 landed
the `cell`/`row` disambiguation; re-measuring at the current base gives 3,050.
Reading the two as a trend would be a change detector — the strip set, the
corpus and the base all differ. For the same reason the earlier 1,096 figure
from the pool-pull receipt (1,447 files stripped) is not comparable to either.

### 15.3 The residual, by disposition

Every failing name is joined against a corpus declaration index, so the class
derives from declaration multiplicity and first-failure shape rather than from
diagnostic prose. **No disposition claims a cause that was not observed**, and
the ones that would carry `_unobserved` in the name:

| disposition | rows | what it means |
| --- | --- | --- |
| `unique_decl_unresolved_mechanism_unobserved` | 1,656 | one indexed declaration, reference unresolved, mechanism not established |
| `variant_mechanism_unobserved` | 599 | variant-shaped, prior mechanism refuted, re-proof owed |
| `variant_owner_unindexed` | 249 | capitalised name the index missed (likely inline `type X = A \| B`) |
| `corpus_hygiene` | 152 | name declared in more than one module |
| `cascade` | 138 | type-expression / downstream mismatch |
| `ordinary_callee_unindexed` | 133 | call to a name the index does not carry (builtin, primitive, or generated) |
| `field_on_unresolved_or_wrong_type` | 88 | field access downstream of an unresolved or wrong receiver |
| `method_on_unresolved_receiver` | 22 | method on a receiver that never resolved |
| `record_shape_cascade` | 12 | missing required field in a literal |
| `unindexed_symbol_candidate` | 1 | residue |
| **total** | **3,050** | |

**The 1,656 are not clause E, and the previous revision was wrong to say so.**
It labelled them "unique declaration, provider not discovered" and dispositioned
them `E` while every row simultaneously recorded
`provider_in_loaded_closure = unobserved`. Those two statements cannot both
stand. What is actually established is: one indexed declaration, plus a stripped
compile reporting an unresolved reference. That is consistent with the provider
never entering the closure, with the provider entering but the occurrence not
being admitted into the referencing environment, with a wrong occurrence
category taking a legacy path, with a fail-open/fabrication arm rewriting the
downstream error, and with the index having matched a plausible declaration that
is not the intended binding. Promotion to clause E requires loader
instrumentation that observes the accepted declaration and whether its provider
entered the closure. Until then the disposition names its own ignorance.

**The strongest lead survives that correction intact.**
`LiveTreeDisposition` (503) and `SubstrateInputsOnly` (440) — **943 rows, 31% of
the entire residual and 57% of the 1,656** — are declared together in
`src/v2/std/live_tree.dag`. Hundreds of stripped files losing one provider
family is strong evidence that the missing cross-file provider/closure path is
load-bearing. It is a lead of exactly that strength: strong, and not yet a
measured seam.

**`variant_mechanism_unobserved` carries a re-proof obligation, not a feature.**
An earlier revision described these 599 rows as "the arity-zero/variant-tag
population §3 describes" — a hypothesis **this document's own §5 refutes by
execution** (a variant's `global_bare` binding points at its owning `Disj`,
whose connective satisfies `pullable()`; only the zero-arg fn/data-by-value case
is untested). Citing the refuted section in the document that recorded the
refutation is the stale-citation class DESIGN §3 names. No variant-tag feature
should be opened from this count. The re-proof must classify each row by owning
coproduct, provider module, whether that provider entered the stripped closure,
whether the first failure is at the tag / the parent type / downstream
inference, and whether the row disappears once its Class-B provider is
available. Likely outcomes are that most collapse into the provider-closure
problem, some are missing owner-type bindings, some are genuine zero-arg
by-value gaps, and some are wrong candidate assignments from same-leaf matching.

**Candidate providers are candidates.** The ledger no longer writes a single
`provider_module` for a row whose binding was never observed. It carries
`candidate_provider_modules` and `candidate_count` alongside
`intended_provider = unobserved` and `accepted_binding = unobserved`, because a
same-spelled index hit is a name match, and a single provider column launders it
into a binding fact.

**The index-miss buckets are mixed by construction and split accordingly.** The
previous revision put 505 rows in one `no_declaration_found` bucket and
described them all as "the index needs widening." Some are (`ReadsLiveTree`,
`Do`, `FailFast`, `Empty` are inline variant tags; `trim` is a primitive — the
same name #8062 is making import-explicit). Many are not: `count`, `first`,
`split` and `ends_with` are method resolutions on receivers that never resolved,
and `steps`, `argv_input_refs`, `content_identity`, `subject_identity`, `kind`
and `qualified_name` are field failures downstream of a missing parent type.
Those are symptoms of a missing provider, not separate index gaps, and the six
sub-dispositions above keep them apart.

### 15.4 Why 69% of stripped files compiling clean is weak evidence

1,773 of the 2,574 stripped files produce no diagnostic. That is **not** proof
their references are independently resolvable. The production type-reference
discriminator is measurement-only — normal compile, resolve, run and emit stay
on the legacy fail-open path unless measurement mode is armed — and §14
confirmed that a type present in the pool but not structurally bound can be
**fabricated as an anonymous product** rather than refused, passing real
consumers without a blocking error. A clean file may be clean because its
provider happened to be in the pool, or because a fabrication arm absorbed the
failure. The ledger marks `binding_outcome = suspected_fabricated` wherever the
diagnostic mentions `Product(<anon>)`, so those specimens are greppable rather
than buried in the cascade bucket.

The terminal test is therefore not `delete imports → compile exits 0`. It is:
every cross-module occurrence has exactly one accepted declaration identity;
every accepted identity projects its provider file; the loader closure equals
that provider projection; adding or removing an unrelated loaded module changes
neither binding nor closure; type, value, callee and emitter consumers all
consume the accepted edge; no fallback or fabrication arm is reachable; and
compile *and execution* are green.

### 15.5 A flat one-row-per-diagnostic ledger cannot size the work

The mixed buckets above are the proof of it: one missing `EffectPlan`,
`PublicationSubject` or `DeclFact` surfaces as many independent method, field
and record-shape failures, each occupying its own row. The ledger carries
`downstream_diagnostic_count` per (consumer, name) pair, which is a partial
grouping only — it does not link a field failure to the *provider* whose absence
caused it. Causal grouping (root occurrence → root provider/binding result →
dependent diagnostics) needs the same loader instrumentation clause E/F needs,
so it lands with that vertical rather than before it.

### 15.6 The hygiene bucket is closed — 152 actionable rows to 0

Everything above was measured before the corpus repair. This section records the
repair and re-measures on the same head, so the ledger and the fix no longer
disagree inside one PR (they did in an earlier revision: the receipts pinned a
pre-repair commit while the diff consolidated the very forks the ledger listed
as open — caught in review 50740).

**Before and after, both measured, both on the head that carries the repair:**

| | actionable `corpus_hygiene` diagnostics |
| --- | --- |
| before | 152 |
| after | **0** |

The bucket does not survive as a smaller number; the disposition is absent from
the ledger. What replaced it is four kinds of row, each named:

| disposition | rows | why it is not hygiene debt |
| --- | --- | --- |
| `per_module_convention_population` | 8 | one row per module by design (`extdeps_external_authority_anchor` ×497, `extdeps_model_scope` ×94) |
| `intentional_ambiguity_fixture` | 3 | fixtures that exist *so that* two declarations collide; renaming them deletes the test subject |
| `field_on_unresolved_or_wrong_type` | 73 | first failure is the receiver, not the name |
| `method_on_unresolved_receiver` | 66 | same |

**The two excusing dispositions are NAMED SUBJECTS WITH CHECKED PROPERTIES, not
thresholds — and an earlier revision of this PR got that wrong in the one place
it matters most.** It classified a convention by population size (ten or more
declarations) and an intentional collision by path shape (every candidate module
containing `.fixture.`), on the reasoning that a rule which generalizes beats a
list that must be maintained. Both rules fail OPEN, and they fail open *into the
number this section asserts at zero*: a genuine accidental fork that happened to
reach ten sites would have been reclassified as a convention and disappeared from
the hygiene count, and an unrelated duplicate between two fixture modules would
have been excused by its path. A rule whose failure mode is "quietly removes rows
from the figure under test" is the absorbing fallback DESIGN §5 forbids, aimed at
this measurement's one load-bearing claim (review 50775).

Each subject is now named, and the property that makes it one is checked against
the tree: a convention must be one declaration per module under a declared module
prefix, and an intentional ambiguity must match an exact pinned module set. A
subject that stops satisfying its property does not keep its disposition — it
lands in `duplicate_unclassified`, which is loud, counted, and not zero. Six
controls in `import-strip-measurement/classifier_controls.py` hold this in both
directions, and the two that matter are the ones the retired rules would have
failed: a planted ten-declaration accidental fork stays hygiene, and an unknown
duplicate across two fixture modules stays hygiene.

**What was done, by treatment.** Consolidated onto one authority:
`declaration_ref_eq` and `declaration_ref_in_list` (44 rows, the largest family)
onto `std.roster_frontier`; `Milliwatt`/`milliwatt`/`milliwatt_count` onto
`std.measure`; `srv3_nbd_proxy_local_port` onto its cited
`extdeps.bmc.webui.nbd_proxy_serve` row; `decl_facts_matching_qualified_name`
onto `v2.std.decl_facts_skeleton`; `gnu_bash_subject_ref` onto the derived row in
`gunbc.language_target_registry`, deleting the hardcoded `DeclarationRef`
literal. Renamed as genuine homonyms: `PublicationSubject` →
`RoadmapPublicationSubject` / `PublicProjectionSubject` (31 rows);
`NetworkInterface` → `DockerContainerNetworkInterfaceStats`; `ampere` →
`ampere_vendor` (the SI constructor keeps the name); `shape_from_catalog` →
`cpu_`/`gpu_`; `ChangeClassification` → `Bootstrap…`/`ChangeRealization…`;
`path_has_prefix` → `rust_source_path_has_prefix`; `note` → `instrument_note` /
`motion_note`. Renamed as fixture-locals: five witness-local `nid` helpers,
`authored` → `authored_status_fixture`, `site_artifact_digest` →
`fixture_site_artifact_digest`, `fixture_repository` →
`git_`/`mercurial_`/`pijul_`, `decl_facts_reflection_fixture_facts` →
`decl_facts_reflection_witness_support_facts`.

**`cell`/`row` are #8056's**, not this lane's: that PR authored the
`fragment_cell` / `fragment_row` disambiguation first, and landed as
`0ee2d85de2a`. This session measured the same repair under different names
before finding it and withdrew the competing edit.

**Two families were renamed on BOTH sides because their bodies diverge**, and
that is a finding rather than a cleanup: `runner_slot_unit_name` and
`RunnerReplacementCause` exist twice with different behaviour — two different
systemd unit-name constructions and two different cause vocabularies — and
`current_walk_attempt_id` differs in its absent-variable arm (`none` vs the empty
string). `gunbc.runner_lifecycle` carries `Disposition = SingleAuthority`, so it
keeps the names and `gunbc.runner_connectivity_recovery`'s copies are prefixed.
**Renaming makes the fork visible; it does not fix it.** Two functions still
compute a unit name differently, and that is a latent defect this section
records rather than resolves.

**Per-spec tokens stay with their spec.** `extdeps.w3c.accelerometer` keeps
`accelerometer_permission_name` because that is its own spec's token; the
DeviceMotion copy becomes `device_motion_accelerometer_permission_name`. Each
upstream spec module owns its own facts, so a shared literal is not consolidated
across specs.

**Global textual uniqueness was never the goal.** The namespace model preserves
distinct same-spelled declarations and feeds the candidate population to the
ambiguity fold. Class A split four ways and only the first two were corpus work:

| subclass | treatment |
| --- | --- |
| two declarations of one concept | consolidate onto one authority |
| two concepts sharing an over-generic leaf | rename semantically |
| two legitimate equal leaves on separate containment paths | preserve; qualify or bind structurally |
| the wrong declaration entering the pool | provider/closure work, not naming |

**A specimen the repair produced, worth more than the cleanup.** Renaming the git
fixture made another module's bare `fixture_repository` bind to an unrelated
declaration in a different witness file, producing five spurious "no field on
type `Repository`" errors. That is **pool coincidence reproduced on the
unstripped tree** — the same mechanism as the residual's largest class, visible
in ordinary compilation. A second one came from the measurement itself: the first
regeneration reported `corpus_hygiene: 2`, which turned out to be a rename of
mine colliding with an existing `import_resolution_facts_live` in
`v2.lens.module_graph`. It was named for its return type when the distinguishing
fact is its source, and is now
`reference_derived_import_resolution_facts_live`. The ledger caught a defect its
own author introduced, which is the argument for running it rather than
predicting it.

**The third defect, and the coverage gap under it (review 50749).** The
`reference_resolution_facts_live` rename in `v2.lens.module_graph` did more than
collide: the blunt pass also rewrote the CALL SITE inside
`dependency_resolution_facts_live`, so `reference_edges:` was sourced from
`import_resolution_facts_live` — the same producer already feeding
`import_edges:`. Bare-reference dependency edges were silently dropped from the
module graph, which feeds affected-set selection, import closure and
compile-clean scope. Two prose rows were rewritten to claim two import sources
as well.

It is repaired (`reference_edges:` now calls
`reference_derived_import_resolution_facts_live`, and both prose rows differ from
main by exactly the rename), but the interesting part is why nothing caught it.
**Both helpers have the identical signature** `(List<String>, List<String>,
List<String>) -> List<ImportResolutionFact>`, so substituting one for the other
is type-correct; the whole-tree compile stayed green at baseline through the
regression. And searching the corpus finds **no witness referencing
`dependency_resolution_facts_live` or `union_import_resolution_fact_lists` at
all** — the two-source union has no discriminating control anywhere, so a wrong
producer at that seam is invisible to the executing corpus. External review
caught what no check does.

That gap is recorded here rather than closed: a real control needs a fixture
where a module is reachable by bare reference but NOT by import, so the two arms
provably disagree — which is the same fixture shape the ordinary-loader vertical
needs, and belongs with it rather than bolted onto a naming PR. Filed as an
obligation, not a plan.

### 15.7 What the residual is now, and what it is not

Re-measured on the head carrying the repair, with the reconciliation checked by
the classifier:

```
stripped_hard (3,098) = control_hard (10) + attributable (3,088)
attributable (3,088)  = Σ ledger rows (3,088)      [reconciliation: OK]
```

| disposition | rows |
| --- | --- |
| `unique_decl_unresolved_mechanism_unobserved` | 1,893 |
| `variant_mechanism_unobserved` | 600 |
| `variant_owner_unindexed` | 249 |
| `cascade` | 138 |
| `field_on_unresolved_or_wrong_type` | 73 |
| `method_on_unresolved_receiver` | 66 |
| `ordinary_callee_unindexed` | 45 |
| `record_shape_cascade` | 12 |
| `per_module_convention_population` | 8 |
| `intentional_ambiguity_fixture` | 3 |
| `unindexed_symbol_candidate` | 1 |

`corpus_hygiene` is **absent from this table, at zero**, which is the close this
section reports. So is `duplicate_unclassified` — the disposition a named
convention or fixture subject falls into when the property that names it stops
holding — so the two remaining excusing dispositions above are not thresholds
that absorbed anything: each was checked against the tree, and both held.

Three numbers moved for reasons that are not this repair, and saying so is the
difference between a measurement and a scoreboard. The unstripped control has
now FALLEN 22 → 10, because main landed #8072 healing part of the annotation-grain
class; every one of the remaining 10 sits in
`dag/test/claim/host_phase_status_witness_test.dag`, a file this branch does not
touch, so the hygiene batch still adds **zero** diagnostics — and this time the
evidence is the located diagnostics themselves rather than a separate run. The
corpus grew again (16,382 imports across 2,583 files) because main advanced
during the work. And `unique_decl_unresolved_mechanism_unobserved` moved with
that advance, not with the renames: it is 1,893 here against 1,656 before main
moved at all.

The re-measurement itself is not free of the effect it measures, which is worth
stating: these figures come from a tree that includes this branch's own four
fixture modules and one new witness. Their contribution is bounded by the control
above — zero diagnostics unstripped — not assumed.

**One defect this lane introduced is worth recording, because the corpus could
not see it.** A homonym rename in `v2.lens.module_graph` left
`dependency_resolution_facts_live` passing the *import* producer into both arms
of its two-source union (review 50749). The two producers share a signature, so
the compile stayed green and the function kept returning well-formed edges — it
simply stopped carrying every dependency that only a bare reference establishes,
which is precisely the edge class this whole lane is about. Nothing executed that
seam. It does now: `test.claim.module_graph_edge_source_witness` points the
production function at a four-module fixture and asserts the union at identity
grain in both directions, and restoring the doubled arm reds two of its five
witnesses. The fixture's shape is derived from the seam rather than invented —
`reference_resolution_facts` emits nothing for a file that still carries imports,
so the two producers are disjoint by file, the reference-only dependency must
live in an import-less consumer, and a same-path duplicate is unconstructible
through this seam; the witness says so rather than asserting a dedup that cannot
arise.

**This does not close import deletion, and nothing here should be read as
progress toward it.** The remaining residual is not a naming tail. It is one
integration vertical:

> the ordinary compiler must produce accepted occurrence bindings, project
> provider-file dependencies from them, and load the same closure independently
> of ambient pool membership.

What the close does buy is that **no one can point at a miscellaneous naming
tail to justify a workaround**. The corpus excuse is gone; §15.0's maturity
matrix is what remains, with the permanent closing contract still reporting all
six capabilities unavailable.
### 15.8 One ordering property of the fork half

Consolidating a fork today requires **adding** an import to the module that
loses its local copy — the very edge this lane deletes. That is an ordering
artifact, since a corpus-unique name resolves bare once the ordinary-path
changeover lands, but it means fork consolidation must not be scored as import
growth. The same caveat applies to #8058 and #8062, which correctly add explicit
imports to close fail-open pool reliance under *today's* compiler: those are
interim containment and need a dissolution trigger tied to the production
changeover, or the temporary mechanism hardens into the concept this lane exists
to delete.
