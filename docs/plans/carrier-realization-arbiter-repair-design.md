# Repair design: shadow census, then an A/B/C/D counterfactual over the one fork (2026-08-21)

**Session:** `royal-dove-436`. **Work item:** `node://adhoc-c735d227-60b`.
**Authorized by:** `smart-ram-730`, after the arm census in
[`t2_t3_realization_route_2026-08-21.md`](../probes/t2_t3_realization_route_2026-08-21.md).
**Status: DESIGN. Nothing here is built.**
**This revision supersedes an earlier one in the same PR** that proposed deleting the short-circuit
as the first experiment. That is still the right eventual repair and it is the wrong first move; §2
says why, and the reason is not caution — it is that a deletion cannot measure the quantity the
decision turns on.

## What the measurement forces, before any design choice

| arm | T2 | T3 |
|---|---:|---:|
| **A** — generic carrier signature (`list_append`, `length`, `is_empty`) | **25** | 0 |
| **B** — declared-structure constructor | 4 | 17 |
| **C** — direct carrier-to-carrier assignment | 3 | 0 |
| UNALIGNED — not attributed | 2 | 1 |

Arm A makes this a forced conclusion rather than a menu:

> `list_append<T>(left: FreeMonoid<T>, right: FreeMonoid<T>)` is emitted **once**, generically, so it
> has exactly **one** host parameter type. Its `.dag` callers pass values declared `String` — the same
> modeled type, by `std.string_type` `String = FreeMonoid<Char>`. With two host realizations the
> generic function can only be one of them, and every call from the other is an E0308 **no call-site
> edit can remove**.

So the landed shape is **one exact identity query → one base target realization → position-specific
reference-layer rendering**, and two candidate repairs are already refuted:

- **Constructor-side only** closes arm B — 21 of 52 — and leaves arm A's 25, the largest arm.
- **Anything relying on monomorphization** cannot help: the type renderer's test is
  `rust_host_text_carrier_elem_name(n) == "Char"`, a syntactic read of the authored element spelling
  decided *before* instantiation. A generic `T` is never spelled `Char`.

## The authority exists, is correct, and is unreachable

`type_realization_decision` **does have consumers** — `v1.compiler.coercion` `lookup_checkpoint` is a
thin derivation of it for every `decl_file != ""` caller. (An earlier revision of this lane's PR body
claimed zero consumers; that was an error, from grepping the type name rather than the function.)

`structural_declaration_modules_for("String")` = `["src/v2/std/text.dag", "dag/std/string_type.dag"]`,
so the strict query **would** refuse the native spelling. But six type renderers — `render_rust_type`, `render_rust_type_without_applied_binding`,
`render_rust_applied_type`, `render_rust_type_with_applied_binding`, `render_rust_decl_type`,
`render_rust_fn_sig_type` — each **return on their first line** via
`is_host_text_carrier_type` → `"String"`, unconditional on `decl_file`. **No `String`-spelled
reference in type position can reach the authority.** An unreachable wall, not a missing one (DESIGN
§6 coverage-by-illusion). Established from those six renderers' control flow — each call verified to be the statement
immediately following its own `fn` declaration, not merely early in the body; **not** established by
a discriminating execution. (This count read *five* until 2026-08-21:
`render_rust_type_with_applied_binding` was missed when the first sweep's grep output was truncated
before reaching it. The full derivation is in gunbc#8805; the correction is recorded rather than
silently applied because the earlier count was relayed upward and a review restated it as fact.)

## §2 — Why the experiment is not "delete the short-circuit"

Two quantities are being confused, and only one is answerable without emitting:

- **Decision divergence** — how many occurrences get a different answer from the short-circuit than
  from the authority. Statically computable.
- **Diagnostic conversion** — which rustc failures appear once the authority controls emitted Rust.
  Only observable by emitting and compiling.

