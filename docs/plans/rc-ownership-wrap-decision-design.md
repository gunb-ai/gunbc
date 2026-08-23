# Rc-ownership wrap-decision predicate — design + deep-emitter gate derisk

Status: LANDED (predicate + translate routing + UseSiteVerdict enrollment), 2026-07-20 verify-by-execution (valiant-dove-723). Re-verified 2026-07-23 (fierce-crab-777: claim_batch `wrap_decision_predicate_witness_holds` 8 sub-witnesses + gate smoke green on main). Original design: 2026-07-16 (bold-seal-166). Plan carrier (`v1_deletion_plan.dag` lane_state rows) is batched in #6909 — not this PR; the mark on the carrier is the plan file, this doc is the design receipt only.
Parent: sharp-bee-290 (Weak → Strong Self Host, Wave 1→4).
Displaced cost: unblocks self-emit for Rc-heavy core compiler modules (`04_infer`, `06_translate`, `05_emit*`, … — each seed-emitted with 100+ `Rc<` sites today) without the latent §5 fail-open that silently wraps every `shared_types` member in `Rc<T>`.

Related lanes (orthogonal axes — do not conflate):
- [emitter-ownership-defork.md](emitter-ownership-defork.md) — clone-vs-move at **value** use sites (`UseSiteVerdict`, `make_decision`).
- [s2-v2-self-emit-direction.md](s2-v2-self-emit-direction.md) Track C4 — `Rc<T>` **type** wrapping derived from the model.
- `program_partition.dag` `partition_derive_target_for_emit` — per-module **user-type** catalog augmentation (`ReferenceLayerOwned` default).

## Problem (§5 construct finding)

Wave 2 `use_site_verdict` behavioral pilot recorded `ssuv_ownership_rc_default_fail_open_finding` in `dag/tools/self_host_use_site_verdict_behavioral_transport.dag`:

> The Rust emitter silently applies default `Rc<>` wrapping on `Node` / `UseSiteVerdict` / fn params and returns with **no typed ownership refusal** (`target_use_site_ownership_lookup_miss` unwired).

Benign for immutable-value pilot modules (zero emit diagnostics). **Latent fail-open** for mutation/aliasing-bearing compiler modules in the wide fan-out: a carrier that should stay owned at a site, or a missing catalog row that should refuse, instead gets the v1 `shared_types` heuristic (`render_rust_shared_type_if_needed` in `05_emit_rust.dag`).

Band A flip policy (`frontier_band_a_emit_readiness.dag`) already blocks `parse_engine_hooks` and `discovery_enumeration` until this gate lands.

## R1 — the emitted-crate census, and the correction it forces to the paragraph above (2026-08-17, bold-lark-722)

The paragraph above says the fail-open is that the emitter *wraps every `shared_types` member*. Measured against a real emitted corpus that is **not what happens**, and the difference decides what a fix may be. This section records the correction, the located root, the two facts a fix has to respect, and what was built in consequence.

**2026-08-23 reporting correction:** per-code partitions are projections, not mechanism membership: one mechanism can surface under several rustc codes, while M=1 closure boards overlap and the published receipts span different revisions. They therefore supply vocabulary and located observations but no board shares, ranking, or completion census. The population-independent producer trace is [`r1_reference_layer_producer_2026-08-23.md`](../probes/r1_reference_layer_producer_2026-08-23.md); cross-code membership is owned by the cross-code classifier and is not re-derived here.

### What was measured, by whom, and on which emitter

smart-ibex-716 built an emitted-crate census at `M=11` compiler-module entries (`05_emit`, `06_translate`, `04_infer`, `03_ingest`, `emit_host`, `01_tokenize`, `materialization_carriers`, `emit_module`, `03_normalize`, `program_partition`, `05_eval`; head `5e1a73fa33`), using the `docs/probes/curated_cargo_probe_one.sh` invocation contract with `cargo check --message-format=json` so the expected/found pair comes from the span label rather than a text grep. **55** E0308 `Rc` mismatches survive dedup to distinct `(file, line, column, code, signature)`: **28 under-wrapped** (`expected Rc<T>, found T`) and **27 over-wrapped**. **54** of those appear in the `03_ingest` closure, the one tree kept on disk, and **53 of those 54 sit at call-argument position** (1 record literal, 0 signatures). The three totals are three denominators, not an arithmetic error, and one site (`v2_compiler_parse.rs:582:12`) carries **both** signs, so 55 diagnostics do not mean 55 distinct positions.

**That denominator is keyed on rendered diagnostic text, which is a method hazard with no measured cost to this count.** A proposed `+18` correction to the 55 was published here and is **withdrawn**: `smart-ibex-716` read a `pub type Nat = …` specimen from one module's artifact and applied it to 18 sites in `01_tokenize`, whose `Nat` is an *enum* (`v2_std_nat::Nat`) and whose closure contains no such alias at all. Those sites did not move across a carrier deletion because nothing about their type changed, and `Rc<enum Nat>` against `i64` is a full carrier disagreement rather than a wrap decision — so they belong where the census already put them. The `43`-site "one mechanism split by inconsistent rendering" reading is withdrawn with it: the two spellings were two *different* types sharing a leaf name, not one type printed two ways. **The 55/28/27 stands unaltered**, and it is recorded this way rather than quietly reverted because a count that survived a challenge is worth more than one nobody tested.

