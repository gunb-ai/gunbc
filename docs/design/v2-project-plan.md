# v2 Self-Hosted Compiler: Project Plan

## Final deliverable

The v2 compiler (written in .dag, run by v1 during bootstrap)
compiles `gist.dag` and its transitive dependencies (~1100 lines
across 6 .dag files), emitting Rust source files equivalent to
v1's output.

**Acceptance test (three levels):**

```bash
# Level 1: v2 emits Rust that compiles
v2-compile gist.dag --backend rust --output /tmp/v2/
cd /tmp/v2/ && cargo build

# Level 2: v2 emits tests that pass
cd /tmp/v2/ && cargo test

# Level 3: v2 output matches v1 output
v1-compile gist.dag --backend rust --output /tmp/v1/
diff -r /tmp/v1/ /tmp/v2/
```

Level 1 proves the emitted code is valid Rust. Level 2 proves the
emitted program behaves correctly (the compiler emits tests alongside
the code). Level 3 proves v2 is compatible with v1 for bootstrap.

The compiler is a file-to-file transform. It reads .dag source and
writes target-language source files AND target-language test files.
There is no interpreter in the compiler. Interpretation and testing
are downstream — consumers of the emitted files.

---

## Workstreams

Two independent workstreams that merge at integration.

### Stream C: Compiler stages (in .dag)

The compiler itself: tokenizer, parser, module resolver, typechecker,
emitter. Each stage is a pure function testable independently.

### Stream T: Testing harness

Per-stage comparison infrastructure. Both v1-equivalence tests
(bootstrap correctness) AND spec-level invariant tests
(sustainability correctness — v2 must not reproduce v1's fail-open
behaviors).

---

## Phases

### Phase 1: Foundation (independent tasks)

#### C1: Core types (`v2/std/core.dag`)
The compiler's domain model for the **bootstrap subset**: Token,
Module, Expr, TypeExpr. Covers the 8 Item variants used by
gist.dag and its transitive dependencies: `module`, `import`,
`type`, `fn`, `func`, `service`, `resource`, `data`.

Does NOT cover v1-only Item variants that gist.dag doesn't use:
`PatternDef`, `InterfaceDef`, `PipelineDef`, `ProfileDef`,
`TestDef`, `FixtureDef`, `ProjectDef`, `FeatureDef`, `TaskDef`,
`DesignDef`, `ComponentDef`, `EnvironmentDef`, `ParamDecl`,
`ExternAssetDecl`. These are added incrementally as the bootstrap
target expands beyond gist.dag.

Key design decisions:
- `TypeExpr` is a structural value, not a string reference
- No `metadata: Map<String, String>` bags
- `SourceSpan` on every blameable node

**Acceptance:**
- v1 compiler parses core.dag without errors
- Every Item variant used by gist.dag's transitive deps has a
  v2 equivalent
- No string-typed fields where structural types exist

**Effort:** Small (mostly done, needs cleanup)
**Depends on:** Nothing

#### T1: Test infrastructure
Two test types:
1. **Equivalence tests:** compile with v1, compile with v2, diff
2. **Invariant tests:** v2 output has no unresolved type references,
   no string-typed metadata bags, no fabrication fallbacks

**Acceptance:** Framework runs on a trivial 1-function .dag file
**Effort:** Small
**Depends on:** Nothing

---

### Phase 2: Tokenizer + Parser

#### C2: Tokenizer (`v2/compiler/tokenize.dag`)
Pure function: `String → List<Token>`.

Keywords and punctuation as data tables. Must emit all token
kinds declared in core.dag, including:
- String interpolation parts (StrBegin/StrMid/StrEnd)
- Float literals
- Newline tokens (significant for the parser)

**Acceptance:**
- Tokenize all 6 gist-dependency .dag files
- Token stream (kinds + spans) matches v1 tokenizer output
- Every TokenKind variant in core.dag is either emitted or
  explicitly documented as unused by the test corpus

**Effort:** Small-Medium
**Depends on:** C1

#### C3: Parser (`v2/compiler/parse.dag`)
Pure function: `List<Token> → Module`.

Recursive descent. Must handle all Item variants:
- `module`, `import`
- `type` (record, sum, alias with refinement predicates)
- `fn`, `func`
- `service` with `operation`
- `resource` with `capability`
- `data` declarations
- All Expr variants used by gist.dag and its dependencies

**Acceptance:**
- Parse all 6 gist-dependency .dag files
- AST structure matches v1 parser (serialized JSON comparison)
- **Invariant test:** every TypeExpr in the parsed AST is
  structural (Named/Product/Coproduct/Container), never a raw
  string

**Effort:** Large (~2000 lines, biggest single task)
**Depends on:** C2

---

### Phase 3: Resolution + Typechecking (splittable)

#### C4: Module resolver (`v2/compiler/resolve.dag`)
Pure function: `List<Module> → ModuleGraph`.

