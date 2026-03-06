# Lane 4: Compiler Pipeline Improvements

> Status lives in [`../tasks.md`](../tasks.md). This document is the detailed
> acceptance/target-contract reference for compiler-pipeline work; some items
> below intentionally describe the end state even when the current branch is
> still at a bridge or phase-1 step.

> Reference: [`docs/design/compilation-pipeline.md`](../docs/design/compilation-pipeline.md) — full pipeline map, data shapes, gap analysis, architectural principles.
>
> **Thesis**: The entire purpose of gunbc is moving contradiction discovery from
> runtime to static analysis. The compiler pipeline IS the product. Every silent
> drop, every auto-fill, every swallowed error, every unclear interface moves a
> contradiction back to runtime — exactly what the compiler exists to prevent.
>
> The interpreted path (Dag<DynOp> executor) is the only real runtime today.
> The compiled path (emit → rustc → binary) exists but is underused and incomplete.
> We should support both, but maintaining parity should be automatic, not manual.
> This lane hardens the shared pipeline and closes the gaps that make the emit path
> a second-class citizen.
>
> **Four invariants** (see design doc "Four Invariants" section):
> 1. **Binary logic** — every signal is PASS or FAIL. No ternary states.
>    No `lossy`, no `Value::Skipped`, no `allow_unresolved`, no `skip_verification`.
> 2. **Minimalism** — no redundancy between stages. Each stage adds new information,
>    never restates what an earlier stage computed.
> 3. **Resolve early** — if information is available at Stage N, resolve it there.
>    Every deferral creates ambiguity that later stages handle with fallbacks.
> 4. **Clear contracts** — every stage is a pure function with typed I/O. The
>    signature IS the contract. No hidden state, no growing parameter lists, no
>    ambient globals. Strong boundaries between participants.
>
> The strategy is to **delete all the forgiveness**.
>
> **Execution strategy**: The next moves should be **deletions, not additions**.
> Rip out the fallbacks (`lossy`, `allow_unresolved`, `lower_warn`, `skip_verification`,
> `Value::Skipped`, auto-fill, `unwrap_or_else` defaults). Tests will break heavily.
> Fix them by making `.dag` files strictly correct and wiring explicit defaults in
> the lowerer. Do not reintroduce forgiveness to make tests pass. Each fixed test
> is a contradiction that now surfaces at compile time instead of runtime.
>
> **Wisdom from the audit**:
> - Silent failures are the worst kind of bug. The pipeline has ~5 places where data
>   is silently dropped (imports, pattern nodes, service args, brace blocks, fn body errors).
>   Each one eventually surfaces as an opaque runtime failure far from the root cause.
> - The lowerer is the bottleneck. It does the most complex transform (typed AST → graph)
>   and has the most gaps. Most "compiler bugs" are actually lowerer bugs.
> - Verification is bypassed by default (`skip_verification: true`) because the lowerer
>   produces benign unwired ports. Fixing the lowerer unlocks strict verification.
> - The emit path produces real service transport code for 4 backends but stubs out
>   callable orchestration. Closing this gap means the compiled path can replace the
>   interpreted path for production binaries.
> - **Push magic to the left**: every implicit runtime behavior (auto-filling lists,
>   defaulting auth, guessing module names) should be an explicit compiler construct.
>   The executor should be maximally dumb — it only knows what the graph tells it.
> - **Preserve intent through lowering**: expanding service calls to 3-node triplets
>   during lowering destroys semantic intent and causes deduplication conflicts.
>   Keep high-level nodes; let the backend expand.
> - **Carry types forward**: the typechecker proves types, the lowerer erases them.
> - **Cross-lane dependency**: Lane 2 Bridge 1+2 (SubDag lowering for fn bodies)
>   is the single biggest open pain point — it causes the `make gist` unguarded
>   transport bug and `make install` compute node failures. This is Lane 2 work
>   that directly enables Lane 4's strict verification (CP-8, CP-23). Don't
>   duplicate it here; just know it's a prerequisite for the hardest CP items.
>   Typed ports would make edge validation automatic and emit trivial.

---

## WS-1: Eliminate Silent Failures (fail-closed pipeline)

> The pipeline should never silently drop data. Every parse, resolve, lower, or
> execution step either succeeds with correct output or fails with a diagnostic
> pointing to the source location. `lower_warn()` behind an env var is not
> acceptable — it should be a `LowerError`.

### CP-1: Fail on unresolved imports (S)

**File**: `core/daglang/daglang-resolve/src/lib.rs` ~line 168
**Problem**: If an import target doesn't exist on disk, the dependency is quietly omitted. `ResolveError::UnresolvedImport` is defined but never returned.
**Fix**: After building the dependency list, check that every import was resolved. Return `ResolveError::UnresolvedImport` for any that weren't.
**Verify**: `cargo test -p daglang-resolve` + add test that imports nonexistent module → error.

### CP-2: Fail on pattern node expansion failure (S)

**File**: `core/daglang/daglang-lower/src/lib.rs` ~line 4908
**Problem**: When a pattern body node expansion fails, the node is silently dropped via `lower_warn()`. Downstream port references fail with obscure "missing port" errors.
**Fix**: Convert `lower_warn()` to `return Err(LowerError::PatternExpansionFailed { ... })`.
**Verify**: `cargo test -p daglang-lower` — existing tests should still pass; add test for expansion failure → error.

### CP-3: Fail on unresolved service call arguments (S)

**File**: `core/daglang/daglang-lower/src/lib.rs` ~line 7233
**Problem**: Unresolved service call arguments are silently skipped. No error unless the executor fails later.
**Fix**: After the argument resolution cascade, if no match was found, return `LowerError::UnresolvedServiceArg { ... }`.
**Verify**: `cargo test -p daglang-lower` + test for unresolved arg → compile error.

### CP-4: Fail on unsupported expression types in pattern bodies (S)

**File**: `core/daglang/daglang-lower/src/lib.rs` ~line 5176
**Problem**: Unknown expression types are quietly skipped with a `lower_warn()`.
**Fix**: Return `LowerError::UnsupportedPatternExpr { ... }`.
**Verify**: Grep for remaining `lower_warn()` calls — each should have a justification or be converted to an error.

### CP-5: Surface FnBodyCallableOp evaluation errors (M)

**File**: `core/resolve/src/resolve.rs` ~line 243
**Problem**: `FnBodyCallableOp` catches all evaluation errors and returns `Value::Skipped` for all outputs. Real errors are hidden.
**Fix**: Propagate evaluation errors as `ExecError` instead of swallowing them. Reserve `Value::Skipped` for intentional skip (guard failure), not error masking.
**Verify**: `cargo test -p gunbc-resolve` — tests that relied on silent skip behavior may need mock updates.

---

## WS-2: Enable Strict DAG Verification

> `skip_verification: true` is the default because the lowerer produces benign
> unwired ports (param_source nodes, fn body cross-wiring gaps). Fix the lowerer,
> then flip the default.

### CP-6: Wire param_source nodes to caller scope (M)

**File**: `core/daglang/daglang-lower/src/lib.rs`
**Problem**: `param_source` nodes are created for service call args referencing callable params, but have no incoming data edges from the caller's scope.
**Fix**: When creating param_source nodes, wire them to the enclosing callable's input port with the matching name.
**Verify**: `cargo test -p daglang-lower` — count of nodes with zero incoming edges should decrease.

### CP-7: Wire default argument literal nodes (M)

