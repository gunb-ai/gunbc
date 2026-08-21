# Self-host cargo refusals: the root partition (shared coordination surface)

**Purpose.** One stable place for the sessions working the v2 self-host wall to agree on
what the wall IS, which root each defect belongs to, and who owns which root. Operator-directed
2026-08-16. This document is coordination state, not an authority: every claim here names how it
was measured, and an unmeasured claim says so.

**Sessions sharing this surface:** `smart-ram-730` (self-host frontier / root partition) ·
`gentle-dove-833` (interpreter cut Y1 — emit `v2.compiler.eval`).

---

## 1. The milestone was mis-stated, and this is the correction

"1/27 self-emitted" was read for months as *get a compiler module to emit Rust*.
**Modules already emit Rust.** Twenty of twenty-one emit between 24 and 133 files. They then
fail `cargo build`.

The milestone is: **emitted Rust that compiles.**

Three separate places still say otherwise and are known-stale:

- `v2.compiler.self_host.frontier_band_a_emit_readiness` `compiler_frontier_band_a_emit_readiness_note`
  — routes the blocker to `parse_grammar_choice_overlap_residue` and thence to the namespace
  import-grammar cut. Predates gunbc#8265, which peels overlap-residue heads off assemble
  receipts, so the reason it names is a mask.
- the `compiler_frontier_roster` rows — several say these modules stop at `ProbeStageAssemble`.
  The banked receipt says they emit sixty files. Both cannot be true. **Being deleted**
  (operator ruling 2026-08-16, §5 below).
- `docs/plans/v2-self-hosting.md` — "0/27 self-host-green" is still true, but its framing
  invites the emission reading.

**A wrong root to avoid.** `v2.compiler.infer` `node_grounding_frontier_note` says v2 derives
only 3 of 12 node kinds; the other nine carry `infer_grounding_not_derived`. True, and NOT a
blocker — `GroundingNotDerived` sits on the **Accepted** path (DESIGN §4b names it a live
specimen of `FrontierAccepted`, "the typed-located-counted diagnostic whose phase result is
still `Accepted`"). smart-ram-730 read that count as a refusal and had to withdraw it.
Likewise `v2_emitter_direct_rust_door_contract` IS red, but it refuses on **source fidelity**
against a canned string with ~17 hand-authored groundings — a fixture-exactness check, not the
emission path. Do not generalize from either.

## 2. The measurement everything here rests on

The curated cargo sweep TSV — banked `941e8034862`, **2026-07-26**. **The TSV itself was DELETED 2026-08-16** (operator: delete anything not actively derived — a dated snapshot nothing regenerates is the same attractor as the frontier roster). The numbers reproduced in this document are the surviving record of it; recover the file from git history if a re-read is needed, and prefer a fresh run.
Produced by `docs/probes/curated_cargo_probe_one.sh` (emit → `cssl_assemble` → cargo).

Caveats the receipt carries, not to be rounded off:

- **Three weeks stale.** Treat as shape, not current counts. Refresh before planning against numbers.
- **`first_error` is `UNRESOLVED_CompilerError` on 20 of 21 rows.** Only the residual histogram
  carries signal. "E0308 dominant" is honest; "first error is E0308" is not.
- `01_tokenize` is the sole row whose first error the classifier coded:
  `error[E0432]: unresolved import crate::std_nat` → `CONFIRMED-namespace`.

**Do NOT route refusal readings through `frontier_probe_survey`.** It has produced zero receipts
on any host since at least 2026-08-06 — six kills in the banked receipt, plus a silent 27-minute
death on a dedicated BuildBuddy runner and a kernel OOM on 2026-08-16. The in-tree note blames
shared-host memory pressure; reproduction on a dedicated runner refutes that. It is scaffolding
with its own deletion trigger; do not repair it. `curated_cargo_probe_one.sh` is the working tool.

**Instrument-vintage trap.** `build.rs` deliberately omits `cargo:rerun-if-changed` on
`.git/HEAD`, so an embedded commit stamp AGES until someone touches `build.rs`. The pin refusal
is the only thing between that and surveying today's tree with a month-old instrument.
`touch src/v1/stage0/build.rs` refreshes it.

## 3. The census

9,444 error instances, 20 modules, 24 distinct rustc codes. Top three are 75%.

| code | instances | modules |
|---|---:|---:|
| E0308 mismatched types | 3260 | 20/20 |
| E0277 trait bound | 1947 | 20/20 |
| E0599 no method | 1912 | 20/20 |
| E0369 binary op unsupported | 671 | 20/20 |
| E0107 generic arg count | 403 | 20/20 |
| E0063 missing struct field | 337 | 19 |
| E0282 type annotations needed | 216 | 20 |
| E0597 borrow lifetime | 202 | 19 |
| E0614 cannot deref | 167 | 18 |
| E0609 no field | 125 | 18 |
| E0061 44 · E0392 40 · E0560 23 · E0631 18 · E0004 18 · E0433 16 · E0310 14 · E0425 12 · E0615/E0573/E0271/E0223 4 · E0533 2 · E0432 1 | | |

Per module, total then histogram:

```
03_ingest              1053  E0308:432 E0277:193 E0599:125 E0369:90 E0107:58 E0614:32 E0597:25 E0282:23 E0063:22 ...
00_compile             1052  E0308:432 E0277:193 E0599:125 E0369:90 E0107:58 E0614:31 ...
source_authority        861  E0308:377 E0277:178 E0599:112 E0369:75 E0107:35 ...
emit_host               621  E0308:202 E0277:129 E0599:96  E0369:79 E0107:31 ...
02_parse                595  E0308:178 E0277:162 E0599:93  E0369:73 E0107:25 ...
emit_produced           435  E0308:141 E0599:96  E0277:90  E0614:21 ...
03_normalize            419  E0308:200 E0599:88  E0277:65  ...
05_eval                 410  E0308:117 E0599:91  E0277:81  E0369:29 E0614:22 ...
program_partition       387  E0308:116 E0599:104 E0277:76  ...
05_emit_orchestration   381  E0308:119 E0599:93  E0277:81  ...
emit_module             372  E0308:122 E0599:93  E0277:76  ...
emit_semantic_decl      366  E0308:115 E0599:93  E0277:76  ...
06_translate            364  E0308:114 E0599:93  E0277:76 E0063:18 E0369:14 E0597:13 E0107:13 E0282:9 E0609:6 E0614:3 E0392:2 E0560:1 E0061:1 E0004:1
05_emit                 364  E0308:114 E0599:93  E0277:76 E0063:18 E0369:14 E0597:13 E0107:13 E0282:9 E0609:6 E0614:3 E0392:2 E0560:1 E0061:1 E0004:1
04_infer                327  E0308:91  E0599:88  E0277:75  ...
materialization_carrier 324  E0277:90  E0599:81  E0308:77  E0369:60 E0107:13 E0061:2 E0282:1
03_name_resolve         315  E0599:89  E0308:89  E0277:65  ...
03_resolve              305  E0599:88  E0308:84  E0277:65 E0063:18 E0369:13 E0107:13 E0282:9 E0609:6 E0614:3 E0597:2 E0392:2 E0061:1 E0004:1
fold_lowering           291  E0599:82  E0308:78  E0277:65 E0063:18 E0369:13 E0107:13 E0282:9 E0609:5 E0614:3 E0392:2 E0597:1 E0061:1 E0004:1
01_tokenize             202  E0599:89  E0308:62  E0277:35  E0369:5 E0107:5 E0597:2 E0282:2 E0432:1 E0063:1
program_assembly          0  emit_fail — the one module that does not reach cargo
```

## 4. Hypotheses, each with what would falsify it

**These are hypotheses. None is confirmed. Do not plan against them as findings.**

**H1 — there is a shared floor, and it is most of the volume.**
`05_emit` is 35 source lines; `06_translate` is 4,226. Both total **364**, with identical
histograms code-for-code. `fold_lowering` (164 lines) is 291 with the same tail. Reading: the
shared emitted closure fails the same way in every crate, ~290 deep, and each module adds a
delta. If true, this is one core plus twenty deltas — the 9,444 is the same defects counted
twenty times, and a core fix drops every module at once.
*Falsified by:* extracting real error TEXTS for two modules and finding the intersection small.
Histogram similarity is suggestive, not proof. **This is the next measurement to run.**

**H2 — E0308 + E0277 + E0599 are one root, not three.**
That triple is the signature of a type-representation fork, which DESIGN already tracks: every
primitive modeled as a coproduct, realized as a native `Value`, reconciled by per-site bridges,
"so coverage is accidental and non-compositional." A modeled `Nat` landing where a native i64 is
expected yields a mismatch at the value, a missing method on the wrong carrier, and a missing
impl for that form.
*Falsified by:* error texts showing the three codes citing disjoint type pairs, or E0599s whose
receivers are unrelated to any coproduct/native straddle.
*Risk being managed:* three roots must not be merged merely because they correlate.

**H3 — `01_tokenize` is unrepresentative.**
It sits BELOW the floor (202) with a different profile — no E0063:18, no E0614, no E0392,
no E0004; E0369/E0107 at 5 rather than 13. Smaller closure, not just a smaller module. It was
smart-ram-730's first pick on the strength of being the only coded first error; that pick is
**withdrawn pending H1**, because fixing an outlier may teach nothing about the other nineteen.

## 5. Frontier roster — DELETED (operator ruling, 2026-08-16)

> "I would not have any dual authority rows in the frontier — either make it derived from live
> state, or non-existent asap — otherwise people get too enamored with it." … "let's delete it,
> it's confusing and unhelpful, which is negative value."

Rationale, in the roster's own terms: `execution_measured_seed_retained_row` takes
`measured_blocker`, `located_stage` and `located_reason` as **ordinary parameters**, so a row
that claims measurement is structurally indistinguishable from one that asserts it — the
constructor NAME asserts a provenance the TYPE does not carry
(`frontier_roster_provenance_constructor_inflation_note` says so in tree). Ten of twenty-seven
were never execution-measured at any head. It is an attractor: three separate readers took its
rows as measurements this week, and one census was withdrawn over it.

Deleted rather than derived: everything worth keeping is already derivable elsewhere — the
module list from the filesystem, composition from source in one command, cargo status from the
sweep receipt. The disposition/blocker/stage fields are the part that cannot be derived and were
the part that lied. Per DESIGN §3 delete-first, **the deletion is the census**: real consumers
refuse loudly. Expected load-bearing consumer: the crate-layout emitter
(`compiler_frontier_crate_layout_note`).

## 6. Division of labour

`05_eval` totals 410, of which ~290 looks like shared floor under H1. So the child's lane is
currently blocked behind defects that are not eval's and cannot be fixed from inside eval.
Consequence: **neither session fixes this per-module.** Root ownership is assigned here once the
partition is confirmed, so the two lanes work disjoint roots rather than the same wall twice.

| root | July size | owner | status |
|---|---:|---|---|
| **A — generic trait bounds not emitted** | ~590 | `smart-ram-730` | ROOT-CAUSED, fix in progress |
| **B — primitive representation fork** | ~196 | new child | dispatched 2026-08-16 |
| **C — Optional collapses to `()` nested** | ~169 | `gentle-dove-833` | first fix refuted; upstream seam next |
| **D — generic argument count** | ~76 | new child | dispatched 2026-08-16 |
| **E — unreachable patterns** | 126 | — | lint-class, lowest value |
| **F — E0282 type annotations needed** | 55 | — | do NOT start; likely dissolves with A |
| **the unclassified ~64%** | — | new child | dispatched 2026-08-16 |

### Root A — SIZING RETRACTED, and it is at least two roots (smart-ram-730, 2026-08-16)

**The ~590 was arithmetic coincidence across two censuses, not a measured root.** Raised by the
side channel, verified in tree against `docs/probes/e0599_phase_b0_emitter_decision_2026-07-29.md`.
The five signatures I summed came partly from the July E0599 diagnosis (whose own 590 was
*590 of 635 E0599 diagnostics*, including further `Outcome<U>`/`Option<T>`/`Vector<T>` shapes I
did not list) and partly from the separately-counted 181 E0277 `T: Clone`. Adding them was
double-counting across two populations that overlap.

**Worse for my framing: the E0599 population was already split by emitter decision, three ways:**

| mechanism | occurrences | what produced it |
|---|---:|---|
| `CloneSharedRequirement` | **369 (61.5%)** | the seed emitter INSERTED a clone at a sharing seam |
| `TargetApiRequirement` | 168 (28.0%) | `is_empty`/`iter` require element `Clone` |
| `OwnedDeconstructionRequirement` | 63 (10.5%) | the emitter's owned head/deconstruction lowering |

So "Root A" as this document defined it spans **two different lanes**. The derive-macro
supplemental-bound mechanism I root-caused below maps to the `TargetApiRequirement` population
(168) — container methods needing an element bound. The larger `CloneSharedRequirement` half is
clones the emitter *synthesised*, which belongs to the **emitter-ownership-defork** thread
DESIGN already carries, not to derive bounds.

**Two warnings that census attaches to its own numbers, and this document must not strip:**
369 is **not** a predicted burn-down — the census records which emitter arm is *associated* with
a site, and does NOT execute the per-site ownership verdict, so removability is unproven; and
`CloneSharedRequirement` means only *the emitter inserted a clone here*, never *this clone can
be deleted*. A withdrawn rationale in the same document is worth carrying: `Rc::make_mut` was
once cited as evidence a copy-free alternative exists, and it is not — it is declared
`where T: Clone` and clones the pointee whenever another strong `Rc` is live, so it **requires
the very bound being counted**.

**What survives:** the mechanism below is real and root-caused. Its scope is the derive/container
population, not 590. What the live `05_emit` run measured (~105 in one module, 49 of them
"exists but its trait bounds were not satisfied") is the number to work against.

### Root A — MECHANISM, for the derive/container population only (smart-ram-730, 2026-08-16)

Live size, measured today on `05_emit` by `gentle-dove-833`: **~105** in one module
(24 `no method clone on type parameter` · 32 `bound …: Clone not satisfied` ·
49 `exists but its trait bounds were not satisfied`; the first two are E0599 subsets summing
under the E0599 total of 80, so read 105 as a tight upper bound and 73 as the E0599-only floor).
July was ~84/module, so **the family did not collapse** and gunbc#7691 did not reach these sites.

**Why it did not.** `v1.trait_derive_emit` `v1_clone_bound_seed_for_item` opens with
`if is_coproduct_type(n: item) { round }` — the derive trigger is **struct-only**. Its note
justifies that as "derive emits per-impl bounds," which is true for `derive(Clone)` on an enum
(that does add `T: Clone`) and **false for `Debug`/`Serialize`/`Deserialize`**, which add only
their own bound and never the supplemental `Clone` a container field's conditional impls need.
The second trigger cannot cover it either: well-formedness propagation fires only on naming a
type that already carries a declaration bound, and that same note records — checked against
`im-15.1.0` — that `im::Vector<A>` carries **no** declaration bound, so a container field is
"a derive-trigger fact only."

**So: a generic coproduct with a container field earns no Clone bound from either trigger.**
`Outcome<T>` is exactly that shape and is the receiver in the largest single signature.

This is also why an item-level bound structurally cannot close the population, however good its
fixpoint: the requirement is **per-derive-impl**, which is what v2's supplemental contracts
already encode with cited authorities. The fix is the wire-through, not a better v1 predicate.

## 9. Working agreements for this surface

**A ROOT SIZE MEASURED IN DIAGNOSTICS IS NOT A COUNT OF DEFECTS** (`gentle-dove-833`, 2026-08-16).
One emitter decision can produce several downstream rustc diagnostics, and the ratio is not 1 and
not constant. Measured in this lane: 159 collapse **events** removed 158 E0308 and 139 total errors
while **adding** 25 of a new class — the same events map to three different numbers depending on
which side you count from, and none of them is the number of things wrong. So: report the **event**
count with the instrument that produced it, and treat any diagnostic-denominated root size as an
upper bound on defects with an unmeasured fan-out. The corollary: **a fix can reduce the total while
revealing a class it had been masking**, so a shrinking total is not by itself evidence the root was
correctly identified.



Read before starting. These are all paid-for lessons from 2026-08-16.

1. **Nothing in this document is current unless it names today's measurement.** The July TSVs
   were DELETED (operator: delete anything not actively derived). The numbers transcribed here
   are a surviving record of a snapshot, which is the same staleness one level up — they are
   kept only because they are all we have, and every one of them is superseded the moment you
   measure live. Root A's ~590 was already wrong when this document quoted it.
2. **The instrument is `gunbc compile --entry <mod> --target rust` then `cargo check` on the
   emitted crate.** It produces error TEXTS. Do NOT use `frontier_probe_survey` — zero receipts
   on any host, including a dedicated runner.
3. **Own your root by generated FILE, not by entry module.** The floor files
   (`v2_std_algebra.rs`, `std_measure.rs`, `v2_std_compilers_target_model.rs`) sit in every
   closure. "05_eval's errors" is not a unit of work.
4. **The emitter is generated.** Authority is `src/v1/05_emit_rust.dag`; `v1_compiler_emit_rust.rs`
   is its output, and it is a member of the derived generated population (`gunbc.stage0_rust_source_lifecycle_scaffold` `derived_generated_stage0_repo_paths`) because the crate-layout authority does not claim it. Probe the `.rs` for fast
   feedback if you like, but revert it — real fixes land in the `.dag` and regenerate.
5. **Measure before/after in your own worktree.** Concurrent fixes to a shared tree make every
   before/after unattributable, which is the failure this whole document exists to prevent.
6. **Null-control your fix.** `gentle-dove-833` suppressed the Optional collapse, confirmed the
   arm was live (3 files differed), and got `|acc: Option<Absent>|` — `Absent` is not a Rust
   type. The rule had been HIDING a bad node, not creating it. A fix that changes one wrong
   output into a different wrong output is not a fix; check what your change actually produced.
7. **Prefer construction to validation.** If your root's bad node arrives already wrong, patching
   the consumer is validation. Find what produced it.

## 8. The partition, from cause signatures (this supersedes the by-error-code view)

**The canonical-seven cause-signature attribution TSV (2026-07-28)** answered the question §4 was still hypothesizing about. It groups by CAUSE SIGNATURE
rather than error code — E0308 by concrete expected/found pair, E0599 by receiver + missing
method, E0277 by unsatisfied trait + self type — over one clean build of the same assembled
crate. **233 signatures across 2,898 diagnostics.**

Found via the side channel; verified in tree before use. Its existence means **H2 is refuted**:
there is no single global top-three root. But grouping the signatures shows far fewer roots than
233, and the top one is not what the by-code view suggested.

**Root A — the emitter does not emit trait bounds on generic parameters (~590 as of 2026-07-28
— SIZE NOW SUSPECT, see below).** One root wearing five signatures:

```
206  E0599: no method `clone` found for type parameter `T`
181  E0277: bound `T: Clone` not satisfied
 84  E0599: `is_empty` exists for Rc<im::Vector<T>> but trait bounds not satisfied
 63  E0599: `clone` exists for Outcome<T> but trait bounds not satisfied
 56  E0599: `iter` exists for im::Vector<T> but trait bounds not satisfied
   + E0277 bounds for U/A/B: Clone, Node: Hash, Node: Eq, EnvironmentBindingKey: Hash
```

Emit `fn f<T>(…)` where the body clones a `T`, or hand a `T` to a container that requires
`Clone`, and rustc refuses at every use site. "exists but its trait bounds were not satisfied"
is rustc naming this directly. Mechanically uniform and closure-wide, so it is floor.

> **THE 590 IS STALE AND MUST NOT BE PLANNED AGAINST.** The cause-signature TSV is dated
> 2026-07-28. gunbc#7691 — *"Propagate item Clone bounds as a fixpoint over the declared-type
> graph"* — landed **2026-08-02**, five days later, and wires
> `emit_item_type_params_with_clone_bounds` + `emit_item_clone_bound_refusal` into
> `emit_type_def_from_connective`, which is exactly this root's site. So a fix targeting Root A
> landed *after* the measurement that sized it, and Root A's live size is **unknown**.
>
> Two further consequences. The July E0277 census's central claim — that
> `emit_type_def_from_connective` "renders generic params via the plain `emit_type_params` with
> no Clone-bound logic at all" — is **stale**: that path now branches into clone-bound emission
> when `capability_surface.impl_bodies == ""` and the item has no fn fields. And what #7691
> emits is a **struct-level** `T: Clone` (via `v1_emit_type_params_with_clone_bounds`), which is
> the shape review 43338 argued *against* as over-constraint — selective rather than blanket,
> since it is gated on `emit_info.clone_bounded_type_params`, but still type-level rather than
> per-derive-impl. Whether that is correct-enough in practice or is itself producing new
> failures is unmeasured.
>
> **Nothing proceeds on Root A until a live cause breakdown replaces the July numbers.** This is
> the receipt-staleness failure this document exists to stop, caught one step before it produced
> a fix aimed at a root that may already be substantially closed.

**The fork is already declared, and its author already named the hazard in the fix.**
`v1.trait_derive_emit` `trait_derive_emit_item_clone_bound_contract_fork_note` is a counted
model-realization fork stating: supplemental generic bounds are modeled at
`v2.std.compilers.target_model` `target_derive_supplemental_generic_bound_contract` with cited
upstream impl authorities and consumed by v2 translate; **"that carrier has no consumer in v1
seed emit today"**; and v1's structural rule is *"a separate interim authority … an
approximation of cited upstream requirements pending v2 emitter subsumption at that grain."*

Its dissolution clause is the wire-through this lane proposed — and it carries a warning worth
quoting exactly: *"dissolution re-grounds onto upstream impl requirements, **not a mechanical
lift of the same predicate — the two can disagree at the edges**."*

That disagreement has a name now. **v1's rule is item/type-level** (`T: Clone` onto the struct's
generic list); **v2's contracts are per-derive-impl** (Debug needs Clone, Serialize needs Clone,
Clone needs nothing extra). So the wire-through changes the **grain**, not merely the source of
truth, and the failure mode to guard is unioning the per-derive requirements back onto the whole
type declaration — which would reproduce v1's over-constraint while claiming v2's authority.
Any wire-through must keep per-derive bounds per-derive, and its discriminating control is a
type whose Debug impl needs `Clone` but whose construction does not: correct output bounds the
derive, not the type.

