# gunbc

A DSL-first workflow compiler where everything is a DAG.

```bash
git clone <repo> && cd gunbc && make install
```

## Philosophy

Software drifts when intent is inconsistent. Every layer of this system
enforces a single invariant. If a layer's invariant holds, the system is
correct by construction.

## The Invariants

### 1. Domain lives in the DSL, not in Rust

`dsl/` is the source of truth for types, data, service contracts, and
workflows. If something can be expressed in `.dag` files, it must not be
hardcoded in Rust.

Rust provides the engine. The DSL provides the domain.

```
dsl/std/errors.dag              "What is an HTTP error shape?"
dsl/extdeps/cloud/gcp/gcp.dag   "What is GCP?" — real OAuth2 scopes, real endpoints
dsl/tools/gist.dag               "How do we upload a gist?" — composes services
```

The compiler pipeline (`src/02_pipeline/` through `src/07_emit/`) transforms
`.dag` -> executable DAG IR.
The engine (`src/`) executes it. Neither knows what the domain is.

### 2. World I/O is structural, not annotated

A DAG node either does I/O or it doesn't, and you can tell by looking at
the graph.

```
[PrepareOp]  ->  [TransportOps::Execute]  ->  [ParseOp]
   (pure)          (the only I/O node)        (pure)
```

`src/08_materialize/transport/` is the **only** crate that performs direct I/O.
All other crates build `TransportRequest` values (pure) and consume
`TransportResponse` values (pure). Dry-run replaces transport nodes with
mocks. Pure nodes always run.

### 3. An extdeps module implements a specification, not an abstraction of one

Every `dsl/extdeps/` module models a real external system grounded in its
actual API documentation — real field names, real endpoints, real versions.
If you can't link to a spec, you're inventing one.

See `dsl/extdeps/extdeps.md` for the full fidelity invariant and grading.

### 4. Each compiler phase is a pure function from input to output

```
source text -> syntax -> resolve -> typecheck -> lower -> derive -> emit
```

No phase mutates its input. No phase performs I/O except filesystem reads
during import resolution. The compiler never executes the DAGs it produces.

### 5. Composition through layers, not abstraction

```
Layer 0  std/errors.dag             "What is an HTTP error?"
Layer 1  extdeps/cloud/cloud.dag    "What is a cloud provider?"
Layer 2  extdeps/cloud/gcp/gcp.dag  "What is GCP?"
Layer 3  extdeps/cloud/gcp/secret_manager.dag  "What is GCP Secret Manager?"
Layer 4  extdeps/secrets/secrets.dag  "What is a secret?" (universal)
Layer 5  tools/gist.dag             "Upload a gist" (composes everything)
```

Each layer only knows about layers below it. Adding a new external
dependency means instantiating existing vocabulary, not inventing new
abstractions.

### 6. Resolution maps DSL constructs to runtime — nothing more

The runtime materialization layer (`src/08_materialize/`) is the wiring layer.
It does not contain domain logic (that's DSL) or engine logic (that's the
pipeline, IR, and executor crates). Every `extern func` backed by Rust is
ratcheted and must be justified.

### 7. Every expression lowers to structural DAG nodes or the compilation fails

The lowerer must produce a graph where every node has explicit typed ports
and every data dependency is an edge. No node may carry an opaque AST
fragment to be evaluated by a runtime interpreter. If the lowerer cannot
represent an expression structurally, the compilation must fail with a
diagnostic — not silently emit an interpreter-backed fallback node.

The `lower_expr` function enforces this: it returns `Result` (not
`Option`) and its match on `Expr` variants is exhaustive with no wildcard.
A new `Expr` variant added to the parser without a corresponding lowering
arm is a Rust compile error.

### 8. Correctness by construction, not by validation

Invariants are enforced by the type system and by the structure of the
compiler's data types — not by validation passes that scan for violations
after the fact. If a property must hold, the API must make violations
unrepresentable. Validation is a diagnostic tool for finding bugs in the
compiler itself, not a substitute for structural guarantees.

## Structure

```
dsl/              Domain: .dag source files, types, data, workflows
src/
  00_foundation/  Shared IR and foundational crates
  01_surfaces/    CLIs, codegen, workflow-facing entrypoints
  02_pipeline/    Driver/orchestrator: prepare -> run staged compile
  03_source/      Source discovery, parse, module graph
  04_semantics/   Typechecking and semantic validation
  05_graph/       Lowering to GraphIR
  06_artifacts/   Derived manifests and obligations
  07_emit/        Code emission
  08_materialize/ Runtime wiring, resolver, primitives, blob, transport
  09_execute/     DAG executor
  10_test/        Test support and generated tests
```

## Testing

```bash
cargo test
cargo clippy --all-targets -- -D warnings
```
