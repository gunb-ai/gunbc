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
| A value that does not inhabit its declared type is accepted — **seam-split, enumeration UNFINISHED** (was filed as "Direct-call argument TYPE conformance, v2 corpus"; renamed 2026-08-22, see the trigger cell) | **Below floor — the judgment never runs.** Not a hole a bad value slipped through: `v1.compiler.04_infer` gates `arg_compat_diags` on `module_skips_direct_call_arg_check(module_name: scope.module_name)`, which returns true for every module whose name begins with `v2.`. The key is the CALLER's module, so the entire active v2 corpus — compiler stages, extdeps, std, witnesses — has the direct-call argument-type judgment switched off. Measured consequence, 2026-08-22: `v2.extdeps.languages.dag` `dag_int_literal_node_from_lexeme` / `dag_int_literal_node_from_magnitude` declared `occurrence_id: OccurrenceId` since gunbc#6558 while storing that parameter straight into `Node.occurrence_id`, declared `NodeOccurrenceId`. For two months 15 call sites passed `SyntheticOccurrence` (a NodeOccurrenceId, contradicting the declaration) and nothing complained; gunbc#8833 made one call site OBEY the declaration and nothing complained about that either. The error surfaced only as an interpreter `non-exhaustive pattern match on: OccurrenceId { value: 79 }` across 16 witnesses, reddening main. **The inversion worth naming: a declaration that lies is inert while every caller contradicts it in the same direction, and detonates on the first caller who takes it at its word.** So the at-risk population is not "callers who got it wrong" but callers who might get it right — a genuinely counterintuitive census. Note this is the TYPE judgment only; the SHAPE judgment (`direct_call_shape_diags`, labels/arity) is exemption-free and does fire | R2 (a declared parameter type is a decidable conformance check the seam already computes — `direct_call_arg_mismatch_diags` exists, is written, and is simply not called, so the distance to the next rung is an if-statement plus a triage, not an implementation) | **THIS ROW IS A CORRECTION TO DESIGN §4b, NOT A NEW OBSERVATION BESIDE IT — read them together or the first will look like it already settled this.** DESIGN §4b names this exact symbol and reports that it "was found (code read, not execution) to be scoped entirely to the direct-call argument-type judgment and does not reach `sole_constructor`'s construction check at either call site — a positive finding that retires this axis, not an absence of any exemption anywhere." That finding is correct and is not disputed here. The question it asked was whether the exemption LEAKS into `sole_constructor`; the question it never asked is what the exemption COSTS inside the judgment it is scoped to. "Scoped entirely to the direct-call argument-type judgment" reads as reassuring only until that judgment is measured, and it is the argument-type check for the entire active v2 corpus. **Confinement was measured and then treated as safety, and the axis that got retired was not the one that mattered.** Two facts about why the seam nonetheless reads as covered: the SHAPE judgment (`direct_call_shape_diags` — labels, arity, duplicate binding) is exemption-free and DOES fire over every module including the compiler's own sources, so half-live is more deceptive than dead — every casual check finds something working. And the arm is NOT unreasoned: `direct_call_shape_wall_note` states the TYPE judgment's false-positive classes are representation gaps (brand aliases, optionality's two forms, anonymous literals, expansion depth — the conformance wall's four measured classes), which is a real stated reason and is why this row asks for a measurement rather than a deletion. What is unmeasured is whether those classes still fire in v2 today and at what rate. The neighbouring compiler-module arm of this same exemption was deleted in place after being found never to have been in force; the `v2.` arm was left standing at that moment without separate justification of its own | This incident, measured end to end: CI run 32542017600 (main, 67437fcbe9) 16 FAIL with the raw-`OccurrenceId` match error; caller census of both functions read at call grain (17 sites: 15 `SyntheticOccurrence`, 1 `node.occurrence_id`, 1 internal pass-through, 1 raw `minted.id`); exemption mechanism read at `v1.compiler.04_infer` `module_skips_direct_call_arg_check` and its single call site gating `arg_compat_diags` on `scope.module_name`. **NOT ENROLLED**: no probe pair asserts this exemption's reach, so nothing reds if it changes | **THE TRIGGER THIS ROW SHIPPED WITH WAS INCOMPLETE AND IS CORRECTED HERE (2026-08-22, swift-badger-524 retracting their own ruling).** As merged in gunbc#8854 this cell named ONE closing condition — delete the `v2.` exemption — which is a NECESSARY condition presented as a sufficient one. The class is not "direct-call argument type conformance"; it is **a value that does not inhabit its declared type is accepted**, and it has at least TWO seams: **(1) direct-call arguments** — gated off for `v2.*` by `module_skips_direct_call_arg_check`, the mechanism this row measures; **(2) record-literal fields** — gated by NOTHING, below floor in ORDINARY NON-v2 MODULES, receipt gunbc#8865 (gentle-eagle-360). Its minimal pair, no compiler internals involved: `CppHolder { subject: CppWrapped { inner: cpp_inner() } }` accepted and correct, against `CppHolder { subject: cpp_inner() }` — a coproduct PAYLOAD inhabiting a field declared as its parent COPRODUCT — ACCEPTED BY TYPING and dying at runtime as `PatternMatchFailure`. A record literal is not the direct-call seam, so deleting the exemption would not close it. **SEAM (2) NOW HAS A LIVE PRODUCTION SPECIMEN, WHICH RANKS IT DIFFERENTLY FROM A SYNTHETIC PAIR** (gentle-eagle-360, 2026-08-22, found while checking for a second content-identity authority before building on one): `v2.std.materialize` declares `MaterializedNode.hash` as `ContentHash` — the subject-generic union — and `materialize_fold_step` stores `content_hash(n)` into it, which returns `Fnv1a64Structural`, the PAYLOAD carried inside the union's `Fnv1a64` arm. Receipt, by execution against the module as it stands on main: taking the value materialize ACTUALLY STORED and asking the union's own family authority `content_hash_family` about it yields `PatternMatchFailure { value: "Fnv1a64Structural { digest: 7ac77ab0e29c6bd8 }" }`; the probe's fallback was a correctly wrapped `Fnv1a64(...)`, so an empty result or a well-formed value would have PASSED and only a stored value can produce that red. **The defect is LATENT and that is the instructive part**: peer lookup compares `e.hash == h`, raw payload against raw payload, which agrees with itself — so every current path is green and the error surfaces only when a materialize hash is routed through the union's own machinery (`content_hash_family`, or `compare_content_hash`, which exists precisely to refuse cross-family collapse). That is the same shape as this row's own originating incident: **a declaration that lies is inert while every consumer contradicts it in the same direction, and detonates on the first consumer that takes it at its word** — here the first cross-family artifact to reach the carrier. Disposition ruled ARM A, wrap the construction (`MaterializedNode { hash: Fnv1a64(h), ... }`), NOT narrow the declaration: narrowing would sever `MaterializedNode.hash` from `RealizationPlan.target`, and §2's Realization spans resolve-cost, sccache and OS provisioning on ONE content hash, so a structural-only key could never name a realization of a fetched artifact or a provisioned image (swift-badger-524 ruling, 2026-08-22; handed to silent-bear-842 with the receipt, unfixed here because it is that lane's module). **THE SEAM ITSELF IS THE FINDING, not this one site**: gunbc#7480 corrected the design document for asserting the wrong type at exactly this `Hash`/`ContentHash` boundary, and a production module now carries the mirror-image error — a boundary documentation has to keep re-explaining is a boundary that wants construction. **THE ENUMERATION IS UNFINISHED: two is what has been measured, not the count.** Assume a third seam exists until someone enumerates them; note that this row's own originating incident (#8854) reached its victim through a record literal one hop downstream of the direct call, so the two seams are not even cleanly separable at a site. **SEAM (1) WAS EXECUTED TO A MEASURED, BLOCKED STOP ON 2026-08-22 — the arm is NOT deleted, and this paragraph is the §4b row for that state rather than a plan.** Rung: **below floor, unchanged** — the judgment still does not run for any `v2.*` caller. Ceiling: **R2** (a declared parameter type is a decidable conformance check the seam already computes). Next trigger, stated as §4b(2) requires and with its disposition: *a formal parameter declared as an applied generic carries its applied form through resolution rather than reaching the comparison seam as the bare constructor* — **DECLINED at the owning layer on 2026-08-22**, on the ground that a bounded compiler-floor packet whose completion condition has moved should close and hand its residue forward rather than extend into a type-system project. A declined trigger is a legitimate §4b state; **a blocked class with no row is how this exemption survived years in the first place**, which is the reason this row exists at all. Enrolled evidence, so the class stays countable while blocked: `direct_call_arg_type_v2_module_red_probe` (the discriminating RED, rostered in `v2.workflow.floor_expected_red` — it cannot pass while the arm stands and PASSES on a tree with the arm deleted, so it is a satisfiable assertion held open by a named cause), plus two controls that pass on main today (`direct_call_arg_type_ordinary_module_red_probe`, the paired nonzero the exemption never covered; and `direct_call_arg_type_v2_green_control_probe`). **What was measured before the stop, and how to read it:** deleting the arm produces **285+k** blocking diagnostics and **67+k** with gunbc#8873 merged, for some k ≥ 1 unmeasured — these are ROSTER-RESOLVED counts, not corpus counts, and `src/v2/extdeps/formatters/lean4_format.dag:184` is the proof: a live site, structurally identical to eight that refused, contributing zero to *both* arms and therefore invisible to their difference. **Zero genuine call-site defects were found in the residue** — every diagnostic examined was a deficit in the type judgment, so no call sites were edited, because 67 mechanical edits would have cemented three compiler deficits permanently. The per-mechanism split of that residue is deliberately NOT recorded here: it was published as 48/11/8, and the rule the 48 were attributed to was then found to admit nothing in this corpus, so the attribution was retracted before it was cited. Trigger for the count to be restated: the residue re-measured with its `why` column. **The counterexample that retracted it is worth carrying on its own, because it will bite anyone who reaches for a name-keyed alias relation:** `type Float = Float64` is declared in BOTH `dag/std/float.dag:18` and `src/v2/std/float.dag:33`, spelling the target identically — but `dag/std/float.dag:16` has `type Float64 = Real64` (a transparent alias) while `src/v2/std/float.dag:30` has `Float64` as a **record**. So the two declarations agree on a *spelling* and disagree on a *type*, and a unanimity rule keyed on the target's NAME admits them as the same concept. Requiring the target itself to be census-unique closes it — and once closed, that rule admits nothing in this corpus, which is what falsified the attribution above. Found by still-carp-717 while building the rule, before it shipped. **The original trigger text, kept because the sequence it specifies was followed and the record should show what was attempted:** seam (1) — turn the exemption off for `v2.` behind a measured diagnostic count, triage, then delete the arm, the disposition the compiler-module arm received; the measured prerequisite is ALIAS TRANSPARENCY, because proud-ant-819's report-only shadow on the 03_ingest closure found 115 `WouldDiagnose` relations at 78 sites of which 115 reduce to a transparent `type A = B` (92 via `type Hash = Fnv1a64Structural`, 23 via the `Node` phase carriers), residue ZERO — so on that closure the exemption is currently suppressing false positives, not defects. Seam (2) — its own wall at record-literal field conformance; nothing to turn off, it was never on. **Closing seam (1) MUST NOT be read as closing the class**, which is the specific failure this correction exists to prevent: a class that looks like it has one closing condition gets closed when that condition fires. Two blind spots neither instrument covers, named rather than left to be rediscovered: production itself SKIPS an anonymous record literal standing as an ACTUAL at a direct call, so its argument type is never judged (521 relations, 2.7%, `Unadjudicated` in proud-ant-819's shadow — upstream of the guard, so invisible to a flip-off arm too); and a function storing into a field from its own differently-named parameter is this class at yet another site, which a callee-name-keyed census cannot see. **THAT FIRST BLIND SPOT IS NOT SEAM (2), AND THE TWO WILL BE MERGED BY ANY READER WHO DOES NOT SEE THIS SENTENCE** (caught by proud-ant-819, who put the mirror-image clause in their own row): both descriptions begin "a record literal" and name DIFFERENT POSITIONS. The blind spot is a record literal at an ARGUMENT position whose type goes unjudged at the direct-call seam — seam (1)'s territory, and a population of 521 relations. Seam (2) / gunbc#8865 is a record literal's FIELD whose value does not inhabit the field's declared type — a different seam with a different mechanism and no measured population. Reading them as one would make the 521 look like evidence for gunbc#8865, or gunbc#8865's receipt look like it bounds the 521. Neither is true |
| Direct-call argument TYPE conformance — **transparent-alias identity**, v2 corpus | **R2 built and approved for the alias-identity class (gunbc#8873 — OPEN, approved, mergeable as of 2026-08-22; NOT merged, and this row must not be read as landed until it is).** A precomputed transparent-alias identity relation, derived once at census build and consumed as a `String -> String` map lookup per comparison, admits two names that alias the same declaration and refuses everything else. It peels ONLY `type A = B` with zero params, no connective, no children, no properties and no type annotation, and a target with zero children/params/annotation and `return_cardinality == Required` — so brands, `sole_constructor` carriers, refinements, coproduct arms and applied generics are all excluded BY THE ADMISSION TEST rather than by a later filter. Cost bar was declared before implementation and measured after: four crossed A/B pairs, +2.0s on a 454s regen (+0.4%) against per-arm spreads of 29.9s and 40.9s — a measured null. **What it does NOT do is create substitutability: it makes an existing fact visible.** A pre-relation binary already accepts `CommitSha` at an `AttemptKey` formal and already refuses `IntKey` there | R2 for this class. R3 is not reachable at this seam and that is a property of the seam, not of the relation | **The residual population is a DIFFERENT class and the distinction is load-bearing.** Measured 2026-08-22 against the exemption-deleted corpus: 67 refusals, silent-badger-817's CI run 32562740527 at commit `d42bb3eb57c`. **Provenance caveat, stated because it changes what the 48 means:** that tree merged this relation at `9bfd5388c03`, TWO COMMITS behind head `c40ab2d8248`, and the intervening `b0f72158c80` ("Qualify alias targets at census-build time") is exactly the commit that decides the `CoreNode` group — those refusals turn on `Node` being ambiguous by BARE name. So the 67 is a measurement of an OLDER revision of this relation, not of a broken copy of it. Classification of those 67, derived from declarations and independent of any binary: 48 are this alias class; 8 are applied generics (row below); 11 are `Optional` against `String`/`Bool`/`Int` in `v2.extdeps.github.gha_fold_pilot_emit` and are NOT an alias class at all. **The 48-cleared half is verified on 5 specimens, not 48:** at head, with the exemption deleted in the mirror and `exempt=judged` on all 20352 ledger rows, the five comparisons that CI reports as errors in `00_compile.dag` (lines 319, 342, 408, 446, 497) are all present in the ledger and all read `Compatible`. The remaining 43 sit outside that entry's import closure and a whole-corpus re-run against head was in flight when this row landed — treat 48 as classified-and-partially-verified, never as measured. **Also NOT the alias class: the 11** — the diagnostic reads got `Primitive(String)`, so the optional marker is already gone upstream at pattern destructuring and no comparison-seam mechanism can reach them. That 11 was carried as alias residue by two independent sessions and by this lane's own framing; it is recorded here so it is not handed forward under a label that guarantees the next attempt fails. **Open question for whoever takes it, to be CHECKED rather than inherited from this row:** an optionality marker erased at destructuring means a value reaches a position whose declared type it does not inhabit, which is the same floor rule as the direct-call seam census (record-literal walled; data-initializer and list-element open). If Group C is a FIFTH seam of that class it belongs on that row and not on a new one — decide that before minting a class for it. **A second mechanism, `unanimity`, was designed, measured and ABANDONED as unsound, not deferred.** Its specimen and refutation are carried on the seam-split row above and are deliberately NOT restated here — one fact, one authority | gunbc#8873, 9 enrolled floor witnesses in `dag/test/claim/transparent_alias_identity_witness_test.dag` including the over-peel boundary (`IntHandle` at a `String` formal), a coproduct-projection climb, and a blast-radius pair whose Int-alias discriminator refuses. Shadow ledger re-run before the exemption was touched: 20527 rows, all 115 `WouldDiagnose` rows one mechanism. Receipt: `docs/probes/transparent_alias_identity_2026-08-22/README.md` — **FORWARD REFERENCE, NOT A PRESENT FACT:** that receipt lands with gunbc#8873, which is open at the time this row merges. Do not cite it as evidence until that PR is in | the `v2.` exemption arm cannot be deleted while the applied-generic class below still refuses 8 live sites; the exemption is NOT narrowed to those modules (a narrowing would be a shape test correlating with the distinction rather than naming it) |
| Direct-call argument TYPE conformance — **applied-generic alias**, v2 corpus | **Below floor — 8 live false refusals, structurally unreachable from the comparison seam.** An alias whose right-hand side is a generic application (`type BlackConfigPatch = ConfigPatchRecord<BlackConfig>`) refuses at every call site passing it to the matching formal. Specimen, measured by execution 2026-08-22 on a fixture diffed byte-for-byte against the live `v2.std.patch` declaration, with the exemption deleted in the Rust MIRROR and `exempt=judged` on all 20352 ledger rows as the control: `formal_type = Primitive(ConfigPatchRecord)`, `actual_type = Node(BlackConfigPatch)`, `nominal=true container=false kernel=false`, `fname=ConfigPatchRecord aname=BlackConfigPatch`. Population: 9 declarations, all under `src/v2/extdeps/formatters/` (`black`, `clang_format`, `gofmt`, `google_java_format`, `ktfmt`, `lean4_format`, `prettier`, `rustfmt`, `swift_format`), of which 8 produced diagnostics in the CI arm — `lean4_format` is structurally identical and produced none in EITHER arm, so the population is 9 and the measured count is 8+k | R2, and it is decidable — the same equality already accepts the applied form when it is written directly at the formal | **THE FORMAL SIDE IS WHERE THE INFORMATION DIES, and that is the sentence that should stop the next attempt from this direction.** The formal is DECLARED `ConfigPatchRecord<Config>` and REACHES the comparison as the bare constructor, with the type argument already discarded. So the equality has one side that structurally cannot hold an argument, and the case split over any representative is EXHAUSTIVE: (a) representative = the applied form with every argument retained never equals the bare constructor, so it admits nothing and all 9 sites keep refusing; (b) representative = the bare constructor admits `ConfigPatchRecord<X>` at `ConfigPatchRecord<Y>` for any X and Y, which is exactly the over-peel that erases the distinction. There is no third representative, and widening the carrier does not help — a node-valued map fails identically, because the missing information is not on the relation's side of the map. **Not to be confused with gunbc#8879**, the withdrawn repair that widened transparency AT RESOLVE and was refused by CI in three classes at once (20 direct-call comparisons, 18 variant projections, 3 files of regen drift). What that established is that resolve is the wrong place for a transparency JUDGMENT; preserving an applied form is resolve DISCARDING LESS, which hands the same consumers MORE structure, and is a different claim that has not been measured | executed fixture + shadow ledger with a positive control (`black.dag` must refuse; a zero there is decidably wrong). **NOT ENROLLED**: no probe pair asserts this class, so nothing reds if it changes. Three false zeros were produced while measuring it — twice from patching the `.dag` authority while the binary is built from the emitted Rust mirror, once from a run that PANICKED and reported zero diagnostics — each caught only by a positive control or by another session's raw per-line export, never by an aggregate | **RESOLVE CARRIES APPLIED FORMS AT THE FORMAL POSITION.** Explicitly NOT scoped as of 2026-08-22 (operator-lane ruling, swift-badger-524): a bounded floor-recovery lane whose completion condition has moved is closed honestly rather than extended, and nothing in that ruling says the change is wrong. Until it lands, the `v2.` exemption stays whole and gunbc#8886 stays blocked |
| Producer/consumer cardinality | **UnknownUnmeasured** (typed-rejection vs silent-degeneration split unmeasured) | R3 (seam unwritable) | forgeable carrier; no signature propagation (`sole_constructor` audit pending) | §4b | Stage-3 vertical slice |
| Refinement predicate (`where`) enforcement at construction | **Writable-and-unverified, one row spanning both measured SOURCE-FORM paths (declared subject grain — reconstruction is NOT in this row's grain; see the scope note at the end of this cell).** Source→v1-interpretation and source→v1-Rust-emission were independently measured and found identical — zero predicate/refinement check on either, no scope split between them. `v1_interpreter` `cast_identity_result` (the `as`-cast identity path) checks only that the underlying kernel name matches; it carries no refinement/predicate concept at all. `05_emit_rust` `render_rust_alias_rhs_type` peels a refinement node to its base type before emission (`is_where_refinement_type` true branch) and discards the predicate list — a **deliberate written branch**, not incidental loss: `04_infer`'s `is_where_refinement_type`/`where_refinement_chain`/`peel_where_refinement_base` carry the predicate live through typecheck, so the data exists at the emit boundary and is thrown away there. Where safety exists today for a refined type (e.g. `PathSegment = NonEmptyStr where brand("PathSegment")`) it comes **entirely from hand-authored per-callsite guards**, not the carrier — `gunbc.merge_admission_subject` `walk_attempt_id` and `gunbc.fleet_known_hosts_anchor` `fleet_ssh_attempt_identity` (mirrored in `claim_executor.rs`) each call `path_segment_is_safe` before casting. This is DESIGN §5's named tell, validation standing where construction was available: the law is declared once (`path_segment_safety_note`) and re-stated as a check every author must remember to write, never tied to the carrier itself. One construction form already bypasses even that discipline: `FilePathParts { segments: [...] }` struct literals (`extdeps.rust.cargo` `cargo_target_source_path`/`rust_module_candidate_paths`, `extdeps.linux.proc_self_cgroup`) coerce raw strings to `PathSegment` with no predicate call at all. A targeted search for an *existing* production site where a refined-type construction actually feeds a predicate-violating value through to a consumer (not manufactured) found none: the two real branded-`PathSegment` callers both guard, and the one `FilePathParts` caller with non-literal input has zero production callers (test-fixture only). A below-floor ("silent wrongness, demonstrated") claim was raised on this evidence and explicitly withdrawn — the class is confirmed writable and the one candidate gap is confirmed currently unexercised, not confirmed violated. **SCOPE NOTE, AND IT IS LOAD-BEARING: that withdrawal is scoped to the two SOURCE-FORM paths this row measures, and it has since been overtaken on a third path that this row does not cover.** The reconstruction seam — `recorded_fixture` `value_from_fixture_json` on replay, and `map_response_to_value_json` on live REST — builds runtime records and scalars from external JSON without passing through a source record literal or cast, so neither `cast_identity_result` nor `render_rust_alias_rhs_type` is on it and neither measurement above reaches it. On THAT path below-floor is not a withdrawn hypothesis but an EXECUTED demonstration: a predicate-violating value was silently accepted and consumed, and on the live-REST arm with no tampering of any kind. **The grain of that demonstration, stated so this cell cannot be read as more than it is:** it was executed against a PURPOSE-BUILT probe service, not against a production caller. What is established is that the MECHANISM silently admits and consumes violating values on an untampered path; what is NOT established is that any production field has actually received one. The production population sitting on that path is measured only as a declaration count (output-block fields typed as a refined alias), and none of those was executed — so read it as mechanism-demonstrated, population-unmeasured, never as demonstrated-in-production. It is intended to be carried as its own row rather than folded into this one, per §4b's one-row-per-structurally-distinct-path rule. **FORWARD REFERENCE, NOT A PRESENT FACT:** that row is authored in a separate in-flight change (gunbc#8658, reconstruction doors, findings-only) and is NOT part of this document as of this cell's landing. If this row merges first, the reconstruction row is pending rather than present, and this sentence is a statement of intent — do not cite it as evidence that the reconstruction path is documented, and do not let the absence of that row be read as the path having been cleared. **Do not read this cell as the class verdict**: per §4b a class's rung is the MINIMUM across its in-scope paths, so if the enclosing class is taken as "refinement enforcement at construction" — reconstruction being a construction path — the governing rung is the reconstruction row's, not this one's. Citing this row's writable-and-unverified while that path stayed silent is exactly the inflation §4b names | R2-shaped on the Rust target (newtype + validating constructor closes construction-time) | Bounded by what the **target** can express, not by what `.dag` knows: `.dag`'s own infer stage already carries the predicate live through typecheck, so the ceiling here is a target-capability question — the bare-alias Rust realization used today has no ceiling above writable-and-unverified regardless of source-side modeling quality; nothing on either measured path ties the declared law to the carrier, so enforcement is 100% per-callsite discipline, which has a zero-frequency-until-violated failure mode (the `FilePathParts` gap is exactly this: zero frequency because it has no caller yet, not because it is guarded) | node://adhoc-897a90b6-a9c items 1–2, 2026-08-20 (merry-tern-237/fierce-ant-91): 219 `where`-declared types found in `.dag` (`grep -rn '^type .* where ' dag/`), 49 confirmed reaching Rust as bare aliases with zero predicate residue (spot-checked `std_types.rs`); emission mechanism confirmed by reading `05_emit_rust` `render_rust_alias_rhs_type` against `04_infer` `is_where_refinement_type`; interpreter path confirmed by reading `v1_interpreter` `cast_identity_result`/`eval_cast`; violation search executed against `PathSegment` construction sites corpus-wide, empty result | a Rust-target realization for refined types (newtype + validating constructor, or equivalent) that ties the declared predicate to the emitted carrier once, replacing per-callsite discipline; the `sole_constructor` completeness audit (in flight, separate lane) is the adjacent unforgeable-construction mechanism this would likely compose with |

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
nonliteral refined argument is **`Absent`** (five `WhereRefinementUnenforced` deferral
reasons are enrolled as *advisory* — `v1.compiler.core`
`where_refinement_deferral_reason_scaffold_note`) — **corrected 2026-08-20 from
`RuntimeBoundaryOnly`, which was measured to be true of no kernel; see the executed row below**; v2 loop termination is a ConstructionWall
(`v2.std.cardinality` requires a declared loop measure, fail-closed to `DescentUnknown`);
unknown method is a fail-open Absent; host state is ExternalNotGuaranteed by the guarantee
statement itself.

**Executed correction to the nonliteral-refined-argument calibration (2026-08-20).** That
example read `RuntimeBoundaryOnly` — *a check that fires at the runtime boundary rather than at
compile time*. Measured across all three kernels the refinement family uses, **no refinement
predicate is ever evaluated, on any kernel**. Three mechanisms, one outcome:

| kernel | mechanism | executed evidence |
| --- | --- | --- |
| String | the conversion runs and is the **identity** | `"" as NonEmptyStr` evaluates successfully and returns `""` |
| Int | the only conversion **refuses every value**, so it is never used; values arrive by *declaration* and no conversion runs | `x as EpochMs` refuses a valid `5` identically to `-1`; and `-1` reaching an `EpochMs`-declared parameter comes back `-1` unchanged |
| collection | a sound, unforgeable wall with **zero traffic and zero declarations behind it** | `NonEmptyVec`/`NonEmptyBTreeSet` have private tuple fields so `new` is the only door, and `new` has **zero call sites** corpus-wide; **zero** refinement declarations exist over any collection base |

A gate that passes everything, a gate nobody walks through, and a wall with no door behind it.
The Int row is the one that most needs stating: *fail-closed by absence of capability* reads as
safe, and it is not — 74 declared `: EpochMs` / `: Duration` positions against effectively zero
casts means the refusing conversion is almost never on the path, so the predicate goes
unevaluated by a third route rather than by a permissive one.

Why this is the document's own §4b failure rather than a wording slip: one state was recorded for
a class whose paths differ, and it was the strongest of them. In emitted Rust `NonEmptyStr` is a
**bare alias** (`pub type NonEmptyStr = String;`), so a violation is not merely unchecked — it is
unrepresentable as a distinction, which does not reach even the *brand* that §4b calls cosmetic.
Each kernel needs a different remedy: String needs a carrier to exist at all; Int needs a
conversion that can succeed; collection needs the emitter to route values through the wall that
already exists (a plausible, **unproven** cause for the orphaned carriers is a name mismatch —
the type mapping emits `non_empty_list` → `NonEmptyList<T>` while the emitted carriers are
`NonEmptyVec<T>`/`NonEmptyBTreeSet<T>`, with `NonEmptyList` modeled through `Refined<List<T>>`
rather than `where`).

**Population, as a dated measurement rather than a property of the tree:** a whole-tree resolve at
`a750b6761da` counted **3939 `WhereRefinementUnenforced` across 612 files** (of `TOTAL_ALL` 8183;
`HARD` 0, so every hard-diagnostic census structurally reads zero for this class). Four
pre-committed predicates passed, including a by-name planted-row control, and the count reproduced
exactly across two dispatches on separate runners. An independent static scan — sharing no
machinery with the resolve-time census — counts 3757 `as NonEmptyStr`, smaller in the predicted
direction because a text scan for one spelling of one type must count strictly less than a census
over every refinement construct. **The instrument was deleted and nothing landed, so the number is
not reproducible without rebuilding it.** And it is an *unverified-obligation* population, not a
defect population: the census cannot distinguish violated from unverified — the planted
known-violated row was indistinguishable from the other 3939 in every output.

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
| Cardinality / multiplicity (empty list into a callee that requires one) | **RepresentableButForgeable, not statically propagated** — reclassified from UNEXPRESSIBLE after independent review | Representation exists: `v2.std.refinement` `Validation<B>`/`Refined<B>`/`refine`, a `NonEmptyList<T>` manual fixture (`v2.test.claim.manual.refinement_nonempty_list` + testgen anchor), and manual value-level algebra specimens (`cardinality_fold_propagation_test` — length homomorphism over literals + runtime `refine_byte`; no binding-level propagation). Forgeable: `Refined<B>` is a public record with no `sole_constructor` seal and no predicate-identity field, so `Refined { base: x }` is a writable record literal at any call site regardless of which predicate (or none) was ever checked against `x` — confirmed live at `v2.std.artifact` `test_claim_generated_artifact`, which mints `Refined { base: Artifact {..} }` directly instead of going through `make_generated_artifact`'s `refine(...)` path in the same file. (`refined_vacuous_stub_pack`'s `Rejected` arm forging `Refined { base }` was a second, narrower instance of the same door; `v2.std.refinement` `refined_vacuous_stub_pack` is now deleted outright, together with its sole non-test consumer — `v2.test.generated.testgen_category_wishlist` `dispatched_refinement_preservation_generator`'s vacuous fallback path — in favor of `v2.lens.testgen` `refinement_preservation_subject_nonempty_list_base`, which already proves the same subject through a real `non_empty` predicate via `refine_nonempty_node_list`. Deleting the mint rather than leaving a more-honest total function is the DESIGN section 3 replacement-migration rule applied to a construction that had exactly one real caller and a correct replacement already in use elsewhere; it establishes no predicate identity and does not touch the carrier-level forgeability above.) Not propagated: no cardinality lattice in signatures (`v2.std.cardinality` is loop-termination), `InterfaceSummary` (`dag/std/interface_summary.dag`) carries no cardinality slot, no transfer functions across `map`/`filter`/`concat`. The substrate `Cardinality` connective remains production-uninhabited and v1 forks the name onto optionality (`Required \| CardOptional`) | wall after grounding |
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

**`sole_constructor` completeness audit (2026-08-20 pass, §11 item 1a, all measured by
execution — `gunbc run --claim-run` against a synthetic cross-module probe corpus compiled
live via `compile_dag_diagnostic_census`, never by reading alone):**

*Construction forms — CONFIRMED, wall fires uniformly for the two forms that reach it; a
THIRD form, variant construction of a coproduct, is a CONFIRMED HOLE.* Record-literal
construction of a cross-module sole_constructor type at every AST position tested —
return, let-binding, call-argument, list-element, nested-field-init, branch-result — and
cast construction at call-argument and list-element position, all refuse with
`SoleConstructorViolation` (discriminating RED) while an in-module (sanctioned) mint of the
same type stays clean (accepted-positive control). This is two *forms* (record literal,
cast), not six-plus independent forms — the AST position varies, the form reaching
`04_infer` `sole_constructor_construction_diags` does not (only `infer_record_lit` and the
`ExprCast` arm call it). **Confirmed by execution, not by trusting that source-comment
claim: variant construction of a `sole_constructor` COPRODUCT is unwired entirely.** Fixture
`test.fixture.sole_constructor_variant_probe.definer` declares
`type SealedChoice sole_constructor = SealedA { n: Int } | SealedB { s: String }`; probe f13
forges `SealedA { n: 999 }` cross-module. Measured: zero `SoleConstructorViolation` rows AND
zero total diagnostic rows of any class (both genuine `CensusObserved` zeros, not the `-1`
`CensusNotRunnable` sentinel `violation_count` already distinguishes) — the synthetic module
compiles completely cleanly while forging a sealed variant from outside its declaring file.
Root cause matches the two-call-site finding exactly: neither the `ExprCast` arm nor
`infer_record_lit_structural` is reached by variant-literal construction (a distinct AST
node), so `sole_constructor_construction_diags` is never invoked for this form at all. This
is a real, third construction form the wall does not cover, not a variant of the
already-known order-dependence hole.

*Deserialization / from_value — CONFIRMED NOT APPLICABLE: no such generic construction path
exists in the language. The registry read is the evidence; f14 is a consistency check on
it, not the reverse.* The load-bearing evidence is enumerating all 124 host builtins
registered in `v1_interpreter_dispatch_generated.rs`'s dispatch table: none returns a
generic/parametric `T` from untyped input (String/JSON/YAML) — every builtin's return type
is a fixed concrete shape, so there is no reflection-based or serde-style mechanism in v1
that constructs an arbitrary user-declared nominal type from external data outside of
ordinary record-literal, cast, or (per above) variant-literal expression forms. Probe f14
(calls a guessed `from_json`-shaped name against the sealed fixture type, cross-module;
measured 1 total diagnostic row, 0 `SoleConstructorViolation` rows, the one row being
name-resolution failure) does **not**, by itself, distinguish "no such path exists" from "a
path exists under some other name I didn't guess" — a single failed guess is consistent with
either. What actually carries the "not applicable" conclusion is the exhaustive registry
enumeration; f14 is only a corroborating data point (confirms the guessed name specifically
isn't the gap) on top of that closed enumeration, stated in the correct evidentiary
direction rather than the reverse.

*Emit-side reconstruction — CORRECTED 2026-08-20: this was assessed as theoretical, and a
sibling audit (ARM 3, `docs/plans/compiler-guarantee-recovery-gap-analysis.md` §11 item 1a,
executed by session lively-bee-274, gunbc PR #8661) proved it by execution on a production
type instead. Checked directly in emitted Rust: the only occurrences of `SoleConstructor`
anywhere in generated output are the compiler's own `CompilerDiagnostic` variant
(`v1_std_core.rs`) — its match arm, span accessor, and message renderer. No emitted
user-defined data type carries any confinement in the emitted target; emitted structs are
plain `pub`-field records. This clause previously read the absence as "not automatically a
defect" because nothing in ordinary self-hosting use hand-writes a violating construction
through the emitted-struct door — that reasoning stopped at the v1 seed's own hand-written
Rust (`v1_interpreter.rs`, `cli_run.rs`) and did not consider the GENERATED mirror itself as
a forgery surface. ARM 3 executed against `extdeps.uri` `UriValidatedScalar` — `sole_constructor`,
single fixed-law mint `uri_validated_scalar_construction`, refusing surrogate and
out-of-range code points — and found its emitted mirror (`extdeps_uri.rs`) is
`pub struct UriValidatedScalar { pub admitted_cp: i64 }` deriving `serde::Serialize,
Deserialize`: both a direct Rust struct literal and `serde_json::from_value` admit every
value the `.dag` mint refuses (the surrogate `55296`, `-1`, `1114112`), with a shape-control
discriminator (`serde_shape_control_rejects_malformed_input`) proving the harness can and
does observe a real `Err`, so those "admits" verdicts are load-bearing, not a silent-harness
artifact. That is a confirmed forgery mechanism on a production sealed carrier's own
generated mirror — not merely the hand-written-Rust-beside-the-mirrors surface this entry
originally scoped to. **No production caller is established to have exercised this path** —
the finding is that the mechanism exists and admits every refused value, not that any caller
has forged one; `uri_percent_encode_admitted_scalar_wire` is the type's sole declared
consumer and would receive a forged value if one reached it, but no such call site is shown
to construct one today. Per §4b's per-path rung rule (source→interpretation,
source→each-emission-target are independently ruled paths): the source→`.dag`-acceptance
path is unaffected by this finding (that wall holds); the source→Rust-emission path is
**below floor, not merely below ceiling** — a value the modeled system refuses is silently
constructible with no typed, located refusal at all, the class §5 forbids outright. The
hand-written-Rust-beside-the-generated-mirrors surface (ii) below remains a second, narrower
finding about code outside `.dag`'s type system entirely; it is not superseded, only no
longer the only finding at this row: (ii) the v1 seed carries hand-written Rust beside the
generated mirrors (`v1_interpreter.rs`, `cli_run.rs`, and others) that sits entirely outside
`.dag`'s type system and therefore outside `sole_constructor` with no mechanism even in
principle — not an unwired form of an existing check, but a construction surface the check
was never positioned to reach at all; a related door in this surface (the interpreter's
fixture-decoder reconstruction path, `recorded_fixture` `value_from_fixture_json`) is a
separate, ongoing audit (fierce-ant-91, not this one). This is recorded as its own ledger
row rather than folded into the record-literal/cast finding or the deserialization finding,
because "a form the wall doesn't cover," "a representation the wall has no jurisdiction
over," and "the wall's own generated mirror admits what it refuses" are three different
claims.

*Remaining record-literal AST positions (module-scope data initializer, map-literal value) —
CONFIRMED, wall holds, no new hole.* Two further positions beyond forms 1-8: f15 forges
`LocalValidated { n: 999 }` as a top-level `data` declaration's initializer (module scope,
outside any fn body) — refuses. f16 forges the same literal as a `Map<String,
LocalValidated>` literal's value (collection-value position, distinct from form 4's
list-element position) — refuses. Both reconfirm the two-call-site finding rather than
surfacing a new form: `infer_record_lit_structural` is reached regardless of the enclosing
declaration or collection shape, only the AST node kind (record literal vs. variant literal)
determines whether the check fires.

*A third, orthogonal mechanism (reframed, not a fourth `sole_constructor` hole) — `admit_callers`,
CONFIRMED functioning, both arms now designed and executed.* Three orthogonal questions, three
different answers: `sole_constructor` gates WHO MAY CONSTRUCT (declaring file — diagnostic
`SoleConstructorViolation`); `admit_callers` gates WHO MAY CALL the mint (named decls —
diagnostic `ConstructorCallAdmissionRefused`); the caller-supplied validator decides WHAT IS
PROVEN, and is defeasible (the validator-identity finding below). Two enforced by distinct
diagnostics, the third enforced by nothing — this is why a sealed-wrapper design needs all
three (confine construction + restrict callers + fix the validator in the declaring module):
missing any one means the other two don't cover for it. Fixture
`test.fixture.sole_constructor_sealed.definer`'s `mint_sealed_local` restricts its callers via
`admit_callers` to exactly `test.fixture.sole_constructor_sealed.admitted_caller`
`admitted_mint_call`. f17 calls `mint_sealed_local` from a synthetic, necessarily-unadmitted
probe module: refuses with `ConstructorCallAdmissionRefused` (not `SoleConstructorViolation`),
and the census's total row count is exactly 1 — that one refusal is the *only* diagnostic the
synthetic module produces, confirming a clean single-cause refusal rather than a cascade
obscuring the real class. **The green half was first claimed incidentally, then corrected to a
designed control.** The original claim — "every prior probe dispatch this session compiled the
full fixture tree including [`admitted_caller.dag`] with zero diagnostics" — was an *observed
absence of complaint*, never a *designed assertion*, and turned out false on inspection: every
f1-f17 synthetic entry imports only `definer`, and `compile_dag_diagnostic_census`'s resolver
(`resolve_virtual_source_with_imports`) pulls in only the transitive import closure of the
synthetic entry, not the whole fixture tree — so `admitted_caller.dag` was never actually
compiled by any prior probe in this session; the claim was assumed, not observed. f18 fixes
this: its synthetic entry imports `test.fixture.sole_constructor_sealed.admitted_caller`
directly and calls its exported `admitted_mint_call` — the one decl named in
`mint_sealed_local`'s `admit_callers` list — which genuinely pulls the real fixture into the
compile closure. Result, executed: census total diagnostic count = **0** across the whole
compile. That is now a designed, executed accept-side control, not an incidental observation.

*Default-value construction position — CONFIRMED HOLE, executed 2026-08-20 (f19).* A fourth
open axis named in `gunbc.roadmap_authority` (generic carriers, coproduct variants, default
values, `module_skips_direct_call_arg_check` — the first two and the fourth are addressed
above and below; this closes the third). f19 forges `LocalValidated { n: 999 }` as a function
parameter's declared default-value expression, cross-module
(`fn forged(param: LocalValidated = LocalValidated { n: 999 })`): the census's total
diagnostic row count reads **0** — the compile produces no diagnostic of any class.
`infer_record_lit_structural` is never reached for this AST position at all; this is a
position gap distinct in kind from the census-ambiguity hole (a name-resolution gap) and the
variant-construction hole (an AST-form gap) — the record-literal *form* is exactly the one the
wall otherwise covers, but this particular *position* is unwired. **Zero live exposure
today** — targeted grep across `dag/` and `src/v2/` for every declared sole_constructor type
used at a fn-parameter default-value site found no hits (not an exhaustive census
methodology, same caveat as the other two holes' exposure numbers below).

*Compiler-module check exemption (`module_skips_direct_call_arg_check`) — CONFIRMED does NOT
reach `sole_constructor`, established by code read (not execution).* Read at
`v1_compiler_infer.rs`: `module_skips_direct_call_arg_check` gates exactly one call site —
`arg_compat_diags`, the direct-call argument *type-compatibility* judgment. Neither
`sole_constructor_construction_diags` call site (the `ExprCast` arm, the record-literal-inference
arm) is wrapped in that guard or conditioned on it anywhere. This matches
`direct_call_shape_wall_note`'s own documented rationale for the exemption's scope: it exists
for the type judgment's representation-gap false-positive classes (brand aliases, optionality,
anonymous literals, expansion depth) — a label/identity check like `sole_constructor` has no
representation to have a gap in, so the exemption's reason does not reach it. This retires the
fourth roadmap axis as a **positive finding** — the exemption exists, is scoped to the argument-
type judgment, and does not create a compiler-module bypass for `sole_constructor` — rather
than "no exemption found."

**Exposure, per hole, so a confirmed defect is never read as a confirmed victim (explicit
ask, 2026-08-20):**
- *Order-dependence (census-ambiguous bare name):* sole_constructor type declarations
  corpus-wide (`dag/` + `src/v2/`, excluding this audit's own planted fixtures) = 69 names
  (fierce-ant-91's independent count-based measurement: 70/86/zero — the 1-name delta is
  immaterial, likely a `^type` regex-boundary difference such as a generic `<T>` line);
  names declared more than once anywhere in that corpus = 86; intersection today = **0**.
  Verified by two independent methods (count-based and name-list-intersection-based); the
  intersection method was itself positive-controlled by re-including this audit's own
  planted `DupShape` collision fixture, which the method correctly surfaced. **Zero live
  exposure today** — the hole requires a future sole_constructor type whose bare name
  collides with any other module-scope declaration anywhere in a consuming corpus's
  transitive closure; nothing warns an author when that PR lands.
- *Variant construction:* fierce-ant-91 independently measured **zero sole_constructor
  coproducts exist in the production corpus today** (every existing sole_constructor type
  is a plain record, `OrderedClosedInterval<T>` included). **Zero live exposure today** —
  the hole requires a future sole_constructor type declared as a coproduct; nothing warns an
  author when that PR lands either.
- *Default-value construction position:* targeted grep, every declared sole_constructor type,
  for a fn-parameter default-value use across `dag/` + `src/v2/` — **0** hits. **Zero live
  exposure today** — the hole requires a future sole_constructor type used as a parameter's
  declared default; nothing warns an author when that PR lands either.
- *Deserialization:* not applicable — no such construction path exists in the language at
  all (registry read is the evidence; f14 a corroborating check, not the proof), so this has
  no exposure dimension; it is closed, not open-with-zero-exposure.
- *Emit-side reconstruction — TWO findings at this row, not one, per the §10 correction above.*
  (a) The generated mirror itself: CONFIRMED BY EXECUTION (ARM 3, PR #8661) that a production
  sole_constructor type's own emitted struct (`extdeps.uri` `UriValidatedScalar`) admits, via
  `serde_json::from_value` and via a direct struct literal, every value its fixed-law `.dag`
  mint refuses. This is not measured the same way as the three within-`.dag` holes above (no
  grep-for-a-shape count applies — the construction path is the ordinary emitted API surface
  itself, not a rare AST position) and **no production caller is established to have exercised
  it** — say plainly: mechanism confirmed, no confirmed victim. Below floor (silent), not
  merely below ceiling, on the source→Rust-emission path specifically — the source→`.dag`
  path is unaffected. (b) Hand-written Rust beside the generated mirrors: not a hole in an
  existing wall, a construction surface the wall was never positioned to reach at all
  (`v1_interpreter.rs`, `cli_run.rs`); the interpreter's fixture-decoder reconstruction door in
  this surface is a separate, ongoing audit, not this one's. Recorded as its own §10 row,
  not merged with the three within-`.dag` holes above.
- *Validator-identity forgeability (f12/f12b, executed — `always_true_le` accepts `low: 10,
  high: 1` via `closed_interval`, `IntervalReady`; honest `le` control correctly refuses via
  `IntervalRefused`) — CATEGORICALLY DIFFERENT EXPOSURE SHAPE, do not flatten alongside the
  three above.* The other three holes require an author to write an unusual, not-yet-existing
  declaration (a sole_constructor coproduct, a colliding bare name) before the gap is live.
  This one requires nothing new: `closed_interval`'s signature already accepts an arbitrary
  caller-supplied predicate at all 4 of its production call sites today (fierce-ant-91's
  count: `millimeter_le` x1, `nanosecond_le` x3, all honest — so zero exploitation today), and
  the trigger is "pass a subtly-wrong comparator to an existing API," not "author a new
  shape." It is also not, strictly, a `sole_constructor` completeness hole at all:
  `sole_constructor` confines *who/where* constructs the carrier and does so correctly here;
  this finding is that confinement alone says nothing about *what invariant* the confined
  mint's caller-supplied predicate actually enforces. Load-bearing for the `Refined<B>`
  roadmap design: a sealed wrapper's safety requires the validator to be **fixed by the
  declaring module**, not caller-supplied — confinement without a fixed validator provides no
  invariant guarantee, and `closed_interval` is the in-tree specimen proving it.

**The pattern across every hole, stated as the headline finding rather than left implicit:**
`sole_constructor` covers exactly the construction shapes the corpus happens to use today
(plain records, census-unique names); its boundary was undocumented until this pass; and its
current safety is a property of the corpus's current contents, not a property of the wall
itself. Every confirmed hole here has zero live victims today, and every one of them is
exactly one ordinary PR away from becoming real — a sole_constructor type declared as a
coproduct, or given a bare name that collides with anything else in scope — with no
diagnostic, warning, or review signal marking that PR as the one that lands in the gap. This
is neither "sole_constructor is broken" (nothing accepted today is wrong) nor "sole_constructor
holds" (its guarantee is narrower than its name claims) — it is a wall whose current
soundness is corpus-contingent, not structural.

*Generic carriers — CONFIRMED for the two forms above, on a parameterized carrier.* A
cross-module record literal and a cross-module cast of `OrderedClosedInterval<T>`
(`std.interval`) each refuse at a first type argument (`<Int>`) and at a second, distinct
type argument (`<ProbeMarker>`) — both instantiations independently flagged when forged in
the same probe module. Report this precisely as "fires for a parameterized carrier on
cross-module record-literal and cross-module cast construction" — not as "generic carriers
are covered" in general; no other construction route on a generic carrier was tested.

*Order-dependence — CONFIRMED HOLE, root-caused and reproduced by exact integer count, not
inferred from a Bool.* Two fixture modules each declare a bare `DupShape` — one
`sole_constructor`, one not — making the name census-ambiguous. `04_infer`
`type_has_sole_constructor` resolves via `04_infer` `lookup_type_by_name` with no ambiguity
guard, unlike the sibling `presence_check_census_gate_note` gate (same file), which stands
down on ambiguity via a local-declares-first carve-out specifically to avoid this outcome.
Measured violation counts (expected in parens):
- probe explicitly `import sealed_variant { DupShape }` first, `import open_variant`
  (unqualified) second, forges `DupShape`: **0** violations (expected ≥1) — the sealed
  type's own construction is silently missed.
- probe explicitly `import open_variant { DupShape }` first, `import sealed_variant`
  (unqualified) second, forges `DupShape`: **1** violation (expected 0) — the open type's
  legitimate construction is silently flagged.
- neither import named-selects `DupShape`; sealed-then-open plain-import order: **0**
  violations. Open-then-sealed order: **1** violation.
All four counts are exactly consistent with a single mechanism: resolution always returns
whichever declaration was imported **last** (`direct_import_export_precedence_note`'s
"later import wins" overlay), independent of which declaration the probe module's own
`import … { Name }` clause named. This is not merely "order-dependent" in the abstract —
it is a silent MIS-RESOLUTION, not a stand-down: unlike the sibling presence-check gate
(which refuses to guess and skips enforcement on ambiguity), `sole_constructor`'s check does
guess, guesses by textual import order rather than by the caller's actual reference, and
reports zero diagnostics either way it guesses wrong. An author whose module explicitly
imports the sealed type by name can still silently forge it if anything else in the
transitive import closure also declares a same-named type and is imported later.

*Absent resolution (`type_has_sole_constructor`'s `None => false` arm) — investigated
separately per instruction, NOT an independent hole for the record-literal path.* Reading
`04_infer` `infer_record_lit_structural`: when the type name does not resolve at all
(`effective_lookup == None`), a **separate**, unconditional `bare_name_miss_diagnostic` fires
regardless of `sole_ctor_diags` — so a record literal naming a wholly nonexistent type is
already refused on a different, always-firing ground, and the `Absent => false` arm never
gets a chance to silently admit anything on this path. The cast path's equivalent guarantee
was NOT traced to the same certainty (`validate_cast` checks cast-domain compatibility, not
general nominal-type existence, and whether some earlier type-expr-resolution pass
independently refuses an unresolvable cast target was not confirmed this pass) — left as an
explicit open sub-question, not asserted either way.

*Compiler-module exemptions — CONFIRMED: none apply to `sole_constructor`.* The only
compiler-module exemption found anywhere in the v1 pipeline, `04_infer`
`module_skips_direct_call_arg_check` (the `v2.`-prefix carve-out), is confirmed by full
read to be scoped to the direct-call-argument type-conformance wall only; neither
`type_has_sole_constructor` nor `sole_constructor_construction_diags` references it or any
other module-path exemption. `v2.compiler.normalized_tree` itself declares a
`sole_constructor` type and is enforced identically to any other module.

*Interpreter/runtime bypass — CONFIRMED: none found.* `v1_interpreter.rs`'s cast evaluation
(`cast_identity_result`, `eval_cast`) is identity/passthrough with no re-check, but this is
moot: `gunbc run` refuses evaluation entirely when the entry's transitive-import closure
carries a blocking diagnostic, so a violating program never reaches execution. No
reflection/deserialization builtin capable of synthesizing an arbitrary user-defined nominal
record was found in `coproduct_reflection.rs`.

*Validator-identity forgeability (addendum, confirmed by execution, orthogonal to
`sole_constructor` itself) — a real completeness gap in the roadmap's planned mitigation,
not in `sole_constructor` as scoped.* `std.interval` `closed_interval` and
`v2.std.refinement` `refine` both accept a caller-supplied predicate
(`le: fn(T,T)->Bool` / `Validation<B>.admits`). Executed: `closed_interval(low: 10, high: 1,
le: always_true_le)` — a deliberately-broken caller-supplied predicate — returns
`IntervalReady` (accepted) despite `low > high`; the same call with an honest predicate
correctly returns `IntervalRefused` (control, confirms the harness measures what it claims).
Even a fully `sole_constructor`-sealed `Refined<B>` would not close this: `sole_constructor`
confines WHO/WHERE constructs the carrier, never WHAT predicate a sanctioned caller supplies
through the carrier's own accepted mint path. This is a distinct invariant from the one
`sole_constructor` completeness is scoped to answer, and the roadmap's `Refined<B>` design
should treat it as a separate open question rather than something the construction wall
retires.

**Falsified precedent, called out as its own headline point (2026-08-20).** `OrderedClosedInterval`
has been cited repeatedly — by this audit's own coordinator, by the `Refined<B>` design lane,
and by an outside advisor — as "the working structural precedent: a generic sole_constructor
carrier whose sole mint refuses the invalid reversed case." **That claim is false as stated.**
f12 (executed, above) shows the mint only refuses when the caller supplies an honest
comparator; a lying one is accepted with zero diagnostic. A falsified reassurance that was
propagating unchecked across three parties is worth surfacing as its own finding, distinct
from "a new hole was found" — it means a design decision was resting on a citation nobody had
executed.

**Centerpiece: `dag/std/interval.dag` contains its own right-shape/wrong-shape A/B for the
SAME sealed carrier, ~16 lines apart, same author, same module — production code, no
synthetic fixture needed.**
- `closed_interval<T>(low, high, le: fn(T,T)->Bool)` (line 32) — the invariant is delegated
  to a caller-supplied predicate. Confined (sole_constructor correctly walls off who/where)
  but **unguaranteed** — f12 breaks it by supplying a dishonest `le`.
- `degenerate_interval<T>(point: T)` (line 48) — **total**, no predicate, no failure arm,
  returns the carrier unconditionally. `[x, x]` is ordered by reflexivity, so there is
  nothing to check and nothing for a caller to lie about — the invariant is established by
  construction, not by a delegated check that can be defeated.

This is §4b's *structurally guaranteed* rung sitting directly beside its *mechanically
preventable-at-best* rung, in one file, for one carrier — and nothing in either function's
**type** distinguishes them for a reader deciding which to call or which pattern to copy for
a new carrier. Both return `OrderedClosedInterval<T>`; only reading the body reveals which
rung each constructor actually occupies.

*Generic-carrier re-confirmation (f11/f11b, re-executed via `--claim-run` this pass):*
PASS `f11_generic_carrier_second_type_arg_refuses`, PASS
`f11b_generic_carrier_both_type_args_each_refuse` — record-literal and cast construction of
`OrderedClosedInterval<T>` each refuse independently at a first (`<Int>`) and a second,
distinct (`<ProbeMarker>`) type argument, both instantiations flagged when forged in the same
probe module. Confirms generic-carrier coverage is real (for these two forms) and is not an
artifact of only ever having exercised `<Int>`.

*Overall verdict — REVISED 2026-08-20 to fold in ARM 3 (PR #8661): this changes the
conclusion, not merely extends it. The audit's prior verdict was, without saying so, scoped
to the source→`.dag`-acceptance path only; per §4b's own per-path rung rule the
source→Rust-emission path is a distinct, independently-ruled path, and it is now confirmed
BELOW FLOOR — silently forgeable on the emitted mirror of a production sealed type — which is
categorically worse than any of the three within-`.dag` holes below (all three are decidable,
zero-exposure, corpus-contingent gaps in an otherwise-real wall; the emission finding is the
wall's complete, silent absence on an entire realization target).** `sole_constructor`
reliably walls off record-literal and cast construction (including generic instantiation) of a
census-UNIQUE, plain-record type, at every tested AST position (fn body, module-scope `data`
initializer, list-element, map-value) and via the interpreter, with no compiler-module
exemption reaching it (`module_skips_direct_call_arg_check` exists and is scoped to the
argument-type judgment only — confirmed by code read, not execution). **On the
source→`.dag`-acceptance path specifically**, it does NOT reliably wall off a census-AMBIGUOUS
type name — resolution silently guesses by last-import-wins rather than refusing or consulting
the caller's actual selection, the `presence_check_census_gate_note` precedent's exact failure
mode, landed here without the stand-down that gate uses to avoid it (0 live exposure today,
corpus-wide census 69/86/0); it does NOT reach variant construction of a sole_constructor
coproduct at all — a distinct, entirely unwired AST form (0 live exposure today — zero
sole_constructor coproducts exist in the corpus); and it does NOT reach a record literal in a
parameter's declared default-value expression — a distinct position gap, executed via f19 (0
live exposure today — targeted grep, no fn-parameter default-value use of any declared
sole_constructor type found). **On the source→Rust-emission path**, it does not exist at all
for a production sole_constructor type's own emitted mirror: ARM 3 (§10) confirmed by
execution that `extdeps.uri` `UriValidatedScalar`'s emitted struct admits, via
`serde_json::from_value` and a direct struct literal, every value its fixed-law mint refuses —
no production caller is established to have exercised this, but the mechanism is real and the
gap is silent (below floor), not a decidable zero-exposure hole among otherwise-sound coverage.
**Two confirmed scope boundaries, complementary rather than duplicative — the SAME underlying
claim (confinement is not an invariant guarantee) proven from opposite directions**:
validator-identity (f12/f12b) shows a PERMISSIVE, caller-supplied validator can lie even when
sole_constructor's own confinement holds perfectly (`closed_interval` accepts a reversed-bounds
value under a lying comparator); the emission gap shows a FIXED, module-owned validator
(`uri_validated_scalar_construction`'s law is not caller-supplied) is still worthless once the
carrier crosses into a realization where confinement itself does not survive. Neither is a
defect in `sole_constructor` as specified — the mechanism does exactly what a construction-site
gate can do (source→`.dag` confinement) and no more; both show sealing requires confinement AND
a fixed validator AND confinement surviving every realization target, and any one absent
voids the guarantee regardless of the other two holding. Recommended next-rung triggers,
kept as three SEPARATE follow-ups needing their own design decisions, not one bundled repair:
(1) apply the same local-declares-first carve-out `presence_check_census_gate_note` documents
(read `str_bindings` — local declarations only — before falling through to the
import-order-overlaid `lookup_type_by_name`) for the ambiguity hole — an exact
declaration-identity fix replacing the bare-string lookup; (2) add a variant-construction call
site alongside the existing `ExprCast`/`infer_record_lit_structural` sites for the coproduct
hole — joining variant construction to the same authority as the other two forms; (3) the
emission/reconstruction door (omit `Deserialize`, emit a validating one, or seal the field
behind `TryFrom`) for the Rust-emission gap. These three are independent because they sit on
different paths (two within `.dag`-acceptance, one at emission) and touch different
mechanisms (a lookup carve-out, a new AST-form call site, an emitter/derive-roster change);
bundling them would conflate a decidable zero-exposure completeness fix with a below-floor
production-realization repair that needs its own design review. All three are decidable fixes
once scoped, not ratchets — the default-value position hole (executed via f19) is folded into
trigger (1)'s sibling scope, not a fourth separate trigger, since it shares the same
`infer_record_lit_structural`/`ExprCast` call-site mechanism as (2). Until landed, any
`sole_constructor` carrier (existing or a planned `Refined<B>`) that is later declared as a
coproduct, whose bare name later collides with another module-scope declaration anywhere in a
consuming corpus's transitive closure, is later used as a parameter's declared default, or is
emitted to Rust at all, silently loses some or all of its guarantee with no diagnostic marking
the PR that introduces or exposes the gap — the wall's soundness today is a fact about what the
corpus currently contains and which realization target is in play, not a fact the wall itself
enforces end to end.

## 11. Audit queue

1. ~~Recover `docs/error-examples.md`~~ **DONE — see §8b**; ~~`correctness-dimensions`~~
   **DONE — see §8c.** Still to pull: `what-falls-out`, `two-groundings`,
   `the-derived-homomorphism`.
1a. ~~Audit `sole_constructor` completeness~~ **MAPPED, not closed — see §10, 2026-08-20 pass
   plus the 2026-08-20 ARM 3 fold-in (PR #8661).** The source→`.dag`-acceptance path is
   CONFIRMED BY EXECUTION: the wall holds uniformly across every literal/cast AST position
   tested, concrete and generic, on that path specifically. CONFIRMED HOLE BY EXECUTION: the check's own resolution
   (`04_infer` `type_has_sole_constructor` → `04_infer` `lookup_type_by_name`) inherits full
   import-order dependence on a census-ambiguous bare name — it does not stand down the way
   the sibling `presence_check_census_gate_note` gate does, so it silently MIS-JUDGES rather
   than merely stands down: a sole_constructor type's own construction can be silently missed,
   and an open type's legitimate construction can be silently flagged, purely as a function of
   which of two same-named declarations was imported last in the probe module — independent of
   which one the author's own `import … { Name }` explicitly selected. `Refined<B>` is NOT
   cleared for a blanket `sole_constructor` landing until this hole is closed (a local-declares-
   first carve-out, mirroring the sibling gate, is the indicated fix) or the roadmap accepts the
   residual risk explicitly. Separately CONFIRMED BY EXECUTION: `std.interval` `closed_interval`
   and `v2.std.refinement` `refine` both accept a caller-supplied validator predicate,
   so `sole_constructor` alone — even fully applied — cannot make a refinement carrier's
   accepted values honor their nominal invariant; the caller can supply a validator that always
   admits. Separately CONFIRMED HOLE BY EXECUTION: variant construction of a
   `sole_constructor` coproduct is entirely unwired — a distinct third AST form from the
   record-literal/cast pair, never reaching `sole_constructor_construction_diags` at all.
   Both holes carry zero live exposure today (corpus-wide census, §10) — no existing
   sole_constructor type is a coproduct, and no sole_constructor bare name collides with
   another declaration today — but neither is a structural property of the wall: either
   condition is one ordinary PR away, with no diagnostic marking that PR. SEPARATELY CONFIRMED
   BY EXECUTION, on the source→Rust-emission path (ARM 3, session lively-bee-274, PR #8661): a
   production `sole_constructor` type's own emitted mirror is silently forgeable. `extdeps.uri`
   `UriValidatedScalar` (fixed-law mint, no caller-supplied validator) is emitted as a `pub`
   struct with a `pub` field deriving `serde::Deserialize`; both a direct struct literal and
   `serde_json::from_value` admit ONE REPRESENTATIVE FROM EACH of the mint's three refusal partitions —
   `55296` (surrogate), `-1` (negative), `1114112` (above the Unicode maximum). NOT every value it refuses: the
   mint refuses whole infinite RANGES, and a finite receipt cannot discharge a universal over them, so three
   representative points are the evidence actually held (an earlier revision of this row said "every value the
   `.dag` mint refuses"). With a shape-control
   discriminator proving the harness's `Err` path is real (so the "admits" verdicts are
   load-bearing, not a silent-harness artifact). No production caller is established to have
   exercised this path — mechanism confirmed, no confirmed victim — but it is below floor
   (silent), not a decidable zero-exposure hole in an otherwise-sound wall the way the two
   above are: the source→`.dag` path and the source→Rust-emission path are independently ruled
   per §4b, and the wall is simply absent on the latter for this carrier. This is the complement
   of the validator-identity finding below (a permissive caller-supplied validator defeats a
   perfectly-confined carrier; a fixed module-owned validator is defeated by confinement not
   surviving emission) — neither a defect in `sole_constructor` as specified, both proof that
   confinement alone is not a sealing guarantee. Full form-by-form table, generic-carrier
   verdict, exposure ledger, and open sub-questions in §10.
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
   recoverable."* **DENOMINATOR ESTABLISHED, AND ITS SUBJECT NAMED (resolved 2026-08-20).** The
   truncation concern below was raised and is now **withdrawn**: exit status was captured on ten
   completed compiles across five dispatches, `exit=1` throughout — a completed compile refusing on
   blocking diagnostics — with `137` (SIGKILL) and `124` (the `timeout 800` guard) excluded on
   every arm, and no pipe in the capture, so `$?` is the process's own status. Four separate builds
   on four separate runners produced identical ten-pair distributions; a truncation would have to
   stop at the same point four times on four machines. **THE SUBJECT MATTERS AS MUCH AS THE EXIT
   CODE AND IS STATED HERE SO NO READER INFERS IT:** these figures are **entry-scoped** — one
   import closure resolved through the `run` verb — NOT corpus-wide. A corpus-wide grep over the
   same class yields a different and larger candidate population, and comparing the two produces an
   apparent contradiction that is purely a denominator. That is the dated-measurement trap on the
   SPACE axis rather than the time axis, and it cost an hour here before it was recognised. Every
   figure in this row carries both its revision and its subject for that reason. **The original
   provisional flag, preserved because the rule it produced outlives it:** The post-repair figures below come from one run whose COMPILE-step
   exit status has not been printed. A sibling whole-root compile was `Killed` at EXIT=137 on the
   same infrastructure while its dispatch still reported 0, because redirect-then-echo does not
   propagate status — and *a process killed partway through a corpus compile does not produce
   zero, it produces a truncated population indistinguishable from a complete one.* The specific
   reason for suspicion: 46 of 46 recovered with residue EXACTLY zero, on a population three lanes
   independently predicted would be non-zero, is as consistent with a run that stopped before
   reaching the residue-bearing files as with a clean result. **Standing rule adopted from this:
   any command reporting a count must print the exit status OF THE PROCESS THAT PRODUCED THE
   COUNT, beside the count — not the dispatch's status.** What survives regardless is
   source-derived and decidable; what does not survive is the denominator. **THE PARTITION SUMS TO 51, NOT 52, AND THE UNACCOUNTED ROW IS THE ONE THAT CANNOT
   BE REDISCOVERED.** 46 caret-literal + 3 `sym` + 2 `boundary_tag` = 51 against 52 measured in
   that file. The missing row is almost certainly the DEGENERATE-SPAN one — the earlier census
   recorded 22 precisely located plus 4 module-only, one of which was this module. That is not an
   ordinary off-by-one: **the missing row is precisely the row with no span identity**, findable
   only by enclosing fn, so if the partition is acted on as covering all 52 it is dropped silently
   and permanently rather than resurfacing later. Supporting arithmetic (not a measurement): the 5
   Symbol rows match the symbol-keyed list exactly — `canonical_hash_of_connective`,
   `canonical_hash_of_edge_label`, `symbol_identity_digest`,
   `byte_offset_cache_digest_ineligible_hash`, `byte_offset_cache_key_fingerprint` — and 46 + 5 + 1
   = 52. **NARROWED, AND THE NARROWING IS SHARPER THAN THE GAP:** `content_hash_atom(` occurs
   exactly 51 times in that file (verified at `origin/main`), so the partition is COMPLETE over
   `content_hash_atom` call sites and the 52nd row is **not a missed member — it is a DIFFERENT
   REFINED POSITION.** The candidate is in the same file and visible:
   `content_hash_combine_structural(` at 5 sites and `combine_hash(` at 53. Different function,
   different formal, so it could never appear in a `content_hash_atom` census however carefully
   that census was run — which is why an off-by-one here is a category gap rather than an arithmetic
   one. **STANDING OBLIGATION, recorded in these terms deliberately:** whatever the regen retires,
   the degenerate-span row is carried as its own NAMED obligation keyed by ENCLOSING FN, never as a
   residual of a count. A row that can only be named one way gets named that way once, while
   someone still knows it exists. **What would settle the composition outright is a per-file ×
   per-type-pair CROSS-TAB**, which no one holds: the two dimensions were captured as separate
   aggregates and never joined. **Two aggregates side by side are not a cross-tab, and the gap
   between them is exactly where a row can live undetected** — the fourth instance of this axis
   error in one investigation, and the first committed by the measuring instrument rather than by a
   reader of it. No rerun is dispatched for it: the population closes on regen and the one row that
   does not close is now named. **A SECOND
   CORRECTION ALREADY LANDED ON THE FRAMING BELOW:** a source partition of the 52 shows 46 of them are
   `content_hash_atom(value: ^caret literal)` — the #8608 class exactly, *already fixed in
   authority* and inert only because that fix is unmirrored. So the file is not the unit of repair;
   **the regen is** — one change retires 46 rows across every file at once. Both the corpus-sweep
   framing and the one-file framing were wrong for the same reason: **both count DIAGNOSTICS, which
   are distributed by where values FLOW, not by where the defect LIVES.** Counting ROOTS makes it
   one landed fix plus one named capability. Same shape as the type-pair axis error one level up,
   and both times the misleading grouping was the one that looked most like a natural unit of work.

   **MEASURED AFTER THE REPAIR (`swift-moth-294`, four-arm pinned run; arms built by
   `git checkout -B <name> <sha>` so no branch-merge could pin their mirrors): THE 46 WERE ONE
   FILE.** At main tip the synthetic count is **zero** and all 72 rows are located, with every
   previously-unlocated row resolving to `src/v2/std/node.dag` — 52 rows there, 6 already located
   plus 46 recovered. So the population described as *"46-72 refinement diagnostics across the
   corpus"* — #8607's own commit message — was never corpus-wide. **The author of the fix made the
   same misreading, from the same broken artifact:** the constructor that destroyed the file field
   hid the CONCENTRATION as well as the location, and it misled the one person who understood the
   mechanism well enough to repair it. That is the defect's last damage on its way out, and it is
   why the remaining repair is one file and 52 sites rather than the corpus sweep the work was
   being scoped as. The attribution is verified rather than assumed — `module` and `file` fields
   populated by different paths agreeing on all 52; 52 DISTINCT `(start,end)` pairs, so not one
   site counted 52 times; offsets 22970–51589 against a 51825-byte file, in range with the maximum
   just under the size; and spacing sequential and tight (23032, 23082, 23140, 23199), consistent
   with consecutive declarations rather than a fabricated constant. **THE OPERATIVE STATE OF THIS CLASS AS OF #8607: THE REPORTER IS BLIND AND THE CLASS
   IS AT 72.** Combine the measurement above (main tip: 72 constructed, **zero** `<synthetic>`) with the
   filter rule this row establishes (the reporter shows *exactly* the rows whose file field is
   `<synthetic>` and hides every row that carries one) and the consequence is that **the reporter now
   prints zero rows while all 72 mismatches still exist.** Nothing was repaired. The class went silent
   because the constructor stopped destroying the file field, and the reporter only ever displayed what
   the constructor had broken — **a fail-open wearing the appearance of a fix**, and one that would
   otherwise have been discovered months later as a class everyone believed closed. The 72-constructed /
   0-synthetic figure is MEASURED; the unpatched reporter's output is INFERRED from the filter rule, and a
   confirming run reading ordinary diagnostic output with the instrument OFF is cheap and still owed — if
   it prints refinement rows, the filter rule is wrong. **Standing consequence, binding rather than
   precautionary: a quiet reporter is not a closed class.** This is also why the row's `Next trigger`
   below is not satisfied by #8607: locating the diagnostics moved them from *reported* to *censored*
   without changing how many exist, so the trigger — report an unlocated diagnostic AS unlocated instead
   of reporting only those — is now the difference between a silent class and a visible one.
   **#8607 changed LOCATEDNESS ONLY** — the ten type pairs are identical count-for-count across all four arms, so it recovered
   no diagnostics and suppressed none. Combined with the pin/`#8579`/main arms: all 72 rows are
   pre-existing and nothing in the `#8579`→`#8592`→`#8607` sequence introduced or removed one.
   **A prediction of a non-zero residue, registered as falsifiable beforehand, was refuted** — the
   reasoning (40-odd unconverted callers, ten in `04_types`) was sound for a corpus-wide population
   and the population was never corpus-wide. **Consequence for every measurement in this row:** the
   72/46/26 split is a property of the corpus *before* that merge, not of the corpus. Post-#8607 runs should show substantially
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
   constructor rather than a discouraged one. **The residue is now sized, and it is denominated in the MIRROR, not the
   authority — the running binary executes the mirror.** At `origin/main`: authority
   `src/v1/*.dag` carries 56 `make_span` occurrences (58 before #8607; the delta of 2 is that
   fix, so 56 is already post-fix and subtracting again double-charges it); the GENERATED mirror
   carries 57. **TWO OF THE THREE LEGS THAT ONCE SUPPORTED THIS ARE RETRACTED, INCLUDING THE ONE
   THIS DOCUMENT CALLED STRONGEST.** An aggregate `make_span` count and a per-file join of each
   generated mirror to the authority its own line 2 declares (six of seven pairs zero, the whole
   delta in `v1_compiler_parse.rs`) were both cited here as independent corroboration. **Neither is
   evidence: a symbol-count difference across the emission boundary is not a drift test**, because
   emission is not obligated to preserve occurrence counts, so a delta cannot be separated from
   drift by counting alone. Seven rows of an unsound test is one unsound test. **A first attempt to
   explain the delta AS emission arithmetic is itself withdrawn, and the correction matters:**
   decomposing the regenerated file gives 3 = 1 `pub use` import + 2 calls, against the authority's
   2 call sites — **so emission IS 1:1 for this symbol**, and the committed-to-regenerated gap is
   not arithmetic at all, it is exactly the missing `call_span` site. The join's conclusion was
   therefore ACCIDENTALLY RIGHT, which does not restore the method: an unsound test that happens to
   agree with a sound one is still unsound, and the distinction has to be held or the method returns
   the next time it agrees. **And the digit itself was an occurrence count masquerading as a call
   count** — `grep -c 'make_file_span'` on the committed mirror returns 2, of which line 72 is the
   `pub use` import and line 11623 is THE ONLY CALL. Committed call sites: 1. Authority: 2. The
   correction that briefly reported 2-vs-2 would have restored the appearance of doneness on the
   one file where doneness is the illusion — worse than the error it corrected. **Standing clause:
   count CALL SITES (`foo(`), never string occurrences, and PRINT THE MATCHING LINES so a reader can
   see what was counted.** **What survives is the METHOD, not the inference:** pairing a mirror to the
   authority its own header declares — rather than by filename or by aggregate — remains the
   correct way to identify the pair, and the third clause of that rule (classify the residue that
   matched no form; a member can be *authority contradicted elsewhere*, not merely undeclared)
   stands. **THE FINDING ITSELF IS UNAFFECTED, because it never rested on the counts.** Its two
   sound legs are a SITE-LEVEL observation — `call_span` still calling `make_span` at a named site
   in the committed mirror, immune to emission arithmetic — and `required-regen` refusing on that
   file independently, which is the only sound form of the test: committed versus REGENERATED, same
   representation, same emitter, one variable. Recorded at length because the retracted legs were
   the most-cited artifact in the investigation, and a reader meeting the finding later would
   otherwise inherit the unsound support along with the sound. **A hand-file bucket was proposed and WITHDRAWN**: 22 further
   `make_span` sites live in non-generated files, but 18 are `make_span(0, 0)` — a null span has
   no file to lose, so the fabricated-plausible-location harm does not apply — and the rest are
   the test asserting the distinction (one named `make_file_span_distinct_from_make_span`). Its
   repair obligation is approximately zero. **What that withdrawal leaves is a binding design
   constraint on the climb, not a residue:** those 18 callers want a NULL-SPAN constructor and
   reach for the file-losing one because it is the one that takes two arguments. So the
   unwritability change must ship a null-span sibling in the same diff, or 18 call sites have
   nowhere to go and the refactor is unmergeable on contact. Every count here carries the revision
   it was measured at, because a count adjusted for a change already contained in the tree it was
   measured on is indistinguishable from a correct one. **Recorded because it is a routing fact and not only a technical one:**
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
   content-addressed identity question, not a comment. **Method note — attribution by mechanism from a confounded arm.** The measurement
   that established #8607's effect contained BOTH #8607 and #8608, so on its face it could not
   separate them. It separated them without a rerun: #8608 touched ZERO stage0 mirror files, the
   diagnostics are constructed in code compiled into the binary, and **an authority-only change
   cannot alter the behaviour of a binary built from an unchanged mirror.** The confound was
   dissolved by a property of the artifacts rather than by another arm. The same run then
   confirmed it by execution — the 5 Symbol rows #8608 targets are identical across arms that do
   and do not contain it — so the fix is demonstrably inert *in the binary* while genuinely landed
   *in the authority*: two true statements about different artifacts, which is the per-module
   authority-state rule arriving as a number rather than a principle. Recorded as a technique
   because the alternative was rerunning a four-arm measurement to break a confound that the
   file list already answered. A companion control: the arms' commit-presence matrix is a monotone
   staircase (lower-triangular), which cannot arise if a reset landed on the wrong tree —
   retro-validating arm construction from data already collected. **Next trigger:** any consumer at
   all reads the declaration — the nearest is the regen convergence model's observe step, whose
   `MembershipPlan` reports this file as `MemberChanged` by construction. **Note the interaction,
   because the remedy erases its own evidence:** convergence repairs the drift and destroys the
   only trace that a generated file was ever hand-authored, so it closes the instance and leaves
   the class exactly where it is.

18. **A where-refinement advisory computes its expected-type-at-position independently of
   the method-arg contract #8592 corrected** (measured 2026-08-20, still-pike-216, while
   landing #8592's permanent RED — see #8625). `WhereRefinementUnenforced` (the coproduct
   variant in `src/v1/00_core.dag`) is produced by a second diagnostic-generation pass,
   `where_refinement_unenforced_error` in `src/v1/04_infer.dag`, running independently of
   `declared_arg_types_for_method` (`src/v1/04_lookup.dag`) and `infer_method_args_with_fold`
   (`src/v1/04_infer.dag`) — the pair #8592 corrected so each method argument infers against
   its declared parameter contract rather than the receiver's element type. The two live in
   different modules and answer the same question — "what type/refinement is expected at this
   argument position" — with no shared authority, which is exactly why nothing forces them
   to agree: a DESIGN §3 duplicate-authority violation, not a standalone false-positive
   nuisance. Evidence, by direct execution on a single binary (not a FIXED/BASELINE
   differential): the fixture `probe(items: List<NonEmptyStr>, ...) { items.get(n: 0) }`
   compiles clean of blocking errors but emits an advisory —
   `where-refinement unenforced: predicate ... on 'Product(NonEmptyStr)' ... non-literal
   value at refined position` — pointing at the `0` literal, even though the emitted Rust
   (`items.clone().get((0) as usize).cloned()`) is correct: `0` is properly cast to `usize`
   per `get`'s declared Int contract. Nothing wrong propagates; the advisory is spurious on
   this fixture. **Scope, stated precisely:** this establishes the advisory is spurious on
   the fixture actually run; it does not establish a corpus-wide population. A search of
   `dag/` and `src/v2/` for literal-index `List<T>.get` call sites outside test fixtures
   returned zero matches (2026-08-20), so no live-cost population is claimed here — the
   defect class is real and reachable (any literal-index `get` call reaches it), but its
   current corpus incidence is zero, not "every caller." Ceiling: decidable and groundable
   once `where_refinement_unenforced_error` and `declared_arg_types_for_method` share an
   authority for "expected type/refinement at an argument position" — a *wall after
   grounding* (§5), not yet a wall now. No rung is claimed; per the "file the rows, omit the
   rung" ruling (#8604), no `rung found at:` field is authored here. Filed separately from
   #8604 (operator/session ruling, smart-ram-730, 2026-08-20: #8604 was at a settled,
   5-approval head and this row's evidence was executed by a different session, so it
   travels with the person who ran it) and cross-referenced from #8604's closing paragraph
   as a fourth member of that diagnostic-channel family.

19. **`explicit_witness_admission`'s `known_red_probe` is an inert lens against the required
   floor — rung `mitigatable`** (measured 2026-08-20, gunbc#8625/#8627). Traced
   `explicit_witness_admission_pairs()`'s one consumption site in `cli_run.rs`
   (`deferred_discovery_rows`, ~line 18861): it feeds only a diagnostic `DeferredDiscoveryRow`
   receipt. `v2.workflow.required_floor`'s discovery-exclusion inputs are `long_home_prefixes()`
   and `ReadsLiveTree` alone; `known_red_held` — the required floor's actual known-red
   pass/fail counter — is driven solely by `v2.workflow.floor_expected_red.floor_expected_red_roster`
   (`cli_run.rs` ~line 40514–40575, single write site ~line 41086). Neither reads
   `explicit_witness_admission` at all. `known_red_probe`'s `expected: ExpectAssertionFalse`
   field is asserted in `.dag` data with zero read sites in `cli_run.rs` outside test code, and
   its `QuarantineProbeExpectRed` cadence tag names the falsifier as intended consumer — deleted
   in the 2026-08-15 floor cut (see "Building & checks" in DESIGN.md), so the row's own
   documented consumer no longer exists. **The honest present-tense description of a
   `known_red_probe` row today is documentation, not a hold**: it records that a witness is
   expected red and why, readable by a human or a future consumer, but nothing in the required
   floor's pass/fail path reads it. It is coverage by illusion in the exact §6 shape — the
   machinery exists, nothing gates on it, and its presence reads as real coverage to anyone who
   greps for the row, worse here because the consumer it names was actively deleted rather than
   merely never built. **Ceiling and trigger:** decidable and grounded once a consumer is
   authored to join `known_red_probe` rows against required-floor discovery/hold decisions (or
   the row is re-scoped to state plainly that it is documentation); until then this sits at
   `mitigatable`, i.e. review diligence must independently notice a `known_red_probe` row is not
   protection, exactly as `v2.workflow.floor_expected_red.floor_expected_red_roster` is. **Not
   a call to build that consumer now** — gunbc#8625/#8627's actual known-red hold was
   discharged by enrolling in `floor_expected_red_roster` directly (operator ruling,
   deep-ant-102, 2026-08-20: no new mechanism), which is the one live authority for this case.

20. **Reconstruction doors, umbrella.** Two interpreter reconstruction mechanisms admit serialized
   observations without completing semantic acceptance: fixture replay reconstructs NOMINALLY TAGGED records or
   variants without declaration admission, while REST JSON projection has access to the resolved return
   declaration but consumes it to varying degrees by arm — ordinary object projection derives all declared field
   names, the array arm uses only the first, and the unresolved and childless arms bypass field projection
   entirely — with no arm semantically accepting the resulting runtime value against the declared return type. (An earlier
   revision said both build a "typed `Value::Record`/`Value::Variant`", which over-granted twice: "typed" claims
   an admission the fixture decoder never performs — the tag is carried, not checked — and only the fixture
   decoder produces variants at all.) Both sit outside any construction call site. (A further earlier revision called the input "untrusted bytes", which overstates it: a recorded fixture DOES carry outer operation, input-hash, input-equality and freshness checks — what is missing is semantic acceptance of the response against the current program declaration, not provenance checking.) This queue
   item covers both — but external review (2026-08-20) found they are **different mechanisms
   that need separate rows**, corrected here as 20a/20b rather than one joint claim: 20a
   (`value_from_fixture_json`) mints a nominally tagged value **without ever establishing the
   named declaration exists**; 20b (`map_response_to_value_json`) **does** consult the
   operation's declared output shape on its ordinary object arm — it looks up the real field names — but never
   validates a field's *value* against that field's declared refinement predicate, and on its other arms consumes
   the declaration only partially (array: first field only) or not at all (unresolved, childless). An earlier
   revision of this umbrella stated the ordinary arm's behavior at function grain, which the 20b control-flow
   tree below disproves. The earlier joint
   headline ("no declaration lookup" for both) was true only of 20a; stating it jointly
   over-answered for 20b. Both share: measurement only, no change to either mechanism, to
   `sole_constructor`, or to where-refinement machinery (see "What repair is not in this
   item," below); distinct from and complementary to item 1a's `sole_constructor` audit, which
   covers ordinary *construction* call sites, not reconstruction from serialized bytes; and
   distinct from #8661, which proves the analogous `sole_constructor` bypass on the **emitted
   Rust** target — this item does not restate that claim for the interpreter, it is a
   different door on a different target.

   **Background — why this needed its own audit.** An earlier pass over the emitted-Rust
   `#[derive(Deserialize)]` door found it writable but structurally unreached in the current
   corpus, and closed. **That conclusion is SUPERSEDED and is retained here only as the history that
   motivated this audit:** gunbc#8661 later executed the emitted-Rust door against a production
   `sole_constructor` carrier and found it forgeable, so "structurally unreached" is no longer the
   standing verdict on that target. Emitted-target realization is carried independently there, not here. That pass's own trace showed a recorded fixture decodes first into
   untyped `serde_json::Value` — but stopped there; it did not follow what the v1 interpreter
   does with that untyped value next. It converts it into a runtime `Value` itself — NOMINALLY TAGGED, not semantically admitted; an earlier revision wrote "*typed*" here, which contradicts this same item's umbrella a few lines above,
   in `src/v1/stage0/src/recorded_fixture.rs` `value_from_fixture_json`, which nobody had
   audited. The lesson generalizes: "decodes to untyped JSON, therefore inert" is not a
   sound inference once a second, typed reconstruction step exists downstream.

20a. **Fixture-replay door: `value_from_fixture_json` mints a value of a type it never checks
   was declared — below the floor: silent wrongness, which §4b places OUTSIDE the ladder
   rather than on it, and §5 forbids outright** (an earlier revision of this row said rung
   `mitigatable`; that is inflation — `mitigatable` means the failure occurs and harm is
   CONTAINED by typed outcomes, bounds, or rollback, and here nothing is contained: the
   violating value is admitted silently, no diagnostic is emitted, nothing is counted, and the
   consumer proceeds. It is also inconsistent with the emission-path row, which correctly
   records the same shape of defect as below floor) (measured 2026-08-20, bold-bear-246; scope
   handed down from fierce-ant-91).

   **Mechanism, confirmed by source read then by execution.** `value_from_fixture_json`'s
   Record arm reads a `__type` string out of the fixture JSON verbatim, interns it, reads
   whatever field names the same JSON object happens to carry, and returns
   `Value::Record { type_name, fields }` — no lookup against any declared type, no
   `sole_constructor` consultation, no refinement-predicate evaluation — the `sole_constructor` half is a SOURCE-INSPECTION finding, not an executed one: this item's probes ran a `NonEmptyStr` refinement and two fabricated undeclared nominal identities, and executed no interpreter reconstruction case against a declared sealed carrier. Its Variant arm does the
   same plus an equally unchecked `__variant` string. `src/v1/stage0/src/v1_interpreter.rs`
   calls this on `fixture.response` during hermetic replay, so the door is reached on the
   ordinary replay path, not a corner case.

   **20a's door — fixture decoder, executed.** (This was labelled "ARM 1"; the arm vocabulary is retired here because 20a/20b already name these two doors, and a second naming scheme for one concept is the §3 nickname violation — the same reason the exposure survey stopped being called "ARM 3".) Probe: `dag/test/claim/reconstruction_door_fixture_probe.dag`,
   a scratch service `DoorProbe.Fetch` (shell transport, `printf "%s" "positive-control"`,
   `output { id: NonEmptyStr from "stdout" }`). Built `claim_batch` at current head
   (`cargo build --release -p v1-compiler --bin claim_batch`, remote), recorded once wet
   (`--record --fixture-store <dir>`), then replayed hermetically (`--hermetic
   --fixture-store <dir>`) four times against the SAME on-disk fixture file, tampered
   between runs:
     - *Case 1, positive control:* untampered fixture, `witness_id_equals_positive_control`.
       **Result: PASS, exit 0.** Confirms the harness actually exercises the door (a
       zero-finding instrument is worthless without this).
     - *Case 2, predicate bypass, no `__type` tamper:* `response.fields.id.value` overwritten
       to `""` on disk, `witness_id_equals_empty`. `NonEmptyStr = String where non_empty`
       (`dag/std/types.dag`). **Predicted:** refusal, since the recorded value violates the
       declared refinement. **Observed: PASS, exit 0.** The empty string reconstructs into
       the `NonEmptyStr`-typed field with no refusal.
     - *Case 3, undeclared-type fabrication:* `response.__type` overwritten to
       `TotallyFabricatedRecordType_NeverDeclaredAnywhere` (no declaration by that name
       exists in either source root), `id.value` set to `"whatever-value"`,
       `witness_id_equals_whatever`. **Observed: PASS, exit 0.** The decoder manufactures a
       `Value::Record` of a type the program never declared — there is no invariant to
       violate here because there is no type to check against. This is a SEPARATE, BROADER nominal-admission
       defect rather than another instance of the refinement failure — an earlier revision called it "strictly
       worse" than Case 2, which is an unsupported ordering: one violates a real declared invariant on an
       ordinary value path, the other shows the decoder's nominal admission set includes identities absent from
       the program. Different defects, not ranked ones.
     - *Case 4, Variant-arm fabrication:* `response` replaced wholesale with a
       `{"__tag":"Variant","__type":"TotallyFabricatedVariantType_NeverDeclaredAnywhere",
       "__variant":"BogusCaseNeverDeclared","fields":{"id":{"__tag":"Str","value":"variant-value"}}}`
       shape, `witness_id_equals_variant_value`. **Observed: PASS, exit 0.** Confirms the
       Variant arm is the same hole as the Record arm, not a narrower one.

20b. **REST JSON-projection door: ordinary object projection derives field names from the
   declaration without accepting their values; the fallback and array arms bypass or truncate
   the declared output shape entirely — same below-floor rung as 20a (silent wrongness,
   §4b/§5), a distinct mechanism** (measured 2026-08-20, bold-bear-246).

   **Mechanism, confirmed by source read then by execution.** `map_response_to_value_json` is
   reached from a genuinely live REST round trip: `dispatch_rest` → `decide_rest_exchange` →
   (for a `Json` response format) `map_response_to_value_json` on the real HTTP response body.
   It reads `op_node.inferred`. Unlike 20a it therefore has access to the operation's resolved
   return declaration — but it consumes that declaration to *varying degrees per arm*, and an
   earlier revision of this row generalized the best arm to the whole function. Ordinary object
   projection derives all output field names from the declaration, so those names are not read
   verbatim off the wire as in 20a; the array arm uses only the *first* declared field and omits
   the rest; and the unresolved and childless arms bypass field projection altogether, returning
   whole-body `json_to_value`. **No arm accepts the resulting runtime value against the declared
   field types or refinement predicates.** So the honest class statement is broader than "field
   values are unchecked": `map_response_to_value_json` does not semantically accept an observation
   against the declared return type at all. On the ordinary arm each field's JSON value is
   converted with the untyped `json_to_value` and assembled into a `Value::Record` with zero
   validation against that field's declared refinement predicate; on the other arms the declared
   shape is not even fully constructed. An earlier revision said "on every branch" — a universal
   this item never enumerated, and it under-counted: `map_response_to_value_json` has TWO distinct arms that skip
   straight to `json_to_value`, one when the operation's return type does not resolve to `Resolved` and a second
   when it resolves but has no children, and the earlier wording named only the first. The arms this item
   identifies are therefore a control-flow tree, not a flat list, and it is stated as a tree because two successive
   flat revisions of it were wrong. At the top level: the unresolved-return-type skip; the childless-return-type
   skip; the array-response arm (JSON body is an array and the return type has children — the whole array is
   converted with `json_to_value` into the *first* declared field, so that field's refinement is unchecked and
   every other declared field is absent entirely); and otherwise the per-field loop. A further top-level guard,
   return type authored `List` with no children, is unreachable because the childless skip above it already
   returned. Within the per-field loop each declared field independently takes one of five outcomes, all of which
   reach the field unchecked against its refinement: with a `from` path, pointer found → `json_to_value` of the
   selected value, pointer absent → `Null`; without a `from` path, field-name key present → `json_to_value` of the
   selected value, key absent with exactly one declared child → `json_to_value` of the **entire response body**,
   key absent with multiple declared children → `Null`. The single-child whole-body fallback is the sharpest of
   these: the sole declared field silently receives the whole response rather than a missing-value marker. An
   earlier revision of this paragraph asserted a uniform "`Null` fill when the JSON body has no matching key",
   which is false for exactly that case. Stated as an enumeration rather than a universal, because nothing here
   establishes that the tree is exhaustive — and that caveat now carries two receipts, the array arm and the
   single-child fallback, each missed by a prior revision of this same enumeration. (Those three top-level arms, and four of the five per-field outcomes,
   are a source-level read only — see "What was NOT executed," below.) **A third,
   separate path exists and is unmeasured by this item:** when the operation's response format
   is `Text` rather than `Json`, `decide_rest_exchange` routes to `map_response_to_value`, not
   to `map_response_to_value_json` — a different function this item did not execute a case
   against. It is named here, source-read only, so Text/shell-transport outputs are not
   silently misclassified as covered by this item's executed evidence.

   **20b's door — REST JSON projection, executed, and the stronger of the two results.** (was "ARM 2") Probe:
   `dag/test/claim/reconstruction_door_rest_probe.dag`, a scratch service `DoorProbeRest.Fetch`
   (`transport rest { method: GET, path: "/fetch" }`, `output { id: NonEmptyStr from "id" }`,
   deliberately **no** `mock_response`). `claim_batch`'s default hermetic-mock mode refuses an
   operation with no `mock_response` ("no mock_response for operation Fetch — refusing to
   fabricate Unit"), which forced `--record --fixture-store <dir>` — i.e. forced a genuine
   HTTP dispatch rather than a `mock_response` evaluation of authored `.dag` source
   (`mock_response` would have measured source construction, not reconstruction, and was
   excluded from this audit for exactly that reason). A local stand-in HTTP server
   (`http.server`, `127.0.0.1:8991`) served two payloads across two separate fixture-store
   directories (one per case — `RecordedFixtureStore::record()` refuses to record a second,
   differently-shaped response for the same operation/input_hash in one store directory):
     - *Case 1, positive control:* server body `{"id":"valid-value"}`,
       `witness_rest_id_equals_valid`. **Result: PASS, exit 0**, with the transport log
       confirming a genuine `GET http://127.0.0.1:8991/fetch` dispatch through the real
       `ureq` client over a real socket.
     - *Case 2, predicate bypass, NO tampering of any kind:* server body `{"id":""}`,
       `witness_rest_id_equals_empty`. **Result: PASS, exit 0.** No fixture file was edited
       for this case — the empty string arrived over the wire from an ordinary HTTP response
       and was placed into the `NonEmptyStr`-declared field unchecked.

   **What was NOT executed (source-level read, stated as such, not overclaimed):** three of the
   top-level arms named in 20b's mechanism paragraph above (return-type-did-not-resolve, childless-return-type,
   and array-response-with-non-empty-declared-fields); four of the five per-field outcomes — the probe declares
   `output { id: NonEmptyStr from "id" }` and supplies an `/id` value in both REST cases, so it executes only the
   from-path-present-and-pointer-found outcome, leaving pointer-absent, no-from-path-with-key-present,
   no-from-path-single-child-whole-body, and no-from-path-multi-child-`Null` unexecuted; and the `Text`-format third path
   (`map_response_to_value`) were read from source, not driven by a constructed executing
   case. Named here as source-level evidence only; no rung claim rests on them.

   **Production reachability and declaration-surface survey — both doors are reached by bytes this repo does not author, and the (RENAMED: this section was called "ARM 3", which now unambiguously denotes the emitted-Rust/serde path carried by gunbc#8661 — a cross-PR identity collision)
   two doors reach that exposure differently.** 20a's fixture-replay door is reached by
   *repo-committed but externally-sourced* bytes: `dag/test/fixture/` carries JSON files
   recorded from real external effects — a live GCP OAuth token refresh
   (`dag/test/fixture/gcp_oauth_access_token_store/oauth2__Google__Refresh/991775fc306dcac0.json`,
   shape `{"response":{"__tag":"Record","__type":"Refresh","fields":{...}}}`, exactly the
   shape `value_from_fixture_json` parses), a `gcloud` ADC read, a Tailscale ACL fetch, a
   GitHub push event — and those fixtures are not idle: numerous `.dag` witness tests under
   `dag/test/claim/` name `dag/test/fixture` as their fixture store, so the door executes
   during ordinary witness-test replay, not only under ad hoc probing. 20b's REST-projection
   door is reached straightforwardly externally: any wet REST dispatch that takes the JSON
   response-projection branch reaches it directly, with no repo-committed intermediary at all. NOT every REST dispatch:
   this row establishes a few paragraphs below that the branch is FORMAT-dependent — `Json` routes to `map_response_to_value_json`
   (20b) while `Text` routes to `map_response_to_value` (unmeasured here). An earlier revision said "any production
   `transport rest` service dispatch ... hits it directly", which silently reinstated the transport-based split this
   row explicitly withdraws.

   **The two doors' Case-2-class findings have different reachability stories, and
   collapsing them would overstate 20a.** 20b's Case 2 needed *no* tampering whatsoever: an
   ordinary, legitimate upstream HTTP response of `{"id":""}` is exactly what a real service
   can return, gets faithfully recorded if a fixture is taken of it, and every subsequent
   hermetic replay of that fixture reconstructs the violating value forever — nobody edited
   anything, ever. 20a's Case 2 demonstrates the identical bypass on the fixture-replay door,
   but reaching it there required an on-disk tamper of the recorded JSON (a deterministic way
   to reach the same state in one run, not the threat model — the threat model is that an
   ordinary recorded response can already carry it, which 20b proves directly and which
   nothing distinguishes the fixture-replay door from once a fixture is taken of a real
   service that happens to return an edge-case value). 20a's Cases 3 and 4 are a different
   claim and must not be folded into Case 2's "no tampering needed" framing: a real service
   does not spontaneously emit a `__type` naming a type your program never declared, or a
   `Variant`-tagged envelope your service never promised — reaching those requires a malformed
   or hand-edited fixture, and what they demonstrate is the decoder's **admission scope** (it
   accepts input shapes with no declaration and no invariant to check at all), not its
   ordinary-case reachability. Both findings are real; stating them as one claim would let a
   reader dismiss the whole result as "if you can edit files you can do bad things," which is
   true only of Cases 3–4.

   **Live production exposure — a scanned figure, corrected in its type set, split on the
   axes that actually govern reachability.** A scan of production `.dag` (`dag/extdeps/`,
   excluding `test`/`fixture` trees) for `output { ... }` blocks whose field types name a
   **genuinely `where`-refined** alias found **22** matching field occurrences (identified at
   `(module, service, operation, field)` grain): `NonEmptyStr` (10), `SmResolvedVersionIdentity`
   (6), `FilePath` (5), `BrowserContext` (1). **CORRECTION, AND THE ERROR WAS IN THE ORIGINAL
   SCAN'S TYPE SET, NOT ITS ARITHMETIC.** An earlier revision of this row reported **33** and
   led with `sha: CommitSha` across three git modules and `access_token: Secret`. Neither type
   carries a refinement predicate at all: `CommitSha` is declared `type CommitSha = String`, a
   bare alias with no `where` clause, and `Secret` is declared `type Secret nominal_opaque =
   String` — opacity is a different mechanism from a predicate, and an opaque carrier has no
   proposition that reconstruction could violate (one nuance worth keeping: `SecretValue =
   Secret where non_empty` **is** a refined secret carrier, so dropping bare `Secret` does not
   mean secrets are categorically unrefined — none of this scan's 22 happens to be
   `SecretValue`, but a future re-scan should not assume the whole `Secret` family is exempt).
   Those `CommitSha`/`Secret` fields were counted as refined because the scan enumerated
   alias-shaped types rather than types with a `where` clause, so the figure was inflated by
   roughly a third AND its two most-cited examples were exactly the two that did not belong.
   The re-measurement restricts the type set to the 219 declarations matching `^type ... = ...
   where `.
   **The split axis is corrected too — transport (shell vs REST) is the wrong one and is
   dropped.** Fixture replay (20a) is **transport-independent**: a recorded shell- or
   REST-transport result can equally end up replayed through `value_from_fixture_json` later,
   so partitioning by transport implied a boundary 20a does not respect. `map_response_to_value_json`
   (20b) is **format-dependent**, not transport-dependent — it is the `Json`-response-format
   branch specifically, with the sibling `Text`-format branch routing elsewhere (named above,
   unmeasured). The two axes that actually govern which door's evidence covers a given field
   are: **wet/observed JSON projection (20b) vs. stored-fixture replay (20a)**, and **JSON vs.
   Text** response format. This scan did not re-classify the 22 occurrences along those axes
   (that reclassification, and the full `(module, service, operation, field)` tuple list
   rather than the per-type counts given here, is unmeasured — future work if this row is
   revisited); it withdraws the earlier shell/REST framing rather than replacing it with a
   verified new split.
   Caveats, unchanged and still binding: the scan matches single-line `output { ... }` blocks
   only, so multi-line and nested declarations are missed, and this session has not
   independently re-verified deduplication at `(module, service, operation, field)` grain —
   both are directions the 22 could move in either direction, so **it is not stated as a lower
   bound**; and **none of the 22 was executed** — **and their path membership was never classified.** An occurrence
   may belong to the executed fixture-replay path, the executed REST-JSON path, or the
   UNMEASURED Text path, and this scan does not say which; some may never be fixture-replayed
   at all. So the supported statement is narrow: the scan found 22 candidate `where`-refined
   output-field occurrences at intended `(module, service, operation, field)` grain, none
   executed and none classified by path. **That is a QUARRY POPULATION, not exposure
   evidence.** An earlier revision concluded "22 declared fields sit on a path proven
   unchecked", which does not follow — it silently assigns every occurrence to a door whose
   evidence is executed, including any that belong only to the unmeasured Text path.

   **What repair is NOT in this item, and why.** This item is measurement only — no change to
   `value_from_fixture_json`, `map_response_to_value_json`, `dispatch_rest`,
   `decide_rest_exchange`, `sole_constructor`, or where-refinement machinery. Two reasons:
   first, the shape of a fix belongs to whoever owns the decoder, not to an audit session;
   second, a repair landed inside a measurement item is exactly the kind of unreviewed
   coupling DESIGN §5 warns against (construction and validation are different obligations,
   and conflating "I found it" with "I fixed it" in one diff removes the operator's ability to
   review either independently). What repair would have to establish, without this item
   designing it further — and stated PER DOOR as CONJUNCTIONS, because an earlier revision
   offered two interchangeable global shapes ("either (a) resolve `__type`/`__variant` ... or
   (b) be a declared boundary"), which is the menu-instead-of-conjunction error #8661 had to
   correct in its own repair note: satisfying one item there would leave the others open.
   **20a (fixture replay)** needs nominal declaration admission AND schema / variant-membership
   admission AND per-field type-and-invariant acceptance. **20b (REST JSON projection)** already
   has the declared shape, so it separately needs declared field type AND typed conversion AND
   refinement / sealed-constructor acceptance AND a missing-/extra-field policy. **The Text
   path** is unmeasured here and needs its own equivalent acceptance receipt before anything is
   claimed about it. Either door may additionally be realized as an explicitly declared §4b
   boundary that refuses an externally-sourced value before it enters the typed `Value` space —
   that is a realization choice, not a substitute for the conjunctions above. Any shape must still pass a DISCRIMINATING INVALID case exactly like
   this item's Case 2 and a fabricated-type case exactly like Case 3 — this item's executed probe
   cases are what "the fix actually closes the door" should be checked against, not a new,
   separately invented test.

   **Open question, raised here for the operator/reviewer rather than decided in this item:**
   should these probe cases — SIX executions in total (three discriminating fixture-door findings plus one fixture positive control; one discriminating REST-door finding plus one REST positive control), or FOUR if counting discriminating invalid cases only; an earlier revision said "four ... (three fixture-door, two REST-door)", which cannot be both — be enrolled as permanent
   §4b regression controls once a wall lands, per the "dissolution on climb" meta-obligation
   (the discriminating RED and its positive control stay enrolled as the executing evidence a
   higher rung stays real)? The right end state is clearly enrollment — an unenrolled
   demonstration decays back into an unmeasured claim the moment nobody remembers it exists,
   exactly item 10's shape above. The open obstacle is mechanical: these cases need a fixture
   store, a `--record` pass, and — for the fixture-door cases — an on-disk tamper step between
   record and replay, none of which the CI required floor's fold (`claim_executor
   --required-floor`, "Building & checks" in DESIGN.md) currently has a form for. This item
   does not resolve whether that harness gets built, extended, or whether these cases are
   instead re-expressed as a form the required floor already runs; it only names enrollment as
   the target and the harness gap as what stands between here and there. **Confirmed
   mechanically unenrolled today, on three independent grounds, so this is a stated absence
   rather than an unverified one:** the required floor's discovery projects rows from `data`
   declarations (`v2.workflow.floor_discovery_producer`), and the two probe modules declare
   only `module`/`import`/`service`/`func` — no `data` row to project; the test-decl naming
   scan (`v2.workflow.floor_naming_hygiene`) enrolls decls from `*_test.dag` files, and the
   probes are named `*_probe.dag`, outside that convention entirely; and the whole-corpus
   census that would once have flagged a claim-less module under `dag/test/claim/` was
   deleted in gunbc#8155 (`floor_naming_hygiene_note` records the deletion), so the probes
   join roughly 90 other claim-less `.dag` files already present in that directory on main —
   not a novel gap. The probes are still ordinary executable `.dag` and do get typechecked by
   compile-clean whenever their import closure is touched; that is unrelated to floor
   enrollment and is not the safety argument here — the argument is the discovery/naming
   mechanics above, not a claim that nothing reads the files. Unenrolled-with-a-named-obstacle
   is the §4b *no untracked stall* shape, not an omission.

   **Reproduction, recoverable without the session that ran it.** 20a's door: build
   `v1-compiler`'s `claim_batch` binary at current head; run it against
   `dag/test/claim/reconstruction_door_fixture_probe.dag` (with `--source-root` covering
   `dag/` and the probe's own directory) once with `--function
   witness_id_equals_positive_control --record --fixture-store <dir>`; locate the single
   `*.json` file `--record` wrote under `<dir>`; for Case 2, edit `response.fields.id.value`
   to `""` in that file and re-run with `--function witness_id_equals_empty --hermetic
   --fixture-store <dir>`; for Case 3, restore then edit `response.__type` to any name absent
   from the source roots and `response.fields.id.value` to `"whatever-value"`, re-run with
   `--function witness_id_equals_whatever --hermetic --fixture-store <dir>`; for Case 4,
   replace the whole `response` object with the `__tag: "Variant"` shape shown in the probe
   file's comment, re-run with `--function witness_id_equals_variant_value --hermetic
   --fixture-store <dir>`. 20b's door: run any HTTP server on `127.0.0.1:8991` that answers
   `GET /fetch` with `{"id":"valid-value"}`; run `claim_batch` against
   `dag/test/claim/reconstruction_door_rest_probe.dag` with `--function
   witness_rest_id_equals_valid --record --fixture-store <dir1>`; point the same server at
   `{"id":""}` instead (or restart it with that body); run again with `--function
   witness_rest_id_equals_empty --record --fixture-store <dir2>` (a fresh directory — the
   fixture store refuses a second response shape for the same operation/input_hash in one
   store). In both arms, `exit_code=0` on the tampered/bypass cases is the finding; a nonzero
   exit or a typed refusal diagnostic would refute it.

21. **A NULLARY coproduct variant inhabits any declared type at a construction position, while
   its PAYLOAD-BEARING sibling is correctly refused.** CONFIRMED BY EXECUTION 2026-08-21, with a
   positive control and a discriminating control, on the source→`.dag`-acceptance path. This is
   the ordinary compiler safety floor (§4) — values inhabiting declared types — and it is a
   *partial* hole in a wall that demonstrably exists and fires, not a missing wall.

   Found from a fabric witness (`gunbc#8733`) that passed a bare `DeployRevisionRelation` into
   `gunbc.fleet_main_revision` `FleetDesiredObservedCurrentInput.relation`, which is declared
   `FleetRevisionRelationObservation` (a record). It compiled, and failed only at run time.

   THE FOUR-ARM MATRIX, one binary, one source revision, one closure. Each arm is a single
   module differing only in the marked expression:

   | arm | construction at the record-declared position | result |
   |---|---|---|
   | B | nullary variant — `relation: SameRevision` | **`0 blocking error(s)`** — ACCEPTED |
   | C | payload variant — `relation: RelationUnverifiable { cause: … }` | **REFUSED**, located |
   | D | nullary variant as a *direct call argument* — `takes_observation(x: SameRevision)` | **`0 blocking error(s)`** — ACCEPTED |
   | CTL | same literal, misspelled field name — `relationX: SameRevision` | **REFUSED**, located |

   Arm C's diagnostic is exactly the one arm B is owed, which is what makes this a located hole
   rather than an absent capability: `type mismatch: expected
   'Product(FleetRevisionRelationObservation)', got 'Coproduct(DeployRevisionRelation)'`.

   WHY BOTH CONTROLS ARE LOAD-BEARING. Arm C proves the type check at that position exists and
   reaches this pair of types. CTL proves the record literal is being examined at all — field
   completeness refuses there — so arm B's silence is not a skipped expression, a skipped
   module, or a harness artifact. Without CTL, "the literal is never checked" would be an equally
   good explanation and the finding would mislocate the defect. Arm D separates the seam: the
   miss is not specific to a nested field initializer, since a direct call argument accepts it
   too.

   RUNG, per path. source→`.dag` acceptance: **gap** — the invalid state is accepted.
   source→interpretation: the interpreter refuses loudly and typed —
   `NoSuchField { type_name: "DeployRevisionRelation", field: "current" }` — so this is NOT
   silent wrongness and NOT below the absolute floor; it is below the *ordinary compiler*
   baseline. The Rust-emission path is UNMEASURED and is a separate row when someone measures
   it; a runtime `NoSuchField` in the interpreter says nothing about what emitted Rust does.

   WHAT THIS CONTRADICTS, stated because the claim is on the record: #8262 claimed a compile-time
   refusal subset covering structured record and coproduct-variant literal mismatches at
   direct-call arguments and record-field positions. Arm C is inside that subset and holds. Arm
   B and arm D are inside its stated scope and do not. The claim is therefore too strong by the
   nullary case rather than wrong in kind — a rung-honesty correction, not a retraction.

   WORKING HYPOTHESIS, NOT ESTABLISHED: the structured-mismatch wall recognizes record-literal
   expressions and payload-bearing variant applications, and a nullary variant reaches the
   position through a different expression path (a bare name/identifier) that never arrives at
   that wall. The matrix is consistent with it and does not prove it; the fix search should start
   by asking which expression forms reach the check, not by widening the check.

   NEXT TRIGGER: a located compile-time refusal for arm B and arm D, with all four arms enrolled
   as permanent controls — B and D as expecting-red that flip to permanent regression controls
   when the wall lands, C and CTL as the positive controls proving the wall stayed reachable.
   Per §4b(4) the controls do not retire when the class climbs.

   REPRODUCTION. Copy `dag/` and `src/v2` to a directory UNDER the workspace root (the compiler
   refuses a `--source-root` outside it), add one module importing
   `gunbc.fleet_main_revision { FleetRevisionRelationObservation, DeployRevisionRelation,
   SameRevision, RelationUnverifiable }`, and compile each arm with
   `gunbc compile --output-dir <out> --source-root <copy>/dag --source-root <copy>/v2 --entry
   <the arm>`. `0 blocking error(s)` on arms B and D is the finding; a located `type mismatch`
   on either would refute it.


22. **`cmp_values` is not a total order, so `method_call.sort_by` orders unrecognised element
   kinds by map/list traversal accident rather than by any order at all** (opened 2026-08-22,
   session lively-moth-59, found while building the `sorted_map_keys` interpreter arm in
   gunbc#8841 — recorded rather than fixed, because it is a different arm with a different
   caller contract and repairing it inside that PR would have been a second, unasked change).

   INVALID STATE. `v1_interpreter` `cmp_values` matches four scalar pairs — `Int`/`Int`,
   `Float`/`Float`, `Str`/`Str`, `Bool`/`Bool` — and its final arm answers
   `std::cmp::Ordering::Equal` for **every other pair**: mismatched kinds, and `Record`,
   `Variant`, `List`, `Map`, `Set`, `Null`, `Unit` against themselves. `Equal` is not "I could
   not compare these"; it is a claim that the two elements tie. `v1_interpreter`
   `method_call.sort_by` passes that comparator to `sort_by`.

   HARM. A comparator that answers `Equal` for distinct elements is not a total order, so
   `sort_by` over a list of records or variants is order-preserving-by-accident rather than
   sorted — and silently so, because the call returns a plausible list and nothing refuses.
   This is the §5 fabricated-plausible-output shape one level down from the one gunbc#8841
   closes: there the risk was two realizations disagreeing on ONE order, here a single
   realization has no order to disagree about. It is the same reason that PR authored its own
   comparator over exactly `Str`/`Int`/`Bool` instead of reusing `cmp_values`, and refuses
   every other key kind: reusing it would have shipped a nondeterministic key order behind a
   green parity test.

   DISTINGUISHING FACTS, stated at the grain actually held. OBSERVED ON: `cmp_values`' arms as
   READ in `src/v1/stage0/src/v1_interpreter.rs`, and the `method_call.sort_by` arm's use of it,
   both by source read. CLAIM ABOUT: `sort_by`'s runtime behaviour on unrecognised element
   kinds, which was NOT executed. So the mechanism is confirmed and the victim is not — no
   corpus `sort_by` call site has been shown to pass non-scalar elements, and the population is
   unmeasured. The two halves are separated deliberately: an executed comparator read does not
   license a sentence about a call this session never ran.

   RUNG FOUND AT: **below the ladder** if the claim-about half holds — silent wrongness, which
   §4b places outside the ladder rather than on its bottom rung — and merely *mitigatable* if
   every live `sort_by` receiver turns out to be scalar-element, which is exactly the unmeasured
   part. CEILING: **structurally impossible**, and the reason is that the ordering question is
   decidable per element kind: a comparator that returns a typed refusal for kinds it cannot
   order (the shape gunbc#8841's `sorted_map_keys_in_emitted_order` already uses) makes the
   silent tie unwritable, and a `sort_by` whose contract requires a total order can demand one.

   NEXT TRIGGER, in order and each cheap: (1) census the corpus `sort_by` call sites by receiver
   element kind, which converts CLAIM ABOUT into an observed population and decides between the
   two rungs above; (2) if any non-scalar receiver exists, that is the discriminating RED —
   a `sort_by` over two distinct records asserting a defined order; (3) replace the fail-open
   final arm with the typed refusal. Step (1) is the blocking one; nothing here should be
   repaired before it, because a comparator widened on speculation is the same unproven
   machinery §4b(2) says to leave as a declared row instead.

23. **An anonymous record literal at an actual position is judged by NOTHING at the direct-call
   seam** (opened 2026-08-22, session `proud-ant-819`, measured on gunbc#8864).

   INVALID STATE. A record literal written directly as an argument may disagree with its
   formal's declared type, in any way, and no diagnostic is produced. `direct_call_arg_mismatch_diags`
   returns the empty list for an `ExprRecordLit` actual before the compatibility predicate is
   ever reached, so the argument's TYPE is never a checked fact at this seam for that
   expression form.

   HARM. The mismatch survives to emission and becomes a rustc error in the emitted target, or
   — where the target's own realization happens to accept it — is not caught at all. It is the
   ordinary argument-conformance floor (DESIGN §4b, "applications bind in exact bijection,
   values inhabit declared types"), unenforced for one writable expression form.

   DISTINGUISHING FACTS, and the reason this is filed HERE rather than against the exemption.
   The skip is in what PRODUCTION judges, not in what `module_skips_direct_call_arg_check`
   hides: it fires for exempt and non-exempt callers alike. Measured on two entries at ref
   `90986d1946`, it covers **521 relations (2.7% of exempt relations) on the 03_ingest closure
   and 520 on 00_compile**, plus 7 non-exempt relations in each — a population of comparable
   size to the 115 candidates the exemption itself hides, every one of which turned out to be a
   false positive.

   WHY NO EXISTING INSTRUMENT SEES IT, which is what makes it a standing gap rather than a
   backlog item. The shadow census
   (`docs/probes/shadow_direct_call_arg_conformance_2026-08-22.md`) preserves the skip as
   `RepresentationRelationUnadjudicated` rather than inventing a judgement, so it counts the
   population but cannot judge it. A guard-removal arm cannot see it either, because the skip
   is UPSTREAM of the guard being removed. **Both available instruments are blind to the same
   rows, in the same place, for different reasons** — so the population's frequency is
   observable and its defect rate is not.

   RUNG FOUND AT. *Below the ladder* on the source→`.dag`-acceptance path for this expression
   form: the wrong state is not mitigated, refused, or counted — it is silent. The neighbouring
   forms (a named value, a call result, a literal) reach the predicate normally, so this is a
   per-AST-form hole in an otherwise-real check, structurally the same shape as the
   coproduct-variant hole in §11.1a rather than a missing check.

   CEILING. *Structurally guaranteed* — decidable, and the authority already exists: the
   formal's declared type is in hand at the seam and the literal's field set is known, so
   inhabitance is the same judgement `direct_call_arg_type_mismatch` already performs for every
   other actual. No grounding is missing; this is a *wall now* in §5's vocabulary, not a wall
   after grounding.

   NEXT-RUNG TRIGGER. Route the `ExprRecordLit` actual to the same predicate as every other
   actual, or — if the skip encodes a real representation gap rather than an oversight — replace
   the silent empty-list return with a typed, located, COUNTED diagnostic naming that gap, so the
   deficit's frequency stops being zero by construction. Either move retires this row; the
   present state, an untyped silent skip, retires nothing.

   NOT CLAIMED. Whether any of the 1,041 measured relations is actually a defect is UNKNOWN and
   deliberately not guessed: no instrument in the repository can currently answer it. The row
   records a population and a blindness, not a defect count.

   NOT THE SAME AS THE #8865 FINDING, and the two are easy to conflate because both say "record
   literal". This row is about a record literal standing as an ACTUAL at a direct call, whose
   ARGUMENT type is never judged. gunbc#8865 is about a coproduct payload inhabiting a FIELD
   declared as its parent coproduct *inside* a record literal (`CppHolder { subject: cpp_inner() }`,
   accepted by typing, `PatternMatchFailure` at runtime) — a construction seam, in ordinary
   non-`v2` modules, which neither this row nor `module_skips_direct_call_arg_check` reaches.
   Two seams have now been measured; the number of seams is NOT known to be two (gunbc#8868).

24. **A type name declared in two modules resolves to whichever binding `lookup_binding_by_name`
   returns, and the occurrence-identity arm that could have adjudicated it is dead in type
   position across an entire compiler closure** (opened 2026-08-22, session proud-ant-819, found
   while establishing the join key for cut A's transparency discriminator — recorded, not fixed,
   because repairing it means editing global resolution and that was explicitly held out of the
   cut).

   INVALID STATE. `v1.compiler.04_env` `lookup_type_for` has two arms: an occurrence-identity arm
   taken when `node.ident` is `Present`, and a name arm — `lookup_type_by_name` — taken when it
   is `Absent`. Where a name is declared by two different modules, the name arm returns one of
   them with no refusal, nothing recording that a choice was made, and no way for a consumer to
   discover that a second declaration existed.

   DISTINGUISHING FACTS, measured by execution over the `src/v2/compiler/03_ingest.dag` closure
   with `lookup_type_for` instrumented at both arms: **370,118 observations, 370,118 on the NAME
   arm, 0 on the IDENT arm**, across 2,690 distinct names, none observed under both arms. So the
   identity arm is not merely rare in type position — it is dead there, and every type resolution
   in that closure is name-keyed. Four names were observed colliding: `Byte` (105 observations),
   `FilePath` (45), `NonNegativeInt` (18), `PositiveInt` (18). Eight names are declared twice by
   source scan (`Byte`, `FilePath`, `FixedPointCheck`, `Float32`, `Float64`, `NonNegativeInt`,
   `ObjectId`, `PositiveInt`); the other four are not reached by this closure. `PositiveInt` is
   the sharpest specimen because its two declarations have *different shapes*, not merely
   different homes: `dag/std/integer.dag` declares `type PositiveInt = Nat where gt_zero`, a
   brand, and `src/v2/std/refinement.dag` declares `type PositiveInt { refined: Refined<Int> }`,
   a record.

   THE TENSION WITH DESIGN.md, stated here rather than left for a reader to collide with. §4b's
   guarantee-recovery paragraph names, as one of three confirmed structural holes on the
   source→`.dag`-acceptance path, "a census-AMBIGUOUS type name resolves by silent
   last-import-wins instead of refusing", and records all three as having **0 live exposure today
   by targeted grep**. This row measures four such names actually resolved, 186 times, in one
   closure, by execution. THE TWO ARE NOT NECESSARILY IN CONTRADICTION AND THIS ROW DOES NOT
   ASSERT THAT THEY ARE: a grep for ambiguous *imports* and an execution count of *names declared
   twice and resolved by name* are different instruments over populations that may not coincide —
   an ambiguous import is a source-level condition, a colliding declaration resolved by name is a
   runtime one, and a name can be the second without ever being the first. What is owed is the
   adjudication, and it is cheap because the execution side already exists: decide whether these
   four are instances of the class DESIGN calls zero-exposure. If they are, that clause needs
   updating and the class needs re-ranking. If they are not, the boundary between the two
   populations needs stating in the clause itself.

   RUNG FOUND AT: **below the floor** if a wrong declaration can be selected silently, because
   DESIGN's own floor clause is *names resolve* and this resolves a name to one of two
   declarations by construction order rather than by any modeled fact. Stated conditionally
   because the selection's *correctness* per site is unmeasured here: what is measured is that
   nothing adjudicates, not that a specific site got the wrong answer.

   CEILING: **structurally impossible**. Ambiguity is decidable — the two declarations are both
   in the module graph the compiler already walks — so a name resolving to more than one
   declaration can be a typed, located refusal rather than a silent pick, and the invalid state
   then has no spelling.

   NEXT TRIGGER, in order: (1) adjudicate the DESIGN tension above, which is a population
   comparison and not a code change; (2) a discriminating RED — a two-module fixture declaring
   one name twice and calling across it, asserting a located refusal; (3) the refusal in
   `lookup_type_by_name`. Step (1) blocks the others: re-ranking a class whose exposure two
   instruments disagree about would set the priority off the wrong number.

25. **Two structurally different named records interchange at the direct-call argument seam with
   no diagnostic** (opened 2026-08-22, session proud-ant-819, found as a *failed negative control*
   while building cut A's acceptance set — it was authored to prove the widened transparency test
   had not erased compatibility checking, and it turned out to prove nothing because the seam was
   already silent there).

   INVALID STATE. `fn takes_other(o: Other) -> Int` called as `takes_other(o: x)` where
   `x: AliasRec` and `type AliasRec = Rec`, with `Rec` and `Other` two records sharing no field
   name, produces **zero diagnostics** — before and after cut A, so this is pre-existing and
   untouched by it.

   HARM, and why it reframes cut A's own result. This is DESIGN's floor clause *values inhabit
   declared types* failing for the most ordinary case there is: two unrelated nominal records.
   The seam was simultaneously refusing a record ALIAS against its own base — a refusal it owed
   nobody — while accepting two unrelated records outright. Cut A removes a false refusal at a
   seam that is failing to make true ones, and stating only the first half would make the seam
   look stricter than it is.

   DISTINGUISHING FACTS, and the paired nonzero is what makes the zero mean anything: measured
   in the same before/after dispatch as cut A's acceptance set, on the same fixture file, so the
   silence is not a scoping artifact — a genuine kernel-vs-record mismatch in the *same file*
   (`String` passed where `Rec` is declared) refuses in both arms, located, with identical text.
   So the seam is reached, is executing, and does judge; it simply does not discriminate record
   against record. Without that control, "no diagnostic appears" would be equally consistent with
   the seam not running at this position at all, and the row would be dismissable.

   THE REFUSAL IS ALSO ASYMMETRIC, measured on one file across two binaries (main, and gunbc#8873
   at `e78c61c888`): `String` passed at a `Rec` formal refuses on both, while an alias of `Rec`
   passed at a `String` formal is SILENT on both. So the seam's kernel-vs-record judgment runs in
   one direction only. Recorded inside this row rather than as a fourth row, because one
   observation of a neighbouring asymmetry is a distinguishing fact about this class, not an
   independent class.

   FOUND TWICE, INDEPENDENTLY, WHICH IS THE ONE FORM OF EVIDENCE AN AUTHOR CANNOT PRODUCE ALONE.
   Reproduced by session still-carp-717 from a different direction and under a different
   hypothesis — they were chasing the empty-record explanation for an unrelated failure, not
   looking for this — with a fixture sharing no text with the one above: `type RevA { a: String }`
   and `type RevB { b: Int }`, no alias relation between them, `consume_a(r: b)` compiling clean,
   identical on both binaries, and two empty records substituting for each other as well. Their
   arm carries the same `String`-at-a-record-formal control and it refuses there too
   (`expected 'Product(RevA)', got 'Primitive(String)'`). Two lanes, two hypotheses, two fixtures,
   one result.

   SCOPE, sharpened by that second reproduction beyond what the first established. (1) It
   reproduces in an ordinary NON-EXEMPT module, so it does not sit behind
   `module_skips_direct_call_arg_check` and cut B — deleting that exemption — would not touch it;
   this is measured, not expected. (2) The two records differ in field NAMES *and* field TYPES and
   stand in no alias relation, so this is not the transparent-alias class: neither the withdrawn
   cut A repair nor gunbc#8873 addresses it, and neither should be credited with closing it.
   Shape-wise it is the gunbc#8865 coproduct-payload class arriving at a different seam.

   NOT THE SAME AS ROW 23, which now sits directly above it and concerns record-typed arguments
   at the same seam — the two are one seam failing in two independent ways and must not be merged.
   Row 23 is about an EXPRESSION FORM: an `ExprRecordLit` actual returns the empty diagnostic list
   *before the compatibility predicate is reached at all*, so no type is compared. This row is
   about the PREDICATE ITSELF: two named record types are compared and found compatible. Fixing
   either leaves the other live — routing record literals into the predicate would hand them to a
   comparison that does not discriminate record against record, and teaching the predicate to
   discriminate would still leave literals bypassing it.

   RUNG FOUND AT: **below the floor** on the source→`.dag`-acceptance path — accepted, silent,
   no diagnostic. The source→interpretation path is UNMEASURED here and is a separate row when
   someone measures it; a field access on the wrong record may well refuse at runtime, and that
   would make this below-*ordinary-compiler*-baseline rather than silent wrongness, exactly as
   row 21 distinguishes.

   CEILING: **structurally guaranteed**. Two named records with different declarations are
   distinguishable from modeled structure the compiler already has.

   NEXT TRIGGER: measure the interpretation path first, which decides the rung and therefore the
   ranking; then a discriminating RED at the seam.

26. **A brand is unenforced at the direct-call argument seam in both directions, and a cross-brand
   equality is not merely deferred but entirely unobserved** (opened 2026-08-22, session
   proud-ant-819, established as cut A's discovery step: find any consumer that ENFORCES brands,
   both outcomes being answers).

   INVALID STATE, led by the sharpest specimen because it is the one with no observation at all:
   given `type Branded = String where brand("Branded")`, the expression `b == s` for
   `b: Branded, s: String` compiles with **no diagnostic of any kind** — not a refusal, not an
   advisory, nothing. That is distinct from, and worse than, the seam's other arms: passing a bare
   `String` into a `Branded` parameter, and a `Branded` into a `String` parameter, both compile
   with no refusal but DO raise the deferred-predicate advisory
   (`where-refinement unenforced: predicate 'Brand' … predicate deferred at compile time`). So the
   class is two rungs, not one: observed-but-unenforced at the argument positions, and *absent*
   at the equality.

   HARM. This is DESIGN §4b's own warning — "richer type names are not safety; a brand, wrapper,
   or `Validated<T>` is cosmetic until construction and acceptance enforce the distinction" —
   instantiated with a receipt rather than left as a caution. A brand that neither refuses nor is
   observed at `==` is a comment.

   WHAT MUST NOT BE READ INTO THE ADVISORY. "Observed but unenforced" is not a licence. The
   advisory records that a predicate was deferred; it does not record that the value satisfies it,
   and nothing downstream consumes it as evidence. A reader who takes the advisory's existence as
   partial enforcement has read a diagnostic, not a guarantee.

   DISTINGUISHING FACTS AND COVERAGE BOUND. Measured PER SITE on an authored fixture
   (`.cutA6/consumers.dag`, five advisory sites, identical before and after cut A), not as a
   corpus population — so this row establishes the mechanism at the sites it names and makes no
   claim about how many brand sites exist in the tree. The baseline is the planted fixture, not a
   measurement of current main.

   RUNG FOUND AT: *mitigatable* at the argument positions (the deferral is typed, located and
   counted, so its frequency is observable) and **below the ladder** at the equality, where
   nothing fires at all.

   CEILING: **structurally guaranteed** for the nominal half — brand-ness is carried structurally
   and the compiler can refuse a cross-brand interchange from modeled facts. The *predicate* half
   is a separate, weaker question (a general `fn(T) -> Bool` refinement is the undecidable residue
   §4b names) and this row does not claim it climbs.

   NEXT TRIGGER: the equality arm first, because it is the one with no observation and therefore
   no frequency — a class whose deficit rate is zero by construction never ranks for climbing
   (§5's absorbing-fallback argument applied to a missing check rather than a widening one).

27. **NEGATIVE RESULT: widening the resolve-seam alias peel breaks variant projection, and the
   mechanism is unidentified after three refuted hypotheses** (opened 2026-08-22, session
   proud-ant-819. This row exists because the change that caused it was WITHDRAWN — nothing in
   `main` is broken by it — so it is recorded for the next person who reaches for the resolve seam,
   not as an outstanding defect).

   WHAT WAS ATTEMPTED. `v1.compiler.04_resolve` `is_transparent_primitive_alias_rhs` decides
   transparency by asking whether the alias's base is a KERNEL type, which makes transparency a
   property of the BASE rather than of the DECLARATION, so `type AliasRec = Rec` keeps a nominal
   identity it never declared and the direct-call seam refuses the interchange. The attempted
   repair replaced the kernel test with "the resolved structural node carries a non-empty authored
   name". Six authored witnesses passed. CI over the corpus refused in three classes at once.

   WHY IT IS THE WRONG SEAM, which is the transferable part. Resolve has THREE downstream
   consumers and the attempt broke all three: the direct-call comparison (20 diagnostics,
   `expected 'Product(Hash)', got 'Product(Fnv1a64Structural)'` — peeled on one side of the
   comparison and not the other, and `type Hash = Fnv1a64Structural` is the alias behind 92 of the
   115 relations the census was built on, so the repair turned its own headline population into a
   NEW red); variant projection (18 diagnostics, below); and EMISSION (regen surface drift in
   `extdeps_cargo_version.rs`, `extdeps_version_semver.rs`, `std_integer.rs`). A transparency
   relation belongs at the COMPARISON, where leniency is meaningful — emission and variant
   projection are not judgment surfaces and there is no coherent sense in which they should be
   lenient. gunbc#8873 places it there and closes this defect: measured on one identical fixture,
   `main` produces 5 diagnostics (4 alias false reds across record and coproduct kinds, both
   directions, plus one genuine kernel-vs-record mismatch) and gunbc#8873 at `e78c61c888` produces
   1 — the genuine one, same site, same text.

   THE UNEXPLAINED CLASS. 18 diagnostics of the form `variant 'X' not found in type 'X'`, the two
   names identical: `OllamaRuntime` (×2), `MtJadeRev1_0` (×6), `AcceptanceNodeReopensActiveFrontier`
   (×6), `GoogleCloudInteractiveAuthenticationRequired` (×4).

   REPRODUCTION, established by corpus subtraction rather than by an authored fixture:
   `dag/gunbc/spark/serving_release.dag` compiled alone at module scope
   (`--source-root dag --source-root src/v2 --entry <it>`) is rc=0 with 0 blocking diagnostics on
   `main` and rc=1 with `variant 'OllamaRuntime' not found in type 'OllamaRuntime'` on the widened
   binary. So the class is attributable to the change and reproduces WITHOUT whole-corpus context.
   `dag/extdeps/ocp/mt_jade/subject.dag` does NOT reproduce at module scope in either arm, so that
   instance needs a wider closure and the two are not the same reproduction.

   THREE REFUTED HYPOTHESES, listed because knowing which explanations are dead is worth more to
   the next reader than a fourth that merely sounds right. (1) *The base is an empty record.*
   `type OllamaRuntime {}` and `type MtJadeRev1_0 {}` are empty records, and an empty record is
   `NoConnective` with zero children — structurally indistinguishable from the childless leaf the
   predicate keys on. REFUTED by its own fixture: an authored `type EmptyRec {}` /
   `type AliasOfEmpty = EmptyRec` pair behaves exactly like a full-record pair (four mismatches
   before, clean after) and produces no variant diagnostic in either arm. (2) *Asymmetric peeling
   between the two call sites.* Consistent with class 1 but never measured, and it does not explain
   a variant lookup failing. (3) *The kernel-versus-named axis is the discriminator.* The corpus
   two-column pass agreed with it at 213 declarations with 0 counterexamples in either column —
   and that agreement is exactly what made the change look safe, so the pass measured what the
   predicate decides, not what its consumers do with the decision.

   RUNG: not on the ladder — this is a property of a withdrawn change, not of `main`. Recorded as a
   hazard for the seam.

   NEXT TRIGGER, for anyone who reaches for the resolve peel: start from the module-scope
   reproduction above, not from an authored fixture. Two independent authors wrote eleven
   adversarial arms between them (six here, five in gunbc#8873) and ALL ELEVEN pass on the widened
   binary, including a boundary control written specifically to catch over-peeling. The corpus was
   the only instrument that detected any of this.

28. **THE POSITION CENSUS, RE-MEASURED INDEPENDENTLY, AND THE TWO CELLS IT ADDS: a parameter's
   default-value expression is analysed by nothing, and the map-key position refuses by grammar
   while passing by typing** (opened 2026-08-22, session quiet-boar-696, measured on the compiler
   floor lane rather than inherited from either lane that found a seam).

   WHY THIS ROW IS NOT A SECOND COPY OF gunbc#8925's. That row enumerates the fourteen
   `parse_type_expr` sites and reports seven accepting a plain kernel value where a coproduct is
   declared. THIS row re-measures the same question from an independent fixture set and REACHES
   THE SAME SEVEN, which is the one form of evidence an author cannot produce alone; it is
   recorded as corroboration, not as a citation of that row. What it ADDS is two cells neither
   census carried, and one correction of tense.

   INVALID STATE (a) — SPLIT OUT AS ITEM 29 BELOW AND LEFT HERE ONLY AS A POINTER. A parameter's
   DEFAULT-VALUE expression is judged by nothing at all. `fn a_pd(r: Rel = 7)` compiles clean, and so does
   `fn a_pd(r: Rel = nosuchname_zzz)`. The reachability control PASSING is what makes this a
   finding rather than a fourteenth silence: at every other value-bearing position an undefined
   name refuses, so this is not a type judgment missing from a live position — the expression is
   not analysed at all. It is strictly worse than the seven, and it is the only cell where "no
   judgment runs here" is established by a control rather than inferred from a quiet arm.

   INVALID STATE (b). The map-KEY position refuses by grammar and passes by typing. `{ 7: 1 }` and
   `{ mk_inner(): 1 }` at a declared `Map<Rel, Int>` are refused as `module index refused: 1
   unparseable .dag source(s)`, a PARSE refusal; `{ nosuchname_zzz: 1 }` is ACCEPTED, the undefined
   name silently read as a string key. So the position reads as walled from its refusal column and
   is not — the refusals belong to the key form, and the one specimen that reaches typing passes.
   A reader folding map-key into the angle-argument site's "refuses" evidence would be citing the
   grammar as a type wall.

   CORRECTION OF TENSE, small and load-bearing. gunbc#8925's disposition table reads the named
   record/variant field position as *refuses*, qualified as "and only because #8876 walls it".
   Measured on `main` at `abf7194e2b2`, #8876 is NOT merged, so the payload-at-parent specimen is
   ACCEPTED at **all twelve reached positions**, the record field included. The qualification is
   in that row and a reader who takes the cell without it will count eleven.

   DISTINGUISHING FACTS. Thirteen arm families, four specimens each — a member of the declared
   coproduct, a plain kernel value, one arm's payload at the parent position, and an undefined
   name as the reachability control — each a single self-contained module differing only in the
   marked expression, compiled with `gunbc compile --source-root <arm> --entry <arm>/probe.dag
   --dry-run` against a binary built from the tree at `abf7194e2b2`. Two independent dispatches;
   every cell identical across both. Receipts, instrument and the site-grain fold:
   `docs/probes/declared_type_inhabitance_position_census_2026-08-22/`.

   TWO INSTRUMENT FAILURES, RECORDED BECAUSE BOTH FAIL TOWARD ZERO and both produced a clean
   all-zero table that reads exactly like the finding: a whole-root compile refuses on the memory
   budget and EXITS 0 (`WholeCorpusCompileBudgetBelowMeasuredDemand`, remedy `--entry`), and a
   `--source-root` outside the workspace root PANICS (`repo_relative_path_normalized`). Neither is
   visible in a diagnostic count. The reachability control's own zero is what exposed both, which
   is the general reason that control is not optional here.

   RUNG FOUND AT: **below the floor**, unchanged — DESIGN §4b puts *values inhabit declared types*
   in the ordinary compiler floor, so this is a below-baseline safety regression and not a class
   sitting at mitigatable. CEILING: **structurally guaranteed** for the decidable classes (exact
   member, transparent alias, payload-at-parent, kernel-at-structured); the undecidable residue
   (generic coproducts, the `Optional` carrier, a produced side whose identity was erased upstream)
   is a typed, COUNTED `Undecidable`, never a silent accept and never a fabricated refusal.

   NEXT TRIGGER, in order. (1) One `DeclaredTypeObligation` per value-bearing position and ONE
   `declared_type_inhabitance` relation deciding it — consuming gunbc#8873's transparent-alias
   identity rather than reimplementing it — with the position supplying only its tag and span;
   seven local predicates would be seven representations of one rule, and the corpus already
   carries the receipt for what that costs (three existing predicates with three different scopes,
   between which an actual that is a CALL falls). (2) Per position, five arms: positive, kernel
   negative, payload negative, reachability, and a DISCRIMINATOR — that position's obligation
   producer disabled, exactly that position's controls red. (3) Positions land one at a time, each
   with its own corpus measurement, because turning a wall on IS the census. The direct-call
   argument position is LAST and is BLOCKED on gunbc#8925 merging: deleting the `v2.` exemption is
   necessary and insufficient, and closing it first would report the class closed while seven
   positions stay silent. The design is
   `docs/probes/declared_type_inhabitance_position_census_2026-08-22/design.md`.

   NOT CLAIMED. No count of live corpus defects at any of these positions. This row measures what
   the compiler judges, not what the corpus contains, and gunbc#8876's eight live sites are the
   standing evidence that the two numbers are not each other.

29. **DECLARATION-SITE DEFAULT-VALUE EXPRESSIONS are RESOLVED but never INFERRED, so an undefined
   name at one is accepted — and this is a SECOND AXIS, cut by pass coverage rather than by
   grammar position** (opened 2026-08-22, session quiet-boar-696, split out of item 28 because
   severity does not follow position-count: at item 28's seven positions a judgment runs and
   reaches the wrong verdict, and here the judgment that would refuse never runs).

   INVALID STATE, MEASURED AT TWO POSITIONS AND NOT ONE. Both of these compile clean:

       fn a_pd(r: Rel = 7)                  fn a_pd(r: Rel = nosuchname_zzz)
       type H { rel: Rel = 7 }              type H { rel: Rel = nosuchname_zzz }

   THE SECOND AXIS IS NON-EMPTY, WHICH IS WHY THIS IS NOT A CELL OF ITEM 28. Item 28 enumerates
   positions by GRAMMAR — the call sites of `parse_type_expr`. This class is cut by PASS COVERAGE:
   expressions `v1.compiler.resolve` walks and `v1.compiler.infer` never does. The two are
   different axes and the second is not a subset of the first — a parameter default and a field
   default are two USES of two different grammar sites (`parse_param`, `parse_field`) that share a
   pass-coverage fate, and a matrix complete on the grammar axis reads as complete full stop unless
   this is said. The control that makes both zeroes readable: an unannotated `let x =
   nosuchname_zzz` in a function body REFUSES in the same run, so undefined names are refused in
   general and these two positions are the exception.

   THE DENOMINATOR, ENUMERATED FROM THE INFERENCE SIDE RATHER THAN SAMPLED. Enumerating from the
   resolve side does not work — `resolve_expr_types(` has ~40 call sites in `04_resolve` and most
   are its own recursive descent — but inference's coverage of DECLARATION nodes is a small closed
   set: which declaration kinds `v1.compiler.infer` descends into, and for each expression-bearing
   field, whether it WALKS the expression or only TESTS ITS PRESENCE. Read out, that gives THREE
   classes, not two:

   *Walked, typed* — `fn` body against the declared return; `data` initializer against its
   annotation. *Walked, UNTYPED* — transport `properties` (body/query/stdin among them) and the
   `svc_auth_source` item property, both through `infer_expr(expected: none)`: an undefined name
   DOES refuse there, and inhabitance has nothing to compare against, which is a different defect
   with a different repair (thread the declared type, do not add a pass). THE SPELLING IS NOT THE
   CLASS: `expected: none` appears at 25 of `04_infer`'s 47 `infer_expr` call sites and is CORRECT
   at most of them — a match scrutinee, an `if` condition, a method receiver, a binary operand and a
   lambda value have no declared type in context to thread. The property is that a declared type WAS
   available and was not threaded, which only the declaration-side read decides; grepping the
   argument returns 25 and a false population. *Never walked* — the two
   confirmed members above, plus four rows flagged by structure and NOT counted as members here:
   non-`svc_auth_source` item properties (`{ prop: p, diagnostics: [] }`, passed through), `uses`
   resource config args (the scope is extended with the resource TYPE; the arg expressions are
   untouched), service `exit`-entry status patterns (no arm in `04_infer` at all), and transport
   `children` (`infer_transport_node` copies `children: t.children` unchanged while
   `resolve_transport_binding` walks them). Full table:
   `docs/probes/declared_type_inhabitance_position_census_2026-08-22/pass_coverage.md`.

   THE SIGNATURE, which is what makes this a class rather than a list: a declaration node whose
   expression child is read by resolve and reached by inference only through a presence test or a
   passthrough. `param_node_default_value(n: param) != none` and `titem.transport != none` are the
   same tell.

   WHY THE SECOND ARM IS THE FINDING AND THE FIRST IS NOT. At item 28's positions the reachability
   control REFUSES, which is what establishes the position is analysed and makes a passing kernel
   arm a missing judgment. Here the reachability control PASSES. An undefined name surviving a
   position is not an inhabitance gap: DESIGN §4b's floor clause opens with *names resolve*, and
   this one does not.

   WHAT THE STRUCTURE SAYS, AND IT CORRECTS AN EARLIER, STRONGER VERSION OF THIS ROW. The first
   draft — and a ruling written on top of it, both withdrawn the same day — said nothing looks at
   the expression at all, and that is FALSE. `v1.compiler.parse` `parse_param` parses the default
   on `EatConsumed` of `=` and stores it (`default_value: Present { value: r4.expr }`);
   `v1.compiler.resolve` `resolve_param` reads it back through `param_node_default_value` and
   passes it to `resolve_expr_types`, the same treatment a field default gets. So the expression is
   parsed, kept and resolved. The gap is one pass further on: `resolve_expr_types`' `ExprVar` arm
   returns the node unchanged with an EMPTY diagnostic list — it resolves TYPE references inside an
   expression and never binds VARIABLE references — and `v1.compiler.infer` touches
   `param_node_default_value` at exactly one site, the call-shape test for whether a parameter is
   required. No inference arm walks the default expression. Undefined-name refusal and declared-type
   inhabitance BOTH live in infer, which is precisely why both arms pass.

   OWNERSHIP, SETTLED RATHER THAN ROUTED. Because the missing pass is inference, this cell sits in
   the same territory as item 28's other positions rather than upstream of it, and the
   declared-type-inhabitance lane keeps it. An interim ruling routed it away on the withdrawn
   premise above; that routing is void. The distinction the two readings turn on is worth keeping:
   *unanalysed* and *analysed by a pass that does not judge this* are different defects with
   different repairs, and only the second is supported by the structure.

   DISTINGUISHING FACTS. Measured on a binary built from `abf7194e2b2`: at the parameter default,
   four specimens (declared member / plain kernel / arm payload at parent / undefined name), all
   four ACCEPTED, against eleven sibling grammar positions whose undefined-name arm refuses in the
   same run; at the field default, three specimens (member / kernel / undefined name), all three
   ACCEPTED, against the in-body `let` control refusing in the same run.
   Structure read at `v1.compiler.parse` `parse_param`, `v1.compiler.resolve` `resolve_param`,
   `v1.compiler.resolve` `resolve_expr_types` and `v1.compiler.infer`'s single
   `param_node_default_value` use. Receipts:
   `docs/probes/declared_type_inhabitance_position_census_2026-08-22/measured.md`.

   RUNG FOUND AT: **below the floor** — an undefined name is accepted, which is the floor's own
   first clause. CEILING: **structurally guaranteed** — a default value is an ordinary expression
   at a position whose declared type is in hand, so routing it through inference is the same
   judgment every other position already receives.

   NEXT TRIGGER: (1) fixture the four structurally-flagged rows to CONFIRM rather than to discover
   — the denominator is closed by the read above, so this is verification and not a search; (2) a discriminating RED per member — the undefined-name arm at a parameter
   default and at a field default, each asserting a located refusal, with the accepted-default
   positive control beside it; (3) route the default expression through inference against the
   declared type, which makes both members ordinary consumers of item 28's obligation rather than
   checks of their own.

30. **WITHDRAWN BEFORE IT WAS ACTED ON: `gunbc compile` does NOT report a budget refusal as
   success — the exit status was measured through a pipe, and the pipe was the defect** (opened
   and withdrawn 2026-08-22, session quiet-boar-696; kept as a row rather than deleted because a
   withdrawn claim that leaves no trace gets re-derived by the next reader from the same evidence).

   WHAT WAS CLAIMED. That a whole-root compile refusing on `WholeCorpusCompileBudgetBelowMeasuredDemand`
   exits 0 — a refusal typed as success, the §5 shape, in a tool other sessions use daily.

   WHAT REFUTED IT. Direct measurement on a runner below the budget, exit status captured from the
   compiler itself with no pipeline in between, against two controls in the same run: the budget
   refusal exits **1**, an ordinary type refusal exits **1**, and a clean compile exits **0**. The
   admission arm is correct and this row asserts no defect in it.

   HOW THE WRONG NUMBER WAS PRODUCED, which is the part worth keeping. The original observation was
   captured as `gunbc compile ... | head -8; echo exit=$?` — `$?` there is **head's** status, and
   `head` exits 0 essentially always. So the measurement could not have returned anything but zero:
   not a wrong number, an UNINFORMATIVE one wearing a number's clothes, which is the same class as
   an instrument that can only return one answer. Use `${PIPESTATUS[0]}`, or drop the pipe.

   WHAT SURVIVES, AND IT IS THE HALF THAT ACTUALLY BROKE THE CENSUS. The refusal is loud in its
   status and silent in the shape a caller greps: it emits no `error[` line and no
   `compiled: … N diagnostics` summary, so a harness that classifies runs by scanning output — which
   is what a diagnostic census does — reads it as a compile that produced nothing. Two successive
   52-arm runs returned a complete table of zeroes that read as "no position refuses anything
   anywhere". That is an instrument obligation, not a compiler defect: classify by
   `compiled:` and by exit status, never by the absence of a marker.

   THE LESSER TWIN, likewise not a defect: a `--source-root` the process cannot use PANICS (status
   101) — `source root does not exist` for an absent path, `repo_relative_path_normalized: … is not
   under process workspace root` for a path outside the workspace. Loud, nonzero, and fatal.

   RUNG: **not on the ladder** — this is a property of a withdrawn claim, not of `main`. Recorded as
   a methodology hazard for anyone measuring a compiler's verdict from a shell.

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
