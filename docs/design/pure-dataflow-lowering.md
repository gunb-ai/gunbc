# Pure Dataflow Lowering (C24) & Service-Driven Codegen (C25)

Status: Design
Owner: `gunbc` daglang/codegen
Scope: Eliminate `ExprComputeOp` runtime interpreter and hand-written service `DynOp` adapters

## 0. Document Contract

1. This document is the normative design for C24 and C25 from Lane 1.
2. C24 replaces the embedded AST interpreter (`ExprComputeOp`) with structural DAG nodes.
3. C25 replaces ~40 hand-written `DynOp` service adapters with generic protocol interpreters.
4. C25 depends on C24. Both are XL tasks.
5. `docs/design/service-codegen.md` remains the normative source for C25 protocol interface
   modeling; this document covers the decomposition strategy and migration path.

## 1. Problem Statement

### 1.1 The ExprComputeOp Problem (C24)

The lowerer converts complex return expressions into `ExprCompute` nodes that embed a
complete `LoweredFnBody` — an AST subtree evaluated at runtime by `evaluate_fn_body`
(a recursive interpreter in `daglang-lower/src/eval.rs`).

This creates three architectural problems:

1. **Hidden interpreter in the executor.** `ExprComputeOp::execute` is a mini-language
   runtime: it flattens Map inputs into `parent__field` identifiers, pre-seeds unbound
   variables with `Value::Skipped`, and delegates to a recursive evaluator that handles
   BinOp, Match, If/Else, Pipe, Lambda, StringInterp, For, and more. The DAG graph
   doesn't see these operations — they're opaque.

2. **The `__` convention.** The lowerer rewrites `entry.kind` → `Ident("entry__kind")` via
   `remap_expr_idents`. At runtime, `ExprComputeOp::execute` flattens Map values into
   `parent__field` environment entries. This convention is implicit, fragile, and invisible
   to graph analysis tools.

3. **Blocked optimizations.** Because ExprCompute nodes are opaque, the compiler cannot:
   - Deduplicate identical field extractions across nodes
   - Parallelize independent sub-expressions
   - Generate target-specific code (Go, C, MIPS) for expression evaluation
   - Apply DRE (Deductive Redundancy Elimination) within expressions

### 1.2 The Hand-Written DynOp Problem (C25)

`gunbc-dag/src/resolve.rs` contains ~40 hand-written `Executable` adapter structs for
service operations. Each REST prepare adapter extracts inputs, interpolates URLs, builds
JSON bodies, and returns `TransportRequest::Rest`. Each parse adapter checks status,
extracts JSON fields, and returns outputs.

These adapters are written *per service operation* when they should be written *per
transport class*. See `docs/design/service-codegen.md` for the full protocol interface
design.

C25 depends on C24 because the generic protocol interpreters need structural input wiring
(GetField, StringInterpolate) rather than opaque ExprCompute blobs.

### 1.3 Current State

| Metric | Value |
|--------|-------|
| ExprCompute nodes across all tools | ~104 unique (175 including pipeline imports) |
| `__`-convention identifiers in fn bodies | Hundreds (every `param.field` inside ExprCompute) |
| `remap_expr_idents` call sites | 1 (lowerer) |
| `evaluate_fn_body` call site | 1 (ExprComputeOp::execute) |
| Hand-written DynOp service adapters | ~40 structs in resolve.rs |
| GetField nodes (C24 step 1, done) | 3 (simple `param.field` return expressions) |

### 1.4 Goal

After C24: **Zero `ExprComputeOp` in any compiled graph.** The `__` convention,
`remap_expr_idents`, `referenced_vars`, and `evaluate_fn_body` are deleted.

After C25: **Zero hand-written `DynOp` for services.** The resolver has 3 generic
executables (REST, Shell, File), not N per-service executables.

## 2. ExprCompute Pattern Census

ExprCompute fn bodies fall into these categories, from simplest to most complex:

### Category A: Pure Field Projection (0 remaining — handled by C24 step 1)

```
Return([("result", Ident("param__field"))])
```

Simple `param.field` return expressions. Already replaced by `GetField` nodes.

### Category B: Pipe Chain on Field (most common, ~40%)

