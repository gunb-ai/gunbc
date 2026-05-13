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

### §1.6 Tier 1 mechanics (coercion = emission / ownership / grounding completeness)

**Promise** (THESIS.md:168-173 Tier 1 — Structural correctness): beyond the dimensions in §1.1-§1.5, Tier 1 makes specific structural commitments:

- **Coercion = emission**: "the compiler reads a target spec and translates. No separate coercion engine."
- **Ownership**: "the compiler proves no aliased mutation in emitted code."
- **Grounding completeness**: "target-side primitive types are structurally modeled from the target language reference (Rust Reference §Types, Python data model, Go specification), with algebra inhabitance declared structurally — not string-typed shortcuts in a lookup table. Mapping from a `.dag` type to a target primitive is a structural algebra-homomorphism search over declared inhabitance, not a name-keyed table lookup. If a `.dag` type cannot be structurally grounded to a target primitive, the compiler refuses to emit (fail-closed)."

**Coercion = emission probes**:

- [ ] Find the "coercion engine." Show me the file or argument-flow. Is it a separate phase, or does it dissolve into emission?
- [ ] Show me a `.dag` value being coerced to a target representation. Cite the call site. Is the coercion logic in emission, or in a separate `coerce.rs`?
- [ ] **Falsification probe**: introduce a `.dag` type with no target inhabitance. Does the compiler emit it via some default-coercion path, or fail-closed?

**Ownership probes**:

- [ ] Show me where the compiler proves "no aliased mutation" in emitted code. Cite the proof site (lens, predicate, or test).
- [ ] Show me an `.dag` program that would COMPILE if aliased mutation were allowed but FAILS now. Demonstrate the diagnostic.
- [ ] Where is "aliasing" modeled in the substrate? Is it a Behavior shape, a Cardinality property, or implicit-via-purity?
- [ ] **Falsification probe**: write a `.dag` program that mutates the same logical resource from two call sites. Does the compiler reject it, or emit potentially-aliasing target code?

**Grounding completeness probes** (highest-load-bearing per THESIS.md:173):

- [ ] Show me the structural model of Rust primitives. Cite `dsl/extdeps/languages/rust/primitives.dag` (or wherever). Does each Rust primitive have algebra-inhabitance declared (e.g., `i32 inhabits OrderedRing`)?
- [ ] Same for Python data model. Same for Go specification. Cite both.
- [ ] Show me the algebra-homomorphism-search code path. Cite the function. Is it a structural search, or a name-keyed table lookup?
- [ ] Pick a target primitive at random. Trace the grounding chain: `.dag` type → algebra inhabitance → target primitive. Is every step structural?
- [ ] **Falsification probe**: introduce a `.dag` type carrying algebra X. No target language inhabits X. Does the compiler fail-closed with a named diagnostic? Or does it pick a "closest" target primitive (failure mode — name-keyed shortcut)?
- [ ] **Per-target falsification**: for each of Rust / Python / Go, find a `.dag` algebra with no target inhabitance. Verify fail-closed on each.

**R3-close acceptance threshold** (PM-surfaced): grounding-completeness is the load-bearing claim for omni-emission. If it's name-keyed shortcuts (string-typed lookup), the whole `O(1)`-per-target story collapses. R3 close MUST demonstrate structural-grounding for at least one non-trivial primitive class per target.

### §1.7 Tier 2 runtime safety (proven safe or total)

**Promise** (THESIS.md:175-176): "Division by zero, integer overflow, out-of-bounds, force-unwrap, partial functions — either proven safe at compile time or made total. No partial functions in the runtime."

**Probes** (per partial-op class):

- [ ] **Division by zero**: show me a `.dag` program with a division. Show the divisor's proven-non-zero predicate (or the total-form making it safe). Cite the lens / check.
- [ ] **Integer overflow**: show me a `.dag` arithmetic expression whose target-side overflow is bound by structural analysis. Cite the cost-lens-or-machine-constraint composition.
- [ ] **Out-of-bounds**: show me a `.dag` indexed access. Show the index's proven-in-range predicate (or the total-form making OOB unrepresentable).
- [ ] **Force-unwrap**: search for force-unwrap patterns in the `.dag` substrate. Should be zero in the language surface. If present, justify as user-input-boundary.
- [ ] **Partial functions generally**: enumerate every partial primitive operation in `dsl/std/`. For each, show the totalization (Option-return / Diagnostic / refinement-precondition).

**Falsification probes**:

- [ ] Author a `.dag` program with division where the divisor's non-zero predicate is unprovable from structure. Does the compiler reject (Tier 1) or insert a runtime check (Tier 2 total)?
- [ ] Author an integer-arithmetic program where overflow CANNOT be proven safe. Does the compiler reject or insert a Diagnostic-returning total form?
- [ ] If a partial primitive lands in `dsl/std/` post-R3, what catches it? (Should be a §1.8 ratchet or anti-pattern; cite.)

**R3-close acceptance threshold**: every Tier-2 partial-op class either has a documented per-program proof path (compile-time) or a documented total form (runtime). Zero "trust me, no overflow happens" handwaving.

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

When the SAME `.dag` substrate emits to all 3 R3 Shape-A targets — **Rust + Python + Go** (per §3.1 / §518 of this doc + `r3-structure.md:11` / `r3-structure.md:86`) — cross-target bugs are structurally impossible for shapes that derive from substrate. But cross-target bugs at the GLUE layer (target-specific realization fidelity) are a real concern. Operator's framing: "seeing bugs between JavaScript and Rust" — the JavaScript references below are illustrative example-language pre-dating R3 scope finalization (R3 = Rust/Python/Go per §518; JavaScript not in R3 scope). The pedagogical bug shapes transfer to any third target, including Go:

