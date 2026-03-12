# Pipeline Purity Audit: v1 Compiler Invariant Violations

Companion to `DESIGN-eval-redesign.md`. Systematic scan of every pipeline
stage for violations of the governing invariants in `src/README.md`.

The goal: each stage is a **pure pipeline function** — deterministic
inputs to outputs, no side effects, no fabrication, no string heuristics,
no duplicated logic, explicit boundary contracts.

---

## Methodology

Every stage was scanned for seven categories of violation:

| # | Category | README invariant |
|---|----------|-----------------|
| 1 | **Side effects in pure stages** | Pure core logic; documented I/O boundaries |
| 2 | **Fabrication fallbacks** | No fallbacks that fabricate |
| 3 | **String-based open-set enumeration** | No case enumeration for open sets |
| 4 | **Parallel implementations** | No parallel implementations |
| 5 | **Shared mutable state** | Clear interfaces; return values not mutated state |
| 6 | **Missing boundary contracts** | Explicit boundary contracts |
| 7 | **Cross-layer concerns** | Each stage describable in one sentence |

Items already tracked in `SUSTAINABILITY.md` are cross-referenced (Snn).

---

## 1. Side effects in pure stages

### 1.1 Lowerer reads process environment (HIGH)

`daglang-lower/src/lib.rs:1155` — `std::env::var()` during
`resolve_profile_config_expr` for `env("VAR")` config bindings.

The lowerer is supposed to be a pure `TypedProject → Dag<LoweredOp>`
function. Reading process environment couples it to the runtime host.

**Fix:** Profile config resolution should be a separate pipeline step
that runs before lowering, or the lowerer should receive resolved
config values as inputs.

### 1.2 `eprintln!` in transport library crate (MEDIUM)

`transport/src/metrics.rs:101–140` — `LogMetricsSink` uses `eprintln!`
for 5 metric recording methods.
`transport/src/executor.rs:599` — `eprintln!` for guard skip message.

`ARCHITECTURE.md` bans `eprintln!` in library crates.

**Fix:** Replace with structured logging or return metrics as data.

### 1.3 Resolver performs filesystem I/O (MEDIUM)

`resolve/src/builder.rs:220–272` — `DagbinCache` reads/writes the
filesystem during resolution via `try_load_from_cache` /
`cache.store()`.

Resolution is conceptually a pure `Dag<LoweredOp> → Dag<DynOp>`
transform. Caching belongs in the driver/orchestrator.

---

## 2. Fabrication fallbacks

### 2.1 `lower_expr().ok()` — 9 sites (HIGH, partly S34)

`daglang-lower/src/lib.rs` lines 11153, 11503, 11619, 11631, 11642,
11762, 11838, 11913, 11992.

`lower_expr` returns `Result` but callers convert with `.ok()`,
silently discarding the error. When lowering fails, the edge is simply
not wired — producing a partial DAG with no diagnostic.

Pattern:
```
let source = lower_expr(builder, ctx, arg_expr, ...).ok();
if let Some((src_node, src_port)) = source {
    builder.add_edge(...);
}
// else: silently unwired
```

Affected: service call arguments, conditional synthesis, match
dispatch, list construction, string interpolation.

**Fix:** Propagate `Result` through all `lower_expr` call sites.
Convert the return type to `Verdict` if partial success is needed.

### 2.2 `CallableOp` eval error passthrough (HIGH, S3)

`resolve/src/resolve.rs:233–259` — When `evaluate_fn_body_with_data`
fails with real inputs, the error is discarded and execution continues
with declared-output passthrough.

```
Err(eval_err) => {
    let _ = eval_err;
    execute_with_declared_output_passthrough(...)
}
```

This masks all evaluator regressions. The evaluator can never
visibly fail for fn-body ops.

**Fix:** S3 exit plan: structured eval contract — evaluator declares
capabilities, unsupported forms are compile-time opaque.

### 2.3 `ValueBacking::Json` as top-type escape (MEDIUM, S23/S35)

