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
  - **Rust**: per-fixture **unconditional** stdout-parity tests in CI ✓ (`src/v3/compiler/tests/boundary/m1_3_emit_rust_test.rs:995–1009` and following — `rustc_roundtrip_*` family); full-matrix `emit_rust_fixtures_rustc_green` at `#[ignore]` (lines 735, 764, 1199, 1218) — local-only, toolchain-gated.
  - **Python**: roundtrip tests at `#[ignore]` (`m1_4_emit_python_test.rs:1003, 1070`) — toolchain-gated (python3); **NOT in CI**.
  - **Go**: roundtrip tests at `#[ignore]` (`m1_3_emit_go_test.rs:252, 279, 324`) — toolchain-gated (go); **NOT in CI**.
  - **Omni demo** (Rust-only slice **runs unconditionally in CI** via `emit_omni_demo_rust_roundtrip` at `m1_5_emit_omni_demo_test.rs:106`; full **3-target** receipt — Rust + Python + Go — at `#[ignore]` via `emit_omni_demo_fixtures_green` at `m1_5_emit_omni_demo_test.rs:125`, requires go + python3 toolchains).
- **L5 corpus status** (gate #15 `l5_cross_target_consistency`): DECLARED, RED at HEAD (r3-program-plan.md:243 + :431) — waits on L4 corpus + Shape A grounding ready.

**Open R3 question (PM-surfaced, not yet routed)**:

What's the close-shape for the omni-emission promise?

- **(a) L6 data-coverage interpretation**: 41 (target × form × behavior) rows declared in v3-side data = ✓ for Rust/Python/Go. Structural-fold property, no runtime evidence needed.
- **(b) L4 runtime stdout-parity interpretation**: compiled emit-target binary stdout equals expected fixture stdout across the corpus = ✓ for Rust in CI (per-fixture + omni-slice), **toolchain-gated for Python/Go** (locally-only). Requires either (b1) running Python/Go roundtrips in CI behind a toolchain-gate, or (b2) explicit acceptance that R3-close evidence-bar is "data-coverage + Rust-runtime" with Python/Go runtime tied to a separate fast-follow gate.

The two interpretations differ on whether `#[ignore]`'d Python/Go runtime roundtrips count as R3-closure-evidence. THESIS.md:180 ("L5: **same .dag produces same behavior** in Rust/Python/Go") reads as runtime-shape; current CI evidence is Rust-runtime-only + L6 data-coverage-for-all-three.

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