The hazard the episode does establish is **real, unmeasured, and not repairable by alias expansion**: the leaf name `Nat` denotes two different types under two different module authorities, and a diagnostic prints both identically. **A census pooled across modules is keyed on a name that is not unique over the population it pools** — so the sound discriminator is resolved type identity *per artifact*, not merely expanded realization. Its extent here is unknown and no number is claimed. None of it bears on the mechanism below, which never reads a printed type name.

**The emitter under measurement is frozen v1, not v2.** Three independent reads, none of them inference from the emitted files' `v2_*` names (those name the *source* modules, which are v2 `.dag` files): `src/v1/stage0/src` contains `v1_compiler_emit_rust.rs` and **no** `v2_compiler_translate.rs`, so `v2.compiler.06_translate` and `wrap_decision_gate` are not compiled into the `gunbc` binary at all; `build_shared_types` appears in exactly two stage0 files, `v1_compiler_emit_rust.rs` and `v1_compiler_infer_emit_info.rs`; and `cli_run.rs`, the `compile` subcommand's own module, imports `crate::v1_compiler_emit_rust` directly. So `gunbc compile --target rust` is the v1 seed emitter end to end, and every one of the 55 sites is v1 output. This retires, for this census, an earlier cross-lane inference that joined the v2 `rust_sg_rc_use_site_ownership_catalog` position fork to these numbers: that fork is real and is described below as a *separate* finding on the v2 path, but it was dormant while these sites were produced. A fact that fits is not a fact that applies.

### The root: a type-level fact consumed through occurrence-level keys

`build_shared_types` computes membership **once per type**, from the type summary — `StructRepr`, or `EnumRepr` with `unit_only == false`, minus grounded-coproduct-native aliases and `is_type_constant`. That is a property of the *type*.

It is then *consumed* at every occurrence through keys and predicates that are properties of **that occurrence's syntax**, by three seams that do not share an authority:

1. **Signature.** `v1.compiler.emit_rust` `render_rust_fn_sig_type` selects among six arms on a cascade over `type_node_has_value_variant_arg`, `connective`, children count, `is_container_type`, `rust_fn_sig_peel_closed_alias`, `rust_fn_sig_preserves_authored_alias_leaf`, whether `generic_param_names` is non-empty, and the presence of an `__applied_type_args` property. Three arms wrap; three delegate to `render_rust_decl_type` or `render_rust_type_with_applied_binding`, which never wrap.
2. **Struct field.** `v1.compiler.emit_rust` `emit_struct_field_from_child` wraps iff `rt_child.return_cardinality != CardOptional` **and** `set_contains(shared_types, authored_name_at(rt_child))` — the authored name *at that occurrence's resolved type node*, which is neither the type's identity nor necessarily the name membership was computed under. (Recursion is handled on the preceding, independent `needs_box_wrapping` arm, for sizedness; that arm is not part of this defect.)
3. **General type rendering.** `v1.compiler.emit` `render_node_type` recomputes `shared = set_contains(shared_types, authored_name_at(n))` and applies it on its Disj, Conj-named, Conj-container, bare-leaf, map and single-child arms, while the `type_annotation != none` refined arm and the tuple arms return unwrapped.

So **one type receives several wrap answers**, selected by how each occurrence happens to be spelled and resolved. Which of those selectors actually fires in the live corpus is a separate question from which ones exist, and the sweep below answers it: seam 2's `authored_name_at` key is the one demonstrably missing, at fourteen field occurrences across ten types. Seams 1 and 3 are named here because they are the same class and are writable, not because this census pins a failure on them — an enclosing-item-genericity reading of seam 1 was predicted and refuted (below). That is §3's single-authority violation in its plainest form — one fact, several representations — and the §5 half is that **no arm refuses**: `render_rust_shared_type_if_needed` returns the unwrapped rendering on a miss, silently, so a disagreement is emitted rather than located. Per §4b this class sits **below the ladder** on the source→Rust-emission path: not mitigatable, not detected, silently wrong until rustc happens to catch it. rustc is not the guarantee; it is the accident that makes this one visible.

### Why the counts split by sign, and why they are per-type stable

Per type the answer is mostly stable, so this is two largely disjoint populations rather than one rule oscillating: over-wrapped only — `ScopeRoster` 6, `SubjectRoster` 6, `ConsumerRequirement` 6, `SpanIndex` 7; under-wrapped only — `DecimalDigitsStep` 4, `Edge` 2, `DeriveGrammarRelationTokensProgress` 2, and a long tail of singletons.

Membership is **not** the discriminator, and this is the measurement that kills the original problem statement: `ScopeRoster`, `SubjectRoster`, `ConsumerRequirement`, `SpanIndex` **and** `DecimalDigitsStep` are all shared types, and they split 28/27 by sign anyway. "The emitter over-wraps `shared_types` members" cannot explain a population that is entirely shared and yet disagrees in both directions.