**File**: `core/daglang/daglang-lower/src/lib.rs`
**Problem**: When a call omits an argument that has a default value, the lowerer does not emit a literal source node. The port gets no incoming edge, and the executor fills it with a mock (see `make gist` postmortem in Lane 2 doc).
**Fix**: For each omitted argument with a default expression, emit a `PrimitiveLiteral` source node and wire it to the call's input port.
**Verify**: `cargo test -p daglang-lower` + specific test: call with omitted default → literal node present in lowered DAG.

### CP-8: Flip `skip_verification` to false (S, after CP-6 + CP-7)

**File**: `core/daglang/daglang-driver/src/lib.rs` ~line 300
**Problem**: DAG verification is bypassed by default.
**Fix**: Change default to `skip_verification: false`. Fix any newly-surfaced unwired port errors.
**Verify**: `cargo test --workspace` — all compilation paths must pass strict verification.

---

## WS-3: Interpreted/Compiled Parity

> The goal is: if a .dag file works via the interpreted path, it should also
> produce correct compiled output. Parity should be tested automatically, not
> manually verified.

### CP-9: Parity test harness (M)

**Problem**: No automated test compares interpreted execution results against compiled binary output for the same .dag input.
**Fix**: Create a test harness that, for each tool .dag file:
  1. Compiles and executes via the interpreted path (Dag<DynOp> executor) in DryRun
  2. Emits Rust code via the emit path
  3. Compiles the emitted Rust
  4. Runs the compiled binary in DryRun
  5. Asserts output equality
**Location**: `core/daglang/daglang-driver/tests/parity_tests.rs` or similar.
**Verify**: Test passes for at least `pragma` and `makegen` (simplest tools).

### CP-10: Callable orchestration in Rust emit (L)

**File**: `core/daglang/daglang-emit/src/lib.rs`
**Problem**: Rust emit produces type defs and service transport code, but callable orchestration (the actual dataflow logic between nodes) is Phase 1 scaffold only.
**Fix**: Emit a topo-sorted execution function that mirrors the graph executor: gather inputs from upstream outputs, call each node's generated function, wire outputs to downstream inputs.
**Verify**: Emitted Rust compiles and produces same outputs as interpreted path (via CP-9 harness).

### CP-11: PureRender / fn body classification in exec-runtime (M)

**File**: `core/daglang/daglang-emit/src/rust_exec_runtime.rs`
**Problem**: Exec-runtime can't classify callables with fn bodies (RF-E5). PureRender callables get passthrough stubs.
**Fix**: Emit fn body evaluation code for Callable nodes that have `fn_body: Some(...)`.
**Verify**: `makegen` and `pragma` emit without passthrough stubs for fn-body nodes.

### CP-12: Go/C/MIPS callable stubs → real orchestration (XL, future)

**Problem**: Non-Rust backends have detailed service transport code but generic callable stubs.
**Fix**: Port the orchestration emitter (CP-10) to Go, C, and MIPS backends.
**Note**: Lower priority. Rust parity first; other backends follow the same pattern.

---

## WS-4: Diagnostic Quality

> When the compiler finds a contradiction, the error message is its **last act
> of usefulness** before stopping. Every diagnostic must answer: what went wrong
> (the contradiction), where (source location), and how to fix it (suggestion).
> Each stage is responsible for emitting errors loud enough, and with enough I/O
> context, that users (and earlier stages) can correlate to the actual failure.
>
> See design doc "When You Can't Store Tautology, Fail Helpfully" for the full
> principle and per-stage diagnostic contracts.

### CP-13: Remove debug eprintln from typecheck (S)

**File**: `core/daglang/daglang-typecheck/src/lib.rs` ~lines 2262, 2281, 2297, 2314, 2359
**Problem**: ~5 `eprintln!("[DEBUG callable_body] ...")` statements in production code.
**Fix**: Delete them or gate behind `DAGLANG_TYPECHECK_DEBUG=1` env var.
**Verify**: `cargo test -p daglang-typecheck` + `grep -r 'eprintln!' core/daglang/daglang-typecheck/src/` returns 0 ungated hits.

### CP-14: Source spans on lowerer errors (M)

**Problem**: Lowerer errors reference node names and port names, not .dag source locations. When lowering fails, users must mentally map node IDs back to their source.
**Fix**: Thread `Span` information from the AST through lowering. `LowerError` variants should carry `span: Span` (not Option) for source attribution.
**Verify**: At least service call arg errors (CP-3) and pattern expansion errors (CP-2) carry source spans.
**Note**: Subsumed by CP-46 (structured LowerError enum). Implement together.

### CP-15: Implement DryRunStrictness (M)

**File**: `core/exec/src/execute/mod.rs` ~line 50
**Problem**: `DryRunStrictness::Strict` is declared but never enforced. Strict mode should inject poison values for unmocked transport nodes so modeling gaps surface immediately.
**Fix**: When `Strict`, unmocked transport nodes return `ExecError::UnmockedTransport { node_id }` instead of default mocks.
**Verify**: `cargo test -p gunbc-exec` + test that strict dry-run fails on unmocked node.

### CP-16: Warn on `consume_brace_block_expr` (S)

**File**: `core/daglang/daglang-syntax/src/parser.rs` ~line 3367
**Problem**: Multi-statement brace blocks silently return empty Record with no diagnostic.
**Fix**: Emit a `ParseError::UnsupportedMultiStmtBlock { span }` diagnostic (can be non-fatal warning) so users know their lambda body was discarded.
**Verify**: `cargo test -p daglang-syntax` + test for multi-stmt lambda → warning emitted.

### CP-48: Unified Diagnostic type with span + context + help (M)

**Problem**: 10 different error types across 10 crates. No shared structure. Spans exist at parse, lost everywhere else. No help/suggestion field anywhere. Each stage's `Display` impl bakes suggestions into format strings — not machine-readable.
**Fix**: Introduce shared `Diagnostic` type (see design doc "Shared Types"):
```rust
struct Diagnostic {
    code: &'static str,          // "E0401" — machine-readable
    message: String,             // human-readable: the contradiction
    span: Span,                  // NOT optional for user-facing errors
    file: PathBuf,               // which .dag file
    context: DiagnosticContext,  // structured: TypeMismatch, Missing, Duplicate, Unsupported
    help: Option<String>,        // concrete fix suggestion
    related: Vec<RelatedSpan>,   // secondary locations ("first defined here")
}
```
Stage-specific error enums convert to `Vec<Diagnostic>` via `Into<Diagnostics>`. The `Verdict<T>` return type carries `Diagnostics` on FAIL.
**Impact**: Foundation for all other diagnostic improvements. Machine-readable errors enable IDE integration, programmatic error handling, and consistent rendering.
**Verify**: `cargo test --workspace` + all stage FAIL paths produce `Diagnostics` with non-empty `span` and `file`.
**Note**: Foundation item — land alongside or shortly after CP-36 (Verdict<T>).

### CP-49: Thread spans through TypeError (M)

**File**: `core/daglang/daglang-typecheck/src/lib.rs`
**Problem**: `TypeError` has 35+ structured variants but zero spans. The typechecker reads AST nodes that carry `Spanned<T>` wrappers — the span is right there but not attached to the error.
**Fix**: Add `span: Span` to every `TypeError` variant. When constructing errors, extract the span from the `Spanned<T>` AST node being checked. Convert to `Diagnostic` with context (`TypeMismatch { expected, got }`, `Missing`, etc.) and file path from `TypedProject.graph`.
**Example**: `TypeError::TypeMismatch { expected: "String", got: "Int" }` becomes `Diagnostic { code: "E0308", span: (42, 55), file: "tools/gist.dag", context: TypeMismatch { expected: "String", got: "Int" }, help: Some("change argument type to String") }`.
**Verify**: `cargo test -p daglang-typecheck` + every `TypeError` variant carries a span. `grep -r 'TypeError::' core/daglang/daglang-typecheck/src/ | grep -v 'span'` returns 0 hits.