**No bootstrap cycle.** Checked rather than assumed: `src/v1/05_emit_rust.dag` already
references `v2.std.compilers.target_model` and `v2.extdeps.languages.rust`, and eight other v1
modules reference v2. The v1→v2 edge exists today, so consuming this authority adds no new
direction. (DESIGN §3: the only structural law on the import graph is acyclicity; the former
layer-direction rule was deleted 2026-07-24.)

**Ownership is assigned by generated file, not by entry module.** The floor files
(`v2_std_algebra.rs`, `std_measure.rs`, `v2_std_compilers_target_model.rs`) appear in every
module's closure, so "05_eval's errors" is not a meaningful unit of work — a shared-file failure
belongs to whoever owns that file's root, never to the lane that happened to compile it.

**Root B — primitive representation fork (~196).** DESIGN's open thread, now with counts:

```
60  expected `bool`  found `Bool`                              modeled Bool vs native bool
50  expected `Rc<Nat>` found `{integer}`                       modeled Nat vs native int
40  expected `Rc<CommutativeSemiring<Magnitude>>` found `{integer}`
33  expected `Rc<Rc<CommutativeSemiring<Magnitude>>>` found `{integer}`
10  expected `Rc<v2_std_nat::Nat>` found `{integer}`
 3  expected `bool` found `True`
```

> **ROOT-CAUSED AND MEASURED LIVE, 2026-08-16 (`eager-deer-389`). The table above is superseded
> as sizing; the mechanism below replaces it.** Full receipt with method and controls:
> [`docs/probes/root_b_primitive_repr_fork_2026-08-16.md`](../probes/root_b_primitive_repr_fork_2026-08-16.md),
> merged to main on gunbc#8337. It was named rather than linked while it lived only on that branch,
> because a link to a file the tree does not contain is a dangling edge the reachability lens
> refuses; now that it is on main the link is the correct form, and it is also what makes the probe
> reachable rather than an orphan.
>
> **The mechanism is one switch.** `v1.compiler.04_infer` `rust_corpus_repr` chooses
> `HostNative` vs `FaithfulFreeMonoid` from `corpus_has_v1_seed_source_indices`, a **path
> substring test** over the closure's source keys (`contains(k, "/v1/") || contains(k, "src/v1")`).
> `HostNative` grounds the numeric tower — `05_emit_rust` `rust_seed_host_numeric_alias` renders
> `Nat`/`Int` as `i64`, gated on `corpus_repr_is_host`. A seed closure contains `src/v1` paths; a
> pure-v2 closure does not. Same source file, emitted twice, executed today:
> seed `pub type Nat = i64;` versus v2 `pub type Nat = Rc<CommutativeSemiring<Magnitude>>;`.
>
> **The double-Rc row is NOT a second defect.** `pub type Nat = Rc<…>` and
> `pub type Int = GroupCompletion<Rc<Nat>>` — the alias carries the `Rc` and the use site wraps
> again. One alias hop, same fix; do not size it separately.
>
> **The `Bool` half will NOT dissolve with a repr fix — executed, not reasoned.** `Bool` is in the
> Rust checkpoint table (`dag/extdeps/languages/rust/types.dag`, `Bool` → `bool`) so references
> render native while `type Bool = True | False` emits an enum, and the host bridge is hardcoded:
> `std.trait_derive_shape` `repr_grounding_supplemental_bool_host_bridge_target` is
> `module_path == "std.types" && name == "Bool"`, while `src/v2/std/logic.dag` declares a **second**
> `Bool` that the predicate is pinned to reject — its own witness asserts that rejection as
> expected behaviour. Two authorities, one bridged: a §3 fork above the repr choice.
>
> **THE DISCRIMINATING EXPERIMENT, AND IT REFUTES THE OBVIOUS FIX.** `rust_corpus_repr` forced to
> `HostNative` in the generated seed only, rebuilt, re-probed, reverted; both instrument controls
> recorded (patched binary verified emitting `i64` *before* reading any number, restored binary
> verified emitting the modeled carrier again afterwards):
>
> | `src/v2/compiler/06_translate.dag` | baseline | forced |
> |---|---:|---:|
> | algebra-carrier errors / **distinct sites** | 84 / **74** | **0 / 0** |
> | `Measure<…>` errors / sites | 11 / 9 | 37 / 37 |
> | `Rc<i64>` errors / sites | 0 / 0 | 125 / 125 |
> | unresolved-name (E0425/E0433/E0422) sites | 19 | **110** |
> | `expected bool found Bool` | 11 | **11** |
> | total error blocks | 693 | **807** |
>
> **COUNT CORRECTION, mine (2026-08-16).** This table first read "342 diagnostics citing
> `CommutativeSemiring<Magnitude>`". That was a `grep -c` over matching LINES — it counted
> rustc's annotation and note lines as well as the error — and overstated by ~4x. Re-counted per
> error block and at distinct `file:line:col` grain, to match §11's denomination, it is **84
> errors / 74 sites → 0**. Conclusion and direction unchanged; the magnitude was wrong. Anyone
> reconciling this against §11's 509 sites should use 74, not 342.
>
> The cause is confirmed (342 → 0 on a real module). The `Bool` half is untouched. **And the total
> ROSE by 121** — working agreement 6 firing exactly as written. The increase is characterized:
> ~76 are E0308 the modeled carrier had been **masking** (`expected i64 found Rc<i64>`, 39, plus 37
> through `Measure` — `Nat` stays in `shared_types` and is still `Rc`-wrapped after becoming a
> `Copy` scalar), and ~87 are missing type names because
> `05_emit_rust` `reference_derived_use_lines_note` gates reference-derived use-line synthesis on
> `corpus_repr_is_faithful` and gives HostNative import-bearing modules `[]`.
>
> **THE ACTUAL FINDING: `RustCorpusRepr` fuses two independent facts** — how modeled primitives are
> realized, and whether namespace-derived use-lines are synthesized — **and a pure-v2 closure needs
> opposite arms of each.** No value of a two-valued enum supplies both, which is why the seed
> compiles, the v2 corpus refuses, and forcing either arm merely relocates the refusals. A §5
> state-space conflation sitting *underneath* Root B. No fix is proposed from this; splitting the
> enum is the shape the evidence points at, but which authority owns the split — and whether the
> numeric grounding belongs in the checkpoint table at all — is a modeling decision above this lane.
>
> **A 30-second reproducer, recommended over a compiler-module probe.** The three-file closure
> `gunbc compile --source-root dag --source-root src/v2 --entry dag/std/nat.dag --target rust`
> refuses with 4 E0369 on `Rc<CommutativeSemiring<Magnitude>>` and goes fully green under the
> forced switch.
>
> **Not claimed:** any corpus-wide Root B size (one module, measured twice); full attribution of
> every one of the 121; and the `emit_host` probe from the same batch is **discarded** — it
> overlapped a rebuild of the instrument, so its `emit_fail` is unattributable and contradicts the
> banked receipt that this module emits.

**Root C — Optional collapses to `()` (~169).** 134 `expected () found Option<_>` plus 35
`expected Rc<Correction> found Option<_>`. Mechanism established by gentle-dove-833 by
instrumenting the emitter (91 hits on `Absent`); see §4. Open question owned with the root:
the node is ALREADY named `Absent` at the emitter, so the mis-resolution is upstream and
patching `rust_type_arg_renders_as_unit` would be validation, not construction.

**Root D — generic argument count (~76).** `E0107: type alias takes 0 generic arguments but 1
was supplied` (48) and `missing generics for enum Witness` (28).

**Root E — unreachable patterns (126, 54 files).** A lint, not a type error. Lowest value.

**Root F — E0282 type annotations needed (55, 37 files).** May dissolve with A, since an
unbounded generic often also fails to infer. Do not work it before A lands.

A+B+C+D is ~1,031 of 2,898 — **36% in four roots**, and A alone is ~20%. Whatever remains after
those four is the tail worth re-censusing, not worth planning against now.

**Prior art not to re-derive.** `docs/probes/` already holds `e0599_diagnosis_2026-07-26.md`,
`e0599_phase_a_body_evidence_2026-07-28.md`, `e0599_phase_b0_emitter_decision_2026-07-29.md`,
`e0277_trait_bound_census_2026-07-26.md`, `e0369_census_2026-07-26.md`,
`gate1_repr_mismatch_e0308_diagnosis_2026-07-24.md`, and
`root4_measure_missing_generics_closure_2026-07-26.md`. Read the one for your root before
starting.

## 7. Open, and who is asked

- H1 confirmation by error-text intersection — smart-ram-730, next action.
- Refresh the three-week-old sweep with `curated_cargo_probe_one.sh` — smart-ram-730.
- Independent root partition requested from the linked side channel; its answer lands here, and
  its claims are evidence to check, not authority (it has already cited one symbol that does not
  exist in the tree).


## 10. CLOSURE-SHAPE FORKS — restated after its own falsifier fired (2026-08-16)

Two lanes independently hit the same underlying fact from different roots, and it may explain
why "the seed compiles and v2 modules do not" better than any per-root mechanism.

**Root B (eager-deer-389, root-caused with an executed receipt).** `v1.compiler.infer`
`rust_corpus_repr` picks the whole corpus's numeric representation from
`corpus_has_v1_seed_source_indices`, which is verbatim:

```
modules |> any(m => map_keys(m.type_env.source_indices)
                    |> any(k => contains(k, "/v1/") || contains(k, "src/v1")))
```

**A path substring decides the emitted representation of every primitive.** `HostNative` grounds
the numeric tower (`rust_seed_host_numeric_alias` renders `Nat`/`Int` as `i64`, gated on
`corpus_repr_is_host`); `FaithfulFreeMonoid` does not. A seed closure contains `src/v1` paths; a
pure-v2 closure does not. The receipt, one source file emitted twice:

```
seed  src/v1/stage0/src/std_nat.rs   pub type Nat = i64;
v2    emitted from dag/std/nat.dag   pub type Nat = Rc<CommutativeSemiring<Magnitude>>;
                                     → E0369 binary operation `<` cannot be applied
```

DESIGN §4 rules a heuristic never necessary in a closed system, and §3 rules that a fact's home
is its layer, not its file — "paths are discriminators, not gospel." This is a path literal
deciding semantics for the entire emitted corpus.

**Root A, same shape, found live by the same lane.** The first error in `06_translate` today is
no longer a rustc error at all — it is the emitter refusing:

> `trait_derive_emit: generic item 'Homomorphism' has a field applying type 'C', whose declared
> parameter list is not readable in this closure — the Clone bound it may require cannot be decided`

That is `v1_type_expr_clone_undecided_head`, the fail-closed arm of #7691, whose own note ends
**"Dead in corpus as of this landing; kept as the fail-closed arm."** It is not dead. It fires
first, and it fires because a declared parameter list is *not readable in this closure*.

**The convergence.** Both mechanisms branch on what the closure contains, and both take the
unfavourable branch exactly when the closure is pure-v2. So the wall is not only "the emitter
emits wrong Rust" — it is substantially **"the emitter has never been exercised on a seed-free
closure,"** and two independent places change behaviour silently when it is.

If that holds, several roots are downstream projections of one fact, and the per-root sizes are
measuring symptoms of it.

**THIRD INSTANCE — confirmed as a mechanism, and it FALSIFIED this section's directional
claim.** It has its own full entry below (`### §10 — the third branch`), written by
`gentle-dove-833` who measured it; not restated here, because two accounts of one finding is the
dual authority this surface exists to prevent and its own authors produced one within minutes.

**What the falsifier was, and that it fired.** This section named its falsifier as *a third
closure-shape branch that does NOT correlate*. `gentle-dove-833` measured exactly that: the seed
closure carries ambiguous names too — including the same `Absent` — and the emitter collapses
them by the same predicate, so there is **no favourable seed arm**. Both closures take the same
branch; the v2 closure merely has ambiguous names in far more type-argument positions. The
per-closure ambiguous sets and method are in their entry below; not copied here.

**So the meta-root is restated, narrower and better supported:** several independent decisions
are keyed off **under-modeled closure facts**. That is what the three instances share. What they
do NOT share — and what I asserted prematurely — is the direction: "the seed is the exercised
configuration and pure-v2 is the untested one" holds for Root B's path-substring switch, which
really is seed-versus-v2, and does not hold for this one.

**A correction to my own framing, from the same message.** I called this a DESIGN state-space
conflation and put that in this section under my name. Reading every consumer says otherwise:
the other six read sites **already guard the sentinel** with an explicit non-empty check. Only
the `map_contains_key` site does not, because that call cannot see the value. So it is **one
consumer diverging from a convention its six siblings already follow** — not a design-wide gap.
That makes the finding smaller and much more actionable: the minimal correct change restores the
authority's own convention rather than inventing a rule. The §2 residue is real and stays — a
guard hand-repeated at six sites is what let the seventh omit it, so the terminal shape is still
a coproduct with no constructor for the conflated read.

**Cancelled, with a receipt:** the three-run overlap partition between this instance and
`eager-deer-389`'s candidate. They retracted the shared-name premise — theirs is an `Option`'s
absent arm from `lookup_type_by_name`, not a corpus name, so the two never shared a subject and
there is no overlap to measure. Their inverted question survives and is cheap: instrument
`lookup_type_by_name` misses on `05_emit` and see whether `Absent` or `Optional` appear.


**Two further facts from the same run, recorded because they cut against tidy stories:**

- **The wall GREW.** `06_translate` measured 671 coded errors today against July's 364. No cause
  attributed, and the lane explicitly does not claim one.
- **Bool will not dissolve with a repr fix, because there are TWO Bool authorities.**
  `dag/extdeps/languages/rust/types.dag` maps `Bool` → native `bool`, so references render
  native while `type Bool = True | False` emits an enum. The host bridge
  (`std.trait_derive_shape` `repr_grounding_supplemental_bool_host_bridge_target`) is hardcoded
  to `module_path == "std.types" && name == "Bool"`, and `src/v2/std/logic.dag` declares a
  SECOND `type Bool = True | False` which that predicate is pinned to reject — its own witness
  asserts the std.logic case is false *as expected behaviour*. That is a §3 fork sitting ABOVE
  the representation choice and it needs its own row.

**Method note worth copying (eager-deer-389):** a three-file closure emitting `dag/std/nat.dag`
is a 30-second discriminating reproducer for the Root B family. Prefer it to a compiler-module
probe, which costs tens of minutes.

### §10 — the third branch, checked as asked (`eager-deer-389`, 2026-08-16)

Root B **is** instance 1: its whole mechanism is a closure-content branch.

**PROMOTED — EXECUTED 2026-08-16 (`eager-deer-389`). The candidate below FIRES, and what it fires
on reopens the `gentle-dove-833` overlap I had retracted.** The miss arm of
`type_leaf_is_unbound_in_closure_scope` was instrumented in the generated seed (an `eprintln` of
the looked-up name on the `None` branch), `05_emit.dag` emitted in a pure-v2 closure, patch
reverted. **34 firings:**

```
28  Absent
 4  TypeExprKindAuthorityInvalid
 2  fn
```

So the arm is live, not dead. And the NAME reaching `lookup_type_by_name` and missing is literally
`Absent`, 28 of 34 times.

**This vindicates `gentle-dove-833`'s hypothesis while leaving my mechanism retraction standing,
and the distinction matters.** Their reading of my sentence was wrong — that `Absent` is still an
`Option` arm in the emitter's control flow, not a name. But their *substantive* caution — that if
`Absent` reaches my predicate it may be a **variant name arriving at a type-name lookup**, making
the two of us consumers of one upstream defect — is now measured true, and they stated it before
either of us had evidence. I killed their three-run experiment on the mechanism when the question
was always about the name; that was my error twice over and the experiment is back on.

**Reading, not finding:** a variant name is being carried in a type position. `gentle-dove-833`'s
mechanism meets it in `derive_variant_to_enum` (two declarers ⇒ ambiguity sentinel ⇒ ignored by
`map_contains_key`); mine meets it in the type env (not a type ⇒ miss ⇒ defaults to unbound). If
that holds, **neither arm is the root** and both populations are partly downstream of whatever
puts a variant name in a type position.

**Cheaper next step than the three-run table:** dump the same miss list on `gentle-dove-833`'s
entry and compare it to their sentinel set. Same two names ⇒ one upstream defect and the runs then
measure double-counting. Disjoint ⇒ genuinely independent and the runs are unnecessary. The whole
answer is in emit stderr — no `cssl_assemble`, no cargo pass.

**Bearing on identity-keying:** this is a caution against over-scoping that fix. Resolved
declaration identity disambiguates two same-named *types*; it does not obviously repair a
*variant* name arriving where a type was expected. Those may be two different defects and the
identity fix should not be sized as if it covers both.

**A third candidate, same shape, NOT executed:** `05_emit_rust`
`type_leaf_is_unbound_in_closure_scope` returns `true` on the `Absent` **match arm of
`lookup_type_by_name`** (which returns `Node?`) — i.e. on a lookup MISS, nothing whatsoever to do
with a type or variant *named* `Absent`. **My original wording here read "returns `true` on
`Absent`" and that was ambiguous enough to be misread as the name; corrected 2026-08-16 after it
did exactly that.** A name missing from the
closure's type env is treated as unbound, which then drives spurious-generic suppression in fold
rendering. Narrower closure, silent defaulting arm. I have not shown it takes the *wrong* arm in a
pure-v2 closure, so it is a candidate and must not be counted as a confirmed instance. What would
promote it: render a fold over a closure that deliberately excludes the leaf's declaring module and
show the arm flip.

