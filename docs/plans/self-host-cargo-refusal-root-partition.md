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
   is its output and is listed in `gunbc.stage0_emit_plan_generated`. Probe the `.rs` for fast
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
> [Root B primitive repr fork](../probes/root_b_primitive_repr_fork_2026-08-16.md).
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
> | diagnostics citing `CommutativeSemiring<Magnitude>` | 342 | **0** |
> | `expected bool found Bool` | 11 | **11** |
> | total coded errors | 652 | **773** |
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


## 10. THE CLOSURE-SHAPE FORK — THREE confirmed instances (2026-08-16)

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

**THIRD INSTANCE — confirmed, and it has its own full entry below** (`### §10 — the third
branch`, written by `gentle-dove-833`, who measured it). Not restated here: two accounts of one
finding in one document is the dual authority this surface exists to prevent, and we produced
one by both writing it up. Theirs is the authority.

What this section adds, and only this: the mechanism was **independently verified in tree** by
`smart-ram-730` (the sentinel insert in `derive_variant_to_enum`, the value-ignoring
`map_contains_key` read in `is_value_variant_type_arg`), and it is DESIGN's **state-space
conflation** by name — the map's value carries three states (absent / a real owner / ambiguous)
and the consumer collapses every present case into one. That framing is what makes it a
closure-shape branch nobody authored as one, and it is why the meta-root is promoted from
hypothesis to three confirmed instances.

**Not claimed:** that the seed closure lacks these collisions. That is the next measurement and
it would make this instance structurally identical to Root B rather than merely analogous.

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