#### How narrow the inconsistency actually is

Two `03_ingest` emits, produced independently on two instruments, were each swept for every type occurring in struct-field position and in `fn` signature position, counting bare against `Rc<…>` occurrences per type — including occurrences **nested inside container types**, which the first pass of this sweep missed by keying only on a field's outermost type. The overwhelming majority of types are perfectly consistent; the inconsistent population is **ten types and fourteen bare field occurrences**, and the two instruments agree on it type for type, count for count, and file-and-line for file-and-line:

| type | sig `Rc<T>` | sig bare | field `Rc<T>` | field bare | a bare field occurrence |
|---|---|---|---|---|---|
| `Nat` | 197 | 0 | 1 | 4 | `v2_std_verdict.rs:138` — `Nat` |
| `Finding` | 26 | 0 | 0 | 1 | `…copy_analyze.rs:602` — `Rc<Vec<Finding>>` |
| `TerminationProof` | 16 | 0 | 1 | 1 | `v2_compiler_infer.rs:86` — `Rc<Witness<TerminationProof>>` |
| `PortReading` | 16 | 0 | 0 | 1 | `…copy_analyze.rs:430` — `Rc<Vec<PortReading>>` |
| `SpanIndex` | 10 | 0 | 1 | 2 | `v2_compiler_parse.rs:80` — `SpanIndex` |
| `Determinism` | 10 | 0 | 0 | 1 | `v2_std_determinism.rs:17` — `Determinism` |
| `ScopeRoster` | 4 | 0 | 0 | 1 | `…standing_intent.rs:75` — `ScopeRoster` |
| `NarrowingReason` | 3 | 0 | 0 | 1 | `…standing_intent.rs:80` — `Rc<Vec<NarrowingReason>>` |
| `SubjectRoster` | 2 | 0 | 0 | 1 | `…standing_intent.rs:76` — `SubjectRoster` |
| `ConsumerRequirement` | 1 | 0 | 0 | 1 | `…standing_intent.rs:78` — `ConsumerRequirement` |

Every row has the **same shape**: wrapped at every signature occurrence without exception, bare at one or more field occurrences, and **zero bare signature occurrences anywhere in the population**. So R1's structural cause is not a broad mis-rule over positions: it is **fourteen bare field occurrences across ten types, where `emit_struct_field_from_child`'s `set_contains(shared_types, authored_name_at(rt_child))` misses while the signature path hits.**

Two refinements the sweep forced, both recorded because they change what a fix must cover:

- **The miss also happens at element position inside a container field**, not only at a field's own head. `Rc<Vec<Finding>>` has the container wrapped and the element bare, while every signature mentioning `Finding` says `Rc<Finding>`. A fix that reconciles only a field's outermost type leaves `Finding`, `PortReading`, `NarrowingReason` and `TerminationProof` untouched.
- **`Witness` is excluded from this population despite matching on a naive read.** It carries 65 bare *signature* occurrences, which no other row has; its cause is the checkpoint-table row asserting a bare target for a generic declaration, a different root. Admitting it would give this population a second mechanism.

The element-position rows also reclassify work across roots: four diagnostics previously filed under a separate "vector wrap/element shape" bucket are this mechanism at element grain (`expected Rc<Vector<Rc<PortReading>>>, found Rc<Vector<PortReading>>` and three siblings), and they show **both** directions at element grain, which is the one-defect-both-signs reading reproducing at a second grain.

That narrowness is the finding, and it settles a question left open above: **this is one defect producing both signs, not two mechanisms.** A signature that returns `Rc<T>` feeding a bare field is the **over**; a bare field read into a parameter typed `Rc<T>` is the **under**. One inconsistency, both directions — which is why a fix that makes field membership agree with signature membership should move both counts together, and why a fix that adjusts a position rule instead would not.

Two earlier position-based readings are retired by this sweep and are recorded as retired rather than deleted, because both were held with confidence and one of them was authored into a doc section: that signature position wraps while field position does not (measured across all shared types in one tree, plain struct fields run 121 wrapped against 5 bare — fields overwhelmingly *do* wrap), and that a shared type inside a generic `fn` or generic struct comes out bare (refuted at 17 of 17 generic signature occurrences and 6 of 6 generic struct-field occurrences, all wrapped). The first was drawn from a sample selected by the failing sites themselves; an error census says where disagreement occurred and never what the majority behaviour is. The second was a prediction from reading the arm cascade, offered for falsification and duly falsified — what survives is that the cascade *has* non-wrapping arms, not that enclosing-item genericity is what selects them.

`OccurrenceIdAllocator` remains the negative control: not a shared type, bare in both positions, **zero** R1 sites. A type with one answer has no disagreement to have.

### The discriminating specimen

One line carries both directions and shows the mechanism without reading the emitter at all:

```
v2_compiler_parse.rs:78    pub struct ParseProvenanceState { pub alloc: OccurrenceIdAllocator, pub index: SpanIndex }
v2_std_provenance.rs:61    pub fn span_index_empty() -> Rc<SpanIndex>
v2_std_provenance.rs:87    pub fn span_index_merge(base: Rc<SpanIndex>, incoming: Rc<SpanIndex>) -> Rc<SpanIndex>

v2_compiler_parse.rs:582   index: span_index_merge(base.index.clone(), incoming.index.clone()),
```