**One explicit non-instance, recorded because a negative costs the next reader the same search:**
`v1.compiler.04_env` `source_tree_of` branches on `src/v1`/`src/v2`/`dag/` path substrings, but its
own `source_tree_partition_note` records the 2026-07-11 ruling that tree no longer decides
refuse-vs-ledger and only *labels* a dissolution partition. Do not count it.

**Evidence from my lane that bears on the hypothesis, in both directions.** Supporting: forcing the
repr arm moved 342 diagnostics to zero, so a single closure-shape branch really does gate a large
population. Cutting against a tidy version of it: the *same* flip introduced ~87 new failures
through a **second, oppositely-directed** `corpus_repr` branch. So the meta-root is better stated as
*several independent decisions are keyed off one under-modeled closure fact and disagree about which
arm a pure-v2 closure wants* — not as one switch in the wrong position.

### §10 — the third branch, CONFIRMED by execution (`gentle-dove-833`, 2026-08-16)

**Instance 3, and it is not an `if` statement anyone wrote about closures — it is emergent from a
fold, which is why reading did not find it.**

`v1.compiler.emit_info` `derive_variant_to_enum` folds the **closure's** `type_summaries` into a
`variant name → owning enum` map. When two enums in that closure claim the same variant name it
inserts the empty string as an **ambiguity sentinel**:

```
Present { value: _ } => map_insert(inner, vn, "")          // claimed by 2+ enums → ambiguous
Absent               => map_insert(inner, vn, summary.name)
```

`v1.compiler.emit_rust` `is_value_variant_type_arg` then reads that map with `map_contains_key`,
**which ignores the value**. So an *ambiguously owned* name tests positive as a value-variant, and
`rust_type_arg_renders_as_unit` collapses the type argument to `()`.

Whether a name is ambiguous is a function of **which modules are in the closure**. Different
closure, different ambiguity set, different collapse decisions, silently. That is the branch.

**Measured, one entry (`src/v2/compiler/05_emit.dag`), live tree at `3473e57962`.** Instrumented at
the predicate's positive arm, reporting the owner it ignored:

```
name Absent           owner ""                    91 positives
name Optional         owner ""                    68 positives
name AlgebraPrimitive owner "CanonicalOperation"   28
name Time             owner "Quantity"             24
… every other row carries a REAL owner
```

The two dominant names are the **only** two carrying the sentinel. Confirmed declarers:
`Optional` is a variant of `v2.std.grammar` `GrammarExpr` **and** of `dag/std/constructors`
`Cardinality`; `Absent` is a variant of `v2.std.optional` **and** of `dag/std/upsert_decision`.
Ordinary §3 nickname collisions, invisible until the emitter reads them.

**Effect, both arms measured on the same entry:**

| | coded errors | E0308 | E0425 |
|---|---|---|---|
| baseline | 666 | 286 | 0 |
| sentinel honored | **527** | **128** | 25 |

−158 E0308 against 159 collapse events; −139 total. One predicate.

**A retraction in the same measurement.** This lane previously reported the emitter was receiving
the body's *constructed variant* type rather than the declared `Optional`, on arity evidence
(`optional_absent()` fieldless → kids 0; `optional_present(value)` → kids 1). The arity evidence is
real and still unexplained, but it is **not** the cause of the collapse: a *correctly spelled*
`Optional` collapses too, for the ambiguity reason. Fixing inference alone would have moved 91
events between buckets and fixed nothing.

**The 25 new E0425 are honest residue, not a regression.** With the collapse gone, a type argument
genuinely spelled `Absent` renders as a Rust type named `Absent`, which does not exist. The
mis-spelling was always there; unit-collapse was **fabricating plausible output** over it (§5). The
arity finding survives as a real second defect, now typed and located instead of masked.

**Relation to the candidate above — RETRACTED 2026-08-16 by `eager-deer-389`, whose wording caused
it.** The two mechanisms do **not** share the name `Absent` and the apparent coincidence was an
artifact of this document. `is_value_variant_type_arg` fires on the *variant name* `Absent`
declared by `v2.std.optional` and `dag/std/upsert_decision`. `type_leaf_is_unbound_in_closure_scope`
fires on the `Absent` **arm of an `Option`** returned by `lookup_type_by_name` — a lookup miss. One
is a name in the corpus; the other is a coproduct arm in the emitter's own control flow. They are
still both closure-membership facts, so an interaction is not excluded — but the specific
same-name premise is false and no experiment should be designed around it.

**Not claimed:** that the seed closure lacks these collisions (the obvious next measurement, and it
would make this structurally identical to Root B rather than analogous); any attribution of the
remaining 527. The fix above was applied to the **generated** Rust as a probe and reverted — the
authority is `src/v1/05_emit_rust.dag` `is_value_variant_type_arg` and nothing is landed.

**The proper fix is construction-first and it is two things.** Rename the colliding variants so the
ambiguous population is empty (single authority; deletes the class). The emitter guard is the
fail-closed backstop and is required regardless — today an ambiguous name silently produces `()`
rather than a typed located refusal, so the landed guard should **refuse**, not the non-empty-owner
test used for measurement, which still silently proceeds.

#### §10 — instance 3, QUALIFIED against the meta-root by a follow-up measurement (`gentle-dove-833`, 2026-08-16)

**I claimed this was a third instance of "the emitter has never been exercised on a seed-free
closure." That claim is now partly falsified, by my own instrument, and the qualification matters
more than the instance did.**

`derive_variant_to_enum`'s sentinel insert was instrumented to dump the ambiguous set per closure,
and run on a pure-v2 entry and a seed entry under identical source roots:

```
src/v2/compiler/05_emit.dag   6 distinct ambiguous names:
    Absent · Named · Optional · Repeat · Terminal · TypeExprKindAuthorityInvalid
src/v1/05_emit_rust.dag       6 distinct ambiguous names:
    Absent · AsAuthored · Bind · Named · SnakeCase · Text
```

**The seed closure is not protected.** It carries ambiguous names too — six of them, including
`Absent`, which both closures share — and the emitter collapses them by the same predicate. So
this mechanism **is** closure-shaped (the *set* is a function of closure membership, and the two
sets genuinely differ) but it does **not** correlate in the direction the meta-root asserts: there
is no favourable seed arm and unfavourable v2 arm here. Both closures take the same arm; the v2
closure merely happens to have ambiguous names (`Optional`, `Absent`) that occupy far more type-arg
positions.

§10 names its own falsifier as "a third closure-shape branch that does NOT correlate." **This is
that.** It does not touch Root B, whose path-substring branch is genuinely seed-vs-v2. It does mean
the meta-root should be stated as *several independent decisions keyed off under-modeled closure
facts* — which is how `eager-deer-389` already restated it — rather than as *the seed is the
exercised configuration*, because at least one closure-shaped branch damages the seed equally.

**A second correction, and it makes the finding smaller and much more actionable.** I described
this as a state-space conflation in the carrier. Reading every consumer says otherwise: the other
six read sites **already guard the sentinel** —

```
2031  Present { value: parent } => if parent != "" { concat(parent, "::", name) } else { name }
2515  let enum_name = …; if enum_name == "" { [] } else { … }
2873  Present { value: enum_name } => if enum_name == "" { [] } else { … }
```

Only line 705 does not, because `map_contains_key` cannot see the value. So this is **one consumer
diverging from a convention its six siblings already follow**, not a design-wide gap. The minimal
correct change restores the authority's own convention rather than inventing a rule.

That said, the guard being hand-repeated at six sites is itself the §2 duplication that let the
seventh omit it, so the terminal shape is still to make the value a coproduct
(`Unowned | OwnedBy { enum } | Ambiguous { enums }`) so the conflated read has no constructor —
and, per `smart-ram-730`'s sequencing, land the refusal first so the renames can be *verified* to
empty the population instead of proving a negative against a silent predicate.

**Also retracted here, since it was written against a premise `eager-deer-389` has now withdrawn:**
my "Relation to the candidate above" paragraph assumed our two mechanisms shared the name `Absent`.
They do not — theirs is the absent arm of an `Option` returned by `lookup_type_by_name`, not a
corpus name. The three-run partition I proposed is cancelled; there was no overlap to partition.

---

## 11. THE UNCLASSIFIED TAIL — live partition (`smart-ibex-716`, 2026-08-16)

**Subject as dispatched:** the ~64% of the July cause-signature corpus that A+B+C+D did not
claim. **Answer, in one line:** the tail is not a long tail of singletons. It is **six further
mid-sized roots** (each 1.5–7% of live distinct sites) plus a genuine singleton residue of
**3.3%** — and the largest root in the whole live corpus is not A, B, C or D but the
**algebra-carrier representation** family at 27.2%. **— WITHDRAWN 2026-08-17, see §18: that class was
assigned by a keyword naming a type and is at least three mechanisms; at most 146 sites are the repr
mechanism. The largest single mechanism in the corpus is NOT this one.**

### 11.1 Instrument and stamp (every number below is from this run, nothing transcribed)