7 sites across `ir/src/types.rs:1191`, `ir/src/type_registry.rs:1409–1412`,
`test/src/auto_mock.rs:222,676`, `emit/src/test_gen.rs:170,206`.

Unknown type IDs fall back to `ValueBacking::Json`, which accepts any
JSON value. This converts type errors into silently accepted data.

**Fix:** `unwrap_or(false)` flip per SUSTAINABILITY.md sequence item 3.

### 2.4 Lexer fabricates zero for invalid numbers (LOW)

`daglang-syntax/src/lexer.rs:496,500` — `parse().unwrap_or(0.0)` and
`parse().unwrap_or(0)`. Invalid numeric literals silently become 0.

**Fix:** Return parse error diagnostic.

### 2.5 Typecheck uses `ValueType::Unknown` as sentinel (MEDIUM)

`daglang-typecheck/src/lib.rs` — 20+ sites return
`ValueType::Unknown` instead of propagating inference failure.
`Unknown` flows through typechecking and can reach lowering unchecked.

**Fix:** Track unresolvable types as explicit errors; fail at the
typecheck→lower boundary.

### 2.6 Emit mock defaults (MEDIUM)

`emit/src/test_mock_emit.rs:544,566,576,580,589` — Missing mock data
defaulted to `"{}"`, status `200`, exit code `0`, empty strings.
`emit/src/computation.rs:352,481,502,643` — Unknown ports become
`"Unknown"`, missing port names default to `"request"`.

**Fix:** Require all mock data explicitly; error on missing.

### 2.7 Auth input default fabrication (LOW)

`daglang-lower/src/spec.rs:66–69` — Missing `auth_input` defaults to
`"auth_token"`. May not match the actual service config.

---

## 3. String-based open-set enumeration

### 3.1 Provider hints from symbol names (MEDIUM, S45)

`daglang-lower/src/lib.rs:767–774,855–872` — Provider classification
via `match tail { "Gcp" | "GcpConfig" => ..., "Aws" => ..., ... }`.
New providers require editing hardcoded string sets.

### 3.2 Transport kind inference from name substrings (MEDIUM, S46)

`emit/src/computation.rs:600–630` — Falls back to
`name.contains("read")` / `name.contains("write")` to infer
`TransportKind`. Request kind inferred from port names containing
`"path"`, `"command"`, `"url"`, `"body"`, `"args"`.

### 3.3 Collection op kind from call name (LOW, S11)

`daglang-lower/src/lib.rs:9594–9606` — Alias map from `"count"` →
`Len`, `"sum"` → `Fold`, etc.
`emit/src/computation.rs:505–524` — Duplicated collection op name map.

### 3.4 Resolver dispatches on node name prefixes (MEDIUM)

`resolve/src/resolve.rs:1025–1027,1084,1111–1115` — Module prefix
checks: `starts_with("services.")`, `starts_with("workspace.")`,
`starts_with("extdeps.")`. Transport phase from name prefix:
`starts_with("service_transport::execute::")`,
`starts_with("service_transport::prepare::")`,
`starts_with("service_transport::parse::")`.

### 3.5 Executor checks type IDs by string (MEDIUM)

`exec/src/execute/mod.rs:1832,2022,2028,2042` —
`port.type_id.0 == "FilesystemHandle"`,
`port.type_id.0 == "TransportRequest"`,
`port.type_id.0 == "ToolHandle"`.

### 3.6 Service ops filter by type ID string (MEDIUM)

`resolve/src/service_ops/service_ops_impl.rs:189,435,466,1144` —
`.filter(|f| f.type_id == "String")`,
`f.type_id.starts_with("List<")`,
`f.type_id == "Secret"`,
`f.type_id == "Bytes"`.

### 3.7 Error classification by substring (MEDIUM)

`transport/src/classify.rs:208–223` — Auth/rate-limit/permission
errors detected via `lower.contains("auth")`,
`lower.contains("rate limit")`, etc.

### 3.8 Container type classification by string (LOW, S47)

