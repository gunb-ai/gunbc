# Compiler and Runtime Invariants

This document governs the engineering invariants for the entire
codebase: the v1 Rust compiler (`src/v1/`), the v2 self-hosted
compiler (`src/v2/`), and the DSL source (`dsl/`).

## Performance Invariant

Performance is a correctness property for this repo, not a cleanup pass
for later. For every exposed interface, reusable helper, and hot path,
we should know the worst-case time and space bound before we commit to
the design.

The standard is not "fast enough on today's inputs." The standard is
"the asymptotic behavior is understood, intentional, and appropriate for
the role this code plays." Accidental quadratic behavior, repeated full
rescans, hidden reparsing, and large incidental clones are design bugs.

**The rule:** choose the data structure and algorithm that satisfy the
required bound up front. Complexity is part of the interface contract,
especially for APIs that may be called inside larger traversals.

**The test:** if you cannot state the upper bound for a non-trivial
algorithm or interface, the design is incomplete. If a call pattern
turns one scan into `N` scans, or one allocation into `N` large clones,
assume the implementation is wrong until proven otherwise.

**The fix:** write down the dominant operations, then implement to the
target bound directly. Prefer one-time indexing over repeated lookup,
single-pass structural walks over nested rescans, and data ownership
that avoids whole-structure cloning in loops.

## Sustainability Invariants

The governing metric for this codebase is **cost of change**: when the
language grows by one type, one expression form, or one transport, how
many files need editing? The sustainable compiler is one where that
number is 1. Every invariant below serves that goal.

Active liabilities and their measured costs are tracked in the
**Open Debt** section at the bottom of this file.

The invariant headings in this document are also the canonical theme
labels for ratchets, review feedback, and queue planning. A review queue
branch must declare exactly one primary theme from this list and stop
before taking a second review item from a different theme, so CI
failures stay attributable to a single ratchet. Review queue branches
must also keep each commit strictly scoped to that invariant fix: no
unrelated helper cleanup, dead-code removal, or opportunistic
refactoring unless it is directly required for the fix to compile and
pass tests.

### No duplicate representations

Every fact should be encoded in exactly one place. When two structures
represent the same information, one gets updated and the other doesn't.
The stale copy produces silently wrong behavior instead of failing.

**The test:** if changing a fact requires editing two files, one of them
is a derived copy that should be deleted or computed.

**The fix:** delete the derived representation and read from the source.
If the source isn't accessible, make it accessible — don't cache a copy
that can go stale. (See POSTMORTEM FC-7: `PortMultiplicity` duplicated
`Cardinality`, ResourceHandle had 3 fields in DSL but 4 in Rust.)

### No case enumeration for open sets

When behavior varies by type, variant, or category, prefer a single
algorithm that walks the structure over a match/list that enumerates
known cases. Enumerated lists rot: every new case requires updating
every list, and the compiler won't tell you which lists you missed.

**The test:** if adding a new type/variant requires editing a match arm
somewhere other than the type definition itself, the code has an
enumeration that should be replaced with a structural walk.

Matching on a closed enum (`WrapperKind::List | Set | Optional | ...`)
is fine — adding a variant is a compiler error. The problem is
open-ended lists keyed by strings, type names, or error message
substrings. (See POSTMORTEM FC-7: `mock_element_expr` was a 100+ line
match on type name strings.)

### No fallbacks that fabricate

Every code path either succeeds fully or fails with a clear error.
No silent degradation: no `.ok()` that swallows errors, no `continue`
that silently drops work, no fallback defaults that produce
valid-looking but wrong output. If a function cannot complete its job,
it must return `Err`.

Fabrication fallbacks are the mechanism by which duplicate
representations and missed enumerations become invisible. They convert
hard failures into silent wrong behavior. (See POSTMORTEM FC-7:
`scalar_witness_for_base` fabricated `Str("<Type>")` instead of
returning `None`.)

Sample: ownership should not compile to
`Rc::try_unwrap(x).unwrap_or_else(|rc| (*rc).clone())`. Either the
compiler proves a single semantic consumer and emits the move, or it
surfaces that the proof is missing. The clone branch is a fallback,
even if it preserves correctness.

### Heuristics indicate lost structure

Heuristics are a code smell in compiler and runtime logic. String
matching, score-based classification, best-effort guessing, "close
enough" defaults, and inference from naming conventions usually mean
the pipeline has already thrown away information that should have been
structural.

**The principle:** do not tune the heuristic first. Trace the pipeline
upstream until you find where the needed fact stopped being explicit,
then restore that structure as close to the source as practical.

**The test:** if a code path has to guess from strings, partial shapes,
error text, or naming patterns, the real bug is upstream information
loss. The preferred fix is to carry the missing fact in the type/IR/API
boundary instead of improving the guess.

**The fix:** push structure earlier in the pipeline so the downstream
stage can make an exact decision. If the local change cannot safely
repair the upstream contract yet, fail clearly or record a follow-up
task naming where the information degraded and what explicit structure
should replace the heuristic.