`base.index` is a field, emitted bare `SpanIndex`, passed into a parameter emitted `Rc<SpanIndex>` — the **under**. The call's `Rc<SpanIndex>` result is assigned into the bare field `index` — the **over**. One expression, one type, two seams, opposite signs. Any candidate fix should be argued against this line first and the bulk second.

### Separately: the v2 path carries its own position fork (not the cause of the above)

On the v2 emit path, which produced none of these sites, `rust_sg_rc_use_site_ownership_catalog` gives `Diagnostics` and `Node` **opposite** layers by position — `OwnershipAtBindingProjection → ReferenceLayerRc`, `OwnershipAtFunctionParameter → ReferenceLayerOwned` — while `TargetOwnershipUseSite` has exactly four inhabitants and **none of them is a call argument**, and `v2.compiler.06_translate` `translate_coerced_with_atom_realization` supplies `OwnershipAtBindingProjection` unconditionally for the generic node fold. So on that path an argument is emitted with the producer-side layer and the callee-parameter question is never asked. Recorded here because it is the same class one authority over and it will produce the same census when the emitter flips; it is **not** evidence for anything measured above, and the two must not be merged into one burn-down.

### Two constraints on any fix

**The fix site is inside frozen v1.** Step 3 of the implementation sequence below already records that v2 emit routes through `wrap_decision_gate` with zero `shared_types` references under `src/v2`, and that `gunbc compile --target rust` keeps the seed `shared_types` **until v1 delete (S3)**. Repairing the three seams in `v1.compiler.emit_rust` / `v1.compiler.emit` is therefore completion work on the structure scheduled for deletion — DESIGN §3's consequences (1) and (2) exactly: improvements that die with X, and optimizations Y is later forced to negotiate with. Fixing it in v2 instead cannot be validated against this census, because the v2 emitter did not produce it and cannot be run over that corpus today. **This is an operator/manager disposition, not an implementer's call.** Three options were put: (a) a declared bounded exception to the freeze, (b) treat the sites as a *specification* for the v2 emitter with synthetic discriminating controls in both directions and leave the v1 image red, or (c) fold R1 into the v1 deletion lane as quarry.

**Ruled (b)** by smart-ram-730, 2026-08-17. (a) was **refused**, on three grounds worth keeping because they generalize: DESIGN §5 puts an exception verdict *outside* the diff and outside the manager, so a manager authoring their own approval is the same forgery as an author authoring it; the frozen-X carve-out requires that no Y can hold the boundary, and the defect is a concept v2 is missing rather than an independent v1 bug; and consequences (1) and (2) apply verbatim to a renderer scheduled for deletion. (a) was escalated to the operator as an open question rather than decided silently. (c) was rejected on the ground that R1 is not un-actionable — it is a specification for a missing concept.

**What (b) costs, declared rather than discovered later:** the v1 image stays red on these sites, the burn-down is **unmeasurable** until the emitter flips, and the after-measurement is synthetic-control-only. That is a real coverage gap in the acceptance evidence, stated here as a declared one.

### What landed under (b)

`v2.compiler.wrap_decision` `wrap_decision_flow_gate` — the flow, not the position, as the decided unit. It projects `wrap_decision_gate` at the producer site and at the consumer site through the **same** authority — one function over one catalog, so no second *opinion* exists to drift — and refuses — typed, located, naming the carrier — when the catalog's own answers for the two positions differ. `wrap_decisions_equal` compares reference *layers*, not merely by-value against by-reference.

Precisely: this is **two traversals of one catalog**, not one traversal. What it rules out is a divergent authority, not a repeated read. The repeated read is a real cost shape — two folds where a per-carrier policy built once would serve both ends — and it is named rather than waved off as small, per §6. It is not fixed here because that policy carrier is the same carrier change the next rung needs, and building half of it now to save one fold would fork what the transition work has to own; the trigger is that carrier landing.

**Deliberately not built: a fifth `TargetOwnershipUseSite` inhabitant.** Adding `OwnershipAtCallArgument` would be a fifth position opinion about a defect whose cause is that every position already holds its own. A missing inhabitant is the right fix only when the missing position is the one that should decide; here no position should decide alone. This is also why *53 of 54 at call-argument position* does not locate the defect: that is where rustc points, and the cause is in the field declaration.

**Rung (§4b), stated with its distance to the ceiling.** This moves the class from silent wrongness — outside the ladder, emitted and compiled and wrong — to **mitigatable**: the line stops with a located refusal. It does **not** reach structural impossibility and must not be read as doing so; the invalid state stays writable, because a catalog may still state different layers at two positions and this function's only answer is to refuse the flow. The next rung is a modeled layer **transition** — `TargetValueExpression` carrying its current layer plus a total raise/lower/refuse over `(from, to)` — at which point an agreeing flow is *derived* rather than checked and the refusal narrows to genuinely unconvertible pairs. That is a carrier change, and model-before-implement puts it ahead of the pipeline edit rather than inside it.

