# R3 Close Interrogation — Adversarial Audit

**Authoring status**: PM-authored draft per operator directive 2026-05-13 ("come up with the final list of questions/interrogation as a sheet where we can check things off to close R3"; refinement directive 2026-05-13: "i want the doc to be antagonistic — we have to go over all the work that was done, acceptance criteria — what was promised, what was delivered"). Director ratification pending.

**Stance**: this is an ADVERSARIAL audit, not a checklist. Predicates being green is necessary but not sufficient — a gate can pass its predicate while the feature it gates is still broken. For every promise R3 made, this sheet demands the receipt:

1. **The promise** — verbatim quote from a doc, with citation
2. **The delivery** — code, test, or demo that satisfies it
3. **A concrete example** — real input → real output, end-to-end
4. **Falsification probe** — what could disprove "delivered"; has it been attempted

R3 closes when every promise has all four. Predicates and counts are evidence; they are not the verdict.

---

## §0. Disposition vocabulary

| Status | Meaning |
|---|---|
| **NOT-CHECKED** | The question hasn't been asked at HEAD yet |
| **PROVEN** | Promise verbatim cited + delivery cited + concrete example reproducible + falsification probe attempted and survived |
| **WEAK-EVIDENCE** | Promise + delivery cited but example is single-fixture, narrow, or non-end-to-end. Acceptable for R4-deferred items; blocker for R3 close |
| **GAP** | Promise exists; delivery absent OR demo doesn't run OR example doesn't reproduce |
| **R4-DEFERRED** | Promise explicitly deferred to R4 with operator acceptance recorded |
| **NOT-PROMISED** | The "expected" behavior was never promised; out of R3 scope |

A close-eligible R3 has every item PROVEN or R4-DEFERRED with operator-recorded acceptance. Zero GAP. Zero WEAK-EVIDENCE.

---

## §1. The dimension promises

### §1.1 Complexity

**Promise** (THESIS.md:87-91 + 119-123): the compiler analyses program complexity as a structural fact carried by the data model, validates by reading structure, and "rejects structural, effect, and complexity bugs that ordinary compilers never model."

**Probes**:

- [ ] Show me a `.dag` program that violates a complexity contract. Quote the code.
- [ ] Compile it. What is the verbatim error message? (`Diagnostic` with `reason` + `at`, not `panic!` or silent skip.)
- [ ] Show me a SECOND example with a different complexity class (linear/quadratic/exponential).
- [ ] Where is the test that pins the error message? Cite `<file>:<line>`.
- [ ] If I REMOVE the contract annotation, does the program still compile? (It should — the contract is the assertion.)
- [ ] If I add a LYING annotation (`complexity: O(1)` on an O(n) body), does the compiler catch the lie? Show me the diagnostic.
- [ ] Is there a demonstration `.dag` program that exercises Complexity end-to-end via the v3 evaluator? Cite path.
- [ ] Run the demonstration on clean checkout. What does stdout/stderr say?
- [ ] **Falsification probe**: write a program whose complexity is wrong in a way the lens hasn't been tested against. Does the lens catch it? Or did we only test the cases we already knew worked?

### §1.2 Cost

