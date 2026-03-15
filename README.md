# gunbc

A DSL-first workflow compiler where everything is a DAG.

```bash
git clone <repo> && cd gunbc && make install
```

## Philosophy

Software drifts when intent is inconsistent. Every layer of this system
enforces a single invariant. If a layer's invariant holds, the system is
correct by construction.

## Two products, one pipeline

The system produces two things from `.dag` source files:

1. **Compiled binaries** — the compiler emits target-language source code
   (Rust, Python, etc.); a host compiler builds it. The compiler's job
   ends at file emission.

2. **Interpreted execution** — the lowered IR is executed directly by an
   interpreter. No code generation.

Both share the compiler pipeline (parse -> typecheck -> lower). They
diverge at the artifact boundary: the compiler produces
`VerifiedDag<LoweredOp>`, and each branch consumes it differently.

```
.dag source
    |
COMPILER: parse -> resolve imports -> typecheck -> lower -> derive -> emit
    |
VerifiedDag<LoweredOp>  <- THE ARTIFACT
    |                         |
Branch A: emit -> cargo    Branch B: interpret directly
```

The compiler and interpreter share the artifact format (`gunbc-ir` types)
and the evaluator (`daglang-eval`). They share nothing else. The compiler
does not know about transport. The interpreter does not know about parsing
or typechecking.

## v2: Self-hosted compiler

The v2 compiler is written in the DSL itself, living at `src/v2/`. The
compiler is a pure transform — `.dag files -> generated files` — with no
embedded interpreter. The v1 Rust pipeline serves as bootstrap host.

```
src/v2/
  00_core.dag        Type system: Token, AST, Expr, TypeExpr
  01_tokenize.dag    String -> List<Token>
  02_parse.dag       List<Token> -> Module (AST)
  03_resolve.dag     List<Module> -> ModuleGraph
  04_typecheck.dag   ModuleGraph -> TypedGraph (types resolved to TypeExpr)
  05_emit.dag        TypedGraph -> List<TextFile>
  06_pipeline.dag    Orchestrator: discover -> compile -> write
```

Start with `src/v2/WORKBOARD.md` for current status, work queue, and doc roles.
Use `src/v2/DESIGN.md` for target architecture and `src/v2/POSTMORTEM.md` for
the exhaustive audit ledger.

The key design change from v1: **types are structural values, not string
references.** A type like `List<Span>` is stored as
`Container { kind: List, element: Product { name: "Span", fields: [...] } }`,
not as a `TypeId("List<Span>")` string requiring a registry lookup. No
registry, no deferred resolution, no stale copies.

See `src/v2/DESIGN.md` for the full design and target type models.

## The Invariants

Detailed compiler/runtime invariants and testing rules live in
`INVARIANTS.md`. The list below is the repo-level summary.

### 1. Performance is a contract

Worst-case runtime and space usage are part of correctness. For any
non-trivial interface or hot path, we should know the upper bound ahead
of time and choose the data structures to match it. Accidental quadratic
behavior, repeated rescans, reparsing, or large incidental clones are
repo-level bugs, not later optimization work.

### 2. Domain lives in the DSL, not in Rust

`dsl/` is the source of truth for types, data, service contracts, and
workflows. If something can be expressed in `.dag` files, it must not be
hardcoded in Rust.

Rust provides the engine. The DSL provides the domain.

```
dsl/std/errors.dag              "What is an HTTP error shape?"
dsl/extdeps/cloud/gcp/gcp.dag   "What is GCP?" — real OAuth2 scopes, real endpoints
dsl/gunbc/tools/gist.dag      "How do we upload a gist?" — composes services
```

The compiler pipeline (`src/02_pipeline/` through `src/07_emit/`) transforms
`.dag` source into `VerifiedDag<LoweredOp>`. The emitter generates
target-language source code. The interpreter executes the IR directly.
Neither knows what the domain is.

### 3. World I/O is structural, not annotated

A DAG node either does I/O or it doesn't, and you can tell by looking at
the graph.

```
[PrepareOp]  ->  [TransportOps::Execute]  ->  [ParseOp]
   (pure)          (the only I/O node)        (pure)
```

`gunbc-lib-transport` is the **only** crate that performs direct I/O.
All other crates build `TransportRequest` values (pure) and consume
`TransportResponse` values (pure). Dry-run replaces transport nodes with
mocks. Pure nodes always run.

### 4. An extdeps module implements a specification, not an abstraction of one

Every `dsl/extdeps/` module models a real external system grounded in its
actual API documentation — real field names, real endpoints, real versions.
If you can't link to a spec, you're inventing one.

See `dsl/extdeps/extdeps.md` for the full fidelity invariant and grading.

### 5. Each compiler phase is a pure function from input to output

