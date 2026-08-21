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

### Step 1 has a shape decision that must be made before it is built

Reconnaissance done, and it surfaced a fork the brief does not settle. Recording it here rather than
picking one, because `v1.compiler.05_emit_rust` is a load-bearing file and this lane exists precisely
because a spelling test was once put where an identity query belonged.

**What is already available, and is not the problem.** Both inputs the census needs are pure and
exist today:

- `v1.compiler.coercion` `type_realization_decision` — importable; `v1.compiler.05_emit`,
  `v1.compiler.05_emit_rust` and `v1.compiler.trait_derive_emit` already import from that module.
- `v1.compiler.05_emit_rust` `is_host_text_carrier_type` — pure, but currently **private to that
  module**: nothing imports it.

**The fork is where the census runs.**

- **(a) Inside the emitter.** Literally what the brief describes: compute both answers at each of the
  six renderers, keep emitting the legacy one, and route a receipt out. The obstacle is that the v1
  Rust emit path's file writing is host-driven, so "route a receipt out" means a **second output
  channel through the emit path** — a real signature change in the load-bearing file, and one whose
  own correctness (did the receipt change any emitted byte?) then has to be established rather than
  assumed.
- **(b) A separate substrate walk.** A new `v2.workflow` census module over the same assembled
  closure, a probe entry, and a thin host driver — the shape `v2.workflow.realization_sweep` and
  `realization_sweep_survey.rs` already establish, where the substrate does the analysis and the host
  is transport and aggregation only. Emitter untouched. Its one cost is making
  `is_host_text_carrier_type` importable — a visibility change, not a logic change, and it must be
  imported rather than re-derived, since a second copy of the predicate is exactly the §3 fork this
  work exists to remove.

  What (b) does **not** get for free: `realization_sweep`'s existing rows are per-entry-fn
  phase/cause (`sweep_rows_with_identities` → `attempt_closure_rows`), not per-occurrence. A
  per-occurrence walk carrying resolved declaration identity is new substrate work under that
  pattern, not a reuse of it.

**Recommendation: (b).** It makes acceptance condition 8 — emitted bytes outside the divergence
population unchanged — true *by construction* for step 1 rather than something step 1 must itself
prove, it keeps the load-bearing emit path out of a measurement change, and it has a working
precedent. (a) is closer to the brief's literal wording and buys one thing (b) cannot: it observes
the answer at the exact call the renderer makes, so it cannot drift from the renderer's real control
flow the way a parallel walk can. That is a genuine advantage and it is why this is a decision rather
than an obvious call.

**Not started pending that decision.** Everything above this heading is unaffected by it.

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