```
Return([("result", Pipe {
  receiver: Ident("partitioned__readable"),
  call: Call { name: "map", args: [(None, Lambda { params: ["e"], body: Ident("e__path") })] }
})])
```

Field extraction (`partitioned.readable`) piped through a collection operation (`map`,
`filter`, `join`, `fold`, `count`, `all`, `any`, `first`). The lambda bodies typically
contain further `__`-convention field access.

### Category C: Match Expression (~15%)

```
Return([("result", Match {
  expr: Ident("entry__kind"),
  arms: [
    LoweredMatchArm { pattern: Ident("Directory"), body: Literal(String("directory")) },
    LoweredMatchArm { pattern: Ident("Missing"), body: Literal(String("missing")) },
    ...
  ]
})])
```

Pattern matching on sum types or strings. Scrutinee is typically a field access.

### Category D: String Interpolation (~10%)

```
Return([("result", StringInterp([
  Literal("\t@echo \"  "),
  Expr(Ident("w__name")),
  Literal("  - "),
  Expr(BinOp { left: Ident("w__comment"), op: NullCoalesce, right: Ident("w__description") }),
  Literal("\"")
])})])
```

Template strings with embedded field access and operators.

### Category E: BinOp / Conditional (~10%)

```
Return([("result", BinOp {
  left: Ident("github_result__written"),
  op: Or,
  right: Ident("gitlab_result__written")
})])
```

Binary operations (arithmetic, comparison, logical) on field-accessed values.

### Category F: Compound with Let Bindings (~25%)

```
Let("joined", Pipe { receiver: Ident("prereqs"), call: Call { name: "join", ... } }),
Return([("result", IfElse {
  cond: BinOp { left: Ident("joined"), op: Ne, right: Literal(String("")) },
  then_: StringInterp([Expr(Ident("joined")), Literal(" first")]),
  else_: Some(Literal(String("auto-fix")))
})])
```

The most complex category. Let bindings create local variables used in subsequent
expressions. Some Let bindings reference service call results (not just pure computation).

## 3. Decomposition Strategy

### 3.1 New Primitive Op Kinds

Add these variants to `PrimitiveOpKind`:

```rust
enum PrimitiveOpKind {
    // Existing
    GetField { field: String },           // C24 step 1 (done)
    ExprCompute { fn_body: Box<LoweredFnBody> },  // Legacy (to be eliminated)

    // C24 new structural ops
    StringInterpolate {
        /// Template parts: alternating literal strings and input port names.
        /// Example: ["hello ", "{name}", "!"] where {name} references input port "name".
        parts: Vec<StringTemplatePart>,
    },
    BinaryOp {
        op: LoweredBinOp,
    },
    UnaryOp {
        op: LoweredUnaryOp,
    },
    Conditional,                          // if/else: inputs are (cond, then, else) → result
    MatchDispatch {
        /// Arms: each arm has a pattern and a literal or port-referenced result.
        arms: Vec<MatchArm>,
    },
    RecordConstruct {
        /// Field names; values come from input ports with matching names.
        fields: Vec<String>,
    },
    NullCoalesce,                         // inputs: (value, fallback) → result
    VariantConstruct {
        tag: String,
        fields: Vec<String>,
    },
}

enum StringTemplatePart {
    Literal(String),
    Port(String),  // references an input port by name
}
```

### 3.2 Decomposition Rules

Each compound expression becomes a **subgraph** of structural nodes wired together:

#### Rule 1: Field Access → GetField node

```
entry.kind  →  [GetField { field: "kind" }]
                   ↑ input: entry (from param source or upstream node)
                   ↓ output: result
```

#### Rule 2: BinOp → BinaryOp node

```
a + b  →  [BinaryOp { op: Add }]
               ↑ inputs: left (from a), right (from b)
               ↓ output: result
```

#### Rule 3: String Interpolation → StringInterpolate node

```
"hello {name}!"  →  [StringInterpolate { parts: [Lit("hello "), Port("name"), Lit("!")] }]
                         ↑ input: name (from upstream)
                         ↓ output: result
```

#### Rule 4: If/Else → Conditional node

```
if cond { a } else { b }  →  [Conditional]
                                  ↑ inputs: cond, then_value, else_value
                                  ↓ output: result
```