- [ ] **Cross-target serialization round-trip**: Module A (emitted Rust) sends `User { id, name, email }` over wire to Module B (emitted JavaScript). Both emissions derive from the same `.dag` substrate. Field rename in `.dag` → both emissions update structurally; no schema-drift class possible. **Falsification**: rename a field in `.dag`; verify both Rust + JS emissions update; observe any cross-target consumer that didn't rebuild.
- [ ] **Cross-target numeric width**: `.dag` declares `Counter: Nat<32>`. Rust emits `u32`, JavaScript emits `number` (53-bit safe-int). At `Counter > 2^32`, JS overflows silently; Rust wraps/panics. Traditional: zero cross-language analysis. gunbc: cost-lens + dimensional carrier reads BOTH emissions' realization cost — JS's `number` carries different overflow semantics than Rust's `u32`, structurally expressible per dim-substrate (R3 anchor: gate #18 `numeric_width_refinements_landed` + Q-MachineConstraint-Carrier).
- [ ] **Cross-target effect divergence**: `.dag` declares an `async` operation. Rust emits via `tokio` futures; JavaScript emits via `Promise`; Python emits via `asyncio`. Cancellation semantics differ. Traditional: each emission target writes its own async-handling; cross-target tests catch only late. gunbc: substrate models async as structural fact; per-target LanguageSpec encodes realization; cost-lens reads composition.
- [ ] **Cross-target boundary trust**: Rust service calls JavaScript via FFI / WASM / HTTP. Type marshaling at the boundary. Traditional: protobuf-like schema at best (post-hoc). gunbc: BOTH ends derive from same `.dag` declaration; marshaling is structural emission per target.
- [ ] **Cross-target test-claim transferability**: `.dag` TestClaim asserts behavior X. Rust emission runs it via `cargo test`; Python via `pytest`; Go via `go test` (JavaScript via `jest` is illustrative-not-scope — see scope clarification above; R3 = Rust/Python/Go). All three should pass-or-fail identically for the SAME claim. Traditional: tests are language-specific; cross-language test-claims don't exist. gunbc: per gate #15 `l5_cross_target_consistency` — for every `.dag` program, emitted Rust/Python/Go produce equivalent runtime behavior on the certification corpus. (R3 close anchor — see §3.1 for L5 status.)

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
- Design doc: `docs/design-affected-set-lens.md` — **design framing + 5 worked examples; NOT a substrate-shape ratification, NOT a §1.8 gate addition** (per design doc §"Scope" line 7 verbatim). Substrate-shape ratification + §1.8 gate landing pending. Consumer-pattern sketch: CLI / agent / IDE invoking `IntrospectApplication`-carrier lens with `Set<NodeRef>` output. Prototype scope at [gunbc#2699](https://github.com/gunb-ai/gunbc/issues/2699).

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

#### §2.5.F.1 Minimality definition + examples table

**Minimality definition** (`docs/design-affected-set-lens.md:44`):

> "Strictly smaller than transitive-downstream — but only relative to the structural dimensions whose values actually changed. If `delta(M, dim_M)` can be proven empty by the lens → consumer N reading `dim_M` is excluded from the affected-set for that dimension. If `delta(M, dim_M)` cannot be proven empty → consumer N is included by default (fail-closed)."

"Minimal" = **minimal among soundly-derivable affected-sets given the lens's provability surface**. NOT theoretically-optimal (which would need omniscient delta-proof). The lens fails OVER-inclusive (sound but not theoretically-complete), never UNDER-inclusive — fail-closed always.

**Two correctness properties** (named per the lens's two-direction failure modes — avoiding standard-static-analysis "sound/complete" labels because their conventional meanings invert under fail-closed over-approximation):

**No-spurious-inclusion** (the "exclusion-correctness" direction): every node in the set has at least one dimension where `delta ≠ ∅` OR `delta = UNKNOWN`. If a node is in the set when every dimension it reads has `delta = ∅` provably, that's a spurious inclusion (lens over-included beyond the fail-closed mandate).

**No-missed-inclusion** (the "coverage" direction): no node with `delta ≠ ∅` on a dimension it reads is excluded. Missing a node with a known non-empty delta is the UNSAFE failure mode — the lens missed an actual cross-module dependency.

The lens is FAIL-CLOSED: when in doubt (UNKNOWN delta), include. This means the lens correctness target is **No-missed-inclusion (strict)** + **No-spurious-inclusion (relative to provability)** — over-inclusion on UNKNOWN is intentional and safe.

**Comprehensive examples** — normal code-change scenarios + expected affected-set behavior:

| # | Change scenario | Expected affected-set | Why (which dimension) |
|---|---|---|---|
| 1 | **Whitespace / formatting only** in function body | **∅** | Every dimension provably-unchanged at AST/Node-tree level |
| 2 | **Comment-only** changes (docstrings, inline comments) | **∅** | Comments are not structural; no Node-tree delta |
| 3 | **Identity-only change** (rename local variable; rename function with no callers) | **∅** | Per design doc §63: "identity change alone is NOT sufficient for propagation" |
| 4 | **Function rename WITH callers** (rename `f` → `g` everywhere) | **∅ on dimension surface** | If all callers update simultaneously, dimension shapes unchanged; identity change is invisible structurally |
| 5 | **Pure value change** (constant 5 → 7) | Consumers reading **value** dimension | Value-dim consumers flagged; cost/complexity/effect-dim consumers NOT flagged (unchanged) |
| 6 | **Algorithm swap, same complexity class** (different O(n log n) sort) | **∅** on cost dim; ∅ on complexity dim | Cost + complexity dimensions unchanged; effect may differ if I/O patterns shift |
| 7 | **Algorithm swap, complexity class changed** (O(n²) → O(n log n)) | Consumers reading **cost / complexity** dim | Value-dim consumers NOT flagged (return values same); cost/complexity-asserting consumers flagged |
| 8 | **Effect added** (pure → I/O — e.g., logging) | Consumers reading **effect** dim | Purity-asserting consumers flagged; value/cost-only consumers not |
| 9 | **Effect removed** (I/O removed) | Consumers reading **effect** dim | Symmetric to #8 |
| 10 | **Type signature change — argument added (required)** | Every **caller** | Type dim changed at the function's interface; every caller's call-site type-checks against the new shape |
| 11 | **Type signature change — argument added (optional with default)** | Callers that rely on **arity-exact** assertions | Most callers ∅; only callers asserting strict arity flagged |
| 12 | **Type signature change — argument removed** | Every caller | Symmetric to #10 |
| 13 | **Field added to struct/type** | Consumers reading the **type's structural shape** | Structural-shape consumers flagged; consumers reading only specific other fields NOT flagged |
| 14 | **Field removed from struct/type** | Consumers reading **that specific field** | Surgical scope; other-field consumers untouched |
| 15 | **Field renamed** (with all references updated) | **∅** on dimension surface | If atomic rename, no structural-shape delta (just naming); per identity-vs-dimension distinction |
| 16 | **Field type changed** (e.g., Int → Nat) | Consumers reading that field's **type** dim | Type-dim consumers; algebra-inhabitance consumers may surface (per §1.6 grounding-completeness) |
| 17 | **Refactor: extract function** (one fn → two with same external surface) | **∅** | External surface unchanged on all dimensions; internal structure is private |
| 18 | **Refactor: inline function** (collapse callsite) | **∅** | Symmetric to #17 |
| 19 | **New code added** (new function with no callers yet) | **∅** | No consumers exist; affected-set is trivially empty |
| 20 | **New code added WITH consumer call** | The consumer(s) that newly reference it | New edge in dependency graph; consumer's structural-shape may have changed |
| 21 | **Deletion of unused code** (no references) | **∅** | No consumers; trivially safe |
| 22 | **Deletion of code WITH consumers** (compile-error class) | Compile error; NOT affected-set's surface | The affected-set lens runs over compilable Dag pairs; uncompilable post-deletion = different surface (Diagnostic-class) |
| 23 | **Parallelism shape change** (sequential → parallel-marked) | Consumers reading **parallelism** dim | Parallelism-asserting consumers flagged; others not |
| 24 | **Test-only change** (test body / new test, no production code touched) | The changed test itself (in affected-set) | Tests don't propagate FORWARD (no production consumers of tests); test's affected-set entry triggers re-run, not transitive expansion |
| 25 | **Doc-only change** (`.md` files, READMEs) | **∅** for code consumers; affects only Markdown-emit consumers | Per `feedback_no_textual_enforcement_bridges`: docs are not structural |
| 26 | **CI / build config change** (`ci.yml`, `Cargo.toml`) | Affected-set as defined for **workflow substrate** (per gate #103 Slice 7 + T-WAD) | Workflow-as-data: CI changes have their own affected-set lens over the workflow graph |
| 27 | **Cross-module import added / removed** | If import provides new substrate fact, consumers of that fact flagged | Typical case: re-import alone is identity-class (∅); substrate-providing import affects the introduced/removed facts' consumers |
| 28 | **Dependency version bump** (Cargo.toml `serde 1.0.X → 1.0.Y`) | Consumers reading the dep's **structural surface** that changed | Depends on what changed in the dep; lens reads the dep's substrate facts post-update |
| 29 | **Generated code regeneration** (regen `_generated.rs` from `.dag` source) | Same as source `.dag` change's affected-set | Generated outputs are derived; affected-set is computed from the source delta, not the regen artifact |
| 30 | **Opaque `ExecuteCommand` / extdeps boundary modification** | **Fail-closed include downstream** | Lens can't prove delta empty across opaque boundary → over-inclusive but SAFE per §63 fail-closed discipline |
| 31 | **PB-Runtime kernel change** | **Fail-closed include downstream** | Bounded kernel changes can affect any consumer; over-inclusive but safe |
| 32 | **Compile-time-only change** (e.g., add `#[ignore]` to a test) | The changed test only; **∅** for production | Compile-time meta-attribute change; production-runtime dim unchanged |
| 33 | **Lens / cementing-test addition** (add a new `.dag` lens) | Programs that the lens NOW reports on (run the lens on every `.dag` program → affected-set of the lens's classification function) | Lens addition is a NEW dimension; the affected-set for "did dimension X change" is over the NEW dim's read surface |
| 34 | **Algebra-law refinement** (e.g., new `OrderedRing` instance for type T) | Consumers reading **algebra-inhabitance** of T | Per §1.6 grounding-completeness + §3.6 L7 algebraic-laws: algebra-inhabitance consumers flagged |
| 35 | **Substrate-shape extension** (e.g., new variant in a sum type) | Consumers pattern-matching on that sum type **without `_` wildcard** | Exhaustiveness-asserting consumers flagged; wildcard consumers may be ∅ |

**Falsification-probe pattern**: for any scenario in the table, the lens's correctness is verified by:

- **No-spurious-inclusion probe**: scenario expected to produce ∅ should produce ∅ at HEAD; if non-empty, identify spurious-inclusion class (lens over-included beyond fail-closed mandate)
- **No-missed-inclusion probe**: scenario expected to flag consumers should flag the EXPECTED set (not a strict subset); if missing, identify missed-inclusion class (unsafe failure mode)
- **Provability boundary probe**: scenarios marked "fail-closed include downstream" should produce non-minimal but SAFE affected-sets; verify they don't UNDER-include (under-inclusion is the unsafe failure mode)

**Honest framing** — as the lens's substrate-coverage improves (more dimensions provably-decidable), the affected-set shrinks toward theoretical-minimum. Current R3-scope substrate gives current observable minimum. R4 extensions (richer dimension grammar, finer-grained extdeps inhabitance) tighten it further. R3-close acceptance: cite affected-set sizes for canonical scenarios + identify the gap between observable and theoretical minimum.

**PM read** (provisional): the affected-set lens is THE structural cash for cross-module subtle-dep detection — without it, the §2.5.E cross-module bug classes are theoretically-impossible but operationally-unverified. With it, the static (compile-time) lens read + the diff-driven (affected-set) lens read compose to give *both* "this single snapshot is consistent" AND "this change preserves consistency." That's the omni-correctness story the operator's directive 2026-05-13 was probing.

### §2.6 Substrate-shape specifics (6 connectives + 5 behaviors + C1 stop-signal)

**Promise** (THESIS.md:198-203 Substrate shape — must not be flattened):

- **Types**: Node trees with **six connectives** — `Atom | Conj | Disj | Arrow | Cardinality | Instantiation`
- **Computation**: **five L1 behaviors** — `Value | Transform | Branch | Loop | Bind`
- **C1-class stop signal**: substrate extension (7th connective or 6th behavior) requires ALL FOUR dissolution patterns from §"Structural decompression" to fail with structural arguments before extension is allowed

**Probes — six connectives**:

- [ ] Enumerate every type at HEAD that is structurally NOT one of the 6 connectives (or composed of them). Should be ZERO.
- [ ] For each of the 6 connectives, cite ONE concrete use case at HEAD (e.g., `Conj` for product types, `Arrow` for function types, `Cardinality` for refinement, `Instantiation` for template-instantiation).
- [ ] Where is "Instantiation matches C++ template-instantiation vocabulary and is ONLY used for type parameterization" enforced? Show me a value-construction site — does it use plain `Conj` with optional inhabits tag (per THESIS.md:199), not `Instantiation`?
- [ ] **Falsification probe**: try to author a type at HEAD that resembles a 7th connective (e.g., a "metadata" or "reflection" variant outside the 6). Does the substrate reject it, or absorb it via one of the 4 dissolution patterns?

**Probes — five behaviors**:

- [ ] Enumerate every Behavior at HEAD that is structurally NOT one of the 5 (Value / Transform / Branch / Loop / Bind). Should be ZERO.
- [ ] For each of the 5, cite ONE concrete use case at HEAD.
- [ ] Where is `Transform → FunctionRef → Arrow` composition enforced (per THESIS.md:201)? Cite the substrate connection.
- [ ] **Falsification probe**: try to author a behavior at HEAD that doesn't fold into the 5 (e.g., a "guard" or "interrupt" variant). Does the substrate reject it, or absorb it?

**Probes — C1 stop signal**:

- [ ] Has anyone proposed a 7th connective or 6th behavior in PRs / canvases since R1? Cite each.
- [ ] For each proposal, was the C1 stop-signal protocol followed (all 4 dissolution patterns attempted before extension)? Cite the canvas / rationale.
- [ ] What's the current count of dissolution patterns documented in §"Structural decompression"? (Should be 4 per THESIS.md.)
- [ ] **Falsification probe**: an R4 proposal arrives proposing a 7th connective. What's the structural gate that ensures it goes through the 4-dissolution-attempt protocol? Is this a §1.8 ratchet or an unwritten norm?

**R3-close honest framing**: substrate-shape invariants are the bedrock of every other thesis claim. If a 7th connective or 6th behavior has slipped in without stop-signal protocol, every downstream thesis claim is at risk. R3 close MUST verify the 6+5 bounds hold at HEAD.

### §2.7 Modeling discipline

**Promise** (THESIS.md:415-419 Modeling discipline):

- Every declared type has at least one structural consumer.
- Every service boundary uses typed enums, not String/Bool proxies.
- No fabrication sentinels (`__BUG_*`, `__EMIT_BUG_*`). Missing facts are compile-time errors, not runtime strings.
- No duplicate record shapes. One type per concept.

Plus THESIS.md:359 — **Rust-authored tests are a language smell**. Every hand-authored `.rs` test flags a predicate, effect-model, or mock surface the language doesn't yet express.

**Probes — every type has a consumer**:

- [ ] Run the unused-types lens (or grep for un-referenced `type` declarations). What's the count? Should be 0.
- [ ] Pick 5 types at random from `dsl/std/`. For each, cite the consumer site.
- [ ] **Falsification probe**: declare a new type with no consumer. Does the compiler / lens flag it pre-merge?

**Probes — typed enums at service boundaries**:

- [ ] Grep for `String` or `Bool` used as discriminators at service boundaries. Enumerate. Each is a candidate for `feedback_opaque_strings_attract_heuristics`-class violation.
- [ ] Pick 3 service boundaries (e.g., extdeps providers, RFC carriers). For each, are discriminators typed enums or `String`/`Bool`?
- [ ] **Falsification probe**: introduce a String-typed discriminator at a service boundary. Does code review / lens / ratchet catch it?

**Probes — no fabrication sentinels**:

- [ ] Grep for `__BUG_`, `__EMIT_BUG_`, `__TODO_`, `__PENDING_` etc. across `dsl/` + `src/v3/`. Count.
- [ ] For any survivors: cite the named-dissolution path or operator-ratified justification.
- [ ] **Falsification probe**: try to add a `__SENTINEL_X__` string-marker as a code path. Does ratchet / lens reject it?

**Probes — no duplicate record shapes**:

- [ ] Run the duplicate-record-shape lens (or grep for structurally-identical types). What's the count?
- [ ] Pick 5 record shapes from `dsl/std/`. For each, verify structurally-distinct from every other.
- [ ] **Falsification probe**: declare two record types with identical fields/types but different names. Does the substrate flag the duplication, or accept it as namespacing?

**Probes — Rust-tests are a language smell**:

- [ ] Count hand-authored `.rs` test files (per SG-0 census `EXPECTED_HAND_AUTHORED_TEST`). What's the number at HEAD?
- [ ] For each hand-authored Rust test: what predicate / effect-model / mock surface does it flag as "language-doesn't-yet-express"? Enumerate.
- [ ] What's the named-retirement path per test? (Cite `pb_rust_tests_outside_residual_zero` ROADMAP gate.)
- [ ] **Falsification probe**: try to add a new hand-authored `.rs` test for a behavior that COULD be expressed in `.dag` `TestClaim`. Does the SG-0 census reject it?

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

### §3.5 L6 — every structural form compiles to every target

**Promise** (THESIS.md:181): "L6: every structural form compiles to every target."

This is the **completeness** claim distinct from L5 (consistency between targets). L5 says "if Rust + Python + Go all emit, they agree." L6 says "EVERY `.dag` form CAN emit to EVERY target — no holes, no per-target gaps."

**Probes**:

- [ ] Enumerate the structural forms in `.dag` (the 6 connectives × 5 behaviors product space, minus disallowed combinations). For each, is it emittable to Rust? Python? Go?
- [ ] Show me the L6 matrix at HEAD: (structural-form × target) → emit-status. Is it dense (all green), or sparse (per-target gaps)?
- [ ] Pick 5 structural forms. For each, trace the emit code path for Rust + Python + Go. Are they parallel, or does one target have special-case branching the others lack?
- [ ] **Falsification probe**: pick a structural form whose emit is implemented for one target but stubbed for another. Does the compiler fail-closed when targeting the stubbed lang, or silently emit broken code?
- [ ] **L6 vs L5 distinction probe**: a form that emits to all 3 targets BUT produces semantically-divergent output is an L5 failure. A form that fails to emit at all on one target is an L6 failure. R3-close: zero of either?

**R3-close honest framing**: L6 completeness is the strong-form omni-emission claim. R3 close MUST cite the form-by-form L6 matrix or explicitly defer to R4 with a named gap-class enumeration.

### §3.6 L7 — operations obey declared algebraic laws

**Promise** (THESIS.md:182): "L7: operations obey declared algebraic laws."

The compiler reads algebra declarations (`Monoid`, `Group`, `Field`, `OrderedRing`, etc.), and emitted operations honor the declared laws (associativity, identity, inverse, commutativity, distributivity). Failure = emitted Rust `+` operation on a `Monoid<T>` violating associativity ⇒ structural bug, not "test it later."

**Probes**:

- [ ] Pick an algebra carrier at HEAD (e.g., `Monoid<X>` for some X). Find the emitted operation. Show me the proof / test that the emitted op honors the algebra's law.
- [ ] Where do the algebra-law tests live? `dsl/std/algebra_axioms.dag` (or equivalent)? Cite path. Are these `.dag` `TestClaim` declarations (per §3.3) or hand-Rust?
- [ ] **Coverage probe**: for the 4-5 most-used algebra carriers (Monoid, Group, AbelianGroup, Ring, Field), is there a per-axiom test demonstrating laws hold post-emit? Tabulate.
- [ ] **Cross-target consistency**: do algebra-law tests pass on Rust + Python + Go independently? Or only Rust?
- [ ] **Falsification probe**: introduce a `.dag` Monoid declaration whose emit deliberately violates associativity (e.g., a free-monoid with a non-associative concat). Does the L7 test surface catch it?

**R3-close honest framing**: L7 is the algebraic-correctness anchor. If algebra-law tests are sparse or non-existent, "operations obey laws" is a claim without receipts. R3 close framing must either cite the per-axiom coverage or explicitly disposition this as R4-deferred.

### §3.7 The verification-machinery promises (testgen / integration / mocks / dry-run)

**Promise** (THESIS.md:166 + :348-368 + TESTING.md): "What mainstream languages catch via testing, profiling, schema validators, integration test suites, and production postmortems, gunbc catches by structurally deriving the proof or test." Testgen is downstream of code (structural coverage), integration tests are deliberate end-to-end, mocks are dependency-injection-by-construction (THESIS:367), and the pure-function posture admits structural dry-run by construction.

§3.3 covers TestClaim-as-data form (the assertion). This section covers the **verification machinery** around it.

#### §3.7.a Testgen — structural coverage derived from code

**Promise** (THESIS.md:356-358): "Testgen is downstream of code: structural coverage derived from the program the user wrote." Every type declared in `dsl/std/` should yield inhabitant + coercion tests automatically (per SELF_HOSTING.md §2 L1.5 step 3).

**Probes**:

- [ ] Where does testgen live? Cite the file or pipeline-stage. Is it `.dag` or hand-Rust?
- [ ] Pick a type from `dsl/std/` at random. Does testgen produce inhabitant + coercion tests for it? Cite the generated test count.
- [ ] What's the testgen→TestClaim flow? Generated TestClaim declarations land where (in-tree, in-memory)?
- [ ] **Coverage probe**: count types in `dsl/std/` vs count generated tests per type. Is coverage monotonic with declared structure?
- [ ] **Reshape probe per TESTING.md:343**: `m1_5_testgen_test.rs` should be spot-check not exhaustive-compile-every-claim. Is this discipline applied at HEAD?
- [ ] **Falsification probe**: declare a new type in `dsl/std/` with no consumer. Does testgen generate inhabitant tests for it, or does it skip? (Per §2.7 "every type has a structural consumer" — testgen IS a consumer if it's downstream of structure.)

**R3-close honest framing**: testgen completeness is the bridge between Tier 1/2 (compile-time proofs) and Tier 3 (runtime verification). If testgen is sparse or per-test-hand-written, the "structurally derived test surface" thesis claim collapses to "we have some generated tests."

#### §3.7.b Integration testing

**Promise** (TESTING.md:128 + :118): integration tests are deliberate end-to-end coverage; the standard form is `compile_to_dag(small_fixture)` exercising multiple substrate carriers + emission targets in one test. Heavy integration tests are exception, not rule (TESTING.md:5).

**Probes**:

- [ ] Enumerate integration tests at HEAD. How many? In what file-locations? (`src/v3/compiler/tests/` etc.)
- [ ] For each: is it `.dag` `TestClaim`-shaped or hand-authored `.rs`? Tabulate.
- [ ] Pick 3 integration tests. For each, what fixture does it compile? What's the end-to-end coverage (which substrate carriers, which emission targets, which lenses)?
- [ ] **Cross-target coverage**: are integration tests run for Rust + Python + Go targets independently? Or only one?
- [ ] **Mock-over-compile anti-pattern probe (per TESTING.md:84)**: does any integration test mock its compile-result rather than actually compiling? Should be zero.
- [ ] **Falsification probe**: introduce a regression that ONLY surfaces end-to-end (passes unit-level tests). Does any integration test catch it pre-merge?

**R3-close honest framing**: integration tests should be the smallest set sufficient to catch class-of-bugs the structural Tier 1/2 proofs can't (cross-module composition, emission-runtime divergence). If the set is huge or growing, the Tier 1/2 surface has gaps.

#### §3.7.c Mocks / dependency injection by construction

**Promise** (THESIS.md:367 + TESTING.md:175-186): "Consequence of the pure-function posture: effects are explicit parameters, mocking is dependency-injection-by-construction, no hidden state means no flaky tests."

The structural claim: every effectful operation takes its effect-source as a typed parameter; substituting a test double IS just substituting a different parameter value. No mock-framework, no test-double-DSL, no monkey-patching — the language gives mocking for free.

**Probes**:

- [ ] Pick a `.dag` program that performs I/O (e.g., HTTP call, file read). Show the effect-source as a typed parameter. Cite the carrier (e.g., `HttpClient`, `FileSystem`).
- [ ] Write a test for that program substituting a fake effect-source. How much new code did you write — a fake `HttpClient` implementation, or a mock-framework invocation?
- [ ] Run the test. Are there any hidden-state interactions (global state, ambient capabilities, environment dependencies)?
- [ ] **"No flaky tests" probe**: enumerate test-flakiness incidents in CI history. For each, what was the root cause? Hidden state, time-dependency, race? Are any caused by mocking infrastructure?
- [ ] **Falsification probe**: try to write a `.dag` program that uses an ambient capability (hidden state not in parameters). Does the substrate reject it, or admit it?
- [ ] **Cross-test pollution probe**: run integration tests in randomized order. Does any test depend on order, or on prior test leftover state? Should be zero.

**R3-close honest framing**: "mocking is dependency-injection-by-construction" is a strong claim — if the test substrate has mock-frameworks or test-double-DSLs at HEAD, the claim is false. R3 close MUST enumerate the test-doubling surface as evidence.

#### §3.7.d Dry-run / structural execution traces

**Promise** (derived from THESIS.md pure-function posture + bounded-execution invariant + §2.5.F affected-set lens): the compiler can answer "what would this program DO without running it" via structural analysis. The pure-function posture means effect-shapes are visible at the type level; bounded-execution means traces are computable.

**Probes**:

- [ ] What's the `dag run --dry-run` (or equivalent) invocation? Does it exist as a CLI flag, an `IntrospectApplication` lens, or implicit-via-purity?
- [ ] Run dry-run on an example `.dag` workflow. What does it output? (Expected: execution-trace lens reading + effect-summary + cost-estimate.)
- [ ] **Effect-shape preview probe**: pick a workflow with HTTP / DB / file effects. Can dry-run enumerate the effects it WOULD perform without performing them? Cite the lens / output.
- [ ] **Cost-preview probe**: can dry-run report the symbolic cost (§1.2) of executing the workflow without executing it? (Should compose with cost-lens output.)
- [ ] **Affected-set composition** (per §2.5.F): does dry-run compose with affected-set lens — i.e., "given this diff, what would actually re-execute"? Or are they separate query surfaces?
- [ ] **Simulated-inputs probe**: can dry-run accept simulated input values + report what the program would compute? Distinguish from "running with fake values" (which is actual execution).
- [ ] **Falsification probe**: a program performs an HTTP POST. Run dry-run. Does the actual HTTP request happen (failure) or is it captured as an effect-trace entry (success)?

**R3-close honest framing**: dry-run isn't an explicit thesis-claim, but it falls out of the pure-function + bounded-execution + lens-framework structure. If dry-run requires special tooling distinct from the lens framework, that's a surface-debt finding. R3-close framing should disposition: is dry-run (a) by-construction-via-lenses, (b) a separate CLI surface, or (c) NYI for R3.

#### §3.7.e Verification-machinery composition

**Probe — the unified question**:

- [ ] Run a single program through testgen + integration + mocks + dry-run. Do they share one substrate-read pass, or are they four separate pipelines? (Per `feedback_holistic_over_patches` + `feedback_compositional_not_templating`: should be one substrate-read with four lens-reads.)
- [ ] **Falsification probe**: a bug surfaces in production. Could ANY of the four (testgen / integration / mocks / dry-run) have caught it pre-merge? Tabulate per bug class. If one surface ALWAYS misses, that's a gap class.

**R3-close framing**: the four verification-machinery surfaces should be lens-compositions over the same substrate, not parallel pipelines. R3 close MUST demonstrate that adding a new verification dimension is one lens, not a separate pipeline.

### §3.8 Multi-program / network-coordinated emission from one `.dag` (FORWARD POINTER)

**Status**: STUB — pointer only. Substantive Q-dispositions land via R4 canvas (separate from path (b) full-stack-from-one-.dag canvas at PR #2847; distributed-coordination warrants its own canvas scope).

**Thesis-probe framing**: extend omni-emission from "one `.dag` → N representations of one program" to "one `.dag` → N cooperating distributed programs with derived wire interfaces, where the SAME structural facts (cost / complexity / effect / parallelism) apply across the system, not just per-endpoint." This is the natural extension axis from gate #28 omni-emission and gate #29 wire-serde-alignment.

**Structural cash at HEAD** (existing R3 substrate the multi-program story extends):
- Gate #25 `omni_openapi_backend_emission_demo` — CONSUMER_LANDED + PASSING (wire-contract emit)
- Gate #28 `omni_layers_share_one_node_tree` — CONSUMER_LANDED + PASSING (per Q4 Director ratification msg_7d51b699: NAME is layer-count-agnostic; the invariant is general)
- Gate #29 `anthropic_wire_typed_serde_alignment` — wire-derivation precedent for a specific external API
- T-Anthropic-Wire lane gates
- Path (b) R4 canvas PR #2847 — Director-ratified TS/React substrate; foundational for cross-deployment programs

**Distinct from path (b)**: path (b) covers single-program-multi-target (Rust backend + TS client + React UI + OpenAPI + SQL DDL from one `.dag`). §3.8 covers **multi-program coordination** — distinct programs at distinct deployment endpoints with explicit coordination semantics (sync / async / stream / pub-sub / eventually-consistent).

**Probes** (deferred to post-R4-canvas-ratification):

- [ ] **Multi-program shape**: how does `.dag` express "this fragment runs on machine A, this on machine B"? Is it a Cluster F lens reading a "deployment-target" dimension, OR substrate-level partitioning (carriers for `DeploymentUnit` / `Endpoint`)?
- [ ] **Wire derivation extension**: does extending gate #28 `omni_layers_share_one_node_tree` to "share one Dag across deployment units" hold, OR does cross-deployment need a new invariant gate?
- [ ] **Coordination semantics modeling**: are sync / async / stream / pub-sub first-class behaviors (a 6th L1 behavior or beyond? — would trigger C1 stop-signal per §2.6) OR compositions over existing 5 behaviors (Bind composition + effect-typed parameters)?
- [ ] **Failure-at-boundary**: is "partial failure" a lens read, an effect annotation, or a substrate variant? How does it compose with effect-enumeration lens (§1.4)?
- [ ] **Idempotency at endpoint**: composes with existing idempotency lens (per THESIS:188 "idempotency + cancellation + redundancy = algebraic simplification" + R1 demo class per THESIS:378-380)?
- [ ] **Cross-endpoint dimension propagation**: does the affected-set lens (§2.5.F) extend across deployment-unit boundaries? When endpoint A's cost dimension changes, are endpoint B's consumers reading A's wire-contract dimension flagged?
- [ ] **Falsification probe**: design a 2-endpoint distributed program in `.dag`. Demonstrate end-to-end emission: each endpoint emits its own backend (per Shape-A) + the wire contract between them (per Shape-B) + coordination behavior captured structurally. Or: identify the gap class.

**Open questions for R4 canvas authoring**:

- Does multi-program coordination warrant a NEW L1 behavior (6th: e.g., `Coordinate` for sync/async/stream/pubsub), OR is Bind composition + Effect annotation sufficient? Note: a 6th behavior would trigger C1 stop-signal per §2.6 (the four dissolution patterns must fail first).
- Are "machine A" / "machine B" addresses substrate-level carriers (concrete `Endpoint` type) or lens-readable dimension (deployment-target dimension reads)?
- How does failure-recovery compose with `feedback_fail_closed_discipline` (C-8)? Distributed systems force "retry-able failure" semantics; gunbc's fail-closed posture must extend coherently.

**PM read** (provisional, pre-canvas-authoring): multi-program coordination is the natural completion of the omni-emission story. Gate #28 already proves N projections share one Dag for ONE program; §3.8 extends to N projections × M programs. The wire substrate (gate #25, #29) provides the partial answer (interface derivation); coordination semantics and failure-mode handling are the open R4 axes. R4 canvas should disposition the questions above before R4 worker dispatch.

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

### §4.3 Concept unifications

**Promise** (THESIS.md:184-188 Concept unifications):

- **Coercion cost = complexity**: the cost of converting between representations IS measured by complexity-lens reads, not a parallel "coercion-cost" carrier.
- **Coercion = emission**: see §1.6 above; restated here as a unification.
- **Target language spec = transport spec = interpreter runtime**: ONE substrate carrier for all three. A Rust language spec IS-A transport spec IS-A interpreter runtime — different lenses read different facts from the same data.
- **Idempotency + cancellation + redundancy = algebraic simplification**: three named runtime concerns are ONE algebraic-simplification mechanism over substrate.

These are unification claims — load-bearing because each pairing-or-tripling that's separately-modeled is a parallel-authority violation per INVARIANTS P1.

**Coercion cost = complexity probes**:

- [ ] Find the "coercion cost" data. Cite the substrate carrier. Is it `Complexity` (the same one used by §1.1), or a separate `CoercionCost` carrier?
- [ ] If separate: that's a parallel-authority finding. Surface as P1 violation candidate.
- [ ] **Falsification probe**: a coercion between two representations has cost C. Is C readable via the complexity lens, or via a separate read?

**Coercion = emission probes**: see §1.6 — same probes apply here.

**Lang spec = transport spec = interpreter runtime probes**:

- [ ] Find the Rust language spec at `dsl/extdeps/languages/rust/`. Find the Rust transport spec. Are they ONE file / ONE substrate carrier, or distinct authorities?
- [ ] Same for an interpreter runtime (if v3 has one at HEAD; or the runtime-emission target). Is it the same substrate, or parallel?
- [ ] If distinct: each pair is a parallel-authority finding. Enumerate.
- [ ] **Falsification probe**: edit the Rust language spec. Does the transport spec auto-update (i.e., it IS the same fact) or require a parallel edit (P1 violation)?

**Idempotency + cancellation + redundancy = algebraic simplification probes**:

- [ ] Find the idempotency lens / mechanism. Find the cancellation lens / mechanism. Find the redundancy lens / mechanism. Are they three separate mechanisms or ONE algebraic-simplification engine with three lenses?
- [ ] If three: where's the unification path? (Should be in algebra.dag's compositional-fold of `Behavior::Bind` per `feedback_closed_system_effects`.)
- [ ] **Falsification probe**: write a `.dag` program with an obviously-redundant operation (e.g., reading the same key twice with no intervening write). Does the idempotency lens, cancellation lens, AND redundancy lens all catch it? Same diagnostic, or three different paths?

**R3-close honest framing**: each unification claim is structurally testable. If any pair / triple are separately-modeled at HEAD, R3 close framing must explicitly disposition the parallel-authority as `feedback_parallel_representation_debt`-class.

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

#### §5.4.d "Pure data" thesis-state at R3 close — interrogate against the committed 0-floor target

**Committed target** (`docs/design-pure-bootstrap-zero.md:41` verbatim): *"Goal: zero hand-authored files in v3's source tree. Better than v2's 1-residual."* The ≤5-floor framing was retracted; the 0-floor target is the LIVE shape per ROADMAP T-PB-A row amendment. `docs/design-pure-bootstrap-zero.md:210` clarifies the hand-authored-vs-generated boundary: *"trampolines are 0 if their content is generated... a 1-line `include!()` trampoline that's itself emitted from a `.dag` authority is generated, not hand-authored."*

**Promise** (THESIS.md substrate-describes-everything + design-pure-bootstrap-zero.md committed target): the compiler is "pure data" — every load-bearing fact lives in `.dag`; Rust files exist ONLY as machine-emitted artifacts of `.dag` authority. Hand-authored Rust at R3 close is debt against the 0-floor target, NOT an acceptable interpretation of "pure data."

**Reconciling against contrary evidence**:

`src/v3/SELF_HOSTING.md` §1 describes Rust at `src/v3/compiler/src/` as a "bootstrap seed: kept for fresh-checkout bootstrapping and for the initial compilation of the `.dag` pipeline files, but no longer the authoritative compiler." This framing describes the POST-0-floor functional shape, NOT an alternative R3-close criterion. The bootstrap-seed Rust is acceptable at R3 close IFF it is itself **machine-emitted from `.dag` authority** (per design-pure-bootstrap-zero.md:210 trampoline framing) — hand-authored bootstrap-seed Rust is R3-close debt against the 0-floor target.

**Probes** (interrogating against the 0-floor target, not between alternative readings):

- [ ] Run the PB-0 census at HEAD. What's the count of hand-authored `.rs` files in `src/v3/`? The committed target is **0** per design-pure-bootstrap-zero.md.
- [ ] For each hand-authored survivor: cite the named-retirement-schedule (PR # / gate # / target SHA). Survivors without named retirement are 0-floor debt.
- [ ] For each hand-authored survivor flagged as "bootstrap-seed": verify it is **machine-emitted from `.dag`** (replayable via `cargo run --bin regen-*` or equivalent). Hand-authored bootstrap-seed is NOT acceptable per design-pure-bootstrap-zero.md:210.
- [ ] Cross-reference SELF_HOSTING.md §1 "bootstrap seed" framing against design-pure-bootstrap-zero.md:210 generated-trampoline framing: is every claimed bootstrap-seed survivor actually generated, not hand-authored?
- [ ] **Falsification probe**: produce the `.dag` source for the LARGEST hand-Rust survivor at R3 close. Compile the `.dag`. Diff the emitted `.rs` against the survivor. If the survivor doesn't match the emission, the survivor is authoring facts not present in `.dag` — that's R3-close debt against the 0-floor target.
- [ ] **Bootstrap-resolution boundary probe** (per design-pure-bootstrap-zero.md:191 STOP-condition): if first-time bootstrap (N=0) resolution requires hand-Rust in v3's source tree, the 0-floor target is unreachable and the framing needs revision back toward an explicit alternative. Verify the N=0 resolution lives OUTSIDE `src/v3/` (install script, gunbc-runtime crate, rustc macro).

#### §5.4.e R3-close honest framing

The committed target (per `docs/design-pure-bootstrap-zero.md`) is 0 hand-authored files in `src/v3/`. R3 close framing must interrogate against this target, not negotiate around it.

PM-recommended answer-shape for R3 close:

- R3 close MUST report PB-0 census count at HEAD. Per the committed 0-floor target: the goal is **0**.
- If census count > 0 at R3 close: each survivor MUST be either (i) machine-emitted from `.dag` (and therefore not actually hand-authored per design-pure-bootstrap-zero.md:210), OR (ii) on the SG-0 ledger with named-retirement-schedule (per `EXPECTED_HAND_AUTHORED_NON_TEST` + `EXPECTED_HAND_AUTHORED_TEST` discipline).
- Hand-authored survivors with named-retirement-schedule are **acknowledged R3 debt against the 0-floor target**, NOT "acceptable close criterion." The named retirement is the dissolution plan; the survivor itself is debt.
- R3 close framing CANNOT claim "pure data" if hand-authored survivors exist without machine-emitted reconciliation. The thesis claim is true iff the census is 0 OR all survivors are machine-emitted.

**Anti-pattern**: silently shipping with hand-authored survivors while claiming "compiler is pure data" or "bootstrap-seed framing satisfies R3 close." Per design-pure-bootstrap-zero.md authority, bootstrap-seed Rust is acceptable IFF it is itself generated; otherwise it is 0-floor debt. R3 close framing must cite the census count + per-survivor disposition (machine-emitted OR named-retirement-schedule) against the committed 0-floor target.

### §5.5 Free consequences (when Tiers 1-2 close)

**Promise** (THESIS.md:205-210): "Free consequences (fall out when Tiers 1-2 close):
- Automatic parallelism from dependency graph.
- Automatic memoization from purity + cost.
- Incremental cross-run execution from purity + bounded execution + dependency graph.
- Space bound proofs from CX.
- Cross-language optimization from shared cost algebra."

§1.3 covers parallelism; §1.1 partially covers space bounds via complexity. This section probes the rest as standalone "free consequence" claims.

**Automatic memoization probes**:

- [ ] Find the memoization mechanism. Cite the lens / decorator / substrate carrier.
- [ ] Is memoization opt-in (annotation) or by-construction (compiler reads purity + cost and applies it automatically)?
- [ ] Show me a `.dag` program that should benefit from memoization. Compile + run. Was memoization applied? How is "was applied" verifiable (cost-lens output? execution-trace lens? cache-hit metric)?
- [ ] **Falsification probe**: a pure function with high cost is called twice with the same args. Does the compiler emit code that memoizes, or naive double-execution?

**Incremental cross-run execution probes**:

- [ ] Find the incremental-execution mechanism. Cite the lens / data carrier.
- [ ] Run an example `.dag` workflow twice with no input changes. Does the second run skip pure subgraphs that haven't changed? Verify via execution-trace lens.
- [ ] Now change ONE input. Verify that only the affected-set subgraph re-executes (composes with §2.5.F affected-set lens).
- [ ] **Falsification probe**: a non-deterministic step is in the graph. Does the incremental-execution mechanism correctly avoid skipping its re-evaluation?

**Cross-language optimization probes**:

- [ ] Find the "shared cost algebra" that enables cross-language optimization. Cite the substrate carrier.
- [ ] Pick an optimization (e.g., loop-fusion / tail-recursion / cse). Is it applied uniformly across Rust + Python + Go targets, or per-target?
- [ ] **Falsification probe**: write a `.dag` program where one target language's optimizer would catch a cost reduction but another wouldn't. Does the shared-cost-algebra propagate the optimization to ALL targets, or only the one whose host optimizer applies?

**R3-close honest framing**: "free consequences" is a strong claim — each consequence must be demonstrably operational, not just structurally available. R3 close MUST cite per-consequence demos OR explicitly defer (per-consequence) to R4 with named gap.

---

## §6. The user-experience / adoption promises

### §6.1 "Show the correct code"

**Promise** (THESIS.md:103-105): "Diagnostics should point to the structurally correct program, not just report that the current one is wrong."

**Probes**:

- [ ] Find a recent compile error. Does the diagnostic point to the correct alternative, or just say "this is wrong"?
- [ ] Pick 5 diagnostic message types. For each, does it satisfy the "show the correct code" criterion?
- [ ] If diagnostic just says "X is wrong" without "Y would be right": is that a GAP for R3 close, or R4-deferred?
- [ ] **Falsification probe**: write a program with a known structural error. Read the diagnostic. Could a user act on it without reading source code?

### §6.2 Audience duality / opt-in depth

**Promise** (THESIS.md:307-321 Audience duality):

- Core language stays approachable — types, functions, match, effects, workflows. Any engineer can write a gunbc program and get multi-target emission without learning the lens/proof surface.
- Advanced surface is opt-in — lenses, cementing tests, user-authored static reflection, complexity/cost/idempotency proofs.
- "gunbc does not pick a tribe. Normal programmers get glue generation; principal engineers get structural proofs. The same compiler serves both because depth is a surface the user opts into."

**Probes — core-language approachability**:

- [ ] Show me the SIMPLEST `.dag` program — types + function + maybe one workflow. How many concepts must the author know? Is the proof / lens surface invisible?
- [ ] Compile the simple program for Rust + Python + Go. Does it emit working code in all 3 without the user touching lens/proof surface?
- [ ] ROADMAP cites `fixture_integration_canonical` for the glue-generation audience. Cite the fixture path. Compile + run. Does it land in <100 LOC of `.dag` surface for the user?

**Probes — opt-in depth**:

- [ ] Show me a `.dag` program that opens the advanced surface — author a user-defined lens. How much new surface does the user touch? Is the base-language surface untouched (per "opening doesn't change the base")?
- [ ] ROADMAP cites `fixture_compiler_nerd_canonical` for the structural-proof audience. Cite the fixture path. Verify it exercises lens + proof surfaces.
- [ ] **Falsification probe**: a base-language change (e.g., a new type connective) shouldn't require the advanced-surface user to relearn. Conversely, an advanced-surface change shouldn't break base-language programs. Demonstrate non-coupling at each direction.

**R3-close honest framing**: audience duality is the recruiting-mechanism claim. R3 close MUST cite per-audience demo fixtures (both T-Demo fixtures landed) OR explicitly defer demo-coverage to R4 with named gap.

### §6.3 Adoption model — economics, not enforcement

**Promise** (THESIS.md:323-346 Adoption model):

- "The thesis claims every program gets complexity, effects, termination, idempotency, and ownership for free — by construction, not by opt-in. ... There is no in-language way to author a program the lenses can't read."
- **Leaving the stack (in-language)**: composing primitives into named patterns (namespacing). "The compiler sees through; lenses still apply. Still inside the stack."
- **Leaving the stack (outside language)**: writing a different compiler on different primitives. "The thesis does not prevent this and does not need to — gunbc's lenses are folds over *our* primitives."
- "Adoption is therefore gated by **economics, not enforcement**: low cost of entry × high free value."

**Probes — every program gets the guarantees**:

- [ ] Pick an arbitrary `.dag` program at HEAD. Apply the complexity lens. Does it produce output? (It should — no opt-in flag should be required.)
- [ ] Same for effect / cost / parallelism lenses. Each should produce output for ANY `.dag` program at HEAD.
- [ ] **Falsification probe**: try to author a `.dag` program that opts OUT of complexity/cost/effect lens reads. Is there ANY in-language syntax that disables lens reads? (Should be zero per `feedback_groundedness_gates_lenses`.)

**Probes — leaving the stack (in-language)**:

- [ ] Author a `.dag` namespace pattern. Run all 4 lenses on it. Do they all read through the namespacing, or does one of them stop at the named boundary? (Should read through per "compiler sees through" claim.)
- [ ] **Falsification probe**: try to "hide" a complexity violation behind a named pattern (e.g., wrap an O(n²) function in a typedef'd "FastLookup<T>"). Does the complexity lens still flag it?

**Probes — leaving the stack (outside language)**:

- [ ] Show me a `.dag` program that calls out to an externally-implemented operation (e.g., `ExecuteCommand` / `extdeps` provider). Where does lens-coverage end? Cite the explicit boundary marker (per `feedback_groundedness_gates_lenses`).
- [ ] **Falsification probe**: try to author a program that EFFECTIVELY leaves the stack inside the language (e.g., via heavy use of `extdeps` opaque calls). Is the lens-coverage boundary explicit + auditable?

**Probes — economics not enforcement**:

- [ ] What's the LOC overhead of a `.dag` program vs. an equivalent program in Rust / Python / Go? Tabulate for a canonical small + medium + large example.
- [ ] What's the percentage of programs where ALL lenses produce green output (no violations)? If the percentage is high, "high free value" is structurally true.
- [ ] **Falsification probe**: is there a class of programs gunbc IS more verbose / more constraining than alternatives, in a way that fails the "economics" test? Enumerate (acceptable to defer as R4 if narrow).

**R3-close honest framing**: adoption model is the recruiting-mechanism story. R3 close framing should explicitly cash whether "low cost × high free value" is structurally demonstrated OR is forward-looking thesis-pitch.

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
