# v4 — XL Task Plan

The XL tasks below define "v4 done" (the count is intentionally NOT stated — it drifts as scope is ratified; the close gate is "every task in this plan," never a hardcoded number — see T-15). Each task is a bounded modeling unit; each produces a typed pure function in declared files; each is honestly hard to game because the work IS the decisions.

**Sizing discipline** (per operator directive 2026-05-15): all tasks are XL by default. Relative sizing (S / M / L / XL within the XL bracket) is used only when conveying scope-risk explicitly. **No timelines, no day estimates** — discuss only technical decisions.

## Execution graph

```
Phase 1 (parallel — substrate foundation):
  T-1   std/node.dag                     [BLOCKS: all]
  T-2   std/algebra.dag                  [needs T-1]
  T-3   std/* supporting (6 files)       [needs T-1]
  T-4   extdeps/languages/{rust,python,go,cpp,typescript}.dag   [needs T-1, T-2]
  T-4.5 extdeps/{process,file_system}.dag                      [needs T-3]
  T-4.6 extdeps/formats/* (6 files: json/yaml/csv/toml/json_schema/openapi)
  T-4.7 extdeps/frameworks/react.dag    [needs T-4 (typescript)]
  T-4.8 extdeps/coordination.dag         [needs T-4, T-4.7]
  T-4.9  extdeps/languages/verilog.dag   [needs T-1, T-2; B2-OMNI falsification probe — concurrency vs the 5 behaviors]
  T-4.10 extdeps/formats/spice.dag       [needs T-1; B2-OMNI falsification probe — LanguageModel generality (no control flow)]
  T-4.11 test/claim/boundary/english_ingest_fail_closed.dag  [needs T-1; boundary-honesty probe — fail-closed ingest, no fabrication]
  T-4.12 extdeps/languages/llvm_ir.dag   [needs T-1, T-2; B2-OMNI probe — generalize DOWN the stack (SSA IR)]
  T-4.13 extdeps/languages/machine_code.dag  [needs T-1; B2-OMNI probe — bottom of stack; disassembly = extreme fail-closed]
  T-4.14 extdeps/languages/ptx.dag       [needs T-1, T-2; B2-OMNI + IN-B probe — SIMT data-parallel vs the 5 behaviors]
  T-5   workflow/* (5 files)             [needs T-1; FIRST IN EXECUTION]

Phase 1.5 (test + bootstrap substrate — early, before compiler stages):
  T-19  lens/testgen.dag                 [needs T-1, T-2, T-3]
        Produces TestClaim corpus from substrate; manual TestClaims in
        test/claim/manual/ serve as anti-regression contract until
        T-19 implementation lands. Every Phase 2+ task benefits from
        testgen-derived test coverage instead of hand-authoring.
  T-20  workflow/bootstrap.dag           [needs T-1; grows incrementally]
        Bootstrap orchestration AS DATA (seed-once → self-host →
        fixed-point). v2 interprets it. Scaffold-early (the parse-
        viability step is the existing CI gate); full self-host
        content lands as the pipeline matures. T-15 consumes it for
        fixed-point validation. NOT a build.rs/shell (that = the v3
        regression door).
  T-21  lens/affected_set.dag            [needs T-1, T-2, T-3]
        Incremental re-exec frontier (operator: "wanted very early").
        Structural authority that replaces scripts/detect-affected-
        components.sh. Consumed by T-24 (ci) + eval (skip pure
        unchanged subgraphs).
  T-24  workflow/ci.dag                  [needs T-21, T-20]
        CI pipeline AS DATA; .github/workflows/ci.yml derived. Closes
        v3's gate-#98 gap (hand-authored CI YAML). Consumes T-21 for
        job selection — the shell bridge dissolves once both land.

Phase 2 (serial — pipeline stages):
  T-6   compiler/01_tokenize.dag         [needs T-3]
  T-7   compiler/02_parse.dag            [needs T-6]
  T-8   compiler/03_normalize.dag + 03_resolve.dag   [needs T-7]
  T-9   compiler/04_infer.dag            [needs T-8, T-2, T-3]
  T-10  compiler/05_emit.dag + 00_compile.dag       [needs T-9, T-4]
  T-22  compiler/05_eval.dag             [needs T-9]
        The interpreter — THE PRIMARY execution path (THESIS:225).
        Sibling of emit (same InferredTree input). workflow/bootstrap.dag
        + TestClaim eval + lens dry-run all compose over it.

Phase 3 (parallel — lens dimensions):
  T-11  emit per-target specialization (extends T-10 across all 5 Shape A targets)
  T-12  lens/complexity.dag + lens/cost.dag      [needs T-9]
  T-13  lens/{parallelism,effect,ownership,idempotency}.dag   [needs T-9]
  T-17  lens/synthesis.dag + std/report.dag  (cross-algorithm complexity, C7;
         XL scope, research-tier risk)              [needs T-12 for current-complexity input]
  T-18  lens/coverage.dag  (meta-lens: L6/L7/impossible-bug/testgen coverage
         discipline; STRUCTURAL not exhaustive-fixture per TESTING.md)
                                                    [needs T-3, T-4, T-12, T-13]
  T-23  lens/application.dag  (apply_lens surface — opt-in depth + the ONLY
         advisory→fail-closed bridge; load-bearing for §1.5 user-defined
         dimensions + §6.2 audience duality + C7 Report→Diagnostic)
                                                    [needs T-1, lens framework]

Phase 4 (serial — close the loop):
  T-14  test/claim/* + test/fixture/* (port load-bearing TestClaims from v3)
  T-15  bin/main.dag + bootstrap glue + self-host fixed-point validation
  T-16  Full-stack omni-emission demo: ONE .dag → Rust+C++ backend
        + React/TS frontend + OpenAPI wire contract
        [needs T-4, T-4.5, T-4.6, T-4.7, T-4.8, T-10, T-11]
        (T-4.8 coordination.dag is load-bearing — T-16 uses it for
        endpoint partitioning; facts must flow forward from the
        coordination substrate into the flagship demo)
```