A deletion produces the second with no record of the first, so every downstream question ("was this
error caused by the change, or exposed by it?") becomes narration. The census below produces the
first *before* any byte moves, which is what makes the second adjudicable.

## Step 1 — Shadow census, no output change

At every affected type and value renderer, compute **both** answers and keep emitting the legacy one.
Write a **sidecar receipt** — explicitly **not** comments into the emitted Rust (an earlier
suggestion, withdrawn: DESIGN §4c makes an annotation unreadable by any `Accepted` program, so a
receipt emitted as commentary is a receipt no consumer can join on).

One row per occurrence, keyed by: source `DeclarationRef`; source occurrence / use site; emitted file
+ enclosing declaration; **position kind** (declaration / fn signature / value constructor / applied
type argument); legacy base realization; authority decision; reference-layer decision; identity
available?

Two constraints that are load-bearing, not stylistic:

1. **Query the strict `type_realization_decision`, never `lookup_checkpoint`.** `lookup_checkpoint`
   deliberately preserves the bare-name fallback when `decl_file == ""`, so censusing through it
   measures the bypass against itself.
2. **Outcomes are `Agrees` / `DivergesWithExactIdentity` / `IdentityUnavailable`, and
   `IdentityUnavailable` is never folded into structural rendering.** `Unrealized` (a known
   structural declaration) and `Refused` (identity not supplied) are different answers. Merging them
   reproduces the bypass under a newer name — the exact defect this work exists to delete.

### Step 1 shape: RULED — (b), a separate substrate walk, plus a mandatory calibration control

Reconnaissance surfaced a fork the brief did not settle, and `smart-ram-730` ruled it. Both the fork
and the ruling are recorded, because the reasoning constrains how step 1 may report.

**What was never the obstacle.** Both inputs the census needs are pure and exist today:
`v1.compiler.coercion` `type_realization_decision` is importable (three modules already import from
that module), and `v1.compiler.05_emit_rust` `is_host_text_carrier_type` is pure — but **private to
that module**, imported by nothing.

**The fork was where the census runs.**

- **(a) Inside the emitter** — the brief's literal shape: both answers at each of the six renderers,
  legacy one emitted, receipt routed out. The v1 Rust emit path's file writing is host-driven, so
  routing a receipt out means a **second output channel through a load-bearing file**.
- **(b) A separate substrate walk** — a `v2.workflow` census module over the same assembled closure,
  a probe entry, and a thin host driver: the shape `v2.workflow.realization_sweep` and
  `realization_sweep_survey.rs` already establish, substrate analysing and host transporting.

**Ruled: (b).** On the merits: (a) obliges step 1 to prove its own inertness before it can report
anything about its subject, and *an instrument that must first prove it did not disturb the thing it
measures is a worse instrument than one that cannot disturb it*. Under (b), acceptance condition 8 —
bytes outside the divergence population unchanged — is true **by construction** for step 1 rather
than something step 1 establishes.

**The objection against (b), and what neutralises it.** A parallel walk can drift from the control
flow it claims to describe, and *this lane exists because a parallel answer path was authoritative in
the wrong place*. That suspicion is correct, but the drift risk is not uniform across the two things
(b) borrows, and separating them is what decides it:

- **The predicate cannot drift**, because `is_host_text_carrier_type` is **imported, not
  re-derived**. A re-derived copy would be the same §3 fork again and is an immediate refusal;
  importing it is the whole reason (b) is admissible.
- **The traversal can drift.** The walk may visit occurrences the emitter never renders, or miss ones
  it does. That is the real exposure, and it is not hypothetical: a subtly wrong occurrence set makes
  every count wrong while looking perfectly well-formed.

**So the calibration control is a precondition of the census being reportable, not a nice-to-have:**

> The walk must **reproduce the known 25 as its diagnostic-producing divergence subset.** Partition
> the walk's `DivergesWithExactIdentity` rows by whether the occurrence currently produces a rustc
> diagnostic; that subset must equal the 25 sites of arm A — joined by source declaration + enclosing
> emitted declaration + operation, **never by line**. **If it does not, the census is WRONG and its
> divergence count must not be published. Report the mismatch instead.**

This converts the drift objection from an argument into a measurement, which is the only way to
settle it. Note what it does **not** require: the walk need not *explain* the 25, only **find** them.
It composes with the cross-tab already committed to — the calibration is one cell of it, and the
interesting cell (`DivergesWithExactIdentity` at zero diagnostics) is trustworthy only once the
calibrated cell checks out.

### Correction to (b) as ruled: the v2 assembly is the WRONG TREE, so (b) is v1-side

Found while building it, and recorded because it falsifies a premise both the recommendation and the
ruling rested on — not the ruling's *reasoning*, which survives intact, but the concrete artifact it
named.

**(b) was described as "a `v2.workflow` census module over the same assembled closure", reusing the
`v2.workflow.realization_sweep` pattern. That is not implementable as stated.** That pattern's
`assemble_program_from_ingest` returns an `Outcome<Node>` over **`v2.std.node.Node`**, while
`v1.compiler.05_emit_rust` — the module whose decisions the census exists to measure — operates on
**`v1.std.core.Node`**, imported by name at the top of that file. They are different types. A walk
over the v2 assembly would be measuring a different tree from the one the emitter renders, which is
the traversal-drift failure the calibration control exists to catch, built in at the foundation.

**(b′), the corrected shape, is v1-side and is implementable today.** `v1.compiler.compile`
`front_end_sources(sources: List<SourceFile>) -> FrontendResult` is a public entry returning
`FrontendResult { graph: ModuleGraph?, newline_indices, intern_table, … }` — the v1 tree the emitter
consumes, plus the `newline_indices` that `authored_name_at` needs. So:

1. the host supplies the closure's `SourceFile` rows — **transport only**, the same role
   `realization_sweep_survey.rs` plays for its probe;
2. the census module calls `front_end_sources` and walks each module's items for type-reference
   occurrences;
3. per occurrence: `authored_name_at`, `v1.compiler.coercion` `type_reference_decl_file`, position
   kind;
4. legacy answer from the **imported** `is_host_text_carrier_type`; authority answer from the strict
   `type_realization_decision`;
5. rows out as TSV.

**Everything the ruling actually reasoned about survives this correction**, which is why it is a
correction and not a re-open: the predicate is still imported rather than re-derived, no output
channel is added to the emit path, acceptance condition 8 is still true by construction for step 1,
and the calibration control is unchanged and now matters more — a v1-side walk *can* still drift from
the emitter's control flow, and reproducing the 25 is still what settles it.

**REFUTED BY CI, 2026-08-21 — the census module DOES have to live in `src/v1/`.** This paragraph
claimed the module need not, on the grounds that import direction is not a layer rule (DESIGN §3
makes acyclicity the only structural law and folders a browsing convention), and concluded that this
"removes the v1-growth question from step 1 entirely".

**The premise is true and the conclusion is false.** Import direction is indeed not a layer rule, but
that is not the constraint that binds. The required floor discovers subjects by **path**, over the
roots `dag` and `src/v2` (`witness_layer_roots`), and resolves that closure as one program —
`modules_resolved=3820`. A module authored at `src/v2/workflow/` is therefore swept into discovery
because of **where it sits**, not because of what imports it, and its `v1.*` imports are unresolvable
inside that envelope. Measured, on PR #8816:

```
src/v2/workflow/carrier_realization_census.dag:3:1: error: unresolved import:
    module 'v1.compiler.compile' not found
required-ci: floor refused: subject=7b5536161546187e modules_resolved=3820 modules_excluded=4
```

This is the same structural unresolvability `dag/test/claim/checkpoint_identity_keying_witness_test.dag`
already documents about itself; I read that precedent and still authored outside `src/v1/`.

The module now lives at `src/v1/tests/claim/carrier_realization_census.dag` and produces
**byte-identical** results there, so nothing measured is invalidated — only this placement claim is.

**So the v1-growth question is REOPENED, and it is an admission to state rather than one to assume.**
Step 1 adds one module to the seed tree. Under DESIGN §3's v1 maintenance standing the admission test
is PURPOSE — *"anything in support of v2 self host is safe"* — and the E0308 board is that program,
so it reads as admissible. It is recorded here as an open admission, not as a settled one: this
document does not get to grant itself the exception, and the paragraph it replaces is exactly what
happens when a design talks a question out of existence instead of answering it.

### There is no visibility change: (b′) is zero-touch on `05_emit_rust` after all

This heading previously read *"(b) is NOT zero-touch on `05_emit_rust`, stated plainly"* and
disclosed a visibility change on `is_host_text_carrier_type` as the one seed edit step 1 needs.
**That disclosure was wrong, and it is corrected in the direction of touching less, which is exactly
the direction a disclosure must not be left wrong in.**

The `.dag` language has **no visibility syntax at all** — corpus-wide there is no `export`, `pub` or
`private` form; every top-level `fn` in a module is importable by name. `is_host_text_carrier_type`
is an ordinary top-level `fn`, exactly like `rust_scalar_checkpoint_render_base` — which
`src/v1/tests/claim/checkpoint_identity_keying_witness_test.dag` **already imports from this same
module** today. `v1.compiler.05_emit_rust` is likewise already imported by `v1.compiler.compile` and
`v1.compiler.stage0_crates`.

So **step 1 requires no edit to `v1.compiler.05_emit_rust`, and no edit to the v1 seed at all.** The
predicate is imported, not re-derived — the property the ruling actually depended on — and nothing is
widened to achieve it.

**The lifetime question dissolves with the change that raised it.** It asked whether a widened
surface survives the repair or is scoped to the census. There is no widened surface: the predicate's
importability is a property of the language, not a grant this lane made. If the terminal repair
deletes the spelling-keyed short-circuits and the predicate with them, the census's import fails
loudly at that point — which is the correct coupling, and is a live dependent rather than residue.
Recorded rather than dropped, because an unasked question and a dissolved one look identical in six
weeks.

### The census's executing consumer is its own driver, because the batch it would have used is gone

`src/v1/tests/claim/checkpoint_identity_keying_witness_test.dag` documents its own execution as
`gunbc.ci_layer_roots` `v1_claim_scoped_witness_entries` → `v1_claim_scoped_witness_batch`, "whose
source-root envelope is dag plus src/v1". **That batch is deleted** — `ci_layer_roots`
`v1_claim_scoped_witness_batch_deleted_note` records the deletion (2026-08-15), and
`witness_fold_src_v1_coverage_gap_note` carries the resulting declared coverage gap.

So a census module cannot inherit that enrollment, and step 1 does not try to. Its executing consumer
is **its own host driver**, the "local recipe" standing `realization_sweep_survey.rs` already
occupies for its probe — the driver supplies the `dag` + `src/v1` source-root envelope for the census
module itself, and separately supplies the subject closure's sources as `SourceFile` **data**. Those
are two different roles for two different populations and the design keeps them apart deliberately:
confusing them is how a census ends up measuring its own pool instead of its subject.

## Step 2 — Four counterfactual crates over one pinned tree

Same toolchain, same assembly path, same manifest, same source population, separate fresh output
directories:

| | type position (decl + fn sig) | value position |
|---|---|---|
| **A** baseline | legacy | legacy |
| **B** | strict | legacy |
| **C** | legacy | strict |
| **D** | strict | strict |

A factorial over exactly the fork the census measured. It answers what no total and no single
treatment can: whether arm A's 25 are controlled by the **type** short-circuit, whether arm B's 21
are controlled by the **value** short-circuit, and whether converging both produces agreement or
merely relocates the disagreement.

**Publish a site-conversion ledger, not four totals:** baseline site, treatment site, source
declaration, old expected/found, new expected/found, old category, new category. **Join by source
declaration + enclosing emitted declaration + operation — never by line.** One error block can split
into several sites, and lines move.

**Do not land the dual renderer.** It is experimental quarry: leaving both answers in production
creates the second authority this work exists to delete (DESIGN §3).

### Ledger requirement: diff the full warning histogram — and the probe's column CANNOT carry it

**The requirement, and the near-miss that produced it.** `smart-ram-730`, from another lane: a repair
keyed on the emitted pattern string moved its board 31 → 32 — noise — while underneath it
`unreachable_pattern:6` appeared. **Those are errors here, not lints.** Two arms of the same variant
emitted different patterns, the first lost its guard and shadowed the second into dead code at six
sites. The headline moved by one; six arms died. A realization change is exactly the edit that can
make one arm subsume another: if a carrier's base realization changes, arms distinguishable by its
host type may cease to be.

**An earlier revision of this section said to satisfy that by diffing the probe's histogram column.
That check cannot work, and the defect is in the instrument, not the diff.** Verified first-hand in
`docs/probes/curated_cargo_probe_one.sh`: `ERROR_HISTOGRAM` greps `'^error\[E[0-9]+\]'` and
`HISTOGRAM_SUM` greps `'^error(\[E[0-9]+\])?:'`, over a plain `cargo build --release --lib` with no
`--message-format=json`. `warning: unreachable pattern` matches **neither**. The class can never
appear in that column.

**So absence of that row is blindness, not zero.** Reporting the class clean from that column would
not be a measurement — it would be an instrument that cannot express the class, which is the
absorbing fallback (DESIGN §5) relocated into the measuring apparatus. (Older readings that *did*
carry the class legitimately came from a `rustc --message-format=json` instrument, which includes
lints. Same question, different instrument, different answer.)

**What the A/B/C/D ledger does instead:** read the kept cargo log directly and diff the **full sorted
`warning:` histogram** across all four crates, in one dispatch covering every arm.

**And it carries the honest bound, because at a refusing baseline nobody can do better:** the emitted
crate fails to build, so rustc stops before typechecking most items and never runs reachability on
them. A zero from that log means *"none among what rustc reached, and unchanged between arms"* — it
does **not** mean the crate has none. That bound is published with the number, not left to a reader.

**NARROWING, verified first-hand after `bright-dove-741` refuted the general form.** The emitted
crate carries `#![deny(unreachable_patterns)]` — confirmed in `v1.compiler.stage0_crates`'s emitted
crate attributes and in the seed's own `lib.rs`. A **denied** lint renders as `error: unreachable
pattern`, which **does** match the probe's uncoded-suffix grep. So the blindness above is real for
**warning-form** lints and **not** for denied ones, and `bright-dove-741` holds the positive control
in band: 6-then-0 across their own defective and shipped runs, on this instrument and entry.

**So the standing rule is narrower than first stated, and this is the corrected form: whether an
instrument can emit a class is a property of the EMITTED ARTIFACT's lint attributes, answered per
artifact — not a fixed property of the probe.** The cheap way to answer it is to find a run where
that class was nonzero; a class no run has ever produced is a class you cannot report clean. The
log-level warning reading above stays the right mechanism for warning-form classes; the rule around
it is what changes.

## Step 3 — Pre-registered acceptance

A rising rustc total is neither proof of progress nor proof of failure. The repair is semantically
correct only if **all** hold:

1. Every changed occurrence was already in the shadow divergence population.
2. The selected base equals `type_realization_decision` for the exact declaration.
3. Type and value positions consume the **same** base answer for one declaration.
4. The reference layer stays a **separate axis**; `Rc` is never inferred from base spelling.
5. Same spelling + different declaration identity can still give different answers.
6. Unavailable identity **refuses** rather than taking the legacy bare-name answer.
7. A generic `FreeMonoid<T>` is not classified as text because some instantiation later uses `Char`.
8. **Emitted bytes outside the divergence population remain byte-identical.**

Condition 8 is the discriminator this lane did not previously have, and it is why it is stated as a
test rather than a hope: **a new error in an emitted file whose bytes did not change is not "debt
exposed by the repair" — it is contamination or an unrelated tree delta.** It converts a vague worry
into a mechanical check.

Then classify every treatment-only error as `ExpectedSuccessor` / `UnrelatedRegression` /
`Unclassified`. `ExpectedSuccessor` requires a **carried** causal relation, not a narrated one: it
occurs at the changed occurrence or a direct use of the changed declaration; its expected/found
contains the authority-selected realization or its independently derived reference layer; the
baseline emitted a different type at that exact semantic position; and the obligation could not have
arisen until that representation was selected.

**Acceptance: wrong-authority decisions = 0 over the scoped population, unrelated regressions = 0,
unclassified = 0.** Only then does the raw total speak to convergence at all.

## The Root B precedent, used for what it actually establishes

Root B's forced `HostNative` changed **two independent axes at once** — primitive realization *and*
import/use-line synthesis. So its 693 → 807 established neither "the rise is hidden debt" nor the
opposite; it established that the repr caused its target family, and that the switch was not a valid
repair because it carried a fused unrelated decision. **Use Root B as precedent for isolating axes —
which is exactly what A/B/C/D does — never as precedent for admitting an increase.** This experiment
can be cleaner than its precedent: the identity authority already exists, and the short-circuit
varies without touching import policy, closure selection, or Cargo config.

## Instrument lesson this design adopts for its own ledgers

`smart-ram-730`, on the RT-builtin misattribution: **a classifier that folds its discriminating input
into its derived label destroys the ability to re-adjudicate later**, and the cost lands on whoever
needs a different partition than the one that was authored. The 2026-08-21 partition consumed each
block's `note: function defined here` into its `root` column, so 5 sites coded `RT-builtin` cannot be
re-split from the TSV — the callee note is a good **field** and was a bad **label**.

This design's ledgers therefore carry the raw discriminating inputs beside every derived label, as
`routes.tsv` and `arbiter_arms.tsv` already do (raw `expected`/`found` beside `expected_route` /
`found_route` / `carrier` / `verdict`). Concretely, the step-2 ledger carries position kind, identity
availability, and both decisions as **fields**, never only the conversion verdict computed from them.

## Population, pinned so it cannot drift

The scoped population is the **52 shared-root sites** in `arbiter_arms.tsv`, and the first deliverable
is the A/B/C/D conversion receipt over **arm A's 25**: largest measured arm, constructor-only already
falsified against it, mechanism located, authority present, and its failure is that the authority is
*unreachable* rather than unspecified.

The 5 `RT-builtin` sites in `v2_std_compilers_target_model.rs` (lines 6183 ×2, 6198, 6242, 6267) are
**outside** this population and stay outside it. Two of them plausibly belong to this root and three
to the cross-realization signature-drift root, but this lane's spelling-keyed classifier cannot
discriminate `HashMap`-from-inhabitant-row from `HashMap`-from-`v1_rt::lookup`-signature, so the
adjudication waits for the callee-note field rather than being made by an instrument that cannot make
it. Two sites either way do not touch the 25.

## What this design does not establish

- **It is not costed.** No estimate is offered, because the A/B/C/D receipt is what produces the
  blast-radius number any estimate would need.
- **It does not claim the arms share one fix.** B and C are treatments precisely so that question is
  measured rather than assumed.
- **It does not pick structural vs native `String`.** That is a policy about the seed's own
  representation; the experiment produces the data it should be decided on, and
  `checkpoint_table_bypasses_identity_note` records what picking a direction without that data cost
  last time (the over-broad `Bool` row: 5 fixture refusals plus `required-regen` drift).
- **It does not touch the v1 freeze question.** Every step edits the v1 seed emitter, admissible
  under the DESIGN §3 purpose test only insofar as it serves the v2 self-host program — which the
  E0308 board is, but the admission is the operator's, not this document's.