`emit/src/type_codegen.rs:57–67` — `match name.as_str() { "List" => ..., "Set" => ..., "Map" => ... }`.
`emit/src/type_codegen.rs:227–234` — Same pattern for Rust type mapping.

### 3.9 Typecheck builtin callables (MEDIUM)

`daglang-typecheck/src/lib.rs:2163–2194` — Hardcoded intrinsic
contracts for `map`, `filter`, `join`, `count`, `replace_section`,
`detect_runtime`, `now`, `generate`, etc.

### 3.10 `"None"` / `"null"` pattern match (LOW)

`daglang-lower/src/expr.rs:347,354–355` — Null literals recognized
via string match: `name == "None" || name == "null"`.

### 3.11 Hardcoded canonical node IDs (MEDIUM)

`daglang-lower/src/lib.rs:3036–3091` — Transport triplet node IDs
hardcoded: `"prepare_github_oidc"`, `"execute_sts"`,
`"parse_impersonate"`, etc. New providers require editing this set.

### 3.12 Parser config fields by string (LOW)

`daglang-syntax/src/parser.rs` — Service config (rate limit units,
backoff strategies, transport binding kinds) all matched by string
against open sets.

### 3.13 `ast_utils` type name checks (LOW)

`daglang-syntax/src/ast_utils.rs:15–68` — `is_secret_type`,
`is_bool_type`, `is_list_type`, `is_map_string_string`,
`is_function_type` all use string comparison.

---

## 4. Parallel implementations

### 4.1 Service call collection (MEDIUM)

`daglang-lower/src/scope.rs` vs `daglang-lower/src/lib.rs:~9689` —
`ScopedBody` (scope-aware) vs `collect_service_calls_from_stmts`
(flat). Two representations of "which service calls exist."

### 4.2 Literal lowering (LOW)

`daglang-lower/src/expr.rs` `lower_literal()` vs
`daglang-lower/src/lib.rs` `expr_to_json_literal()` — Two paths from
AST to literal representations with different output types.

### 4.3 Resolver vs interp fn-body evaluation (HIGH, S3)

`resolve/src/resolve.rs:191` and `interp/src/lib.rs:174` — Both call
`evaluate_fn_body_with_data`, but the resolver adds a silent-fallback
passthrough on error while interp does not. Same computation, two
failure modes.

### 4.4 Transport classification (MEDIUM, S16)

`emit/src/computation.rs:600–630` — `infer_transport_kind` uses
`ServiceTransportClass` when available but falls back to name
heuristics, creating two classification paths.

### 4.5 Collection op name mapping duplicated (LOW)

`daglang-lower/src/lib.rs:9594–9606` and
`emit/src/computation.rs:505–524` — Same `match name { "count" => ..., "sum" => ... }`
in both lowerer and emitter.

---

## 5. Shared mutable state

### 5.1 `TmpCounter` thread-local in emit (LOW)

`emit/src/fn_codegen.rs:72–96` — Thread-local `Cell<usize>` counter
for temp names. Makes emission non-deterministic across threads.

**Fix:** Pass explicit counter through emission functions.

### 5.2 `DagBuilder` accumulator (LOW — inherent to IR building)

`daglang-lower/src/lib.rs` — Mutable builder passed through all
helpers. Standard IR builder pattern; not strictly a violation.

---

## 6. Missing boundary contracts

### 6.1 Lowerer output not validated (HIGH, S18)

Lowering produces `Dag<LoweredOp>` but there is no structural
validation that:
- All port TypeIds resolve
- No unresolved placeholders remain
- Transport triplets are complete (prepare/execute/parse)
- Callable return outputs are wired (`S34`)

### 6.2 Typecheck → lower boundary unvalidated (MEDIUM)

`TypedProject` is passed to the lowerer with no explicit post-
condition checking. `ValueType::Unknown` can flow through.

### 6.3 Resolve → execute boundary unvalidated (MEDIUM)

Resolution maps `LoweredOp` → `DynOp` but does not validate:
- All required inputs are connected
- Output shapes match downstream expectations
- Transport triplets are consistent

