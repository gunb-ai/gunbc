# v4 — XL Task Plan

15 XL tasks define "v4 done." Each task is a bounded modeling unit; each produces a typed pure function in declared files; each is honestly hard to game because the work IS the decisions.

## Execution graph

```
Phase 1 (parallel — substrate foundation):
  T-1   std/node.dag                     [BLOCKS: all]
  T-2   std/algebra.dag                  [needs T-1]
  T-3   std/* supporting (6 files)       [needs T-1]
  T-4   extdeps/languages/{rust,python,go}.dag   [needs T-1, T-2]
  T-5   workflow/* (5 files)             [needs T-1; FIRST IN EXECUTION]

Phase 2 (serial — pipeline stages):
  T-6   compiler/01_tokenize.dag         [needs T-3]
  T-7   compiler/02_parse.dag            [needs T-6]
  T-8   compiler/03_normalize.dag + 03_resolve.dag   [needs T-7]
  T-9   compiler/04_infer.dag            [needs T-8, T-2, T-3]
  T-10  compiler/05_emit.dag             [needs T-9, T-4]

Phase 3 (parallel — lens dimensions):
  T-11  emit per-target specialization (extends T-10 across rust/python/go)
  T-12  lens/complexity.dag + lens/cost.dag      [needs T-9]
  T-13  lens/{parallelism,effect,ownership,idempotency}.dag   [needs T-9]

Phase 4 (serial — close the loop):
  T-14  test/claim/* + test/fixture/* (port load-bearing TestClaims from v3)
  T-15  bin/main.dag + bootstrap glue + self-host fixed-point validation
```

## Task definitions

### T-1: std/node.dag — substrate root

**File**: `src/v4/std/node.dag`
**Estimate**: 3-5 days
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
**Estimate**: 3-5 days
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
**Estimate**: 5-7 days (bundle)
**Why bundled**: smaller individually, all interrelated, foundation for everything.

**Modeling decisions per file** (see file headers for specifics).

**Reference**:
- v3 mirrors of each (study for design, audit for honesty)
- TestClaim schema: import directly from `dsl/std/verification.dag:38`

---

### T-4: extdeps/languages/{rust,python,go}.dag

**File**: 3 files in `src/v4/extdeps/languages/`
**Estimate**: 4-6 days (bundle — same shape per target)
**Why bundled**: identical structural shape per target; the SHAPE is the work.

**Modeling decisions**:
- Per-target primitive inhabitance (i32 -> OrderedRing, etc.)
- Per-target realization cost shape
- Emission rule encoding (declarative spec vs procedural)

**Reference**:
- v2: `src/v2/languages.dag`
- v3: `dsl/extdeps/languages/` (audit each for honesty)

---

### T-4.5: extdeps/process.dag + extdeps/file_system.dag

**File**: 2 files in `src/v4/extdeps/`
**Estimate**: 3-5 days (bundle — both are OS contracts modeled per their canonical anchors)
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
**Estimate**: 5-7 days
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

**Estimate**: 3-5 days
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

**Estimate**: 5-7 days (the parser is real work)
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

**Estimate**: 5-7 days (bundle)
**I/O**: `ParseTree -> NormalizedTree -> ResolvedTree`

**Modeling decisions**:
- Surface sugar dissolution rules (service/fn/type -> Node tree)
- Identifier binding strategy (scope chain vs flat namespace)

**Reference**:
- v2: `src/v2/03_normalize.dag`, `src/v2/03_resolve.dag`

---

### T-9: compiler/04_infer.dag

**Estimate**: 7-10 days (the meat — type inference is hard)
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

**Estimate**: 5-7 days (bundle — orchestrator is the trivial wiring of the stages)
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

**Estimate**: 5-7 days
**Why separate from T-10**: T-10 is the orchestrator; T-11 is the per-target translation tables that populate emit's behavior across rust/python/go.

**Modeling decisions**:
- Per-target translation rules
- Target-specific optimizations (or absence thereof)

---

### T-12: lens/complexity.dag + lens/cost.dag

**Estimate**: 6-8 days (bundle — closely related)
**I/O**: `Node -> Witness<ComplexityBound>`, `Node -> Witness<SymbolicCost>`

**Modeling decisions**:
- Complexity class encoding
- SymbolicCost lattice shape (per `docs/audit/sub-value-relation-bounded-lattice-claim.md`)
- Composition with Sum/Product algebra

---

### T-13: lens/{parallelism,effect,ownership,idempotency}.dag

**Estimate**: 6-8 days (bundle — smaller per-lens)
**I/O**: `Node -> Witness<...>` per lens

**Modeling decisions per lens** (see file headers).

---

### T-14: test/claim/* + test/fixture/*

**Estimate**: 4-6 days
**Why**: test infra port + fixture authoring. TestClaim data lives here.

**Modeling decisions**:
- Fixture corpus shape (how many fixtures? per-stage vs end-to-end?)
- TestClaim coverage discipline (every Diagnostic path covered)

**Reference**:
- v3 TestClaim demonstration: `src/v3/compiler/tests/dag/t_r3_tests_as_data_demonstration.dag`

---

### T-15: bin/main.dag + bootstrap glue + self-host validation

**Estimate**: 4-6 days
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

## Summary

15 tasks. Roughly 6-10 weeks at 2-3 parallel workers. Every task is a bounded, modeling-load-bearing pure function. Gaming surface is structurally bounded because adding files / splitting files / reaching outside declared substrate all require operator escalation.

If a task overruns or escalations pile up, that's a substrate-design signal — STOP, re-model, do not paper over.

The release is when v4-done. Not before, not after.