## Task definitions

### T-1: std/node.dag — substrate root

**File**: `src/v4/std/node.dag`
**Why first**: every other file consumes this. Get this right; the rest follows.

**Modeling decisions**:
- Exact shape of the 6 connectives (do they share a common base, or are they truly disjoint?)
- Encoding of the 5 L1 behaviors (sum-type vs separate types?)
- C1 stop-signal mechanism (how does the substrate refuse a 7th connective by construction?)

**Reference**:
- v2: `src/v2/00_core.dag` — prior approximation
- v3: `dsl/std/` directory — substrate refinement attempts (not all honest)
- `THESIS.md` "Substrate shape" section + `docs/thesis/the-substrate-two-coordinated-shapes.md`

---

### T-2: std/algebra.dag — algebraic primitives

**File**: `src/v4/std/algebra.dag`
**Why critical**: the epistemic chain roots here. Without this, codegen has no walk path.

**Modeling decisions**:
- Inhabitance declaration shape (relation? predicate? typeclass-style?)
- Composition: how do Sum/Product algebras compose for the cost lens?
- Free constructions: FreeMonoid<T> as primitive vs derived?

**Reference**:
- v3: `dsl/std/algebra.dag` (study; expected substantive)
- `THESIS.md` "Epistemic stacking" section

---

### T-3: std/* supporting (cardinality, witness, diagnostic, primitive, collection, verification)

**File**: 6 files in `src/v4/std/`
**Why bundled**: smaller individually, all interrelated, foundation for everything.

**Modeling decisions per file** (see file headers for specifics).

**Reference**:
- v3 mirrors of each (study for design, audit for honesty)
- TestClaim schema: import directly from `dsl/std/verification.dag:38`

---

### T-4: extdeps/languages/{rust,python,go,cpp,typescript}.dag

**File**: 5 files in `src/v4/extdeps/languages/` (operator-ratified 2026-05-15: cpp + typescript added; cpp subsumes C subset; Go retained)
**Why bundled**: identical structural shape per language; the SHAPE is the work. Each file declares the language MODEL (grammar + types + semantics) — direction-agnostic; emit AND ingest are operations against the same model.

**Authoring contract (operator-ratified 2026-05-15):**
- **Model the SPECIFICATION, not libraries (L-2).** Model the versioned upstream spec (Rust Reference, ECMAScript/TS Handbook, IEEE 1364, …) — the anchor IS that spec. Do NOT model std/crates/packages: a library is just a program in the modeled language = `Node`. Modeling libraries is infinite, non-general, the wrong layer.
- **Declare every surface feature's disposition (C5-fidelity).** For each feature: `Modeled` (∈ F, Node-bearing, round-trips both ways — e.g. Python indentation IS block structure) | `Declared-normalized` (deliberately not in F; `emit∘ingest` canonicalizes — Go/C++ insignificant whitespace; a *declared*, reviewable loss, never silent) | `Fail-closed` (encountered but neither → Diagnostic, no-engine). F = the spec's own meaning-vs-lexical distinction, not worker judgment. Round-trip fidelity = declared model completeness.

**Modeling decisions**:
- Per-language primitive inhabitance (i32 -> OrderedRing, std::vector<T> -> List<T>, etc.)
- Per-language realization cost shape
- Grammar encoding: declarative production data — the **bidirectional relation** (concrete syntax ⟷ Node), read as ingest (partial, many→one, fail-closed off F) and emit (the chosen canonical section); NOT a procedural recognizer. The ingest reading MUST be unambiguous, or ambiguity ⇒ Diagnostic (never "parser picks one" = fabrication). Syntax needing semantic feedback to parse (C++ most-vexing-parse, `<` template-vs-less-than) is a STOP/escalation, not silently absorbed.
- Type system: nominal (Rust, Java) vs structural (TypeScript, Go), or both (C++)

**Reference**:
- v2: `src/v2/languages.dag`
- v3: `dsl/extdeps/languages/` (audit each for honesty)

---

### T-4.5: extdeps/process.dag + extdeps/file_system.dag

**File**: 2 files in `src/v4/extdeps/`
**Why bundled**: both are OS-interaction substrate; both are required for v4 to function as a self-hosting compiler (read source files, write emitted files, ExecuteCommand for boundary tests per THESIS facet 3).
**Why anchored**: each file carries a `# Anchor:` to its canonical reference (Wikipedia/POSIX). Reviewers validate the modeling against the reference — no invented vocabulary.

**Modeling decisions**:
- `process.dag`: how to model parent/child relationships? Signal handling depth (full POSIX signal set vs minimal {SIGTERM, SIGKILL, SIGINT})? Pipe model for capture (live-streaming vs buffered)?
- `file_system.dag`: AbsolutePath vs RelativePath as Disj sum or refinement on Path? Symlink target as recursive Path or opaque? Read failure modes (NotFound vs PermissionDenied vs IOError) as Diagnostic NamedReason variants.

**Reference**:
- Anchors in file headers (Wikipedia: Process, Wikipedia: File system, POSIX File and Directory Operations)
- v2 / v3 had ad-hoc I/O sprinkled across files — v4 consolidates per substrate-cohesion discipline

---

### T-5: workflow/* — recursive-flex (FIRST IN EXECUTION ORDER)