### 6.4 No input validation in emit (MEDIUM)

`emit_rust_bundle`, `emit_go_bundle`, `emit_c_bundle` take
`ReachableDag<LoweredOp>` and `DerivedArtifacts` but do not
validate input completeness. Malformed inputs produce incorrect
emission with no diagnostic.

### 6.5 Interp/transport boundary unspecified (LOW)

No formal schema or validation at the boundary between pure eval
and I/O transport dispatch.

---

## 7. Cross-layer concerns

### 7.1 Computation classification in emit (MEDIUM)

`emit/src/computation.rs` — The emitter classifies nodes into
`Computation` (Pure, Transport, ResourceAcquire, Collection) by
re-walking `LoweredOp`. This semantic classification should happen
in the lowerer and be attached to nodes, not recomputed.

### 7.2 Computation classification in typecheck (MEDIUM)

`daglang-typecheck/src/lib.rs:~1190–1410` — `classify_computation()`,
`classify_primitive()`, `classify_callable()`, `classify_fn_body()`,
`TransportKind`, `RequestKind`, `ResponseKind` all live in the
typecheck crate but are consumed by emit. This mixes emit concerns
into the semantics layer.

### 7.3 `EmitCollectionFamily` in syntax AST (LOW)

`daglang-syntax/src/lib.rs:608` — `EmitCollectionFamily { Map, Filter, Fold, Sort }` is an emit-level enum living in the parser
AST. Couples parsing to codegen.

### 7.4 AST → CodeIR lowering in emit (MEDIUM)

`emit/src/fn_codegen.rs` — DSL AST is lowered to `code_ir` during
emission. This is a second lowering pass that should either reuse
the main lowering pipeline or be a declared separate stage.

### 7.5 Resolver invokes compilation and caching (MEDIUM)

`resolve/src/builder.rs` — `build_dsl_graph` compiles and resolves
in one function, mixing driver-level compilation with resolution.
The resolver also manages `DagbinCache`, which is infra concern.

---

## 8. Dependency direction violations

### 8.1 `gunbc-resolve` → `daglang-driver` (HIGH)

`resolve/Cargo.toml` — Resolution (layer 08) depends on the pipeline
driver (layer 02). Inverts the expected layering: materialize stages
should sit downstream of the pipeline, not import it.

### 8.2 `daglang-lower` → `gunbc-resolve` (dev) (LOW)

`daglang-lower/Cargo.toml` — Lowering (layer 05) depends on the
resolver (layer 08) in dev-dependencies. Creates a cycle in the
conceptual layer order.

### 8.3 `gunbc-interp` → `gunbc-lib-transport` (LOW)

`interp/Cargo.toml` — Interpreter depends directly on concrete
transport implementation rather than an abstraction.

---

## Summary by severity

### High severity (pipeline invariant violations that mask bugs)

| ID | Stage | Issue |
|----|-------|-------|
| 2.1 | lower | `lower_expr().ok()` — 9 sites silently drop errors |
| 2.2 | resolve | `CallableOp` eval passthrough masks all evaluator failures (S3) |
| 1.1 | lower | `std::env::var()` I/O during lowering |
| 6.1 | lower→resolve | No structural validation of lowerer output (S18) |
| 8.1 | resolve | Dependency on driver inverts layer order |

### Medium severity (degrade maintainability, hide errors)

| ID | Stage | Issue |
|----|-------|-------|
| 2.3 | ir/emit/test | `ValueBacking::Json` top-type escape (S23/S35) |
| 2.5 | typecheck | `ValueType::Unknown` sentinel flows through unchecked |
| 2.6 | emit | Mock defaults fabricate valid-looking but wrong output |
| 3.1–3.12 | all | 12+ string-based open-set enumerations across stages |
| 4.3 | resolve/interp | Same fn eval, different failure modes (S3) |
| 4.4 | emit | Transport classification has parallel paths (S16) |
| 7.1–7.4 | typecheck/emit | Classification logic in wrong stages |
| 6.2–6.4 | boundaries | No input/output validation at 3+ stage boundaries |