### CP-50: Help text on common errors (M)

**Problem**: No error anywhere in the pipeline carries a fix suggestion. Users see what's wrong but not how to fix it. Fix suggestions are the highest-value diagnostic improvement for a DSL where users are often learning the language.
**Fix**: Add `help: Option<String>` to the most common error paths:
  - Parse: `"expected '}' to close block opened at line 12"`
  - Module Resolve: `"import std.rendor not found — did you mean std.render?"` (Levenshtein on available modules)
  - Typecheck: `"expected String, got Int for arg 'id' — change type or add cast"`
  - Lower: `"service call gist.Create missing required arg 'description' (String) — add description: \"...\" to the call"`
  - Verify: `"node compare_content has unwired input 'expected_content' — add content source or wire a default"`
  - Execute (DryRun): `"unmocked transport node 'execute_gist_create' — add mock in test fixture or run in Real mode"`
**Verify**: At least 10 of the most-hit error paths carry non-empty `help` text. Measure via test that constructs each error variant and asserts `help.is_some()`.

### CP-51: NodeOrigin on every lowered node for span traceability (M)

**Problem**: Lowered DAG nodes carry `NodeId` (a string) but no link to the source `.dag` location. Verify/derive/emit errors reference node IDs that users must manually correlate to source. Pattern-expanded nodes (transport triplets, content_upsert chains) have no link to the source pattern.
**Fix**: Add `origin: NodeOrigin` to every `Node<LoweredOp>`:
```rust
enum NodeOrigin {
    UserCode { span: Span, file: PathBuf },
    PatternExpansion { instance_id: PatternInstanceId, local_idx: u8 },
}
```
The lowerer sets `UserCode` for nodes derived from AST items. Pattern expansions set `PatternExpansion` with a backref to the source pattern instance (which itself carries the span). Verify/derive/emit read `origin` for error attribution.
**Impact**: All post-lowering errors can point to the `.dag` source. Subsumes CP-25 (pattern instance backrefs) for the span-tracing aspect.
**Verify**: `cargo test -p daglang-lower` + every node in a lowered DAG has `origin != None`. Test that transport triplet nodes trace back to the service call span.

---

## Ordering

```
WS-1 (silent failures)  ──→  WS-2 (strict verification)  ──→  WS-3 (parity)
                                                                     ↑
WS-4 (diagnostics) ──────────────────────────────────────────────────┘
```

WS-1 is prerequisite for WS-2: you can't enable strict verification until the
lowerer stops producing benign unwired ports.

WS-3 depends on WS-2: parity tests need reliable compilation. WS-4 can
proceed in parallel with everything — better diagnostics help debug all other
workstreams.

**Recommended start**: CP-1 + CP-13 (both S, independent) as warm-up, then
CP-2 through CP-5 (close the silent failures), then CP-6 + CP-7 → CP-8
(enable strict verification).

---

## WS-5: Tautology Enforcement (structural improvements)

> These are deeper architectural changes motivated by the "tautology preservation"
> principle. Each one eliminates a category of bugs by making invalid states
> unrepresentable, rather than catching them with runtime checks.

### CP-17: Typed ports in IR (L)

**Problem**: The typechecker proves types, but lowering erases them. `Port` has a name and cardinality but no type. Edges are validated by name matching only — a `String` port can be wired to an `Int` port and the error surfaces at execution time.
**Fix**: Add `port_type: Option<TypeId>` to `Port`. During lowering, carry type information from `TypedCallableSignature` into port definitions. Add edge validation: `from_port.type == to_port.type` (or compatible via subtyping).
**Impact**: Type mismatches become compile-time errors. Emit backends can read types directly from ports instead of reconstructing them. Unlocks typed code generation.
**Verify**: `cargo test -p daglang-lower` + test that wiring String→Int port produces `LowerError::TypeMismatch`.

### CP-18: Defer transport expansion to backend (L)

**Problem**: The lowerer expands service calls into 3-node transport triplets (prepare/execute/parse) and content_upsert into 5-node chains. This destroys the user's semantic intent and creates deduplication conflicts when multiple callables in one module call the same service.
**Fix**: Keep `ServiceCall { op: ServiceOpId, args, transport_kind, auth }` as a single fully-resolved node in `Dag<LoweredOp>`. Keep `ContentUpsert { path, content_source }` as a single node. The canonical `VerifiedDag` preserves high-level intent. Backend expansion (Resolve for interpreted, Emit for compiled) produces `RealizedDag<LoweredOp>` — a separate type where transport triplets are materialized. This expansion is a **total function** (cannot fail) because all decisions were already made during typecheck/lower.
**Impact**: Solves transport node deduplication structurally. Lowered DAGs are smaller and closer to the user's intent. Splits "decision" (early, can fail) from "realization" (late, provably total).
**Verify**: `cargo test -p daglang-lower` — lowered DAGs have fewer nodes; service call node count == number of service calls in source.
**Note**: This is a significant refactor. Depends on CP-40 (ExternRegistry) for fully-resolved service ops. The resolve stage and emit stage both need to understand the new high-level nodes. Plan carefully.

### CP-19: Remove `allow_unresolved_references` from compilation path (M)

**File**: `core/daglang/daglang-typecheck/src/lib.rs`
**Problem**: When `allow_unresolved_references=true`, missing call targets are silently skipped. This breaks the tautology: `TypedProject` claims the program is sound, but it isn't. Downstream lowering fails with cryptic "missing node" errors instead of clear "unresolved reference" errors.
**Fix**: Remove the flag from `TypecheckOptions` for the main compilation path. If partial checking is needed for IDE/LSP, create a separate `typecheck_for_ide()` entry point that returns diagnostics without halting.
**Verify**: `cargo test -p daglang-typecheck` + `grep -r 'allow_unresolved' core/daglang/` — only IDE path uses it.

### CP-20: Explicit empty-list wiring in lowerer (S)

**Problem**: The executor auto-fills unwired optional list ports with `Value::List(vec![])`. This is "lying by invention" — the executor is manufacturing data that no stage produced.
**Fix**: When the lowerer creates a node with an optional list input that has no incoming edge, emit a `Primitive::MakeLiteral(List([]))` node and wire it explicitly.
**Impact**: The executor can be simplified — unwired ports are always an error, no special-casing needed.
**Verify**: `cargo test -p gunbc-exec` + remove auto-fill logic from executor; all tests still pass.

### CP-21: Separate parser recovery modes (S)

**File**: `core/daglang/daglang-syntax/src/parser.rs`
**Problem**: `parse_body_lossy()` truncates errors and returns empty bodies. This is used in the main compilation path, not just IDE tooling. It breaks the tautology: the AST claims the body is empty when the source text has content.
**Fix**: Remove `lossy` recovery from the main `parse()` entry point. Create a separate `parse_for_ide()` that uses lossy recovery and returns partial ASTs with diagnostics. The main path should fail fast on invalid syntax.
**Verify**: `cargo test -p daglang-syntax` — compilation-path tests never see `lossy: true`.

### CP-22: Reject "unknown" module paths (S)