**Not wired into `06_translate` by this change.** The authority and its executing controls land; wiring the consuming seams is a separate motion with its own evidence.

**The miss refuses at this authority; the class stays open.** A missing use-site ownership catalog and an absent value-semantics bundle both reach `outcome_rejected` with a typed located diagnostic — absence-as-no-facts has no arm here, and `wrap_decision_flow_missing_row_refuses` drives that arm rather than inferring it. What that does **not** do is close the class: absence-as-no-facts remains writable at other ownership lookups, including a live one in frozen v1 where `map_get(emit_info.ownership_index, …)` answers an `Absent` with `empty_set()` and no refusal — structurally the same shape as `render_rust_shared_type_if_needed` returning the unwrapped rendering on a miss, and an under-wrap producer, which is 28 of the 55. Closing the class means making ownership-absence *unrepresentable* rather than refused — the same modeled layer carrier named as this row's next rung — so widening refusal to N further lookups would add N validations where construction is the answer, which is more of §5's patch, not less. Stated here so the rung is read as what it is: one authority refusing, not a class walled.

**Evidence, by execution.** Six new witnesses in `wrap_decision_predicate_test.dag`, green, alongside the six pre-existing ones unregressed. Both error directions are drawn from the live rust catalog by varying only flow direction over `Diagnostics`, whose rows already disagree by position (`OwnershipAtBindingProjection → ReferenceLayerRc`, `OwnershipAtFunctionParameter → ReferenceLayerOwned`): binding→param is the over shape, param→binding the under shape, and both must refuse.

Two mutations were executed rather than asserted, and the profile is **not uniform**, so it is reported as measured:

| mutation | RED | GREEN |
|---|---|---|
| `wrap_decision_gates_agree` forced `true` | over-wrap, under-wrap, distinct-layer | missing-row, both accept controls |
| `wrap_decisions_equal` made layer-blind | distinct-layer only | everything else |

The missing-row witness therefore does **not** discriminate this reconciliation — it is a regression control proving an underlying catalog miss still propagates through the flow gate rather than being swallowed by it. Calling it a disagreement control would be rung inflation. The distinct-layer witness earns its place from mutation two: `Rc` against `Box` is a real disagreement that a by-value-versus-by-reference comparison calls agreement, and every other witness passes under that bug.

### Two-arm artifact diff, and both halves of the prediction that failed

The claim "this changes no emitted output outside its own module" is the kind that is comfortable to assert and cheap to test, so it was tested. Two emissions of the **same entry** (`03_ingest`), same command, same source roots, same `gunbc` binary — the binary is the compiler and the edits are its `.dag` input, so it is held constant by construction — differing **only** in the commit. `M` is therefore fixed by construction; a corpus census at a different `M` is the right before-arm for a corpus-wide claim and the wrong one for a change-detection question.

The prediction was written before the run, in both directions: `v2_compiler_wrap_decision.rs` **would** change (gaining the flow-gate functions, losing two note constants); every other file **would not**.

The whole artifact was diffed — all 177 files, no grep for the shape believed touched, since the hazard is precisely a second consequence the author did not predict. Result: **2 files differ, not 1**, and both halves of the prediction were wrong in detail.

| file | predicted | actual |
|---|---|---|
| `v2_compiler_wrap_decision.rs` | +4 fns, −2 note constants | +45 lines (4 fns), −1 line — a **glob import** `use …WrapDecision::*` replaced by explicit imports. The note constants are absent from both arms — see the retraction below. |
| `v2_std_cross_tree_resolution.rs` | unchanged | **2 `pub use` lines reordered** — a module this change never touched |

**RETRACTED — the second file is excluded from attribution, not explained.** An earlier revision of this receipt read the reorder in `v2_std_cross_tree_resolution.rs` as *deterministically caused by this change*, on the strength of a `B vs B′` repeat-emit control returning 0 differing files. That inference is withdrawn. `smart-ibex-716` ran the pure null — same binary, same SHA (detached worktree at `a6bceb6903`), fresh output dir per emission, `--dependency-pool-index primary-precedence` passed, 177 files both times — and obtained **the same file, the same shape, and the same two lines**: `pub use crate::v2_std_qualified_name::{QualifiedName}` and `…{qualified_name_from_dotted_string}` at a different position, with byte count, line count and sorted content identical. A phenomenon the null produces on its own cannot be attributed to a change by a control that merely failed to observe it; two agreeing repeats do not establish that a sometimes-moving thing never moves. The moved pair here is *the same pair*, so the two phenomena are not separable, which was the one test that could have saved the finding.

Recorded because it is the error this receipt elsewhere warns against: a control can only accumulate failures to refute, so its verdict must be stated as `n`, not as proof. The repeat count in this environment is **n=4 pairs at 0 differing** (`A` vs an SHA-pinned re-take of `A`; `B` vs `B′`; `B` vs `B″`; `B′` vs `B″`), and the closure **does** contain the unstable file, so this is not the vacuous zero. It clears nothing: the file's flip rate was later measured at **4 of 12** on a twelve-emission run, so four pairs miss a genuinely unstable file often, and no repeat count is offered here as a clearing argument. What excludes the file is **membership in the known-unstable set**, not repetition. DESIGN's `v2.std.determinism` open thread names the mechanism a re-export list would take: a Map projection over host-unspecified traversal order, whose generic aliases the determinism lens cannot see.