Resolves import references, builds dependency order, detects
cycles.

**Acceptance:**
- Resolve gist.dag + 5 dependencies
- Import references resolve to correct modules
- Cycle detection catches circular imports (spec test with
  synthetic cycle)

**Effort:** Small
**Depends on:** C3

#### C5: Type resolver (`v2/compiler/typecheck.dag`)
Pure function: `ModuleGraph → TypedGraph`.

Resolves every `Named { name }` TypeExpr to its structural form.
After this stage, no TypeExpr in the graph is `Named` — everything
is `Product`, `Coproduct`, `Container`, `Refined`, etc.

No TypeRegistry. Resolution walks the module graph's type
definitions and substitutes structurally.

**Acceptance:**
- Resolve all types in gist.dag dependencies
- `ResourceHandle` resolves to Product with 4 fields
- `CommitSha` resolves to Refined { base: String, predicates: [Pattern] }
- Service operation types resolve to products
- **Invariant test:** no `Named` TypeExpr survives typecheck
  (all resolved to structural form)

**Effort:** Medium
**Depends on:** C4
**Parallelizable with:** C4 can be done by a different person,
C5 starts as soon as C4 is done

---

### Phase 4: Emission

#### C6: Rust emitter (`v2/compiler/emit.dag`)
Pure function: `TypedGraph → List<TextFile>`.

Emits BOTH Rust source files AND Rust test files from the typed
AST. The compiler owns its downstream — if it emits Rust, it also
emits the tests that prove the Rust works.

For bootstrap, needs to handle the constructs used by gist.dag:
- Function definitions → Rust functions + unit tests
- Service operations → transport call scaffolding + mock tests
- Type definitions → Rust structs/enums + construction tests
- Data declarations → Rust constants + value tests
- Expressions → Rust expression syntax

**Acceptance:**
- Emit Rust for gist.dag
- Emitted files compile with `cargo build`
- Emitted tests pass with `cargo test`
- Emitted files are functionally equivalent to v1's output
  (diff, ignoring formatting)
- **Invariant test:** no emitted file contains `unwrap()` without
  a clear error message, no `todo!()`, no `unimplemented!()`

**Effort:** Large (~2500 lines — code emit + test emit)
**Depends on:** C5

---

### Phase 5: Integration

#### I1: Pipeline wiring (`v2/compiler/pipeline.dag`)
Wire C2–C6 into the full pipeline: read files → tokenize → parse →
resolve → typecheck → emit.

**Acceptance:** The final deliverable acceptance test passes:
v2 compiles gist.dag and emitted files match v1's output

**Effort:** Small (plumbing)
**Depends on:** C6

---

## Parallelism map

```
         Person A              Person B
Week 1:  C1 (types)            T1 (test harness)
Week 1:  C2 (tokenizer)        T1 continued
Week 2:  C3 (parser)           (parser is large — both can pair)
Week 3:  C4 (resolver)         C5 (typechecker — can start from C4 types)
Week 3:  C6 (emitter)          C5 continued
Week 4:  I1 (integration)      invariant tests
```

**Critical path:** C1 → C2 → C3 → C4 → C5 → C6 → I1

**Parallelizable:** T1 runs independently. C4 and C5 overlap
partially (different concerns, same input shape).

**Estimated effort:** ~3 weeks for 1 person, ~2 weeks for 2.

---

## Per-task test strategy

Two test types per task:

| Task | Equivalence test (v1 = oracle) | Invariant test (spec) |
|------|------|------|
| C2 | Token stream matches v1 | All TokenKind variants covered or documented |
| C3 | AST matches v1 (JSON diff) | No raw-string TypeExprs in AST |
| C4 | Module graph matches v1 | Cycle detection works on synthetic input |
| C5 | Resolved types match v1 | No `Named` TypeExpr survives (all structural) |
| C6 | Emitted Rust compiles, emitted tests pass, matches v1 | No `unwrap_or`, no string-typed metadata, no `todo!()` |
| I1 | Full pipeline output matches v1 | All invariants hold end-to-end |

Equivalence tests ensure bootstrap correctness. Invariant tests
ensure v2 doesn't reproduce v1's sustainability problems.

---

## Risk register

| Risk | Impact | Mitigation |
|------|--------|------------|
| Parser too complex for .dag | Blocks C3 | Prototype string interpolation + operator precedence early |
| v1 missing intrinsics that v2 compiler needs | Blocks execution | Inventory builtins used by C2-C6; add to v1 eval if missing |
| v1/v2 output comparison too strict | False test failures | Normalize before comparison (sort keys, strip formatting) |
| Recursive types in AST (Expr contains Expr) | Design complexity | v1 handles this; same recursive descent in .dag |
| Emitter scope creep (too many Rust constructs) | Delays C6 | Scope to gist.dag's constructs only; add others incrementally |