Sample: on 2026-03-20 the emission-representation work was first framed
as five per-binding lattices: flow cardinality, value width, call
reachability, loop invariance, and build-reduce. Backends then mapped
each lattice position to constructs like bare move vs Rc, byte vs
String, inline vs stacker. That was cleaner than ad hoc guesses, but in
practice it still risked becoming a heuristic matrix: pick a bucket,
then let the backend widen to a fallback-shaped implementation. The
stronger version kept only direct behavioral facts plus forcing
witnesses: consumed/read/projected edges, semantic consumer count,
escape/materialization, loop invariance, value shape. "Shared" needs
the specific extra consume site. "Materialized" needs the specific
escape. If no witness exists, the compiler has lost structure and
should be fixed upstream instead of widening by default.

### No parallel implementations

When the same computation exists in two forms (e.g., an AST interpreter
AND a resolved DAG op), they will diverge as the language evolves.
Every new expression form must be implemented in both, and the one that
lags will be masked by a fallback (see above).

**The test:** if a code path exists only to provide a result that
another code path also produces, one of them should be deleted.

### Explicit boundary contracts

Each stage of the pipeline (parse → typecheck → lower → resolve →
execute) passes a complex IR type to the next stage. The receiving
stage's preconditions must be structural — encoded in the type of the
boundary, not checked by a validation pass after the fact.

