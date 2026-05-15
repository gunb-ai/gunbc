# v4 — XL Task Plan

20 XL tasks define "v4 done." Each task is a bounded modeling unit; each produces a typed pure function in declared files; each is honestly hard to game because the work IS the decisions.

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
  T-5   workflow/* (5 files)             [needs T-1; FIRST IN EXECUTION]

Phase 2 (serial — pipeline stages):
  T-6   compiler/01_tokenize.dag         [needs T-3]
  T-7   compiler/02_parse.dag            [needs T-6]
  T-8   compiler/03_normalize.dag + 03_resolve.dag   [needs T-7]
  T-9   compiler/04_infer.dag            [needs T-8, T-2, T-3]
  T-10  compiler/05_emit.dag + 00_compile.dag       [needs T-9, T-4]

Phase 3 (parallel — lens dimensions):
  T-11  emit per-target specialization (extends T-10 across all 5 Shape A targets)
  T-12  lens/complexity.dag + lens/cost.dag      [needs T-9]
  T-13  lens/{parallelism,effect,ownership,idempotency}.dag   [needs T-9]
  T-17  lens/synthesis.dag + std/report.dag  (cross-algorithm complexity, C7;
         XL scope, research-tier risk)              [needs T-12 for current-complexity input]

Phase 4 (serial — close the loop):
  T-14  test/claim/* + test/fixture/* (port load-bearing TestClaims from v3)
  T-15  bin/main.dag + bootstrap glue + self-host fixed-point validation
  T-16  Full-stack omni-emission demo: ONE .dag → Rust+C++ backend
        + React/TS frontend + OpenAPI wire contract
        [needs T-4, T-4.5, T-4.6, T-4.7, T-10, T-11]
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

**Modeling decisions**:
- Per-language primitive inhabitance (i32 -> OrderedRing, std::vector<T> -> List<T>, etc.)
- Per-language realization cost shape
- Grammar encoding (declarative production rules vs procedural recognizer)
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
- Retirement predicate: how does `retired(HandResidual)` cash structurally?
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

### T-14: test/claim/* + test/fixture/*

**Why**: test infra port + fixture authoring. TestClaim data lives here.

**Modeling decisions**:
- Fixture corpus shape (how many fixtures? per-stage vs end-to-end?)
- TestClaim coverage discipline (every Diagnostic path covered)

**Reference**:
- v3 TestClaim demonstration: `src/v3/compiler/tests/dag/t_r3_tests_as_data_demonstration.dag`

---

### T-15: bin/main.dag + bootstrap glue + self-host validation

**Why last**: validates the whole stack. v4 compiles itself, produces bit-identical output, ships.

**Modeling decisions**:
- main.rs trampoline shape (1-line `include!()`)
- Bootstrap-stage progression (v2 binary -> v4 first compile -> v4 self-compile)
- Self-host fixed-point check

**Definition of v4-done**:
- All 14 prior tasks complete
- v4 compiles `src/v4/compiler/*.dag` end-to-end
- v4 emits Rust source that compiles to a binary
- That binary, run on `src/v4/compiler/*.dag`, produces bit-identical output
- TestClaim suite passes
- Hand-authored Rust count = **0** (excluding the machine-emitted trampoline)

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
- Hook-as-substrate: HookKind closed enum (UseState | UseEffect | UseMemo | UseRef | UseContext | ...)
- Effect lifecycle modeling (Mount / Unmount / DependencyChange / EveryRender)
- Rules-of-Hooks discipline (lens-checkable: no Hooks in conditionals — surface as Diagnostic)
- Component composition (props-down, events-up; structural propagation through Node tree)
- Server Components vs Client Components distinction (or unified with Effect annotation)

**Scope**: L (large — substrate decisions cascade across full-stack demo T-16)

**Reference**:
- Anchor in file header (https://react.dev/reference/react)
- `docs/design-r4-full-stack-omni-emission-canvas.md` — 5-Q canvas (consult, do not block)

---

### T-4.8: extdeps/coordination.dag

**File**: `src/v4/extdeps/coordination.dag` (operator-ratified 2026-05-15 IN-B: Bind composition + Effect annotation; NO 6th L1 behavior)
**Why solo**: multi-program coordination is the most consequential effect-typing in v4 — discipline matters.

**Modeling decisions**:
- Endpoint shape (NetworkAddress + LanguageRef + optional FrameworkRef)
- DeploymentUnit = collection of Endpoints + WireContracts between them
- WireContract = typed interface between two endpoints + CoordinationSemantics
- CoordinationSemantics = Sync | Async | Stream | PubSub | EventuallyConsistent (closed enum — operator-ratified C1 closure per node.dag discipline)
- Effect-typing: HttpEffect, QueueEffect, StreamEffect, PubSubEffect — each is a typed parameter to Bind
- Failure-at-boundary modeling (composes with std/diagnostic.dag — no silent partial-failure)
- Idempotency at endpoint (composes with lens/idempotency.dag)

**Scope**: L (large — substrate decisions affect every distributed-app demo)

**Discipline**: NO 6th L1 behavior. If during work the temptation surfaces to add a `Coordinate` behavior to `std/node.dag`, STOP and escalate. The IN-B decision (operator 2026-05-15) is binding — coordination IS Bind composition + Effect annotation.

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

**Scope**: **XL, research-tier risk** — substrate additions cascade; semantic-equivalence representation, pattern-recognition substrate, transformation-rule library are all substrate-design decisions individually. STOP-and-escalate discipline applies fully.

**Modeling decisions**:
- Semantic equivalence: how is "two programs compute the same I/O relation" represented structurally? Pure-function input/output specifications? Algebraic-axiom-preserving rewrites? Tree-rewriting under a typed equivalence?
- Pattern-recognition substrate: how does the lens match user program against algorithm-class templates? (Template matching? Constraint solving? Structural unification?)
- Transformation rule library: bubble-sort → merge-sort, naive-matmul → Strassen, naive-string-match → KMP, etc. — encoded as algebraic rewrites OR (input-shape → output-shape) pairs OR named patterns
- Report carrier shape (`std/report.dag`): closed-enum `ReportReason` disjoint from Diagnostic's `NamedReason`; advisory by construction; opt-in fail-closed via `apply_lens(synthesis, Enforce { ... })`
- Composition with lens/complexity.dag: synthesis reads current program's complexity (via complexity lens), produces Report with proposed-algorithm-complexity for comparison

**Reference**:
- `docs/r4-carve-out-routing.md` C7 — Director-tier design scope spec
- `lens/complexity.dag` — current-complexity input
- THESIS.md correctness dimensions §1.1 — complexity dimension parent
- INVARIANTS C-8 — fail-closed discipline (Report is the IS-NOT-fail-closed branch)

---

## Summary

20 XL tasks. Every task is a bounded, modeling-load-bearing pure function. Gaming surface is structurally bounded because adding files / splitting files / reaching outside declared substrate all require operator escalation. Per zero-deferrals: "I'll just do this for now" is forbidden — STOP and escalate.

If a task hits an unmodelable case or escalations pile up, that's a substrate-design signal — STOP, re-model, do not paper over.

The release is when v4-done. Not before, not after.