**File**: `core/daglang/daglang-resolve/src/lib.rs` ~line 318
**Problem**: Files outside configured roots get module path `["unknown"]`, making multiple such files indistinguishable. This is "lying by invention."
**Fix**: Return `ResolveError::FileOutsideRoot { path }` instead of inventing a fake module path.
**Verify**: `cargo test -p daglang-resolve` + test that out-of-root file → error.

---

## WS-6: Structural Invariants

> These enforce invariants at the type level or via tests, so violations
> become impossible rather than merely unlikely.

### CP-23: VerifiedDag<T> type wrapper (M)

**Problem**: `skip_verification` is a boolean flag. Nothing prevents passing an unverified DAG to Resolve or Emit. CP-8 flips the default, but a flag can always be flipped back.
**Fix**: Introduce `VerifiedDag<T>` as a newtype wrapper around `Dag<T>`. The only way to construct it is `verify_dag(dag) -> Result<VerifiedDag<T>, Vec<VerifyError>>`. Both `resolve_lowered_dag_with()` and `emit_*_bundle()` take `&VerifiedDag<LoweredOp>` as input.
**Impact**: Verification becomes a structural requirement. The `skip_verification` flag becomes a test-only escape hatch, not a production default.
**Verify**: `cargo test --workspace` — all production paths go through `VerifiedDag`. `grep -r 'skip_verification' core/` shows only test contexts.

### CP-24: Resolution topology invariant (S)

**Problem**: The Resolve stage should be a pure relabeling (swap op bodies, preserve topology). Nothing enforces this — resolution could accidentally add/remove nodes or edges.
**Fix**: Add a post-resolve assertion: `assert_eq!(resolved.nodes.len(), lowered.nodes.len())` and `assert_eq!(resolved.edges.len(), lowered.edges.len())`. Wrap in a `debug_assert!` for production, hard assert in tests.
**Verify**: `cargo test -p gunbc-resolve` + add dedicated test that resolution preserves node/edge counts.

### CP-25: Pattern instance backrefs in lowered DAG (M)

**Problem**: When the lowerer expands service calls (→ 3 nodes), content_upsert (→ 5 nodes), and loops (→ unpack/body/pack), the generated nodes have no link back to the source pattern. Errors reference generated node names, not the user's service call at line 42.
**Fix**: Add `PatternInstance { kind: PatternKind, source_span: Span, inner_node_ids: Vec<NodeId> }` to the lowered DAG metadata. Each generated node carries a `pattern_instance_id: Option<PatternInstanceId>`.
**Impact**: Better error messages. Verification can structurally recognize "this 5-node chain IS a content_upsert." Emit can reconstruct high-level intent without reverse-engineering.
**Verify**: `cargo test -p daglang-lower` + test that every transport triplet node has a `pattern_instance_id` linking to a `ServiceCall` instance.

### CP-26: ParseResult { ast, diagnostics } (M)

**File**: `core/daglang/daglang-syntax/src/lib.rs`
**Problem**: `parse()` returns `Result<SourceFile, Vec<ParseError>>` — either a complete AST or a list of errors. In practice, partial ASTs are useful (IDE, error recovery), and the current `lossy` mode works around this by silently mutating the AST.
**Fix**: Change signature to:
```rust
struct ParseResult { ast: SourceFile, diagnostics: Vec<ParseDiagnostic> }
fn parse(source: &str) -> ParseResult;
```
The parser always produces an AST (possibly with `Expr::ErrorNode { span }` placeholders for unparseable regions). Diagnostics are separate. The main compilation path checks `diagnostics.is_empty()` and halts if not. IDE path uses the partial AST.
**Impact**: Eliminates `lossy` mode, `parse_body_lossy()`, and the `Result` bifurcation. One parser entry point, two consumers.
**Verify**: `cargo test -p daglang-syntax` + `grep -r 'lossy' core/daglang/daglang-syntax/` returns 0 hits.
**Note**: Subsumes CP-16 and CP-21 — those become unnecessary once ParseResult exists.

### CP-27: Model skipping as control flow, not Value variant (L)

**Problem**: `Value::Skipped` is a variant in the `Value` enum, meaning every value consumer must handle "what if this is Skipped?" It conflates two meanings: intentional skip (guard failure) and error masking (fn body evaluation failed).
**Fix**: Remove `Value::Skipped`. Skipping becomes a per-node control edge effect:
  - A node with a guard that evaluates to false is not executed.
  - Its outputs are never materialized (not "Skipped values", just absent).
  - Downstream nodes that depend on absent outputs are transitively skipped (control edge propagation) or receive explicit default values from the lowerer.
**Impact**: Eliminates "what does Skipped mean here?" from every value consumer. Makes skip behavior visible in graph structure. Forces explicit defaults where needed (push magic to the left).
**Verify**: `cargo test --workspace` + `grep -r 'Value::Skipped' core/` returns 0 hits outside of tests.
**Note**: Large refactor. Depends on CP-20 (explicit empty-list wiring) being done first so the executor can treat missing inputs as hard errors.

### CP-28: Emit `todo!()` → `EmitError::UnsupportedFeature` (S)

**Files**: `core/codegen/src/rest_gen.rs`, `core/codegen/src/lambda_gen.rs`
**Problem**: REST and Lambda exposure `generate()` methods use `todo!()`, which panics at runtime. A tautological pipeline should be honest about what it doesn't support.
**Fix**: Replace `todo!()` with `return Err(EmitError::UnsupportedFeature { feature: "REST exposure", span })`. Callers get a clear diagnostic instead of a panic.
**Verify**: `cargo test --workspace` + `grep -r 'todo!()' core/codegen/src/` returns 0 hits in generate() methods.

---

## WS-7: Resolve Early (eliminate deferrals)

> If information is available at Stage N, resolve it at Stage N. Every
> deferral creates an ambiguous intermediate state that later stages handle
> with fallbacks — violating binary logic.

### CP-29: Validate required inputs after lower (S)

**Problem**: Required (scalar, `Cardinality::ONE`) input ports without incoming edges are not detected until runtime, where they surface as opaque "missing required input" executor errors.
**Fix**: After lowering, iterate all nodes. For each required input port, check `has_incoming_edge || has_mock`. If not, return `LowerError::UnwiredRequiredInput { node, port, span }`.
**Verify**: `cargo test -p daglang-lower` + test that a missing required input → compile-time error. Catches the `make gist` class of bugs (`missing required 'expected_content' input for compare_content`).

### CP-30: Emit transport resource ports during lower (M)

**File**: `core/resolve/src/resolve.rs` ~line 1803
**Problem**: Transport nodes that need `res:file` (filesystem handle) don't declare that dependency until the Resolve stage, which introspects node kind and injects resource ports. Resource dependencies are invisible in the lowered IR.
**Fix**: During lowering, when creating content_upsert execute nodes or service transport execute nodes, emit `res:file` input ports with the correct `AccessMode` directly. Remove `needs_transport_resource()` and `wire_missing_filesystem_resources()` from resolve.
**Verify**: `cargo test -p daglang-lower` + `grep -r 'needs_transport_resource' core/resolve/` returns 0 hits.

### CP-31: Validate auth scheme in typecheck (S)