**The principle:** make illegal states unrepresentable. When a
downstream stage needs a guarantee (e.g., "all type references are
resolved"), the upstream stage must produce an output type that
*cannot* represent the unresolved case. The compiler enforces the
contract; no runtime validation walk is needed.

**The test:** if you find yourself wanting to add a validation pass
at a boundary, instead refactor the upstream stage's output type so
the invalid state is impossible to construct.

Examples (current state and target):
- After lowering (done): transport nodes are a distinct `LoweredOp::Transport`
  variant with required `ServiceCallMetadata` and `TransportObligation`.
  Transport obligations are structurally excluded from `LoweredOp::Callable`.
- After lowering (target): ports embed `ResolvedType` instead of `TypeId(String)`.
  `ResolvedType` is defined in `gunbc-ir` but not yet wired into ports;
  the migration is additive (`resolved_type` alongside `type_id`).
- After typecheck (target): the output type embeds resolved type structure,
  not a string TypeId that might not resolve.
- After resolve: the output DAG is parameterized by a trait that
  requires `Executable`, so non-executable nodes are unrepresentable.

When a boundary today uses a type that *can* represent invalid states,
that is the root cause — not the absence of a validation function.
Every fabrication fallback in FC-7 existed because the producing
stage's output type was too permissive, and the consuming stage
compensated with a fallback instead of failing.

A boundary fact table is only valid when both of these hold:

1. Every entry is an exact derivation from upstream structure. If the
   table collapses distinct bindings, guesses a classification, or drops
   witnesses needed downstream, it is a lossy representation and is
   already an invariant violation.
2. A downstream stage actually consumes the table as the authority for a
   decision. If no consumer reads it, the table is speculative metadata
   or a parallel representation waiting to diverge.

Unused or lossy fact tables are not harmless scaffolding. Unused tables
violate "No parallel implementations" / "Single-authority metadata."
Lossy tables violate "Explicit boundary contracts" / "Heuristics
indicate lost structure." The default action is to delete the table
until a concrete consumer exists, or tighten it until the missing
distinctions are structurally preserved.

### Single-authority metadata

The compiler should provide all metadata (tool definitions, output
paths, type registries) through its own output types (`CompileOutput`,
`InferredEntrypoint`, etc.), not through runtime callbacks, string
conventions, or hardcoded lists. Each piece of metadata should have
exactly one producer.

## Engineering Standards

These serve sustainability indirectly by reducing the blast radius of
changes:

- **Clear interfaces.** Every public module should have a small,
  well-defined API surface. Prefer returning values over mutating
  shared state.

- **Pure core logic.** Deterministic functions from inputs to outputs.
  Side effects (filesystem, network, process spawning) belong at the
  edges, not in the middle of computation.

- **Documented I/O boundaries.** Any function that performs I/O must
  document that fact in its signature or doc comment. Callers should
  never be surprised by hidden I/O.

## Testing Invariants

- **Behavioral only.** Tests assert observable behavior — outputs given
  inputs, error messages, public API contracts. Never assert internal
  implementation details like which private functions were called, what
  order internal steps execute in, or how many times an internal helper
  runs.

- **Hermetic unit tests only.** Tests must not touch the filesystem,
  network, or environment. All external dependencies are injected or
  mocked. A test that passes on one machine must pass on every machine.
  Corpus/integration tests (e.g., `daglang-syntax/tests/item_coverage.rs`)
  that walk the `dsl/` source tree are a recognized exception — they
  live in `tests/` directories and are clearly labeled as non-hermetic.

- **No tautological tests.** A test that mirrors the implementation —
  restating the production code in test form — proves nothing. Tests
  must encode an independent specification of *what* the code should do,
  not *how* it does it. If deleting the test body and replacing it with
  a copy of the production code would still pass, the test is
  tautological.

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
`src/v1/09_execute/exec/src/execute/tests.rs`.

### Tier 3 — Full Real (integration)

All nodes execute for real against live services. Only viable in
controlled environments with sandboxed credentials (CI runners with
scoped tokens, disposable cloud resources). Proves end-to-end behavior
including HTTP transport and cloud API interactions. Not yet implemented;
requires credential injection infrastructure.

## Branch Review Findings

### 2026-03-21 — `v2-compiler-convergence`

- `src/v2/04a_normalize.dag`: `EmitGraph.func_facts.binding_facts` is
  keyed only by raw variable name, so shadowed bindings collapse into a
  single entry. That violates the normalize boundary contract: the type
  claims per-binding facts but cannot represent two distinct bindings
  named `x` in different scopes.

- `src/v2/04a_normalize.dag`: `classify_arms` hardcodes `Read` for every
  match-arm body instead of threading the parent edge context through the
  arm. A value consumed through a `match` therefore reaches
  `EmitGraph.func_facts` with the wrong usage classification.

- `src/v2/04a_normalize.dag`: `classify_pattern_bindings` records
  `FieldBinding.field_name` instead of the variable introduced by
  `FieldBinding.binding`, and ignores non-variant bind patterns. Match
  locals therefore receive missing or incorrect `BindingFacts`.

- `src/v2/04a_normalize.dag`: `classify_func` returns `InteriorFunc` for
  every non-recursive function, making `LeafFunc` unreachable. If the
  call-graph pass is not implemented yet, the boundary needs an explicit
  unknown state rather than fabricating a false classification.

- `src/v2/04a_normalize.dag`: `EmitGraph.func_facts`,
  `EmitGraph.enum_facts`, and `EmitGraph.field_facts` are currently not
  consumed by the Rust, Python, or Go emitters, which immediately peel
  back to `typed.graph`. That makes the new normalize boundary
  speculative metadata instead of authoritative boundary structure. Per
  the invariants above, these fact maps should either be deleted until
  emit uses them or wired through as the single source of truth.

### 2026-03-21 — transport/expr dissolution review

Fixed:

| # | Violation | Fix |
|---|-----------|-----|
| TD-1 | `LitString` typo in `auth_properties` and `find_property_string` (variant does not exist) | Fixed to `LitStr` (3 sites in `00_core.dag`). Latent — no test breakage because `auth_properties` never called in current test paths. |
| TD-4 | Dead `parent_enum == "Expr"` in `05_emit_rust.dag` variant construction | 7 lines removed. |
| TD-5 | Dead `classify_transport_kind()` in `05_emit.dag`, imported but never called | Function deleted, imports removed from Go/Python emitters. |
| TD-6 | Stale DESIGN.md Layer 2 documented old `TransportBinding` sum type | Updated to Node-based transport model. |

---

## Open Debt

### Structural

| # | Severity | Invariant | Description |
|---|----------|-----------|-------------|
| TD-2 | HIGH | No case enumeration / No parallel implementations | String-keyed dispatch `transport.name == "rest"` across 3 emitters (21 sites). Adding a transport kind requires editing all 3. Fix: closed enum `TransportKind` or structural fact dispatch. |
| TD-3 | MEDIUM | No duplicate representations / Single-authority metadata | Hardcoded `config_names` list in `transport_headers()` (`00_core.dag`). Same field names in constructors, accessors, and filter — triple representation. |
| F2 | MEDIUM | No case enumeration for open sets | `ItemInfo.kind` is String (`"fn"`, `"func"`, `"other"`) in `04_reconcile.dag`. Should be closed enum `ItemKind`. |
| F6 | MEDIUM | Single-authority metadata | `05_emit_rust.dag` re-discovers structural facts through string heuristic lists (`known_opt_fields`, `types_with_value_field`, etc.). Reconciler already knows these. |
| F7 | MEDIUM | No case enumeration for open sets | `emit_typed_method_call` in `05_emit_rust.dag` is a growing `if method == ...` ladder for special lowerings. |
| TD-7 | MEDIUM | No fallbacks that fabricate | 5 `LitNull` fabrication fallbacks in `emit_typed_call` (`05_emit_rust.dag:1754,1755,1763,1764,1789`). Sentinel when arguments missing. |

### Cleanup

| # | Severity | Description |
|---|----------|-------------|
| F5 | LOW | `infer → reconcile` rename lacks documented contract justification. |
| SG-9 | LOW | .dag workarounds for force_clone (TokPos extraction, branch-aware use counting). Revert after verification at scale — may be redundant after R9. |
