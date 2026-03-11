# v2 Self-Hosted Compiler: Project Plan

## Final deliverable

The v2 compiler (written in .dag, executed by the Rust kernel)
successfully compiles `gist.dag` and its transitive dependencies
(~1100 lines across 6 .dag files), producing the same DAG output
as the v1 Rust compiler.

**Acceptance test:**
```bash
# v1 compiles gist.dag
cargo run -p gunbc-codegen --bin gunbc-testgen -- gist > /tmp/v1-output.json

# v2 compiles gist.dag (executed by kernel)
cargo run --bin dag-kernel -- src/v2/compiler/pipeline.dag \
  --entry compile --arg root=dsl/gunbc/tools/gist.dag > /tmp/v2-output.json

# Outputs match
diff /tmp/v1-output.json /tmp/v2-output.json
```

When these match, the v2 compiler is functionally correct for a
real-world DSL program with imports, types, services, resources,
pure functions, and string manipulation.

---

## Workstreams

Three independent workstreams that can run in parallel, merging at
integration milestones.

### Stream K: Kernel (Rust runtime)

The minimal Rust program that loads and executes a serialized DAG.
This is the only Rust code in v2's runtime path.

### Stream C: Compiler stages (in .dag)

The compiler itself: tokenizer, parser, type resolver, lowerer.
Each stage is a pure function testable independently.

### Stream T: Testing harness

Golden-file comparison infrastructure and per-stage test fixtures.
Ensures each stage produces correct output before integration.

---

## Phases

### Phase 1: Foundation (no dependencies between tasks)

#### K1: DAG serialization format
Define a JSON schema for serialized DAGs (nodes, edges, ports with
TypeExpr). Write serialize/deserialize in Rust.

**Input:** `Dag<LoweredOp>` (v1 IR)
**Output:** JSON file
**Acceptance:** Round-trip: serialize → deserialize → re-serialize = identical
**Effort:** Small
**Depends on:** Nothing

#### K2: Kernel executor
Rust binary that loads a serialized DAG and executes it. Supports
`Value` types, topological execution, edge routing, transport
dispatch (Shell, File).

**Input:** Serialized DAG JSON + entry function + arguments
**Output:** Execution result (Value)
**Acceptance:** Execute a hand-written 5-node DAG with string
manipulation and produce correct output
**Effort:** Medium (extract from v1 exec, strip DynOp dispatch)
**Depends on:** K1

#### C1: Core types (`v2/std/core.dag`)
The compiler's domain model: Token, AST, Expr, TypeExpr, DAG IR,
CompileOutput. Already sketched — needs review and refinement.

**Input:** v1 AST/IR types as reference
**Output:** `src/v2/std/core.dag`
**Acceptance:** v1 compiler parses this file without errors; every
v1 AST type has a v2 equivalent
**Effort:** Small (mostly done)
**Depends on:** Nothing

#### T1: Golden-file test infrastructure
Script/tool that compiles a .dag file with v1, serializes the output,
and compares against a golden file. Used by all subsequent phases.

**Input:** .dag file path
**Output:** PASS/FAIL + diff on mismatch
**Acceptance:** Works for a trivial 1-function .dag file
**Effort:** Small
**Depends on:** K1

---

### Phase 2: Tokenizer + Parser (C1 → C2 → C3)

#### C2: Tokenizer (`v2/compiler/tokenize.dag`)
Pure function: `String → List<Token>`. Keywords and punctuation as
data tables, not match arms.

**Input:** Source string
**Output:** `List<Token>` with spans
**Acceptance:** Tokenize all 6 gist-dependency .dag files; output
matches v1 tokenizer's token stream (kinds + spans)
**Effort:** Small (mostly done)
**Depends on:** C1

#### C3: Parser (`v2/compiler/parse.dag`)
Pure function: `List<Token> → Module`. Recursive descent. Must handle:
- `module`, `import`
- `type` (record, sum, alias with predicates)
- `fn`, `func`
- `service` with `operation`
- `resource` with `capability`
- `data` declarations
- Expressions: let, match, if/else, field access, function call,
  lambda, string interpolation, binary operators, record/list literals

**Input:** `List<Token>`
**Output:** `Module` (AST)
**Acceptance:** Parse all 6 gist-dependency .dag files; AST structure
matches v1 parser output (compared as serialized JSON)
**Effort:** Large — this is the biggest single task (~2000 lines)
**Depends on:** C2

---

### Phase 3: Type resolution + Lowering (can split across people)

#### C4: Module resolver (`v2/compiler/resolve.dag`)
Pure function: `List<Module> → ModuleGraph`. Resolves import
references, builds dependency order, detects cycles.

**Input:** `List<Module>` (parsed ASTs from multiple files)
**Output:** `ModuleGraph` (ordered modules with resolved references)
**Acceptance:** Resolve gist.dag + 5 dependencies; import references
resolve to correct modules; cycle detection catches circular imports
**Effort:** Small
**Depends on:** C3