The differing files were then re-read with the two line-grain instruments the lane converged on, and the second one **misreported this branch's own subject file**. The per-file sorted-multiset test separates them correctly: `v2_compiler_wrap_decision.rs` is a real content change, `v2_std_cross_tree_resolution.rs` is multiset-identical — a pure reorder. The common-line **relative-order** test, which is meant to subsume it, reported `REORDERED` on *both*. It is wrong about the subject file: that file is a pure **addition** of four functions, and adding code adds copies of high-multiplicity boilerplate lines (`}`, `},`, `} else {`), which a greedy multiplicity cap mis-pairs — so the surviving content lines appear transposed around the mis-paired braces. The verdict only resolved to `SAME-ORDER` after excluding that boilerplate (163 → 163 → 117 → 113 common lines across successive filters), while the genuinely reordered file stayed `REORDERED` at every level (226 → 163 → 144). Both the planted-transposition positive control and the pure-addition negative control behaved correctly at every level, so **the instrument passes its own validation while giving a wrong answer on a real file** — its failing arm is a false *fire*, not a false clear, and it is invisible to synthetic fixtures because a planted transposition in an unmodified file adds no multiplicity. The boilerplate filter used here is a heuristic that can itself hide a reorder *of* boilerplate; the principled form does not depend on a filter at all. The instrument this conclusion rests on is the **diff-based** one: a reorder exists iff some line is both *deleted and added* by an alignment (`SequenceMatcher(autojunk=False)`, deleted ∩ added by min-count). It asks the question directly instead of extracting a common subsequence and comparing two orderings, so it is immune to the whole multiplicity family — every earlier variant was at the mercy of *which copies* got extracted. It reports `SAME-ORDER` for the subject file and `REORDERED` for the re-export file, and it **names the moved lines**: `pub use crate::v2_std_qualified_name::{QualifiedName}` and `…{qualified_name_from_dotted_string}` — one occurrence each. Naming a mover at all requires an argument, because a transposition admits two alignments and a symmetric one has no fact of the matter about which side moved. The operational test is **run the mover set in both directions**: agreement names the movers, disagreement *is* the observable signature of a symmetric transposition, where the honest output is the unordered pair and printing one confident name for an arbitrarily broken tie would be fabricated plausible output inside a measuring instrument. Here forward and reverse return the same two lines, so the naming stands. It stands as a **wager, not an observation**: the two groups are unequal — a block of *two* `qualified_name` re-exports crosses a block of *four* `cross_tree_import_model` lines — and minimality is a cost argument, so if the emitter in fact moved the four-line block the instrument names the two-line group and is confidently wrong. Read it as *the likelier mover*; only the equal-size refusal carries no wager. Two weaker readings taken along the way are recorded only as the route: the boilerplate heuristic (which can hide a reorder *of* boilerplate) and the unique-valued restriction (which cannot see a reorder of *repeated* lines at all, so it is a cross-check and never a clearing instrument). A first-divergence index is not used as evidence here — the planted positive control diverges at the same index as the real one, purely from where it was planted.

What survives: **the first file's 4-file/8-line substitution half is untouched** — it never rested on the reorder. And the argument the whole-artifact diff was run for survives in weakened form: 177 files, 2 differ, and the one differing file outside the change's subject is a member of the closure's known-unstable set, so it carries no information either way rather than being counted as a caused consequence.

What this buys is a positive argument rather than an absence: 177 files, 2 differ, 46 lines total, every other byte identical — so if some path had keyed on emitted text this change perturbed, it would have had to surface as some *other* emitted movement, and there is none. A targeted count of the shape believed edited could not have supported that, and would have missed the second file entirely.

**Retracted from this receipt: any claim about whether prose rows emit.** An earlier revision read the note constants' absence from both arms as evidence that a `data NAME_note: String` row emits nothing. That is the vacuous-closure error this section elsewhere warns about, committed here against its own author: the prose rows existed only on an intermediate commit, so **neither arm ever contained one**, and an absence measured over a closure that never held the shape proves nothing about the shape. Worse, the general form is independently known false — another lane watched a `_note` row appear in generated `.rs`. Whether a prose row emits is a **per-site fact requiring its own two-arm receipt**, and this receipt does not carry one. The prose-to-annotation conversion in this branch is therefore justified by §4c modeling debt alone, which was always its only real argument.

A related discipline that follows, for anyone converting prose rows: do not convert on a PR whose emission measurement is already banked, since the conversion may itself be an artifact change that invalidates it.

Two hazards checked and not applicable here, recorded so the next reader need not re-derive them: no `string_contains`/substring predicate over emitted content appears anywhere in this diff, so the emitter-self-match class has no subject; and the emission logs carry real output (`0 blocking, 545 advisory`) rather than a bare exit status, so the exit-0-without-a-compile class does not apply.