**File**: `core/daglang/daglang-typecheck/src/lib.rs`
**Problem**: Auth scheme strings pass through typecheck unvalidated. The lowerer does a passthrough match (`other => other.to_string()`), and resolve defaults missing schemes to `"none"`. Unknown schemes produce silently invalid IR.
**Fix**: In typecheck, validate `config.auth` against known enum: `BearerToken | Basic | ApiKey | None`. Reject unknown values with `TypeError::InvalidAuthScheme { span, value }`.
**Verify**: `cargo test -p daglang-typecheck` + test that `config { auth: "Bogus" }` → type error.

### CP-32: Eliminate TypedModule field duplication (M)

**File**: `core/daglang/daglang-typecheck/src/lib.rs`
**Problem**: `TypedModule` copies `path`, `module_path`, `imports`, and `ast` from `ResolvedModule`. The AST is validated but never mutated — it's the same object. This violates "stages add, never restate."
**Fix**: `TypedModule` carries `graph_index: usize` (index into `ModuleGraph.modules`) plus `signatures: Vec<TypedItemSignature>`. `TypedProject` holds a reference or owned `ModuleGraph`. Callers that need `path` or `ast` go through the graph index.
**Verify**: `cargo test --workspace` + `TypedModule` struct has no `path`, `module_path`, or `ast` fields.

### CP-33: Compute CompileOutput fields once, not re-extract (M)

**File**: `core/daglang/daglang-driver/src/lib.rs` ~lines 739-793
**Problem**: After lowering, the driver re-walks the TypedProject AST to extract `output_paths`, `pipeline_params`, `dsl_type_registry`, `available_profiles`, `data_values`, and `inferred_entrypoints`. All of these are derivable from information already computed during earlier stages.
**Fix**: Compute each value once during its canonical stage and thread it forward:
  - `output_paths` → computed during lower (already walks content_upsert nodes)
  - `dsl_type_registry` → computed during typecheck
  - `available_profiles` → computed during typecheck
  - `pipeline_params` → computed during typecheck
  - `data_values` → computed during lower
  - `inferred_entrypoints` → computed during lower
**Verify**: `cargo test --workspace` + `grep -r 'extract_output_paths\|collect_pipeline_params\|extract_dsl_type_registry\|collect_available_profiles\|build_data_values\|infer_entrypoints' core/daglang/daglang-driver/` — these extraction functions are removed from driver, moved to their canonical stage.

### CP-34: Derive uses Node.kind, not port string heuristics (S)

**File**: `core/daglang/daglang-derive/src/lib.rs` ~line 195
**Problem**: `derive_transport_triplets()` identifies transport nodes by matching port type strings ("TransportRequest", "TransportResponse"). But the lowerer already stamps nodes with `Node.kind` and `Node.transport_class`.
**Fix**: Use `node.kind == NodeKind::TransportPrepare` etc., instead of parsing port type strings. Delete the string-matching heuristic.
**Verify**: `cargo test -p daglang-derive` + `grep -r 'TransportRequest' core/daglang/daglang-derive/` returns 0 hits.

### CP-35: Stamp loop metadata on SubDag, don't re-detect (S)

**Problem**: The executor's `detect_loop_pattern()` reverse-engineers 3-node SubDag structure (checking `nodes.len() == 3`, finding by ID "unpack"/"pack") at execution time. The lowerer already knows this is a loop when it creates the SubDag.
**Fix**: Add `SubDagKind { Loop(LoopInfo), Branch, Generic }` to `NodeBody::SubDag`. The lowerer stamps the kind at creation. The executor reads `SubDagKind::Loop(info)` directly — no pattern detection.
**Verify**: `cargo test -p gunbc-exec` + `grep -r 'detect_loop_pattern' core/exec/` returns 0 hits.

---

## WS-8: Cross-Cutting Patterns

> Implementation patterns that span multiple workstreams. These establish
> conventions that all other CP items should follow.

### CP-36: Verdict<T> result type for all stage APIs (M)

**Problem**: Stage APIs use inconsistent result types: `Result<T, Vec<Error>>`, `Result<T, Error>`, flags like `lossy`, etc. No unified PASS/FAIL contract.
**Fix**: Introduce a shared `Verdict<T>` type:
```rust
type Verdict<T> = Result<T, Diagnostics>;
struct Diagnostics { errors: Vec<Diagnostic>, warnings: Vec<Diagnostic> }
```
All main-path stage APIs (`parse`, `discover_module_graph`, `typecheck_strict`, `lower_strict`, `verify_dag`) return `Verdict<T>`. Non-empty `errors` = FAIL. Warnings never affect PASS/FAIL.
**Impact**: Eliminates all third-state signals at the API boundary. IDE/tooling helpers are separate functions.
**Verify**: `cargo test --workspace` + all stage entry points return `Verdict<T>`.
**Note**: This is a foundation item — do it before or alongside WS-1 work so new error returns use the right type.

### CP-37: side_effecting annotation for DryRun (S)

**Problem**: DryRunStrictness (CP-15) needs a way to know which nodes perform I/O. Currently this is inferred from node kind at execution time.
**Fix**: The lowerer stamps each node with `side_effecting: bool`. In `DryRun(Strict)`, encountering a `side_effecting` node without an explicit mock returns `ExecError::StrictDryRunBlocked { node_id }`.
**Verify**: `cargo test -p daglang-lower` + test that transport execute nodes have `side_effecting: true`, pure fn nodes have `side_effecting: false`.
**Note**: Prerequisite for CP-15 (DryRunStrictness). Land this first so CP-15 has a clean implementation.

### CP-38: Parser keeps interface type defs (S)

**File**: `core/daglang/daglang-syntax/src/parser.rs` ~line 2121
**Problem**: `parse_interface_def()` parses `type` items inside interfaces but discards them (`let _ = self.parse_type_def()`). This violates "AST == Source Text."
**Fix**: Add `InterfaceItem::TypeDef(TypeDef)` variant. Store parsed type defs in `InterfaceDef`. If downstream stages don't support them yet, typecheck rejects with `TypeError::UnsupportedFeature { feature: "interface type defs" }`.
**Verify**: `cargo test -p daglang-syntax` + test that `interface Foo { type Bar { x: Int } }` produces an AST with the type def present.

### CP-39: CompileReceipt is not Option (S)

**File**: `core/daglang/daglang-driver/src/lib.rs`
**Problem**: `CompileOutput.receipt: Option<CompileReceipt>`. If compilation succeeds (PASS), a receipt MUST exist. The `Option` creates a third state: "succeeded but no receipt."
**Fix**: Change to `receipt: CompileReceipt`. Compute the receipt unconditionally on success. Callers no longer need `if let Some(receipt)`.
**Verify**: `cargo test --workspace` + `grep -r 'Option<CompileReceipt>' core/` returns 0 hits.

---

## WS-9: Extern & Verification Consolidation

> These items push extern validation left (to typecheck) and consolidate
> verification into a single mandatory step.

### CP-40: ExternRegistry — validate externs in typecheck (M)

**Problem**: Extern symbol resolution happens at Stage 5a (Resolve). Unknown externs aren't detected until you try to run the graph. The ExternResolver trait takes `(module, name)` strings and returns `Option<DynOp>` — a runtime lookup that can silently return `None`.
**Fix**: Introduce `ExternRegistry { symbols: Vec<ExternSymbol> }` where `ExternSymbol { module, name, signature: TypedCallableSignature, id: ExternId }`. Typecheck consumes the registry and proves every `extern func` call resolves to an `ExternId` with a matching signature. The lowerer carries `ExternId`, not `(module, name)` strings. Stage 5a resolve becomes total — it cannot fail on missing externs.
**Impact**: Eliminates a class of "symbol not found" runtime errors. Makes the `ExternResolver::resolve()` return type non-optional (total function). Pairs with CP-23 (VerifiedDag) to make the resolved path provably complete.
**Verify**: `cargo test --workspace` + `grep -r 'ExternCall.*module.*name' core/daglang/daglang-lower/` returns 0 hits (all carry ExternId).

