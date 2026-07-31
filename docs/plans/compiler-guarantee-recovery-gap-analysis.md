# Compiler guarantee recovery — gap analysis

**Status:** reconciled analysis — three audit passes complete (initial archaeology 2026-07-30;
independent review + receipt verification 2026-07-31; post-merge verdict adopted 2026-07-31 —
seed rungs demoted to specimen-denominated honesty, capability premise corrected to the
`sole_constructor` audit, ledger in §10). No code lands from this
note. Open by design: §12 sequencing awaits operator sign-off; prevalence numbers are
deliberately unmeasured until Stage 7.
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
upstream half — validation — was never completed as an exhaustive acceptance contract (real argument checks, match machinery for resolved types, constructor arity, the v2 loop wall, and the lens door all exist; what never existed is the total obligation set a program must discharge before `Accepted`).

## 1b. The safety ladder (operator framing, 2026-07-31 — the organizing frame for every class in this note)

The guarantee statement's operational form is a ladder that every discovered error class
climbs. It is DESIGN §1's safety axis made kinetic: safety is time-to-recover from a silent
wrong answer, paid later at interest — so each rung moves detection of the *same* defect
earlier and cheaper, and the top rung means the defect is never paid for by anyone.

| Rung | Name | Who pays, and when |
|---|---|---|
| **R3** | Structurally impossible | No one, ever — the bad state has no writable form (construction / derivation / sealed carrier) |
| **R2** | Structural guarantee | The author, at compile time — typed, located refusal |
| **R1** | Testable / validatable / preventable | CI, before ship — witness, lens, RED control |
| **R0** | Mitigatable | The operator, at runtime — typed refusal, budget, rollback |
| — | **below the floor: silent** | The end user, at interest, unknowingly — **forbidden outright (§5)** |

Three rules make it a discipline rather than a diagram:

1. **The floor is absolute.** No class may sit below R0. A silently-wrong behavior — the
   misspelled label that binds positionally and computes the wrong answer with no signal —
   is not a low rung; it is outside the ladder, and §5 already outlaws it. The first move
   for any discovered class is *to the floor*: loud, typed, counted. Climbing starts after.
2. **Every class climbs to its ceiling, and states its ceiling.** Ceilings come in exactly
   three kinds: **mathematical** (undecidability, the external-reality boundary,
   root-authority self-governance — permanent and honest), **capability** (R3 for
   carrier-borne invariants requires unforgeable construction — `sole_constructor` is the
   *existing* candidate wall: a cross-module construction refusal with its own
   `SoleConstructorViolation` diagnostic and a sealed-record test specimen; its completeness
   for generic carriers like `Refined<B>`, for all construction forms, and under the
   compiler-module exemptions is **unverified**, so this ceiling is an audit finding, never a
   declared absence — corrected per post-merge review 2026-07-31, which refuted an earlier
   revision's claim that no such capability exists), and **price** (deferral compounds in
   sites — `Value::Null` at 131 — so lateness raises the bill but never closes the door).
   "Climb the ladder" always means *to the stated ceiling*, not unconditionally to R3.
3. **Reported rung == measured rung, at a declared subject grain.** A rung is established
   by an executing probe, never asserted — and it is measured against a *declared acceptance
   boundary*. Source→interpretation, source→each emission target, and phase→phase are
   structurally distinct paths with independently different rungs (the interpreter refuses
   the mislabeled call that the emitter's `order_typed_call_args` reorders into positional
   leftovers), so a class's current rung is the **minimum across its in-scope paths**, or one
   row per path; citing the strongest path while another stays silent is inflation. Rung *inflation* — v2's `CanonicalGrounding` reporting R3 while occupying R0;
   the historical ledger's "Yes (blocking)" — is worse than sitting low, because an inflated
   class never ranks for climbing. A lens reds inflation and reds stalls (below ceiling
   with no named next-rung trigger).