**The acceptance condition is two numbers, never one.** The populations are largely disjoint and oppositely signed, so a fix validated on a sample drawn from one direction will look correct, move about half the population, and make the other half worse. Any candidate must carry a discriminating control **in each direction**, and the after-measurement must show **both** counts falling, reported separately. 28→0 while 27→40 is a regression wearing a burn-down.

### Status of the reproduction

The 55-site census and its direction/position breakdown are smart-ibex-716's, on their instrument, and have **not** been independently reproduced — reproducing them needs the `cargo check` half, which was not run here. The emitter-identity reads and the three located seams are this session's, read against `src/v1/05_emit_rust.dag` and `src/v1/05_emit.dag`. The 928-type consistency sweep and the six-type table are this session's, on an independent `03_ingest` emit, and agree with the other instrument where the two overlap (`ParseProvenanceState` at `v2_compiler_parse.rs:78`, `SpanIndex` 10 wrapped signatures against 2 bare fields). So: the root is read-verified *and* corroborated across two independently produced trees; the site counts the acceptance condition is stated in terms of remain single-instrument. Said plainly rather than smoothed over, because those counts are what a burn-down would be judged against.

## The fork (§3)

Three independent opinions on whether to wrap a type in `Rc<T>` / `Box<T>`:

| Authority | Location | Predicate | On miss |
|---|---|---|---|
| **A — v1 shared_types** | `05_emit_rust.dag` `render_rust_shared_type_if_needed` | `set_contains(shared_types, type_name)` | **silent bare type** (no wrap) — but any shared type silently wraps |
| **B — v2 translate catalog** | `06_translate.dag` `translate_apply_use_site_ownership_*` | `target_use_site_ownership_lookup_in_catalog_node` keyed by `(carrier, use_site)` | **typed `Rejected`** (`^target_use_site_ownership_lookup_miss`) |
| **C — partition derive** | `program_partition.dag` `partition_derive_target_for_emit` | user semantic types → `ReferenceLayerOwned` rows appended to catalog | N/A (synthetic rows) |

A and B disagree today on compiler std carriers (`Node`, `Diagnostics`, …): B has explicit per-site rows in `rust_sg_rc_use_site_ownership_catalog` (return/binding → `Rc`, param → owned); A wraps whenever `build_shared_types` names the carrier, regardless of use site.

**Single authority target:** B's catalog lookup, gated by bundle readiness, exposed as one total function `wrap_decision_gate`.

## Wrap-decision predicate (the model)

### Types (`target_model.dag`)

```dag
type WrapDecision
  = WrapByValue
  | WrapByReference { layer: TargetReferenceLayer }

type WrapDecisionGate
  = WrapGateInapplicable
  | WrapGateDecided { decision: WrapDecision }
```

`WrapByValue` = emit the inner type shell with no reference layer.
`WrapByReference { layer }` = apply `target_reference_layer_apply_type_emitted` / `target_reference_layer_apply_value_expression` with the given layer (`Rc` or `Box` only — catalog `ReferenceLayerOwned` rows normalize to `WrapByValue` in `wrap_decision_from_carrier_ownership`).

### Core lookup (`wrap_decision_lookup_in_catalog_node`)

Thin rename over the existing authority — no new semantics:

```
wrap_decision_lookup_in_catalog_node(catalog, value_semantics_carriers, carrier, use_site)
  = target_use_site_ownership_lookup_in_catalog_node(...)
    |> map CarrierOwnership → WrapDecision
```

`value_semantics_carriers` short-circuit (already in `target_use_site_ownership_lookup_in_catalog_node`): atom carriers enrolled as value-semantics bypass the catalog and return `WrapByValue`. Used by `program_partition` for user types and structural surfaces.

### Bundle gate (`v2.compiler.wrap_decision` `wrap_decision_gate`)

Mirrors `translate_sg_rc_bundle_apply_disposition` (the SG-RC readiness check):

| `has_catalog` | `has_tokens` | Result |
|---|---|---|
| false | false | `Accepted(WrapGateInapplicable)` — no SG-RC opinion; legacy v1 path may still run (retire in implementation PR) |
| true | true | run `wrap_decision_lookup_in_catalog_node` → `WrapGateDecided` or `Rejected` |
| xor | | `Rejected(^translate_sg_rc_bundle_partial)` — partial bundle is fail-closed |

**Deep emitter ownership gate** = every v2 Rust emit path that today calls `translate_apply_use_site_ownership_*` **or** would have fallen through to v1 `shared_types` wrapping must instead call `wrap_decision_gate` and:

1. `WrapGateInapplicable` → pass shell through unchanged (no silent `Rc` default).
2. `WrapGateDecided WrapByValue` → pass shell through unchanged.
3. `WrapGateDecided WrapByReference` → apply reference layer (existing `target_reference_layer_apply_*`).
4. `Rejected` → propagate diagnostic; emit aborts for that site.

This is **construction over validation**: the emitter cannot state a second wrap opinion.

## Compiler-module carrier census

Std carriers with explicit SG-RC rows today (`rust.dag` `rust_sg_rc_use_site_ownership_catalog`):