### Low severity (style/minor coupling)

| ID | Stage | Issue |
|----|-------|-------|
| 2.4 | syntax | Lexer fabricates 0 for invalid numbers |
| 3.8–3.13 | syntax/emit | Container/type name string matching |
| 4.2, 4.5 | lower/emit | Duplicated literal/collection-op name logic |
| 5.1 | emit | Thread-local counter |
| 7.3 | syntax | `EmitCollectionFamily` in parser AST |

---

## Relationship to SUSTAINABILITY.md

Most high-severity findings map directly to tracked sustainability
items:

- 2.1 → new (not tracked)
- 2.2 → S3
- 1.1 → new (not tracked)
- 2.3 → S23/S35
- 6.1 → S18
- 8.1 → new (not tracked)
- 3.1 → S45
- 3.2 → S46
- 3.3 → S11
- 4.3 → S3

**Newly discovered issues not in SUSTAINABILITY.md:**

1. `lower_expr().ok()` — 9 silent error drops in the lowerer
2. `std::env::var()` — I/O in a pure stage
3. `gunbc-resolve` → `daglang-driver` dependency inversion
4. `ValueType::Unknown` sentinel propagation (20+ sites)
5. `EmitCollectionFamily` in parser AST (cross-layer coupling)
6. Computation classification duplicated across typecheck and emit
7. No input validation at emit stage boundary
8. Thread-local counter in emit (`TmpCounter`)

---

## Governing principle: make illegal states unrepresentable

The project invariant is: **don't validate; refactor the upstream type
so the invalid state is impossible to construct.** A validation pass
at a boundary is a symptom that the boundary type is too permissive.

Every recommendation below is an upstream structural change, not a
downstream check.

---

## Recommended structural changes

Ordered by the number of downstream violations each change eliminates.

### A. Embed resolved type structure in ports (eliminates Branch 1)

**Current:** `Port { type_id: TypeId }` where `TypeId(String)`.
Downstream must look up the string in a `TypeRegistry` to learn
anything about the type. The lookup can fail → `ValueBacking::Json`
fallback → any value accepted → wrong emit → no test coverage. This
is the "deep root" from SUSTAINABILITY.md.

**Structural fix:** `Port { typ: ResolvedType }` where `ResolvedType`
is an algebraic type that embeds structure directly:

```rust
enum ResolvedType {
    Scalar(ScalarType),
    List(Box<ResolvedType>),
    Set(Box<ResolvedType>),
    Map { key: Box<ResolvedType>, value: Box<ResolvedType> },
    Optional(Box<ResolvedType>),
    Record { fields: Vec<(String, ResolvedType)> },
    Sum { variants: Vec<(String, ResolvedType)> },
}
```

There is no `Unknown` variant. There is no `String` handle. The
typechecker must fully resolve before constructing the output type.
If it can't resolve, it returns an error — not `Unknown`.

**Eliminates:** S18 (TypeId validation), S23/S35 (`ValueBacking::Json`
fallback), S7 (identity placeholder), 3.5 (executor type_id checks),
3.6 (service ops type_id checks), S30 (testgen re-parsing TypeId
strings), 3.8 (container type string matching in emit), S1 (registry
duplication), S2 (mock element enumeration), S4 (cardinality cache),
S13 (carrier classification), 2.4 (lexer number fabrication — types
would enforce validity).

**Why this is upstream, not validation:** A validation walk after
lowering ("assert all TypeIds resolve") would catch the symptom but
leave the root cause: the `TypeId(String)` type *can* represent
unresolved references. Embedding structure makes "unresolved" not a
state the type can express.

---

### B. Classify expressions before lowering (eliminates S64)

**Current:** `lower_expr` takes an AST `Expr` and tries to turn it
into a DAG node. It can fail because not all expressions can become
DAG nodes. Callers use `.ok()` to silently discard the error (9 sites).