**How this compiler makes programming safer than a traditional one — the two ends of the
ladder.** A traditional compiler holds a fixed, non-negotiable set of classes at R2 (types,
syntax, constructor arity), leaves everything else at R1 (hand-written tests) or below the
floor (silent), and barely uses R3 at all. The bet here is at both ends: the **floor is
enforced** (§5 — nothing silent survives), and the **top is open** — the R2 set is
user-extensible by rows rather than compiler forks (§7's language-design-opens-up), and R3
is a *working* rung (derive-don't-store, closed algebras, sealed construction pending the `sole_constructor` completeness audit). The recovered `error-examples.md` cases — `reverse |> reverse`,
non-idempotent retry, O(n²) — are ordinary classes *lifted to R2* that no mainstream
compiler holds there. The scandal this note documents is the inversion: the ambition
examples sit above mainstream while method-existence and return-conformance sit below it.

### The conversation's own specimens, placed on the ladder

| Class | Floor status | Today | Ceiling | The climb |
|---|---|---|---|---|
| Misspelled label binds positionally (§4) | **BELOW FLOOR — silent wrong binding** | — | R2 | floor first (any diagnostic), then the exact call-shape judgment |
| Unknown method (`filter_map`, #7479) | at floor on the interpretation path | R0 on source→interpretation (typed `Unimplemented`, the lab 500); **emission-path disposition unmeasured** | **R3 in the accepted model** — a resolved call carries a unique declaration identity, so an unresolved name has no accepted form; the author experiences it as R2-style refusal at the source boundary | zero-resolution wall now; ambiguity wall after the identity join; `FunctionRef`-carrying IR |
| Return / `data` / generic inhabitance (#7481 probes) | **floor status UNPROVEN** | **UnknownUnmeasured** — compile admission proven (#7481); a typed runtime mitigation is not proven for the class, and a silent path (into `==`, serialization) is not excluded. *Demoted from R0 per post-merge review* | R3 (typed Arrow-body boundary) | return-position checking (wall-after-grounding) |
| Empty list into requires-one (the operator's case, §4b) | **split unmeasured** | **UnknownUnmeasured** — "runtime surprise or degenerate arm" spans R0 (typed rejection) and below-floor (silent degeneration); which occurs where is unmeasured | **R3** — the bad *seam* becomes unwritable | seal `Refined` (capability ceiling) + propagate through `InterfaceSummary`; expecting-red probes land first |
| **Numeric-tower** cross-representation `==` | **was below floor** (silent `false`) | **R3** — grounded by construction (#5428), guard dead-in-corpus for numerics | R3 | **done — the exemplar full climb, floor to top** |
| Cross-representation `==`, full class (`Bool` straddle; `Optional`/`Witness` over `Value::Null`) | **floor status UNPROVEN** | **UnknownUnmeasured / below-floor-permitted** — the typed backstop covers the *numeric* straddle only; `Bool` has no wall (and no corpus `==` site); `Optional`/`Witness` cannot use a blanket guard because `Value::Null` conflates legitimate `false` with a representation straddle. *Split from one "R3 done" row per review 45310, then demoted from R0 per post-merge review — the backstop does not reach this class* | R3 — each primitive grounded in its realization | the `Value::Null` split (its own runway, `value-null-split` plan) |
| v2 loop termination | — | **R2** (undeclared measure refused) with R3 character (unbounded iteration has no substrate spelling; recursion is sugar over `Loop`) | R3 | done in v2; the v1 seed's classes climb by deletion, not hardening |
| Idempotency-unsafe retry | at floor | R1-capable — algebra declared, consumers exist, not at binding sites | R2 | the dimension migrates onto `TypeBinding` (§8c) |
| Non-exhaustive match via blocked lookup | below-floor candidate | **UnknownUnmeasured** — compile-time silence proven (`diagnostics: []`); the shipped match's runtime disposition unmeasured | R2 | `ExhaustivenessUnknown` refuses instead of passing as success |
| Doc orphan (this PR's own incident) | was below floor — silent rot, twice historically | **R1** — `ReadsLiveTree` witness; it caught this very PR | R2-ish (registration-derived roots) | live proof that climbing pays: the wall that failed open twice, repaired, did its job |

### Non-goals — ceilings that are low BY DESIGN, stated so they are never mistaken for gaps

- **External reality** (host state, network, tool presence): outside the program graph *by
  the guarantee statement*; the ceiling is a typed refusal at the boundary — R0-at-boundary
  is correct there, not a deficit.
- **Arbitrary refinement predicates** (`admits: fn(B) -> Bool`): undecidable; ceiling is the
  runtime constructor boundary. The decidable cardinality *fragment* climbs instead — that
  split is the whole design of `cardinality-refinement.md`.
- **Resource / budget exhaustion** (`EvalBudgetExceeded`, ARG_MAX, witness-wall budgets):
  R0 by design — the bound *is* the product.
- **Optimality**: ratchet forever (Rice); §5 already names "never" as the trap.
- **Root-authority self-governance**: the substrate cannot wall edits to its own axioms;
  operator review is the permanent mechanism, not a temporary one.
- **Byte-identical self-emission**: retired non-goal — behavioral equivalence replaced it
  (DESIGN §7) precisely to avoid cementing the seed's accidents.

**Landing target:** this section is drafted here for operator sign-off and lands as
`gunbc.design_document` rows projected into DESIGN.md beside §5's construction/validation
doctrine (§7b discipline — never a hand edit of the markdown). The §3 status lattice below
refines the rungs (e.g. `RepresentableButForgeable` = R1 wanting R3 behind the capability
ceiling; `LiteralOnlyWall` = R2 on a fragment); the claim carrier of §11/§12 gives every
class a **derived disposition** (folded per path from executed measurements — no stored rung
field exists to inflate; see §12 Stage 1b, corrected per review 45367), `ceiling` (with its
justification kind), and `next_rung_trigger`.

## 1c. Current rung census (seed population for the Stage-1 carrier — status `Unknown` where this audit has not measured)

The operator's reorganization directive (2026-07-31): the ladder is the mechanism, the
historical tiers are *example domains*. These rows are the draft population for the Stage-1
`ErrorClassGuarantee` carrier; every `Unknown` is honest unmeasured state, not a gap claim.
Floor-domain failures are **below-baseline safety regressions** — never compensated by
higher-order capability. **Specimen-denominated (post-merge review, 2026-07-31):** every rung
below is established only by the cited specimen on the cited path; coverage across the
class's population and its other paths (interpretation vs each emission target) is unproven
unless the row says otherwise, and `Unknown` is the honest default.

**Domain: ordinary compiler safety floor**

| Class | Current rung | Ceiling | Why not higher | Evidence | Next trigger |
|---|---|---|---|---|---|
| Unknown fn/method | **Main:** compile-time method existence remains unenforced outside existing paths (R0 on the interpretation specimen; compile and emission paths BF/U). **Open candidate gunbc#7484** (unmerged): narrow R2 coverage for established receiver surfaces plus a typed, countable `MethodExistenceUndecided` frontier elsewhere — candidate implementation evidence on an open branch, never a guarantee held by main *(the prior revision of this row said "landed"; corrected per the post-merge verdict — rung inflation via open-PR state)* | R3 (resolved call carries declaration identity) | fabricating fallback live on main; the candidate's frontier is `FrontierAccepted`-shaped, not a wall | #7479 · open gunbc#7484 | receiver normalization → zero-resolution refusal over the union of current admissible sources (identity join gates only the >1 half); `FunctionRef` IR |
| Call shape (labels/count) | **partly below floor** (misspelled label binds positionally, silent) | R3 (exact bijection in normalized IR) | formal-driven walk; `ArityMismatch` is constructor-grain | `direct_call_arg_mismatch_diags` | floor diagnostic first, then exact-shape judgment |
| Return conformance | **UnknownUnmeasured** (compile admission proven; runtime disposition and silent paths unmeasured) | R3 (body edge inhabits Arrow codomain) | no general judgment | #7481 | return-position checking |
| `data` annotation | **UnknownUnmeasured** (same basis) | R3 | same lane | #7481 | same lane |
| Generic instantiation | **UnknownUnmeasured** (same basis) | R2 | substitution unproven | #7481 | inhabitance at instantiation |
| Field through generics | **UnknownUnmeasured** (same basis) | R2 (pending-constraint discharge) | `field_of_type_var` minted | §4 | constraint carried + unique discharge |
| Closed-match exhaustiveness | below-floor candidate / **UnknownUnmeasured** (compile silence proven; runtime unmeasured) | R3 (full arm population at elimination) | `PatternLookupBlocked => []` | §4 | `ExhaustivenessUnknown` refuses |
| Record completeness | **Unknown — unmeasured** | R3 | not audited this pass | — | audit probe |
| Parse: list separator dropped | **Below floor — silent** (measured: `[ {a}, {b}  {c} ]` compiles with zero diagnostics — a dropped comma is a silent semantic change, two- vs three-element list; survived regen, whole-corpus compile, fixed-point verify and a 15-case matrix, caught only by a human diff read) | R3 (decidable grammar fact) | separator omission parses as element juxtaposition | tidy-deer-730 probe on gunbc#7484 + review 45347, 2026-07-31 | probe pair in the corpus; refusal in the list production (§11 item 6) |
| Producer/consumer cardinality | **UnknownUnmeasured** (typed-rejection vs silent-degeneration split unmeasured) | R3 (seam unwritable) | forgeable carrier; no signature propagation (`sole_constructor` audit pending) | §4b | Stage-3 vertical slice |

**Domain: structural safety extensions (the differentiator)**

| Class | Current rung | Ceiling | Why not higher | Evidence | Next trigger |
|---|---|---|---|---|---|
| Termination | v2 loops R2 (R3 character: unbounded iteration has no substrate spelling); v1 recursion **Unknown** (thesis-era "421 non-blocking") | R3 | v1 coverage unmeasured | `v2.std.cardinality` | universal recursive-call lowering |
| Retry × idempotency | R1/R2 partial (algebra declared; `Retry` emit refuses shapes, #7303/#7318) | R2 (`Retry` admits only retry-safe evidence or compensation) | not at binding sites | `std.effects` | dimension onto `TypeBinding` (§8c) |
| Complexity budget | R2 for enrolled classes (accumulator-copy in the compile door; bare-minimum-cost rule) | R2 | coverage partial | door roster | contract propagation |
| Exponential branching | **Unknown** | R2 (against declared budget — exponential can be intentional) | unmeasured | error-examples #7 | measure, then gate |
| Algebraic cancellation | **Unknown, likely absent** | R2 (derive where algebra declared) | unmeasured | error-examples #6 | algebraic-rewrite lane |
| Redundant cross-service computation | R1 (duplicate-work design; flagship proven) | R2 | `ComputationIdentity` staged | DESIGN open thread | Half A/B landing |
| Partial-failure convergence | R2/R3 partial in-model (`MemberTeardownRefused` has no effect arm — teardown-of-unowned unwritable) | R3 in-model; external residue stays boundary | grain expansion pending | membership-reconcile | remaining grains |
| Numeric-tower cross-representation `==` | **R3 — done** (exemplar climb) | R3 | — | #5428 | backstop guard dissolves with the Null split |
| Cross-representation `==`, full class (`Bool`; `Optional`/`Witness` over `Value::Null`) | **UnknownUnmeasured / below-floor-permitted** (backstop is numeric-only; `Bool` unwalled; Null conflated) | R3 | Null split is its own runway | value-null-split plan | `Value::Null` split *(split per review 45310; demoted from R0 post-merge)* |
| L4: emitted vs modeled behavior | **UnknownUnmeasured** — §6 records L4–L7 unaudited; the witness-realization lane exists but class coverage is unestablished *(was R1; contradiction with §6 caught post-merge)* | R2 for supported forms | derivation not proof | witness-realization plan | realization proof |
| L5: cross-target equivalence | **Unknown** | R1→R2 | unmeasured | — | witness matrix |

**Domain: external boundary (low ceilings BY DESIGN — correct, not gaps)**

| Class | Rung | Note |
|---|---|---|
| Host/tool availability | R0 typed at the boundary | no higher rung claimed |
| Resource/budget exhaustion | R0 by design | the bound *is* the product |
| Optimality | outside the ladder | Rice; permanent ratchet |
| Unstated business intent | outside | never fabricated |
| Unmodeled upstream behavior | outside | observe → typed evidence → admit or refuse |

## 1d. Provisional guarantee grid (operator post-merge verdict, 2026-07-31 — hand-authored interim)

**This section is the interim management surface the verdict asked for — "which guarantees
do we require now, how unsafe is `main`, what do the open PRs change, and what exact
dependency chain raises each class" — emitted by hand until `ladder-census-emitters`
projects it from the claims carrier. Dissolve-on: the carrier-emitted grid lands; this
section then deletes rather than being maintained beside it (§3). Open PRs are candidate
evidence, never `main`'s rung.**

Legend: **BF** = silent/below floor · **F** = typed and counted frontier still accepted
(`FrontierAccepted`) · **U** = unmeasured · **R0** = runtime refusal/mitigation · **R1** =
required gate or test · **R2** = compile-time structural refusal · **R3** = invalid state
structurally unwritable.

| Rank | Guarantee | `main` today | Effect of open P0s | Minimum required now | Ceiling and dependency path |
|---|---|---|---|---|---|
| **P0.0** | `Accepted` means all applicable modeled judgments were established | **BF/U** — no exhaustive acceptance contract exists | Neither PR closes the meta-contract | **R2** | R3 via distinct phase carriers. Probe corpus → claim/path/measurement authority → every required consumer → acceptance-completeness door |
| **P0.1** | Parsing preserves required separators and structural formation | **BF** — dropped list separator silently changes the program | Not owned by #7484/#7485 | **R2** | R3 canonical parse structure. Baseline → parse-formation wall |
| **P0.1** | Calls have exact labels, count, defaults, and parameter binding | **BF** — misspelled label can bind positionally | Unchanged | **R2** | R3 normalized exact-bijection invocation. Baseline → call-shape wall → compiler-source exemption deletion |
| **P0.1** | Callable/method existence | R0 on the known interpretation specimen; compile paths remain BF/U | Open #7484: candidate R2 on established receiver surfaces, F elsewhere | **R2 on every compile/emit path** | R3 resolved identity. Receiver normalization → zero-resolution refusal over the union of current admissible sources; identity join → ambiguity refusal |
| **P0.1** | Function body and `data` value inhabit declared types | **U/BF candidate** — bad specimens compile | Open #7484 carries a narrow ground-scalar/container candidate fragment | **R2 for every grounded declared type** | R3 typed Arrow/data construction. Conformance grounding → returns/data → exemption deletion |
| **P0.1** | Generic instantiation, required record fields, and defaults are sound | **U/BF candidate** | Not established by either P0 | **R2** | R3 typed construction. Conformance grounding → generic instantiation + record-construction wall |
| **P0.1** | Field access has a receiver proven to carry that field | **U/BF candidate** — `field_of_type_var` fabricates | Unchanged | **R2** | R3 field-carrying bound. Baseline → generic-field constraint wall |
| **P0.1** | Closed variants eliminate exhaustively | **U/BF candidate** — blocked lookup returns no diagnostics | Unchanged | **R2** | R3 full arm population at elimination. Baseline → `ExhaustivenessUnknown` refusal |
| **P0.1** | V2 never treats source structure as inferred semantic fact | **BF** — generic self-grounding remains on main | Open #7485: removes exact self-evidence; infer becomes **F**, Eval reaches R2 for its specimen, Translate remains open | **R2 in both Eval and Translate** | R3 distinct inferred carrier. Self-grounding slice → Translate propagation → all-derived `InferredTree` → derivation coverage |
| **P0.1** | Every executable operation has a realization for the selected target | **U/BF candidate** | Explicitly out of scope for #7485 | **R2 target-relative refusal** | R3 `RealizedTree<Target>`. Identity/realization join + true inferred carrier → target realization gate |
| **P0.2** | A `0..n` producer cannot feed a `1..n` consumer | **U; representable but forgeable and unpropagated** | Unchanged | **R2 for the seam** | R3 unwritable seam. `sole_constructor` audit → interval lattice → transfer functions → `InterfaceSummary` contract → sealed refined construction |
| **P1** | Termination, retry/idempotency, complexity, algebraic laws, fidelity | Mixed: v2 Loop is R2; most others partial or U | Mostly unchanged | R2 where modeled and decidable | Generic dimension mechanism + binding propagation + realized-target evidence |

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

## 4. Domain: the ordinary compiler safety floor (historical Tier 1 — "impossible to write the bug")

| Claim (THESIS.md) | Today | Receipt | §5 class |
|---|---|---|---|
| Type mismatches caught at compile time | **UNENFORCED** (return position, `data` annotation, generic instantiation) | `fn f() -> Int { "not an int" }` typechecks — probed by execution, PR #7481; argument position *is* checked | wall after grounding |
| …in compiler source | **UNENFORCED by exemption** | `v1.compiler.infer` `module_skips_direct_call_arg_check` — skips `v2.*` and `v1.compiler.*` | wall now |
| Field typos | **PARTIAL** — concrete types checked; through a type variable, not | `v1.compiler.infer` mints `TypeVariable { id: "field_of_type_var" }` instead of refusing | wall after grounding |
| Application arity / call shape (missing, extra, misspelled-label args) | **fail-open by construction of the walk** | `v1.compiler.infer` `direct_call_arg_mismatch_diags` is *formal-driven*: per formal it seeks a same-named arg, else falls back to the **positional** arg at the same index (a misspelled label silently binds by position if the type fits), and `Absent => []` (missing arg → no diagnostic); extra args are never visited. The `ArityMismatch` diagnostic is **type-constructor** arity ("expects N *type* arguments"), not invocation arity — invocation arity has no compile diagnostic; #6896's wall is runtime-only | wall now |
| Non-exhaustive matches | **PARTIAL — one confirmed silent arm** | resolved coproducts have exhaustiveness machinery; but `v1.compiler.infer_patterns` `lookup_variant_in_type` / `lookup_field_in_variant` both have `PatternLookupBlocked => node_lookup_failed(diagnostics: [])` — a blocked scrutinee lookup fails with **zero diagnostics** and the pattern types as `error_type` (`PatternDynamic`, by contrast, does diagnose at these sites). "Exhaustiveness not established" is treated as success-adjacent, not refused | wall after grounding |
| Cardinality / multiplicity (empty list into a callee that requires one) | **RepresentableButForgeable, not statically propagated** — reclassified from UNEXPRESSIBLE after independent review | Representation exists: `v2.std.refinement` `Validation<B>`/`Refined<B>`/`refine`, a `NonEmptyList<T>` manual fixture (`v2.test.claim.manual.refinement_nonempty_list` + testgen anchor), and manual value-level algebra specimens (`cardinality_fold_propagation_test` — length homomorphism over literals + runtime `refine_byte`; no binding-level propagation). Forgeable: `Refined<B>` is a public record — `refined_vacuous_stub_pack`'s `Rejected` arm literally returns `Refined { base }`, so the carrier proves nothing about validation. Not propagated: no cardinality lattice in signatures (`v2.std.cardinality` is loop-termination), `InterfaceSummary` (`dag/std/interface_summary.dag`) carries no cardinality slot, no transfer functions across `map`/`filter`/`concat`. The substrate `Cardinality` connective remains production-uninhabited and v1 forks the name onto optionality (`Required \| CardOptional`) | wall after grounding |
| Method existence | **UNENFORCED — fabricates** | `v1.compiler.infer` `method_pipe_map_keys_values_fallback` else-arm returns the *receiver type* with `kernel_diags: []`; unresolved method stamped `PlainMethodSemantics` | wall now |
| Grounding completeness — "**not** a name-keyed table lookup" | **VIOLATED literally** | `v1.compiler.infer_method` `builtin_function_registry() -> Map<String, Node>` is a name-keyed table (~120 entries), one of ≥5 independent primitive-existence authorities | wall after grounding |
| Circular deps / stale imports / cross-target drift | **UNVERIFIED** | not yet audited | — |
| CX gate: every recursive fn terminates with a proven bound | **UNVERIFIED** | `DescentEvidence` exists (`dag/std/termination.dag`, fail-closed to `DescentUnknown`); enforcement scope unaudited | — |
| Ownership: no aliased mutation in emitted code | **UNVERIFIED — known latent fail-open** | DESIGN.md open thread: emitter silently wraps every `shared_types` member in `Rc<T>` | — |

### 4b. The cardinality case was independently re-derived in-tree — the intent is not lost, the design pass is

The sharpest discovery of the reconciliation pass:
`docs/plans/interface-summary-declared-use-arity.md` **§3.1 "Value-cardinality arity at
seams (operator direction 2026-07-04)"** records the operator's exact scenario — *"the
canonical divergence being one side honoring the empty list while the other assumes
non-empty. Today that disagreement is a runtime surprise or a silently-degenerate arm; **the
operator wants it a hard error in the language.**"* — with the seam design sketched (the
invariant is an interface fact in `InterfaceSummary`'s contract slot; a `0..n` producer
feeding a `1..n` consumer is *"a located type mismatch at the boundary — unwritable, not
reviewed"*). Its FLAG E names what is missing: the cardinality lattice itself, deliberately
not designed there. `docs/plans/cardinality-refinement.md` (status: scoping; registered as
`gunbc.plans.cardinality_refinement`) carries the lattice proposal — a **closed decidable
predicate vocabulary** (`Length<N>`, `NonEmpty`, `Bounded<Lo,Hi>`, `Width<N>`, all linear
arithmetic over counts, chosen precisely to stay inside §4's bounded substrate where general
refinement would not) plus fold-propagation, whose std seed already has green witnesses.

So the thesis-era guarantee was dropped, and then **the operator re-derived it from scratch
on 2026-07-04** — which is simultaneously the strongest evidence that the guarantee is real
(it keeps being wanted independently) and of what the missing specification costs (it had to
be re-invented rather than consulted, and 27 days later the design pass has not started
because nothing ranks it). The salvage for this axis is therefore **not new design**: it is
connecting three artifacts that already exist — the operator direction, the scoped lattice
plan, and the manual value-level specimens — and closing the two named gaps (unforgeable
construction; signature-level propagation through `InterfaceSummary`).

## 5. Domain: runtime-safety floor obligations (historical Tier 2 — "proven safe or total")

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

## 6. Domain: realization and fidelity (historical Tier 3 — verification from structure)

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

**Calibration added after independent review — the terminal is better than the phase
carriers.** Three nuances that bound the claim:

- `validate_then_compile` (`v2.compiler.00_compile`) is a real single gate authority
  ("no second gate surface exists"): the root roster (`accumulator_copy`, `determinism`,
  `machine_shape`, `mandatory_tag`) plus the per-node roster (`fact_density`,
  `unit_modeling`) fire on every compile before an output exists. The staging *shape* is
  worth keeping.
- The evaluator refuses unrealized semantics (`v2_eval_semantics_deferred`) rather than
  fabricating results, and `canonical_grounding_for_node` does impose a coherence gate
  (`node == facts.grounding.node`) — though a self-certified fact passes that gate *by
  construction*, so it screens malformed trees, not fabricated groundings.
- v2 **loop termination is a genuine construction wall** (`v2.std.cardinality`: a loop
  without a declared bound measure gets `cardinality_loop_bound_undeclared`; descent
  fail-closed to `DescentUnknown`).

So the correct diagnosis is not "v2 inference is fake"; it is: Branch/Match/Loop carry real
semantic work, loop termination is walled, the *generic* arm self-certifies, the terminal
may still refuse later for other reasons — and therefore **`InferredTree` is not a
trustworthy proof boundary**: its lens consumers read facts whose generic entries were never
derived. The fix is the phase-carrier separation (§12 stage 4), not a rebuild of the
terminal.

## 7b. Where the recovered specification must land — and proof the prose already drifted

Two facts that redirect step 1 of the sequencing:

**DESIGN.md is a projection, not the authority.** `dag/gunbc/design_document.dag` carries the
document's content as data rows; DESIGN.md is generated from it. So "re-home the guarantee
statement into DESIGN.md" is mis-aimed as stated — the guarantee lands as **`.dag` rows**
(the claims authority of §11), and the DESIGN.md text is derived. Hand-editing the markdown
would mint a parallel representation of exactly the kind §3 forbids.

**The prose has already drifted from the substrate, on the substrate's most load-bearing
sentence.** DESIGN.md §4 says the closed vocabulary is *"6 connectives + 5 behaviors"*.
`v2.std.node` declares **six** behaviors: `Value | Transform | Branch | Loop | Bind |
Match`. The recovered thesis is sharper about what that means: it listed five behaviors and
declared *"Substrate extension is a C1-class stop signal (seventh connective or **sixth
behavior**) — all four dissolution patterns … must fail with structural arguments before
extension is allowed."* Whether `Match`'s promotion to a behavior was adjudicated under that
rule is not established here (it may well have been — v2's `Match` inference is real work);
what is established is that the live authority's count is wrong against the live substrate,
which is a clean specimen of why the guarantee must be modeled where a lens can read it,
not written where only a reader can.

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
// Expected on landing: located refusal at the seam — a 0..n producer bound to a
// 1..n consumer position (interface-summary-declared-use-arity.md Sec 3.1).
// Today it compiles. The RED exercises unforgeable construction + seam
// propagation — NOT absence of representation: Refined<List<T>> exists and is
// forgeable (Sec 4/4b). (Corrected per review 45305 — an earlier draft said
// "unexpressible" here, which would have aimed the fix at inventing a carrier
// instead of sealing and propagating the one that exists.)
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

**A binding-site integration point exists and one dimension prototype reached it**, verified
on `main` — stated at that grain because the stronger claim an earlier revision made here
("the mechanism was built, not just designed") was refuted post-merge: `TypeBinding` carries
two *bespoke* coordinates, not a generic dimension population — there is no lattice registry,
no dimension-keyed fact map, and no user-defined dimension path. Whether the generic,
user-extensible architecture is recoverable by *extension* rather than redesign is an open
audit question, and it materially affects the pricing of every migration row below.

- `v1.compiler.infer_env` `TypeBinding { name, resolved, provenance: SubValueRelation }` —
  the thesis names `TypeBinding.provenance`, and it is there.
- `v1.compiler.core` `ExprCall { call_semantics, descent_evidence: List<SubValueRelation>? }`
  — the thesis names `ExprCall.descent_evidence`, and it is there.

So the *integration point* the thesis describes is real and load-bearing for two coordinates.
What did not happen is the generic mechanism, or the rest of the dimensions moving anywhere. Restated in the thesis's own table format
(*Carried on bindings* / *Enforced* are the load-bearing columns):

| Dimension | Declared today | Carried on bindings | Enforced |
|---|---|---|---|
| Type safety | `dag/std/types.dag` | `TypeBinding.resolved` | **Partial** — argument position only; return, `data` annotation, generic instantiation unchecked; `v2.*`/`v1.compiler.*` exempt |
| Termination | `dag/std/termination.dag` (BoundedLattice, bottom = fail-closed) | `TypeBinding.provenance`, `ExprCall.descent_evidence` | **UNVERIFIED in v1** — thesis-era status was "Partial, 421 violations, non-blocking"; **walled in v2 for loops** (`v2.std.cardinality` bound measure) |
| Cardinality / multiplicity | `v2.std.refinement` (+ scoped lattice plan `gunbc.plans.cardinality_refinement`) | No | **RepresentableButForgeable** — see §4/§4b; operator re-directed 2026-07-04 |
| Ownership | `src/v1/ownership.dag`, `src/v2/lens/ownership.dag` | No — still a separate pass | **Partial**, plus a known latent fail-open (Rc wrap) |
| Side effects | `dag/std/behavioral.dag`, `dag/std/effects.dag` | No | Consumers now exist (`std.effect_grant`, `std.realization`, `gunbc.host_effect`) — an improvement on the thesis-era "declared, not consumed" — but **not at binding sites** |
| Idempotence | `dag/std/effects.dag` (lattice from `EffectShape`) | No | No |
| Purity | not declared | — | No |
| Space bounds | not declared | — | No |

**Why this matters for sequencing:** two coordinates reached the binding carrier and it still
works, so the binding site is a *viable* integration point — but whether salvage is
"finish a stalled migration" or "build the generic mechanism first" is exactly the open audit
question above, and the roadmap must price both branches rather than assume the cheaper one.

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

**Post-merge verdict ledger (third pass, all checked on `main`):** CONFIRMED —
`sole_constructor` exists (`SoleConstructorViolation`, "cannot be constructed outside its
defining module", sealed-record specimen in the generated corpus); my §1b keyword-set
inference ("no such capability") is WITHDRAWN — the keyword table was the wrong place to
look, `sole_constructor` is a type-decl modifier. CONFIRMED — `order_typed_call_args`
(`v1.compiler.emit`) reorders recognized labels to signature order on the named-only path,
so interpretation and emission are structurally distinct acceptance paths for the call-shape
class. CONFIRMED — `cardinality_fold_propagation_test` is value-level manual algebra (length
homomorphism over literals; runtime `refine_byte`), not compiler propagation; the "green
fold-propagation witnesses" phrasing was rung inflation on evidence and is withdrawn.
ADOPTED — every seed rung not established by executed evidence on a declared path demoted to
`UnknownUnmeasured`; the census is specimen-denominated until coverage is proven.

**Independent-review verification ledger (2026-07-31 pass, all checked on `main`):**
CONFIRMED — `Refined<B>` forgeable incl. the `stub_pack` `Rejected => Refined { base }` arm;
five advisory `WhereRefinementUnenforced` deferral reasons; `NonEmptyList` manual fixture +
testgen anchor; green `cardinality_fold_propagation_test`; `direct_call_arg_mismatch_diags`
formal-driven walk incl. positional fallback for misspelled labels; `ArityMismatch` =
type-constructor arity; `validate_then_compile` single-door roster exactly as stated;
`v2.std.cardinality` loop-bound wall; exemption introduction 2026-06-08 (pre-bankruptcy);
six behaviors in `v2.std.node` vs DESIGN.md's "5"; DESIGN.md projected from
`gunbc.design_document`; operator direction 2026-07-04 in
`interface-summary-declared-use-arity.md` §3.1. CORRECTED against the review —
`PatternDynamic` *does* diagnose at the variant/field lookup sites (`VariantNotFound` /
`FieldNotFound` with `type_name: "unresolved"`); the silent `diagnostics: []` arm is
`PatternLookupBlocked` specifically. PRECISION — the `0..n`/`1..n` seam is an operator
*direction* recorded inside a signed lane, with its lattice design pass (FLAG E) explicitly
not started; "operator-signed design" slightly overstates it. STILL UNLOCATED — the "104
TypeMismatch false positives" (no receipt in tree or history found by either pass), and the
specific historical wording "branch, argument, and return type checking DONE" (the
`correctness-dimensions.md` *"Yes (blocking)"* row is the receipt actually in hand).

**Restructure-pass ledger (2026-07-31, fourth pass):** ADOPTED with provenance —
tidy-deer-730's measured receipts from the gunbc#7484 lane: the parser silent-separator
specimen (`[ {a}, {b}  {c} ]` compiles with zero diagnostics; caught only by review 45347's
human diff read after surviving regen, whole-corpus compile, fixed-point verify, and a
15-case matrix), the kernel-profile/interpreter-dispatch fork (`String`/`Map` declare
`length` but not `count`; the interpreter dispatches `length`/`count`/`size`), and two
receiver-type resolution defects gating method-wall promotion (where-refinement alias
unpeeled to base; pattern-destructured payload typed as its variant name) — landed as
census/queue rows with the dependency corrected: the identity join **precedes** the general
method wall (§12 Stage 2 amendment). CORRECTED per review 45367 — Stage 1 listed
`current_rung` as a stored carrier field in two places while the roadmap node said no such
field exists; the stored-field reading is withdrawn everywhere (disposition is derived,
§12 Stage 1b), and the DESIGN §4b tail's "per-class current rung" phrasing re-projected to
match. STRUCTURAL (operator verdict on gunbc#7489) — the ladder roadmap nodes moved inside
`declared_roadmap_nodes()`/`declared_roadmap_edges()` under lane owner `compiler-guarantee`
(the section-local list was a second roadmap universe — uncounted, receipt-unreachable,
invisible to the identity/endpoint/acyclicity witnesses; review 45356 independently flagged
the owner-less rows the placement forced), with rendered⊆declared∪derived now witnessed
plus a synthetic-ghost RED; `HandAuthoredDocBind` re-typed `primary_work +
additional_works` so the zero-anchor bind is unwritable (the empty-works RED deleted with
the validation it backed, its residue named on the carrier note), the two remaining
duplicate `(home, slug)` pairs merged, and bind-identity uniqueness witnessed with a
synthetic-duplicate RED.

**Post-merge-verdict ledger (2026-07-31, fifth pass — on the merged #7489, verified against
live state before adoption):** CONFIRMED AND CORRECTED — the census and roadmap said the
narrow method wall "landed on gunbc#7484" while #7484 is **open, unmerged, and grown to a
43-file head**: rung inflation via open-PR state, my transcription of the implementing
session's "landed" without checking merge state — every site now carries open-candidate
wording, with main's disposition stated separately (the ladder's own honesty rule applied
to PR state). ADOPTED — baseline prevalence anchored to `anchor_commit 6c6e2dcb8587`
(content-addressed, reproducible after in-flight merges). ADOPTED — `FrontierAccepted`
added as the disposition's fifth state (typed/located/counted diagnostic, phase result
still `Accepted`; specimens `MethodExistenceUndecided`, `GroundingNotDerived`). ADOPTED —
sequencing nuance: zero-resolution decides absence over the union of current admissible
sources; the identity join gates only the >1 wall and realization completeness (the
method ← join edge deleted). STRUCTURAL — `v2-phase-carriers` split into five staged nodes
(first registry tombstone, superseded_by the frontier node); `floor-parse-formation-wall`,
`floor-record-construction-wall`, and the `compiler-accepted-obligation-closure` terminal
added. CORRECTED — DESIGN §4's closed vocabulary said "5 behaviors" while
`v2.std.node.Behavior` has six (`Value, Transform, Branch, Loop, Bind, Match`); the
denominator of the decidability argument now matches the live coproduct. EMITTED — the §1d
provisional grid, hand-authored interim, dissolve-on the carrier-emitted projection.

## 11. Audit queue

1. ~~Recover `docs/error-examples.md`~~ **DONE — see §8b**; ~~`correctness-dimensions`~~
   **DONE — see §8c.** Still to pull: `what-falls-out`, `two-groundings`,
   `the-derived-homomorphism`.
1a. **Audit `sole_constructor` completeness** (new, post-merge review): generic carriers
   (`Refined<B>`), every construction form, interaction with `module_skips_direct_call_arg_check`.
   The capability ceiling's status — and the cardinality wall's first candidate
   (`type Refined<B> sole_constructor`) — both hang on this audit.
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
6. **Parser: separator-omission refusal** (measured 2026-07-31 — see the §1c census
   row): a dropped list separator must be a parse refusal, never element juxtaposition.
   Probe pair first (ladder-probe-corpus), then the wall. **Now tracked as the
   `floor-parse-formation-wall` roadmap node** (post-merge verdict closed the
   claim-without-track gap this queue item left).
7. **Parser: block-after-bare-identifier misparse** (measured 2026-07-31,
   tidy-deer-730): `if c == CardOpt { f(n: 1) } else { 2 }` fails with `expected RParen,
   found Colon` — the block is read as a variant/record literal. A **false refusal of
   legitimate input** (loud, wrong locus) — a parser-completeness defect rather than
   a ladder harm class; the second slice of `floor-parse-formation-wall`.
8. **Receiver-type resolution defects** (new, measured 2026-07-31): (a) a where-refinement
   alias arrives as its brand, unpeeled to the base; (b) a pattern-destructured coproduct
   payload arrives typed as the variant name rather than the field type. Both gate
   method-wall promotion beyond kernel receivers (§12 Stage 2 amendment).

## 12. Proposed sequencing (reconciled with the independent review; for operator sign-off)

**(2026-07-31 restructure.)** The canonical dependency order now lives in the roadmap
authority itself — the `compiler-guarantee` lane's declared nodes and edges
(`gunbc.roadmap_authority` `guarantee_ladder_nodes` / `guarantee_ladder_edges`) — after the
operator's verdict caught the first cut rendering these nodes from a section-local list
outside `declared_roadmap_nodes()`. This section stays as the narrative rationale; where
prose and edges disagree, **the edges are the authority**.

**Stage 1a — the probe corpus first, as `.dag`.** One probe pair per floor class
(deliberately-bad input expected to refuse; legitimate control expected to accept), landed
as **enrolled expecting-red rows** (the known-red quarantine mechanism already exists: rows
execute *expecting* red, so a wall landing flips them loudly to controls). That makes the
corpus a continuous measurement of the gap before any wall exists — each
compiles-when-it-should-refuse is a counted deficit, not an anecdote — and it is what the
carrier's dispositions derive from, which is why it precedes the carrier.

**Stage 1b — the claims carrier, measurement separated from claim.** Model the guarantee
population (the review's G0–G9 families are a good candidate cut: binding · formation ·
inhabitance · invocation · domain/totality · control soundness · semantic dimensions ·
realization · fidelity · external boundary) as **four carriers**: `GuaranteeRequirement`
(class identity, domain, harm, `ceiling` with its mathematical/capability/price
justification, `next_rung_trigger`); `GuaranteePath` (`subject_grain`,
`acceptance_boundary`, `compile_mode`, `realization_target` — one row per in-scope path,
the population **derived from the census's path axes or completeness-witnessed against
them**, so a strongest-path-only reading is a red, not an oversight); `GuaranteeMeasurement`
(an executed probe/witness receipt, **with a named consumer in `Accepted`**); and
`GuaranteeDisposition` **derived per path, never stored** — `Unmeasured |
BelowFloor{evidence} | FrontierAccepted{diagnostic, accepted_boundary, evidence} |
OnLadder{rung, evidence} | OutsideModeledGuarantee{reason}` — folded to class state as
below-floor dominates, then frontier, then unmeasured, then the **minimum rung across
paths**. `FrontierAccepted` is the fifth state the post-merge verdict caught the four-state
draft missing: **the missing judgment is typed, located, and counted, but the phase result
is still `Accepted`** — not R1 (nothing blocks shipment), not ordinary below-floor (the
deficit is no longer silent). Live specimens from the open candidates: gunbc#7484's
`MethodExistenceUndecided` and gunbc#7485's `GroundingNotDerived` inside an accepted
`InferredTree`. Without the state, the carrier either over-credits a counted advisory as
prevention or erases the real improvement from silent failure to honest frontier. There is
no stored `current_rung` column, so a transcribed rung is *unrepresentable* rather than
lens-caught (corrected per review 45367: an earlier draft of this stage listed
`current_rung` as a carrier field, which would have reintroduced exactly the
rung-inflation class §1b forbids). What survives as checks: an `OnLadder` disposition's
evidence refs must execute (honesty — v2's generic self-grounding is the day-one
below-target disposition), and a class below its ceiling with no `next_rung_trigger` reds
(stall). The five recovered fragment-vocabularies (§3's lattice, `wall
now`/`after`/`ratchet`, `LifecycleByConstruction|Convention`, `DecodeFidelity`, the thesis
tiers) are **joined by class identity and kept orthogonal** — decidability/grounding,
fidelity, construction/enforcement state, historical domain, and harm-timing rung are
different axes and none consolidates into another (corrected per the operator's 2026-07-31
verdict: an earlier draft said "consolidates five vocabularies onto one axis"). Two §3
grounding rules so this doesn't fork existing machinery: the families ground on the
recovered *dimension* architecture (§8c) and the DESIGN open thread `StandingIntent` — this
IS that thread's missing claims list, not a second taxonomy beside it; and per §7b the
authority is `.dag` rows projected into DESIGN.md, never hand-edited prose. Historical
claims enter `Required` + `Gap` (mode-2 rule, §2).

**Stage 1c — baseline prevalence, anchored** (split out of the old Stage 7 per the first
verdict; anchored per the post-merge verdict): the whole-corpus floor rerun bucketed by
ladder position, keyed to the carrier's class ids — the honest *before* picture, pinned to
`anchor_commit 6c6e2dcb8587d73350ac252f5b07a6b50d684485` (the #7489 merge, pre-P0
implementation state). Content-addressing the baseline is what makes it reproducible after
the in-flight branches merge — requiring every implementation node to wait on an unanchored
live-tree measurement would be fragile, so the walls sequence after the baseline *node*
without racing the live tree.

**Stage 2 — close the ordinary premises (wall-now set):** method/callable existence; exact
call labels and counts (the §4 application-arity row); return/`data`/field/generic
inhabitance (incl. deleting the `field_of_type_var` fabrication — its own roadmap node);
`PatternLookupBlocked` refusing instead of `[]`; delete the success-shaped
fallbacks. One staging correction to the review's interim method wall: the
**zero-resolution half lands now** (strictly narrowing — refuse when *no* path resolves a
method; matches runtime reality, cannot red legitimate code), but the **ambiguity half
(>1 resolution) starts as a census**, walled only after the primitive-identity join defines
which dual-path resolutions are the *same* primitive — otherwise `map` resolving via both
registry and algebra template reds the whole corpus on day one and the wall gets reverted
exactly the way the 104 preserved the exemption.

**Stage 2 measured amendment (2026-07-31, tidy-deer-730's receipts on OPEN gunbc#7484;
sequencing re-ruled by the post-merge verdict):** the open candidate carries a narrow
per-receiver method-existence wall over **kernel receivers** (six REDs/positive controls,
regen divergence 0) — candidate evidence on an open branch, never main's rung. What keeps
it narrow, measured: (a) two receiver-type resolution defects — a where-refinement alias
arrives as its brand, unpeeled to the base; a pattern-destructured coproduct payload
arrives typed as the *variant name* rather than the field type — and (b) the
primitive-identity fork (`String` and `Map` declare `length` but not `count` while the
interpreter dispatches `length`/`count`/`size`). **The verdict's sequencing nuance,
adopted:** zero-resolution does NOT wait on the full identity join — absence is decidable
by enumerating the **union of current admissible sources**; the join is required to tell
one primitive represented twice from two genuinely ambiguous candidates, so it gates the
**>1 wall** (and target-realization completeness), not the zero half. The roadmap edges
encode exactly that: `floor-method-ambiguity-wall ← primitive-identity-join`, while
`floor-method-existence-wall` follows the anchored baseline only. The two resolution
defects are §11 item 8; receiver normalization is the wall's first slice.

**Stage 3 — one cardinality vertical slice** (§4b: connect the 2026-07-04 operator
direction + the scoped lattice plan + the manual value-level fold specimens; acceptance =
the operator's scenario refusing at the seam, with the nonempty-proof positive control
compiling; the construction-wall candidate is gated on the §11 item 1a `sole_constructor`
audit — a declared roadmap edge, not prose).

**Stage 4 — v2 phase work, split (post-merge verdict: the monolithic phase-carriers node
over-promised against what gunbc#7485 delivers, so it is five staged nodes; its roadmap
identity is the registry's first tombstone):** `v2-self-grounding-frontier` (the
strict-narrowing slice, carried by open candidate #7485 — self-evidence out, underived
facts become counted `GroundingNotDerived` frontier diagnostics) →
`v2-translate-underived-refusal` (#7485 records Translate can swallow the rejection and
continue; Eval and Translate must agree) → `v2-inferred-tree-completeness` (the all-derived
carrier; candidate-equals-source not well-typed as proof) →
`v2-node-kind-derivation-coverage` (every kind derives or refuses; the generic arm deletes)
→ `v2-target-realization-gate` (exactly one realization per target, refused
target-relatively; also consumes the identity join). The `validate_then_compile` door stays
throughout.

**Stage 5 — primitive identity consolidation** (definition / realization / cost as three
facts on one identity; dispatch derived, not hand-listed; gates the >1 ambiguity wall and
target-realization completeness — NOT zero-resolution, per the Stage-2 nuance).

**Stage 6 — remove the compiler-source exemption on fresh evidence** (rerun the probes over
`v2.*`/`v1.compiler.*`, classify every failure, fix, delete; the unsourced 104 is neither a
blocker nor a promise).

**Stage 6b — the acceptance-completeness door (`compiler-accepted-obligation-closure`,
added per the post-merge verdict):** the terminal P0 wall making the carrier the contract
rather than observability — every `Required` guarantee path has exactly one live consumer,
consumed before the relevant `Accepted` constructor; acceptance refuses when a required
class/path is `Unmeasured`, `BelowFloor`, `FrontierAccepted`, or lacks its live consumer.
Census-first (count, then flip). **Every required-now climb is a prerequisite of the door
— the floor walls, the ambiguity wall, and the cardinality seam included** (review 45545
caught the first edge set leaving ambiguity and cardinality outside it, which would have
let the headline guarantee land while required judgments stayed open; residual prevalence
now depends on the door alone).

**Stage 7 — residual prevalence, measured last** (the baseline half moved to Stage 1c per
the operator's verdict, anchored at `anchor_commit 6c6e2dcb8587` — the #7489 merge — so it
is content-addressed and reproducible after in-flight branches merge): the same bucketing
classifier — statically-decidable / runtime-value-dependent / external-boundary /
resource-budget / capability-not-grounded / interpreter-defect — re-run after the declared
climbs land and **diffed against the anchored baseline**, so each wall's landing is a
measured before/after receipt, never an assumed win.