**File**: 5 files in `src/v4/workflow/`
**Why FIRST**: this IS the structural fix to v3's hierarchy/gaming failure. Implement workflow substrate before any compiler work, so every subsequent task's WorkerOutput is a typed instance.

**Modeling decisions**:
- Brief contract shape (what fields are mandatory?)
- Retirement predicate (A3, operator-ratified 2026-05-15): `retired` is a REPRODUCTION, never a count. `retired ⟺ rebuild-from-(.dag + frozen-pinned seed)-only reproduces the pinned hash ∧ the seed's own hash matches its pin`. `HandResidual` = the Rust the .dag-rebuild cannot reproduce — empty by reproduction, not by count (defeats paper-shrink: relocation/inlining is non-seed Rust, removed by the test). NOT un-gameable (Trusting-Trust — pin/CI/seed are editable); its job is early/loud surfacing per-PR on the affected set so gaming is un-missable + operator-routed. Seed trust = named axiom, not proof. Enforcement = operator-ratification + STOP-culture; structure makes defection conspicuous, not impossible. Typed workflow substrate, NOT a CI grep (feedback_no_textual_enforcement_bridges).
- Cycle data: lens-readable progression vs prose status

**Reference**:
- This conversation (the failure mode that motivated this substrate)
- `feedback_doc_authority_must_propagate_to_execution_authority` (memory)
- `feedback_paper_shrink_variants` (memory) — the failure modes to refuse

---

### T-6: compiler/01_tokenize.dag

**I/O**: `FreeMonoid<Char> -> Result<TokenStream, Diagnostic>`

**Modeling decisions**:
- Character class encoding (predicate fn vs enum vs charset)
- Whitespace/comment handling (preserve vs discard)
- Token boundary discipline

**Reference**:
- v2: `src/v2/01_tokenize.dag`
- v3 L2.5 design: `docs/r3-path-b-tokenize-parse-brief-set.md` PB-2

---

### T-7: compiler/02_parse.dag

**I/O**: `TokenStream -> Result<ParseTree, Diagnostic>`

**Modeling decisions**:
- Grammar productions as Node trees vs separate parser substrate
- Error recovery (single Diagnostic vs continued)
- ParseTree shape (layout-preserving?)

**Reference**:
- v2: `src/v2/02_parse.dag`
- v3 L2.5 design: `docs/r3-path-b-tokenize-parse-brief-set.md` PB-3

---

### T-8: compiler/03_normalize.dag + 03_resolve.dag

**I/O**: `ParseTree -> NormalizedTree -> ResolvedTree`

**Modeling decisions**:
- Surface sugar dissolution rules (service/fn/type -> Node tree)
- Identifier binding strategy (scope chain vs flat namespace)

**Reference**:
- v2: `src/v2/03_normalize.dag`, `src/v2/03_resolve.dag`

---

### T-9: compiler/04_infer.dag

**I/O**: `ResolvedTree -> Result<InferredTree, Diagnostic>`

**This is the file v2 split into 12 files (`04_*`).** v4's discipline: this is ONE file. Pressure to split = substrate design escalation, not a worker decision.

**Modeling decisions**:
- Algebra-homomorphism search algorithm
- Cardinality propagation
- Diagnostic precision when inference fails

