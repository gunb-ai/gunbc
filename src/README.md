# src/ — Compiler and Runtime

All Rust crates live here. The DSL compiler pipeline, execution engine,
IR, and runtime operations.

## Coding Standards

Adapted from Google C++ style for Rust:

- **Clear interfaces.** Every public module should have a small, well-defined
  API surface. Prefer returning values over mutating shared state.

- **Pure libraries and helper functions.** Core logic should be pure —
  deterministic functions from inputs to outputs. Side effects (filesystem,
  network, process spawning) belong at the edges, not in the middle of
  computation.

- **I/O documentation at interface boundaries.** Any function that performs I/O
  (reads files, spawns processes, makes network calls) must document that fact
  in its signature or doc comment. Callers should never be surprised by hidden
  I/O.

- **No backdoors.** The compiler should provide all metadata (tool definitions,
  output paths, type registries) through its own output types (`CompileOutput`,
  `InferredEntrypoint`, etc.), not through runtime `extern func` callbacks.
  The `extern func` feature has been eliminated.

- **No hacks or fallbacks.** Every code path either succeeds fully or fails
  with a clear error. No silent degradation: no lossy recovery that discards
  input, no `.ok()` that swallows errors on fallible operations, no `continue`
  that silently drops work, no fallback defaults that produce valid-looking but
  wrong output. If a function cannot complete its job, it must return `Err` —
  not an empty value, not `Value::Skipped`, not a quietly truncated result.
  Caching is the sole exception (cache miss on error is acceptable).

- **Single source of truth (no degenerate representations).** Every fact in the
  system should be encoded in exactly one place. When two structures represent
  the same information, one inevitably gets updated while the other doesn't —
  and the stale copy produces silently wrong behavior instead of failing.

  The failure mode is always the same: information is computed or declared in
  one place, a second representation is derived from it, and then downstream
  code reads the derived copy instead of the source. When the source changes,
  the derived copy is stale, and instead of erroring, the system fabricates a
  plausible result from the stale data.

  Examples from FC-7 (POSTMORTEM.md):
  - `PortMultiplicity` duplicated what `Cardinality.is_list()` already encoded.
    When cardinality was set to ZERO_OR_MORE but multiplicity stayed Singular,
    the executor silently used the wrong merge strategy.
  - `base_type()` extracted a type name from Identity nodes but didn't recurse
    through Wrap/Brand nodes. The type DAG encoded the full structure, but
    `base_type` read a partial view, producing `None` and triggering a
    `Str("mock")` fabrication fallback.
  - ResourceHandle had 3 fields in the DSL but 4 in the Rust impl. Mocks were
    generated from the DSL definition but validated against the Rust shape.
  - `resolve_type_checked()` routed through a parser that rejected names the
    registry accepted. The registry was the source of truth, but the parser
    gate made some registered types unreachable.

  The fix is always the same: delete the derived representation and read from
  the source. If the source isn't accessible, make it accessible — don't cache
  a copy that can go stale. When deletion isn't possible (e.g., serialized
  formats), the derived value must be computed, not declared separately.

- **Structural rules over case enumeration.** When handling varies by type,
  variant, or category, prefer a single algorithm that walks the structure over
  a match/list that enumerates known cases. Enumerated lists rot: every new
  case requires updating every list, and the compiler won't tell you which
  lists you missed.

  The test: if adding a new type/variant requires editing a match arm somewhere
  other than the type definition itself, the code has an enumeration that should
  be replaced with a structural walk.

  Examples from FC-7:
  - `mock_element_expr` was a 100+ line match on type name strings that had to
    be manually extended for every new DSL type. Replaced by `typed_witness_value`
    which walks the type DAG structurally — new types get correct witnesses
    automatically.
  - `scalar_witness_for_base` enumerated known primitives and fabricated
    `Str("<TypeName>")` for everything else. Replaced by returning `None` for
    unknown bases and letting `product_witness` handle structured types via
    the registry.
  - The C3 string-matching heuristic (`err_msg.contains("unbound variable") ||
    err_msg.contains("cannot access field") || ...`) was a growing list of
    error patterns. Each new evaluator failure mode required adding another
    string check. Replaced by a structural rule: fn body evaluation is
    best-effort, all eval errors fall back to passthrough.

  Not all matches are bad — matching on a closed enum (`WrapperKind::List |
  Set | Optional | ...`) is fine because adding a variant is a compiler error.
  The problem is open-ended lists keyed by strings, type names, or error
  message substrings.

## Testing Invariants

- **Behavioral only.** Tests assert observable behavior — outputs given inputs,
  error messages, public API contracts. Never assert internal implementation
  details like which private functions were called, what order internal steps
  execute in, or how many times an internal helper runs.

- **Hermetic unit tests only.** Tests must not touch the filesystem, network,
  or environment. All external dependencies are injected or mocked. A test
  that passes on one machine must pass on every machine. Corpus/integration
  tests (e.g., `daglang-syntax/tests/item_coverage.rs`) that walk the `dsl/`
  source tree are a recognized exception — they live in `tests/` directories
  and are clearly labeled as non-hermetic.

- **No tautological tests.** A test that mirrors the implementation — restating
  the production code in test form — proves nothing. Tests must encode an
  independent specification of *what* the code should do, not *how* it does it.
  If deleting the test body and replacing it with a copy of the production code
  would still pass, the test is tautological.

## Tiered Test Execution (T11)

DAG execution tests use three tiers, each proving a different layer of
correctness. Every test explicitly chooses its tier via `ExecutionMode`.

### Tier 1 — DryRun (structure)

All transport, resource-environment, and tool nodes are intercepted with
explicit mocks (`ExecutionMode::DryRun(mocks)`). Pure nodes execute
normally. This tier proves DAG wiring, port cardinality, coercion, guard
evaluation, conditional branching, and topological ordering — without
performing any real I/O. The majority of existing tests operate at this
tier.

### Tier 2 — Selective Real (computation)

The DAG executes in `ExecutionMode::Real`, but the operations themselves
are limited to safe, hermetic effects: reading environment variables,
filesystem operations in temporary directories, timestamps, and
conditional logic. No external HTTP calls or cloud API interactions.
This tier proves that computation within the DAG produces correct
*values*, not just correct *shapes*.

Reference tests: `env_var_read_real_mode`,
`real_mode_executes_resource_environment_node` in
`src/09_execute/exec/src/execute/tests.rs`.

### Tier 3 — Full Real (integration)

All nodes execute for real against live services. Only viable in
controlled environments with sandboxed credentials (CI runners with
scoped tokens, disposable cloud resources). Proves end-to-end behavior
including HTTP transport and cloud API interactions. Not yet implemented;
requires credential injection infrastructure.
