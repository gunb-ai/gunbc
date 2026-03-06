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

`gunbc-app/src/resolve.rs` contains ~40 hand-written `Executable` adapter structs for
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
| ExprCompute nodes (unique / total appearances) | 46 unique / 175 total (shared stdlib nodes appear in many files) |
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

**46 unique nodes** across 9 categories, ordered by complexity. 175 total appearances
(shared stdlib nodes appear in many files via imports). 35 of 46 nodes appear in 2+
files. The 17 shared stdlib/utility nodes account for ~143 of the 175 appearances.

### Cat 1: Pure Record Construction — 10 nodes (21.7%)

Record literals mixing `Ident(...)` field references and `Literal(...)` constants.
No Let bindings, no control flow. Used as constructor functions for structured data.
All 10 are from `shared.dag_util` (stage/document model builders).

```
Return([("result", Record { fields: [("name", Ident("name")), ("success", Ident("success")),
    ("stdout", Literal(String(""))), ("stderr", Ident("stderr")), ("skipped", Literal(Bool(false)))] })])
```

Examples: `stage_result::return`, `blank_line::return`, `doc::return`, `section::return`.

### Cat 2: Simple Pipe Chain — 6 nodes (13.0%)

Single `Pipe { receiver, call }` expression, no Let bindings. Typically
`list |> map(fn)`, `list |> count()`, or multi-step `list |> map(f) |> join(sep)`.

```
Return([("result", Pipe {
  receiver: Ident("partitioned__readable"),
  call: Call { name: "map", args: [(None, Lambda { params: ["e"], body: Ident("e__path") })] }
})])
```

Examples: `classify_files::readable`, `classify_files::skipped`, `bootstrap::crate_count`,
`collect_core_phony_str::return`.

### Cat 3: Pipe Chain + BinOp Tail — 5 nodes (10.9%)

Pipe expression whose result feeds into a comparison or logical op.
Common pattern: `list |> count() > 0` or `list |> all(predicate)`.

```
Return([("result", Pipe {
  receiver: Ident("scan__dirs"),
  call: BinOp { left: Call { name: "count", args: [] }, op: Gt, right: Literal(Int(0)) }
})])
```

Examples: `bootstrap::success`, `all_succeeded::return`, `has_label::return`.

### Cat 4: Simple BinOp — 6 nodes (13.0%)

Direct BinOp expression, no Let bindings. Equality checks, logical OR/AND,
string concatenation with Call sub-expressions.

```
Return([("result", BinOp { left: Ident("github_result__written"), op: Or,
    right: Ident("gitlab_result__written") })])
```

Examples: `is_terminal::return` (`status == "TerminalFailed"`), `cigen::written`,
`in_block::return` (`cp >= block__start && cp <= block__end_inclusive`).

### Cat 5: Simple Match (flat) — 4 nodes (8.7%)

Match on a sum type or value returning flat literals. No nesting, no Let bindings.

```
match stage { Idea => 0, Design => 1, DesignReview => 2, ..., TerminalFailed => 8 }
```

Examples: `stage_ordinal::return`, `stage_to_label::return`, `display_width_columns::return`,
`apply_prefix::return`.

### Cat 6: Nested Match — 3 nodes (6.5%)

Match expressions with Match sub-expressions in arm bodies. Most structurally complex
pure-expression pattern. The single most complex node in the codebase is
`validate_transition_with_budget::return` — **3-level nested Match** returning typed
sum-type Records with StringInterp error messages.

```
match entry__kind {
    Directory => "directory",
    Symlink => match entry__symlink_target {
        TargetDir => "symlink to directory", Broken => "broken symlink", ...
    },
    RegularFile => "binary file"
}
```

### Cat 7: Simple IfElse — 1 node (2.2%)

Direct IfElse with nested IfElse, no Let bindings. Only one node:
`render_document_line::return` — `if line__is_blank { "" } else { if line__is_comment { ... } else { line__text } }`.

### Cat 8: Compound Let + Local Computation — 8 nodes (17.4%)

Let bindings that bind to local expressions (PipeChain, BinOp, IfElse, Match, StringInterp).
Multi-step computations with intermediate variables. All pure — no service calls.

```
Let("joined", Pipe { receiver: Ident("prereqs"), call: Call { name: "join", ... } }),
Return([("result", IfElse {
  cond: BinOp { left: Ident("joined"), op: Ne, right: Literal(String("")) },
  then_: StringInterp([Expr(Ident("joined")), Literal(" first")]),
  else_: Some(Literal(String("auto-fix")))
})])
```

Examples: `aggregate_results::return` (count passed/failed stages),
`render_document::return` (two-phase rendering), `resolve_resource_target::return`
(filter + first + match), `tool_phony_names::return` (multi-part string assembly).

### Cat 9: Compound Let + Service Calls — 3 nodes (6.5%)

Let bindings that invoke service operations. The most effectful ExprCompute nodes.
Only **3 nodes** in this category:

1. `build_all::overall_success` — `let test = cargo.Build.Test()`, `let clippy = ...`
2. `codegen::success` — `let run = shell.Codegen.Run()`, returns `check__needed || run__success`
3. `codegen::ran` — `let run = shell.Codegen.Run()`, returns `!check__needed`

### Summary: Migration Ordering by Category

| Priority | Categories | Nodes | Approach |
|----------|-----------|-------|----------|
| P1 (easy wins) | Cat 4, 5 | 10 | BinaryOp + MatchDispatch nodes |
| P2 (medium) | Cat 1, 2, 7 | 17 | RecordConstruct + Collection + Conditional nodes |
| P3 (harder) | Cat 3, 6 | 8 | Compound subgraph decomposition |
| P4 (hardest) | Cat 8, 9 | 11 | Let binding → intermediate node chains |
| **Total** | | **46** | |

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
| `gunbc-app/src/resolve.rs` | `ExprComputeOp` struct, `collect_fn_body_idents()`, `collect_lowered_expr_idents()`, Map flattening code |
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

**Migration strategy: incremental by priority tier (see §2 summary table).**

- **P1** (10 nodes, Cat 4+5): BinaryOp + MatchDispatch. Simplest — no pipe chains
  or Let bindings. Pure expression → node mapping.
- **P2** (17 nodes, Cat 1+2+7): RecordConstruct + Collection + Conditional. Record
  constructors are mechanical. Pipe chains use existing Collection infra.
- **P3** (8 nodes, Cat 3+6): Compound subgraphs for pipe+BinOp and nested Match.
  Requires multi-node emission per expression.
- **P4** (11 nodes, Cat 8+9): Let binding chains. Each Let becomes an intermediate
  node. Cat 9 (3 nodes with service calls) is the hardest — service call results
  must be wired as DAG dependencies, not local variables.

**Verification per category:**
- ExprCompute count decreases by expected amount
- `cargo test -p gunbc-app --test gist_recent_regressions` passes
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