**Reference**:
- v2: `src/v2/04_*.dag` (12 files — read AS the cautionary tale on substrate inflation)
- v3 L2.5 design: PB-5 infer model (PR #3085)

---

### T-10: compiler/05_emit.dag + compiler/00_compile.dag — emission + orchestrator

**I/O**:
- `emit: (InferredTree, TargetSpec) -> Result<TargetSource, Diagnostic>`
- `compile: (Source, TargetSpec) -> Result<TargetSource, Diagnostic>` (orchestrator)

**Modeling decisions**:
- Target-agnostic IR shape
- How target spec drives concrete emission (interpreter vs codegen)
- Orchestrator: monadic `Result` chaining vs early-return pattern

**Reference**:
- v2: `src/v2/05_emit.dag`, `src/v2/compile.dag`
- v3 L2.5 design: PB-emit model (`docs/r3-retirement-modeling-emit-rs.md`)

---

### T-11: emit per-target specialization

**Why separate from T-10**: T-10 is the orchestrator; T-11 is the per-target translation tables that populate emit's behavior across rust/python/go.

**Modeling decisions**:
- Per-target translation rules
- Target-specific optimizations (or absence thereof)

---

### T-12: lens/complexity.dag + lens/cost.dag

**I/O**: `Node -> Witness<ComplexityBound>`, `Node -> Witness<SymbolicCost>`

**Modeling decisions**:
- Complexity class encoding
- SymbolicCost lattice shape (per `docs/audit/sub-value-relation-bounded-lattice-claim.md`)
- Composition with Sum/Product algebra

---

### T-13: lens/{parallelism,effect,ownership,idempotency}.dag

**I/O**: `Node -> Witness<...>` per lens

**Modeling decisions per lens** (see file headers).

---

### T-14: test/claim/* + test/fixture/* — TestClaim corpus

**Files**: `src/v4/test/claim/*` directories (6 impossible_bug + algebra_laws + diagnostic_correction + future categories) + `src/v4/test/fixture/*`
**Operator-ratified additions 2026-05-15**: scaffolds for all 6 R1+R2+ impossible-bug classes already present (`test/claim/impossible_bug/{suboptimal_complexity,idempotency_contract,transport_type_drift,nested_optional_flatten,unenumerated_effects,unhandled_diagnostic_paths}.dag`); diagnostic_correction/ + algebra_laws/ directories ready for fill-in.

**Why bundled**: TestClaim corpus is one cohesive workstream; the coverage lens (T-18) enforces completeness structurally.

**Modeling decisions**:
- TestClaim shape per concern (input/expected/falsification triple)
- Demonstration vs verification — impossible-bug TestClaims are demos for the thesis claim; algebra-laws are testgen-derived; diagnostic_correction is end-to-end demos
- Fixture corpus shape (per-stage vs end-to-end?)

**Reference**:
- v3 TestClaim demonstration: `src/v3/compiler/tests/dag/t_r3_tests_as_data_demonstration.dag`

**Why**: test infra port + fixture authoring. TestClaim data lives here.

**Modeling decisions**:
- Fixture corpus shape (how many fixtures? per-stage vs end-to-end?)
- TestClaim coverage discipline (every Diagnostic path covered)

**Reference**:
- v3 TestClaim demonstration: `src/v3/compiler/tests/dag/t_r3_tests_as_data_demonstration.dag`

---

### T-15: bin/main.dag + self-host fixed-point validation (the anti-regression gate)

**Why last**: validates the whole stack. v4 compiles itself, produces bit-identical output, ships.

**Reframe (operator 2026-05-15)**: T-15's `BitIdentical` assertion is not just a self-host check — it IS the structural anti-regression guarantee. The v4 binary is a content-addressed release artifact; its fixed-point hash is pinned; any change rebuilds and must reproduce the exact hash or CI goes red. "Off Rust" is cashed here: the only editable authority is `.dag`; Rust cannot regress because none is authored and the binary hash is structurally locked. Consumes `workflow/bootstrap.dag` (T-20) for the orchestration.

**Modeling decisions**:
- `bin/main.dag` trampoline shape (1-line `include!()`; 0-floor per design-pure-bootstrap-zero.md:210)
- Fixed-point check: stage1-emitted == stage2-emitted (NOT stage0==stage1 — stage0 is v2-emission-style)
- Content-addressing scheme for the pinned binary hash
- CI gate shape: rebuild-from-.dag-via-frozen-seed must reproduce pinned hash

**Falsification probe — what "bit-identical self-host failure" looks like as TestClaim**:

```
data t_15_self_host_fixed_point: TestClaim {
  kind: BitIdentical,
  label: "v4 compiler is a fixed point — iteration N matches iteration N+1",
  input: compile(src/v4/compiler/*.dag, target=Rust),  // iteration N+1
  expected: <committed v4 stage binary bytes>          // iteration N
}
```

Failure modes the probe MUST catch (each enumerable, each testable):
- **Non-determinism**: HashMap-iteration-order dependency in emit → different bytes between compilations
- **Hidden state**: global/static/ambient capability used in compiler logic → bytes vary with build environment
- **Test-double leakage**: mock or test scaffold loaded at compile-time → bytes differ when test toolchain absent
- **Substrate drift**: worker silently changed a substrate type without ratification → bytes differ from N to N+1

Once T-15 lands and stays green, all four failure modes are impossible-by-construction. A CI gate runs `cargo test t_15_self_host_fixed_point` per-PR on the v4 affected-set.

**Definition of v4-done**:
- **Every other task in this plan complete** — all of T-1..T-24 plus
  T-4.5/4.6/4.7/4.8, i.e. every task except T-15 itself. (Drift-proof
  phrasing: NOT a hardcoded count — the close gate requires the whole
  plan, never a stale number that omits in-scope work.)
- v4 compiles `src/v4/compiler/*.dag` end-to-end
- v4 emits Rust source that compiles to a binary
- That binary, run on `src/v4/compiler/*.dag`, produces bit-identical output
- TestClaim suite passes
- Hand-authored Rust is **not the editable authority** — proven by REPRODUCTION, not a count (A3): rebuild-from-(.dag + frozen-pinned seed)-only reproduces the pinned hash; the seed's own hash matches its pin. (The old "count = 0" phrasing was the gameable v3 proxy — replaced. The machine-emitted trampoline is build-dir-transient, never authority.) The check is an early-surfacing amplifier run per-PR on the affected set, not an un-gameability claim.

### T-4.6: extdeps/formats/* (json/yaml/csv/toml/json_schema/openapi)

**File**: 6 files in `src/v4/extdeps/formats/` (operator-ratified 2026-05-15: arbitrary ingestion via direction-agnostic format models)
**Why bundled**: identical structural shape per format; each file declares the format MODEL (data structure + parse/emit operations).

**Modeling decisions**:
- Recursive vs iterative parsing strategy (per-format)
- Number model (RFC 8259 §6 for JSON: arbitrary-precision OR IEEE-754; v4 default + opt-in)
- Schema-to-type derivation (json_schema.dag): given a schema, generate corresponding `.dag` types via `schema_to_type` operation
- Anchor/Alias resolution (yaml.dag): YAML's structure-sharing must resolve before producing typed value
- Dialect handling (csv.dag): delimiter/quote/escape/line-terminator parameterization

**Scope**: M-L (medium-to-large; six files but each is bounded by its anchored spec)

---

### T-4.7: extdeps/frameworks/react.dag

**File**: `src/v4/extdeps/frameworks/react.dag` (operator-ratified 2026-05-15: React framework substrate; coupled with T-16 full-stack demo)
**Why solo**: framework substrates are conceptually rich (Component / Hook / Effect / Lifecycle); React is the load-bearing first.

**Modeling decisions**:
- Hook-as-substrate: `HookKind = Builtin(BuiltinHook) | Custom(Node)`; `BuiltinHook` = the COMPLETE react.dev built-in set (no "..." — see react.dag header); custom hooks are Node composition per Rules-of-Hooks, not a new kind
- Effect lifecycle modeling (Mount / Unmount / DependencyChange / EveryRender)
- Rules-of-Hooks discipline (lens-checkable: no Hooks in conditionals — surface as Diagnostic)
- Component composition (props-down, events-up; structural propagation through Node tree)
- Server Components vs Client Components distinction (or unified via effect typing — effects intrinsic to the type signature, not an annotation)

**Scope**: L (large — substrate decisions cascade across full-stack demo T-16)

**Reference**:
- Anchor in file header (https://react.dev/reference/react)
- `docs/design-r4-full-stack-omni-emission-canvas.md` — 5-Q canvas (consult, do not block)

---

### T-4.8: extdeps/coordination.dag

**File**: `src/v4/extdeps/coordination.dag` (operator-ratified 2026-05-15 IN-B: Bind composition + effect typing — effects intrinsic to the type signature, NOT an annotation layer; NO 6th L1 behavior)
**Why solo**: multi-program coordination is the most consequential effect-typing in v4 — discipline matters.

**Modeling decisions**:
- Endpoint shape (NetworkAddress + LanguageRef + optional FrameworkRef)
- DeploymentUnit = collection of Endpoints + WireContracts between them
- WireContract = typed interface between two endpoints + CoordinationSemantics
- CoordinationSemantics = Sync | Async(SettleBound) | Stream | PubSub | EventuallyConsistent(ConvergeBound) (closed enum — operator-ratified C1 closure per node.dag discipline; non-immediate-settlement variants carry their bound as a STRUCTURAL field per operator fork 2026-05-15, read deterministically by the testgen simulator arm — see coordination.dag header)
- Effect-typing: HttpEffect, QueueEffect, StreamEffect, PubSubEffect — each is a typed parameter to Bind
- Failure-at-boundary modeling (composes with std/diagnostic.dag — no silent partial-failure)
- Idempotency at endpoint (composes with lens/idempotency.dag)

**Scope**: L (large — substrate decisions affect every distributed-app demo)

**Discipline**: NO 6th L1 behavior. If during work the temptation surfaces to add a `Coordinate` behavior to `std/node.dag`, STOP and escalate. The IN-B decision (operator 2026-05-15) is binding — coordination IS Bind composition + effect typing (effects intrinsic to the type signature, NOT an annotation layer).

---

### T-4.9 … T-4.14 — architecture stress probes (operator-ratified 2026-05-15)

**Parallel** tasks (need only T-1 + the B2-OMNI `LanguageModel` contract; independent of each other and of T-4). Their value is that they are **maximally diverse on purpose** — each is a *falsification probe* for the B2-OMNI O(N+M) claim. If adding one is genuinely O(1) (one declarative model, instantly cross-composing through the Node pivot), B2-OMNI is empirically validated; if any forces a core/pipeline change, B2-OMNI is leaking — surfaced now, before it is load-bearing. T-4.9-4.11 span the *upper* stack (HDL / netlist / NL boundary); T-4.12-4.14 span *down* the stack (IR / machine code / GPU) — together they validate the model across the **full target spectrum**: source (F may include cosmetics by intent) → IR (F structural) → machine code (F = encoding, no cosmetics). Long-held v2 intent, de-deferred per the frontload-the-hard-cases discipline.

#### T-4.9: `extdeps/languages/verilog.dag`
- **Stress axis**: hardware **concurrency** vs the 5 L1 behaviors. This is the **IN-B validation probe** — if Verilog (`always @(posedge clk)`, continuous assignment) cannot be modeled as effect-typed `Bind` composition without a 6th `Concurrent` behavior, that is a **C1 stop-signal escalation**, and catching it early is the entire point.
- **Clear win**: one `.dag` FSM → simulable Verilog + a Rust reference model, same Node, zero translator.
- **Scope**: L (substrate-validating; concurrency model is the risk).

#### T-4.10: `extdeps/formats/spice.dag`
- **Stress axis**: is the format/`LanguageModel` abstraction *actually* general, or secretly programming-language-shaped? A SPICE netlist has **no control flow** — components + a connection graph.
- **Clear win**: one `.dag` circuit declaration → a SPICE netlist that simulates (omni-emission reaches analog hardware).
- **Placement (operator-ratified fork)**: `extdeps/formats/` — a netlist is a data format, not a programming language (sibling of csv/json), Shape B.
- **Scope**: M-L.

#### T-4.11: `test/claim/boundary/english_ingest_fail_closed.dag`
- **Framing (operator-ratified fork)**: English is **NOT a language model** (no formal grammar). It is a **boundary-honesty probe**, not `extdeps/languages/english.dag`.
- **Stress axis**: the C5 lossless-core boundary at its extreme, and the no-engine thesis made visible.
- **Clear win**: (a) Shape B emit — `.dag` → English docs (≈ T-16's existing Markdown artifact, no new substrate); (b) the honest win — `ingest(English prose)` → a precise Diagnostic, **never a fabricated parse**. The architecture refusing to lie *is* the demonstrable result.
- **Scope**: M (the substrate it needs already exists; the claim is the work).

#### T-4.12: `extdeps/languages/llvm_ir.dag`
- **Stress axis**: does the model generalize **down** the abstraction stack? SSA form / dominance / phi is structurally unlike a source AST. F is ~all-structural (LLVM IR has negligible cosmetic surface — the clean contrast point for C5-fidelity).
- **Clear win**: `.dag → LLVM IR` (LLVM lowers to machine code) + `LLVM IR → Node` ingest — the down-stack half of O(N+M).
- **Anchor**: LLVM Language Reference Manual, pinned release (L-2).
- **Scope**: L.

#### T-4.13: `extdeps/languages/machine_code.dag`
- **Stress axis**: the **bottom of the stack** — no cosmetic surface at all (the limit test for C5-fidelity), and **disassembly is the extreme fail-closed case** (most byte runs are not valid instructions; a disassembler that guesses = the no-engine violation made visible).
- **Fork (PROPOSED — confirm)**: ONE `machine_code.dag` parameterized by an `Isa` model (recommended — parameterize, don't enumerate per-ISA; matches B2-OMNI/O(N+M)) vs per-ISA files.
- **Anchor**: the ISA spec (Intel 64 SDM / Arm ARM), pinned revision (L-2).
- **Scope**: L.

#### T-4.14: `extdeps/languages/ptx.dag` (CUDA)
- **Stress axis**: the **SIMT data-parallel execution model** vs the 5 L1 behaviors — the IN-B bet again (like Verilog's concurrency, but data-parallel). A needed 6th `Parallel`/`Kernel` behavior = C1 escalation, by design caught early.
- **Fork (PROPOSED — confirm)**: model **PTX** (the spec'd IR — clean, general, captures SIMT directly, parallel to llvm_ir; recommended) vs CUDA-C++ as a `cpp.dag` extension (entangled; the C++ surface is not where the stress is).
- **Anchor**: NVIDIA PTX ISA spec, pinned version (L-2).
- **Scope**: L.

---

### T-16: Full-stack omni-emission demo

**Output**: ONE `.dag` program → multi-language multi-endpoint application
**Operator framing 2026-05-15**: "consider pipeline emission i.e. 'backend program using react in the frontend (and say rust/C++ in the backend)' — i suggest we frontload this style of work — this is exactly what we keep deferring"

**Deliverable**: a single .dag file declaring a TODO-app-class application that emits:
- Rust backend (+ optionally C++ backend variant)
- React/TypeScript frontend
- OpenAPI wire contract between backend and frontend
- SQL DDL for persistence
- Markdown docs

All 5 artifacts share ONE Node tree (per gate #28 omni_layers_share_one_node_tree); coherence is structural, not test-checked.

**Modeling decisions**:
- How does the .dag file express endpoint partitioning (which fragment runs where)? (uses extdeps/coordination.dag's Endpoint + DeploymentUnit)
- Wire contract derivation (does it auto-derive from shared types, or is it explicitly declared?)
- Cross-target consistency: same domain types in Rust + TypeScript — tested via L5

**Scope**: XL (extra-large — this is the visceral cash of the omni-emission thesis)

**Why this task is the v4-flagship demo**: per operator "this is exactly what we keep deferring" — v4 fronts loading it because it forces the substrate decisions (T-4.7 React, T-4.8 coordination) to be made well, not as afterthoughts.

---

### T-17: lens/synthesis.dag + std/report.dag — cross-algorithm complexity (C7)

**File**: 2 files (operator-ratified 2026-05-15 IN: cross-algorithm complexity synthesis lens)
**Why bundled**: the synthesis lens is the consumer of the Report advisory carrier; both must land together for the lens to have anything to emit.

**Scope**: **XL** — research-tier risk **collapsed by the C2 reframe** (operator-ratified 2026-05-15). No undecidable equivalence engine, no unbounded rule library. Bounded to encoding the closed `LowerBoundTechnique` set + a few honest worked examples. STOP-and-escalate still applies if a relation class demands a 6th technique.

**Modeling decisions** (C2-reframed):
- **No semantic equivalence, no pattern library.** Synthesis reads the user's **declared** I/O relation (its contract/type — declared, never inferred; this dissolves the Rice-undecidability collision). It does NOT prove two programs equivalent and does NOT match against a `(naive→better)` catalogue (engine-shaped + unbounded — `feedback_no_engine`).
- **`LowerBoundTechnique` = closed set, enumerated up front**: `DecisionTree | AlgebraicRank | AdversaryCommunication | InformationTheoretic | ReductionConditional`. Each is a *general algebraic property over a relation class*, encoded once; adding the Nth algorithm adds ZERO entries.
- Synthesis = `compare(cost-lens-derived cost of the user's realization, the declared relation's lower bound derived via the technique set)`. No applicable technique ⇒ helpful Diagnostic (honest, never fabricated — `feedback_no_engine`).
- Report carrier shape (`std/report.dag`): closed-enum `ReportReason` disjoint from Diagnostic's `NamedReason`; advisory by construction; opt-in fail-closed via `apply_lens(synthesis, Enforce { ... })`.
- **Honest worked examples (for the worker later — illustrations of the technique→relation→lower-bound→compare flow, NOT a rule catalogue):**
  - *Sorting* — relation: "ordered permutation under a comparison oracle". Technique: `DecisionTree` ⇒ ≥ n! leaves ⇒ Θ(n log n). User Θ(n²) ⇒ Report the gap. (Merge-sort never named — the provable gap to optimum is surfaced, not a fix.)
  - *Matrix multiply* — relation: bilinear form. Technique: `AlgebraicRank` ⇒ naive n³ is rank-suboptimal vs n^ω. (Strassen never named; ω is open — the model surfaces structural suboptimality, refuses to fabricate an optimal.)
  - *Substring search* — relation: match positions over an n-length input. Technique: `InformationTheoretic` ⇒ Ω(n) (input must be read once); naive re-reads ⇒ Report the unforced re-scan. (KMP never named.)

**Reference**:
- `docs/r4-carve-out-routing.md` C7 — Director-tier design scope spec
- `lens/complexity.dag` — current-complexity input
- THESIS.md correctness dimensions §1.1 — complexity dimension parent
- INVARIANTS C-8 — fail-closed discipline (Report is the IS-NOT-fail-closed branch)

---

### T-19: lens/testgen.dag — producer of TestClaim corpus from substrate

**File**: `src/v4/lens/testgen.dag` (operator-ratified 2026-05-15: testgen as substrate fold; Phase 1.5 placement so test corpus exists before compiler stages need it)
**Why early**: per operator "i want testgen to be working fairly early — for the compiler itself". Phase 1.5 placement means T-6+ tasks consume testgen-derived TestClaims rather than hand-authoring.
**Why solo**: testgen is a producer with cross-cutting consumption of every substrate file; one cohesive home.

**Modeling decisions**:
- Generator<C> generic carrier shape — one lens, parameterized over substrate concept type
- Per-substrate-kind testgen rules (see file header for the 5 categories: type-construction / algebra-law / diagnostic-exhaustiveness / lens-applicability / bidirectional-roundtrip)
- TestClassification = (Tier, Layer) on every produced claim — Tier1/2/3 (correctness) × Unit/Integration/Boundary (test layer)
- Bootstrap path: hand-authored TestClaims in `test/claim/manual/` are the contract testgen must satisfy; coverage lens (T-18) enforces produced ⊇ manual

**Scope**: L (large — substrate-traversal across every concept; cross-cutting consumption)

**Bootstrap pragma** (per operator: "manual authoring is fine as well"):
- After T-1 (`std/node.dag`) lands: hand-author 5-10 TestClaims in `test/claim/manual/` covering type-construction for the 6 connectives + 5 behaviors. Validates schema + shape immediately.
- After T-2 (`std/algebra.dag`) lands: hand-author algebra-law TestClaims for at least Magma/Monoid.
- After T-19 implementation: testgen produces same set programmatically; manual claims become regression anchors.

**Reference**:
- TESTING.md §141 "Test layers (target ratios)" — Unit ~75% / Integration ~15% / Boundary ~10%
- THESIS.md §168-182 — correctness Tier 1/2/3
- THESIS.md §348-368 — "Tests are structural data"

---

### T-20: workflow/bootstrap.dag — bootstrap orchestration AS DATA

**File**: `src/v4/workflow/bootstrap.dag` (operator-ratified 2026-05-15: the "off Rust, can't regress" load-bearing file)
**Why early (Phase 1.5)**: the parse-viability step (v2 indexes src/v4) is needed from day 1 — it's the existing CI gate. The full self-host chain content grows incrementally as the pipeline matures. T-15 consumes the completed file for fixed-point validation.
**Why solo**: bootstrap orchestration is its own concern — it's the file that makes "compiler as data" structurally true rather than aspirational.

**Modeling decisions**:
- BootstrapPlan step sequence: seed (v2→stage0) / self0 (stage0→stage1) / self1 (stage1→stage2) / fixpt (assert stage1==stage2 BitIdentical)
- How v2's `run` interpreter executes this (the workflow is data v2 interprets, not Rust v2 compiles)
- Content-addressing of the pinned v4-stage-final binary
- Fail-closed on any step (compose with std/diagnostic.dag)

**Scope**: L (large — load-bearing for the entire anti-regression guarantee)

**The non-negotiable discipline**: this file is the ONLY bootstrap authority. A worker reaching for `build.rs` or `bootstrap.sh` has reintroduced editable Rust/script authority = the v3 regression door = STOP signal. v2 interprets this `.dag`; v2 is the frozen external seed (in `src/v2/`, outside `src/v4/`), touched exactly once per fresh bootstrap.

**Reference**:
- THESIS.md §223-226 — meta-process modeling ("Bootstrap ... modeled as .dag workflows")
- `docs/design-pure-bootstrap-zero.md` §"N=0 runtime boundary"
- STRUCTURE.md §"Bootstrap chain" + closed-system invariant 7

---

### T-18: lens/coverage.dag — meta-lens for coverage discipline

**File**: `src/v4/lens/coverage.dag` (operator-ratified 2026-05-15: structural coverage enforcement, not exhaustive fixtures)
**Why solo**: coverage discipline is its own concern — meta over the other lenses. One file owns the unified mechanism.

**Modeling decisions**:
- Coverage<C> generic carrier shape — one lens, parameterized over coverage concern (L6 form×target / L7 algebra×law×inhabitant / impossible-bug class enum / testgen type×inhabitant)
- Substrate read for each concern: derive EXPECTED set from substrate authority (not hand-enumerated)
- Comparison shape: actual TestClaim corpus vs expected derived set; emit Diagnostic per missing
- Composition with testgen: testgen produces TestClaims; coverage lens checks they cover the expected combinatorics
- Per operator: "make the target clear so we cannot bypass it this time" — the coverage lens MUST be structurally derived from substrate; cannot be opted-out, cannot be narrowed without substrate change

**Scope**: L (large — substrate-meta lens; multiple coverage concerns)

**Reference**:
- TESTING.md hermetic + behavior-driven discipline
- THESIS L6 §181 + L7 §182 + impossible-bug §370-413
- memory: feedback_no_textual_enforcement_bridges (coverage is structural, not grep-enforced)

---

### T-21: lens/affected_set.dag — incremental re-exec frontier

**File**: `src/v4/lens/affected_set.dag` (operator-ratified 2026-05-15: "something i wanted to get working very early on")
**Why early (Phase 1.5)**: load-bearing for incremental cross-run execution AND it is the structural replacement for `scripts/detect-affected-components.sh` (the interim shell bridge currently gating v2/v3/v4 CI selection).

**Modeling decisions**:
- `affected_set: (Dag, Diff) -> Witness<ReExecFrontier>` shape
- Diff representation (file-set? node-set? structural-delta over the Dag?)
- Purity-aware skipping: an unchanged pure subgraph is incrementally skippable; what makes a subgraph "unchanged" structurally?
- Composition with `compiler/05_eval.dag` (skip) and `workflow/ci.dag` (job selection)
- Structural caching is the **dual** of the affected set — the same mechanism. A build/exec artifact's cache key is `content_hash` (B1) of its input subgraph: the affected set names what re-runs, a cache restores what doesn't. Caching is not a separate system. The cache backend (GHA `actions/cache`, a remote build cache, a local memo table) is just an emission target of the hash.

**Scope**: L (large — load-bearing for incremental execution + CI dissolution)

**Reference**: THESIS §205-210 free consequences (incremental cross-run) + v4-close-interrogation.md §2.5.F + memory: feedback_no_textual_enforcement_bridges

---

### T-22: compiler/05_eval.dag — the interpreter (PRIMARY execution path)

**File**: `src/v4/compiler/05_eval.dag` (operator-raised 2026-05-15: "what about the interpreter")
**Why load-bearing**: THESIS:225 — `dag run` is THE primary execution path. eval is not an afterthought to emit; it is the default. Sibling of `05_emit.dag` (same `InferredTree` input; eval executes, emit projects to target languages).

**Modeling decisions**:
- `eval: (InferredTree, Inputs) -> Result<Value, Diagnostic>` shape
- Bounded-execution enforcement (INVARIANTS P4 — no unbounded loops; how does the evaluator structurally refuse non-termination?)
- The shared substrate three consumers compose over: `workflow/bootstrap.dag` (interpreted, not compiled), TestClaim evaluation, lens dry-run
- Concept-unification (THESIS:188): interpreter runtime = language spec = transport spec — eval reads the same `extdeps/languages/*.dag` carriers emit does

**Scope**: XL (extra-large — THE primary execution path; bootstrap + tests + dry-run all depend on it)

**Reference**: THESIS:225 + concept-unification THESIS:188 + STRUCTURE.md §"Bootstrap chain" (v2's eval seeds; v4's eval takes over)

---

### T-23: lens/application.dag — apply_lens surface (opt-in depth)

**File**: `src/v4/lens/application.dag` (closes prior-audit BLOCKING GAP 1)
**Why load-bearing**: `apply_lens(<lens>, Enforce { ... })` is referenced by `report.dag`, `synthesis.dag`, and the C7 advisory→blocking bridge — but had no substrate home until now. It is simultaneously: §1.5 user-defined-dimensions surface, §6.2 audience-duality opt-in-depth mechanism, and the ONLY advisory→fail-closed path.

**Modeling decisions**:
- `EnforcedApplication<Output, Budget>` vs `IntrospectApplication<Output>` carrier shapes (v3 T-Lens-Application-Surface precedent: two separate carriers, NOT a sum — per r3-structure.md:40)
- `SectionRef = DeclarationScope | NodeScope` (where a lens attaches)
- The advisory→fail-closed conversion: how `Enforce { }` turns a lens's `Set<Report>` into fail-closed Diagnostics (the single explicit bridge per `std/report.dag` discipline)
- Default policy: a function with no `apply_lens(<lens>, Enforce { ... })` declaration gets synthesized Introspect-only (no implicit Enforce) per THESIS:307-321 opt-in depth. `apply_lens` is a first-class declaration (a Node), not an annotation — absence of the declaration, not absence of a tag, is the default trigger.

**Scope**: L (large — connective tissue for three thesis claims)

**Reference**: THESIS:95-101 + THESIS:307-321 + r3-structure.md:40 (v3 precedent) + std/report.dag discipline

---

### T-24: workflow/ci.dag — CI pipeline AS DATA

**File**: `src/v4/workflow/ci.dag` (closes prior-audit BLOCKING GAP 2)
**Why load-bearing**: THESIS:223-226 — "adding a CI gate = editing one .dag file." v3's gate #98 `ci_yml_hand_authority_dissolved` was an open R3 gap precisely because CI YAML stayed hand-authored. v4 must not reproduce it.

**Modeling decisions**:
- `CiPipeline { jobs, gates }` shape
- `.github/workflows/ci.yml` as DERIVED Shape-B artifact (.dag walks CiPipeline, emits YAML)
- Affected-set-driven job selection consuming `lens/affected_set.dag` (T-21) — this is what dissolves `scripts/detect-affected-components.sh`
- Structural cache keys: a cacheable job's `actions/cache` key is `content_hash` (B1) of its input subgraph, not a hand-authored `hashFiles(...)` glob. The interim `hashFiles(...)` keys in the committed `ci.yml` (e.g. the v2-compiler-binary cache) are manual approximations, replaced by emitted content-hashes when `ci.yml` is emitted from this file.
- The bootstrap interaction: CI runs `workflow/bootstrap.dag` (T-20)

**Scope**: L (large — closes the v3 hand-authored-CI gap; dissolves the shell bridge)

**Reference**: THESIS:223-226 + v4-close-interrogation.md §3.2 + v3 gate #98 (the gap not to reproduce)

---

## Summary

Every task in this plan is a bounded, modeling-load-bearing pure function (the count is intentionally unstated — it drifts as scope is ratified; see T-15's drift-proof close gate). Gaming surface is structurally bounded because adding files / splitting files / reaching outside declared substrate all require operator escalation. Per zero-deferrals: "I'll just do this for now" is forbidden — STOP and escalate.

If a task hits an unmodelable case or escalations pile up, that's a substrate-design signal — STOP, re-model, do not paper over.

The release is when v4-done. Not before, not after.