**Promise** (r3-program-plan.md §1.6 + §1.8 #70: `cost_lens_demonstration` + §1.8 #105: `symbolic_cost_textbook_coverage_landed`): the cost lens reads representative target programs, composes algebra+realization cost end-to-end, and produces observable cost-bound output. **R3-committed carrier scope** (operator directive 2026-05-13 + Director ratification msg_ad5e934d): Tier 1 algorithms-textbook coverage — Constant + Linear + Polynomial{degree: Rational} + PolyLog + Exponential + Factorial + Log + Sum/Product composites + UnknownCost (algebra-top floor only, NOT routine collapse target for textbook-Tier-1-coverable bounds).

**Implementation probes** (carrier scope satisfies the promise):

- [ ] Show me a `.dag` program with a cost budget. Quote it.
- [ ] Show me the symbolic-cost output the lens produces. Verbatim.
- [ ] Show me a program that EXCEEDS its cost budget. What error fires?
- [ ] Where is the test? Cite path.
- [ ] Is the cost arithmetic actually composing? Show me a multi-level program (call within call) and the symbolic-cost result.

**Scope probes** (Tier 1 textbook coverage per gate #105):

- [ ] Show me a √n algorithm (e.g., trial division primality). Cost-lens output should be `PolynomialCost { degree: 1/2 }`, NOT `UnknownCost`.
- [ ] Show me an exponential algorithm (subset enumeration). Cost-lens output should be `ExponentialCost { base: 2 }`, NOT `UnknownCost`.
- [ ] Show me a factorial algorithm (permutation enumeration). Cost-lens output should be `FactorialCost`, NOT `UnknownCost`.
- [ ] Show me a polylog algorithm (e.g., repeated binary search log² n). Cost-lens output should be `PolyLogCost { exponent: 2 }`, NOT `UnknownCost`.
- [ ] Show me a matrix-multiplication cost program (n^2.373 Coppersmith-Winograd). Cost-lens output should be `PolynomialCost { degree: 2.373 }`, NOT `UnknownCost`.

**Tier 2 boundary probes** (R4-deferred per Director ratification):

- [ ] Show me a union-find program (α(n) inverse Ackermann). Currently expected: `UnknownCost("Tier 2 — pending R4 named-variant canvas")` OR `ConstantCost-with-named-reason` per Director rationale; NOT a Tier-1-coverable bound being squelched.
- [ ] Show me a vEB-trees program (log log n). Same expectation; nested-log structure may compose via PolyLogCost — canvas surfaces if yes.

**Falsification probes**:

- [ ] Program with recursive call whose cost is bounded by a Tier-3 fact. Does the lens compute the bound or punt?
- [ ] Program with cost in a class deliberately NOT in Tier 1 (e.g., O(n^n) hyperexponential). Is `UnknownCost("reason")` actionable for the user? Does the diagnostic name the gap class?
- [ ] **STOP-SIGNAL trigger**: program with a Tier-1-coverable bound (e.g., √n) — verify the cost-lens does NOT collapse to UnknownCost post-gate-#105-landing. Pre-gate-#105: expected current behavior is UnknownCost (regression test).

### §1.3 Parallelism

**Promise** (THESIS.md:17 "Parallelism is the default because independence is visible in the structure"; r3-program-plan.md §1.6 lane T-Lens-Behavioral-Parity needs parallelism gate per Cluster F):

**Probes**:

- [ ] Show me a `.dag` program where the parallelism lens identifies independent subgraphs. Quote the program.
- [ ] Show me the lens's output — which nodes are flagged parallel, which serial.
- [ ] If I add a fake dependency, does the lens correctly serialize? Demo.
- [ ] **Falsification probe**: program with hidden state coupling that LOOKS independent. Does the lens report independent (wrong) or report state coupling (right)?

### §1.4 Effect enumeration

**Promise** (r3-program-plan.md §1.6 needs effect_enumeration gate per Cluster F): the effect lens enumerates effects (I/O, memory mutation, etc.) per program location.

**Probes**:

- [ ] Show me a program with mixed pure + I/O effects. What does the lens classify per location?
- [ ] Show me a "pure" claim on a function that secretly does I/O. Does the lens catch the contradiction?
- [ ] Where is the test fixture?
- [ ] **Falsification probe**: effect-leaks through a callback or higher-order construct. Does the lens trace it?

### §1.5 User-defined dimensions

**Promise** (THESIS.md:95-101 + docs/thesis/correctness-dimensions.md#user-defined-dimensions): "A user writes a lens in `.dag` — e.g., 'max external HTTP calls per workflow,' 'bounded memory footprint per request,' 'no cross-tenant data flow' — and the compiler validates every program against it using the same mechanism it uses for built-in dimensions."

**Probes**:

- [ ] Show me ONE user-defined lens that lives in `.dag` (not Rust). Cite the file.
- [ ] Show me a program it validates. Show the output.
- [ ] Did the user-defined lens use the same mechanism as built-in? Or a parallel infrastructure?
- [ ] **Falsification probe**: would a NEW user-defined lens (one we haven't written yet) actually work end-to-end? Author a simple one. Does it compile and run?

**Escape-hatch probes for Tier 2+ cost bounds** (per Director ratification msg_ad5e934d structural-extension caveat):

- [ ] **If a user needs O(α(n)) or O(log log n)** — bounds NOT in Tier 1 of gate #105: can they author a user-defined cost-variant lens that integrates with `SymbolicCost`'s dominance lattice + Sum/Product algebra, OR does the user-defined-dim mechanism stop at producing a separate Dimension carrier with no algebra-integration?
- [ ] **Compositional vs named-variant**: if the user-defined-dim infrastructure supports compositional cost-bound extension (e.g., authoring a `LogLogCost` lens that participates in dominance), R3 may NOT need named-variants for Tier 2 (per Director caveat: "if Substrate Mgr canvas surfaces a compositional mechanism for Tier 2 that satisfies feedback_groundedness_gates_lenses + composes with Sum/Product algebra + carries consumer-evidence justification, accept that mechanism IN-R3 instead of named variants"). Test this end-to-end.
- [ ] **Falsification probe (Tier 2 escape hatch)**: write a user-defined cost lens for a textbook-known bound that's R4-deferred (e.g., inverse Ackermann). Does it integrate? If yes — Tier 2 R4-deferral is structurally bounded. If no — user-defined-dim promise has a load-bearing gap; surface as Director-tier scope question.

---

## §2. The substrate promises

### §2.1 Pure Bootstrap (zero hand-Rust)

**Promise** (r3-program-plan.md §1.8 gates #8 + #84; project memory `feedback_pb_zero_is_r3_close_target`): "R3 close = 0 hand-Rust + 0 TESTING residual per §1.8 gates #8 + #84."

**Probes**:

- [ ] Run the PB-0 census predicate. What's the count? Should be 0.
- [ ] If non-zero: enumerate each survivor. Justify each as named-retirement-schedule.
- [ ] If zero: when was census last run? Cite SHA.
- [ ] Compile gunbc with itself end-to-end. Does the output match a reference build? Bit-for-bit, or just behavioral?
- [ ] **Falsification probe**: write a new compiler feature in pure `.dag` (no Rust). Does it self-host?

### §2.2 Closed system / no escape hatches

**Promise** (THESIS.md:23 + INVARIANTS.md): "`.dag` is designed as a closed system: bounded data, bounded iteration, and composition that preserves those bounds." Memory `feedback_groundedness_gates_lenses`: "no escape syntax; lenses apply to every program by construction."

**Probes**:

- [ ] grep for `unsafe` in `src/v3/`. Count: 0 expected.
- [ ] grep for `panic!()` outside narrow boundaries. Where are the survivors? Justify each.
- [ ] grep for annotations (`#[...]` on user-facing constructs). Should be zero per `feedback_no_annotations`.
- [ ] grep for metadata markers (`__is_X` strings). Should be zero per `feedback_no_metadata_markers`.
- [ ] **Falsification probe**: write a program that tries to "leave the stack" via an opaque-string back-channel. Does the compiler accept it?

### §2.3 Single authority / cost-of-change = 1

**Promise** (CLAUDE.md): "When the language grows by one type, one expression, or one transport, how many files need editing? The answer should be 1."

**Probes**:

- [ ] Add a new built-in type to the language. How many files change? Show the diff line count per file.
- [ ] Add a new expression form. Same question.
- [ ] Add a new emission target (or extend an existing one). Same question.
- [ ] If the count is >1 for any case: name the files, explain WHY they need editing.
- [ ] **Falsification probe**: pick a recent PR that added a feature. Re-do the change. Was it ACTUALLY 1 file, or were there hidden auxiliary edits?

### §2.4 Fail-closed discipline

**Promise** (INVARIANTS.md C-8; project memory `feedback_fail_closed_discipline`): "every detectable problem is a Diagnostic; no warnings, no silent Nones, no panics."

**Probes**:

- [ ] grep for `Option::None` returns in lens read channels. Should be zero (replaced with `Witness<C>::Violates`).
- [ ] grep for `unreachable!()` outside truly-impossible-by-construction paths. Justify each survivor.
- [ ] Trigger an error condition at runtime. Is the result a `Diagnostic` with `reason: <named>` + `at: <port_or_node>`, or something less structured?
- [ ] **Falsification probe**: write a malformed program. Does the compiler emit a clear diagnostic that points to the structurally correct alternative (per THESIS.md:103-105 "Show the correct code")?

### §2.5 Impossible bugs by construction (THE META-PROMISE)

**Promise** (THESIS.md:23 closed system + :120 "rejects structural, effect, and complexity bugs that ordinary compilers never model" + **INVARIANTS P4 Decidability** "Every accepted program stays within a closed, fail-closed system whose correctness questions are structurally decidable" + INVARIANTS P3 Fail-Closed + `feedback_closed_system_design` + `feedback_groundedness_gates_lenses`): the language's design makes whole classes of bug **impossible by construction** — not via runtime checks or static analysis added after the fact, but by the substrate refusing to admit the bug-shape. This is the META-claim that ties §1 (dimension promises) + §2.1-§2.4 (substrate promises) together. The structural anchor is **P4 Decidability**: closed-system + structurally-decidable-correctness-questions are precisely what makes the bug-class-impossibility claim cash structurally rather than rest on case-by-case enforcement.

The claim is sharp on some bug classes and softer on others. This section interrogates BOTH sides honestly.

**§2.5.A Probes — What "impossible" actually means**:

- [ ] Enumerate the bug classes the architecture claims are impossible **against canonical authority** (THESIS.md:370-413 "Enumerable impossible-bug classes" + ROADMAP.md:93 T-Demo R1/R2+ split — NOT a PM-fabricated count). At HEAD, THESIS commits to: **[R1]** Suboptimal-complexity contract violation + Idempotency-contract violation + Transport/type drift; **[R2+]** Nested-optional flatten + Unenumerated effects + Unhandled diagnostic paths. R3 close audit MUST verify against the canonical THESIS enumeration, NOT a session-author guess. For each [R1] class: cite the substrate fact that prevents it + the demo fixture (per ROADMAP T-Demo). For each [R2+] class: confirm R4-DEFERRED disposition per §0 vocabulary with operator-recorded acceptance. **Anti-pattern**: hard-coding a candidate list at audit-author time rather than deferring to THESIS authority — `feedback_thesis_gate_state_drift`-class miss; classes may have been added/removed in THESIS since the audit was written.
- [ ] For each: write a `.dag` program that ATTEMPTS the bug-class. Does the compiler refuse to compile? Show the diagnostic.
- [ ] **Discrimination probe**: is "impossible" actually "(a) impossible to express in surface vocabulary" OR "(b) caught at compile time by lens"? They differ. (a) means the bug-shape has no syntactic form. (b) means the bug compiles syntactically but fails a check. Both prevent the bug at user-visible level; only (a) prevents the bug-shape from existing in the substrate at all.
- [ ] **Compiler-correctness gating probe**: if the prevention is "caught at compile" (b-shape), what happens if the compiler itself has a bug? Is the prevention then defective? How is the compiler's correctness gated? (R3 close pointer: PB-self-compile fixed point + lens self-application.)
- [ ] **Falsification probe**: name ONE bug class the architecture historically claimed was "impossible" but turned out to have an instance. What was the cause — structural gap, lens gap, modeling error, or implementation bug?

**§2.5.B Probes — Glue bugs** (most concerning class — interface boundaries between subsystems):

"Glue" here = the interfaces between substrate ↔ emission target, evaluator ↔ host, lens A ↔ lens B output, v3 substrate ↔ stage0 Rust. The architecture's "impossible by construction" claim is STRONGEST inside the substrate and WEAKEST at glue boundaries.

- [ ] **Glue #1 — substrate (`.dag`) → emit target (Rust/Python/Go)**: How does the system PROVE that emitted target output faithfully realizes substrate semantics? Cite the proof. Show a concrete example. (R3 anchor: §3.1 L4 runtime equivalence + §3.1 L6 EmissionPathProjection data-coverage.)
- [ ] **Glue #2 — emitter ↔ runtime**: even if substrate is correct + emission faithful, runtime behavior (memory layout, scheduler, OS, language-runtime) may diverge. Where does the architecture handle these vs. assume them? Where does "impossible by construction" become "trust the runtime"?
- [ ] **Glue #3 — lens composition** (lens A output → lens B input): if lens A produces `WitnessA<C>` and lens B reads it as input, what enforces structural compatibility? (R3 anchor: `Witness<C>` shape lock + `Lens<C>` per-Behavior typed channel per design-lens-framework.md.)
- [ ] **Glue #4 — bootstrap** (v3 substrate ↔ stage0 Rust): if stage0 has a bug that affects v3 substrate generation, is the bug caught by `pb_self_compile_fixed_point`? If stage0's generated code is wrong but compiles cleanly, what catches it? (R3 anchor: gate #16 self-compile fixed point + gate #18 numeric_width_refinements.)
- [ ] **Glue #5 — `ExecuteCommand` PB-Runtime boundary**: external-toolchain boundary tests run via `ExecuteCommand`. The boundary itself is opaque to the lens framework. Is THAT a glue bug class, or is "calling external thing is by-definition outside scope"?
- [ ] **Falsification probe**: identify a glue boundary in the current architecture with NO structural enforcement of correctness — just convention, comment, or runtime assertion. List 3 concrete examples at HEAD. Is each an "impossible bug" class, or a class that's still possible just by being at an interface?

**§2.5.C Probes — User error** (user authoring a `.dag` program that's syntactically valid + lens-compliant but semantically wrong for their intent):

This is the "wrong specification" class. The compiler can verify a `.dag` program against ITS contracts (lenses, types, algebra), but the contracts themselves are user-authored.

- [ ] **Intent vs. spec**: write a `.dag` program that compiles cleanly + satisfies ALL lens contracts + does something the user obviously didn't intend (e.g., `sort_descending` named but body is `sort_ascending`; both type-correct). How many "impossible bug" classes does this hit?
- [ ] **Wrong contract**: if the user's CONTRACT (Lens enforcement budget, complexity annotation, effect declaration) is wrong, the compiler accepts compliance with the wrong contract. How is the CONTRACT'S correctness checked? Or is that meta-level out-of-scope by design?
- [ ] **Empty program**: an always-correct but useless program (`fn main = unit`). Does the system distinguish "no work" from "intended no work"? Or is "user wrote what they meant" axiomatic?
- [ ] **Spec-as-program collapse**: when spec and implementation are the same artifact (.dag), is the user error "wrote wrong spec" identical to "wrote wrong program"? Does the architecture's "no parallel authority" discipline mean user-error is single-point-of-failure rather than divergence-detectable?
- [ ] **Falsification probe**: enumerate 3 concrete user-error classes the architecture can NEVER catch by construction. Confirm that "impossible bugs by construction" is shorthand for "structurally-defined bug classes are impossible", not "all user errors are impossible".

**§2.5.D Probes — Emergent behavior** (composition of correct individual parts producing surprising aggregate behavior):

"Emergent" = pieces that are individually correct but compose into surprising aggregate behavior. The architecture's lens framework is strong on per-program facts; emergent claims need explicit treatment.

- [ ] **Lens-composition emergent**: lens A + lens B individually correct + composed in unexpected order. Show 2 lenses where composition order MATTERS; demonstrate ordering effect on output. (R3 anchor: `Lens<C>` typed per-Behavior channels per design-lens-framework.md — is ordering structural or behavioral?)
- [ ] **Scale emergent**: 1000-program corpus where each is individually correct but together produce a performance / cost / behavior surprise (e.g., aggregate cost dominates per-program cost; cross-program coupling). What's the architecture's tooling for catching scale-only bugs?
- [ ] **Time-evolving emergent**: substrate at time T compiles cleanly + at time T+1 (with new dependencies / extdeps revision) compiles differently or with different cost. Is the divergence caught? (R3 anchor: `feedback_thesis_gate_state_drift` — gate state drifts over time.)
- [ ] **Lens-set silent gap**: a 2-lens composition where the composition reports "all lenses green" but the actual program behavior is wrong because a third dimension was needed (and not modeled). Does the architecture's "lenses are folds over physics" claim catch THIS, or does "all modeled lenses green" silently mean "the modeled dimensions are individually green; jointly silent on unmodeled dimensions"?
- [ ] **Falsification probe**: construct a `.dag` program where 4 individual lens claims (complexity, cost, parallelism, effect) all pass cleanly + the program is observably wrong (e.g., wrong arithmetic, wrong I/O sequence). How many such constructions exist? Is the architecture's response "model the missing dimension" or "user error is out-of-scope"?

**§2.5.E Probes — Cross-module + cross-target subtle bugs (the omni-emission story)**:

The class operator framed 2026-05-13 as the most personally interesting + the strongest omni-emission story: bugs between DISPARATE MODULES (cross-module within same language) and CROSS-EMISSION-TARGET (Rust ↔ JavaScript ↔ Python via shared .dag substrate). Traditional compilers see modules as opaque link-units with name/signature contracts and have ZERO visibility across emission-target boundaries; gunbc's lens framework reads ALL substrate facts globally, so cross-module / cross-target composition is structurally tractable.

This sub-section enumerates the class with concrete shapes. The promise is **NOT** that all these are caught at HEAD R3 today — the promise is the architecture admits structurally-decidable enforcement (per P4) and the gates check the modeling exists. R3 close audit verifies which classes are mechanized + which remain at the modeling level pending demonstration.

**Cross-module bug shapes** (within same emission target):

- [ ] **Cross-module effect-leak through "pure" boundary**: Module A declares `compute_stats(x)` as pure. Module B's helper it calls writes to disk. Traditional compilers catch only if Module A is RE-ANNOTATED. gunbc's effect lens folds over the full Dag — Module B's I/O propagates to A's call sites structurally without annotation maintenance. (R3 anchor: gate #82 `effect_enumeration_lens_behaviorally_complete`.)
- [ ] **Cross-module cost-composition emergence**: Module A wraps `each(items, |x| Module_B.process(x))` thinking it's O(n). Module B's `process` is O(n²) internally. Traditional: zero visibility into composed cost. gunbc: cost lens reads algebra + realization across modules — `Cost(A.each) = Cost(B.process) · n` falls out structurally. (R3 anchor: gates #70 `cost_lens_demonstration` + #105 `symbolic_cost_textbook_coverage_landed`.)
- [ ] **Cross-module dimensional drift / unit confusion**: Module A's `Time` is "ms since epoch"; Module B's `Time` is "ns since epoch". Both ground to `i64`. Traditional: type system doesn't carry the unit. gunbc: dimensional fact lives in the algebra/witness — `Time<MS>` and `Time<NS>` are structurally different even when both ground to the same primitive.
- [ ] **Cross-module ordering / sequencing assumption**: Module A: `validate(x); save(x)`. Module B refactors to `save(x); validate(x)`. Traditional: zero structural way to express ordering invariant. gunbc: the Dag IS a structural ordering; "save must follow validate" is a relation on Node identities.
- [ ] **Cross-module callback effect-set drift**: Module A's `with_lock(callback)` assumes callback is pure. Module B's callback later acquires another lock → deadlock class. Traditional: callback contracts are convention. gunbc: callback's effect-set is structurally readable by lens framework.
- [ ] **Cross-module aliasing / shadow definition**: Module A: `const MAX_RETRIES = 3`. Module B re-declares `MAX_RETRIES = 5`. Traditional: name collision detected only at link time. gunbc: P1 single-authority refuses parallel definitions structurally — second declaration is a compile-time INVARIANTS-P1 violation.
- [ ] **Cross-module data-flow capability leak**: Module A's `read_secrets()` returns `Secret<String>`. Module B's caller strips into plain `String` via implicit conversion. Traditional: capability typing if you have it (most don't). gunbc: Secret is a structural carrier in the algebra DAG; downcasting requires explicit lens application, structurally trackable.

**Cross-emission-target bug shapes** (the omni-emission story):

When the SAME `.dag` substrate emits to Rust + JavaScript + Python (3 R3 Shape-A targets per §3.1), cross-target bugs are structurally impossible for shapes that derive from substrate. But cross-target bugs at the GLUE layer (target-specific realization fidelity) are a real concern. Operator's framing: "seeing bugs between JavaScript and Rust" — concrete shapes:

- [ ] **Cross-target serialization round-trip**: Module A (emitted Rust) sends `User { id, name, email }` over wire to Module B (emitted JavaScript). Both emissions derive from the same `.dag` substrate. Field rename in `.dag` → both emissions update structurally; no schema-drift class possible. **Falsification**: rename a field in `.dag`; verify both Rust + JS emissions update; observe any cross-target consumer that didn't rebuild.
- [ ] **Cross-target numeric width**: `.dag` declares `Counter: Nat<32>`. Rust emits `u32`, JavaScript emits `number` (53-bit safe-int). At `Counter > 2^32`, JS overflows silently; Rust wraps/panics. Traditional: zero cross-language analysis. gunbc: cost-lens + dimensional carrier reads BOTH emissions' realization cost — JS's `number` carries different overflow semantics than Rust's `u32`, structurally expressible per dim-substrate (R3 anchor: gate #18 `numeric_width_refinements_landed` + Q-MachineConstraint-Carrier).
- [ ] **Cross-target effect divergence**: `.dag` declares an `async` operation. Rust emits via `tokio` futures; JavaScript emits via `Promise`; Python emits via `asyncio`. Cancellation semantics differ. Traditional: each emission target writes its own async-handling; cross-target tests catch only late. gunbc: substrate models async as structural fact; per-target LanguageSpec encodes realization; cost-lens reads composition.
- [ ] **Cross-target boundary trust**: Rust service calls JavaScript via FFI / WASM / HTTP. Type marshaling at the boundary. Traditional: protobuf-like schema at best (post-hoc). gunbc: BOTH ends derive from same `.dag` declaration; marshaling is structural emission per target.
- [ ] **Cross-target test-claim transferability**: `.dag` TestClaim asserts behavior X. Rust emission runs it via `cargo test`; JavaScript via `jest`; Python via `pytest`. All three should pass-or-fail identically for the SAME claim. Traditional: tests are language-specific; cross-language test-claims don't exist. gunbc: per gate #15 `l5_cross_target_consistency` — for every `.dag` program, emitted Rust/Python/Go produce equivalent runtime behavior on the certification corpus. (R3 close anchor — see §3.1 for L5 status.)

**Falsification probes for cross-target class**:

- [ ] **Pick a `.dag` program** that compiles + lens-passes cleanly + emit to all 3 R3 targets (Rust/Python/Go). Run the same fixture through each. Do outputs agree on every (input, output) pair? Where they don't agree, is the divergence: (a) a target-realization fidelity gap caught by L5 lens, (b) a substrate ambiguity (modeling-level error), or (c) a target-LanguageSpec drift (extdeps issue)?
- [ ] **Construct a cross-language wire scenario**: emit Rust client + JavaScript server from the same `.dag` substrate. Have client send `User { ... }` to server; verify field-rename propagation, type-marshaling, async-cancellation semantics. Does the structural emission preserve all the facts the lens framework reads?
- [ ] **Modeling-level cross-target gap**: identify a target-language-specific bug class (e.g., JavaScript's prototype-pollution, Rust's lifetime-pin-projection, Python's GIL-aware threading) that does NOT have substrate representation. Confirm the architecture's response: "model the missing dimension" or "target-specific gap is in extdeps lane".

**The cross-module / cross-target story** (PM-derived, for thesis-pitch):

This class is one of the strongest "bugs impossible by construction" stories because traditional compilers genuinely have ZERO visibility:
- Inside same compilation: modules are linked via symbol tables — opaque
- Across emission targets: each target's compiler is independent — no shared semantic substrate
- Across the wire / FFI: schemas (protobuf, OpenAPI, JSON Schema) are external authority files — drift class

gunbc's substrate-shared / emission-as-projection architecture means:
- Module boundaries are naming partitions, not semantic firewalls
- Emission targets are projections of the SAME Node tree
- Cross-target structural facts (effects, costs, parallelism, types) flow forward to all emissions
- Wire schemas are derived FROM substrate, not maintained alongside

**R3 close audit for this class** (PM-recommended): demonstrate ONE end-to-end cross-target scenario (e.g., `.dag` declares a Service with typed request/response → emit Rust server + JavaScript client → demonstrate field-rename propagation + L5 stdout-parity). The omni-emission demo (gate #28 `omni_layers_share_one_node_tree` + gate #15 `l5_cross_target_consistency`) is the closest existing structural anchor; a cross-language wire demo would cash the story most viscerally.

**The architectural honest answer** (PM-derived, surfaced for Director ratification):

The "bugs are impossible by construction" claim is **SHARP** on:
- **Structural facts absent from user-surface vocabulary** (no annotations → no annotation-rot; no escape hatches → no escape-hatch-leak; no metadata-string markers → no string-tag-collision)
- **Lens-mediated dimension violations within the modeled set** (complexity / cost / parallelism / effect bugs caught at compile via lens read, FOR THE PROGRAMS where lenses apply structurally)
- **Single-authority class** (no parallel substrate → no divergence-between-mirrors class)
- **Closure-bound discipline** (closed system, bounded iteration, no `unsafe` outside narrow boundary)

The claim is **LESS SHARP** on:
- **Glue layers** (interface boundaries; emission target realization fidelity; lens composition; PB-Runtime boundary)
- **User intent vs. specification** (the compiler enforces what's written; "user meant" is out of scope)
- **Scale-only emergent behavior** (individual-correctness doesn't compose to system-correctness automatically)
- **Modeling-level errors** (the model itself can be wrong; e.g., wrong complexity classification leads to lens reading the wrong fact)
- **Unmodeled dimensions** (a bug class outside the 5 modeled behaviors / 4 in-R3 lenses is invisible to "all lenses green")
- **Compiler-correctness self-reference** ("compiler catches X" is only as strong as the compiler's correctness; loop-closes via PB-self-compile + lens-self-application but isn't a structural fact at user-visible level)

**Honest R3 close framing** (PM-recommended for Director ratification):

R3 close SHOULD NOT claim universal impossibility. R3 close SHOULD claim:
- **Closed-set bug-class impossibility** for the explicitly modeled dimensions + substrate disciplines, with the modeled set enumerated
- **Reduction-to-glue-boundary** for bugs at interfaces — glue boundaries are themselves probe-able, but the impossibility-by-construction claim does NOT extend to them by default
- **User-intent out-of-scope** acknowledged: the architecture verifies SPECS, not INTENTS; user-error class is by-design outside the impossibility claim
- **Emergent-behavior probes** as PM-curated R3-close-evidence: the lens-composition / scale / time-evolving / unmodeled-dimension surfaces have probe expectations defined here

**Anti-pattern**: "all lenses green = bug-free program" is the silent-impossibility-claim. R3 close framing should explicitly NAME the bug classes that remain possible (user-intent, unmodeled-dimension, glue-boundary, modeling-error, emergent-composition) rather than let "lenses green" carry the universal claim implicitly.

#### §2.5.F Cross-module subtle-dependency detection via affected-set lens

**Promise** (`docs/design-affected-set-lens.md` + r3-program-plan.md §1.8 gate #103 `ci_uses_affected_set_selection`): the affected-set lens is the **structural mechanism** for catching cross-module subtle dependencies — not value-level "did the return change," but **dimension-parameterized** "did any structural dimension (complexity / cost / effect / value) the consumer reads change." The aggregate affected-set is the union across all dimensions; if a helper's *cost* changes while its *return value* stays the same, downstream consumers reading the *cost* dimension still flag as affected.

**Why this is the answer to §2.5.E cross-module bugs**:

§2.5.E enumerated 7 cross-module + 5 cross-target bug shapes (shared-vocabulary mismatch, memory-layout assumption, async-semantic divergence, serialization round-trip, numeric width, etc.). The affected-set lens is what mechanically *catches* the dynamic / change-driven sub-classes of those: when module A changes in a way that affects module B's structural read of A, B is in the affected-set. The static / single-snapshot classes (e.g., shared vocabulary at a moment in time) are caught by other R3 lenses (complexity, cost, effect); the affected-set lens specifically catches the **diff-driven** propagation.

**Status at HEAD** (r3-program-plan.md §1.8 + docs/r3-remaining-work-dependency-graph.md):

- Gate #103 `ci_uses_affected_set_selection` — **DECLARED** (NEW 2026-05-12 per PR #2744 §1); R3-load-bearing
- Slice 7 in T-WAD lane sequencing: Slice 4 (#100 substrate) → Slice 5 (#98 ci.yml swap) → Slice 7 (#103 affected-set) → Slice 8 (substrate completion)
- Design doc: `docs/design-affected-set-lens.md` — substrate-shape ratified; consumer pattern is CLI / agent / IDE invoking `IntrospectApplication`-carrier lens with `Set<NodeRef>` output

**Probes** (post-gate-#103-CONSUMER_LANDED):

- [ ] Where does affected-set live in CI/build? Cite the workflow file + the invocation site. Is it `gunbc query affected-set --since=main`?
- [ ] What's the SHA-diff input shape? `(Dag_before, Dag_after)` per design doc, or a different surface?
- [ ] Show me a concrete subtle dep that **ONLY** the dimension-parameterized variant catches — a case where value-only-diff would miss the affected-set but cost-or-complexity-diff catches it. Cite the test fixture.
- [ ] Run affected-set on the last 5 merged PRs. What's the cardinality of the affected-set per PR? (Should be much smaller than transitive-downstream — that's the structural progress claim.)
- [ ] Show me a PR where the affected-set predicted a test would run, AND the test caught a regression that value-only-diff would have skipped.

**Falsification probes**:

- [ ] **Cost-regression-in-uncovered-helper**: introduce a complexity regression in a helper (e.g., O(n) → O(n²)) that no test directly covers. Does affected-set flag every downstream consumer reading the cost dimension? Or does it miss them because no test directly asserts cost on the helper?
- [ ] **Effect-leak-across-module-boundary**: add a side effect (I/O) to a previously-pure function. Does affected-set flag the consumers asserting purity? Or only the consumers calling the function directly?
- [ ] **Cross-module dimension narrowing**: change function F's cost from `O(n)` to `O(n log n)` in a way the lens classifies correctly. Does the affected-set surface every transitive consumer reading the cost dimension, AT LEAST as far as the dimension flow propagates?
- [ ] **PB-Runtime opacity**: introduce a change inside PB-Runtime (the bounded kernel) that affects a structural-dimension claim. Does the affected-set lens correctly identify the dimension-flow boundary at PB-Runtime, or does it under/over-propagate?

**Open questions for ratification** (PM-surfaced):

- Is the dimension-parameterized affected-set the **PRIMARY** R3-close artifact for catching cross-module subtle deps, or is it complementary to other mechanisms (per-dimension lens read at compile-time)?
- Does R3 close require gate #103 CONSUMER_LANDED + PASSING, OR is DECLARED status with CI-integration deferred to the slice cascade?
- Are the 4 in-R3 lenses (complexity / cost / parallelism / effect_enum) sufficient dimensions for the union, or does the affected-set need to surface user-defined-lens dimensions too?

**PM read** (provisional): the affected-set lens is THE structural cash for cross-module subtle-dep detection — without it, the §2.5.E cross-module bug classes are theoretically-impossible but operationally-unverified. With it, the static (compile-time) lens read + the diff-driven (affected-set) lens read compose to give *both* "this single snapshot is consistent" AND "this change preserves consistency." That's the omni-correctness story the operator's directive 2026-05-13 was probing.

---

## §3. The emission promises

### §3.1 Omni-emission (R3 = 3 Shape-A targets: Rust / Python / Go)

**Promise** (THESIS.md:115 omni-emission claim + THESIS.md:180 L5 + r3-program-plan.md:185 gate #15 `l5_cross_target_consistency`): "automatic parallelism, memoization, omni-emission ... are consequences of the same structural commitments." R3 scope is **Rust + Python + Go** (Shape A — programming-language targets per THESIS.md:215). C/C++ are **R4.A**-scope (WISHLIST.md:67-73, operator ratification 2026-05-12), not R3. LLVM IR / assembly / machine-code are **R4.C**-scope (WISHLIST.md:107), not R3. Shape B targets (Markdown / OpenAPI / data-shapes) are tracked separately under T-Omni-Shape-B (r3-program-plan.md:430).

**Probes**:

- [ ] Show me a single `.dag` program. Emit it to all 3 Shape-A targets (Rust / Python / Go). Are the outputs runnable?
- [ ] Behavioral parity: same input through each output target. Do they produce equivalent results? (THESIS.md:180 L5 claim.)
- [ ] Pick the most complex Shape-A target (Go's goroutine substrate? Python's dynamic dispatch?). Is there a non-trivial example that compiles and runs?
- [ ] **Falsification probe**: program with feature X. Does target Y handle X correctly, or punt to a stub?

**Findings at HEAD (2026-05-13)**:

- **Target substrate inventory** (`src/v3/spec/`): `rust.dag`, `python.dag`, `go.dag` — 3 declared targets matching R3 L5 scope. C/C++/LLVM/assembly correctly absent (R4-scope).
- **L6 data-coverage substrate** (`src/v3/std/cross_target_coverage.dag`): 41-row `emission_path_projections` over (target × form × behavior) cross-product, populated for Rust/Python/Go. Phase-1 carrier-only per Director ratification gunbc#828 2026-05-05.
- **L4 runtime equivalence at HEAD** (oracle-style stdout-parity: compiled emit-target binary stdout = expected fixture stdout; **not** literal artifact byte-equality):
  - **CI integration-binary execution state** (`.github/workflows/ci.yml:478-501`): integration harness is **prebuilt** in CI but **execution is HOT-FIX-SKIPPED** via `__HOT_FIX_NONEXISTENT_FILTER__` (zero tests selected) per operator directive at gunbc#846 ("cut all demos and integration tests for now, get v3 to 10 minutes"). Lane2d (Stage 2d integration module, `m1_5_testgen`-class) also HOT-FIX-SKIPPED at `ci.yml:385`. Restore criteria at `ci.yml:501`: OnceLock/cached_compile amortization → per-test wall ≤ 2s ratchet → re-enable per cluster filter (replace nonexistent-filter with cluster-specific filter). **Until restore, in-CI L4 evidence at HEAD is: integration-binary compilation passes (structural exercise of emit code paths) + non-integration test surfaces (lib + bins + determinism_test + doc) only.**
  - **Rust**: per-fixture stdout-parity tests EXIST + are unconditional in source (`src/v3/compiler/tests/boundary/m1_3_emit_rust_test.rs:995–1009` and following — `rustc_roundtrip_*` family) BUT live in the integration binary that is HOT-FIX zero-test-filtered above; **execution at HEAD is local-only** (`cargo test -p v3-compiler --test integration rustc_roundtrip` ... locally). Full-matrix `emit_rust_fixtures_rustc_green` at `#[ignore]` (lines 735, 764, 1199, 1218) — local-only, toolchain-gated.
  - **Python**: roundtrip tests at `#[ignore]` (`m1_4_emit_python_test.rs:1003, 1070`) — toolchain-gated (python3); local-only.
  - **Go**: roundtrip tests at `#[ignore]` (`m1_3_emit_go_test.rs:252, 279, 324`) — toolchain-gated (go); local-only.
  - **Omni demo**: Rust-only slice (`emit_omni_demo_rust_roundtrip` at `m1_5_emit_omni_demo_test.rs:106`) **unconditional in source** BUT also lives in integration binary under the HOT-FIX zero-test-filter — execution at HEAD is local-only. Full 3-target receipt (`emit_omni_demo_fixtures_green` at `:125`, Rust + Python + Go) at `#[ignore]`, requires go + python3 toolchains.
- **L5 corpus status** (gate #15 `l5_cross_target_consistency`): DECLARED, RED at HEAD (r3-program-plan.md:243 + :431) — waits on L4 corpus + Shape A grounding ready.

**Open R3 question (PM-surfaced, not yet routed)**:

What's the close-shape for the omni-emission promise?

- **(a) L6 data-coverage interpretation**: 41 (target × form × behavior) rows declared in v3-side data = ✓ for Rust/Python/Go. Structural-fold property, no runtime evidence needed.
- **(b) L4 runtime stdout-parity interpretation**: compiled emit-target binary stdout equals expected fixture stdout across the corpus. At HEAD this is **runnable locally** across all 3 targets (Rust unconditional + Python/Go via `--ignored`) but **not actually executed in CI for any target** because the integration binary is HOT-FIX zero-test-filtered (`ci.yml:478-501`). Three sub-shapes: (b1) restore integration harness CI execution per `ci.yml:501` restore-criteria before R3 close (depends on OnceLock/cached_compile amortization landing), (b2) explicit acceptance that R3-close evidence-bar is "data-coverage + local-runtime + integration-binary-compilation-passes" with full CI runtime tied to a separate fast-follow gate, (b3) gate-class promotion: add the integration-harness-execution-state restore to a new §1.8 gate explicitly anchored to R3-close.

The two interpretations differ on whether locally-executable runtime roundtrips count as R3-closure-evidence when CI doesn't actually run them. THESIS.md:180 ("L5: **same .dag produces same behavior** in Rust/Python/Go") reads as runtime-shape; current **in-CI** evidence is L6 data-coverage-for-all-three + integration-binary-prebuild-passes; current **runnable** evidence is per-fixture Rust + (with toolchains) Python/Go locally.

**Falsification candidate** (under interpretation (b) only): pick a non-trivial fixture with Python-specific or Go-specific structural sensitivity (e.g., variant-with-payload pattern-matching, fold composition). Run roundtrip locally via `cargo test ... -- --ignored`. Does it pass without toolchain-skip? If not, that's the gap shape.

### §3.2 Workflow-as-data

**Promise** (r3-program-plan.md §1.8 #82 / Cluster F: `ci_workflow_modeled_as_dag`): CI workflow is itself a `.dag` program, not hand-authored YAML.

**Probes**:

- [ ] Show me the `.dag` source for the CI workflow. Cite path.
- [ ] Where does the YAML get generated from the `.dag`? Show the emitter.
- [ ] When the CI runs, is it reading from a generated YAML, OR is `.dag` directly executed by some runner?
- [ ] If the `.dag` source is changed, does the YAML auto-regenerate? Show me the regen path.
- [ ] **Falsification probe**: change the `.dag` source, leave the YAML stale. Does CI fail? Or silently pass?

### §3.3 Tests-as-data

**Promise** (r3-program-plan.md §1.8 gate `tests_as_data_demonstration`): tests live in `.dag` as `TestClaim` data, not as hand-Rust behavior assertions.

**Probes**:

- [ ] How many tests are ported to `.dag` TestClaim form? Count.
- [ ] Pick one. Show the TestClaim source. Show it executing.
- [ ] Where's the runner? Is it itself `.dag` or hand-Rust?
- [ ] **Falsification probe**: add a new TestClaim entirely in `.dag`, no Rust. Does it run?

### §3.4 Full-stack-from-one-`.dag` — visceral 4-layer omni-emission + R4 framework substrate (FORWARD POINTER)

**Status**: STUB — pointer only. Substantive Q-dispositions land post-canvas-ratification per Director msg_428b032e + operator directive 2026-05-13.

**Thesis-probe framing**: a single `.dag` program generates a coherent full-stack application — Rust backend + SQL DDL schema + OpenAPI spec + Markdown docs — all sharing one Dag, all guaranteed coherent by gate #28 `omni_layers_share_one_node_tree`. This is the visceral cash of omni-emission as a thesis: not "we have 3 backends," but "you write one program, you get four artifacts, they cannot diverge by construction."

**Structural cash at HEAD** (r3-program-plan.md §1.8, Director-verified 2026-05-13):
- Gate #25 `omni_openapi_backend_emission_demo` — **CONSUMER_LANDED + PASSING** (runnable Rust backend)
- Gate #26 `omni_documentation_drift_lock_demo` — **CONSUMER_LANDED + PASSING** (Markdown drift-lock)
- Gate #27 `omni_sql_ddl_alternative_demo` — **CONSUMER_LANDED + PASSING** (SQL DDL projection)
- Gate #28 `omni_layers_share_one_node_tree` — **CONSUMER_LANDED + PASSING** (Rust + OpenAPI + Markdown + SQL DDL share one Dag)

**Active artifacts** (in flight pre-R3-close):
- **Path (a)** — pre-R3 visceral demo: one `.dag` (TODO-service) exercises the 4 existing emitters, lands 4 human-visible artifacts + integration test pinning gate #28 invariant at demo scope. Director-direct work-item `adhoc-e9bb6ef1-b4d`.
- **Path (b)** — pre-R3 R4 canvas: `docs/design-r4-full-stack-omni-emission-canvas.md` (Substrate Mgr authoring; 5-Q Director framing covers TS LanguageSpec carrier shape / React-as-framework-substrate carriers / ingest direction / cross-target consistency / Cluster F lens composition).

**Probes** (deferred to post-canvas-ratification):
- [ ] Demo: one `.dag` → 4 artifacts (Rust + SQL DDL + OpenAPI + Markdown). Show them. Show coherence test passing.
- [ ] Canvas: 5 Director Q-dispositions ratified. Cite anchor.
- [ ] Falsification probe: introduce a divergence between any two of the 4 emitted artifacts at the substrate level. Does gate #28 catch it?
- [ ] R4 thesis-pitch: extend story to TS client + React frontend. What's the smallest structural extension?

---

## §4. The self-application promises

### §4.1 Lens self-application

**Promise** (r3-program-plan.md §1.8 `lens_self_application_demonstrated`): the compiler's own lenses analyze the compiler itself.

**Probes**:

- [ ] Run the complexity lens on the compiler's own source. What does it report?
- [ ] Run the cost lens on a self-host pipeline. Show the symbolic cost.
- [ ] If the compiler has a complexity contract violation against itself, does it fail to compile?
- [ ] **Falsification probe**: introduce a known complexity-violation into the compiler. Does the self-application lens catch it before tests do?

### §4.2 Self-host fixed point

**Promise** (r3-program-plan.md §1.8 #16: `pb_self_compile_fixed_point`): compiler compiles itself, the output compiles itself, and the result is identical (bit or behavioral).

**Probes**:

- [ ] Run the fixed-point pipeline. Does iteration N == iteration N+1?
- [ ] How many iterations to converge? (Should be ≤2.)
- [ ] If divergent: what's the byte-diff or behavior-diff?
- [ ] **Falsification probe**: introduce a non-deterministic compiler step. Does the fixed-point predicate catch it?

---

## §5. The closure-criteria promises

### §5.1 5 substrate-gap classes closed

**Promise** (r3-program-plan.md §1.4 + §4): five substrate-gap classes (parser/grammar, function-valued data, file-ingestion, workflow/scheduling, reflection-closure) closed per Brian-ratified scope 2026-05-06.

**Probes** (per class):

- [ ] Class 1 (parser/grammar #60): show concept-faithful `Int<64>`, `Real<64>`, `Nat<8>` lower without v2-fallback. Where? Demo.
- [ ] Class 2 (function-valued #61): show a function-valued data flow program working end-to-end. Demo.
- [ ] Class 3 (file-ingestion #62): show `.dag` program reading an external file via `FileAttachment` carrier (per Director ratification 2026-05-13). Demo.
- [ ] Class 4 (workflow/scheduling #63): show CI workflow scheduling executing via `.dag`. Demo.
- [ ] Class 5 (reflection-closure #64): show `lens_apply.rs` reflection via PB-Runtime, end-to-end. Demo.
- [ ] **Falsification probe**: write a new substrate-gap (a 6th class). Could the framework absorb it without changes to the closure criteria? Or would §1.4 need re-authoring?

### §5.2 v2 fully retired

**Promise** (r3-program-plan.md §1.2): v2 not the live compiler; v2-oracle has no remaining test consumers per dissolution gates.

**Probes**:

- [ ] grep for `src/v2/` imports in `src/v3/`. Should be zero.
- [ ] grep for v2-oracle fixtures in v3 tests. Survivors must be frozen-snapshot consumption only, not live.
- [ ] **Falsification probe**: delete `src/v2/`. Does the test suite still pass?

### §5.3 BridgeLedgerZero

**Promise** (r3-program-plan.md §1.3): bridge inventory at zero — no bolt-on bridge between concept layers.

**Probes**:

- [ ] What's the current bridge ledger? Should be 0.
- [ ] If non-zero: enumerate each bridge. Has each a named dissolution trigger and target date?
- [ ] **Falsification probe**: would a code reviewer notice if a new bridge type was introduced without ledger entry?

### §5.4 Compiler-as-data residual — "is the compiler pure data yet?"

**Promise** (operator framing 2026-05-09 quoted verbatim in r3-program-plan.md §1.5 row #94 + r3-program-plan.md §1.8 row 1060 Q-Lens-Behavioral-Parity-R3-Closeability amendment): *"0 hand-Rust including tests AND stage0; bootstrap is data + self-generated"* — the R3 close criterion for the gunbc thesis claim "the compiler IS data, not code."

**Supporting framings**:
- THESIS.md: "substrate describes everything including itself"
- `src/v3/SELF_HOSTING.md` §1: "Self-hosting means v3's entire compiler pipeline — parse, lower, infer, emit — is written in `.dag`, compiled by v3's own compile loop, and produces the same byte-for-byte output as the current Rust stage0. The Rust code at `src/v3/compiler/src/` becomes a bootstrap seed: kept for fresh-checkout bootstrapping and for the initial compilation of the `.dag` pipeline files, but no longer the authoritative compiler. The 'real' compiler is the `.dag` one; Rust stage0 exists to get it off the ground."

**This section is the STRONG-FORM probe set** (§2.1 Pure Bootstrap covers the PB-0 census promise; §5.4 enumerates the specific file-class probes the operator asked for 2026-05-13):

#### §5.4.a Stage0 edit-requirement at R3 close

**Probes**:

- [ ] Walk through R3-close-anchored changes: how many required edits to `src/v3/compiler/src/*.rs` (hand-Rust) vs. `.dag` substrate?
- [ ] List the last 20 merged R3 PRs. For each, was the load-bearing change in `.rs` files or `.dag` files? Tabulate.
- [ ] Is there a SINGLE merged R3 PR where the load-bearing change touched ONLY `.dag` files (no `_generated.rs` regen, no hand-Rust)? Cite SHA.
- [ ] **Falsification probe**: pick a random `.dag` file. Edit it (e.g., add a field). Run the compiler. Does the compiler produce a correct emission without requiring any `_generated.rs` regen step that itself needs hand-Rust orchestration?

#### §5.4.b Hand-Rust file count at R3 close

**Probes** (against `src/v3/`):

- [ ] Count `.rs` files in `src/v3/` total. Cite the number at HEAD.
- [ ] Count `_generated.rs` (machine-emitted from `.dag`). Cite the number.
- [ ] **Hand-Rust = total − generated**. Cite the count. Is it 0 per operator's 2026-05-09 framing?
- [ ] If non-zero: enumerate each hand-Rust survivor. For each, cite the named-retirement-schedule (PR # / gate # / target SHA).
- [ ] Cross-check against `src/v3/SELF_HOSTING.md` §1 "bootstrap seed" framing: are the survivors *bootstrap-seed-only* (i.e., regenerable from `.dag` via the compiler itself), or do any encode authoritative behavior absent from `.dag`?
- [ ] **Falsification probe**: delete one hand-Rust file. Can the `.dag` substrate regenerate it via the compiler running on itself? Cite the regen invocation.

#### §5.4.c Other hand-maintained files (non-Rust)

**Probes** (broader than .rs):

- [ ] Enumerate hand-maintained files by extension class. `.yml` (CI/build). `.toml` (Cargo / Rust). `.md` (docs). `.sh` (scripts). Others.
- [ ] For each class, is the count R3-close-acceptable, or is there a named-dissolution-schedule?
  - CI / `.yml`: per gate #103 affected-set + T-WAD slices, is the long-term plan to derive CI YAML from `.dag` workflow declarations? Cite the gate.
  - `Cargo.toml`: is the long-term plan to derive Cargo manifests from `.dag` substrate? Or is `Cargo.toml` a "bootstrap-seed" peer to stage0 (kept-but-not-authoritative)?
  - Docs `.md` (this doc included): are docs hand-authored R3-close-acceptable, or is there a thesis-claim that docs derive from `.dag` (e.g., gate #26 `omni_documentation_drift_lock_demo`)?
  - Scripts `.sh`: enumerate. Each one is a process-discipline-bridge per `feedback_no_textual_enforcement_bridges` candidate — is each scoped to dissolve, or accepted as out-of-thesis?
- [ ] **Falsification probe**: pick a hand-maintained non-Rust file (e.g., a build script). Is the load-bearing fact it encodes derivable from `.dag` substrate? If yes, why isn't it derived? If no, what's the named carrier for the fact?

#### §5.4.d "Pure data" thesis-state at R3 close

**Promise** (THESIS.md substrate-describes-everything claim): the compiler is "pure data" — meaning the load-bearing fact about every behavior in the compiler lives in `.dag`, with Rust as mechanical execution-layer.

**Distinguishing two readings**:

- **Strong reading**: "0 hand-Rust files on disk" — every `.rs` is `_generated.rs` or absent.
- **Weak reading**: "0 hand-Rust *authoritative*" — `.rs` survivors are mechanically-derived bootstrap-seed (replayable from `.dag`), even if not literally machine-emitted today.

**Probes**:

- [ ] Which reading does R3 close target? Cite the disposition.
- [ ] If strong: PB-0 census at 0 per §2.1; this section dissolves into §2.1.
- [ ] If weak: what's the named distinction between "bootstrap-seed Rust" (acceptable) and "hand-authored Rust" (R3 debt)? Where is the line defined?
- [ ] Cross-reference SELF_HOSTING.md §1 "bootstrap seed" framing: is the bootstrap seed itself authored or derived? If authored, what makes it different from generic hand-Rust?
- [ ] **Falsification probe**: produce the `.dag` source for the LARGEST hand-Rust survivor at R3 close. Compile the `.dag`. Diff the emitted `.rs` against the survivor. Does the survivor match the emission byte-for-byte (or behaviorally), or is the survivor authoring facts not present in the `.dag`?

#### §5.4.e R3-close honest framing

This section's probes feed the close framing. PM-recommended answer-shape for R3 close:

- R3 close DOES NOT claim "0 hand-Rust files on disk" if the count is non-zero
- R3 close MAY claim "all hand-Rust survivors are bootstrap-seed-class with named retirement" if true and ledger-cross-referenced
- R3 close MAY claim "compiler authority is in `.dag`; Rust is mechanical execution-layer" if every hand-Rust survivor's authoritative behavior is mirrored in `.dag` and `pb_self_compile_fixed_point` (gate #16) holds
- R3 close MUST distinguish the strong vs weak reading explicitly — the operator's 2026-05-09 "0 hand-Rust including stage0" framing is the strong reading; the SELF_HOSTING.md "bootstrap seed" framing is the weak reading; reconciling these is itself an R3-close question

**Anti-pattern**: silently shipping with the weak reading while citing the strong reading. R3 close framing must explicitly cash which reading is operative + cite per-survivor disposition.

---

## §6. The "show the correct code" promise

**Promise** (THESIS.md:103-105): "Diagnostics should point to the structurally correct program, not just report that the current one is wrong."

**Probes**:

- [ ] Find a recent compile error. Does the diagnostic point to the correct alternative, or just say "this is wrong"?
- [ ] Pick 5 diagnostic message types. For each, does it satisfy the "show the correct code" criterion?
- [ ] If diagnostic just says "X is wrong" without "Y would be right": is that a GAP for R3 close, or R4-deferred?
- [ ] **Falsification probe**: write a program with a known structural error. Read the diagnostic. Could a user act on it without reading source code?

---

## §7. Cross-doc ledger coherence (structural — keep from v0)

These structural checks remain — they don't replace the adversarial promise-audit but they're necessary supporting evidence.

- [ ] §1.5 lane counts sum to §1.8 enumerated total
- [ ] §1.8 enumerated count matches r3-structure.md §"Acceptance"
- [ ] Q1 row in §10 matches §1.5 + §1.8 totals
- [ ] No gate ID gaps or duplicates in §1.8
- [ ] All `docs/audit/*.md` ratifications represented in §1.8 OR §10 (per `feedback_grep_audit_docs_before_answering_close_questions`)
- [ ] `docs/r3-remaining-work-dependency-graph.md` consistent with §1.8 PASSING/DECLARED state
- [ ] `r3_debt_paydown_zero_remaining` at close: 0
- [ ] ROADMAP `Post-merge debt` rows: 0
- [ ] §10 RED escalations: ALL CLOSED with owner sign-off on-ledger

---

## §8. Per-gate predicate execution at close

Every §1.8 PASSING gate's predicate must EXECUTE at close-time, not just at declaration-time. The execution log is preserved as `docs/audit/r3-close-predicate-execution-YYYY-MM-DD.md`.

- [ ] All 104 §1.8 predicates executed at HEAD within 24h of close ceremony
- [ ] Execution log preserved as audit artifact
- [ ] No predicate trivially passes via empty match-target or narrowed scope without justification
- [ ] PASSING gates whose predicate produces output: output preserved in close audit doc

---

## §9. Anti-patterns post-close (regression prevention)

To prevent R3-class debt from re-emerging:

- New gate added to §1.8 without §1.7 status assignment
- `docs/audit/*.md` authored without §1.8 row OR §10 entry OR explicit R4-deferred disposition
- Demonstration gate added without §1.6 (a)/(b)/(c) bar receipt
- Predicate that trivially passes (empty match-target, narrowed scope without justification)
- `Lookup<C>::Miss` or equivalent deferral-pattern re-introduction
- Hand-Rust added without PB-0 receipt or named retirement schedule
- Cross-doc count drift (§1.5 vs §1.8 vs r3-structure.md) without same-PR sync
- Diagnostic that says "X is wrong" without "Y would be right" (regression against THESIS.md:103-105)

---

## §10. Close ceremony

- [ ] Every §1-§8 item PROVEN or R4-DEFERRED with operator acceptance
- [ ] Zero GAP, zero WEAK-EVIDENCE
- [ ] Director sign-off recorded on-ledger
- [ ] Operator sign-off recorded on-ledger
- [ ] `ROADMAP.md` R3 → R3-closed milestone update merged
- [ ] §1.8 status sweep frozen
- [ ] R4 work-item creation greenlit
- [ ] R3 close audit doc preserved: `docs/audit/r3-close-YYYY-MM-DD.md` capturing per-gate predicate execution output + every probe's disposition with evidence link

---

## §11. Open questions for Director ratification

- **Q1**: For "Show me an example" probes — is one example per dimension sufficient, or do we require N examples covering the design space?
- **Q2**: For "Falsification probe" probes — do we require a probe attempt for every promise, or only for the headline thesis claims?
- **Q3**: For WEAK-EVIDENCE items — what's the operator threshold for "WEAK is good enough" vs "must be PROVEN"? (E.g., gate #10 L7 exhaustive — currently bounded `Int` slice; is that WEAK or R4-DEFERRED?)
- **Q4**: For external-checking — does R3 close require an independent reviewer (cross-Mgr or external) to attempt the falsification probes, or PM/Director self-audit sufficient?
- **Q5**: For evidence preservation — is `docs/audit/r3-close-YYYY-MM-DD.md` the canonical artifact, or do we also need per-probe evidence files?
- **Q6**: For the "diagnostics show correct code" probe — what counts as PROVEN? "Some diagnostics do" vs "every diagnostic class does" vs "the lens-discipline diagnostics specifically do"?

---

## §12. Authoring history

- **2026-05-13 v0** — PM-authored structural meta-acceptance checklist (12 categories, predicate-execution focus). Insufficient: structural, not adversarial.
- **2026-05-13 v1** (this version) — restructured per operator directive ("antagonistic"). Promise-vs-delivery interrogation with falsification probes. Awaits Director ratification on Q1-Q6 + cross-Mgr refinement before R3-close ceremony.
