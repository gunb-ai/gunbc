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
| Misspelled label binds positionally (§4) | **was BELOW FLOOR — silent wrong binding** | **R2 at the direct-call seam** (session/cool-badger-514): `CallArgumentNameUnknown` + `CallPositionalSurplus` blocking, mirroring the two classes `call_function_inner` refuses at runtime (underscore idiom included); probe pair pre `0 diagnostics` → post two located refusals; the census caught **28 live fossils in 5 fns** (8× `to_string(i:)` after the param renamed to `value`, `arm_body(arm:)`, `is_import_slot_node(p:)`, 17× `fold_list(init:)` vs declared `empty`, `…refusal_reason(path:)` vs `path_opt`) — every one a parameter rename the positional fallback had absorbed silently; +3 more found by grep in the wall's measured blind spot (callee sig unresolved → `sig == none` fallthrough, `dag_collect.dag`), fixed in the same change | R2 (bijection full: duplicate/missing/method-seam pending) | duplicate-label + missing-arg land interpreter-first (runtime doesn't refuse them today — walling compile stricter would diverge the other way); method-pipe seam with the method wall; sig-unresolved fallthrough closes with resolution coverage |
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
| Unknown fn/method | **Main (gunbc#7484 MERGED 2026-08-01):** R2 on established receiver surfaces — an unresolved method on a kernel-profiled receiver refuses (`MethodNotFound`, blocking; the #7479 class), with a rostered, occurrence-budgeted `MethodExistenceUndecided`/`MethodExistenceFrontierAdmitted` frontier elsewhere (FrontierAccepted-shaped). `resolve_builtin_call_type` still fabricates `unit_type` on absence, so the full callable-existence class stays open | R3 (resolved call carries declaration identity) | landed fragment + typed frontier; general wall gated on receiver normalization | #7479 · merged gunbc#7484 | receiver normalization → zero-resolution refusal over the union of current admissible sources (identity join gates only the >1 half); `FunctionRef` IR |
| Call shape (labels/count) | **floor landed at the direct-call seam** (unknown label + surplus positional refuse, blocking; was: misspelled label binds positionally, silent) | R3 (exact bijection in normalized IR) | formal-driven walk; `ArityMismatch` is constructor-grain; `direct_call_shape_diags` runs exemption-free (labels have no representation gap) | `direct_call_shape_diags` beside `direct_call_arg_mismatch_diags` | remaining: duplicate/missing (interpreter-first), method seam, sig-unresolved fallthrough |
| Return conformance | **UnknownUnmeasured** (compile admission proven; runtime disposition and silent paths unmeasured) | R3 (body edge inhabits Arrow codomain) | no general judgment | #7481 | return-position checking |
| `data` annotation | **UnknownUnmeasured** (same basis) | R3 | same lane | #7481 | same lane |
| Generic instantiation | **Below floor — silent** (measured 2026-08-01: `type Boxed<T> { inner: T }` constructed as `Boxed { inner: "not an int" }` at declared return `Boxed<Int>` compiles with zero diagnostics of any severity) | R2 | substitution unproven | one-off execution 2026-08-01 via `compile_dag_diagnostic_census` on the v1 CompileAccept path, source and result in the §10 eighth-pass ledger — **NOT ENROLLED**: no probe pair for this class exists in the tree, so nothing re-runs this measurement and nothing reds if the behaviour changes (§4b meta-obligation 4; codex review 46306). Enrollment is §11 item 10 | inhabitance at instantiation |
| Field through generics | **Below floor — silent** (measured 2026-08-01: `fn get_field<T>(t: T) -> Int { t.no_such_field }` compiles with zero diagnostics — `field_of_type_var` fabricates rather than refusing or carrying a constraint) | R2 (pending-constraint discharge) | `field_of_type_var` minted | one-off execution 2026-08-01 via `compile_dag_diagnostic_census` on the v1 CompileAccept path, source and result in the §10 eighth-pass ledger — **NOT ENROLLED**: no probe pair for this class exists in the tree, so nothing re-runs this measurement and nothing reds if the behaviour changes (§4b meta-obligation 4; codex review 46306). Enrollment is §11 item 10 | constraint carried + unique discharge |
| Closed-match exhaustiveness | **Path-split, measured 2026-08-01 — the class is not one rung.** Coproduct-typed scrutinee: **R2** (a missing arm on a declared closed variant refuses `NonExhaustiveMatch`, blocking, naming the absent variant). Type-variable scrutinee: **below floor — silent** (`fn pick<T>(t: T) -> Int { match t { Red => 1 } }` compiles with zero diagnostics — one arm, an unconstrained subject, and a variant belonging to an unrelated type). Class rung is the minimum, so **below floor** | R3 (full arm population at elimination) | the silent arm is `PatternDynamic { span: _ } => []`, **not** `PatternLookupBlocked => []` as this row previously said — `pattern_subject_from_node` reaches `PatternLookupBlocked` only when the scrutinee's inferred type `is_compiler_error`, i.e. where a diagnostic already exists, so that arm is not the silent one and its silence is **not** established by these probes | one-off execution 2026-08-01 via `compile_dag_diagnostic_census` on the v1 CompileAccept path, source and result in the §10 eighth-pass ledger — **NOT ENROLLED**: no probe pair for this class exists in the tree, so nothing re-runs this measurement and nothing reds if the behaviour changes (§4b meta-obligation 4; codex review 46306). Enrollment is §11 item 10 | `ExhaustivenessUnknown` refuses on the dynamic subject |
| Record completeness | **R2 measured 2026-08-01** (a record literal omitting a declared required field refuses `MissingField`, blocking, naming the field and type — the class was carried as `Unknown — unmeasured` and the measurement raises it) | R3 | judgment is per-literal; construction-side and generic-instantiation completeness are separate and the latter measures **below floor** in the row above | one-off execution 2026-08-01 via `compile_dag_diagnostic_census` on the v1 CompileAccept path, source and result in the §10 eighth-pass ledger — **NOT ENROLLED**: no probe pair for this class exists in the tree, so nothing re-runs this measurement and nothing reds if the behaviour changes (§4b meta-obligation 4; codex review 46306). Enrollment is §11 item 10 | required-field construction at every construction form |
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
| **P0.1** | Calls have exact labels, count, defaults, and parameter binding | **R2 for unknown-label and surplus-positional at the direct-call seam** (session/cool-badger-514, mirroring the runtime contract; was BF — misspelled label bound positionally; census: 28 live fossils refused and fixed, +3 in the measured sig-unresolved fallthrough) | Unchanged by the open P0s | **R2** (landed half); duplicate/missing land interpreter-first | R3 normalized exact-bijection invocation. Call-shape wall (landed) → duplicate/missing parity pair → compiler-source exemption deletion |
| **P0.1** | Callable/method existence | **R2 on established receiver surfaces (merged #7484)** + FrontierAccepted roster elsewhere; `resolve_builtin_call_type` absence-fabrication still open | Merged | **R2 on every compile/emit path** | R3 resolved identity. Receiver normalization → zero-resolution refusal over the union of current admissible sources; identity join → ambiguity refusal |
| **P0.1** | Function body and `data` value inhabit declared types | **R2 for ground kernel scalars + ground element collections (merged #7484)**; everything wider returns to main-silent (the counted advisory was excluded in final review scope) | Merged | **R2 for every grounded declared type** | R3 typed Arrow/data construction. Conformance grounding → returns/data → exemption deletion |
| **P0.1** | Generic instantiation, required record fields, and defaults are sound | **U/BF candidate** | Not established by either P0 | **R2** | R3 typed construction. Conformance grounding → generic instantiation + record-construction wall |
| **P0.1** | Field access has a receiver proven to carry that field | **U/BF candidate** — `field_of_type_var` fabricates | Unchanged | **R2** | R3 field-carrying bound. Baseline → generic-field constraint wall |
| **P0.1** | Closed variants eliminate exhaustively | **U/BF candidate** — blocked lookup returns no diagnostics | Unchanged | **R2** | R3 full arm population at elimination. Baseline → `ExhaustivenessUnknown` refusal |
| **P0.1** | V2 never treats source structure as inferred semantic fact | **F in infer (merged #7485)** — exact self-evidence removed, `GroundingNotDerived` typed frontier; **Translate can still swallow the refusal and proceed** (the documented fail-open, next node) | Merged | **R2 in both Eval and Translate** | R3 distinct inferred carrier. Self-grounding slice ✓ → Translate propagation → all-derived `InferredTree` → derivation coverage |
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

**The prose had drifted from the substrate, on the substrate's most load-bearing sentence
— found by this audit, CORRECTED 2026-07-31 (post-merge verdict; stale present-tense
wording here caught by review 45558).** DESIGN.md §4 said the closed vocabulary is *"6
connectives + 5 behaviors"* while `v2.std.node` declares **six** behaviors: `Value |
Transform | Branch | Loop | Bind | Match`; the authority (`gunbc.design_document`) now
states six with the members named, and the projection matches. The specimen retains its
evidentiary value in past tense: the count sat wrong in the live authority for an unknown
period, and the recovered thesis is sharper about what that means — it listed five
behaviors and declared *"Substrate extension is a C1-class stop signal (seventh connective
or **sixth behavior**) — all four dissolution patterns … must fail with structural
arguments before extension is allowed."* Whether `Match`'s promotion was adjudicated under
that rule is still not established here (it may well have been — v2's `Match` inference is
real work; the §11 queue owns finding the adjudication or filing its absence). What the
episode establishes either way: the denominator of the decidability argument drifted
silently in prose, which is a clean specimen of why the guarantee must be modeled where a
lens can read it, not written where only a reader can.

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

**Call-shape landing + dark-suite incident ledger (2026-08-01, sixth pass — all by
execution):** LANDED (gunbc#7519, MERGED 2026-07-31) — the
call-shape floor: `CallArgumentNameUnknown` + `CallPositionalSurplus` blocking at the
direct-call seam, mirroring `call_function_inner`'s two refused classes; census refused 28
live rename fossils (all one class: a declaration's parameter renamed while call sites kept
the old label, absorbed positionally — 8× `to_string(i:)`, `arm_body(arm:)`,
`is_import_slot_node(p:)`, 17× `fold_list(init:)`, `…refusal_reason(path:)`), +3 fixed in
the measured `sig == none` fallthrough that the wall does not judge. The `fold_list` sites
never failed live because the interpreter grounds that call natively (label-blind) while
the user-fn path would refuse it — dispatch-tier-dependent meaning, the wall's sharpest
justification. INCIDENT (found by tidy-deer-730 during the #7484 main-integration, byte-identical
red on both parents): `rust_btree_set_ord_eligibility_requires_nominal_carrier_shape` has
been RED on main since `d975e1001a1` (2026-07-21) moved `Symbol` from the shape-aware
nominal-carrier representation onto the name-only opaque-alias roster, dropping the
argument check its negative control pins (`Symbol<Float>` admitted by name) — invisible for
ten days because the Rust unit suite left CI on 2026-07-11
(`commit_gate_rust_suite_removed_disposition`), so an enrolled RED was executing nowhere:
specification-without-execution one rung up, the exact state §4b's dissolution rule
forbids. Repaired by gating the name-grain arms on childlessness
(`rust_btree_set_ord_name_grain_note`, the method wall's name-grain lesson at the emit
seam); the dark-suite gap itself is queue item 9, an operator decision priced by this
incident, not silently patched.

**Reconciliation ledger (2026-08-01, seventh pass — operator spine verdict, adopted):**
MERGED — #7519 (2026-07-31), #7484 and its adoption (2026-08-01), #7485 (earlier): every
open-candidate row above converted to main-state; the hand-authored census went internally
inconsistent within hours of the merges (call-shape said LANDED while #7484/#7485 still
said open candidate), which is the exact failure the claims carrier exists to delete and
the reason no further wall work precedes the spine. STRUCTURAL (roadmap, this pass) — the
three merged P0 slices recut at their actual grain with tombstoned identities: call-shape
→ label/surplus (accepted, #7519) + missing/duplicate + signature-resolution coverage
(method seam to the method lane); method-existence → established-surface (accepted, #7484)
+ receiver normalization + general zero-resolution; inhabitance → conformance ground
fragment (accepted, #7484) + conformance grounding + general wall;
`v2-self-grounding-frontier` accepted against #7485 with `v2-translate-underived-refusal`
activated as the containment workstream (Translate can still swallow the typed refusal —
the documented fail-open). DEPENDENCY CORRECTIONS — exemption removal re-grounded on
argument-type-compatibility grounding + conformance grounding (the label wall never gated
it: labels have no representation gap); the `Accepted` door split into
mechanism/floor-closure/extended-closure so the door lands early as audit without claiming
open classes closed; census emitters re-parented on the carrier alone (parallel with the
baseline, deleting the hand grid at the earliest point); the spine gains a
measurement-schema stage ahead of probes and carrier (class/path/probe identities +
receipt schema carrying subject revision × harness revision × probe-set digest), breaking
the probes⇄carrier protocol cycle; the baseline is a two-revision execution (the anchor's
compiler artifacts under the new content-addressed probe set — running "the baseline at
commit X" is otherwise ambiguous three ways). CORRECTED in-flight (review 45918, on the
reconciliation PR itself): the extended activation's first cut was one requires-all node
whose prose promised class-by-class widening — recut as four per-class admission nodes
plus a terminal roster-completeness certification (§12 Stage 6b carries the shape).

**Floor-class probe measurement + the observation surface it required (2026-08-01, eighth pass
— `ladder-probe-corpus`, all by execution).** THE SURFACE, and why it was not optional: the only
`.dag`-callable v1 compile was `compile_dag_rust_emit_check`, which counts diagnostics passing
`compile_clean_diagnostic_is_hard` and answers `false` when that count is nonzero — so class
identity, severity, and every advisory were discarded inside the host. That is not merely a
weaker probe, it is three specific losses measured against the tests Stage 1a migrates: a probe
refusing for *any* hard reason (a typo in the probe source included) reads as the wall firing;
demoting a landed wall from blocking to advisory turns its RED silently GREEN, because the
filter **is** the severity predicate; and a positive control cannot state
zero-diagnostics-of-any-severity, the assertion review 45357 added after an advisory
`MethodExistenceUndecided` passed unnoticed as a green control. Structurally it is worse than a
fidelity preference: Stage 0's `RefusedTyped` and `AcceptedCounted` both carry a diagnostic class
and a count, so **the Stage-0 vocabulary was uninhabitable on every v1 path** until a
class-and-count surface existed. `compile_dag_diagnostic_census` is that surface (operator-amended
scope, 2026-08-01) — one measurement-only builtin projecting `compile_clean_diagnostic_histogram_key`
and the existing severity delegation, filtering nothing, with a typed `CensusNotRunnable` arm kept
distinct from an empty census so could-not-measure never reads as the subject passing. THE
MEASUREMENT, six floor classes as probe pairs (deliberately-bad input + legitimate control, v1
pipeline → Rust target, synthetic single module): **four are below floor and silent** — generic
instantiation, field-through-generics, the dropped list separator (reproducing tidy-deer-730's
specimen independently), and closed-match exhaustiveness *on a type-variable scrutinee*; **two
refuse** — record completeness (`MissingField`, blocking) and closed-match exhaustiveness on a
coproduct-typed scrutinee (`NonExhaustiveMatch`, blocking). Every control compiles clean, so the
harness is discriminating rather than uniformly refusing. TWO §1c ROWS CORRECTED BY THIS RUN, both
in the direction the census's own `Unknown` default protects against: record completeness was
carried as unmeasured and measures **R2**; and closed-match exhaustiveness is **not one rung** —
it splits by scrutinee path, and the row's attribution of the silence to `PatternLookupBlocked =>
[]` is wrong on the carrier, since `pattern_subject_from_node` reaches that arm only where the
scrutinee's inferred type `is_compiler_error` (a diagnostic already exists). The silent arm is
`PatternDynamic { span: _ } => []`. `PatternLookupBlocked`'s own silence remains **unestablished**
— no probe here reproduced it, and it is not asserted as though one had. **`AcceptedCounted` has a
producer, and the route to that finding is itself a measurement lesson.** An earlier draft of this
entry recorded the opposite — that the census could *represent* a counted advisory but no probe
could *produce* one, so a `FrontierAccepted` disposition would be derived from an empty population.
That was **wrong, and wrong because of the probe rather than the compiler** — but the first
diagnosis of *why* was also wrong, and the second correction is the load-bearing one. The initial
account blamed the closure: the three candidates (a where-refinement alias, an unlisted-import
shape, an unresolved method on a bare type parameter) ran through a fixture-only CLI compile whose
single source root carries no `std`, so the refinement supposedly never resolved *to* a refinement.
**A discriminating control refuted that.** Run under the full `dag` + `src/v2` pool, the
where-refinement alias `type Tight = String where non_empty` is **still silent** — zero diagnostics
with `std` fully available — while a *different* shape, a cast to `std`'s refined brand
(`fn tighten(s: String) -> NonEmptyStr { s as NonEmptyStr }`), fires `WhereRefinementUnenforced` as
a counted advisory on the same harness. So the closure was not the discriminator between the
failure and the success: **the probe SHAPE was.** A **2×2 pins the actual axis**, and it is not the
one two successive explanations guessed. Crossing declaration site (locally-declared alias vs
`std`'s brand — structurally identical, `type NonEmptyStr = String where non_empty` and
`type Tight = String where non_empty`, same predicate) against cast subject (a literal vs an
unknown parameter): local+parameter **fires**, local+literal **silent**, std+parameter **fires**,
std+literal **silent**. Declaration site is irrelevant — a user-declared refinement alias is judged
exactly as `std`'s brand is — and the diagnostic names the real axis itself: *"where-refinement
unenforced: predicate `non_empty` on `Product(Tight)` — **non-literal value at refined
position**"*. So the original probe's silence is **correct behaviour**, not a gap: it cast the
literal `"x"`, and a literal at a refined position is deliberately exempt. It is a **dead probe** —
it looked like it exercised the judgment (it casts, at a `fn` boundary) while the judgment
deliberately exempts its exact form — and **no seventh census row is filed**, because filing one
would have been a below-floor claim against behaviour that was already right. This also retires
the intermediate guess that the alias "never exercises the judgment because its predicate resolves
nowhere": the predicate resolves fine, as local+parameter firing proves. The corrected rule, and
the one the baseline stage should carry:
**a synthetic-probe negative is only as good as the probe's ability to reach the judgment**, and
that can fail two independent ways — a closure too narrow for the judgment's machinery, *or* a
shape that never triggers it. **MANDATE (lane policy, operator-adopted 2026-08-01): every
below-floor or silent-class row the baseline stage produces carries a probe-adequacy receipt —
closure AND shape AND the corpus-prevalence cross-check where the class has a population — and it
is required, not advisory.** What bought the mandate is this entry's own record: *three* consecutive
mechanism explanations (no advisory is producible → the closure was too narrow → the predicate
resolves nowhere) were each refuted, while the conclusions they explained kept surviving. Mechanism
stories do not survive contact; executed discriminating controls do, and a row asserted on the
former is how a wall gets built against behaviour that was already correct. **Companion mandate,
bought by this entry's own review incident (lane policy, operator-adopted 2026-08-01): enrolled
evidence must be DISCRIMINATING, not merely present — one executing RED per refusal arm, and a
refusal arm without its RED is unmergeable regardless of approval tally.** The receipt is that this
module's *own* exported accessor collapsed `CensusNotRunnable` to `[]`, so could-not-measure and
observed-nothing became one empty list at the API boundary — the exact conflation the coproduct
exists to prevent, reintroduced by the carrier built to prevent it — and its witness **asserted the
collapse as contract** (`count(census_rows(not_runnable)) == 0`). A green witness pinning the wrong
contract is worse than no witness: it defends the defect against its own fix. Three approving
review providers passed over it; one caught it (`review 46144`). Two consequences the carrier stage
inherits: a review tally is **never** a substitute for an executed discriminating control on a
spine carrier; and where construction is available it, not reviewer vigilance, is the wall — the
collapsing accessor was deleted rather than documented, so the conflation is now unwritable at that
boundary and a reviewer missing what cannot be written costs nothing. Where only validation is
available, the RED is the wall and the tally is decoration. Corpus prevalence is
the cheap cross-check for both probe failure modes: the whole-tree
census carries **1,981** `where-refinement` advisories, so the class was demonstrably reachable
while the probe found none, and a probe disagreeing with corpus prevalence is evidence against the
probe. Both the original claim and its first correction are recorded here rather than overwritten,
because the sequence is the lesson: a plausible mechanism accepted without a discriminating control
is how a wrong explanation survives its own correction. **PROBE-ADEQUACY RECEIPTS for the four
below-floor findings, since the rule above applies first to the rows that prompted it.** Each was
originally measured under a fixture-only closure and has been re-run under the full `dag` +
`src/v2` pool; the discriminating control for the re-run is the refined-brand cast above, which
surfaces a diagnostic on that same harness, so a zero result there is the compiler's silence and
not the harness's. `generic-instantiation` (`Boxed { inner: "not an int" }` at `Boxed<Int>`) — full
pool, zero diagnostics, **verdict unchanged**. `field-through-generics` (`t.no_such_field` on a
bare `T`) — full pool, zero diagnostics, **verdict unchanged**. `parse-list-separator` (three
elements, one comma dropped) — full pool, zero diagnostics, **verdict unchanged**.
`exhaustiveness-type-variable-scrutinee` (`match t { Red => 1 }` on `T`) — full pool, zero
diagnostics, **verdict unchanged**. All four declare only local types over kernel scalars, so the
wider pool adds no machinery they depend on; the receipts record that this was *measured* rather
than argued.

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
9. **Dark-suite evidence gap** (measured 2026-08-01 — the ord-eligibility incident, §10
   sixth pass): every `ct_*` wall RED in the v1 Rust unit suite executes in **no** CI leg
   since the suite's 2026-07-11 removal, so a wall regression surfaces only when someone
   runs `cargo test` by hand (this one sat red for ten days; the call-shape wall's own
   RED rides the same dark suite). The removal was a deliberate operator cost ruling —
   the decision to revisit is **which REDs must execute on an acceptance path and at what
   cost** (the wall-witness subset runs in milliseconds; the 27-minute cost that drove
   removal was the self-compile/profile tests). Options: a filtered fast-suite CI leg, or
   migrating wall REDs to claim witnesses the floor already runs. Operator sign-off
   required; priced by the incident, tracked here until dispositioned. **Scope ruling
   (operator, 2026-08-01): migrating the guarantee-wall REDs does NOT dissolve the dark
   suite** — the remaining suite needs a finite classification, every test dispositioned
   as exactly one of: migrated into an active `.dag` probe · kept and re-enrolled in an
   active Rust gate · superseded by stronger active coverage · deleted as genuinely
   redundant. "All guarantee probes moved" must never be read as "the suite is safe to
   delete." **Prerequisite discharged (2026-08-01):** migration was blocked on something the
   ruling did not name — the `.dag` side could observe only *that* a synthetic compile refused,
   never *which* judgment fired, whether it blocked, or how many times, so every `ct_*` wall RED
   would have lost its class and severity assertions on the way across. `compile_dag_diagnostic_census`
   (§10 eighth pass) closes that; the per-test disposition ledger this item calls for opens with
   the first migrated pair, and each row names the executing `.dag` probe that replaced the Rust
   assert rather than merely recording that the Rust test was removed.

10. **Unenrolled floor measurements** (opened 2026-08-01 by codex review 46306, which caught
   the §1c rows citing a "ladder-probe-corpus probe pair" as their evidence when no such probe
   exists in the tree). Four §1c rows — generic instantiation, field through generics,
   closed-match exhaustiveness, record completeness — carry rungs established by a **one-off
   manual execution** of `compile_dag_diagnostic_census`, recorded with sources and results in
   the §10 eighth-pass ledger. The measurements are real and reproducible; what they are not is
   **enrolled**. Nothing in the corpus re-runs them, so if any of these behaviours changes, no
   check reds and the row silently becomes false. That is the §4b meta-obligation-4 shape
   (evidence must remain enrolled as the executing proof the rung stays real) and it is distinct
   from item 9: item 9 migrates REDs that EXIST but run dark, whereas these classes have no
   probe at all. The distinction matters for sizing — item 9 is a move, this is authorship.
   **Bound while it stands:** a rung whose only evidence is an unenrolled measurement is honest
   about the past and silent about the present; it must not be cited as durable evidence, and
   the §1c Evidence cells say so in-line rather than relying on this item being read.
   **Dissolves when** each of the four carries a probe pair in `dag/test/claim/` on the
   ladder-probe-corpus class/path identities, at which point the Evidence cells name the
   executing probe the way the parse-separator row already names `tidy-deer-730`'s.

**Rows 11–17 are one class, and the class is the finding.** They were opened separately
   over one night's auditing and read as four unrelated defects until the fourth arrived; what
   they share is not a subsystem but a *property*. Each is a diagnostic channel that **reports
   something other than what it appears to report**, and in every case the misreport is silent
   and plausible — there is no arm, no count, and no shape difference between the true output
   and the false one. A `Bool` that fired for one of three reasons looks exactly like a `Bool`
   that fired for the reason you assumed; a byte offset rendered `file:29073-29163` looks
   exactly like a line range; a fixture emptied of its subject passes exactly like one that
   still carries it; a filter keyed on location-presence reports a population exactly like one
   that reports the whole. **Why the family matters more than the rows:** these are the channel
   by which every *other* guarantee in this document is observed, so a defect here is not one
   more error class — it is a discount applied to the evidence for all of them, and it applies
   silently. The §4b obligation to derive a rung from executed measurement assumes the
   measurement channel is faithful; where it is not, a derived rung is as transcribed as a
   stored one. **One member differs in kind and the distinction is load-bearing.** Rows 11–13
   *degrade* information — the channel is under-informative, and a careful reader who knows the
   defect can compensate. Row 15 *inverts the selection*: it reports exactly the rows nobody can
   act on and conceals exactly the rows carrying a file and byte span, so it is actively
   anti-correlated with usefulness rather than merely lossy. A reader cannot out-think that by
   being careful — the receipt is `proud-crane-845`'s no-tail conclusion, correct on the data
   the instrument showed and wrong about the corpus, because the tail is precisely what the
   instrument hid. **The standing rule this family produces** (`royal-hawk-392`, generalizing a
   weaker rule that raw lines beat a summary): *raw-not-summary is necessary and not sufficient,
   because rawness is relative to the instrument that emitted the lines.* A summary visibly
   collapses codes into categories someone chose; raw output carries the same selection
   invisibly, precisely because rawness reads as absence of processing. **Ceiling and trigger
   are per-row below**; what the class adds is that a repair to any one of them leaves the
   property intact in the others, so they should be prioritized as a channel-fidelity program
   rather than picked off by whoever trips over one.

11. **The emit-check oracle collapses three arms into one `Bool`** (opened 2026-08-19;
   measured by `swift-moth-294` reading `cli_run.rs` `compile_dag_rust_emit_check_uncached`).
   The function returns `false` from three structurally distinct arms — hard diagnostics
   nonzero · the requested file absent from `result.files` · the includes/excludes **content**
   assertion failed — and discards which one fired. **Harm:** every consumer must
   hand-establish the mechanism, so the honest reading of any red is "one of three things
   happened", and a *stale realization* is indistinguishable from a *misaimed test* and from a
   *broken emitter*. **Live cost already paid:** a session gave a confident mechanism account
   of a CI red that their evidence could not establish — arm 3 is exactly the shape a stale
   seed produces — and caught it themselves only on reading the implementation. The sharper
   half is second-order: a coordinator's own record of this instrument listed **two** arms, so
   every positive control recommended against it that day covered arm 2 alone. A conflated
   oracle does not merely mislead its readers; it silently narrows what anyone thinks to
   control for. **Ceiling — structurally impossible, and it is a construction fix rather than a
   validation one:** the arm is KNOWN at the return site and thrown away, so a typed outcome
   does not add a check, it stops the loss. **Next trigger:** replace the `Bool` return with a
   typed outcome carrying the arm; consumers then read the arm instead of inferring it. Small
   change, large readability gain on every future emit witness.
   **AMENDED 2026-08-20, by measurement: it is three arms PLUS A LIVENESS PRECONDITION, and the
   precondition is the dangerous part.** Establishing the arm on a live red found that a
   *compiler crash* produces the same observable surface as a legitimate arm 2 — the first
   replication attempt returned `compile_exit=101`, `FILE_FOUND=no`, nothing emitted, which
   reads as a clean arm-2 result and would have been reported as one. The actual cause was a
   panic in `repo_relative_path_normalized`, which refuses a source root outside the workspace
   root, because the probe module had been placed in `/tmp`. So **`FILE_FOUND=no` is arm 2 ONLY
   IF THE COMPILER RAN**, and arm 2 needs its own liveness control — a positive control through
   the identical invocation on a known-good in-tree module — before it can be claimed at all. A
   typed outcome must therefore distinguish *did not emit this file* from *did not run*, or it
   reproduces the same conflation one level in.
   **A second finding from the same run, and it is the reason per-arm typing is not sufficient
   on its own:** the arm was established as arm 3, and the *mechanism inside* the arm was still
   wrong. The predicted tripwire — an EXCLUDE on the bounded item header `struct
   FreeMonoidSupplementalStruct<T: Clone>` — **passes**. The header was already bare before the
   change. What actually fails is three other assertions: two INCLUDEs for hand-written `Debug`
   and `PartialEq` impls, and an EXCLUDE on `#[derive(Debug`. The missing mechanism is
   hand-written impls, not a bound on the item. A per-assertion verdict caught that; a per-arm
   verdict would not have. The general shape is that **a well-formed account can be correct at
   the level it is checked and wrong one level down** — arm-vs-`Bool` caught the first level,
   per-string-vs-arm caught the second — which argues for reporting the failing assertion, not
   merely the failing arm. No rung is authored here — the
   rung is the thing that must be DERIVED from executed measurement (§1c, and the Stage 0
   carrier `gunbc.guarantee_measurement` deliberately stores none).

12. **`SourceSpan` carries untyped magnitudes, so a byte offset renders as a line number**
   (opened 2026-08-19; **relocated one layer down before filing** — see the correction note).
   `dag/std/types.dag` `SourceSpan` declares `start: Int` and `end: Int`. Nothing in the type
   says what unit they are: byte offset, character offset and line number are all inhabitants
   of `Int` and the type admits all three identically. The producer fills bytes, the general
   diagnostic renderer prints `file:START-END` — a form universally read as a line range — and
   no mechanism can catch the disagreement, because there is no unit present to disagree with.
   **Observed:** the in-body annotation refusal rendered
   `(src/v1/trait_derive_emit.dag:29073-29163)` against a file of 1407 lines / 62288 bytes;
   byte 29073 lands on line 478. **Harm:** confidently located and WRONG, in a format
   indistinguishable from a correct citation. An absent location announces its absence and
   sends the reader looking; a plausible wrong one does not — the reader finds the file ended
   long ago and concludes the diagnostic or the path is broken, never that the answer is a few
   lines from what they wanted. That is fabricated plausible output (§5) in the one channel
   whose entire function is to be believed. **Distinguishing:** the cited value exceeds the
   file's line count (here twentyfold); `head -c N file | wc -l` lands within a line or two of
   the true site. **Population:** 17 files construct `SourceSpan` — bounded and countable.
   **Ceiling — structurally impossible, and PROVEN ATTAINABLE IN THIS REPO:**
   `src/v2/test/claim/long/a4_opacity_test.dag` compiles three sources through
   `compile_ingest_staging` and asserts refusal of `ByteOffset`-for-`CharOffset` AND
   `CharOffset`-for-`ByteOffset`, with an accepting same-type control. The capability is
   demonstrated, not speculative — this is not a wall-after-grounding. **Two precisions that
   change the fix, both easy to get wrong:** (i) the witness executes the **record-wrapper**
   form (`type ByteOffset { value: Int }`), NOT the `where brand(..)` refinement form, despite
   one of its test identities being *named* `a4_opacity_same_brand_accepts` — brands are
   separately known to be unenforced at acceptance positions, so adopting the brand spelling
   because it reads more elegantly would land a change that looks like the climb and enforces
   nothing, which is the rung inflation §4b calls worse than sitting low; (ii) **that witness
   does not currently execute** — it is a `witness_deferral_freeze` member under
   `LegacyFrozenPathDeferral`, which the coverage gate deliberately never admits as covered, so
   the capability is proven *in source* and unexercised *in the present tree*. **Next trigger:**
   declare the unit types in the record-wrapper form and give `SourceSpan.start`/`.end` those
   types instead of `Int`; the renderer then either consults `build_newline_index`
   (`src/v1/00_core.dag`, which already carries `offsets` and `char_codes`, so byte-to-line is
   already modeled and simply not consulted) or names its unit. **Blast radius:** every
   diagnostic carrying a `SourceSpan`, i.e. the general renderer.
   **The class is GROWING, not static — a second instance landed the same day, in a NEW module**
   (review 53871 on gunbc#8527): `src/v1/expected_red_roster_join.dag` declares
   `BudgetExceeded { elapsed_ms: Int, budget_ms: Int, ... }` — Duration-semantic fields as bare
   `Int`, ported from hand-Rust `u64`/`u128`, with no `feature:` or dissolve-on row, at the exact
   moment `std.measure.Duration` was available to ground them. Same shape one domain over:
   nothing prevents a caller passing seconds, a count, or the other field. That matters for how
   this row is priced — the 17 `SourceSpan` construction sites are inherited debt, but new
   modules are still extending the class, so the population is not a fixed backlog waiting for a
   sweep. Any fix that grounds only the existing sites leaves the authoring path open.
   **The §3 reading, which is the durable half:** the compiler emitted a POSITIONAL citation
   about its own input at a moment when the declaration NAME was in hand — the diagnostic knows
   it sits inside a declaration body, that is its entire complaint. §3's cite-the-symbol rule
   was written against human prose; this is a machine-side instance in the compiler's own
   diagnostic channel, and its argument applies unchanged.
   **Correction note, kept rather than silently fixed:** this row was first reported — and
   relayed onward by a second session — as a defect in `dag/std/source_annotation.dag`. That
   module is clean: `annotation_attachment_refusal_message` returns only the sentence, the
   refusal variants carry `origin: SourceSpan`, and the header at
   `annotation_attachment_refusal_origin` explicitly reasons that a consumer should ASK for the
   origin rather than store a second copy. The symptom was right and the attribution was one
   layer too shallow; it is recorded here because two readers repeated it before the formatter
   was read.

13. **§4c comment-placement cannot be swept by text grep, and three shapes are therefore
   UNKNOWN** (opened 2026-08-19; two sessions, independently sampled). A bot commit landed an
   11-line rationale comment inside a fold lambda body in `04_infer.dag`
   `symbol_index_insert_unique_disj_variant_aliases`, which blocked `required-regen`
   **corpus-wide** with 11 §4c refusals and sat on main undetected — self-compile and regen are
   not live gates under the CI rung-drop, so it surfaced only because someone needed regen for
   an unrelated reason. Relocated verbatim to module-item grain in `10542cb3bb4`.
   **What IS established, as a controlled zero:** a sweep for the exact shape that fired —
   *indented* `//` lines across `dag/` and `src/` — returns **0**, and the zero is readable
   because the instrument carries controls in both directions: it finds 22,429 column-0
   comments, and it detects a planted violation of that shape in a temp copy.
   **What is NOT established, and cannot be by this method:** trailing `//` after code on the
   same line, block-comment `/* */` forms, and column-0 comments in a between-items position
   §4c refuses for a different reason. A text grep returned 225 and 75 hits for the first two;
   **both are essentially all false positive**, independently sampled by two sessions at 6 and 8
   hits respectively with not one instance of comment syntax among them — every hit was inside
   a string literal. **The reason is structural rather than incidental, which is why no better
   filter exists:** this corpus's job is to carry other languages' syntax as data. Emitters hold
   target-language comments as template strings (`extdeps/languages/rust/emit.dag`), witnesses
   hold `.dag` source as fixtures (`compiler_tests_rust.dag`, and
   `dag_line_comment_annotation_channel_test` which is *literally* a comment-annotation
   fixture), goldens hold generated do-not-hand-edit banners
   (`gunbc/stage0_crate_layout_emit.dag`), plan rows hold `CodeBlock.code`, and an rsync pattern
   holds `objects/pack/*.keep`. A regex for comment syntax over a corpus built to carry syntax
   as data is measuring the wrong thing **by construction**.
   **Status is UNKNOWN-with-a-named-reason, deliberately, and that is a real state rather than a
   gap:** reporting 0 or reporting 225 would both be fabrications in opposite directions, and
   the 225 would be the more expensive one — it would send someone to audit 225 string
   literals. §5's objection to the absorbing fallback is exactly that ⊥-as-ignorance gets
   rendered as a verdict; this row declines to render it as either.
   **Next trigger:** a lens over `Node` with string-literal awareness, which is the only
   instrument that can separate comment syntax from carried data. **Priority: file, do not
   staff** — the class that actually fired is covered by the controlled zero above.
   **A standing measurement rule falls out of this one, and it inverts the common intuition:**
   a ZERO looks like nothing and therefore invites a control; a NONZERO looks like a finding and
   therefore does not. That is backwards. A false nonzero is strictly more expensive than a
   false zero — a false zero costs an unnoticed gap, a false nonzero manufactures work at
   phantom sites *and* discredits every number reported beside it — and it is self-reinforcing,
   because a count that arrives looking like evidence invites explanation rather than doubt.
   **Control the instrument before reading the number, and sample a nonzero before reporting
   it.** Note also that cheap-to-RUN is not cheap-to-BELIEVE: the marginal cost of one more
   grep is near zero, but the marginal cost of an uncontrolled grep is unbounded once its
   number enters the record. Cost scales with instruments, not with keystrokes.

14. **A fixture can be emptied of the subject it exists to exhibit, and its test still passes**
   (opened 2026-08-20; found on the namespace-cut branch, filed here rather than with that
   incident because the CLASS is the same subject as items 11-13 — a channel reporting something
   other than what it appears to report). **Invalid state:** a test carrying source-as-data whose
   carried data no longer contains the construct the test is named for, while every assertion
   still passes. **Discovery vector:** a scripted rename silently rewrote 51 string literals, of
   which **46 were bare-to-qualified** and therefore invisible to any check comparing strings
   that already contain dots — a bulk edit transforms CODE, where names are checked so a bad one
   is loud, and it transforms STRING LITERALS, which are data and emit no diagnostic anywhere.
   One rewrite moved a roster identity to a **homonym** — main carries both a discoverable
   `test fn` and a plain lens fn named `non_fold_residue_clean_holds`, so the enrolled identity
   moved from the executable one to the unexecutable one and the roster count stayed 306 on both
   sides. Another would have rewritten bare target-language spellings carried as emitter data
   (`"Node"`, `"Outcome"` in `src/v2/extdeps/languages/rust.dag`) into qualified `.dag` paths,
   **emitting a `.dag` path as a Rust type name** — invalid Rust from a compiler that believed
   it was correct. **Harm, and why it is distinct from the well-known weak-assertion case:** this
   is a non-discriminating control arriving through DATA rather than through the assertion.
   Every review habit, every RED-control discipline and every rung-honesty rule in this document
   watches what a test ASSERTS. Nothing watches whether the fixture still CONTAINS its subject,
   so the test keeps asserting true things about a specimen that no longer exhibits the property
   under test. **The codemod is only one vector.** The same end state is reachable without any
   bulk edit — a witness shrunk to fit a budget, a fixture simplified during a refactor, an
   import removed because it looked unused — and the §4b witness-cost thread already records the
   adjacent case where relocating an over-budget witness deleted the evidence while retaining
   the file. **Distinguishing facts:** the test passes; the assertions are individually correct;
   the fixture no longer contains the construct named in the test identity; counts over the
   enclosing roster are unchanged. **Ceiling — mechanically preventable now, with a plausible
   route to structurally impossible, and the two must not be conflated.** Preventable now: a
   test whose fixture carries source can assert, as its first act, that the carried source
   exhibits its subject — a positive control on the SPECIMEN rather than on the outcome. That is
   validation and it is cheap. Structurally impossible would require the fixture to be *derived
   from* the construct it exhibits rather than carrying an independently editable copy, at which
   point an empty fixture has no representation; that is a real but much larger change and it is
   not claimed here. **Next trigger:** the cheap half — establish whether a specimen-level
   positive control can be expressed against the existing carried-source fixtures, and if so
   whether it can be made a condition of admission rather than an author's option. **Immediate
   operational rule, independent of any of the above:** after a bulk edit, **default to
   restore-first** — any string literal a codemod touched is damage until proven otherwise —
   and for each branch-only identity `a.b.C`, check whether main's *same file* carried bare
   `"C"` as a complete literal. That is the detector for the 46-of-51 class and nobody writes it
   by default.

15. **The diagnostic reporting filter is INVERTED: it emits the unactionable rows and
   suppresses the located ones** (opened 2026-08-20; measured by `stern-tern-636`, relayed by
   `royal-hawk-392`, second run by `stern-tern-636` from a different entry byte-identical except
   the header). Counting at diagnostic **construction** rather than at **report** splits one
   population three ways: **72 constructed · 46 reported · 26 hidden**, where the reported set is
   *exactly* the rows whose `file` field is the `<synthetic>` sentinel and the hidden set is
   *exactly* the rows carrying a real file and a byte span. The selection predicate is
   location-presence and nothing else. **Harm:** the channel spends the reader's entire attention
   on the half nobody can act on and conceals the half that carries offsets — so all five type-pair
   shapes other than `NonEmptyStr <- String` (including two *refinement-to-refinement* pairs) are
   invisible in the reported output, across five files that never appear in it. This is the one
   member of the class above that **inverts** rather than degrades: a degraded channel is
   under-informative and a careful reader can compensate; an inverted one is anti-correlated with
   usefulness and cannot be out-thought. **Receipt that it cannot:** `proud-crane-845` concluded
   there was no tail and retracted it — the conclusion was correct on the data the instrument
   showed, and wrong about the corpus, because the tail is precisely what the instrument hides.
   **Provenance, carried deliberately:** the `<synthetic>` sentinel is verified in tree
   (`src/v1/00_core.dag` constructs `SourceSpan { file: "<synthetic>", .. }`), so the predicate is
   keyed on a genuine marker rather than on absent text; the 72/46/26 counts are **not** verified
   here — they require the regen path the mirror hold forbids, and rest on a construction-site
   count plus one independent corpus-wide control. Anyone repeating the split downstream carries
   that provenance with it. **Attribution is open and its test is designed** (`swift-moth-294`):
   the five diagnosed files are **byte-identical by blob hash** from census pin `90b1e4e7ff` to
   main across 27 commits, with `04_infer.dag` and its mirror as controls that *do* differ — so
   the subjects are invariant and any diagnostic difference across SHAs is attributable entirely
   to compiler behaviour, the source-changed confound eliminated by construction rather than
   bounded by judgement. The test is **three-armed, not two**: pin · `76e96333af` (#8579 hardcode
   live) · main — because a two-point test cannot distinguish *pre-existing* from *suppressed by
   #8579 and now resurfacing*, and suppression is precisely what a fail-open hardcode produces.
   **THE TYPE-PAIR BREAKDOWN ABOVE IS A CENSUS OF EVIDENCE SHAPES, NOT OF FAILURE CLASSES** (added
   2026-08-20 on `stern-heron-695`'s reading; cross-tab by execution pending). The ten pairs are
   grouped by *formal ← actual*, and the failure class is not a type pair — it is which branch of
   `where_refinement_diags_for_predicate` the value lands in. **That axis was already in the data:**
   `src/v1/00_core.dag` `WhereRefinementUnenforced` carries a `reason: String` that is a closed sum
   of exactly five deferral strings, declared as such by
   `where_refinement_deferral_reason_scaffold_note` — *"a closed-string sum enrolled in
   `is_where_refinement_unenforced_advisory_reason`; any unlisted reason fails closed blocking."*
   The diagnostic names its own class and the census grouped on the subject instead. **Why it
   matters more than a relabelling:** #8608's fix edits the literal-extraction arm, and the 21
   remaining rows are NON-literal expressions — parameters, field accesses, call results — so
   `expr_is_any_literal` is false and they never reach the extractor at all. A per-arm fallback
   chain would have been **inert for five of six arms**. Predicted decomposition, to be confirmed
   or refuted: 2 a decidable wall (`lower_hex_40` implies `non_empty`, but the checker compares
   predicate NAMES so the implication is invisible) · 5 corpus modeling debt with no compiler
   change · 4 path-sensitive refinement · 2 needing `join`'s monoid modeled · 2 generic
   instantiation, unread. **The error shape is the same one this row already records, in a second
   place:** a property of the SUBJECT read as a property of the FAILURE — twice in one night, by
   lanes with no contact.

   **ROOT CAUSE FOUND AND ALREADY REPAIRED — and it is not a filter** (2026-08-20, found while
   verifying an unrelated measurement base; the row above is preserved because the *reporter's*
   selection rule is described correctly, but its cause was not). `src/v1/00_core.dag` declares
   `fn make_span(start, end) -> SourceSpan { file: "<synthetic>", .. }` — **a constructor with no
   file parameter that substitutes the sentinel for the argument it cannot accept.** Diagnostics
   built from a *combined* span (start from one token, end from another) therefore lost their file
   while keeping correct offsets; diagnostics built from a single token's span kept it. That is why
   the split was *exactly* location-presence and why every hidden row carried a real file — not a
   filter selecting, **a constructor destroying.** `b0b061764e3` (#8607, merged 2026-08-19 21:40)
   repairs it by switching two parse call sites to `make_file_span`, and its own message names the
   population independently: *"46-72 refinement diagnostics across the corpus went unlocated because
   file information was discarded even though byte offsets were correct and positions were
   recoverable."* **Consequence for every measurement in this row:** the 72/46/26 split is a property
   of the corpus *before* that merge, not of the corpus. Post-#8607 runs should show substantially
   more located rows, and a reader comparing them to these numbers is seeing a fix, not a regression.
   **Ceiling:** structurally impossible. **Why the defect looked survivable, and this is the part that generalizes:** a span
   with a *missing* file would have refused somewhere. A span carrying the STRING `"<synthetic>"`
   is well-formed, flows through every consumer, and renders as a plausible location — the
   fabricated-plausible-output failure (§5), committed by a span constructor. The sentinel is not
   a lesser form of absence; it is absence wearing the costume of a value, which is precisely what
   let 46 rows travel to a reporter and be read as a policy decision. **Next trigger — the residue,
   which is what survives the repair:** `make_span` still fabricates `"<synthetic>"` for any future
   caller, so the invalid state
   remains writable and the class sits at *mitigatable* on that axis rather than repaired; the climb
   is a span constructor that cannot be called without a file, at which point the sentinel has no
   constructor rather than a discouraged one. Secondarily, the count of genuinely-unlocatable rows
   remaining after #8607 has never been measured, and that residue — not the 46 — is the real
   remaining population. **Recorded because it is a routing fact and not only a technical one:**
   #8607 was authored, reviewed and merged by a lane outside this investigation, naming the same
   46-72 population independently, while this row's instruments were being built to characterize
   it. Two lanes converged on one defect from opposite directions and neither knew; the hours spent
   on the instrument were spent against a fix already sitting on main. Nobody was wrong — the fleet
   had no channel that would have surfaced it, which is the actual finding.

16. **The regen refusal receipt reports NOT-COMPUTED as MEASURED-AND-CLEAN** (opened 2026-08-20;
   measured by `snappy-eagle-615` reading `required_regen_host.rs` `run_required_regen` control
   flow, confirmed in tree). `validate_compared_populations` returns `Some(reason)` whenever the
   emitted and committed basename sets differ, and that arm **returns early** — so
   `compare_generated_surfaces` is never called and **no content comparison runs at all**. The
   population gate is a hard fail-fast ahead of every content check, not a parallel signal. The
   receipt written on that path then asserts, as facts, five values nothing computed, and they are
   **not equally bad**. `first_generation_equal` and `fixed_point_equal` are set to **`false`** —
   and a Bool has no arm for *not measured*, so `false` reads as **measured and unequal**: the
   receipt asserts a negative RESULT for a comparison that never ran. The other three are at least
   self-describing to a careful reader — both digests carry the literal sentinel
   `"refused:population"`, and `changed_paths` is an **actually-empty** `Vec` rather than a typed
   absence. **The compounding is the real harm:** this receipt shape was live *while* the
   population gate was masking content drift, so a reader held a receipt asserting
   `fixed_point_equal=false` — measured, unequal — produced by a run that compared nothing. Two
   defects that individually mislead, composed into an artifact that is confidently wrong in the
   format designed to be believed. **Harm:** a receipt reader — including
   anything keying off `changed_paths.is_empty()` — cannot distinguish **no drift** from **drift
   never checked**, and the two have opposite remedies. **Measured consequence:** at
   `9b29509e5c9` the mirror `infer_method_args_with_fold` was carrying **six parameters against
   the authority's seven** (the missing `arg_contract: DeclaredArgContract`), and that drift was
   in the emitted set, comparable, and simply never compared — masked behind a two-file population
   mismatch unrelated to it. **This is the class DESIGN names as the absorbing fallback's mirror,
   the empty-observation narrow** (⊥-as-answer conflated with ⊥-as-ignorance) and it is the
   strictly worse direction: a widen is merely expensive, a narrow is silently uncovered. It is
   also distinct from rows 11–15 in *where* it sits — those are reporting channels, this is a
   **receipt**, the artifact whose entire purpose is to be believed later by a reader who did not
   watch the run. **Ceiling:** structurally impossible — a receipt for a comparison that did not
   run has no honest values to carry, so the refusal arm should construct a *different type* with
   no digest, equality or changed-path fields, rather than the same record filled with sentinels.
   **Next trigger:** `RegenReceipt` splits into computed and refused variants, at which point the
   `"refused:population"` sentinel and the two fabricated `false`s become unwritable rather than
   discouraged. **Under the convergence model the operator ruled for (2026-08-20) that split is not
   optional:** a *converged* verdict and a *could-not-determine* must not inhabit one Bool, which is
   the same hazard on the read-back side — a converged verdict derived from the staged candidate
   rather than the committed tree would report success while the committed files stayed stale.

17. **A generated file declares its own provenance in a comment no consumer reads** (opened
   2026-08-20; found by `stern-tern-636` in a candidate tree, after `royal-hawk-392` and this
   session both mis-inferred the opposite from style). 129 of 167 files under
   `src/v1/stage0/src/` open with `// Generated by v1 compiler -- do not edit.` and a
   `// Source module:` line. **Nothing reads either.** `v1_compiler_parse.rs` — carrying both
   lines — was hand-edited under #8607, landed applying ONE of that change's two call sites
   (authority `make_file_span(` = 2, mirror = 1; `call_span` still calls `make_span`), passed
   every check, and was then cited as the fix by three sessions including this one twice.
   **Harm:** the running compiler still fabricates `"<synthetic>"` on the caret-call arm while
   the tree reads as repaired — and *half-applied is worse than either pole, because it reads as
   done.* **The class is specification-without-execution in its purest form:** the file states a
   machine-checkable claim about itself, in a format designed to be read, and no machine reads
   it. **Why style could not settle it, which is the reusable part:** both prior readings argued
   from a nine-line import reflow that "no human performs for a two-line edit." **A reflow proves
   a FORMATTER ran, not that the EMITTER ran** — `rustfmt` stands between author and committed
   bytes, so any hand edit acquires the emitter's formatting, and the signature read as a
   fingerprint is applied by a third party to both suspects. Two sessions agreeing on one unsound
   inference is not two pieces of evidence. The discriminator that *did* settle it is structural
   and came from running the generator: true emission hoists
   `pub use crate::v1_std_core::make_file_span;` onto its own line and leaves the grouped `use`
   without it; the committed file has the grouped form. **Rule: do not infer provenance from
   style — compare against what the producer actually produces.** **Ceiling:** structurally
   impossible; a generated artifact that cannot be hand-authored without detection is a
   content-addressed identity question, not a comment. **Next trigger:** any consumer at all
   reads the declaration — the nearest is the regen convergence model's observe step, whose
   `MembershipPlan` reports this file as `MemberChanged` by construction. **Note the interaction,
   because the remedy erases its own evidence:** convergence repairs the drift and destroys the
   only trace that a generated file was ever hand-authored, so it closes the instance and leaves
   the class exactly where it is.


## 12. Proposed sequencing (reconciled with the independent review; for operator sign-off)

**(2026-07-31 restructure.)** The canonical dependency order now lives in the roadmap
authority itself — the `compiler-guarantee` lane's declared nodes and edges
(`gunbc.roadmap_authority` `guarantee_ladder_nodes` / `guarantee_ladder_edges`) — after the
operator's verdict caught the first cut rendering these nodes from a section-local list
outside `declared_roadmap_nodes()`. This section stays as the narrative rationale; where
prose and edges disagree, **the edges are the authority**.

**Stage 0 — the measurement schema (added by the seventh-pass verdict, before probes AND
carrier).** The first spine increment is neither probes nor carrier but the protocol they
meet through: `GuaranteeClassId` / `GuaranteePathId` / `GuaranteeProbeId` as typed
identities, `GuaranteePath` (`subject_grain`, `acceptance_boundary`,
`realization_target` — the landed `gunbc.guarantee_measurement` shape: `compile_mode` was
deleted as a derivable dual representation and `InterpreterRun` re-read as the
realization-agnostic `RuntimeRun`, review 46308 on gunbc#7572), and
`GuaranteeMeasurementReceipt` (class × path × probe ×
`subject_revision` × `harness_revision` × `probe_set_digest` × observed outcome). Without
this stage the probe corpus and the carrier each invent the receipt shape and meet in a
protocol cycle — probes can't emit rows the carrier hasn't defined, the carrier can't
derive dispositions from receipts that don't exist. Schema lands with synthetic-row
witnesses only; no probe executes here.

**Stage 1a — the probe corpus, as `.dag`.** One probe pair per floor class
(deliberately-bad input expected to refuse; legitimate control expected to accept), landed
as **enrolled expecting-red rows** (the known-red quarantine mechanism already exists: rows
execute *expecting* red, so a wall landing flips them loudly to controls). That makes the
corpus a continuous measurement of the gap before any wall exists — each
compiles-when-it-should-refuse is a counted deficit, not an anecdote — and it is what the
carrier's dispositions derive from, which is why it precedes the carrier. **MIGRATE, never
duplicate** (seventh-pass verdict): the landed walls' REDs and positive controls
(gunbc#7484, #7519, #7485) move here as the classes' enrolled evidence — a second copy
beside the dark-suite original would be the §2 fork; migration is also what begins
discharging §11 item 9's finite classification.

**Stage 1b — the claims carrier, measurement separated from claim.** Model the guarantee
population (the review's G0–G9 families are a good candidate cut: binding · formation ·
inhabitance · invocation · domain/totality · control soundness · semantic dimensions ·
realization · fidelity · external boundary) as **four carriers**: `GuaranteeRequirement`
(class identity, domain, harm, `ceiling` with its mathematical/capability/price
justification, `next_rung_trigger`); `GuaranteePath` (`subject_grain`,
`acceptance_boundary`, `realization_target` — the landed schema row; one per in-scope path,
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

**Stage 1c — baseline prevalence, anchored as a two-revision execution** (split out of the
old Stage 7 per the first verdict; anchored per the post-merge verdict; execution semantics
fixed by the seventh-pass verdict): the whole-corpus floor rerun bucketed by ladder
position, keyed to the carrier's class ids — the honest *before* picture. "The baseline at
commit X" is ambiguous three ways (whose compiler, whose probes, whose corpus), so the
baseline is defined as a **two-revision execution**: the anchor's compiler artifacts
(`anchor_commit 6c6e2dcb8587d73350ac252f5b07a6b50d684485`, the #7489 merge, pre-P0
implementation state) run under **today's content-addressed probe set** (`probe_set_digest`
from the Stage-0 receipt schema), so the before/after diff varies exactly one factor.
Content-addressing both sides is what makes it reproducible after the in-flight branches
merge — requiring every implementation node to wait on an unanchored live-tree measurement
would be fragile, so the walls sequence after the baseline *node* without racing the live
tree.

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
encode exactly that (slugs per the seventh-pass recut — the monolithic
`floor-method-existence-wall` is tombstoned): `floor-method-ambiguity-wall ←
primitive-identity-join + method-zero-resolution-general-wall`, while the method chain's
open entry `method-receiver-normalization` follows the anchored baseline only and
`method-established-surface-wall` stands accepted against merged #7484. The two resolution
defects are §11 item 8 and are exactly `method-receiver-normalization`'s scope.

**Stage 3 — one cardinality vertical slice** (§4b: connect the 2026-07-04 operator
direction + the scoped lattice plan + the manual value-level fold specimens; acceptance =
the operator's scenario refusing at the seam, with the nonempty-proof positive control
compiling; the construction-wall candidate is gated on the §11 item 1a `sole_constructor`
audit — a declared roadmap edge, not prose).

**Stage 4 — v2 phase work, split (post-merge verdict: the monolithic phase-carriers node
over-promised against what gunbc#7485 delivers, so it is five staged nodes; its roadmap
identity is the registry's first tombstone):** `v2-self-grounding-frontier` (the
strict-narrowing slice, **landed — #7485 merged, acceptance receipt 2026-08-01**:
self-evidence out, underived facts become counted `GroundingNotDerived` frontier
diagnostics) →
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

**Stage 6 — remove the compiler-source exemption, gated on the two groundings** (deps
corrected by the seventh-pass verdict: the exemption skips the argument-TYPE judgment,
whose false positives are the four measured representation gaps — the label wall never
gated it, and #7519's wall already runs exemption-free). Once
`argument-type-compatibility-grounding` and `declared-conformance-grounding` land, rerun
exemption-free over `v2.*`/`v1.compiler.*`, classify every failure fresh, fix relation or
source, delete; the unsourced 104 is neither a blocker nor a promise.

**Stage 6b — the acceptance door, split mechanism-then-activation (seventh-pass verdict;
supersedes the monolithic `compiler-accepted-obligation-closure`, tombstoned):** the
terminal P0 contract making the carrier the contract rather than observability — every
`Required` guarantee path has exactly one live consumer, consumed before the relevant
`Accepted` constructor. The monolithic cut delayed the mechanism until every wall landed;
the split lands it early without certifying anything false: **door-mechanism** (← carrier
alone) reads Required paths and derives the missing/unmeasured/frontier/no-live-consumer
census, audit-mode and expecting-red — it reports, never blocks; **floor-closure** flips
refusal on over the floor domain once the floor walls land; **extended-closure** widens
class-by-class (cardinality, ambiguity, v2 target realization, Translate containment) as
each climb lands. Review 45545's substance survives in the activation nodes — refusal
never turns on over a known-open required judgment; what changed is that the non-blocking
audit no longer waits (residual prevalence now depends on the extended closure).
**Correction (review 45918, on the first cut of this split):** the extended activation as
one node carried requires-all edges on all four climbs while its prose promised
class-by-class widening — the graph semantics would have deferred every admission behind
the slowest climb, preserving exactly the monolithic deferral the split dissolves. Recut:
**four per-class admission nodes** (`extended-admission-cardinality` / `-ambiguity` /
`-v2-realization` / `-v2-translate`, each ← floor-closure + its own climb, so each becomes
ready the day its climb lands), and `accepted-extended-obligation-closure` is the terminal
**roster-completeness certification** (every admission receipt present; nothing
climbed-but-unadmitted, nothing admitted-but-unclimbed) — requires-all belongs on the
completion receipt, never on the admission work.

**Stage 7 — residual prevalence, measured last** (the baseline half moved to Stage 1c per
the operator's verdict, anchored at `anchor_commit 6c6e2dcb8587` — the #7489 merge — so it
is content-addressed and reproducible after in-flight branches merge): the same bucketing
classifier — statically-decidable / runtime-value-dependent / external-boundary /
resource-budget / capability-not-grounded / interpreter-defect — re-run after the declared
climbs land and **diffed against the anchored baseline**, so each wall's landing is a
measured before/after receipt, never an assumed win.