#### C5: Type resolver (`v2/compiler/typecheck.dag`)
Pure function: `ModuleGraph → TypedGraph`. Resolves type references
to structural TypeExpr values. Validates field types, function
signatures, service operation types.

No TypeRegistry. No TypeId. Types flow as TypeExpr values.

**Input:** `ModuleGraph`
**Output:** `TypedGraph` (modules + resolved TypeExpr on every reference)
**Acceptance:** Resolve all types in gist.dag dependencies;
`ResourceHandle` has 4 fields; `CommitSha` is a refined String;
service operation types resolve to products
**Effort:** Medium
**Depends on:** C4

#### C6: Lowerer (`v2/compiler/lower.dag`)
Pure function: `TypedGraph → DAG`. Maps each AST item to DAG nodes:
- `type` → TypeNode
- `fn`/`func` → SubGraph node with input/output ports
- `service.operation` → prepare/execute/parse triplet
- `resource` → acquire/release pair
- `data` → PureExpr node with literal value
- Expressions → edges between nodes

**Input:** `TypedGraph`
**Output:** `DAG` (nodes + edges)
**Acceptance:** Lower gist.dag; output DAG has the same node IDs,
port names, and edge topology as v1's `Dag<LoweredOp>` (compared
as serialized JSON, ignoring field ordering)
**Effort:** Large (~3000 lines, but mechanical)
**Depends on:** C5

---

### Phase 4: Integration

#### I1: End-to-end pipeline (`v2/compiler/pipeline.dag`)
Wire C2–C6 into the full pipeline: read files → tokenize → parse →
resolve → typecheck → lower → serialize output.

**Input:** Root .dag file path
**Output:** Serialized DAG JSON
**Acceptance:** `pipeline.dag` compiles gist.dag and produces the
same serialized DAG as v1
**Effort:** Small (plumbing)
**Depends on:** C6, K2

#### I2: Kernel runs v2 compiler
The kernel (K2) executes the v2 compiler pipeline DAG, which compiles
gist.dag. End-to-end: Rust kernel → v2 compiler (in .dag) → gist DAG.

**Acceptance:** The final deliverable acceptance test passes:
v1 output == v2 output for gist.dag
**Effort:** Small (integration testing)
**Depends on:** I1

---

## Parallelism map

```
Week 1:          K1    C1    T1        (all independent)
Week 1-2:        K2    C2              (K2 needs K1; C2 needs C1)
Week 2-3:              C3              (parser — largest task)
Week 3:          C4         C5         (can split: resolver + typecheck)
Week 3-4:              C6              (lowerer — second largest)
Week 4:               I1    I2         (integration)
```

**Critical path:** C1 → C2 → C3 → C4 → C5 → C6 → I1 → I2

**Parallelizable:**
- K1, K2 run independently of C-stream (merge at I1)
- T1 runs independently (used by all C tasks for validation)
- C4 and C5 can overlap if C3 is done (different concerns)

**Estimated total effort:** ~3-4 focused weeks for 1 person,
~2 weeks with 2 people (K-stream + C-stream in parallel)

---

## Per-task test strategy

Each task has a concrete acceptance test that can be run independently:

| Task | Test | Comparison |
|------|------|------------|
| K1 | Round-trip serialize/deserialize | Byte-identical JSON |
| K2 | Execute 5-node DAG | Expected output values |
| C1 | v1 parses core.dag | No parse errors |
| C2 | Tokenize 6 .dag files | Token stream matches v1 |
| C3 | Parse 6 .dag files | AST matches v1 (JSON diff) |
| C4 | Resolve gist imports | Module graph matches v1 |
| C5 | Resolve gist types | TypeExpr structure matches v1 |
| C6 | Lower gist to DAG | Node/edge topology matches v1 |
| I1 | Pipeline compiles gist | Serialized DAG matches v1 |
| I2 | Kernel runs pipeline on gist | Same as I1 via kernel |

Every test compares v2 output to v1 output. No manual golden files —
v1 IS the oracle. When they match, v2 is correct.

---

## Risk register

| Risk | Impact | Mitigation |
|------|--------|------------|
| Parser too complex for .dag expressiveness | Blocks C3 | Prototype tricky constructs (string interpolation, precedence) early in C2 |
| Kernel missing intrinsics that compiler needs | Blocks I1 | Inventory all builtins used by C2-C6 during implementation; add to K2 |
| v1/v2 output comparison too strict (ordering, whitespace) | Blocks testing | Normalize before comparison (sort keys, strip whitespace) |
| Performance: interpreted .dag too slow for compiler | Degrades DX | Acceptable for bootstrap; optimize kernel hot paths if needed |
| Recursive types in AST (Expr contains Expr) | Parser complexity | v1 already handles this; same recursive descent pattern in .dag |
