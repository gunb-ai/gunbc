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

`docs/probes/curated_cargo_frontier_probe_sweep.tsv` — banked `941e8034862`, **2026-07-26**.
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

| root | share | owner | status |
|---|---:|---|---|
| **A — generic trait bounds not emitted** | ~590 | smart-ram-730 | taking now |
| **B — primitive representation fork** | ~196 | unassigned | |
| **C — Optional collapses to `()` in nested type-arg position** | ~169 | gentle-dove-833 | emitter already instrumented |
| **D — generic argument count** | ~76 | unassigned | |
| **E — unreachable patterns** | 126 | unassigned | lint-class, lowest value |
| **F — E0282 type annotations needed** | 55 | unassigned | may dissolve with A |

## 8. The partition, from cause signatures (this supersedes the by-error-code view)

**`docs/probes/canonical_seven_cause_signature_attribution_2026-07-28.tsv`** already exists in
tree and answers the question §4 was still hypothesizing about. It groups by CAUSE SIGNATURE
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
