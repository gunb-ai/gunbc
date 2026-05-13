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

**Promise** (r3-program-plan.md §1.6 + §1.8 #70: `cost_lens_demonstration`): the cost lens reads representative target programs, composes algebra+realization cost end-to-end, and produces observable cost-bound output.

**Probes**:

- [ ] Show me a `.dag` program with a cost budget. Quote it.
- [ ] Show me the symbolic-cost output the lens produces. Verbatim.
- [ ] Show me a program that EXCEEDS its cost budget. What error fires?
- [ ] Where is the test? Cite path.
- [ ] Is the cost arithmetic actually composing? Show me a multi-level program (call within call) and the symbolic-cost result.
- [ ] **Falsification probe**: program with recursive call whose cost is bounded by a Tier-3 fact. Does the lens compute the bound or punt?

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