| Carrier | Return / binding | Param | Struct field |
|---|---|---|---|
| `Diagnostics` | `Rc` | owned | — |
| `Node` | `Rc` | owned | `Box` |
| `TestClaim` | `Rc` (instantiation head) | — | — |
| `FreeMonoid` | `Rc` | — | — |
| `Outcome` | `Rc` (instantiation head) | — | — |
| `ModelCore` | `Rc` | — | — |
| `AlgebraInhabitanceDecl` | `Rc` | — | — |
| `ProbeHeap` | `Rc` | — | — |

**Not yet in catalog:** `UseSiteVerdict` (pilot module), and most compiler-local types. For self-emit of `04_infer` / `06_translate` / `05_emit*`:

- Std carriers above: covered by static `rust.dag` rows.
- Module-local `data`/`type` aliases: `partition_derive_target_for_emit` appends `ReferenceLayerOwned` rows at emit time (value-semantics enrollment + owned-at-all-sites default).
- **Gap to close in implementation:** `UseSiteVerdict` needs a catalog row (or value-semantics enrollment) before flip; missing row must **reject**, not inherit v1 `shared_types` default.

### Rc>100 modules (seed census, 2026-07-16)

| Seed module | `Rc<` count | Blocker until gate |
|---|---|---|
| `v1_compiler_emit_rust.rs` | ~1527 | emit surface (Track B + gate) |
| `v1_compiler_infer.rs` | ~1023 | gate + body producer |
| `v1_compiler_resolve.rs` | ~74 | namespace lane |

The gate does not alone flip these — Track B body emit and namespace resolution remain — but **without** the gate, a cargo-green emit attempt on these modules risks silent wrong `Rc` shapes (fail-open), masking aliasing bugs that behavioral receipts would not catch.

## Implementation sequence (for the follow-on PR)

1. **Land predicate + witnesses** — LANDED (#6776): `wrap_decision_lookup_in_catalog_node`, `wrap_decision_gate`, `wrap_decision_predicate_test.dag` green.
2. **Rewire `06_translate`** — LANDED (#6775): inline `translate_sg_rc_bundle_apply_disposition` + lookup chains replaced with `wrap_decision_gate` (behavior-preserving refactor; `sg_rc_layering_test` stays green).
3. **Retire v1 `shared_types` wrap for v2 emit entry** — LANDED (v2 path): zero `shared_types` / `render_rust_shared_type_if_needed` references under `src/v2/`; v1 `gunbc compile --target rust` still uses seed `shared_types` until v1 delete (S3).
4. **Enroll `UseSiteVerdict` carrier** — LANDED: `target_carrier_use_site_verdict` + owned-at-all-sites rows in `rust.dag` catalog; witnesses in `wrap_decision_predicate_test.dag`.
5. **Frontier flip** — unblock `parse_engine_hooks`, `discovery_enumeration` per `frontier_band_a_emit_readiness` (still gated on rustc-green behavioral receipts + Track B body emit).

## Witness / RED controls (`wrap_decision_predicate_test.dag`)

| Witness | Proves |
|---|---|
| `wrap_decision_diagnostics_return_is_rc` | catalog row `(Diagnostics, Return) → WrapByReference Rc` |
| `wrap_decision_diagnostics_param_is_owned` | `(Diagnostics, Param) → WrapByValue` |
| `wrap_decision_use_site_verdict_return_is_owned` | `(UseSiteVerdict, Return) → WrapByValue` |
| `wrap_decision_node_struct_field_is_box` | catalog row `(Node, StructField) → WrapByReference Box` |
| `wrap_decision_probe_heap_param_miss_rejects` | missing row → `Rejected` (fail-closed) |
| `wrap_decision_bundle_absent_inapplicable` | target without SG-RC edges → `WrapGateInapplicable` |
| `wrap_decision_bundle_partial_rejects` | catalog without tokens → `Rejected` |

Discriminating RED (both directions):
- **OMIT** — delete `Diagnostics` param row from catalog → param witness must flip from `WrapByValue` to `Rejected`.
- **WIDEN** — add v1-style silent wrap fallback → test `wrap_decision_bundle_absent_inapplicable` must fail (gate must not default-wrap).

## Non-goals (this design)

- `UseSiteVerdict` clone-vs-move (`emitter-ownership-defork`) — orthogonal axis.
- v1 seed `build_shared_types` deletion — follows v1 retirement (S3).
- Auto-generating catalog rows from a corpus census — `program_partition` derive covers user types; std carriers stay explicit in `rust.dag` (§3: cite upstream, no nickname).

## Open question (escalate if implementation stalls)

**Arc migration** (`floor_materialization.dag` P2 note): cross-process `ResolvedGraph` sharing wants `Arc`, not `Rc`. The wrap-decision predicate is layer-agnostic (`ReferenceLayerRc` is a modeled enum, not hardcoded `Rc` in translate). If Arc lands, only `rust.dag` token rows and `ReferenceLayer` inhabitants change — the predicate shape is stable.

## Probe receipts

- [Emitter residual site map (post-#6981)](../probes/emitter_residual_site_map_2026-07-21.md) — crisp-fox-839 E0425/E0255 class breakdown from emitted deep-module histograms (MAP ONLY; owner quiet-bee #6924).