```
source text -> syntax -> resolve -> typecheck -> lower -> derive -> emit
```

No phase mutates its input. No phase performs I/O except filesystem reads
during import resolution. The compiler never executes the DAGs it produces.

### 6. Composition through layers, not abstraction

```
Layer 0  std/errors.dag             "What is an HTTP error?"
Layer 1  extdeps/cloud/cloud.dag    "What is a cloud provider?"
Layer 2  extdeps/cloud/gcp/gcp.dag  "What is GCP?"
Layer 3  extdeps/cloud/gcp/secret_manager.dag  "What is GCP Secret Manager?"
Layer 4  extdeps/secrets/secrets.dag  "What is a secret?" (universal)
Layer 5  gunbc/tools/gist.dag    "Upload a gist" (composes everything)
```

Each layer only knows about layers below it. Adding a new external
dependency means instantiating existing vocabulary, not inventing new
abstractions.

### 7. The interpreter maps IR to execution — nothing more

The interpreter (`gunbc-interp`) dispatches `LoweredOp` nodes: pure ops
go to the evaluator, I/O ops go to the transport layer. It does not
contain domain logic (that's DSL) or compiler logic (that's the pipeline).
Every `extern func` backed by Rust is ratcheted and must be justified.

### 8. Every expression lowers to structural DAG nodes or the compilation fails

The lowerer must produce a graph where every node has explicit typed ports
and every data dependency is an edge. No node may carry an opaque AST
fragment to be evaluated by a runtime interpreter. If the lowerer cannot
represent an expression structurally, the compilation must fail with a
diagnostic — not silently emit an interpreter-backed fallback node.

The `lower_expr` function enforces this: it returns `Result` (not
`Option`) and its match on `Expr` variants is exhaustive with no wildcard.
A new `Expr` variant added to the parser without a corresponding lowering
arm is a Rust compile error.

### 9. Correctness by construction, not by validation

Invariants are enforced by the type system and by the structure of the
compiler's data types — not by validation passes that scan for violations
after the fact. If a property must hold, the API must make violations
unrepresentable. Validation is a diagnostic tool for finding bugs in the
compiler itself, not a substitute for structural guarantees.

## Structure

```
dsl/                    Domain: .dag source files, types, data, workflows
  std/                  Shared vocabulary (errors, types, resources, ...)
  extdeps/              External system models (GCP, GitHub, shell, git, ...)
  config/               Repo-specific policy (clippy, test policy, ...)
  tools/                Tool definitions (bootstrap, build, codegen, ...)

src/
  v1/                   v1 compiler and runtime (Rust)
    00_foundation/        Shared IR and foundational crates
      infra/                gunbc-infra: hashing, resource IDs, manifests (leaf)
      ir/                   gunbc-ir: DAG, Node, Port, Edge, Value types
      daglang-contract/     Verdict, Diagnostic, spans (leaf)
      delegate-macros/      Proc macros for delegation
    01_surfaces/          CLIs, codegen, workflow-facing entrypoints
      cli/                  Main CLI entrypoint
      codegen/              Code generation, testgen, DSL-generated binaries
      daglang-cli/          DSL compiler CLI
      workflow/             Workflow runner
    02_pipeline/          Compiler orchestration
      daglang-driver/       Pipeline driver: discover -> compile staged
    03_source/            Source discovery, parse, module graph
      daglang-syntax/       Tokenizer and parser (.dag -> AST)
      daglang-resolve/      Import resolution, module graph
    04_semantics/         Typechecking and semantic validation
      daglang-typecheck/    Type resolution, validation
    05_graph/             Lowering and evaluation
      daglang-lower/        Typed AST -> VerifiedDag<LoweredOp>
      daglang-eval/         Expression evaluator (shared by compiler + interpreter)
    06_artifacts/         Derived metadata
      daglang-derive/       Structural graph walks, callable properties
    07_emit/              Code emission
      daglang-emit/         Generate target-language source from IR
    08_materialize/       Runtime wiring and I/O
      resolve/              gunbc-resolve: LoweredOp -> DynOp resolution
      interp/               gunbc-interp: interpreter dispatch
      transport/            gunbc-lib-transport: the I/O boundary
      blob/                 gunbc-lib-blob: content-addressed storage
    09_execute/           DAG executor
      exec/                 gunbc-exec: topological scheduling, dry-run
    10_test/              Test support and generated tests
      test/                 Test utilities
      generated-tests/      Auto-generated DAG integration tests
      testgen-registry/     Test generation registry
  v2/                   Self-hosted compiler (.dag source)
```

## Testing

```bash
# All hand-written tests
cargo test --workspace --exclude gunbc-dag-tests

# Auto-generated DAG integration tests
cargo test -p gunbc-dag-tests

# Lint
cargo clippy --all-targets -- -D warnings
```