### CP-41: Merge validate_structural_primitive_wiring() into verify() (S)

**File**: `core/daglang/daglang-driver/src/lib.rs`
**Problem**: Two separate validation passes run after lowering: `validate_structural_primitive_wiring()` (unconditional) and `verify_dag()` (conditional, `skip_verification`). Under PASS/FAIL, there should be exactly one structural proof step.
**Fix**: Move the primitive wiring checks into `verify_dag()`. Delete `validate_structural_primitive_wiring()` as a separate function. `verify()` becomes the single mandatory verification step that returns `VerifiedDag`.
**Verify**: `cargo test --workspace` + `grep -r 'validate_structural_primitive_wiring' core/` returns 0 hits.

### CP-42: Dag<T>.map_bodies() for topology preservation (S)

**Problem**: The Resolve stage should only relabel node bodies (symbolic → executable), never touch topology. Currently `resolve_lowered_dag_with()` manually constructs a new `Dag<DynOp>` — nothing prevents it from accidentally adding/removing nodes or edges.
**Fix**: Add `Dag<T>.map_bodies<U>(f: impl FnMut(T) -> U) -> Dag<U>` that preserves nodes, edges, and ports exactly, only transforming the body label. Rewrite `resolve_lowered_dag_with()` to use `map_bodies()`.
**Impact**: Makes topology preservation structural, not just tested. The CP-24 assertion becomes redundant because the type system prevents violations.
**Verify**: `cargo test -p gunbc-ir` + `cargo test -p gunbc-resolve` — resolution uses `map_bodies`, manual node/edge construction removed.

---

## WS-10: Stage Interface Cleanup (clear contracts)

> Every stage should be a pure function with typed I/O. The signature IS the
> contract. See design doc "Target Stage Interfaces" for the full specification.

### CP-43: Typecheck borrows ModuleGraph instead of moving it (M)

**File**: `core/daglang/daglang-typecheck/src/lib.rs`
**Problem**: `typecheck_module_graph(graph: ModuleGraph)` takes ownership. Later stages that need module data must get it from the `TypedProject` copy. This forces TypedModule to duplicate `path`, `module_path`, `imports`, `ast`.
**Fix**: Change signature to `typecheck(graph: &ModuleGraph, externs: &ExternRegistry) -> Verdict<TypedProject>`. `TypedProject` holds `graph: &'a ModuleGraph` (by reference) plus its own `signatures` map. CP-32 (eliminate field duplication) becomes a natural consequence.
**Impact**: Eliminates forced copy. `ModuleGraph` stays alive in the caller (driver). `TypedProject<'a>` is a lifetime-scoped overlay, not a standalone blob.
**Verify**: `cargo test --workspace` + `TypedModule` struct has no `path`, `module_path`, `imports`, or `ast` fields.
**Note**: Subsumes CP-32. Implement together.
**Current branch note (2026-03-06)**: The borrow landed, but `TypedModule`
still clones module facts. Treat CP-43/CP-32 as only partially complete until
the typed overlay stops duplicating `ResolvedModule` data.

### CP-44: LowerOutput replaces driver re-extraction (M)

**File**: `core/daglang/daglang-lower/src/lib.rs`, `core/daglang/daglang-driver/src/lib.rs`
**Problem**: After lowering, the driver re-walks the DAG and TypedProject to extract `output_paths`, `data_values`, `inferred_entrypoints`, `pipeline_params`, `dsl_type_registry`, `available_profiles`. These are all derivable during lowering but computed after the fact.
**Fix**: Lower returns `LowerOutput { dag, output_paths, data_values, entrypoints }`. The driver consumes these directly — no re-extraction functions. (Type registry and profiles come from `TypedProject`, computed during typecheck.)
**Impact**: Eliminates 6 post-hoc extraction functions from the driver. Each piece of data is computed once, during its canonical stage.
**Verify**: `cargo test --workspace` + `grep -r 'extract_output_paths\|infer_entrypoints\|build_data_values' core/daglang/daglang-driver/` returns 0 hits.
**Note**: Subsumes CP-33. Implement together.
**Current branch note (2026-03-06)**: `LowerOutput` now exists and the main
compile path consumes bundled `output_paths`, `inferred_entrypoints`, and
`data_values`. Helper/embedded paths still use some legacy extraction helpers,
so the single-owner cleanup is not fully finished yet.

### CP-45: Consolidate Execute entry points into one (S)

**File**: `core/exec/src/execute/mod.rs`
**Problem**: Four progressively wider entry points: `execute()`, `execute_with_mode()`, `execute_with_mode_and_inputs()`, `execute_with_mode_and_inputs_and_detail()`. Growing parameter lists hide the real contract.
**Fix**: One entry point: `fn execute(dag: &Dag<DynOp>, config: &ExecConfig) -> Verdict<ExecResult>`. `ExecConfig` carries mode, mocks, and detail level. `ExecResult` carries outputs + log.
**Verify**: `cargo test --workspace` + `grep -r 'execute_with_mode' core/exec/` returns 0 hits (only `execute` remains).

### CP-46: Structured LowerError with spans (M)

**File**: `core/daglang/daglang-lower/src/lib.rs`
**Problem**: `LowerError { node_id: Option<String>, reason: String }` — unstructured, no span, `Option` for required context. When lowering fails, users get a bare string with no source location.
**Fix**: Replace with a proper enum:
```rust
enum LowerError {
    UnwiredRequiredInput { node: NodeId, port: PortName, span: Span },
    PatternExpansionFailed { pattern: String, span: Span, reason: String },
    UnresolvedServiceArg { service: String, arg: String, span: Span },
    UnsupportedExpr { expr_kind: String, span: Span },
    // ... one variant per failure mode
}
```
Every variant carries a `Span` for source attribution. No `Option<String>` node IDs.
**Impact**: Lowerer errors point to the `.dag` source line, not internal node names. Pairs with CP-36 (Verdict<T>) for consistent error reporting.
**Verify**: `cargo test -p daglang-lower` + `LowerError` has no `String`-only reason fields.
**Note**: Subsumes CP-14 (source spans on lowerer errors). Implement together.

### CP-47: RuntimeBindings replaces ExternResolver trait (M, after CP-40)

**File**: `core/resolve/src/lib.rs`
**Problem**: `ExternResolver` is a trait with `resolve(module: &str, name: &str) -> Option<DynOp>`. Stringly-typed, partial (returns `Option`), runtime lookup. After CP-40 (ExternRegistry), extern symbols have validated `ExternId`s.
**Fix**: Replace `dyn ExternResolver` with `RuntimeBindings { externs: HashMap<ExternId, DynOp> }`. The binding is total — every `ExternId` has a value. Resolution via `map_bodies()` indexes by `ExternId`, not string lookup.
**Impact**: Resolution becomes total (cannot fail on missing externs). Eliminates the `ExternResolver` trait, `NullExternResolver`, `GunbcExternResolver`. The resolve stage signature becomes `fn resolve(verified: &VerifiedDag<LoweredOp>, bindings: &RuntimeBindings) -> Verdict<Dag<DynOp>>`.
**Verify**: `cargo test --workspace` + `grep -r 'dyn ExternResolver' core/` returns 0 hits.
**Current branch note (2026-03-06)**: The repo has a bridge version of
`RuntimeBindings` today, but it is still keyed by `(module, name)` strings and
implements `ExternResolver` for migration compatibility. ExternId-keyed total
binding remains open.