Note: both branches are eagerly evaluated (they're values, not effects).
For effectful branches, the existing `BranchBuilder` pattern remains.

#### Rule 5: Match → MatchDispatch node

```
match x { A => "a", B => "b", _ => "c" }  →  [MatchDispatch { arms }]
                                                   ↑ input: scrutinee
                                                   ↓ output: result
```

For match arms with complex bodies, the body becomes a separate subgraph
wired to the MatchDispatch output.

#### Rule 6: Pipe Chain → Collection node (already exists for `map`, `filter`, `fold`)

The lowerer already has `LoweredOp::Collection` for top-level pipe chains.
The gap is that pipe chains *inside return expressions* become ExprCompute
instead. The fix: detect pipe chains in return expressions and emit them
as Collection nodes.

#### Rule 7: Let Bindings → Intermediate nodes

Each `let x = expr` in an ExprCompute body becomes a separate structural node
whose output feeds into downstream nodes that reference `x`.

```
let joined = prereqs |> join(" + ")
return { result: if joined != "" { "{joined} first" } else { "auto-fix" } }

→  [CollectionJoin] → [BinaryOp { Ne }] → [Conditional]
       ↑ prereqs          ↑ right: ""         ↑ then: [StringInterpolate]
                                               ↑ else: [LiteralSource "auto-fix"]
```

### 3.3 Resolver Changes

Each new `PrimitiveOpKind` variant gets a corresponding `Executable` impl:

```rust
struct StringInterpolateOp { parts: Vec<StringTemplatePart> }
struct BinaryOpOp { op: LoweredBinOp }
struct UnaryOpOp { op: LoweredUnaryOp }
struct ConditionalOp;
struct MatchDispatchOp { arms: Vec<...> }
struct RecordConstructOp { fields: Vec<String> }
struct NullCoalesceOp;
```

These are all pure, stateless, and trivial — typically 10-20 lines each.
No Map flattening, no `__` convention, no pre-seeding `Value::Skipped`.

### 3.4 What Gets Deleted

After C24 completion:

| File | What's deleted |
|------|---------------|
| `daglang-lower/src/lib.rs` | `remap_expr_idents()`, `synthesize_expr_compute()`, `collect_expr_leaf_refs()`, `ExprLeafRef` struct |
| `daglang-lower/src/lib.rs` | `PrimitiveOpKind::ExprCompute` variant |
| `daglang-lower/src/eval.rs` | `evaluate_fn_body()` and all supporting functions (~400 lines) |
| `gunbc-dag/src/resolve.rs` | `ExprComputeOp` struct, `collect_fn_body_idents()`, `collect_lowered_expr_idents()`, Map flattening code |
| `daglang-lower/src/expr.rs` | `LoweredFnBody`, `LoweredStmt`, `LoweredExpr` (entire file — ~300 lines) |

Estimated net deletion: **~800 lines** of interpreter code replaced by
**~200 lines** of structural node implementations.

## 4. Migration Path

### Phase 1: Structural Primitives (M-sized)

Add the 7 new `PrimitiveOpKind` variants and their `Executable` impls.
Wire them in emit backends (`computation.rs`, `rust_exec_runtime.rs`).
No ExprCompute nodes eliminated yet — just the infrastructure.

**Verification:** `cargo build` passes. New ops have unit tests.

### Phase 2: Lowerer Expression Decomposition (L-sized)

Replace `synthesize_expr_compute` with `decompose_return_expr` that walks
the expression tree and emits structural nodes instead of a single
ExprCompute blob.

The key function signature:

```rust
fn decompose_return_expr(
    builder: &mut DagBuilder,
    ctx: &LoweringContext<'_>,
    expr: &Expr,
    output_port: &Port,
    output_name: &str,
    disambiguator: &str,
) -> Option<(String, String)>
```

This recurses into the expression, creating nodes bottom-up:

1. Leaf expressions (Ident, Literal) → param source / literal source nodes
2. Field access → GetField node
3. BinOp → BinaryOp node with edges from left/right subtrees
4. StringInterp → StringInterpolate node with edges from each expr part
5. If/Else → Conditional node with edges from cond/then/else subtrees
6. Match → MatchDispatch node with edges from scrutinee
7. Pipe → Collection node (or CallableOp for non-collection methods)
8. Let bindings → intermediate nodes feeding downstream consumers

**Migration strategy: incremental.** Start with the simplest categories
(B, C, D, E) and leave Category F (compound with Let) for last. Each
category can be migrated independently with snapshot test updates.

**Verification per category:**
- ExprCompute count decreases by expected amount
- `cargo test -p gunbc-dag --test gist_recent_regressions` passes
- Workflow obligation fixture counts update (new structural nodes appear)
- `cargo run -p daglang-cli -- expand dsl/tools/makegen.dag` shows
  structural nodes instead of ExprCompute blobs

### Phase 3: Delete Legacy (S-sized)

Once ExprCompute count reaches zero:

1. Delete `PrimitiveOpKind::ExprCompute`
2. Delete `remap_expr_idents`, `ExprComputeOp`, `evaluate_fn_body`
3. Delete `LoweredFnBody`, `LoweredStmt`, `LoweredExpr` types (or repurpose
   for emit if still needed)
4. Delete `referenced_vars` and Map flattening code

**Verification:** `cargo test --workspace` passes. Zero grep hits for
`ExprCompute`, `remap_expr_idents`, `__` convention in non-test code.

### Phase 4: Service-Driven Codegen (C25, L-sized after C24)

With structural input wiring available, implement the generic protocol
interpreters per `docs/design/service-codegen.md`:

1. `RestPrepareOp` + `RestParseOp` parameterized by `RestOperationSpec`
2. `ShellPrepareOp` + `ShellParseOp` parameterized by `ShellOperationSpec`
3. Delete per-service adapter structs from `resolve.rs`

**Verification:** Same transport triplet tests pass. Service count in
resolve.rs drops from ~40 structs to 4 generic structs.

## 5. Risks and Mitigations

### Risk: Evaluation semantics divergence

The structural nodes must exactly replicate `evaluate_fn_body` behavior for
truthiness, null coalescing, variant matching, and `Value::Skipped` propagation.

**Mitigation:** Each structural op's `execute` method is tested against the
equivalent `evaluate_fn_body` call on identical inputs. The existing gist e2e
DryRun test catches semantic regressions.

### Risk: Expression complexity explosion

Some expressions are deeply nested (match inside if inside pipe inside let).
The decomposed subgraph could have many nodes.

**Mitigation:** Node count increase is acceptable — the DAG is an IR, not
user-facing. More nodes = more optimization opportunities. The obligation
count fixtures already track pure node counts and will catch unexpected growth.

### Risk: Lambda evaluation in structural nodes

Pipe methods like `map(e => e.path)` require lambda evaluation. The current
Collection ops already handle this via `LoweredFnBody` in their op definition.

**Mitigation:** Collection ops keep their existing lambda evaluation mechanism.
C24 only decomposes the *outer* ExprCompute wrapper, not the *inner* collection
operation. `LoweredFnBody` may survive inside Collection ops even after
ExprCompute is deleted.

### Risk: Let bindings with service call dependencies

Some Let bindings reference service calls (`let run = shell.Codegen.Run()`).
These aren't pure computation — they're DAG dependencies.

**Mitigation:** The lowerer already resolves service calls into separate DAG
nodes. Let bindings that reference service calls are resolved via
`ctx.bound_service_sources` and wired as edges. Only pure Let bindings
(local computation) need decomposition into structural nodes.

## 6. Acceptance Criteria

### C24 Complete

- [ ] Zero `PrimitiveOpKind::ExprCompute` variants in `daglang-lower`
- [ ] Zero `ExprComputeOp` structs in `resolve.rs`
- [ ] `remap_expr_idents` deleted
- [ ] `evaluate_fn_body` deleted (or moved to test-only)
- [ ] No `__` convention in lowered output
- [ ] All existing tests pass (with updated snapshots/fixtures)
- [ ] `make gist` end-to-end still works

### C25 Complete

- [ ] Zero per-service `Executable` adapter structs in `resolve.rs`
- [ ] 3 generic protocol executables (REST, Shell, File)
- [ ] `ServiceOperationSpec` fully populated from DSL service definitions
- [ ] Adding a new service requires only a `.dag` file
- [ ] All existing transport triplet tests pass