**Structural fix:** The typechecker (or a pre-lowering pass)
classifies each expression into one of two types:

```rust
enum DagExpr { ... }     // service calls, data references, literals
                         // — will become DAG nodes
enum FnBodyExpr { ... }  // arbitrary computations
                         // — will become LoweredExpr for the evaluator
```

Then:
- `lower_dag_expr(DagExpr) → (NodeId, PortName)` is **total**. No
  `Result`, no `.ok()`. If the typechecker classified it as
  `DagExpr`, it will succeed.
- `lower_fn_expr(FnBodyExpr) → LoweredExpr` is **total**. The
  evaluator can handle all `FnBodyExpr` forms.

**Eliminates:** S64 (all 9 `.ok()` sites), the `.ok()?` double
conversion pattern, and partial DAGs from silently unwired edges.

**Why this is upstream, not validation:** Propagating `Result` (the
validation approach) would surface the error instead of hiding it,
but wouldn't prevent the error. Classification makes the error
impossible: a `DagExpr` can always become a node.

---

### C. Resolve config before lowering (eliminates S65)

**Current:** `resolve_profile_config_expr` calls `std::env::var()`
during lowering.

**Structural fix:** The pipeline becomes:

```
TypedProject
    → resolve_config(env_provider) → ResolvedProject
    → lower(ResolvedProject) → Dag
```

`ResolvedProject`'s config type is:

```rust
enum ResolvedConfigValue {
    Literal(String),
    SecretRef(String),
}
// No Env(String) variant — it's been resolved.
```

The lowerer's match has no `env()` arm. The `std::env::var` call
is structurally absent from the lowerer.

**Eliminates:** S65, and makes the lowerer a pure function.

---

### D. Split `LoweredOp::Callable` by computation kind (eliminates S68, S46)

**Current:** `LoweredOp::Callable` is a kitchen-sink variant with
optional `service_metadata`. The emitter re-derives computation kind
(`Pure`/`Transport`/`ResourceAcquire`) by inspecting metadata fields.
Transport kind is inferred from name substrings when metadata is
missing.

**Structural fix:** Split `Callable` into distinct enum variants:

```rust
enum LoweredOp {
    FnBody { module: String, name: String, fn_body: LoweredFnBody },
    Transport {
        module: String,
        name: String,
        class: ServiceTransportClass,  // not optional, not inferred
        phase: TransportPhase,         // Prepare | Execute | Parse
        obligation: ObligationCategory,
    },
    ResourceAcquire { ... },
    Collection { ... },
    Primitive { ... },
    Pipeline { ... },
    Pattern(PatternOp),
}
```

The emitter matches on variants — `Transport { class, phase, .. }` —
not on optional metadata and name heuristics.

**Eliminates:** S68 (classification duplication in emit and typecheck),
S46 (transport kind from name substrings), 3.2
(`infer_transport_kind` fallback), 4.4 (parallel classification),
7.1/7.2 (cross-layer classification), 3.4 (resolver name prefix
dispatch — the resolver matches on the variant instead).

**Why this is upstream, not validation:** Validating "all Callable
nodes have transport metadata" after lowering would surface the
gap but leave the door open. A distinct `Transport` variant with
required fields makes "transport node without transport class"
unrepresentable.

---

### E. ANF lowering contract (eliminates S3, part of eval redesign)

**Current:** `LoweredExpr` can contain nested `Call` nodes. The
evaluator must handle calls inside expressions, which it can't
always do. The `CallableOp` catches evaluator failures and
passthroughs.

**Structural fix:** (from `DESIGN-eval-redesign.md`) Hoist calls
to statement level during lowering. After lowering, `LoweredExpr`
structurally cannot contain a `Call`:

```rust
enum LoweredExpr {
    Literal(LoweredLiteral),
    Var(String),
    BinOp { ... },
    FieldAccess { ... },
    Record { ... },
    // No Call variant. Calls are LoweredStmt::Let(name, Call{...}).
}
```