---

## WS-11: Obligation Boundary (move proofs from testgen to compiler)

> Today, when the compiler can't prove a property, it punts to testgen as a
> proof obligation. Testgen then re-derives metadata that the DSL author already
> declared. The goal: preserve enough metadata through the IR that testgen can
> trust the compiler's output directly, instead of guessing.
>
> See design doc "The obligation boundary" for the full motivation.

### CP-52: Preserve service provider metadata on lowered nodes (M)

**Problem**: The DSL declares `service github.Gist { operation Create { ... } }`. The lowerer knows this is a GitHub service. But after lowering, testgen's `auto_mock.rs` re-infers the provider by parsing the node ID string: `if node_id.to_lowercase().contains("gist") { MockProvider::GitHub }`. This is fragile — if a service is misnamed, provider inference fails and testgen falls back to a "kitchen sink" mock with fields from all providers.
**Fix**: Add `provider: Option<String>` and `operation: Option<String>` to `ServiceCallMetadata` (already exists on `LoweredOp::Callable`). The lowerer sets these from the service definition. Testgen reads `service_metadata.provider` instead of parsing node IDs.
**Impact**: Eliminates `infer_provider_from_node_id()` heuristic in `auto_mock.rs`. Mock responses are precise per-provider. Kitchen sink fallback becomes rare.
**Verify**: `cargo test -p gunbc-test` + `grep -r 'infer_provider_from_node_id' core/test/` returns 0 hits.

### CP-53: Flow response contracts through IR to testgen (L)

**Problem**: The DSL declares `response { 200 => GistResponse, 404 => NotFound }` on service operations. The parser and typechecker validate this. But the response type map doesn't flow through to the lowered IR. When testgen needs to synthesize mock responses, it probe-executes downstream consumer nodes in `Real` mode to find which response shape "fits" — executing the DAG at mock-generation time. This is slow, fragile, and wrong (side effects during test generation).
**Fix**: Add `response_contract: Option<Vec<(StatusCode, TypeId)>>` to `ServiceCallMetadata`. The lowerer populates it from the parsed `response { }` block. `derive_artifacts()` extracts it into `DerivedArtifacts`. Testgen reads the contract directly to synthesize correctly-typed mock responses.
**Impact**: Eliminates `response_candidate_satisfies_consumers()` probe-execution. Mock responses are typed from source declarations. Testgen never executes the DAG during generation.
**Verify**: `cargo test -p gunbc-codegen` + `grep -r 'response_candidate_satisfies' core/test/` returns 0 hits. Auto-mock no longer calls `execute()` at generation time.

### CP-54: Derive behavioral properties once, flow to testgen (S)

**Problem**: `readonly` and `idempotent` are declared on service operations and validated by the typechecker. But testgen re-derives them via `CallableProperties` BFS graph walk in `daglang-derive`. The derive pass walks the entire graph from each func entrypoint, collecting transport classes and behavioral flags — work that the typechecker already did.
**Fix**: Attach `readonly: bool` and `idempotent: bool` to `ServiceCallMetadata` during lowering. The derive pass reads these flags directly instead of BFS-walking the graph. Testgen reads from `DerivedArtifacts.callable_properties` which are now pre-computed.
**Impact**: Eliminates redundant BFS walks. `CallableProperties` derivation becomes a lookup, not a graph traversal. Aligns with Invariant 2 (minimalism — no redundancy).
**Verify**: `cargo test -p daglang-derive` + derive pass no longer BFS-walks for `readonly`/`idempotent` (reads from node metadata).

### CP-55: Track obligation provenance — what the compiler proved vs what testgen must check (M)

**Problem**: Testgen's `collect_obligations()` generates proof obligations for properties the compiler couldn't prove. But there's no tracking of *why* — which compiler stage should have proved it, and what information was missing. Without this, we can't measure whether pipeline improvements are actually moving the obligation boundary left.
**Fix**: Add `provenance: ObligationProvenance` to `ProofObligation`:
```rust
enum ObligationProvenance {
    CompilerGap { stage: &'static str, reason: String },
    InherentlyRuntime { reason: String },
}
```
`CompilerGap` means "the compiler should have proved this but couldn't" (with the stage and reason). `InherentlyRuntime` means "this can only be tested at runtime by design" (e.g., live API response shapes). Count of `CompilerGap` obligations is a ratchet — it should only go down.
**Impact**: Measurable metric for obligation boundary movement. Each CP item that improves the compiler should reduce `CompilerGap` count. CI can enforce the ratchet.
**Verify**: `cargo test -p gunbc-codegen` + `CompilerGap` count is tracked per testgen run. Baseline established.

### CP-56: Eliminate testgen re-derivation of transport triplets (S)

**Problem**: `derive_transport_triplets()` in `daglang-derive` identifies transport nodes by matching port type strings ("TransportRequest", "TransportResponse"). This is a heuristic that re-derives structure the lowerer already created. CP-34 addresses the derive side, but testgen also independently walks the DAG to find transport executor nodes via `NodeKind::TransportExecute`.
**Fix**: After CP-34 (derive uses `Node.kind`), ensure testgen also reads from `DerivedArtifacts.transport_triplets` instead of independently walking the DAG. Testgen's `analyze.rs` should consume derive output, not re-analyze the graph.
**Impact**: Single source of truth for transport structure. Testgen analysis becomes a lookup, not a graph walk.
**Verify**: `cargo test -p gunbc-codegen` + testgen's `analyze.rs` reads from `DerivedArtifacts`, not from raw DAG inspection.

### CP-57: Vfs trait / Source Ingest stage — isolate filesystem impurity (M)

**Problem**: Module resolve interleaves filesystem reads (walking directories, reading `.dag` files) with parsing and graph construction. This means the parser, typechecker, and lowerer are all transitively impure — they can't be tested with synthetic inputs without mocking the filesystem.
**Fix**: Introduce a `Vfs` trait (`read_to_string`, `list_files`) and a Stage 0: Ingest that captures all filesystem access into an immutable `SourceBundle`. Module resolve becomes a pure function over `SourceBundle` — it doesn't read the filesystem.
**Impact**: Every stage after ingest is a pure function of its inputs. Tests can use synthetic `SourceBundle` values without filesystem mocking. Enables parallel compilation (each source bundle is independent).
**Verify**: `cargo test --workspace` + `grep -r 'std::fs::read_to_string' core/daglang/daglang-resolve/` returns 0 hits (all reads go through `Vfs`).
**Note**: See design doc "Stage 0: Ingest" for the full interface sketch.

### CP-58: Contracts crate — shared interface types only (S)

**Problem**: Stage crates have ad-hoc error types and no shared interface boundary. Each stage invents its own result type (`Vec<ParseError>`, `Vec<TypeError>`, `LowerError { .. }`, etc.). Shared types like `Verdict<T>`, `Diagnostics`, `Span`, `FileId`, `ExternId` have no single home.
**Fix**: Create `core/daglang/daglang-contracts/` containing only: `Verdict<T>`, `Diagnostics`, `Diagnostic`, `Span`, `FileId`, `ModuleId`, `ExternId`, `TypeId`, and the stage function signatures. Zero implementation code. All stage crates depend on it.
**Impact**: Turns the "pure function" interface style into an enforced boundary. Adding a hidden dependency requires changing the contract crate, which is visible to everyone.
**Verify**: `daglang-contracts` crate exists. `cargo test -p daglang-contracts` passes. All stage crates list it as a dependency.
**Note**: Land alongside or shortly after CP-36 + CP-48 (which create the types that live here).