| field | value |
|---|---|
| date | 2026-08-16 |
| tree | `5e1a73fa33` (`origin/main` tip at probe) |
| route | `gunbc compile --source-root dag --source-root src/v2 --entry <mod> --target rust --dependency-pool-index primary-precedence` → `cssl_assemble` → `cargo check --release --lib --message-format=json` |
| contract | `CSSL_STD_SEED_LINK=1`, empty shim (per §9.2 and the probe script's invocation contract) |
| modules | `05_emit`, `06_translate`, `04_infer`, `03_ingest`, `emit_host`, `01_tokenize`, `materialization_carriers` |
| unit of count | **one distinct `(generated file, line, column, rustc code, cause signature)`**, deduplicated — not one diagnostic |
| signature | expected/found pair for E0308-family; receiver+method for E0599; trait+self-type for E0277; message otherwise — extracted from the JSON `spans[].label` / `children[]`, never from rendered text |

`frontier_probe_survey` was not used. The JSON route was chosen over the text log because
expected/found lives in a span label, and a text grep recovers it only by heuristic.

### 11.2 The count everyone has been quoting is inflated ~2.75×, and here is the exact factor

```
sum over the seven modules      5,156   <- the shape every prior census reports
distinct sites, deduplicated    1,874   <- the number of things that are actually wrong
inflation factor                 2.75x
```

**05_emit (35 source lines) and 06_translate (4,226) do not merely have similar histograms —
their diagnostic sets are byte-identical: 666 rows each, intersection 666, symmetric difference
0**, at file, line, column, code and signature. Their emitted closures are *not* identical (89
vs 88 resolved sources), so the difference between those two entry modules contributes **zero**
cargo diagnostics. `04_infer`'s 614 rows are a strict **subset** of that same set.

Consequence, stated plainly because §6 assigns work by it: **a per-module error total is not a
measurement of that module.** It is mostly a measurement of the shared closure, and for these
three entries it is *only* that.

| entry module | distinct sites | in the five-module floor | own delta |
|---|---:|---:|---:|
| 05_emit | 666 | 605 | 61 |
| 06_translate | 666 | 605 | 61 |
| 04_infer | 614 | 605 | 9 |
| 01_tokenize | 203 | 110 | 93 |
| materialization_carriers | 386 | 108 | 278 |
| emit_host | 1,073 | 605 | 468 |
| 03_ingest | 1,548 | 605 | 943 |

Two refinements of H1 that matter, because the strong form is false:

- The floor is **cluster-shaped, not universal.** Intersecting all seven gives only **96** rows.
  Intersecting the five larger entries gives **605**. `01_tokenize` and `materialization_carriers`
  sit on a different closure and share ~110 rows with it, which is why §4's H3 was right to call
  `01_tokenize` unrepresentative — but for a measured reason now: it shares 110 of the floor's 605.
- The delta is **not small** for the two largest entries (943 and 468). "One core plus twenty
  thin deltas" is wrong; it is one core plus two thick deltas plus four thin ones.

### 11.3 The partition

Every one of the 1,874 rows is in exactly one row of this table. `RESIDUE` is printed, never
absorbed — the classifier is fail-closed, so an unmatched signature raises the residue count
rather than joining the nearest root.

| root | sites | % | in floor | in delta |
|---|---:|---:|---:|---:|
| **B1 — algebra-carrier representation** ⚠️ **WITHDRAWN as one root — see §18** (≤146 repr-shaped; 167 derive-shaped on one underivable declaration; 191 E0369 repr_fork per §18.4) | ~~509~~ | ~~27.2~~ | 80 | 429 |
| **C — Optional collapses to `()`** (owner `gentle-dove-833`) | 167 | 8.9 | 136 | 31 |
| **A — generic Clone bound not emitted** (owner `smart-ram-730`) | 142 | 7.6 | 98 | 44 |
| **K — unsynthesized use-line** (E0433/E0425/E0422 unresolved names) | 132 | 7.0 | 13 | 119 |
| **D — generic argument count** (owner `vivid-wren`) | 116 | 6.2 | 25 | 91 |
| **T3 — collection-carrier fork** (`PartialFunction`/`PointwisePower`/`OrdSet` vs `im`) | 110 | 5.9 | 53 | 57 |
| **T7 — ContentHash carrier vs `String`** (`Fnv1a64Structural`) | 105 | 5.6 | 8 | 97 |
| **T5 — missing derives on named types** (serde/Debug/Hash/Eq/PartialEq) | 92 | 4.9 | 48 | 44 |
| **B3 — numeric representation** (`Nat`/`Int` vs `{integer}`/`i64`) | 75 | 4.0 | 21 | 54 |
| **RESIDUE — genuine singletons** | 62 | 3.3 | 22 | 40 |
| R1 — bare↔`Rc` wrap decision (same leaf type, one side wrapped) | 55 | 2.9 | 9 | 46 |
| E — unreachable patterns (lint) | 42 | 2.2 | 33 | 9 |
| T5b — deref of a non-pointer (`Option<_>`) | 33 | 1.8 | 4 | 29 |
| T2 — text carrier (`String` vs `Vector<i64>`/`FreeMonoid`) | 31 | 1.7 | 4 | 27 |
| R2 — Optional *variant* surface (`Present`/`Absent` vs `Option`) | 29 | 1.5 | 10 | 19 |
| T4 — record emitted as a tuple (`(Rc<Node>, Rc<Node>)`) | 27 | 1.4 | 0 | 27 |
| L — borrow lifetime (E0597) | 26 | 1.4 | 5 | 21 |
| R3 — function-value carrier (`Rc<dyn Fn>` vs closure / `Fn` bound) | 23 | 1.2 | 1 | 22 |
| B2 — `Bool` vs `bool` | 20 | 1.1 | 2 | 18 |
| F — type annotations needed (E0282) | 19 | 1.0 | 5 | 14 |
| N — argument count (E0061) | 19 | 1.0 | 1 | 18 |
| M — struct-literal missing fields (E0063) | 18 | 1.0 | 16 | 2 |
| R5 — duplicate type authority across emitted modules | 16 | 0.9 | 8 | 8 |
| O — misc generics (E0392/E0631/E0271/E0310) | 4 | 0.2 | 1 | 3 |
| P — emitter refusal embedded in the source | 2 | 0.1 | 2 | 0 |

**Answer to the dispatched question:** A+B+C+D as owned today account for roughly 1,029 of 1,874
(≈55%) once B is read as B1+B2+B3. The remainder is **not** 140 singletons; it is K, T3, T7, T5,
R1, T4, R3, R5 — eight nameable mechanisms — plus 62 singleton rows.

### 11.4 The six unowned roots: mechanism, assigning evidence, size, falsifier

> **ENUMERATION INCOMPLETE since §18 (annotated 2026-08-17).** This list is not false and nothing in
> it directs anyone to do the wrong thing — but it is the list a session greps to learn what the
> partition contains, and **B1 is no longer one root**: §18 decomposes it into at most 146
> repr-shaped sites, 167 derive-shaped on one underivable declaration, and 191 E0369 operator-on-carrier
> sites (all repr_fork — §18.4). Read §18 before treating "six" as the root count or before
> re-deriving a mechanism for the algebra carrier.


**K — unsynthesized use-line. 132 sites.**
*Mechanism, already named in tree:* `reference_derived_use_lines_note` (`src/v1/05_emit_rust.dag`)
states that namespace-only resolution references cross-module names without importing them, the
resolver declines the use-line as a non-error advisory `UnlistedImportUse`, and the emitted Rust
is invalid — the note itself predicts "E0422/E0433/E0425 downstream". This root is that
prediction, measured: 132 sites, and the note's own §5 fail-open language is the root cause.
*Evidence rule:* rustc code in {E0433, E0425, E0422, E0412, E0573} — "cannot find type/struct X
in this scope" for a name the corpus does declare elsewhere.
*Falsified by:* any of these names not being declared in the compiled closure at all (that would
make it a real missing declaration, not a missing `use`), or by the synthesis walk being disabled
for these modules for an unrelated reason.

**T3 — collection-carrier fork. 110 sites.**
*Mechanism:* modeled collection algebra (`PartialFunction`, `PointwisePower`, `OrdSet`) is
constructed and field-accessed as a record (`.member`, `.lookup`, `.keys`) while the emitted Rust
type is a native `im` container that has no such field. Same shape as Root B one level up: a
model↔realization fork, but over *containers* rather than scalars.
*Evidence rule:* E0560/E0609/E0615 naming `member`/`lookup`/`keys` on those carriers, plus the
E0308 pairs `OrdSet<_>` ↔ `Rc<PointwisePower<_>>` and `HashMap<_,_>` ↔ `Rc<PartialFunction<_,_>>`.
*Falsified by:* the field names resolving on some emitted definition of those types (i.e. the
emitter *does* emit a record with `.member` and the failures are wrapper-depth only), which would
move this population into R1.

**T7 — ContentHash carrier vs `String`. 105 sites.**
*Mechanism:* `Fnv1a64Structural` (DESIGN's landed ContentHash family grounding) is emitted where a
`String` is expected and vice versa — 63 one way, 39 the other. The bidirectionality is the tell:
this is not one wrong declaration but a seam where the modeled hash carrier and its wire/string
serialization are not distinguished at emission.
*Evidence rule:* signature mentions `Fnv1a64` or `ContentHash`.
*Falsified by:* the two directions localizing to disjoint files with unrelated causes — in which
case this is two roots, not one. **Not yet checked; the cheapest next observation for this root.**

**T5 — missing derives on named types. 92 sites.**
*Mechanism:* an emitted struct/enum is used as a map key, a serde payload or a `Debug` argument
without the corresponding derive. Distinct from Root A: A is a **bound on a generic parameter**,
T5 is a **derive on a concrete type**. Both surface as E0277, which is exactly why grouping by
code hid them.
*Evidence rule:* E0277 whose self-type is a concrete named type and whose trait is
Serialize/Deserialize/Debug/Hash/Eq/Ord; plus E0369 `==`/`!=` on a plain named type.
*Falsified by:* a fix that adds the missing generic bounds (Root A) also closing these — which
would prove they were A's derived-impl bounds all along. **Separating observation:** T5's self
types (`Node`, `EnvironmentBindingKey`, `ParsePositionKey`, `ValueInterpreter`) are non-generic,
so no bound on a type parameter can reach them.

**R1 — bare↔`Rc` wrap decision. 55 sites.**
*Mechanism:* DESIGN's open `Rc`-ownership wrap-decision thread, measured. `expected X, found Rc<X>`
and the exact reverse, for `SpanIndex`, `ScopeRoster`, `SubjectRoster`, `ConsumerRequirement`,
`DecimalDigitsStep`, `Edge`. July's diagnosis reported this as "RC_WRAP/OWNERSHIP 17–23% of
E0308"; live it is 2.9% of all sites, so **this root has shrunk substantially** and is no longer a
headline.
*Evidence rule:* the expected and found strings differ only by an `Rc<…>` or `&` wrapper.
*Falsified by:* the wrapper difference being a consequence of a wrong upstream carrier (in which
case the row belongs to that carrier's root, not here).

**R3 — function-value carrier. 23 sites. R5 — duplicate type authority. 16 sites. T4 — record as
tuple. 27 sites.** Small, named for completeness. R5 is the one worth a second look for its
*kind* rather than its size: it is two modules declaring one concept (`OccurrenceId` in both
`std_occurrence_identity` and `v2_std_node`; `Nat` in both `std.nat` and `v2.std.nat`, the latter
fork already declared in tree at `nat_max_two_nat_authorities_note`). That is a §3 violation
producing type errors, and no amount of emitter work fixes it.

### 11.5 Two July roots are DEAD, and one is far larger than its ticket

Checked because §9.1 says nothing here is current unless it names today's measurement:

- **"DIAGNOSTICS carrier fork — 26–30% of E0308, the largest single bucket"**
  (`gate1_repr_mismatch_e0308_diagnosis_2026-07-24.md` Root 1) is **8 sites, 0.4%**, and none of
  them is the `Option<String>` vs `Diagnostics` pair that defined it. That root closed.
- **"WITNESS<T> parametrization gap — 18–23%"** (same document, Root 2) is **zero sites**. The
  string `Witness<_>` does not occur in the live corpus at all. What remains under D is
  `missing generics for enum Witness` (33), a different signature.
- **Root B, as ticketed at ~196, is undersized if the algebra carrier belongs to it.** B1 alone is
  509 sites — **but see §18: B1 is withdrawn as one root, so any B1+B3 unification argument must be
  re-made against the ≤146 repr-shaped subset, not against 509.** See 11.6 for the original argument.

This is the receipt-staleness class §9.1 warns about, caught twice more. Anyone planning against
the July E0308 bucket shares should stop.

### 11.6 The closure-shape meta-root: checked as asked, and it is bigger than two instances

§10 asks each lane to check cheaply whether its root's mechanism has a closure-shape branch. Mine
does, and the fan-out is wider than the numeric representation:

`corpus_has_v1_seed_source_indices` has **three** call sites (`04_infer` `reconcile_with_census_extra`,
`05_emit_rust` `emit_rust`, `05_emit_rust` `emit_module`). Each feeds `build_emit_graph_info`,
which stores `rust_corpus_repr(has_v1_seed)` on `EmitGraphInfo.corpus_repr`. That one field is
then read at these decision sites in `src/v1/05_emit_rust.dag` — named by symbol, per §3:

| reader | what it decides | root it lands in |
|---|---|---|
| `rust_seed_host_numeric_alias` | `Nat`/`Int` → `i64`, or stay modeled | B3, and via `std.nat Nat = CommutativeSemiring<Magnitude>`, **B1** |
| `rust_zero_value` | `String` zero is `"".to_string()` or `v1_rt::freemonoid_empty::<i64>()` | T2 |
| `emit_fn_def` (`host_text_op` via `rust_host_string_op_fn_emit`) | emit a host string-op fn at all | T2 |
| `module_needs_faithful_carrier_imports` / `module_renders_faithful_text_carrier` | faithful carrier imports | T2 |
| `emit_v2_std_text_closure_stub_module`, `v2_std_integer_stub` gates | whether stub modules exist in the crate | B3/T2 |
| `emit_module_full`'s `reference_derived_use_lines` arm | whether import-bearing modules get synthesized use-lines | **K** |

So the flag decides numeric representation, text representation, stub-module presence **and
import synthesis**. B1 + B3 + T2 + K is **347 sites, 18.5% of the live corpus**, downstream of one
`contains(k, "src/v1")` test. With `eager-deer-389`'s instance that is not a third instance — it
is the same instance, correctly sized.

**Two facts that bound the hypothesis rather than support it**, reported because §10 asks for the
falsifier:

1. **Two gates take `corpus_repr` and never read it.** `rust_seed_host_container_base` and
   `is_host_text_carrier_type` both accept the parameter and branch only on the type's name. So
   container realization and text-carrier *detection* are **not** closure-branched, and T3's 110
   sites are therefore **not** downstream of the flag. A dead parameter threaded through a
   decision surface is its own §3 smell, and it is the reason a reader can over-attribute here.
2. **K's direction is the opposite of B's.** The faithful (pure-v2) branch is the one that *runs*
   `reference_derived_use_lines`; the seed branch gets `[]`. So K is not "the v2 path was never
   exercised" — it is "the v2-only path exists, runs, and is incomplete". Flipping the repr switch
   would not close K; it would disable the walk that is trying to fix it.

*Falsifier, and it has since been EXECUTED by `eager-deer-389` — see §8:* forcing `HostNative` on a
pure-v2 `06_translate` takes the algebra carrier from 74 distinct sites to **0**, which confirms B1
is the numeric alias surfacing through `std.nat Nat = CommutativeSemiring<Magnitude>`. The same run
refutes flag-flipping as a *fix*: unresolved names go 19 → **110**, because forcing the seed branch
disables the use-line walk — the asymmetry predicted in point 2 below, measured. It also measures
`Bool` at 11 → 11, so B2 is not repr-switched at all. Their numbers are one module executed; the
1,874 here is the corpus denominator. **Do not average the two.**

### 11.7 The negative result, stated as a result

The residue is **62 sites, 3.3%**, and it is genuinely miscellaneous: the largest single entry is
13 (`Coverage<Rc<…>>` vs `CoverageDefectAcceptanceKey`), then 9 (`arguments to this function are
incorrect`, a rustc message that carries no pair), then a 3-and-below tail including two `await`
outside `async`. There is **no eighth root hiding in the residue** at this measurement.

So the dispatch's hoped-for outcome — "a root larger than B sitting in there unnamed" — is
**answered yes, but not in the residue**: it was sitting in plain sight as the algebra carrier,
mis-sized at 196 because the July TSV counted its numeric surface and not its algebra surface.

### 11.8 What I did not establish

- **No before/after.** This is one measurement of one head; nothing here says a root is shrinking
  or growing except where I compare to a July *document* (11.5), and those comparisons inherit
  that document's staleness in the other direction.
- ~~**Seven modules, not twenty.**~~ **SUPERSEDED by §11.14 (2026-08-16):** `emit_module`,
  `03_normalize`, `program_partition` and `05_eval` were subsequently probed at the same head and
  added **nine** new distinct sites in total, so the same-floor suggestion was measured and held.
  Two of the modules this bullet named as unprobed are now among the eleven. What remains genuinely
  unmeasured is `emit_produced` and `05_emit_orchestration` — named here so the residue is a list
  rather than a feeling.
- **`unreachable_patterns` is counted as an error row** because the crate denies it; if the
  denial is lifted, E drops out of the denominator and every percentage above moves ~2%.
- **Root ownership is by generated file, and I have not published the file map.** The per-root
  file concentration is available in the receipts and is sometimes extreme — Root C is 113 of its
  167 sites inside `src/v2_compiler_body_lowering_fold.rs` alone, which is a fact `gentle-dove-833`
  should have.

### 11.9 Receipts

Raw JSON diagnostic logs, the extracted per-module signature TSVs, and the classifier are in this
session's scratchpad, not committed: a dated snapshot nothing regenerates is the attractor §2
names. The two scripts are ~60 lines each and the route is fully specified in 11.1 — re-derive
rather than trusting the table. Anything above that is not reproducible by that route is a defect
in this section.

### 11.10 Name-keyed realization, answered against the tail — YES for 253 sites, NO for ~1,500

Asked by `smart-ram-730` (2026-08-16): *would identity-keyed realization have prevented this class,
or is the root orthogonal to it?* Answered per root, with the strongest specimen executed rather
than argued.

**NAME RETIRED FIRST, so nobody reconciles against it:** the root I published in 11.3/11.4 as
**"T7 — ContentHash carrier fork"** is **withdrawn as a name**. The population is unchanged (105
sites, same signatures, same classifier rule) but the cause is not a carrier fork; it is a
**seed-prelude name collision**, established below. Read every earlier "T7 — ContentHash carrier
fork" in this section as "T7 — seed-prelude `Hash` collision".

**YES — and here is the cleanest specimen in the corpus, which is my root T7 (105 sites, 5.6%).**
`src/v2/std/node.dag:14` declares `type Hash = Fnv1a64Structural`. The emitted crate contains

```
src/v2_std_node.rs:41   pub type Hash = v1_rt::Hash;          // and v1_rt.rs: pub type Hash = String;
src/std_content_hash.rs pub fn content_hash_atom(value: String) -> Rc<Fnv1a64Structural>
```

The alias's declared right-hand side is **discarded** and replaced by the seed runtime type that
shares its authored name — `Hash` is the return type of the v1 builtins `atom_identity_hash` /
`hash_combine` (`src/v1/00_core.dag` `hash_type`, a Node named `"Hash"` carrying a kernel span).
The mismatch is then visible *inside a single emitted signature*:
`pub fn bag_hash_digest(empty: Hash, xs: Rc<Vec<v1_rt::Hash>>) -> Hash`. Every one of T7's
63 `expected Rc<Fnv1a64Structural>, found String` and 39 reverse sites is that one substitution
read at a use site. The fact that separates the two `Hash`es — which declaration the name denotes —
is carried on the node and is not consulted. This is `eager-deer-389`'s mechanism exactly, with a
seed *prelude* homonym rather than two corpus modules, and it means **T7 is not a "ContentHash
carrier fork" at all; it is a name-collision with the seed runtime.** I am re-labelling it in this
section rather than in the table above so the table keeps matching its classifier.

**YES — K (132 sites).** `reference_derived_use_lines_note` states the synthesis resolves
candidates through the **bare-name registry**, describes that registry as last-write-wins, and
erects an export-proof construction wall specifically against "the `v1_rt::member` fabrication
class: registry homonym without export proof". A wall against homonyms is the shape of a mechanism
that has a name where it needed an identity: `Node.inferred = Resolved{…}.ident_span.file` names
the declaring module, so an identity-keyed emitter derives the use-line directly and neither the
registry lookup nor its wall is needed.

**YES — R5 (16 sites).** Two modules declaring one concept (`OccurrenceId` in both
`std_occurrence_identity` and `v2_std_node`) is the same shape as the `Bool` specimen, and the
diagnostics are literally `expected v2_std_node::OccurrenceId, found std_occurrence_identity::OccurrenceId`.

**PARTLY — T3 (110 sites).** The realization table *is* name-keyed (`rust_seed_host_container_base`
tests `name == "List" || name == "FreeMonoid"`; `is_host_text_carrier_type` tests
`nm == "String"` and an element named `"Char"`, both via `authored_name_at`), so it carries the
same homonym exposure. But T3's actual failures are `.member` / `.lookup` on `OrdSet` /
`PointwisePower` / `PartialFunction`, and those types are **absent from the table entirely**.
Identity-keying changes how the row is looked up; it does not author the missing row. Count this
as exposure, not cause.

**NO — the remaining ~1,500 sites.** A (generic bound), C (Optional→unit), T5 (derives on concrete
types), R1 (Rc wrap), L, M, N, F, E, T4 and the residue all turn on *shape*, *ownership* or
*which traits a declaration needs* — questions where the emitter already has the right declaration
and computes the wrong answer about it. Identity-keying leaves every one of them exactly where it is.
That bound is the useful half of this answer: name-keying is a real deficit with **253 sites
(13.5%) directly attributable and 110 more exposed**, and it is not the wall.

**And the masking warning applies here too, so I am budgeting for it in advance:** T7's sites are
`String`-vs-record errors standing in front of whatever those call sites do with the value. Fixing
the alias will expose refinement-carrier work (`Fnv1a64StructuralDigestHex = String where lower_hex_16`)
that the type error is currently firing ahead of. A burn-down of 105 that does not budget for the
unmasked population will overshoot, exactly as the algebra-carrier fix exposed 125 `Rc<i64>` sites.

### 11.11 The name-keyed substitution, located — and it is one authored table, not a code path

Following 11.10 to its cause rather than leaving it at "the emitter uses the name". The chain, by
symbol:

1. `dag/extdeps/languages/rust/types.dag` `rust_type_checkpoints` is a row table **keyed on
   `dag_name: String`** — a bare authored spelling carrying no module, no declaration reference.
   One of its rows is `{ dag_name: "Hash", target_type: "v1_rt::Hash", … }`, authored for the v1
   seed's builtin `Hash` (`src/v1/00_core.dag` `hash_type`, the return type registered in
   `04_method.dag` for `atom_identity_hash` / `hash_combine`).
2. `src/v1/coercion.dag` `lookup_checkpoint` / `coerce_primitive_type` resolve against that table
   by name alone.
3. `src/v1/05_emit_rust.dag`, in the `is_type_alias_item` arm of item emission, calls
   `rust_scalar_checkpoint_render_base(dag_name: item_text, …)` **before** rendering the declared
   right-hand side, and on a hit emits `pub type <name> = <cp.target_type>;` — the RHS is never
   consulted.
4. `src/v2/std/node.dag:14` declares `type Hash = Fnv1a64Structural`. The name matches the row.
   Output: `pub type Hash = v1_rt::Hash;`, and `v1_rt::Hash = String`.

So the substitution is not a bug in a rendering function; it is **an authored row whose key cannot
express which declaration it is about**. That is the same table `eager-deer-389`'s `Bool` finding
names, which makes the table — not the emitter — the shared subject of at least two lanes.

**One consequence for `vivid-wren`, and it splits Root D rather than claiming it.** Root D's 116
sites are two mechanisms, and only one is name-keyed:

- **`missing generics for enum Witness` (33) — name-keyed, same table.** `rust_type_checkpoints`
  carries `{ dag_name: "Witness", target_type: "Witness" }` *and* a second row spelled `"witness"`.
  A checkpoint row states a bare target type, i.e. arity zero, for a name whose declaration is
  generic. This is "the checkpoint row that claims arity for a bare name" exactly.
- **`type alias takes 0 generic arguments but N were supplied` (73) — NOT name-keyed.** Executed
  check: the emitted `src/v2_lens_coverage.rs:45` reads
  `pub type CoverageDefectAcceptance = Rc<Coverage<Rc<CoverageDefectAcceptanceKey>>>;` — the alias
  is emitted **already applied**, with its generic parameter list dropped at the declaration while
  use sites still supply an argument. Nothing about naming decides that; the parameter list was
  lost on the way to the declaration. (It also accounts for my residue's 13
  `expected Coverage<Rc<…>>, found CoverageDefectAcceptanceKey`.)

Keeping that split is the same discipline as the T3 one above: a real mechanism must not absorb the
symptoms next to it. **Name-keying's directly-attributable total therefore moves from 253 to 286
sites (15.3%)** — T7 105 + K 132 + R5 16 + D-Witness 33 — with T3's 110 still exposure rather than
cause, and the ~1,470 NO unchanged in substance.

## 12. WHERE THE PROGRAM STANDS AT END OF 2026-08-16 (`smart-ram-730`)

Written as a reader's entry point, because sections 1–11 were authored in the order they were
discovered and several of their early claims are superseded by later ones in the same document.
Nothing here is new evidence — every number below is cited to the section that measured it.

### 12.1 The one sentence to carry

**Name-keyed realization is real, is directly attributable to 286 sites, and is NOT the wall.**

The Rust emitter answers questions about a type from its *authored token* when the resolved
declaring module is already attached to the node being rendered. Three lanes found this
independently, from three different symptoms, and it converges on one function —
`rust_scalar_checkpoint_render_base`, a lookup keyed on a bare `String`.

Then it was bounded, and the bound is the load-bearing half: **286 attributable, 110
exposed-but-not-caused, ~1,500 not attributable** (§11.10). 15.3% — the §11.10 amendment, which
supersedes the 253 / 13.5% this section carried until T7, K, R5 and D-Witness were counted in. The remaining sites turn on
shape, ownership, or which traits a declaration needs — cases where the emitter already has the
right declaration and computes the wrong answer about it.

I record plainly that I was one confirming report away from calling name-keying the root of the
program. Two structural confirmations arrived before any measurement of the negative space, and
generalizing from them would have been wrong. The corpus partition is what prevented it.

### 12.2 What was falsified today, and by whom

Five published claims died on measurement. Listing them together because the rate matters more
than any one of them: this partition is roughly a day old and a third of its early content did
not survive contact with the live corpus.

| claim | status | who |
|---|---|---|
| July Root 1 — DIAGNOSTICS carrier fork, "largest single bucket", 26–30% of E0308 | **8 sites, 0.4%** | §11 |
| July Root 2 — `Witness<T>` parametrization, 18–23% | **zero occurrences of the string** | §11 |
| Root B ≈ 196 sites | **under-measured — but the 509 that replaced it is itself WITHDRAWN (§18): keyword-assigned, at most 146 repr-shaped** | §11, superseded by §18 |
| "342 diagnostics citing the algebra carrier" | **74 distinct sites; the 342 was a line count** | `eager-deer-389`, self-corrected |
| T7 = "a ContentHash carrier fork" | **a seed-prelude name collision; same 105 sites, different cause** | `smart-ibex-716`, self-corrected |
| Root A = struct-only derive trigger | **under challenge by its own author; see 12.5** | this session |
| "an empty `EmitGraphInfo` is a shared upstream beneath Roots B2 and R1" | **DEAD, twice over — and it was mine** | this session, killed by `lively-ibex-709` + `bold-lark-722` |

**The last row is a hypothesis killed before it cost anything, and it is recorded because the next
session will regenerate it.** While root-causing a Root D residue, `keen-ibex-435` found the emit
authority rendering a type with `empty_emit_graph_info()` — an env-blind render — and classified all
14 production sites: 2 legitimately context-free, 7 latent, 5 plumbing. Only one was fixed, because a
discriminating case existed for exactly one; for the five plumbing sites a case was **attempted per
site and not found**, which is why they are enumerated rather than threaded (threading an env into a
callee has zero observable effect unless some type renders differently, so symmetry is not a reason).

It is a natural hypothesis that this is the shared upstream beneath the two open operator/wrap roots —
both are "the decision disagrees between declaration and call site", which is what a missing env looks
like from downstream. **It is false, and it was killed twice by different strengths of evidence.**
`lively-ibex-709` established population disjointness (**none** of the B1 E0369 sites they examined are
in the operation/capability emit paths — the sites span `std_*` files exclusively, a file set disjoint
from the seven-site latent set counted above, which shares no member with it and is a different
quantity despite the coincident 7) and then signature absence (**none** show a `_` or unresolved type
var in the rustc operand types; all name concrete carriers). Their report gave both as fractions of
112. **§18.4 reconciles that denominator** and it is not a discrepancy: 112 is the distinct-site count
of the **July 7-module measured bank**, and 191 is the same classification at **M=11 scope** — two
scopes of one population, both `repr_fork`, both with zero `missing_trait_impl`. The seven `std_*`
files named above are that bank's module set, which is a **different quantity** from the seven latent
`empty_emit_graph_info()` sites counted earlier in this section; the two sevens share no member and
their coincidence is arithmetic, not structural. Quote 112 only as the measured bank and 191 only as
the M=11 partition, never interchangeably. The two findings above do not depend on either number:
both are universals over whatever set was examined, and a wrong count cannot turn a "none" into a
"some". `bold-lark-722`
killed it for the wrap axis by mechanism: at those sites `shared_types` arrives as its OWN positional
argument carrying the real set, and `render_rust_type` reads that parameter rather than
`emit_info.shared_types`, so wrap membership is not degraded — what an empty info actually zeroes is
`fn_generic_param_names`, `variant_to_enum` and `fn_type_env`, which degrade generic-scope rendering
and applied-binding resolution, a different diagnostic class. That last read is **structural, not
executed**, and its author declined to call it proof.

So the seven latent sites are their own defect, not a hidden root beneath the open ones — which is a
more useful result than a positive would have been, because it stops three lanes from converging on
one wrong cause. Recorded here so the hypothesis is re-read rather than re-run.

Two of those are authors falsifying their own published numbers unprompted. That is the behaviour
this surface exists to make cheap, and it is the reason the remaining numbers are worth anything.

### 12.3 The masking law — applies to every fix in this program

Every root measured so far is firing *in front of* another population. Fixing it does not create
the sites underneath; it stops hiding them. Their frequency was zero by construction, which is
DESIGN §5's absorbing fallback read at the diagnostic level.

- algebra carrier → **125 `Rc<i64>` sites** appear and `Measure` rises 9 → 37 (§8, executed)
- variant unit-collapse → **25 new E0425** (`gentle-dove-833`)
- Hash substitution → the `Fnv1a64StructuralDigestHex` where-refinement population (§11.10)

**Standing rule for this program: quote net, never gross, and state what you expect to unmask
before you unmask it.** A reviewer who sees new sites appear will otherwise read the fix as their
cause.

### 12.4 Ownership and collision, as of this writing

| root | live size | owner | state |
|---|---|---|---|
| B — algebra/numeric carrier, closure flag | ~~509~~ → **at most 146 repr-shaped (§18)**, and narrower still: sites whose type carries a checkpoint row do not move | `eager-deer-389` | identity-keying design; flip control executed both directions |
| C — variant ambiguity sentinel | 167, 113 in one file | `gentle-dove-833` | fix in `src/v1/05_emit_rust.dag`, regen fixed point confirmed twice |
| D — checkpoint arity | 116 = 65 alias + 33 `Witness` | half 2 `vivid-wren-870`, half 1 `stern-badger-166` | **both halves LANDED** — half 1 as `bbb52138b25` ([PR #8350](https://github.com/gunb-ai/gunbc/pull/8350)), which also retired an enrolled witness that had been asserting the broken shape; half 2 as `7cfeb6f0fd7` ([PR #8341](https://github.com/gunb-ai/gunbc/pull/8341)), deleting the `Witness` checkpoint-scalar rows so scalar arity derives from its single authority |
| A — derive/bounds | **LANDED**, [PR #8347](https://github.com/gunb-ai/gunbc/pull/8347) merged as `c4e9cc918c5` | this session | see 12.5 for what the diagnosis got wrong on the way |
| tail — six mid-sized roots + 62 singletons | 1,874 distinct total | `smart-ibex-716` | partitioned, §11 |

**Live collision:** B and D both land on `rust_scalar_checkpoint_render_base`. The split asked for
is *one owns the rows, one owns the key* — a wrong checkpoint row stays wrong under any key, so
`Witness` deletion is independently correct; re-sourcing the lookup key is the other change. Both
editing the arm's logic is the thing to avoid.

### 12.5 Root A is unverified and I am saying so before anyone builds on it

I published a root cause for A: `v1_clone_bound_seed_for_item` skipping coproducts
(`if is_coproduct_type(n: item) { round }`) in `src/v1/trait_derive_emit.dag`. Going back to
implement it, two things argue against it.

The skip is **deliberate and defended in the same file**:
`trait_derive_emit_item_clone_bound_wf_propagation_note` states the derive trigger is correctly
scoped to structs because derive emits per-impl bounds for enums, and that well-formedness
propagation — which does apply to both — is a separate trigger with its own fixpoint.

And the population is **E0599, no method found**. A missing `Clone` bound is E0277. Those are
different failures with different fixes, so a bounds diagnosis for an E0599 population is
suspicious independent of which trigger scopes what. A's split into CloneSharedRequirement 369 /
TargetApiRequirement 168 / OwnedDeconstructionRequirement 63 came from the same July TSV that
produced two of the dead claims in 12.2.

~~**A's live size, code mix and cause signatures are requested from §11's instrument. Until they
land, treat A as unpartitioned, not as diagnosed.**~~ — **SUPERSEDED by the RESOLVED block below;
A is landed, not open.** Struck rather than deleted because the instruction was correct when
written and the paragraph above it records *why* the then-current diagnosis was refused, which is
the part that still governs.

**RESOLVED 2026-08-17 — and the challenge above was right, which is why what landed is not what
this section proposed.** [PR #8347](https://github.com/gunb-ai/gunbc/pull/8347) merged as
`c4e9cc918c5`: *split the clone-bounds map into the two facts it was answering.* The fix is a
fused-map decomposition, not the coproduct-skip edit the retracted diagnosis pointed at — the skip
is still there and still deliberate, exactly as `trait_derive_emit_item_clone_bound_wf_propagation_note`
defends it. The E0599-vs-E0277 mismatch this section flagged is what kept that edit from being made.

Two receipts worth keeping, because both were nearly missed:

- **The landed change needed a stage0 regeneration that was not in the first push.** Four generated
  files (`v1_compiler_emit_rust`, `v1_compiler_infer`, `v1_compiler_infer_emit_info`,
  `v1_compiler_trait_derive_emit`) were stale against the `.dag` edit; `regen` was green on the
  final head, which is the receipt that matters.
- **Two regen runs before that reported a PERFECT FIXED POINT while measuring nothing.**
  `ctrl-build --remote` syncs to the *invoking* head, and the fix lived on a different branch, so
  the runs compiled a tree that never contained it. A wrong-branch arm is worse than a stale one —
  correct SHA, correct binary, correct argv, clean provenance, and every other check passes on it.
  Only a subject-presence grep printed beside the verdict (`grep -c` on a construct the change
  introduces: 0 in base, >0 in the change arm) distinguishes the two. Anyone regenerating stage0
  for a root fix should print that count next to the fixed-point verdict, always.

### 12.6 Adjacent, landed: the self-host frontier roster is deleted

Operator-ordered, [PR #8344](https://github.com/gunb-ai/gunbc/pull/8344), 34 files, −5,836/+284,
on `session/smart-ram-730-frontier-cut`. Relevant to readers of this document for one reason: the
roster is what several sections originally used to talk about self-host progress, and it never
measured anything. `execution_measured_seed_retained_row` took its measurement fields as ordinary
parameters, so a row claiming measurement was indistinguishable from one asserting it.

Cutting at the root made the census cheap and the answer was that nearly nothing depended on it —
crate layout, wet enrollment and crate partition each read a projection that was empty at every
call. **Do not cite roster rows as evidence of anything in this document.**

### 11.12 Root A characterized at site grain (asked by `smart-ram-730`, answered by measurement)

The question was whether A is really a *bound* root given that its population is E0599, "no method
found" — a different failure from E0277's unsatisfied bound. **It is a bound root; the two codes are
one failure reported from two positions.**

| slice | sites | reading |
|---|---:|---|
| E0599 `<method> exists for <generic type>, but its trait bounds were not satisfied` | 64 | bound failure, reported at a call site — receivers are `Outcome<T>`, `im::Vector<T>`, `AudienceSet<P>`, `CacheLookupResult<T>`, `Option<T>` |
| E0277 `the trait bound `T: Clone` is not satisfied` | 48 | the same failure at a coercion site (`T` 25, `A` 6, `U` 5, `P` 4, …) |
| E0599 `no method named `clone` found for type parameter `T`` | 21 | an *unbounded* `T` genuinely has no methods — still a bound failure |
| E0599 `as_ref` exists for `&v2_std_nat::Nat`, bounds not satisfied | 9 | **not A** — concrete receiver, no type parameter for a bound to reach |
| **A, after deduction** | **133** | one mechanism |

Totals: **142 sites** under the classifier's rule, E0599 94 / E0277 48; **133** after moving the 9
concrete-receiver rows to the `Nat` representation family. Summed over the seven modules the same
rows are **722**, a **5.09× inflation** — well above the corpus-wide 2.75×, so A is *more*
concentrated in the shared floor than the average root and per-module A counts overstate it worse
than most. Concentration by generated file: `src/v2_std_algebra.rs` **69**, `v2_std_diagnostic.rs`
18, `std_authorization_profile.rs` 9, `v2_lens_cost.rs` 9 (four sites land inside `im-15.1.0`'s own
source).

**Is A one root? Yes at this measurement** — unlike the July DIAGNOSTICS and WITNESS buckets, which
this run found at 8 sites and 0. There is no second population hiding inside it.

**A discriminator the owner can run, offered because my data cannot settle which trigger is at
fault:** 64 of the 142 name a *generic type* as receiver rather than a bare parameter. That is
exactly the container-field-on-a-generic-coproduct shape `trait_derive_emit`'s own note predicts
earns a bound from neither trigger. If the published root-cause is right, those 64 are its
signature; if a well-formedness fix leaves them standing, the root-cause is refuted by that alone.

**Do not reconcile the July emitter-decision split (369/168/63, summing 600) against the 133.**
That census counted diagnostics summed over modules; these are distinct sites. The two are
different denominators, and 600 vs 722 is the comparison that would mean something.

## 13. ROOT A, DIAGNOSED — one map answering two questions (`smart-ram-730`, 2026-08-16)

Supersedes 12.5, which withdrew A pending live signatures. They landed. **A is 133 sites, one
root, bound-shaped.**

### 13.1 What A actually is (`smart-ibex-716`, live, seven modules, distinct-site grain)

142 distinct sites; codes E0599 94 / E0277 48 and nothing else. The apparent code split is a
reporting artifact, not two mechanisms:

| | count | shape |
|---|---|---|
| E0599 `<m> exists for <T-parametrized type>, but its trait bounds were not satisfied` | 64 | generic receiver |
| E0277 `the trait bound T: Clone is not satisfied` | 48 | coercion position |
| E0599 `no method named clone found for type parameter T` | 21 | bare parameter |
| E0599 `as_ref exists for &v2_std_nat::Nat, ...` | 9 | **concrete receiver — not A** |

rustc reports an unsatisfied bound as E0277 at a coercion site and as E0599 at a call site, and
says "no method" for a bare parameter because an unbounded `T` genuinely has none. Deduct the 9
concrete-receiver sites (they belong with the `Nat` repr family) and **A = 133 sharing one
mechanism**. 69 of 142 land in one generated file, `src/v2_std_algebra.rs`.

*Cited so it is not re-derived wrong: the earlier split (CloneSharedRequirement 369 /
TargetApiRequirement 168 / OwnedDeconstructionRequirement 63, summing 600) came from the July TSV
and does not reconcile against 133. A's summed count is 722 against 142 distinct — 5.09x, above
the corpus-wide 2.75x, so A is more concentrated in the shared floor than average and per-module
counts overstate it worse than most.*

### 13.2 The mechanism

`v1_clone_bounded_type_params` (`src/v1/trait_derive_emit.dag`) is ONE
`Map<String, Set<String>>`, and two different consumers read it:

1. **Declaration emission** — `v1_item_clone_bounded_param_names`, reached from
   `emit_item_type_params_with_clone_bounds`: does this item's own declaration print `<T: Clone>`?
2. **Well-formedness propagation** — `v1_type_expr_wf_needs_clone_param`: does NAMING someone
   else's `G<T>` oblige the naming item to bound its own parameter?

Those are different questions, and the map is seeded as if they were one.
`v1_clone_bound_seed_for_item` opens `if is_coproduct_type(n: item) { round }`.

**For question 1 that skip is correct**, and the carrier
(`trait_derive_emit_item_clone_bound_wf_propagation_note`) is right to defend it: `derive(Clone)`
on an enum emits `impl<T: Clone> Clone for E<T>`, so the declaration needs no item-level bound.

**For question 2 that per-impl bound is exactly the fact a consumer must be told about.** Cloning
`Outcome<T>` *does* require `T: Clone`, and the map is where propagation looks for it. Since no
coproduct is ever seeded, a generic coproduct contributes nothing and every consumer that names
and clones one earns no bound.

`v1_item_field_type_exprs` already flat-maps variants, so the **fixpoint** handles enums fine — it
has nothing seeded to propagate from. Likewise `v1_item_type_param_needs_clone_bound_struct` is
struct-scoped in name only; its body takes a `List<Node>` of field type exprs and is shape-agnostic.

**Specimens, read in tree:**

- `Outcome<T> = Accepted { value: T, diagnostics } | Rejected { diagnostics }` — bare `T` in a
  variant payload, the shape the struct seed would have caught.
- `CacheLookupResult<T> = Hit { receipt: CachedArtifactReceipt<T> } | Miss | RejectedHit` — the
  whole chain is starved: `CachedArtifactReceipt<T>` is a struct with no bare `T`,
  `ArtifactIdentity<T>` never uses `T` at all (phantom), and `ProducerReceipt<T>` is another
  coproduct. Nothing in the chain can seed.

**This is the program's recurring shape, third instance today.** One answer serving two questions —
the same defect as the checkpoint arm answering "arity 0" and "I have a row", and the variant
sentinel answering "no owner" and "empty name".

### 13.3 Correction to my own earlier claim

I published A's root cause as "the struct-only derive trigger", pointing at the same line. **Right
location, wrong reason, and the difference changes the fix.** Deleting the coproduct skip would
print a spurious item-level bound on enum declarations that `derive` already supplies — and the
enum decl site (`05_emit_rust.dag`, the `emit_enum_from_children` caller) already routes through
`emit_item_type_params_with_clone_bounds`, so seeding coproducts without splitting the read would
change emitted enum declarations corpus-wide.

### 13.4 The construction, and why this shape

**Give well-formedness its own seeded record; leave the declaration-bounds map untouched.**
Strictly additive: no existing emitted byte changes except where a consumer newly earns a bound it
already needed. The alternative — one map plus a kind filter at the read — would work but keeps
the fusion and re-invites the same confusion at the next reader.

Declaration emission keeps its current map and behaviour. The WF record is seeded from structs
*and* coproducts through the same shape-agnostic predicate, and `EmitGraphInfo` carries both.

**Discriminating control, and it must go both ways:** a generic coproduct with a bare-`T` payload
whose consumer clones it (currently E0599, must go green), *and* a generic enum declaration whose
emitted `<T>` must stay bare — because the failure mode of this fix is over-bounding enum
declarations, which no "the errors went away" measurement would catch.

**Falsifier put to `smart-ibex-716`, unanswered at time of writing:** the diagnosis predicts A's
64 generic receivers are dominated by coproducts, *not* generic structs, since structs are already
seeded. A broad struct share refutes it and A returns to unpartitioned. `im::Vector<T>` (a
container with no declaration bound — a derive-trigger fact only, per the carrier) is not a
counterexample.

**Not started.** Diagnosis is by reading; there is no executed before/after, and verification needs
a remote regen plus rebuild. Nothing here is a receipt.

### 11.13 Root A's generic receivers, split enum vs struct — the seeding diagnosis survives

Run against the 64 generic-receiver sites from 11.12, at `smart-ram-730`'s request, to test the
prediction that A's failing receivers are generic **coproducts** and not generic structs.

```
34  enum     Outcome<T> 10 · AudienceSet<P> 5 · Option<T> 5 · CacheLookupResult<T> 4 ·
             Outcome<U> 4 · Grounding<E> 2 · ShowEffectiveRead<R> 1 · Reconciliation<A, E> 1
30  struct   im::Vector<T> 23 · im::Vector<U>/<A>/<B> 7
```

**Corpus-declared generic structs in the failing population: zero.** All 30 struct-shaped receivers
are the upstream `im` container, which `trait_derive_emit`'s own note already classifies as carrying
no declaration bound to propagate. So the population is exactly the two predicted shapes — generic
coproducts that were never seeded, plus an external container with nothing to seed from — and the
prediction survives. `AudienceSet<P>`, previously unclassified, is an enum.

**But the two halves take different fixes, and the raw 34/30 hides that.** The 34 enum sites are the
seeding gap. The 30 `im::Vector` sites cannot be closed by seeding from a declaration at all — the
declaration is upstream and carries no bound — so they need the requirement to come from the target
API's cited impl requirements, i.e. the v2 per-derive-impl contracts named in
`trait_derive_emit_item_clone_bound_contract_fork_note`'s dissolution clause. A fix reported against
64 will land ~34.

**Method note, kept because this nearly inverted:** "struct" in a rustc message names the
*receiver's declaration kind*, not *whose corpus declared it*. Read at face value, 30 structs refutes
the diagnosis; the discriminating question is which module the receiver comes from, and only that
separates the hypotheses. This is the same shape as the E0599/E0277 reading in 11.12 — the code, or
the noun, answers a neighbouring question to the one being asked.

**Not settled by this run:** whether `Outcome` is seeded by some path other than
`v1_clone_bound_seed_for_item`. These measurements see receivers, never the seeding map.

## 14. ROOT A, CORRECTED AGAIN — §13.4's construction is REFUTED (`smart-ram-730`, 2026-08-16)

**The mechanism in §13.2 survives two falsifiers. The fix in §13.4 does not.** Refuted before it
ran, by two witnesses that were already in the tree.

### 14.1 What survived

`smart-ibex-716` ran both checks I asked for.

**Receiver split of A's 64 generic-receiver sites — diagnosis survives, but read it carefully:**
34 enum / 30 struct. The struct share looks like a 47% counterexample and is not — **all 30 are
`im::Vector<T>`**, the upstream container the carrier already names as having no declaration bound
to propagate. **Corpus-declared generic structs in the failing population: zero**, which is
exactly what the diagnosis predicts, since structs are already seeded. Receivers:
`Outcome<T>` 10 · `AudienceSet<P>` 5 (an enum) · `Option<T>` 5 · `CacheLookupResult<T>` 4 ·
`Outcome<U>` 4 · `Grounding<E>` 2 · others 2; `im::Vector` 30.

**Method note worth more than the result** (theirs): *"struct" in a rustc message is a fact about
the receiver's DECLARATION KIND, not about whose corpus declared it.* The discriminating question
is which module the receiver comes from. A one-minute check answering the adjacent question would
have killed a correct diagnosis while wearing the authority of a measurement.

**Refuter — is `Outcome` seeded by some other path? No, and structurally so.** There is one
seeding call; the fixpoint can only add through `v1_type_expr_wf_needs_clone_param`, which opens
`if (type_expr.children |> count) == 0 { false }`. **A bare `T` has no children, so the
well-formedness axis can never bound a bare parameter.** `Outcome`'s only `T`-bearing field is the
bare `value: T` (both `Diagnostics` types are non-generic). The two axes have complementary blind
spots and a generic coproduct with a bare payload falls in the gap.

**A is two roots wearing one label.** The 34 enum sites are the seeding gap. The 30 `im::Vector`
sites cannot be seeded from a declaration at all — upstream, no bound — and need the target API's
own impl requirements, which is `trait_derive_emit_item_clone_bound_contract_fork_note`'s
dissolution. Quoting 64 as one target would deliver 34 and look like missing half.

### 14.2 What was refuted, and by what

§13.4 proposed: seed coproducts, and keep declaration emission struct-only by filtering coproducts
at `v1_item_clone_bounded_param_names`. **Premise false.** `dag/test/claim/generic_item_clone_bound_witness_test.dag`
already asserts, in `w_bound_propagates_into_coproduct_item`, that
`type OccurrenceBinding<N> = BoundTo { path: ContainmentPath<N> } | Unbound {...}` emits
**`enum OccurrenceBinding<N: Clone>`** — an enum declaration DOES print an item-level bound when
well-formedness propagation fires. The kind filter would have redded it.

Its sibling `w_coproduct_bare_payload_param_stays_bare` asserts `enum PlainChoice<N>` stays bare,
and that is also correct: `derive(Clone)` supplies `impl<N: Clone>`, so the declaration does not
need it.

**Both witnesses are right; the two facts really are different, and they really are in one map.**
An enum's declaration must print a bound earned from *well-formedness* and must not print one
earned from the *derive seed*. A kind filter cannot separate those — only two records can. §13.4's
own first sentence proposed exactly that and I implemented the shortcut instead.

Reverted before execution. It cost an hour rather than a day only because I went looking for
existing evidence before writing new evidence — the general lesson, not an A-specific one.

### 14.3 Where the failure actually bites, corrected

Not at consumer-item declarations. `derive(Clone)` on a consumer struct already yields
`impl<T: Clone>`. The failures are **fn-level**: `fn f<T>(o: Outcome<T>) { ... o.clone() ... }`
earns nothing from either fn trigger — the structural one (`v1_generic_params_needing_clone_bound`:
bare-generic return or direct container element) does not match `Outcome<T>`, and the wf one
(`v1_fn_param_wf_needs_clone`) finds `bounds[Outcome]` empty. rustc then reports *clone exists for
`Outcome<T>` but its trait bounds were not satisfied*, which is the measured message verbatim.

**Open, and the reason this is not yet implementable.** The correct fn-level question is
*cloning* `tau` requires `P: Clone` — `v1_type_expr_clone_impl_needs_param`, which reduces to
"P occurs in tau". Applying that unconditionally over-bounds: a fn that takes an `Outcome<T>` and
never clones it does not need the bound, which is why the existing structural trigger is
usage-shaped. **Deciding a fn's clone-impl requirement needs a usage fact this diagnosis has not
established.** That is the next thing to establish, and it is not a fourth guess at the same
question.

### 14.4 Two testing traps found today, both corpus-wide

**Reverting the generated `.rs` is a false green** (`gentle-dove-833`). `claim_batch` interprets
the `.dag`, so perturbing the emitted artifact perturbs something the witness never reads. Four
rows stayed green against a reverted projection; reverting the AUTHORITY flipped exactly one. Two
verifications through two execution paths, one measuring nothing. Anyone verifying an emitter
change is one step from this, because reverting the visible output is the intuitive control.

**The variant-name wall is module-local** (same). The compiler already refuses two enums claiming
one variant name *within* a module, so the ambiguous state is reachable only ACROSS modules —
which is where every live case sits. The wall is drawn exactly at the boundary the defect crosses.
Whether it should be corpus-wide is an open decision nobody owns.

### 11.14 The seven-module bound in 11.8 is now measured: four more modules add NINE new sites

11.8 listed "seven modules, not twenty" as an unmeasured limit, with the suggestion that the
unprobed entries sit on the same floor. Probed: `emit_module`, `03_normalize`, `program_partition`,
`05_eval`, same route, same head.

```
prior five-module floor                                       605
emit_module        total 674   in-floor 605   own delta  69
03_normalize       total 568   in-floor 556   own delta  12
program_partition  total 675   in-floor 605   own delta  70
05_eval            total 773   in-floor 605   own delta 168
nine-module floor                                             556

corpus distinct sites   7 modules  1,874
corpus distinct sites  11 modules  1,883      <- FOUR more entries, NINE new sites
sum over 11 modules                7,846      <- inflation 4.17x
```

**The distinct-defect corpus is saturating.** 2,690 further diagnostics produced **nine** sites not
already seen, and they are shallow: four E0282, two E0392 and one E0308 in `v2_lens_application.rs`,
one E0308 and one `unreachable_patterns` in `v2_compiler_program_partition.rs`. Every root's size in
11.3 is unchanged to within one site (`T3` 110 → 111, `E` 42 → 43, residue 62 → 63).

Three consequences:

1. **The partition is not a sample of the wall; it is close to the wall.** Planning against these
   root sizes does not need a twenty-module census first.
2. **The inflation factor grows with the number of entries probed** — 2.75× at seven, 4.17× at
   eleven — because each new entry re-counts the same floor. Any figure of the form "N diagnostics
   across M modules" therefore says more about M than about the defect population, and two such
   figures taken at different M are not comparable in either direction.
3. `05_eval`'s 168-site delta is the largest of the four, which is consistent with §6's note that its
   lane sits mostly behind defects that are not eval's — but 605 of its 773 are floor, so the
   proportion holds.

### 11.15 Where Root A actually bites: fn generic parameter lists, measured

`smart-ram-730` reported (after its published fix was refuted by two existing witnesses) that the
failure bites at **fn bounds, not consumer declarations**. Measured independently, by walking each of
A's sites in the emitted `03_ingest` tree up to its enclosing item (121 of A's sites are in files
that tree contains):

```
91  fn whose generic parameter list carries NO bound      75%
13  fn whose generic parameter list carries SOME bound    11%   <- see below
 9  fn with no generic parameters at all                   7%   (the concrete-receiver residue, 11.12)
 6  struct / enum declaration                              5%
```

**104 of 121 (86%) are fn signatures; 6 are type declarations.** A fix that adds item-level bounds to
consumer *declarations* cannot reach this population, which is consistent with the refutation.

**The 13 partially-bounded signatures are the sharpest evidence in this root**, because they show the
mechanism is not failing wholesale — it bounds some parameters of a signature and misses others, and
the missed one is always the parameter used *through another generic type*:

```
fold_list<T, A: Clone>                                          fails on T   via Rc<im::Vector<T>>
fold_list_right<T, A: Clone>                                    fails on T   via Rc<im::Vector<T>>
resolve_probe<A: Clone, B>                                      fails on B   via CacheProbe<B>
reconcile_grounded<A: Clone, R, E>                              fails on R   via ShowEffectiveRead<R>
grounding_qualified_by_durability<E, Target: Clone, ReadbackSubject: Clone>
                                                                fails on E   via Grounding<E>
```

This is the two-blind-spot composition stated at fn grain: the wf axis can only propagate a bound
*from a declared type that already carries one*, and the carriers these parameters flow through carry
none — `CacheProbe`, `ShowEffectiveRead`, `Grounding` and `Outcome` are generic **coproducts**, which
the seed skips, and `im::Vector` is upstream with no declaration bound at all. Every parameter that
DID earn its bound in these same signatures flows through something else.

**Discriminating prediction, recorded before any fix lands:** seeding the propagation record from
coproducts should bound the parameters flowing into `CacheProbe`/`ShowEffectiveRead`/`Grounding`/
`Outcome` and leave the `im::Vector<T>` ones standing, because `im::Vector` has no corpus declaration
to seed from. If both move, the seeding gap was not the mechanism; if neither moves, the fix did not
reach fn signatures at all.

### 14.5 Why the two-map fix still does not close A on its own

Working the construction one step further than §14.2, because "two records instead of a kind
filter" is necessary and **not sufficient**, and the reason is worth stating before someone
implements the easy half.

The two records are:

| record | fact | why a declaration prints it |
|---|---|---|
| `clone_bounded_type_params` (today) | **naming** `G<A..>` requires `P: Clone` | well-formedness — naming an ill-formed type is an error whether or not you clone |
| new: clone-impl requirement | **cloning** `G<A..>` requires `P: Clone` | `derive(Clone)` emits `impl<P: Clone>` |

Seeding coproducts into the *new* record is correct and puts `T` into `Outcome`'s entry. The
temptation is then to point the existing fn trigger `v1_fn_param_wf_needs_clone` at the new record
and stop. **That over-bounds, for a reason that is not true.**

The wf trigger is unconditional because naming a bounded declared type is ill-formed on its own —
no usage required. That justification does not transfer: `enum Outcome<T>` declares **no** bound,
so `fn f<T>(o: Outcome<T>)` that never clones is perfectly well-formed, and bounding its `T` would
be a fabricated requirement. It also cascades — every caller then needs `T: Clone` too, which is
exactly the cascade the existing narrow structural trigger (`bare-generic return or direct
container element`) was scoped to avoid.

**So the clone-impl record must be consulted under a usage condition: does this fn body actually
clone a value of that type.** That fact is the open item from §14.3, and the place to look for it
is wherever body emission decides to insert a `.clone()` — if the emitter knows it emitted the
clone, it knows the requirement. Nothing in the current fn triggers reads a body; both are
signature-shaped.

Stated as a bound on the work rather than a plan: **the two-record split is mechanical, the usage
gate is not, and landing the split without the gate would trade 34 refusals for an unmeasured
population of over-bounded functions and their callers.** That trade has not been priced and
should not be made by default.

### 11.16 The over-bounding exposure is ~1 fn, so the usage gate is a refinement, not a prerequisite

`smart-ram-730` held the two-record split behind an objection: bounding `T` in a fn that *names*
`Outcome<T>` without ever cloning it fabricates a requirement. The objection is sound in principle;
the question it turns on — how many such fns exist — is measurable. Measured, on the emitted
`03_ingest` closure (177 files), over the generic coproduct carriers named in 11.15 plus their
siblings (`Outcome`, `CacheProbe`, `ShowEffectiveRead`, `Grounding`, `AudienceSet`,
`CacheLookupResult`, `Reconciliation`):

```
32  generic fns naming one of those carriers in a signature
21    carrier in PARAMETER position
21      ...and the body clones that parameter        <- 21 of 21
 0      ...and does not
 1  fns whose body contains no `.clone()` at all (carrier in return position only)
```

**No fn in this closure would be spuriously bounded on a parameter it never clones.** The single
no-clone fn has the carrier in return position only, so at most one site is exposed. The usage gate
is therefore a later refinement, not a prerequisite for the split.

*Method and its limits, so the number is not read as stronger than it is:* signatures are recovered
by joining continuation lines (180 of 197 generic fn signatures match single-line; the join covers
the rest), and "clones it" is `<param>.clone()` textually. That proxy is exact for the obligation in
question — a derive-generated `Clone` for `C<T>` is `impl<T: Clone> Clone for C<T>`, so cloning the
carrier *is* what requires `T: Clone`. One closure, not eleven: a carrier named only in a module
outside `03_ingest`'s closure is not counted, and the check is cheap to repeat elsewhere.

### 14.6 A's enclosing-item split, and the discriminator that confirms the mechanism from outside

`smart-ibex-716` walked each A site up to its enclosing item (121 sites, 03_ingest closure):

| enclosing item | count | share |
|---|---|---|
| fn whose generic parameter list carries NO bound | 91 | 75% |
| fn whose generic parameter list carries SOME bound | 13 | 11% |
| fn with no generic parameters (the concrete-receiver residue) | 9 | 7% |
| struct / enum declaration | 6 | 5% |

**104 of 121 are fn signatures. Six are type declarations.** So the refuted fix in §14.2 was aimed
at 5% of its own population — a sharper statement than "two witnesses killed it", and true
independently of the witnesses.

**The 13 partially-bounded signatures are the discriminator.** The mechanism does not fail
wholesale; it bounds *some* parameters of a signature and misses others, and the missed one always
flows through a carrier with no bound to propagate:

```
fold_list<T, A: Clone>                      fails on T   via Rc<im::Vector<T>>
fold_list_right<T, A: Clone>                fails on T   via Rc<im::Vector<T>>
resolve_probe<A: Clone, B>                  fails on B   via CacheProbe<B>
reconcile_grounded<A: Clone, R, E>          fails on R   via ShowEffectiveRead<R>
grounding_qualified_by_durability<E, Target: Clone, ReadbackSubject: Clone>
                                            fails on E   via Grounding<E>
```

Every parameter that DID earn its bound flows through something that has one. `CacheProbe`,
`ShowEffectiveRead`, `Grounding` and `Outcome` are generic coproducts the seed skips; `im::Vector`
is upstream with no declaration bound at all. A wholesale failure is consistent with half a dozen
causes; **a per-signature partial failure along exactly this line is consistent with one** — and
it was produced from outside the fixpoint, by someone who could not read the predicates.

**A's acceptance test, recorded before the fix so it cannot be retrofitted** (theirs). Seeding
coproducts into the propagation record should:

- **move** the parameters flowing into `CacheProbe` / `ShowEffectiveRead` / `Grounding` / `Outcome`
- **leave** the `im::Vector<T>` ones standing — no corpus declaration to seed from

Both moving refutes the seeding gap. Neither moving means the fix never reached fn signatures.
Three distinguishable outcomes from one run at M=11.

**§14.5's objection is downgraded from a blocker to a measurement.** The reasoning stands — the
well-formedness justification does not transfer to a coproduct that declares no bound, so bounding
a fn that never clones fabricates a requirement. But the 13 show the mechanism *already*
over-bounds in this shape for carriers that do have bounds. So the open question is not whether
over-bounding occurs (it does, today, by design) but **how many fns name these carriers without
cloning them.** That is countable. Small → the split lands and the usage gate is a later
refinement; large → the gate is a prerequisite. Treating it as a blocker was treating an unmeasured
quantity as a decided one.

## 15. MEASUREMENT RULES FOR THIS PROGRAM (adopted 2026-08-16)

Three rules, each derived from a measurement rather than a preference, each already having caught
a wrong number today.

**15.1 — Quote distinct sites at a fixed published M, and state M IN THE FIGURE.** Diagnostic
inflation is a function of how many entries were probed, because every entry re-counts the same
shared floor: 2.75x at M=7, 4.17x at M=11. So "N diagnostics across M modules" is mostly a
statement about M, and it cuts both ways — a wall that shrank after a fix is not evidence of the
fix if M fell, and one that grew is not a regression if M rose. `smart-ibex-716`'s addition is the
operative half: the failure mode is not people omitting M, it is two figures read side by side that
each mentioned M *somewhere*. **"1,883 sites at M=11" survives being quoted out of context;
"1,883 sites" does not.** The program's fixed denominator is the eleven-module census.

**15.2 — Across a wave boundary there is no honest delta at all** (`tidy-gull-813`, the harder
case). Where 15.1 says M drifts if you are careless, a delete-first cut in a compiled language
cannot hold M fixed *even in principle*: wave one is measured over the lib because that is all
cargo can reach, wave two over lib plus 35 bins **because fixing wave one is what made them
reachable**. Changing M is what advancing the work does. So a wave-two total exceeding wave one is
not a regression and not a measure of the fix — it is targets becoming visible. Comparison is valid
only *within* a wave, and a cross-wave before/after must not be published with a caveat; it must
not be published.

**15.3 — Net, never gross, and name the unmasked population before it appears.** Every root
measured so far fires in front of another it was hiding, so its frequency was zero by construction.
Algebra carrier → 125 `Rc<i64>` sites appear and `Measure` rises 9→37. Variant unit-collapse → 25
new E0425. Hash substitution → the where-refinement population. A reviewer who sees new sites
appear will otherwise read the fix as their cause.

### 15.4 Two testing traps and one diagnostic instruction

**Reverting the generated `.rs` is a false green** (`gentle-dove-833`). `claim_batch` interprets the
`.dag`, so perturbing the emitted artifact perturbs what the witness never reads: four rows stayed
green against a reverted projection, while reverting the AUTHORITY flipped exactly one. Two
verifications through two execution paths, one measuring nothing.

**A count is worth what its author's checking is worth.** `tidy-gull-813`'s data-reference sweep
nearly reported nine instead of seven, because `v1_interpreter_dispatch_generated` contains the
deleted name as a prefix — the substring trap, running the opposite direction from the same trap
corrected that morning. They reported the near-miss unasked, which is the only reason the seven is
usable.

**A reader looking for a wrong line will not find one.** Every root on this wall is a *correct local
answer to a question nobody asked at that site*. The checkpoint row is not wrong about the seed's
`Hash`. The closure switch is not wrong about a seed corpus. The variant sentinel correctly records
an ambiguity. The derive trigger is not wrong about enum declarations, and the well-formedness axis
is not wrong to ignore a bare parameter. The defect lives in the *relation* between two sites that
are each individually right — which is why they survived review, and why reading each predicate in
isolation keeps producing "this looks correct". **Ask which question each site is answering, and
whether anyone asked it there.**

Corroborated from a second corpus and a different activity (`tidy-gull-813`, deletion census rather
than diagnostic partition): two crate-layout emitters carry the module list as TEXT, so a regen
would emit a layout declaring a file that does not exist. The emitter is correct about the list it
was given; `lib.rs` was correct about the module it declared; the deletion is correct. Their
operational form of it is worth quoting exactly — **the silent residue of a deletion is exactly the
set of sites that reference the deleted thing as DATA rather than as CODE**: enumerable by grep,
invisible to every compiler, and the named specimen DESIGN's "what cannot break loudly" clause was
missing.

### 11.17 Root K characterized: it is a VARIANT-QUALIFICATION gap, not a general missing-import gap

K (132 sites at M=11) is the largest root with no owner. Characterized on the `03_ingest` closure
(127 of its sites), by reading the emitted line each diagnostic cites:

```
109  the missing name is used as a qualified path  `Name::Variant`      86%
 17  the missing name is a bare type reference                          13%
  1  the name is not on the cited line
```

and by syntactic position:

```
63  match arm or pattern        NodeKind::ComputationNode { .. } =>, Outcome::Accepted { .. } =>
54  other body position         Rc::new(Correction::Unavailable { .. }), NoCorrectionReason::…
 5  let-binding annotation
 5  fn signature
```

**So K is overwhelmingly the emitter qualifying a variant by its parent enum (`Parent::Variant`) in
a construction or a pattern, while the parent enum has no use-line.** Only 5 sites are fn signatures.
`reference_derived_use_lines_note` does claim to cover the variant case ("record-literal type name
+ its parent_enum", "variant constructors routed through their parent's import") — so the finding is
not that the mechanism ignores variants, it is that **the variant coverage does not reach pattern
positions and nested construction arguments**, which is where 117 of the 127 sit.

All 36 distinct missing names ARE declared in the corpus — spot-checked at their declaring modules:
`NamedEdgeTargetLookup` and `NodeKind` in `src/v2/std/node.dag`, `Outcome` and `Correction` in
`src/v2/std/diagnostic.dag`, `ConstructionMechanism` in `dag/std/disposition.dag`. Nothing here is a
missing declaration; every one is a missing `use`.

Concentration: `v2_lens_complexity_accumulator_copy_analyze.rs` 31, `v2_lens_reference_deps.rs` 18,
`v2_lens_unit_modeling.rs` 9, `v2_lens_fact_density.rs` 8 — and the worst file's source
(`src/v2/lens/complexity_accumulator_copy/analyze.dag`) has **no import lines at all**, which is the
import-FREE path the note says runs the full union walk. So this is not the import-bearing arm being
skipped; it is the arm that does run, under-collecting.

**One unrelated specimen surfaced by the same read, recorded so it is not lost:** the emitted corpus
contains `let read = filesystem.read(path.clone()).await?;` — an `.await` inside a non-async fn,
which is the residue's two E0728 sites. That is an emitter producing a construct the target language
cannot accept in that position, and it belongs to no root above.

### 11.18 Root T3 characterized: `Set`/`Map` are modeled as function-records and realized as native containers, with no side winning consistently

T3 (111 sites at M=11) is the next-largest unowned root. The authority is
`dag/std/types.dag:107-108`:

```
type Set<element>   = PointwisePower<element>      # dag/std/algebra.dag: { member: fn(T) -> Bool }
type Map<key,value> = PartialFunction<key,value>   # { lookup, empty, get, insert, merge, keys, values, has, size }
```

Both are **records of function fields** — a set IS its membership function, a map IS its lookup
function. The emitted crate contains those structs faithfully (`std_algebra.rs`
`pub struct PointwisePower<T> { pub member: Rc<dyn Fn(T) -> bool>, … }`). It ALSO renders `Set`/`Map`
*type positions* as native `im::OrdSet` / `im::HashMap`. Neither realization is wrong on its own;
they are simply not the same type, and both are reached from one alias.

Split by mechanism (M=11, all 111 rows, fail-closed):

| # | mechanism | sites | specimen |
|---|---|---:|---|
| a | **modeled literal vs native annotation** — `Rc::new(Set { member: … })` / `Rc::new(Map { lookup: … })` emitted where the annotation rendered natively | 43 | `expected OrdSet<String>, found Rc<PointwisePower<_>>` · `struct Rc<PartialFunction<_,_>> has no field named lookup` |
| d | Vector wrap / element-shape mismatches (`Vector<T>` vs `T`, `Rc<Vector<Rc<X>>>` vs `Rc<Vector<X>>`) — adjacent to R1, kept separate | 36 | `expected Rc<Vector<Rc<PortReading>>>, found Rc<Vector<PortReading>>` |
| b | **modeled field access on the native carrier** — the consumer half of (a) | 16 | `(terminals.member)(x)` on `Rc<OrdSet<Rc<FormalTerminal>>>` · `.keys` on `Rc<HashMap<…>>` |
| c1 | **the seed runtime's map API is String-keyed** — `v1_rt::lookup<V>(table: &HashMap<String, V>, key: String)` (`src/v1/stage0/src/v1_rt.rs`), so a modeled map keyed by anything else cannot reach it | 15 | `expected &HashMap<String, _>, found &Rc<HashMap<OccurrenceId, Rc<OriginEvent>>>` — also `Rc<Node>`, `ParsePositionKey`, `EnvironmentBindingKey` keys |
| c2 | `v1_rt::keys` does not exist | 1 | `cannot find function keys in module v1_rt` |

**Three things this settles for whoever takes it.**

1. **It is not a missing realization row** (my earlier reading in 11.10 said the types were "absent
   from the table entirely" — that was right about the checkpoint table and wrong as a description of
   the root). The types ARE realized, twice, and the defect is that construction and annotation
   disagree about which realization is in force at a given site.
2. **c1 is an independent obstacle and does not dissolve with a–b.** Even if every site agreed on the
   native carrier, a map keyed by `Rc<Node>` still cannot call the seed runtime's String-keyed
   `lookup`. That is a seed-API limitation, and 15 sites sit on it today.
3. **Direction matters and the corpus has already chosen** — the modeled record cannot be the
   realization for a map that must be *iterated* (`keys` returns a `FreeMonoid<K>` built by the record
   itself), while the native container cannot serve `.member` as a field. A decision that keeps the
   record shape has to author the whole API; a decision that keeps the container has to rewrite the
   literals and generalize the runtime API's key type. Both are real work; the current state is
   paying for both.

### 11.19 Roots T5 (92) and R1 (55) characterized

Done ahead of the request so they are dispatch-ready; neither is owned.

**T5 — "missing derives" is two different things, and one of them cannot be fixed by deriving.**

```
by trait          22 serde::Deserialize · 17 PartialEq (via E0369 ==/!=) · 16 Hash · 11 Debug · 11 Serialize · 11 Eq · 4 Ord
by self type      14 ParsePositionKey · 8 Node · 8 PartialFunction<String,…> · 5 Target ·
                   5 EnvironmentBindingKey · 4 each: ReadbackSubject, EffectIoEvalBundle, CompiledLexRule,
                   BindInterpreter, BranchInterpreter, LoopInterpreter
```

*T5a — map-key requirements the derive roster does not consult (~27 sites: Hash 16 + Eq 11).* The
emitted derives are a fixed roster that ignores how the type is USED. `ParsePositionKey` derives
`Debug, Clone, PartialEq, Serialize, Deserialize` and is then used as a `HashMap` key, which needs
`Hash + Eq`; `Node` has the same set and the same problem; `EnvironmentBindingKey` derives
`Eq, PartialOrd, Ord` but not `Hash`. **This half is a real missing derive** — the fix is to derive
from the use, or to make map-key-ness a modeled fact.

*T5b — serde/Debug demanded of types that carry function fields (~44 sites: Deserialize 22,
Serialize 11, Debug 11).* `CompiledLexRule` emits `#[derive(Clone)]` and nothing else, because
`fn_field_derive_traits()` is Clone-only — correctly, since `Rc<dyn Fn>` is not serializable and not
`Debug`. The failures arrive because a *containing* record derives serde while holding one of these
as a field, and `PartialFunction<String, …>` (a record of closures, §11.18) is the most-demanded self
type. **Adding the derive is impossible here; the requirement has to go.** So T5b is the same shape
as T3: a modeling decision (do not serialize a value containing closures), not a repair.

**DECIDED 2026-08-21** (session `tidy-dove-648`): "the requirement has to go" is only half true.
It holds for process-local realization carriers (`CompiledLexRule`, `PartialFunction`, the
interpreter family) — extend the already-correct `fn_field_derive_traits()` rule through
coproducts, where it is currently unwired. It does NOT hold for `ProducedDeclSupport`, which sits
inside `TargetModel` and should stay fully serializable — there the embedded `render` closure is
redundant dispatch beside an identity (`scaffold_relation_rule_name`) the record already carries,
and the fix is to remove the closure, not the record's derives. Full decision, per-declaration
disposition table, and the two handoffs:
[`t5b-closure-bearing-serde-debug-decision-2026-08-21.md`](t5b-closure-bearing-serde-debug-decision-2026-08-21.md).

**R1 — the Rc wrap decision is INCONSISTENT, not uniformly over- or under-wrapping.**

```
28  expected Rc<X>, found bare X
27  expected bare X, found Rc<X>          <- near-perfect symmetry
14  SpanIndex · 6 each ScopeRoster, SubjectRoster, ConsumerRequirement · 4 DecimalDigitsStep · …
```

Position (03_ingest, 54 sites read): **53 are function-call arguments**, 1 is a record literal, 0 are
signatures. So the declaration side and the call side disagree about the wrap for the SAME type — the
signature is internally consistent, the argument expression is not — and the symmetry means no
blanket rule ("always wrap", "never wrap") describes it. `whole_corpus_scope()` returning
`Rc<ScopeRoster>` into a parameter typed `ScopeRoster` is the canonical specimen.

**CORRECTION to the paragraph above, made before it propagated (2026-08-17).** "Inconsistent, no
blanket rule" is a population-level reading, and I published it without asking whether the two
directions involve the SAME types. They do not: **only 2 of ~20 types appear in both directions**
(`SpanIndex` 7/7, `Determinism` 1/1). The rest are one-directional —
`ScopeRoster` 6, `SubjectRoster` 6, `ConsumerRequirement` 6 over-wrapped; `DecimalDigitsStep` 4,
`Edge` 2, `Diagnostic` 1 and others under-wrapped. So R1 is **two largely disjoint populations**, and
per type the decision is mostly stable.

Consequence for the DESIGN open thread *Rc-ownership wrap-decision* and its note
(`docs/plans/rc-ownership-wrap-decision-design.md`), whose premise is a uniform over-wrap of
`shared_types` members: that premise is **incomplete, not refuted**. It remains a good candidate
explanation for the 28 over-wraps — `pub fn whole_corpus_scope() -> Rc<ScopeRoster>` is a data anchor
emitted wrapped, flowing into a parameter emitted bare, which is exactly a definition-side rule not
carried to parameter positions — and it is silent on the 27 under-wraps. Demoting the note wholesale
on the symmetry number would discard a possibly-correct mechanism.

**What would settle it, unrun and outside this instrument:** whether `ScopeRoster` / `SubjectRoster` /
`ConsumerRequirement` / `SpanIndex` are `shared_types` members at emission. That is a read of
`build_shared_types`' input, not of diagnostics.

Both roots are floor-heavy and cheap to re-measure at M=11 after any fix.

### 11.20 The `shared_types` read: BOTH R1 populations are shared types, so membership is not the discriminator

Run at `smart-ram-730`'s request, to settle whether the DESIGN note's premise (a uniform over-wrap of
`shared_types` members) is confirmed on R1's 28 over-wraps. Criterion, read from
`src/v1/05_emit_rust.dag` `build_shared_types` → `maybe_mark_shared_type`: for Rust
(`sharing.needs_sharing`), a type is shared when its repr is `StructRepr`, or `EnumRepr` with
`unit_only == false`, minus grounded-coproduct native aliases and `is_type_constant` — and
`is_type_constant` requires **every** field type to be `is_copy` in the Rust checkpoint table.

| type | declaration | shared? | R1 direction |
|---|---|---|---|
| `ScopeRoster` | `{ roots: List<String> }` | yes (field not copy) | OVER (6) |
| `SubjectRoster` | `{ entries: List<QualifiedName> }` | yes | OVER (6) |
| `ConsumerRequirement` | payload coproduct | yes (`unit_only == false`) | OVER (6) |
| `SpanIndex` | `{ entries: Map<OccurrenceId, OriginEvent> }` | yes | BOTH (7/7) |
| `DecimalDigitsStep` | `{ digits: FreeMonoid<DecimalDigit>, carry: Bool }` | yes | UNDER (4) |

**Every type in both populations is a shared type.** So membership does not separate over- from
under-wrapping, and "the emitter wraps every `shared_types` member" cannot by itself explain a set
that is uniformly shared and yet splits by sign. The premise is neither confirmed as the cause nor
refuted as a description; it is **not discriminating**, which is a third outcome and the one that
actually occurred.

**AMENDED 2026-08-17, and the amendment is the point: the paragraph below WAS WRONG and its error is
instructive.** I attached `bold-lark-722`'s v2 catalog read to this v1-produced census because the two
had the same shape, without checking that the emitter being described was the emitter that produced
the corpus. It was not. `bold-lark-722` established, three ways, that `gunbc compile --target rust`
is the **v1 seed emitter end to end** — there is no `v2_compiler_translate.rs` in stage0,
`build_shared_types` appears only in `v1_compiler_emit_rust.rs` and `v1_compiler_infer_emit_info.rs`,
and `cli_run.rs` imports `crate::v1_compiler_emit_rust` directly — so `rust_sg_rc_use_site_ownership_catalog`,
`TargetOwnershipUseSite` and `translate_coerced_with_atom_realization` were **dormant** while these 55
sites were emitted. The catalog fork is real and is a fact about the v2 path; it is not the cause of
anything measured here. *A fact that fits is not a fact that applies* — the same error as the 28/27
aggregate one section earlier, made in the opposite direction (there I over-generalized my own data;
here I under-checked someone else's).

**What replaced it, confirmed by inspection rather than by reading either emitter.** The discriminator
is *which renderer a position is wired to*, not membership. From the emitted `03_ingest` tree:

```
v2_compiler_parse.rs:78   pub struct ParseProvenanceState { pub alloc: OccurrenceIdAllocator, pub index: SpanIndex }
v2_std_provenance.rs:61   pub fn span_index_empty() -> Rc<SpanIndex>
v2_std_provenance.rs:87   pub fn span_index_merge(base: Rc<SpanIndex>, incoming: Rc<SpanIndex>) -> Rc<SpanIndex>

:575  index: span_index_empty(),                                           OVER
:582  index: span_index_merge(base.index.clone(), incoming.index.clone()), BOTH SIGNS
```

`base.index` is a field, emitted **bare**, passed to a parameter emitted **`Rc<`-wrapped** (the
under-wrap); the call's wrapped result is assigned into the bare field (the over-wrap). One
expression, one type, both signs — which is the only way a single file:line:column carries both.

Measured across the whole emitted tree, signature position is wrapped for **every** shared type
(`SpanIndex` 10, `ParseProvenanceState` 22, `ScopeRoster` 4, `SubjectRoster` 2, `ConsumerRequirement` 1,
`DecimalDigitsStep` 2) while field position is bare for four of five. **The wrinkle, stated because a
fix predicated on a clean split will leave a residue:** `ParseProvenanceState` is wrapped in all four
of its field occurrences and `SpanIndex` in one of three, so some second axis also wraps fields.
`OccurrenceIdAllocator` is the negative control — bare in both positions, not shared, and zero R1
sites.

The superseded reading follows, kept because the retraction is more useful than the deletion:

What this leaves standing is `bold-lark-722`'s catalog read (reported to me, not verified here):
`rust_sg_rc_use_site_ownership_catalog` rows disagree *by position* for the same carrier —
`BindingProjection → ReferenceLayerRc` while `FunctionParameter → ReferenceLayerOwned` — and
`TargetOwnershipUseSite` has exactly four inhabitants, **none of which is a call argument**, while
`06_translate` `translate_coerced_with_atom_realization` hardcodes `OwnershipAtBindingProjection` for
the generic node fold. An argument therefore carries the producer-side layer and the
callee-parameter question is never asked. That predicts precisely what was measured: per-type-stable,
opposite-signed, all shared, and 53 of 54 readable sites at call-argument position.

**Reconciliation of the two counts, recorded because they will be quoted together:** R1 is **55** at
M=11; **54** of those appear in the `03_ingest` run, which is the only closure whose emitted tree was
kept, so the position classification is over 54. The absent one is `v2_compiler_emit_host.rs:417`.
And `v2_compiler_parse.rs:582:12` carries **both** signs at one file:line:column, so 55 counts
diagnostics-at-sites, not distinct source positions.

### 11.21 The field/signature split is ALSO refuted — and the way it failed is a method warning

11.20's amendment reported that signature positions wrap and field positions are mostly bare, on five
types. `bold-lark-722` then proposed a third prediction — a shared type in a *generic* fn or struct
comes out bare regardless of position — and testing it broke both claims.

Measured over the emitted `03_ingest` tree, occurrences of shared types by enclosing item:

```
GENERIC fn signatures     Node 12 · Diagnostic 5                    17 WRAPPED,   0 bare
PLAIN   fn signatures     Node 2404 · Diagnostic 212 · Edge 215 ·
                          ParseProvenanceState 22 · SpanIndex 10 · …    ALL WRAPPED, 0 bare
GENERIC struct fields                                                6 WRAPPED,   0 bare
PLAIN   struct fields                                              121 WRAPPED,   5 bare
```

**Prediction 3 is refuted** in its unconditional form: zero bare occurrences in any generic signature
or generic struct field. The honest bound: 17 and 6 are small denominators, and the test keyed on the
*enclosing item's* generics, so a narrower condition (the occurrence itself being a type argument, or
its resolved node carrying `__applied_type_args`) is untested.

**And my own field/signature split is refuted by the same run.** Struct fields are **121 wrapped
against 5 bare** — fields overwhelmingly DO wrap. The "fields are bare" reading came from five types
I had picked *because their bare occurrences appeared in the R1 diagnostics*, i.e. a sample selected
by the sites that failed.

**The method warning, which is the transferable part:** an error census tells you where disagreement
occurred; it never tells you what the majority behaviour is. Using the failing population to infer
the emitter's general rule inverted the ratio — 5 exceptions read as the rule against 121 that were
not in the sample because they worked. Any claim of the form "the emitter does X" drawn from a
diagnostic census needs a denominator drawn from the emitted corpus instead.

What survives, and is now better supported than any position rule: **121 wrapped vs 5 bare inside one
position** is exactly "the same semantic type answers the membership question differently at
different occurrences", which is the occurrence-keyed reading (`authored_name_at(rt_child)` at fields
rather than the type identity `build_shared_types` computed membership under). The five bare fields —
two of them the `SpanIndex` fields of `ParseProvenanceState` — are the whole specimen set for that
claim, and testing it needs the resolved node rather than the emitted text.

### 11.22 Independent consistency sweep: ten types, fourteen occurrences — and four rows move from T3 to R1

`bold-lark-722` swept their own emitted tree for types answering the wrap question inconsistently and
reported 921 of 928 consistent with six real offenders. Re-run independently on this session's tree
(913 types with occurrences), the same question returns **eleven** inconsistent types:

```
Nat                 sig_rc 197  sig_bare  0   field_rc 2  field_bare 4
Witness             sig_rc  26  sig_bare 65   field_rc 2  field_bare 6   <- different mechanism
Finding             sig_rc  26  sig_bare  0   field_rc 0  field_bare 1
PortReading         sig_rc  16  sig_bare  0   field_rc 0  field_bare 1
TerminationProof    sig_rc  16  sig_bare  0   field_rc 1  field_bare 1
Determinism         sig_rc  10  sig_bare  0   field_rc 0  field_bare 1
SpanIndex           sig_rc  10  sig_bare  0   field_rc 1  field_bare 2
ScopeRoster         sig_rc   4  sig_bare  0   field_rc 0  field_bare 1
NarrowingReason     sig_rc   3  sig_bare  0   field_rc 0  field_bare 1
SubjectRoster       sig_rc   2  sig_bare  0   field_rc 0  field_bare 1
ConsumerRequirement sig_rc   1  sig_bare  0   field_rc 0  field_bare 1
```

Their six all appear and agree row for row. **`Witness` must be excluded** — it is the only row with
bare *signature* occurrences (65), and it is Root D: the checkpoint table carries
`{ dag_name: "Witness", target_type: "Witness" }`, a row asserting a bare target for a generic
declaration. Including it would give the population two causes.

That leaves **ten types and fourteen bare field occurrences.** The extras are all the same shape at a
deeper position — the miss happens at **element position inside a container field**, not only at the
field's head:

```
v2_lens_complexity_accumulator_copy_analyze.rs:602   pub findings: Rc<Vec<Finding>>
v2_lens_complexity_accumulator_copy_analyze.rs:430   pub readings: Rc<Vec<PortReading>>
v2_lens_enforcement_standing_intent.rs:80            pub allowed_narrowing: Rc<Vec<NarrowingReason>>
v2_compiler_infer.rs:86                              pub descent: Rc<Witness<TerminationProof>>
```

The container is wrapped and the element is bare, while every signature naming those types wraps them.

**Partition correction, mine not theirs:** four rows filed under T3's "Vector wrap / element shape"
bucket are this mechanism, so at mechanism grain **R1 is 59 and T3 is 107**:

```
expected Rc<Vector<Rc<PortReading>>>, found Rc<Vector<PortReading>>
expected Rc<Vector<Finding>>,         found Rc<Vector<Rc<Finding>>>
expected Vector<NarrowingReason>,     found Vector<Rc<NarrowingReason>>
expected Vector<PortReading>,         found Vector<Rc<PortReading>>
```

Both directions appear among those four, which is the one-defect-both-signs claim reproducing at a
second grain.

**Two independently produced trees agree at the specimen** (`ParseProvenanceState` at
`v2_compiler_parse.rs:78`, `alloc` bare, `index` bare) and two independent instruments converge on the
same fourteen-occurrence population from opposite ends — 121-vs-5 counted by position here,
921-vs-7 counted by type there. That convergence is worth more than either count alone.

### 11.23 T5b characterized: ~10 declarations demand serde/Debug over closure-bearing values

The 44 sites, attributed to the declaration whose derive *demands* the trait (not the type that fails
to implement it). All 44 land inside twelve enclosing declarations, and the top five carry 32:

```
18  RuntimeBehaviorInterpreter                            v2_std_runtime.rs
 6  InterpretationStructureWitness                        v2_std_runtime.rs
 3  ProducedDeclSupport                                   v2_std_compilers_target_model.rs
 3  EffectIoEvalContext                                   v2_compiler_eval.rs
 2  LexWalkAcc                                            v2_compiler_tokenize.rs
 2  TargetDeriveSupplementalGenericBoundContractAuthority v2_std_compilers_target_model.rs
 2  TargetDeriveSupplementalGenericBoundContract          "
 2  TargetCollectionRealization                           "
 2  TargetRepresentationParameterSlot                     "
 1  EffectIoYieldOutcome                                  v2_compiler_eval.rs
 3  (attribution not resolvable within 200 lines)
```

The failing values are `ValueInterpreter` / `TransformInterpreter` / `BranchInterpreter` /
`LoopInterpreter` / `BindInterpreter` / `MatchInterpreter` (6 × 3 traits = 18 under
`RuntimeBehaviorInterpreter` alone), `EffectIoEvalBundle`, `CompiledLexRule`,
`PartialFunction<String, …>`, and — in one case — a bare
`dyn Fn(Rc<Node>) -> Rc<Outcome<Rc<TargetBodiedArrowStatementScaffold>>>`.

**The operator question this makes precise.** `ProducedDeclSupport` holds
`render: Rc<dyn Fn(Rc<Node>) -> …>` **directly in a serialized variant**, so the question is not a
repair detail: *should a declaration whose value contains a function be serializable at all?* Three
answers exist and they are different work — drop serde/Debug from these declarations; split each into
a serializable description plus a non-serializable realization; or keep the derive and make the
function field a named, resolvable reference (the `PrimitiveDefinition`-style identity move DESIGN
already contemplates elsewhere). The derive roster applies serde unconditionally to every record and
coproduct, so **any** type transitively reaching a closure fails — the population grows with the model,
not with the corpus.

**Method note, and I got this wrong first:** my initial scan for "records deriving serde while holding
a closure-bearing field" found exactly **one**, and I nearly reported that the hypothesis was refuted.
It matched only `pub field:` lines at struct-body depth and therefore missed **enum variant fields** —
`ValueRuntimeInterpreter { interpreter: Rc<ValueInterpreter> }` is one level deeper. The hypothesis
was right and the instrument was wrong, which is the same failure as reading a rustc noun at face
value: a negative result from an unvalidated scan is not evidence of absence.

**The operator question above is answered** (session `tidy-dove-648`, 2026-08-21): not one answer
for all twelve declarations. `ProducedDeclSupport` takes the third option named above — keep the
record serializable, make `render` a named resolvable reference dispatched through the
`scaffold_relation_rule_name` it already carries — because `TargetModel` is real per-language
configuration, not a runtime-only carrier. The interpreter-family majority (`RuntimeBehaviorInterpreter`
and its six payload types, `EffectIoEvalBundle`, `CompiledLexRule`, `PartialFunction`) takes the
first option, which is not new: `fn_field_derive_traits()` already answers it for the direct-field
struct case, and it is simply unwired for coproducts (`v1_emit_enum_derives` takes no
`has_fn_fields` parameter) and for enum-transitive reachability (`build_type_summary`'s enum branch
hardcodes `field_type_map: empty_map()` in `04_emit_info.dag`, so `type_summary_reaches_fn` cannot
see through a variant payload the way it already sees through a struct field). `InterpretationStructureWitness`
and the four remaining `target_model.rs` sites hold no function field of their own and are treated
as collateral of the above, not independent decisions, pending re-measurement.
[`t5b-closure-bearing-serde-debug-decision-2026-08-21.md`](t5b-closure-bearing-serde-debug-decision-2026-08-21.md)
has the full per-declaration table and the two repair handoffs.

## 16. The finding above the findings: a site count measures where the compiler pointed

**Corrected 2026-08-17, and the correction is the same class this section is about.** The first
version of this section published a pooled aggregate (37×) and median (64×) that **silently
included Root C — the row the sentence beneath the table said was excluded.** 167 sites and 1 unit,
carried over from an earlier draft whose population included C, surviving a change of population
that the prose recorded and the arithmetic did not. It also sized Root A at 142, the classifier's
gross bucket, while §13 of this document establishes **A = 133** after nine concrete `Nat` receiver
rows move to the repr family. Found in review, not by me. The pooled statistics are **deleted**
rather than repaired, for the reason in 16.2.

### 16.1 The per-root observations

| root | sites | authored decisions | ratio | what one decision is |
|---|---:|---:|---:|---|
| B1 algebra carrier — **WITHDRAWN as one root, see §18** | ~~509~~ | ~~1~~ | — | was: one closure-shape flag. It is at least three mechanisms. |
| K unsynthesized use-line | 132 | 1 | 132× | one collector that does not reach pattern / nested-argument positions |
| T7 `Hash` collision | 105 | 1 | 105× | one row in `rust_type_checkpoints` |
| A generic clone bound | 133 | 2 | 67× | two axes with complementary blind spots |
| D generic argument count | 116 | 2 | 58× | one checkpoint row + one alias-arity drop |
| T3 collection carrier | 107 | 5 | 21× | five mechanisms over two alias declarations |
| T5a Hash/Eq | 27 | 3 | 9× | three declarations |
| R1 wrap decision | 59 | 10 | 6× | ten types, fourteen bare occurrences |
| T5b serde/Debug | 44 | 12 | 4× | twelve demanding declarations |

Root C is not in this table and is not in any figure derived from it — it is owned elsewhere and I
did not characterize it.

### 16.2 No population ratio is claimed, and the reason is dimensional

**In these nine characterized specimens, diagnostic sites substantially outnumbered the
independently identified authored causes, by amounts ranging from 4× to 509×. No population-level
ratio is claimed.**

A median or aggregate over that column would require the denominators to be one kind of thing, and
they are not: one lookup row, one collector, one checkpoint row, two conceptual axes, five
mechanisms, three declarations, ten types, twelve demanding declarations. Averaging those is
inventing a common currency for work. Stating that the denominator is judgment — which the first
version did — does not license aggregating across the judgments; a caveat is not a unit.

A defensible unit would have an operational identity: **one independently disposable authored
decision, with one named authority, one disposition, and one acceptance result** — enumerated as
IDs, not asserted as a count. Until that exists the table above is a qualitative characterization
of nine specimens and nothing is derived from it.

### 16.3 Three grains, and which of them may appear before characterization

The claim this section originally made — that the ratio predicts fix shape *before anyone opens the
file* — is **withdrawn**. It cannot: the denominator is produced BY characterization, so it is a
handback result and never a dispatch input. Quoting a predicted ratio in a work-item title would put
an unverifiable number in the one place a fresh owner is most likely to trust it.

| grain | what it is | may appear at dispatch |
|---|---|---|
| site count | observed refusal surface under a stamped census | yes, with M |
| diagnostic-signature count | syntactic diversity of the rustc observations | yes |
| authored-decision count, and any ratio over it | post-characterization causal decomposition | **no** |

So a dispatch title carries raw census facts only — `Root K — 132 sites @ M=11 · N diagnostic
signatures · uncharacterized` — and no predicted shape.

**Terminology correction while restating this:** what the instrument computes is a **diagnostic
signature** (expected/found pair, receiver+method, trait+self-type, else message), not a cause. One
mechanism emits several signatures and one signature can conceal several mechanisms; §11 uses
"cause signature" throughout and the instrument definition there is what is authoritative.

### 16.4 What survives

The direction, which every specimen agrees on and which is the part that changes how work is
planned: **a root's site count measures where the compiler pointed, never what has to change.** A
burn-down quoted in sites is a statement about rustc's reporting density. That is worth saying
without a number attached to it, and the number attached to it is exactly what had to be retracted.

## 17. Two-arm provenance: three controls that run BEFORE the measurement

Established while standing up the after-arm service for Root B's `RustCorpusRepr` cut. The
measurement itself is pending; **these three controls are not** — each is complete on its own and
each closes a route by which a two-arm receipt reports a clean result while measuring nothing.

**17.1 Discriminate the binary, not its mtime.** The stale-binary hazard is usually answered by
checking that a rebuild happened. That is a proxy. On a self-hosting compiler the direct check is
available: the arms differ in named symbols, so read them out of the binary.

```
BEFORE binary, built at a6bceb6903, sccache disabled:
  RustCorpusRepr              9 hits     <- pre-cut emitter PRESENT
  FaithfulFreeMonoid          4
  HostNative                  5
  decl_file_realizes_natively 0          <- post-cut symbol ABSENT
```

The after-arm must invert this, and if it does not I stop rather than probe. This answers "does
this binary carry the change" positively, in both directions, without depending on a timestamp —
and a timestamp is what moves when a build recompiles something unrelated.

**17.2 Run the determinism control before you have a diff to explain.** Same binary, same sources,
`04_infer` re-emitted: **85 files, 0 differing.**

**Scope of that claim, stated because it is narrower than the sentence I first wrote.** The original
read "emission is deterministic on this path". The evidence is **one module, one repeat** — n=1 in
both dimensions. What it supports is: *this entry, re-emitted once under identical inputs, produced
identical bytes.* A single agreeing repeat cannot distinguish a deterministic emitter from a
nondeterministic one that happened to agree, and the other ten modules are unmeasured. It is enough
to remove nondeterminism as a ready explanation **for a diff in this module**, which is what it was
run for; it is not a determinism receipt for the emitter.

The sequencing carries more than the result. Run *after* an inconvenient diff appears, this is a
check you chose to perform on a result you did not like, and "that is just emission
nondeterminism" is a ready-made, authoritative-sounding way to make the diff not your problem.
Run *first*, **you have destroyed your own escape route before you know whether you will want
it.** Same discipline as writing the prediction down before the run, applied to the excuse rather
than to the claim.

**17.3 Vary the compiler; hold the emitted subject byte-identical.** The natural two-arm design —
BEFORE = binary+sources at merge-base, AFTER = binary+sources at branch head — varies the
compiler *and* the subject corpus together, so its diff cannot say which one moved the artifact.
An emitter change is both cases at once: it edits `.dag` authorities *and* lives in the binary,
because the emitter is generated Rust compiled into `gunbc`.

So the arms take **binary and source tree as independent inputs**, and the measurement is the
after-binary against the *merge-base* corpus. The failure mode this closes is specific: if the
branch also touches `src/v2` or `dag/`, that corpus change folds into the emitter delta and is
reported as emission change — attributed to the branch owner, by the measurer.

**And the shared property of all three:** each is a control on the *instrument*, not on the
subject, and each is cheap enough that its only real cost is remembering to run it before the
result exists rather than after.

## 18. B1 is not one root — the tighten-your-class audit run on my own largest bucket

**§11.3 sized B1 at 509 sites (27.2%) and called it the largest root in the live corpus. That is
withdrawn.** The class was assigned by keyword — rustc code in a set AND the signature *contains*
`CommutativeSemiring` / `Magnitude` / `Measure<` / `Semiring` anywhere. Tightening it to *the
carrier must be one side of an expected/found pair* leaves **54 of 509**.

Re-decomposed by mechanism, over the same M=11 census:

| mechanism | sites | share | moved by a repr flag? |
|---|---:|---:|---|
| operator on carrier (E0369) | 191 | 37.5% | **repr_fork: 191 / missing_trait_impl: 0** (§18.4) |
| derive: serde Serialize/Deserialize | 132 | 25.9% | no |
| REPR: carrier expected, integer literal found | 92 | 18.1% | yes |
| REPR: carrier vs another named type | 54 | 10.6% | yes |
| derive: Debug | 35 | 6.9% | no |
| other | 5 | 1.0% | — |

**At most 146 sites are the repr mechanism I named.** The 191 E0369 rows are classified in
§18.4: **all repr_fork, zero missing_trait_impl** within the B1 keyword filter — the
apparent ambiguity applies only to unfiltered E0369 (R1 `im::Vector` / `dyn Fn` sites are
missing_trait_impl but excluded from this bucket).

### 18.1 The 167 derive-shaped sites are ONE declaration, and it is underivable

Split by missing derive and by type:

```
96  serde::Deserialize      CommutativeSemiring<Magnitude>
36  serde::Serialize        CommutativeSemiring<Magnitude>
35  Debug                   CommutativeSemiring<Magnitude>
                            distinct type names: 1     Hash: 0     Eq: 0
```

The emitted declaration says why no derive roster change can move them:

```rust
#[derive(Clone)]
pub struct CommutativeSemiring<T> {
    pub add:  Rc<dyn Fn(T, T) -> T>,
    pub mul:  Rc<dyn Fn(T, T) -> T>,
    pub zero: T, pub one: T, pub _phantom: PhantomData<T>,
}
```

Two `Rc<dyn Fn>` fields — `serde` and `Debug` are not derivable on it in principle. So all 167 are
T5b's modeling-decision class **on one declaration**. Zero are T5a: no site in the 167 is missing
`Hash` or `Eq`, so the keyed-collection axis does not apply to any of them.

**A correction I issued to the T5 owner and repeat here:** I first relayed that "whoever holds T5 is
holding materially more than 27+44". The T5a half is wrong — that population is unchanged. The T5b
half is right in sites and misleading in shape: **167 more sites, one more type.** Which is §16
happening to my own correction — I reported a site count where the unit of work was one declaration.

### 18.2 What the failure was

I grouped by mechanism in every root except the largest, where I grouped by a keyword naming a
**type**. A type name in a diagnostic says the carrier was *involved*; it does not say *how*. The
dispatch brief that opened this lane said "group by mechanism, not by error code" — and the same
warning applies to grouping by symbol, which is what a type-name keyword is. The population was
large enough to hide three mechanisms inside one plausible name.

**Consequence for anyone measuring the repr cut:** a `RustCorpusRepr` change should be expected to
move the repr-shaped sites, at most 146 and possibly fewer — **not 509.** If a receipt shows far
less than 509 moving, that is this mis-sizing and not an underperforming cut.

### 18.3 The measured Root B flip, and one retraction

`smart-ibex-716` executed the flip in both directions. **What holds, measured:** 211 → 204 errors,
203 → 196 sites, with 195 common, 8 removed and 1 added; containment 0-of-18 disjoint.

**Retracted in full and recorded so nobody re-derives it:** a reading that 43 sites showed renderer
inconsistency, that 18 sites reassigned from B to R1, and that the two roots were fused. The
specimen behind it was read from a different module's artifact. Nothing from that reading reached a
carrier.

**What survives from it is worth more than the retracted claim.** The leaf name `Nat` denotes two
different types under two module authorities — `01_tokenize`'s is an enum, not the algebra alias —
so **a census pooled across modules is keyed on a name that is not unique over the population it
pools.** That is §18.2's failure one level deeper: grouping by a type name is unsound not only
because a name says involvement rather than mechanism, but because the same name may not denote one
type at all. Key on resolved type identity per artifact, never on the printed name, and allow a site
to carry more than one root.

### 18.4 E0369 operator-on-carrier — classified (`lively-ibex-709`, 2026-08-17)

Receipt: [`docs/probes/e0369_b1_operator_classification_2026-08-17.md`](../probes/e0369_b1_operator_classification_2026-08-17.md)

| classification | sites (M=11 §18) | sites (July 7-module bank, measured) |
|---|---:|---:|
| repr_fork | 191 | 112 distinct |
| missing_trait_impl | 0 | 0 |

**Mechanism, stated once:** every B1-keyword E0369 site is an operator or `PartialEq` derive
expansion on `CommutativeSemiring<Magnitude>` / `Measure<…>` emitted under
`FaithfulFreeMonoid`. Fixing it requires grounding the numeric tower (`RustCorpusRepr` /
identity-keyed checkpoint rows), not adding `Add`/`PartialEq` impls to the algebra stub.

The R1 E0369 population (`im::Vector`, `dyn Fn`, `*Interpreter` — 116 instances in the July
census) is **missing_trait_impl** but **outside** the B1 keyword filter and therefore outside
the 191. That is why the bucket-wide ambiguity does not survive filtering.

Per-site TSV (July bank): `docs/probes/e0369_b1_classification_2026-08-17/sites_classified_july_bank.tsv`
Repro script (M=11): `docs/probes/run_e0369_b1_classification.sh`

## 19. E0308 root partition — mechanism grain (`sharp-owl-720`, 2026-08-18)

**Dispatched question:** E0308 is the dominant emitted-Rust error class (~40–47% of coded errors
on floor modules, ~86–94 distinct sites per floor entry) and had **no partition at E0308-only
grain** — §11 sized all codes together. **Answer:** E0308 is **not one root**; it is **13
mechanism roots** at **408 distinct sites** (M=11, deduplicated), with **1555** diagnostic
blocks summed (**3.81×** inflation within E0308 alone).

| root | E0308 sites | % of E0308 | §11 owner |
|---|---:|---:|---|
| T7 — `Hash` / `Fnv1a64Structural` ↔ `String` | 99 | 24.3% | vivid-wren / checkpoint table |
| R1 — bare↔`Rc` wrap | 91 | 22.3% | bold-lark-722 |
| RESIDUE — unclassified pairs | 59 | 14.5% | misc (largest: Root D alias) |
| T2 — text carrier | 38 | 9.3% | unowned |
| T3 — collection vs `im` | 32 | 7.8% | unowned |
| B3 — `Nat` vs integer | 18 | 4.4% | eager-deer-389 |
| B2 — `Bool` vs `bool` | 17 | 4.2% | eager-deer-389 |
| RESIDUE-witness | 15 | 3.7% | tail, not July 18–23% bucket |
| R5 — duplicate type authority | 15 | 3.7% | unowned |
| C — Optional→`()` | 11 | 2.7% | gentle-dove-833 |
| B1-repr — algebra carrier | 6 | 1.5% | §18 repr-shaped |
| RESIDUE-diagnostics | 4 | 1.0% | July Root 1 dead |
| T4 — record as tuple | 3 | 0.7% | unowned |

**July E0308 bucket shares falsified again at this grain:** DIAGNOSTICS fork **4 sites (1.0%)**;
`Witness<_>` string absent — remaining witness-shaped pairs are **15 sites** with concrete
mismatches like `Witness<ExitOk>` vs `Witness<Rc<Outcome<…>>>`.

Full receipt:
[`docs/probes/e0308_root_partition_2026-08-18.md`](../probes/e0308_root_partition_2026-08-18.md).
Per-site TSV:
[`docs/probes/e0308_partition_2026-08-18/sites_classified.tsv`](../probes/e0308_partition_2026-08-18/sites_classified.tsv).
Measurement route and entry set are in the receipt's Method table.

## 20. E0277 root partition — trait × self-type grain (`bright-moth-92`, 2026-08-21)

**Dispatched question:** E0277 is the second-largest emitted-Rust class and had no partition at
E0277-only grain — §11 sized all codes together, and the July census
(`e0277_trait_bound_census_2026-07-26.md`) counted occurrences rather than sites. **Answer:** E0277
is **four mechanisms (five root labels) at 82 distinct sites**, **365** blocks summed over M=6
(**4.45×** inflation within E0277 alone), **zero unclassified**.

| root | E0277 sites | % of E0277 | disposition |
|---|---:|---:|---|
| T5b — serde/Debug demanded over closure-bearing values | 35 | 42.7% | dispatched (was §11.23, unowned) |
| A — generic parameter bound not emitted (`Clone` 25, **`Ord` 5**) | 30 | 36.6% | Root A lane; the `Ord` 5 dispatched |
| R3 — `Rc<dyn Fn..>` where an `Fn` bound is expected | 9 | 11.0% | dispatched |
| T7 — `Hash`/`Eq` on `Fnv1a64Structural` | 7 | 8.5% | blocked in tree, do NOT re-dispatch |
| T5a — `Eq` on `OccurrenceId` | 1 | 1.2% | same blocker |

**Three findings that change what someone should do next, none of which the by-code view showed.**

1. **The July census's ranking is falsified at site grain.** Its "dominant family" (generic `Clone`)
   is second at 36.6%; its family 2 self types (`Node`, `EnvironmentBindingKey`) carry **zero**
   E0277 sites today. No attribution is offered for the move — §16 applies.
2. **Root A's five `Ord` sites are outside the mechanism's expressible range, not gaps in its
   coverage.** `std.authorization_profile` `AudienceSet` declares `EnumeratedAudience { members: Set<P> }`;
   `Set<P>` realizes as `BTreeSet<P>`, which demands `P: Ord` the way `im::Vector<A>` demands
   `A: Clone`. The entire v1 supplemental-bound apparatus is **`Clone`-only** — a one-trait fixpoint
   (`v1_clone_bounded_type_params`) with no arm that can emit any other bound. So these are an
   executed specimen of exactly what `trait_derive_emit_item_clone_bound_contract_fork_note`'s
   dissolution clause exists for: v2's `target_derive_supplemental_generic_bound_contract` is
   per-derive-impl and cited, v1's is per-type and Clone-shaped, and the requirement side now has
   evidence, not just the fork-hygiene side. That note's warning still binds — the wire-through
   changes the grain, and unioning per-derive requirements onto the declaration reproduces v1's
   over-constraint under v2's name.
3. **T7/T5a is characterized AND blocked, in tree, already.** `v1.trait_derive_emit`
   `map_key_alias_hop_gap_note` names this population, records that the obvious alias-following fix
   was attempted, measured and reverted (it drags `Int`/`Nat` to map-key positions and diverges two
   stage0 files), and states its dissolution as a realization binding keyed on `DeclarationRef` —
   the same threading the identity-keyed `lookup_checkpoint` cut waits on. Anyone sizing an
   E0277 lane should subtract these 8 rather than staff them.

**Method correction worth carrying, because it cost a full remote build cycle.** A comparison set
must be **one dispatch**. Three parallel dispatches pinned with `PROBE_EXPECT_BASE_SHA` taken from an
earlier dispatch's resolved HEAD all died on `SAME_BASE_REFUSE` — `ctrl-build --remote` resolves the
repo-root HEAD *when the run starts*, and main moved twice inside the window. Capture `HEAD` inside
the dispatch and export it there; then "one tree" is a property of the run. The pin worked exactly as
specified: it stopped the line instead of yielding six numbers from three trees.

Full receipt, including the controls (the exact-100 truncation check against a 120-error rustc
control, and the classifier's known-positive RESIDUE arm):
[`docs/probes/e0277_root_partition_2026-08-21.md`](../probes/e0277_root_partition_2026-08-21.md).
Per-site TSV:
[`docs/probes/e0277_partition_2026-08-21/sites_classified.tsv`](../probes/e0277_partition_2026-08-21/sites_classified.tsv).
