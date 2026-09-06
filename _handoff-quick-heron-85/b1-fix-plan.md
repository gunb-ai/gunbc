# B1: remove the order-dependent callable owner index (plan, pre-rebase)

## Established by execution (order probe, 2026-09-06)

Subject: `compile --source-root dag` (the REDUCED subject — the full-corpus arm
overflowed the stack and formed NO PROFILE, so it is an absent observation, not a zero).
Figures below are observations of THAT subject and must travel with it.

| arm | blocking | advisory |
|---|---|---|
| main forward | 7376 | 24245 |
| main reversed | 7376 | 24245 |
| cut forward | 8157 | 60284 |
| cut reversed | 36172 | 3293 |

Main is order-INVARIANT; the cut is not. The reversed cut arm is dominated by
~11,000 `missing parent environment for imported module` against ~30 forward — a
RESOLUTION difference by class composition, which rules out the pre-registered
third outcome (arms differing for reasons unrelated to candidate resolution).

Consequence, pre-registered: the constant-index repair's controls are RECEIPTS,
not regression controls. "It never lands in this shape" has a measurement behind it.

## THE PROBE'S METHOD — RECORD IT, THE FIGURES ARE MEANINGLESS WITHOUT IT

The order probe is a ONE-LINE SOURCE PATCH, and WHICH LINE decides everything:

    src/v1/stage0/src/v1_compiler_infer.rs, the realize fold
      let state = graph.modules.clone().iter().cloned().fold(   <- forward arm
      let state = graph.modules.clone().iter().rev().cloned().fold(   <- reversed arm

Rebuild `cargo build --release -p v1-compiler --bin gunbc` per arm, then
`compile --source-root dag --output-dir <dir>` and count `^error[` / `^advisory[`.

THERE ARE TWO MODULE-ITERATION SITES AND ONLY ONE OF THEM IS THE SUBJECT. The other
is `closure_modules` in `cli_run.rs` (`graph.modules.iter().cloned().collect()`,
feeding `module_schedule_batches`). Both sites reach code that extends
`callable_owner_index`, so the wrong one looks entirely plausible.

MEASURED: reversing the `cli_run.rs` site changes NOTHING. On the pre-merge cut
head 44680db40e, where the realize-fold probe swings 8157 -> 36172, the cli_run
probe gives 8157 -> 8157. The Kahn topological sort downstream normalises the input
order away. So that arm is an INERT PROBE, and an inert probe and a repaired defect
produce the SAME OBSERVATION -- identical arms.

I ran the wrong site first, got identical arms on both main and the cut, and nearly
reported that the order dependence had disappeared across a merge that cannot have
caused it. The thing that caught it was not care in the moment: it was that
IDENTICAL is what an inert probe says, so the result carried no information until
the instrument was shown able to produce the other answer. Keep the pre-merge
control arm for exactly this reason.

## The site

`typecheck_with_census_extra` folds `graph.modules` through `realize_module`
carrying `RealizeState.callable_owner_index`, extended per realized module via
`callable_owner_index_from_own_env(env: typed.func_env)`. A module therefore sees
only the owners realized BEFORE it — a complete population consulted through a
partially-filled accumulator. That is the whole defect.

## Ruled shape

- build the index ONCE from `graph.modules` before the fold
- `callable_owner_index` leaves `RealizeState` ENTIRELY, passed as a parameter
- the incremental extend is DELETED, not left dormant

## The obstacle, and it is real

`CallableOwnerRow.sig` is a `ResolvedFuncSig` and IS consumed —
`reference_derived_parent_envs` feeds it through `supply_admit_row` into the
supplied parent env's `local` map, so callers receive real signatures. The index
cannot be built from declared NAMES alone.

So the lift is: run declared-sig collection + `resolve_func_sigs` for every module
up front with `parent_envs: []`, and build the index from those. Note
`extend_callable_owner_index` ALREADY passes `parents: []`, so only `env.local`
matters, and `local` is the module's own declarations. Do NOT pre-typecheck.

THE RISK I FIRST WROTE HERE IS DISCHARGED BY READING, NOT BY MEASURING.
`parent_envs` is threaded through `topo_resolve_loop` and lands ONLY in the
returned `ResolvedFuncEnv { parents: parent_envs }`. `all_resolved` — the `local`
map — is computed from `declared_sigs`, `call_edges` and `local_func_set`, every
one of them module-local. So `env.local` is INDEPENDENT of `parent_envs` by
construction, and the up-front lift with `parent_envs: []` yields identical rows.

THE RISK THAT ACTUALLY REMAINS IS ONE HOP LATER, and it is the question to answer
before writing the fix. The extend site consumes `typed.func_env`, i.e. the env
AFTER typechecking, and typechecking DOES rewrite `local`:
`populate_output_provenance` returns an env with an `updated_local`, and
`bind_local_func_conformance` rebinds too. So a pre-fold index carries
resolve-time sigs while today's carries post-typecheck sigs.

ANSWERED, BY READING, 2026-09-06 -- AND THE ANSWER IS THAT THE NAIVE LIFT BREAKS.

`declared_to_resolved` stamps every resolve-time sig with
`resolved_formals: LocalFormalsAwaitingModuleContext {}` -- the substrate names
the not-yet-bound state explicitly. `bind_local_func_conformance` is what replaces
it with `DeclarationBoundFormals`, and it runs inside `typecheck_module` against
that module's `TypeEnv`.

So an index built pre-fold from raw `resolve_func_sigs` output carries
`LocalFormalsAwaitingModuleContext` on every row. The landed repair (#10656) then
reads that arm in `formal_authority_available` as **false**, with the located
reason "local declaration has not passed through build_module_context". Every
cross-module callee would REFUSE.

That is the correct failure mode -- loud, located, no silent widening, which is
the repair doing its job -- but it means the lift is not free and the ruled shape
"build it once from graph.modules before the fold" cannot be taken literally.

WHY IT IS NOT A ONE-LINE MOVE: `bind_local_func_conformance` calls
`peel_nominal_alias_identity` and `declaration_substitution_basis` against the
module's `TypeEnv`, so it resolves type aliases that may be declared in OTHER
modules. The binding therefore needs cross-module type resolution to have already
happened -- it is not module-local the way sig resolution is.

RECOMMENDED SHAPE (needs a ruling before it is built): split realize into a SIG
PHASE and a BODY PHASE. Phase 1 resolves types and binds sigs for every module and
builds the complete index; phase 2 folds bodies against that finished index. This
is the manager's "lift the per-module func_sigs collection ahead rather than
pre-typechecking" made concrete, plus the conformance binding that the reading
shows must travel with it. It is a larger restructure than the ruled sentence
implies, which is why it is written down rather than improvised.

SUPERSEDED QUESTION (kept so the reasoning is re-interrogable): does any consumer of the supplied parent env depend on the
provenance/conformance enrichment in those sigs? The rows reach callers through
`supply_admit_row` into `ResolvedFuncEnv.local`, so the question is answerable by
reading what reads a supplied `local` sig. If it does depend, the lift is not
free and the fix needs a different join — do not assume it away.

## Oracle (ruled)

Two-directional, at IDENTITY grain, with the expected set derived INDEPENDENTLY
from the module tree — never from either index. Plus the baseline control arm on
main, which is what made the probe above attributable.

## Sequencing

Rebase onto the fail-open repair (#10656) FIRST — both touch `src/v1/04_sigs.dag`.


## Phase 1a's ordering authority is INTERIM — cite the roster, do not re-declare it

Phase 1a orders modules topologically over IMPORTS. This program deletes imports,
so that authority is what STEP 5 removes.

THE REMOVAL IS ALREADY ROSTERED AND THE ROSTER IS THE CITATION. `gunbc.namespace`
`namespace_cut_subject_roster` carries the subject **ImportDerivedModuleAdjacency**,
probing `v1.compiler.resolve.topological_sort` and `build_import_adjacency`
(cli_run/entry_resolve.rs). The import-derived ORDERING is therefore already a
named namespace-cut subject with an authority older than anything this plan
declares.

An earlier revision of this file declared the interim status with a
self-authored capability trigger. THAT PARAGRAPH IS DELETED RATHER THAN KEPT
ALONGSIDE: it was a second name for a fact the roster already owns, and two
authorities answering for one fact is the thing this cut exists to remove. The
successor ordering authority is whatever discharges ImportDerivedModuleAdjacency
— cite the subject, not a paraphrase of it.


## The parent-callable claim, NARROWED — the broad form is FALSE

I wrote that after B1 there is "NO read of a parent's func_env anywhere". THAT
SENTENCE IS FALSE and must not appear in the PR body or any durable record. The
census behind it was `grep` over `src/v1/04_infer.dag` ALONE, and I reported a
one-file result as a whole-tree fact — the second time today a .dag-scoped census
missed a consumer living in the handwritten seed.

VERIFIED BY DIRECT INSPECTION of src/v1/stage0/src/cli_run.rs, not by search:

  39523  struct ScopeOrderIndex
  39543  m.func_env.name              build_scope_order_index, over every completed module
  39924  entry_module.func_env.name   claim-scope assembly
  39927  entry_module.func_env.parents  walked as closure evidence

That is real semantic consumption: executing a completed module's body requires
its imported closure to accompany it, and claim-scope assembly treats the derived
`ResolvedFuncEnv.parents` as that evidence.

THE NARROW CLAIM, which survives all three roots (.dag, generated stage0, and the
handwritten host):

  After B1, NO CHILD CALLABLE-CONSTRUCTION PATH consumes a parent
  TypedModule.func_env. Cross-module callable supply is CallableOwnerIndex,
  populated only from each module's OWN local signatures.
  ResolvedFuncEnv.parents remains a derived PER-MODULE OUTPUT and HAS downstream
  consumers, including handwritten claim-scope assembly.

WHY THIS DOES NOT REINSTATE A TRANSITIONAL Map<String, ResolvedFuncEnv>: the edge
direction. Claim-scope consumption is DOWNSTREAM —
CallableOwnerIndex -> child 1b -> child ResolvedFuncEnv -> final TypedModule ->
claim-scope reads that module's derived parents. The edge that would justify such
a carrier is the opposite one, parent TypedModule.func_env -> child 1b, and it
does not exist: zero `typed_parent.func_env` / `parent.func_env` in source and in
the generated mirror, and build_module_context takes its callable parent
population from `reference_derived_parent_envs` over `callable_owner_index`
rather than walking parent_index. RULED: do not introduce that map.

## 1c must DELETE the incremental publication, not retype it

The handwritten host still grows `callable_owner_index` at the SAME SITE that
completes and inserts each typed module — the traversal-prefix realization this
split exists to remove. When 1c becomes whole-population-before-bodies, that
incremental publication must DISAPPEAR, not switch payload types. A retyped
incremental publication is the same scheduling regime wearing the new carrier,
and it makes "complete index before any bodies" FALSE while looking migrated.
State it in the 1c commit as the thing being deleted.

## Method note, twice-earned

GitHub code search returned ZERO results for `collect_parent_envs` in cli_run.rs
— a call that is demonstrably there. Code-search absence is not evidence of
absence, and this file specifically has now hidden a consumer twice in one day.
Inspect the handwritten blob DIRECTLY whenever a claim depends on what it does or
does not contain.


## The evidence posture of this branch — state it in three parts, never one

For the 1a divergence RED, and for every RED this lane enrolls, the honest
sentence has THREE separate facts and none may do another's work:

  1. EXECUTED — locally, by mutation, with the result stated (green intact; red
     with `left: 1, right: 0` when interface_index_of is sourced from body grain).
  2. ENROLLED AND MERGE-GATING on the required floor lane — demonstrated by
     `judged=1` for v2.test.claim.infer_semantics_witness on GREEN MAIN run
     34047693380.
  3. NOT JUDGED IN CI ON THIS BRANCH — the floor lane refuses at
     generated-artifact before reaching judgement, and will for the whole life of
     this PR while the cut's standing reds sit upstream.

"Merge-gating" is true of the CHECK and false of THIS PR, and a reader will not
draw that distinction unless it is drawn for them. Collapsing the three is how a
witness gets cited later as coverage it never provided on the branch that needed
it.

### Consequence for the cut's endgame

EVERY red this lane enrolls has property 3. While the branch cannot reach
judgement, none of its discriminating evidence executes in CI, so the cut
accumulates witnesses whose FIRST REAL CI JUDGEMENT happens at cutover — the
moment with the least appetite for a surprise. That is an argument for working
the census DOWN rather than carrying it, and it belongs in the plan rather than
being rediscovered at the end.


## Witness requirements while the floor cannot reach judgement

PRECEDENT (gunbc #10671, ae566ad0): a central witness was found DETERMINISTICALLY
FALSE and could not even compile -- it constructed a sole_constructor type outside
its defining module -- yet read as enrolled and passed review. "The floor had died
earlier on generated-artifact drift, so nothing had contradicted it." Its parent
commit: "The modules typechecked and executed nothing ... an approval reports no
blocking defect, it does not establish that evidence exists."

EVERY CONDITION HOLDS ON THIS BRANCH. The floor refuses at generated-artifact
before judgement, so nothing here can contradict an enrolled witness. In that
state a witness can be false, can fail to compile, and still read as enrolled.

So for EVERY red this lane enrols:

  1. AN EXECUTED MUTATION RESULT, STATED. Green alone is not evidence. No
     mutation result, it does not count as evidence in the PR body.
  2. DEMONSTRATE IT COMPILES AND RUNS. "Enrolled" has been shown to cover a dead
     check as well as a live one.
  3. SAY WHICH STATE IT IS IN, separately from enrollment, using the three-part
     sentence.

NOTE THE MUTATION DISCHARGES 1 AND 2 TOGETHER. Under mutation the harness printed
MY probe's own assertion text (`left: 1, right: 0` with the reason string). A
probe that goes red under mutation has thereby PROVED IT EXECUTES -- the output
came from inside it. Order-in-the-array plus a later failure is only inference;
the mutation is direct evidence. Prefer the mutation as the execution proof.


## The phase split stays on the cut branch — admission test for any main extraction

RULED: do not rebase or cherry-pick the phase split to main.

DISQUALIFYING FACT: `overlay_direct_import_exports` is explicitly on the namespace
cut's STEP-1 DELETION LIST, alongside `interface_env_for_import` and
`build_imported_variants`. 1a changes that function's signature and accessor and
edits import-driven portions of both type-env builders. So the diff is partly an
IMPROVEMENT TO MACHINERY ALREADY SCHEDULED TO DIE, and while X stands it is an
attractor — landing it on main would pay optimization into X and make it a better
attractor on the way out.

Corroborated by this branch's own text: the divergence RED takes
`interface_index_of` as its subject, and that fold is declared transitional with
the witness required to MOVE when it goes. A unit whose own author says its
production locus is temporary is not a unit to plant on main.

ADMISSION TEST for any future proposal to extract a slice to main:
  1. The exact production SYMBOLS survive Y — not merely the concept. On the
     deletion denominator means cut-side by default.
  2. There is a CURRENT terminal consumer on main. "1c will use this later" does
     not qualify.
  3. The slice does not add to, improve, or make more composable any X authority
     — especially visibility, import or reachability machinery.
  4. Removing the import cut from the roadmap would not change whether this exact
     diff should land now.

1a fails 1 and 2. 1b fails 2 — its named boundary exists so a FUTURE 1c can call
it over the whole population, which on main today is preparation for a consumer
that has not landed: the scaffold shape.

## The evidence gap is an evidence-topology problem, not a migration-topology one

Moving production code to get greener CI would treat the wrong axis. The
sanctioned second mode is a STOPPED-LINE AUDIT (DESIGN section 5): it replays to
ledger deficits, it REPORTS, IT DOES NOT GREEN. Execute the exact head's enrolled
changed witnesses independently of the known whole-floor blockers; compile and run
the ACTUAL consumer rather than a substitute compiler or an array-position
argument; emit identity-grain prepared/executed/judged/result receipts; never
green the cut, never authorize merge. The ordinary required floor stays the only
cutover acceptance.

HONEST STANDING, no axis borrowed from another:
  discriminating by mutation      ESTABLISHED
  actual consumer execution       ESTABLISHED (in audit, once it exists)
  enrollment                      ESTABLISHED
  required-floor judgement        DEFERRED — the stopped whole-cut subject
                                  refuses earlier
The audit is NOT rung-2 enforcement: mechanically preventable requires the
blocking mechanism to execute on the acceptance path, and the audit deliberately
is not that.

## Correction: 1c's ordering is ALREADY reference-derived on this branch

My earlier note said "1a orders topologically over IMPORTS, and this program
deletes imports". That is wrong for the cut branch, and the distinction is
load-bearing for how 1c's pre-pass is described.

Two distinct axes, verified by reading, not grep-inference:

- TRAVERSAL ORDER of `realize_module`'s dependency recursion (04_infer.dag:13789)
  is `reference_provider_module_paths` (declared 04_sigs.dag:343) — REFERENCE-derived
  already, i.e. the successor ordering authority `ImportDerivedModuleAdjacency`
  names is what the recursion in fact uses on this branch.
- PARENT EDGES consulted by the type layer are still `module.resolved_imports`
  (04_infer.dag:12453, 12460, 12515, 12545, 12557, 12618) — IMPORT-derived.

So the cut has already moved ordering off imports and has NOT moved the type
layer's parent edges off them. 1c's pre-pass therefore inherits a reference-derived
order for free and owes no new ordering device; what it must NOT do is quietly
present the type layer's surviving import-derived parent edges as if the cut had
converted them. That conversion is a later step's subject, not 1c's.