---

## Pain Point Correlation

These are real bugs from recent weeks, mapped to the CP items that would have
prevented them. Items marked **DONE** have already been fixed. Items marked
**GAP** were not covered until this audit.

| Bug | Root cause | Caught by | Status |
|-----|-----------|-----------|--------|
| `make gist` DNS error (Bug 1) | Transport nodes for unreachable match arms | CP-18 (defer expansion) + Lane 2 Bridge 1+2 | Open (Lane 2) |
| `make gist` audience=true (Bug 2) | Default params not injected into DAG | CP-7 | **DONE** (Bridge 2b) |
| `gunbc-ci` false failure | Complex return exprs silently dropped | CP-2, CP-4 | Open |
| `make gist` 401 | No credential wiring to execute node | CP-6 (param_source), CP-29 (validate required inputs) | Open |
| SDLC won't compile | Unresolved imports silently dropped | CP-1 | Open |
| Testgen kitchen sink mocks | Provider metadata lost after lowering | CP-52 | Open (**GAP** — added) |
| Testgen probe-execution | Response contracts not tracked through IR | CP-53 | Open (**GAP** — added) |
| Testgen redundant BFS | Behavioral properties re-derived | CP-54 | Open (**GAP** — added) |

---

## Summary

| WS | Items | Theme | Invariant |
|----|-------|-------|-----------|
| WS-1 | CP-1 through CP-5 | Eliminate silent failures | Binary logic |
| WS-2 | CP-6 through CP-8 | Enable strict DAG verification | Binary logic |
| WS-3 | CP-9 through CP-12 | Interpreted/compiled parity | Minimalism |
| WS-4 | CP-13 through CP-16, CP-48 through CP-51 | Diagnostic quality | Binary logic + Clear contracts |
| WS-5 | CP-17 through CP-22 | Tautology enforcement | All four |
| WS-6 | CP-23 through CP-28 | Structural invariants | Binary logic + Minimalism |
| WS-7 | CP-29 through CP-35 | Resolve early | Resolve early + Minimalism |
| WS-8 | CP-36 through CP-39 | Cross-cutting patterns | All four |
| WS-9 | CP-40 through CP-42 | Extern & verification consolidation | All four |
| WS-10 | CP-43 through CP-47 | Stage interface cleanup | Clear contracts |
| WS-11 | CP-52 through CP-56 | Obligation boundary (testgen) | Minimalism + Resolve early |
| WS-12 | CP-57, CP-58 | Purity & boundaries | Clear contracts |

## Ordering

```
CP-36 (Verdict<T>)  ──→  WS-1 (silent failures)  ──→  WS-2 (strict verification)  ──→  WS-3 (parity)
                                     ^                             ^
          WS-4 (diagnostics) ────────┘                             |
                                                                   |
          WS-5 (tautology enforcement) ───────────────────────────-┘
                                     ^                             ^
          WS-6 (structural invariants)  ────────────-──────────────┘
                                     ^                             ^
          WS-7 (resolve early)  ─────┘                             |
                                                                   |
          WS-8 (cross-cutting) ───────────────────────────────────-┘
                                                                   |
          WS-9 (extern + verify consolidation) ────────────────────┘
                                                                   |
          WS-10 (interface cleanup) ───────────────────────────────┘
                                                                   |
          WS-11 (obligation boundary) ─────────────────────────────┘
```

### Subsumption

Some later items subsume earlier ones. When implementing, do them together:

| Later item | Subsumes | Reason |
|------------|----------|--------|
| CP-43 (typecheck borrows ModuleGraph) | CP-32 (eliminate TypedModule duplication) | Borrowing removes the forced move; full subsumption only lands once `TypedModule` stops cloning module facts |
| CP-44 (LowerOutput) | CP-33 (compute CompileOutput fields once) | LowerOutput bundles the computed fields at source |
| CP-46 (structured LowerError) | CP-14 (source spans on lowerer errors) | Structured enum with spans covers the span requirement |
| CP-47 (RuntimeBindings) | Part of CP-40 (ExternRegistry) | CP-40 creates `ExternId`; full subsumption only lands once `RuntimeBindings` binds by ID instead of string lookup |
| CP-51 (NodeOrigin) | Part of CP-25 (pattern instance backrefs) | NodeOrigin covers the span-tracing aspect of pattern instances |

### Dependencies

**Foundation tier** (land first — everything depends on these):
- CP-36 (Verdict<T>) — unified result type for all stages
- CP-48 (unified Diagnostic type) — shared error structure with span + context + help

**Warm-up tier** (S-sized, independent, can land in parallel):
- CP-1, CP-13, CP-16, CP-28, CP-29, CP-31, CP-37, CP-38, CP-41, CP-42, CP-45

**Critical path**: WS-1 → WS-2 → WS-3 (silent failures → strict verification → parity)

**Diagnostic path** (parallel with critical path, accelerates everything):
- CP-48 (Diagnostic type) → CP-49 (TypeError spans) → CP-50 (help text) → CP-51 (NodeOrigin)
- CP-48 + CP-46 together (both reshape error types)
- CP-51 enables span-rich errors in verify/derive/emit

WS-9 items interleave: CP-41 pairs with CP-8. CP-42 pairs with CP-24. CP-40
is independently high-value and unlocks CP-47.

WS-10 depends on CP-36 (Verdict) and CP-40 (ExternRegistry for CP-47).
CP-43 and CP-45 are independent. CP-44 pairs with CP-43. CP-46 pairs with CP-48.

WS-11 (obligation boundary) can start anytime — CP-52 and CP-54 are independent
of other workstreams. CP-53 (response contracts) benefits from CP-18 (defer
transport expansion) but can land before it. CP-55 (provenance tracking) is
independent infrastructure. CP-56 depends on CP-34.

**Recommended start**: CP-36 + CP-48 + CP-58 (foundation: Verdict + Diagnostic +
contracts crate) first. Then S-sized warm-ups: CP-1 + CP-13 + CP-16 + CP-28 +
CP-29 + CP-31 + CP-37 + CP-38 + CP-41 + CP-42 + CP-45 + CP-52 + CP-54. Then
CP-2 through CP-5 (close silent failures). Then CP-49 + CP-50 + CP-51
(diagnostic quality, parallel with WS-1). Then CP-6 + CP-7 → CP-8 + CP-41 →
CP-23 (verification as type gate). Then CP-40 → CP-47 (ExternRegistry →
RuntimeBindings). Then CP-43 + CP-44 + CP-46 (interface reshape). Then CP-53 +
CP-55 (response contracts + provenance — higher effort, high impact). CP-57
(Vfs/Ingest) can land anytime — independent of the critical path.

**Highest-leverage single change**: `VerifiedDag<LoweredOp>` (CP-23) required
for both Resolve and Emit — then work backwards eliminating every reason
verification currently can't be strict (param_source CP-6, implicit defaults
CP-7/CP-20, silent drops CP-2/CP-3, unresolveds CP-1/CP-19).

**Measurable progress**: CP-55 (obligation provenance) establishes a ratchet —
count of `CompilerGap` obligations tracked per testgen run. Each pipeline
improvement should reduce this count. CI can enforce it never increases.