`eval_expr` is a total function over call-free trees. It cannot fail
for "unsupported expression forms" because the type prevents them.
The `CallableOp` catch-all becomes dead code.

**Eliminates:** S3 (evaluator passthrough), 4.3 (resolver vs interp
dual failure modes), the `execute_with_declared_output_passthrough`
function.

---

### F. Provider/transport metadata as DSL annotations (eliminates S45, 3.1, 3.11)

**Current:** Provider and transport kind are inferred from symbol
name substrings (`"Gcp"`, `"read"`, etc.) and hardcoded canonical
node ID sets.

**Structural fix:** DSL annotations at the definition site:

```dag
@transport shell
@provider gcp
service gcp.STS {
  ...
}
```

The parser produces `ServiceDef { transport: ServiceTransportClass, provider: ProviderHint }`. The typechecker propagates them. The lowerer stamps them on nodes. No inference.

The node ID sets in `lib.rs:3036–3091` become unnecessary because
provider and transport are fields, not properties derived from the ID.

**Eliminates:** S45 (provider classification), 3.1 (provider hints
from symbol names), 3.11 (hardcoded canonical node IDs), S25
(virtual backend defaulting to REST).

---

### G. Remove `EmitCollectionFamily` from syntax (eliminates 7.3)

**Current:** `EmitCollectionFamily { Map, Filter, Fold, Sort }` lives
in `daglang-syntax/src/lib.rs:608` — a codegen concept in the parser.

**Structural fix:** Delete from syntax. The lowerer determines
collection operation kind and stamps it on `LoweredOp::Collection { kind: CollectionOpKind }` (which already exists). The emitter reads
`CollectionOpKind`, not a syntax-level enum.

---

### H. Invert resolve→driver dependency (eliminates S66)

**Current:** `gunbc-resolve` depends on `daglang-driver` (layer 08 →
layer 02).

**Structural fix:** The resolver's input type is
`Dag<LoweredOp>` — the driver compiles and passes the artifact
down. The resolver never invokes compilation. The
`daglang-driver` dependency disappears because the resolver has no
path to it.

---

## How each violation maps to a structural change

| Violation | Fix |
|-----------|-----|
| S64: `lower_expr().ok()` | **B** (expression classification) |
| S3: CallableOp passthrough | **E** (ANF contract) |
| S65: `env()` I/O in lowerer | **C** (resolve config first) |
| S18: unresolved TypeIds | **A** (embedded type structure) |
| S66: resolve→driver dep | **H** (invert dependency) |
| S23/S35: `ValueBacking::Json` | **A** (embedded type structure) |
| S67: `ValueType::Unknown` | **A** (output type has no Unknown) |
| S45/S46: provider/transport heuristics | **D** + **F** (split variant + DSL annotations) |
| S68: classification duplication | **D** (split LoweredOp variant) |
| S69: `EmitCollectionFamily` in syntax | **G** (delete from syntax) |
| S70: no emit input validation | **A** + **D** (richer input types) |
| S71: thread-local TmpCounter | standalone (explicit counter) |
| 3.4: resolver name prefix dispatch | **D** (match on variant) |
| 3.5: executor type_id strings | **A** (embedded type structure) |
| 3.6: service ops type_id strings | **A** (embedded type structure) |
| 3.7: error classification by substring | standalone (structured error types) |
| 7.1–7.4: cross-layer classification | **D** (lowerer stamps kind) |
| 4.3: dual fn-body eval paths | **E** (ANF makes eval total) |

---

## Cascade order

Changes depend on each other. This is the order that avoids rework:

```
F (DSL annotations)
  → D (split LoweredOp — uses annotated metadata)
    → B (expression classification — depends on LoweredOp shape)

C (resolve config — independent)

A (embedded type structure — largest, can proceed in parallel)

E (ANF contract — from eval redesign, parallel to above)

G, H — small/mechanical, any time
```

A and E are already described in `DESIGN-eval-redesign.md` and
`SUSTAINABILITY.md` respectively. D and F are new. B and C are new.
