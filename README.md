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

The compiler pipeline (`src/0_pipeline/` through `src/5_emit/`) transforms
`.dag` -> executable DAG IR.
The engine (`src/`) executes it. Neither knows what the domain is.

### 2. World I/O is structural, not annotated

A DAG node either does I/O or it doesn't, and you can tell by looking at
the graph.

```
[PrepareOp]  ->  [TransportOps::Execute]  ->  [ParseOp]
   (pure)          (the only I/O node)        (pure)
```

`src/6_materialize/transport/` is the **only** crate that performs direct I/O.
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

The runtime materialization layer (`src/6_materialize/`) is the wiring layer.
It does not contain domain logic (that's DSL) or engine logic (that's the
pipeline, IR, and executor crates). Every `extern func` backed by Rust is
ratcheted and must be justified.

## Structure

```
dsl/              Domain: .dag source files, types, data, workflows
src/
  0_foundation/   Shared IR and foundational crates
  0_pipeline/     Driver/orchestrator: prepare -> run staged compile
  1_source/       Source discovery, parse, module graph
  2_semantics/    Typechecking and semantic validation
  3_graph/        Lowering to GraphIR
  4_artifacts/    Derived manifests and obligations
  5_emit/         Code emission
  6_materialize/  Runtime wiring, resolver, primitives, blob, transport
  7_execute/      DAG executor
  8_surfaces/     CLIs, codegen, workflow-facing entrypoints
  9_test/         Test support and generated tests
```

## Testing

```bash
cargo test
cargo clippy --all-targets -- -D warnings
```
