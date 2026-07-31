# Compiler guarantee recovery — gap analysis

**Status:** WORKING DRAFT, audit in progress. No code lands from this note.
**Question it answers:** what did this compiler intend to guarantee, and what does it guarantee today?

## 0. Provenance

The specification half of the project's authority was lost in the 2026-06-16 doc
bankruptcy (`3127161e878` "cleanup day" #5027, re-deleted in `80ea2664925` #5029). The
epistemology survived into DESIGN.md; the specification did not. Recovered sources, all
readable at `3127161e878^`:

- `THESIS.md` — the guarantee statement, the tier structure, the complete claims list
- `INVARIANTS.md` — five principles (they map cleanly onto today's DESIGN.md §3–§6)
- `MODELING.md`, `docs/thesis/*` (~10 files), `docs/error-examples.md`

DESIGN.md today is a strong reconstruction of **how to reason** (§1–§2 axioms and
redundancy), **where facts live** (§3), **what makes them decidable** (§4), **how to fail**
(§5), and **how to work** (§6–§7). What it does not contain is a statement of **what must be
established before a program is `Accepted`**. That absence is the subject of this note.

## 1. The recovered guarantee statement

From `THESIS.md`, verbatim:

> **If it compiles, the declared intent is sound inside the modeled system; what remains
> unverifiable is only external reality not carried in the program graph.**

And the framing immediately above it:

> gunbc is a causal engine: it validates that a program's declared causes, dependencies, and
> drains are structurally coherent **before emission becomes a mechanical translation**.

Emission is specified as the downstream, mechanical half of a two-stage contract. The
upstream half — validation — is the one that was not built.

## 2. The failure history (revised after independent review, 2026-07-31)

An earlier draft of this section told a two-part story: the spec was lost, then the claims
were priced out one at a time. Independent review corrected it, and the corrected history has
**three distinct failure modes**, each with its own receipt:

1. **The implementation never matched the specification, even while the specification was
   live.** `module_skips_direct_call_arg_check` (the `v2.*`/`v1.compiler.*` exemption) dates
   to **2026-06-08** (`a13fb57b149`, first `-S` occurrence) — eight days *before* the
   2026-06-16 bankruptcy. The gaps predate the loss of the document that named them.
2. **The status ledger overstated completeness.** The recovered
   `docs/thesis/correctness-dimensions.md` table marks Type safety as *"Yes (blocking)"*
   while return-position, `data`-annotation, and generic-instantiation checking did not
   exist and the compiler-module exemption was live. So "we had it and lost it" is wrong;
   the honest statement is *"it was declared DONE, was not, and then the declaration that
   could have been audited was deleted."*
3. **The bankruptcy removed the contract against which (2) could have been noticed.**
   DESIGN.md kept the epistemology and the cost discipline; nothing survived that says what
   must be established before `Accepted`.

**A withdrawn receipt, kept for the record:** an earlier draft cited `v2.std.orchestration`
`pipeline_steps_empty_arm_note` (the declined `NonEmpty<PipelineStep>` carrier) as proof that
a required invariant was priced out by §6's purity trap. Independent review rejected that
reading and the rejection is correct: the note *deliberately* gives the empty pipeline total
no-op semantics (empty conditional arms emit the bash no-op `:`; a both-empty `If` is "run
the condition, discard the outcome"). Emptiness there is a **legitimate total semantics**, so
the decline was a correct engineering decision, not a dropped wall. The note remains a fair
illustration of the *decision procedure* — a carrier addition priced by §6 with no
specification to consult, "revisit if construction sites appear" as the only trigger — but
it is not evidence the procedure ever produced a wrong answer. No receipt currently in hand
shows a *required* invariant being declined.

**Consequence for the fix, sharpened by mode (2):** restoring the claims list is still the
move that makes the next decline-decision answerable — but historical claims must enter as
**provenance and candidate obligations, never as implementation truth**. A recovered claim
lands as `Required` + `CurrentStatus: Gap` until a discriminating RED/green pair proves
otherwise. Copying the historical "DONE" column would repeat mode (2) verbatim.

## 3. Status vocabulary (widened after independent review)

The original three-state vocabulary (ENFORCED / UNENFORCED / UNEXPRESSIBLE) lost exactly the
information this tree carries: a guarantee can have a carrier that proves nothing, a wall
that holds only for literals, or a check that fires at the runtime boundary rather than at
compile time — and each of those has a different fix. Working states, adopted from the
independent review:

`Absent` · `Declared` · `RepresentableButForgeable` · `LiteralOnlyWall` · `LocallyChecked` ·
`RuntimeBoundaryOnly` · `StaticallyPropagated` · `ConstructionWall` ·
`TargetRealizationGated` · `ExternalNotGuaranteed` · `UnknownUnmeasured`

Calibration examples, each verified on `main`: `NonEmptyList<T>` is
RepresentableButForgeable; string `non_empty` where-refinement is LiteralOnlyWall; a
nonliteral refined argument is RuntimeBoundaryOnly (five `WhereRefinementUnenforced` deferral
reasons are enrolled as *advisory* — `v1.compiler.core`
`where_refinement_deferral_reason_scaffold_note`); v2 loop termination is a ConstructionWall
(`v2.std.cardinality` requires a declared loop measure, fail-closed to `DescentUnknown`);
unknown method is a fail-open Absent; host state is ExternalNotGuaranteed by the guarantee
statement itself.

This vocabulary is prose-interim: it becomes a typed enum on the claim carrier when the
guarantee authority lands (§11), the same way `DescentEvidence` and `DecodeFidelity` model
their own verdicts. The DESIGN.md §5 wall-classification (*wall now* / *wall after
grounding* / *ratchet forever*) stays as the orthogonal decidability axis.

## 4. Tier 1 — structural correctness ("impossible to write the bug")

| Claim (THESIS.md) | Today | Receipt | §5 class |
|---|---|---|---|
| Type mismatches caught at compile time | **UNENFORCED** (return position, `data` annotation, generic instantiation) | `fn f() -> Int { "not an int" }` typechecks — probed by execution, PR #7481; argument position *is* checked | wall after grounding |
| …in compiler source | **UNENFORCED by exemption** | `v1.compiler.infer` `module_skips_direct_call_arg_check` — skips `v2.*` and `v1.compiler.*` | wall now |
| Field typos | **PARTIAL** — concrete types checked; through a type variable, not | `v1.compiler.infer` mints `TypeVariable { id: "field_of_type_var" }` instead of refusing | wall after grounding |
| Application arity / call shape (missing, extra, misspelled-label args) | **fail-open by construction of the walk** | `v1.compiler.infer` `direct_call_arg_mismatch_diags` is *formal-driven*: per formal it seeks a same-named arg, else falls back to the **positional** arg at the same index (a misspelled label silently binds by position if the type fits), and `Absent => []` (missing arg → no diagnostic); extra args are never visited. The `ArityMismatch` diagnostic is **type-constructor** arity ("expects N *type* arguments"), not invocation arity — invocation arity has no compile diagnostic; #6896's wall is runtime-only | wall now |
| Non-exhaustive matches | **PARTIAL — one confirmed silent arm** | resolved coproducts have exhaustiveness machinery; but `v1.compiler.infer_patterns` `lookup_variant_in_type` / `lookup_field_in_variant` both have `PatternLookupBlocked => node_lookup_failed(diagnostics: [])` — a blocked scrutinee lookup fails with **zero diagnostics** and the pattern types as `error_type` (`PatternDynamic`, by contrast, does diagnose at these sites). "Exhaustiveness not established" is treated as success-adjacent, not refused | wall after grounding |
| Cardinality / multiplicity (empty list into a callee that requires one) | **RepresentableButForgeable, not statically propagated** — reclassified from UNEXPRESSIBLE after independent review | Representation exists: `v2.std.refinement` `Validation<B>`/`Refined<B>`/`refine`, a `NonEmptyList<T>` manual fixture (`v2.test.claim.manual.refinement_nonempty_list` + testgen anchor), and **green fold-propagation witnesses** (`cardinality_fold_propagation_test`: `cardinality_is_a_fold_homomorphism`, `fold_overflow_rejects`). Forgeable: `Refined<B>` is a public record — `refined_vacuous_stub_pack`'s `Rejected` arm literally returns `Refined { base }`, so the carrier proves nothing about validation. Not propagated: no cardinality lattice in signatures (`v2.std.cardinality` is loop-termination), `InterfaceSummary` (`dag/std/interface_summary.dag`) carries no cardinality slot, no transfer functions across `map`/`filter`/`concat`. The substrate `Cardinality` connective remains production-uninhabited and v1 forks the name onto optionality (`Required \| CardOptional`) | wall after grounding |
| Method existence | **UNENFORCED — fabricates** | `v1.compiler.infer` `method_pipe_map_keys_values_fallback` else-arm returns the *receiver type* with `kernel_diags: []`; unresolved method stamped `PlainMethodSemantics` | wall now |
| Grounding completeness — "**not** a name-keyed table lookup" | **VIOLATED literally** | `v1.compiler.infer_method` `builtin_function_registry() -> Map<String, Node>` is a name-keyed table (~120 entries), one of ≥5 independent primitive-existence authorities | wall after grounding |
| Circular deps / stale imports / cross-target drift | **UNVERIFIED** | not yet audited | — |
| CX gate: every recursive fn terminates with a proven bound | **UNVERIFIED** | `DescentEvidence` exists (`dag/std/termination.dag`, fail-closed to `DescentUnknown`); enforcement scope unaudited | — |
| Ownership: no aliased mutation in emitted code | **UNVERIFIED — known latent fail-open** | DESIGN.md open thread: emitter silently wraps every `shared_types` member in `Rc<T>` | — |

## 5. Tier 2 — runtime safety ("proven safe or total")

The claim is one sentence: **"No partial functions in the runtime."**

Every `InterpError` variant that can fire on an *accepted* program is a counterexample to it.
The enum is not the specification — it is the **evidence register**. Seventeen variants
today; the ones already known to fire on accepted programs:

| Variant | Fires on accepted program? | Receipt |
|---|---|---|
| `Unimplemented` | **YES** | PR #7479 — `filter_map` type-checked, reached dispatch as `not yet implemented`, returned HTTP 500 on the lab. Whole-tree compile: 0 blocking errors |
| `CallContractMismatch` | **YES** — runtime wall only | #6896 landed the wall in the interpreter; the compile-time twin does not exist |
| `NoSuchFunction`, `NoSuchVariable`, `NoSuchField` | **reported at runtime** in #6848's floor histogram | UNVERIFIED sub-counts |
| `DivisionByZero` | **reported at runtime** | Tier 2 names it explicitly as "proven safe or made total" |
| `CrossRepresentationEquality` | fail-closed backstop, dead-in-corpus for numerics | DESIGN.md — waits on the `Value::Null` split |
| `PatternMatchFailure` | **UNVERIFIED** | — |

Genuinely out of scope (environment/resource, not program soundness — these belong in a
*declared not guaranteed* column, with a reason): `EvalBudgetExceeded`,
`WitnessWallBudgetExceeded`, `HostToolUnresolved`, `HostToolRelativePathAmbiguous`,
`ArgvExceedsHostArgMax`. `EarlyReturn` is a control-flow mechanism, not an error (unverified).

**The discipline to restore:** every variant lands in exactly one column with a reason.
Unclassified is the gap census. This is decidable because the vocabulary is closed — but the
vocabulary must be the *substrate's*, not the v1 Rust seed's. `InterpError` is one
realization's taxonomy; using it as the denominator is the §3 inversion. It is admitted here
as **evidence**, not as spec.

## 6. Tier 3 — verification from structure

L4 (emitted matches `.dag` eval) · L5 (same behavior across targets) · L6 (every form
compiles to every target) · L7 (operations obey declared algebraic laws).

**Entirely unaudited.** Flagged because L6 has a known counterexample shape: `v1.compiler.emit`
`emit_error_expr` renders an "unsupported cast" error expression through the target's
`error_expr_template` rather than refusing at compile time — a form that compiles and emits a
failing artifact.

## 7. The v2 compiler is separately exposed

Two problems that compound, with a direction:

1. **v1 admits v2 source unchecked** — `module_skips_direct_call_arg_check` exempts `v2.*`.
2. **v2 certifies its own nodes** — `v2.std.constraints` `solve_constraints` passes
   `graph.root` as source, algebra, *and* the sole candidate;
   `v2.std.constraint_satisfaction_predicate` `constraint_satisfaction_preservation_holds`
   reduces to `well_formed(root)` checked three times, then relabels it `CanonicalGrounding`.
   Specialized inference exists for exactly `Branch`, `Match`, `Loop`
   (`v2.compiler.04_infer` `infer_gather_fold_init`); everything else takes the `_` arm.

So neither compiler ever establishes the facts for v2's source. **v1 deletion does not close
this**, and the §7 self-host frontier reads greener than it is: `emit` can be green-by-
execution on a module whose types were never checked, because emission consumes a `Node` plus
a name string, not a proof.

## 8. What the thesis predicted, in its own words

Three recovered lines that name today's defects directly:

- **The keystone.** *"It only works if the types are actually distinct… That distinctness is
  the keystone: no enforcement, no derivable coercion — meaning-in-the-types is cosmetic
  without it."* Return position, `data` annotation, and generic instantiation are unchecked,
  and `v2.*` is exempt. Since **coercion = emission** is a declared concept unification, the
  chain closes: no enforced distinctness → no derivable coercion → **emission runs on
  cosmetic types.**
- **Grounding completeness.** *"…algebra inhabitance declared structurally — not string-typed
  shortcuts in a lookup table… not a name-keyed table lookup."* `builtin_function_registry`
  is literally that.
- **Epistemic stacking**, flagged *"load-bearing for codegen — must not be dropped"*: *"Every
  emitter special case is evidence of an ungrounded concept upstream."*

## 8b. The acceptance corpus was recovered — and it has a hole in the same place

`docs/error-examples.md` survived intact (487 lines, readable at `3127161e878^`). It is
structured as a RED-control corpus: `.dag` input, the exact expected compiler error, the
algebra that catches it, and why a traditional compiler cannot. Its own framing:

> These serve as TDD targets: each example is a test case. The `.dag` code is the test input;
> the error message is the acceptance criterion. When the feature lands, the test should pass.

Seven cases: (1) non-terminating recursion through type resolution · (2) cross-service data
corruption through non-idempotent retry · (3) redundant computation across service boundaries
· (4) accidentally quadratic with a non-obvious cause · (5) infrastructure drift through
partial failure · (6) semantic cancellation across function boundaries · (7) exponential
blowup from unguarded recursive branching.

**The finding: every one is a Tier 2-or-above case. Not one is Tier 1.** There is no
method-existence example, no declared-return-conformance example, and — searched explicitly —
**no cardinality or empty-list example anywhere in the corpus** (zero occurrences of
`empty` / `cardinal` / `NonEmpty` / `arity` outside an unrelated `fold` body).

This revises §2's account of the drop. The specification did not merely get priced out later;
it was **under-specified at the source**. Tier 1 was treated as solved-by-assumption and
given no acceptance criteria, while the exotic wins got seven fully-worked ones. So the
foundation had no test, therefore no consumer, therefore nothing to notice it was never
built — the specification-without-execution trap (§5) operating on the *specification itself*.

**Consequence for salvage, and it is the load-bearing one:** the recovered corpus is directly
usable for Tier 2/3 walls, but **the Tier 1 RED controls do not exist and must be authored.**
They are the cheapest and most urgent artifacts in this whole effort — each is a
three-line `.dag` program — and they were never written because no one imagined needing them.
The user-supplied case is the archetype and belongs in the corpus as example 0:

```dag
// callee requires at least one; caller can supply zero.
// Today: unexpressible — no NonEmpty<T> carrier, Cardinality connective uninhabited.
```

## 8c. The dimension architecture — recovered contract, and it is still standing

`docs/thesis/correctness-dimensions.md` defines the mechanism that was supposed to make all of
this uniform, and it is the strongest salvage news in this note.

> Correctness is not one property — it is many orthogonal dimensions… In gunbc they are
> **inescapable properties of the system, like conservation laws in physics. You don't opt
> into gravity.**

Four-part contract for every dimension: (1) declared in `std/` as a structural type with
lattice operations; (2) **computed at binding sites during inference — no separate analysis
pass**; (3) carried through the IR on bindings; (4) **enforced universally — all code is
subject to all dimensions, no escape hatch, no wrapper functions.**

**Clause 4 is violated by name.** `v1.compiler.infer` `module_skips_direct_call_arg_check` is
exactly an escape hatch, and it exempts the two module trees that most need the dimension.

**But the carriers are real and present**, verified on `main`:

- `v1.compiler.infer_env` `TypeBinding { name, resolved, provenance: SubValueRelation }` —
  the thesis names `TypeBinding.provenance`, and it is there.
- `v1.compiler.core` `ExprCall { call_semantics, descent_evidence: List<SubValueRelation>? }`
  — the thesis names `ExprCall.descent_evidence`, and it is there.

So the mechanism the thesis describes was **built, not just designed**. What did not happen is
the rest of the dimensions moving onto it. Restated in the thesis's own table format
(*Carried on bindings* / *Enforced* are the load-bearing columns):

| Dimension | Declared today | Carried on bindings | Enforced |
|---|---|---|---|
| Type safety | `dag/std/types.dag` | `TypeBinding.resolved` | **Partial** — argument position only; return, `data` annotation, generic instantiation unchecked; `v2.*`/`v1.compiler.*` exempt |
| Termination | `dag/std/termination.dag` (BoundedLattice, bottom = fail-closed) | `TypeBinding.provenance`, `ExprCall.descent_evidence` | **UNVERIFIED** — thesis-era status was "Partial, 421 violations, non-blocking"; needs re-measurement |
| Cardinality / multiplicity | **nowhere** — connective uninhabited | No | **UNEXPRESSIBLE** (§4) |
| Ownership | `src/v1/ownership.dag`, `src/v2/lens/ownership.dag` | No — still a separate pass | **Partial**, plus a known latent fail-open (Rc wrap) |
| Side effects | `dag/std/behavioral.dag`, `dag/std/effects.dag` | No | Consumers now exist (`std.effect_grant`, `std.realization`, `gunbc.host_effect`) — an improvement on the thesis-era "declared, not consumed" — but **not at binding sites** |
| Idempotence | `dag/std/effects.dag` (lattice from `EffectShape`) | No | No |
| Purity | not declared | — | No |
| Space bounds | not declared | — | No |

**Why this matters for sequencing:** two of eight dimensions reached the binding carrier and
the carrier still works. The salvage is therefore *not* an architecture rebuild — it is
finishing a migration that stalled, plus deleting one escape hatch. That is a materially
cheaper and more defensible proposition than the wall-by-wall roadmap implies, and it should
be the frame for step 3.

The thesis also names its own falsification test: *"If user-defined dimensions work the same
as built-in ones, the mechanism is general. If they require special compiler support, the
mechanism is incomplete."* Unaudited. `src/v1/dimensions-design.md` (396 lines, recoverable)
is the abstracted mechanism and has not been read yet.

## 9. Where the ambition sat, for calibration

`docs/thesis/what-dag-catches-that-normal-compilers-dont.md` — its examples are
non-terminating recursion (CX descent proof), accidentally quadratic, `reverse |> reverse`
(involution ⇒ "equivalent to doing nothing"), `map(f) |> map(g)` fusion, non-idempotent
workflow marked retry-safe, `create_resource()` in a retry loop, `fib` at O(2ⁿ). Closing
line: *"No special-case analysis. No lint rules. No opt-in annotations. The algebra does the
work."*

The live defects are "does this method exist" and "does the body match the declared return."
**The documented ambition is two tiers above mainstream compilers; the current position is one
tier below them.** That gap is the honest statement of the problem.

## 10. Evidence status

**Verified by reading `main` (`26f747fd9ec`) during this pass:** the method fallback;
`resolve_builtin_call_type`; `module_skips_direct_call_arg_check`; the `Unimplemented`
dispatch default; `filter_map_contract` present with zero seed implementation (`flat_map`
registered, for contrast); v2 self-grounding end-to-end; v2's three specialized arms;
`field_of_type_var`; 63 `PrimitiveContract` rows; `Cardinality` declared with zero production
inhabitants; v1's `Cardinality` name fork; absence of `NonEmpty<T>`; the
`pipeline_steps_empty_arm_note` decline; the `emit_error_expr` cast site.

**Carried forward UNVERIFIED — do not cite until measured:** #6848's floor histogram
(1,454 pass / 149 fail; 66 `contains`, 40 no-such-function, 6 undefined-variable); #6896's
32 malformed sites / 180 intermediate failures; the "104 `TypeMismatch` false positives" from
the exemption experiment (**no receipt found at all** — this is the entire difficulty estimate
for removing the exemption); the fallback-arm census figures (2,586 / 408 / 24 / 198).

**Known citation defect in the source audit:** PR #5585 was cited for the emitted-`panic!`
claim; #5585 is base64/RFC 4648 in `std.encoding`. The claim is true, the receipt was
fabricated. (§3 citation class.)

## 11. Audit queue

1. ~~Recover `docs/error-examples.md`~~ **DONE — see §8b.** Recovered intact; 7 cases, all
   Tier 2+, zero Tier 1. Still to pull: the rest of `docs/thesis/` —
   `correctness-dimensions`, `what-falls-out`, `two-groundings`, `the-derived-homomorphism`.
1b. **Author the missing Tier 1 RED controls** (new, and the cheapest item here). They never
   existed. Start with the cardinality archetype, method existence, and declared-return
   conformance — each a three-line `.dag` program with an expected-error acceptance criterion,
   in the recovered corpus's own format.
2. Re-measure the floor histogram, bucketed **statically-decidable vs genuinely-runtime**.
   This is an execution, not a design, and it is what denominates the work in displaced cost
   (§6) instead of elegance.
3. Audit Tier 3 (L4–L7) and the unverified Tier 1 rows.
4. Locate or refute the 104-false-positive claim.
5. Reconcile the recovered claims list against the DESIGN.md open thread *"enforcement intent
   — ask once, compile forever."* That thread proposes minting `StandingIntent` to enforce
   claim⇄coverage. The claims list already carried that rule in prose — *"If a claim IS here
   but the ROADMAP has no track for it, that's a gap"* — so the thread should be **grounded
   on the recovered list, not re-coined** (§3).

## 12. Proposed sequencing (for discussion, not yet agreed)

**Step 1 — restore the specification.** Re-home the guarantee statement, the tier claims, and
the claims-list completeness rule into DESIGN.md. Mostly transcription of text already in git
history, annotated with §4/§5 status. This is first because §2 explains why nothing else
holds without it.

**Step 2 — the gap census against that**, replacing this draft's UNVERIFIED rows with
measurements.

**Step 3 — walls, in dependency order.** Note two orderings that differ from the obvious one:

- **Primitive single-authority precedes the method-existence wall.** Method existence is
  currently spread across ≥5 name sets (registry, `PrimitiveContract` rows, algebra
  templates, interpreter arms, per-target templates). A wall built against any one of them
  reproduces #7479 inverted — correctly refusing `filter_map`, wrongly refusing what
  legitimately resolves elsewhere.
- **The `v2.*` exemption comes off only after the 104 are re-adjudicated.** Otherwise the
  likely outcome is that it goes back on.

DESIGN.md is untouched by this note; it is the load-bearing authority and the §1/§4 addition
needs agreement on shape first.
