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

Temporary heuristics must not become invisible debt. If a heuristic is
introduced as a staging move, the same change should record a shrinking
ratchet for it: a named debt entry, a bounded site count, or a test that
proves one heuristic family was removed. "Temporary" without a ratchet
means "permanent later."

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

New semantic boundaries must land end-to-end. A new normalize/pass/fact
layer is not accepted just because it computes plausible metadata; at
least one downstream consumer in the same change must read it as the
authority for a real compilation decision. Otherwise the layer is still
speculative metadata and should stay out of the pipeline until the
consumer exists.

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

- Deleted `src/v2/04a_normalize.dag` and removed the extra
  reconcile→normalize→emit boundary. The stage introduced unused and
  lossy fact tables (`func_facts`, `enum_facts`, `field_facts`) that
  were not consumed by any emitter, and some entries were already
  degraded (shadowed bindings collapsed by name, match-arm context lost,
  placeholder function classifications). Emit now consumes the existing
  reconcile boundary directly again until an exact, authoritative
  emitter-facing index is needed.

### 2026-03-21 — transport/expr dissolution review

Fixed:

| # | Violation | Fix |
|---|-----------|-----|
| TD-1 | `LitString` typo in `auth_properties` and `find_property_string` (variant does not exist) | Fixed to `LitStr` (3 sites in `00_core.dag`). Latent — no test breakage because `auth_properties` never called in current test paths. |
| TD-4 | Dead `parent_enum == "Expr"` in `05_emit_rust.dag` variant construction | 7 lines removed. |
| TD-5 | Dead `classify_transport_kind()` in `05_emit.dag`, imported but never called | Function deleted, imports removed from Go/Python emitters. |
| TD-6 | Stale DESIGN.md Layer 2 documented old `TransportBinding` sum type | Updated to Node-based transport model. |

### 2026-03-21 — semantic-boundary review

Classified as invariant violations:

- Rust emission still repairs semantics downstream instead of consuming a
  fully classified boundary: `emit_typed_field_access` branches on
  `.typed`, `.value`, `is_likely_optional_receiver(...)`, and
  `emit_typed_expr` conditionally appends `.map(Rc::new)` via
  `lookup_on_data_needs_rc_wrap(...)`. This violates "Heuristics
  indicate lost structure" / "Explicit boundary contracts."
- `lookup_in_scope` falls back to `lookup_func_sig(...).return_type` for
  function-as-value references. That fabricates a non-callable value from
  a callable binding and violates "Explicit boundary contracts" / "No
  fallbacks that fabricate."
- `node_type_equals` still contains permissive compatibility rules
  (`Dynamic` matches anything, plus same-name/same-connective/same-child-count
  fallback) that hide missing earlier normalization. This violates "No
  fallbacks that fabricate" / "Explicit boundary contracts."
- Reconcile downgrades semantic gaps to `Warning`
  (`access_error` / `inference_error`), and `compile_sources` gates only
  on `Error`, so emit still runs on known inference/access gaps. This is
  a warning-permissive boundary rather than a fail-closed one.

Not invariant violations by themselves:

- Roadmap/docs drift (`A7 full retirement`, `P1b done`, acceptance text
  that still names future work).
- Loose ratchets (`SELF_COMPILE_ERROR_RATCHET == 2700`) and unlanded
  StageMetrics/performance-contract work. These are backlog/test debt,
  not direct invariant violations until a concrete boundary or algorithm
  violates a stated rule.

---

## Open Debt

Three root causes account for ~50 individual sites. Fixing the root causes
eliminates the symptoms; fixing symptoms individually is whack-a-mole.

---

### Root Cause A: Reconcile→Emit Boundary is Information-Lossy

**Invariants violated:** Explicit boundary contracts, Heuristics indicate lost
structure, No fallbacks that fabricate.

**The problem:** Reconcile computes semantic facts (field access style, method
classification, function-vs-value, fold accumulator type, optional unwrap,
call→method bridging, Rc ownership) but does not attach them to output nodes.
Emit receives typed Nodes with `return_type` populated and must re-derive
everything else — producing 16+ compensation patterns including heuristic
scans, string-dispatch ladders, and global map lookups.

**Design decision required:** Define the reconcile→emit boundary contract.
Reconcile must attach per-expression classification so emit only translates.
Candidate approach: enrich ExprData variants with the facts reconcile already
computes (access style on ExprFieldAccess, intrinsic ID on ExprMethodCall,
callable-vs-value on ExprVar, accumulator type on fold calls).

This replaces the Multi-Walk Refactor Program P0–P3 items in ROADMAP.md —
the real motivation is boundary completeness, not performance. The perf win
is a side effect of not re-scanning.

| # | What reconcile computes | Where it's lost | How emit compensates |
|---|------------------------|-----------------|---------------------|
| A-1 | Field access style (StoredField / EnumAccessor / OptionalUnwrap) — `build_field_summaries_*` at `04_reconcile.dag:1070-1175` | Not attached to ExprFieldAccess nodes | `emit_typed_field_access` calls `lookup_emit_field_summary_in_scope` at codegen time (redundant); `is_likely_optional_receiver` scans all type_summaries; `is_optional_field_in_any_type` / `is_enum_accessor_in_any_type` do global sweeps (`05_emit_rust.dag:1576-1601`) |
| A-2 | Method classification (which intrinsic, return type shape) — `infer_method_call_type_node` at `04_reconcile.dag:1284-1340` | Only return type attached; no method-kind tag | Emit re-dispatches via `classify_intrinsic_method` (`05_emit.dag:195-212`) and `emit_typed_method_call` ladder (`05_emit_rust.dag:2091+`). 40+ string branches in reconcile, ~20 in emit. |
| A-3 | Call→MethodCall bridging — `04_reconcile.dag:1825-1841` | Bridge decision not recorded on node | Emit reverse-engineers via `rt_functions` / `rt_ref_map_functions` map lookups (`05_emit_rust.dag:2093-2103`) |
| A-4 | Function-as-value reference — `lookup_in_scope` fallback to `lookup_func_sig` at `04_reconcile.dag:751-754` | ExprVar node gets return type only; callable-vs-value distinction lost | Emit cannot distinguish function reference from local binding (SB-1). Fabricates value type from callable's return type. |
| A-5 | Fold accumulator type — computed at `04_reconcile.dag:1756-1788` | Not attached to MethodCall node | Emit has no access; falls back to Dynamic for unresolvable cases |
| A-6 | Rc-wrapping requirement — known per type during reconcile | Not per-expression; stored only in global `rc_types` map | Emit uses 4 fallback strategies: `is_rc_wrapped_scrutinee` (line 2151), `arms_need_rc_deref` (line 2162), `arms_need_option_rc_deref` (line 2216), `lookup_on_data_needs_rc_wrap` (line 1852 — extracts table name string from first arg) |
| A-7 | Variant→parent enum mapping — resolved during type resolution | Only available via global `vtoe` map, not per-expression | Emit builds module-local vtoe disambiguation (`05_emit_rust.dag:430-467`); `emit_var_ref` does fallback lookup (line 1508) |
| A-8 | Dynamic/error type propagation — `node_is_dynamic` at `04_reconcile.dag:900` | Error state encoded as `string_contains("<error:")` in type name | Emit replicates check at `05_emit_rust.dag:1473`; `node_type_equals` treats Dynamic as universally compatible (SB-2) |
| A-9 | Lambda parameter types — unresolved when collection type is Dynamic | Bound to `Dynamic` in `extend_scope_for_lambda` (`05_emit_rust.dag:1959`) | Auto-wrap disabled entirely (`let needs_wrap = false` at line 2445) because `is_already_optional` can't detect Optional inside Dynamic-typed lambdas |
| A-10 | Primitive/collection type identity — structurally known | Only available as type name strings | Emit hardcodes `"Int"`, `"Bool"`, `"Float"`, `"List"`, `"Map"`, `"Set"`, `"String"` in name-matching functions (`05_emit_rust.dag:1145-1150`, `882-908`, `1488-1494`) |

**Previously tracked as:** F6, F7, SB-1, SB-2

---

### Root Cause B: Closed Sets Dispatched as Strings

**Invariants violated:** No case enumeration for open sets, No parallel
implementations.

**The problem:** Several finite, known-at-compile-time sets are encoded as
strings and dispatched via `if x == "..."` ladders across multiple files.
Adding a value to any set requires editing every dispatch site — there is
no compiler-enforced exhaustiveness.

**Design decision required (methods only):** Are method/builtin intrinsics a
closed compiler-known set (→ enum) or structural DSL-defined facts the
compiler discovers? The language thesis says "smart facts + dumb compiler,"
so methods should eventually be data declarations in `.dag`. Pragmatically,
an `IntrinsicId` enum is the right intermediate step — it centralizes the
set and gives exhaustiveness checking. The enum definition becomes the single
authority; reconcile tags each call with an `IntrinsicId`; emit matches on
the enum instead of strings.

Transport kind, item kind, and type structure are mechanical enum conversions
with no design ambiguity.

| # | Closed set | Values | Dispatch sites | Files affected |
|---|-----------|--------|---------------|----------------|
| B-1 | Transport kind | rest, shell, file, local | 21 | 04_reconcile, 05_emit_rust, 05_emit_go, 05_emit_python |
| B-2 | Item kind (`classify_typed_item`) | type_def, type_alias, function, data_def, service_def, resource_def, extern_func, unhandled | 8 dispatch chains | 05_emit, 05_emit_rust, 05_emit_go, 05_emit_python |
| B-3 | Type structure (`classify_type_structure`) | leaf, conj, disj | 3 dispatch chains | 05_emit, 05_emit_rust, 05_emit_go, 05_emit_python |
| B-4 | Method/builtin intrinsics | ~35 methods + ~20 builtins | ~60 string branches | 04_reconcile (inference), 05_emit (classification), 05_emit_rust (lowering) |
| B-5 | Operation modifiers | idempotent, readonly, hermetic | 1 filter expression | 05_emit_rust:2836 |
| B-6 | Config property names | base_url, auth_scheme, auth_header, auth_token | `config_names` list + constructors + accessors | 00_core.dag (triple representation) |

**Previously tracked as:** TD-2, TD-3, F7 (partially — the emit-side ladder is Root Cause A)

---

### Root Cause C: Errors Propagate as Valid-Looking Fabrications

**Invariants violated:** No fallbacks that fabricate, Explicit boundary
contracts, Correctness by construction.

**The problem:** When the compiler encounters an error (missing argument,
unresolved type, unknown function), it fabricates a valid-looking node
(LitNull, Dynamic, `<error:*>` string) and continues. This lets broken
programs reach emit, which generates invalid target code containing
sentinels like `<error:unknown_with_type>` or empty strings.

**Design decision required:** Structural error representation. Currently
error state is encoded as:
- `LitNull` with `return_type: none` (37 sites across parser/reconcile/emit)
- `Dynamic` type name (universal compat in `node_type_equals`)
- `<error:*>` strings detected by `string_contains` (2 check sites, 4 production sites)
- `Warning` severity for semantic errors (`access_error`, `inference_error`)

The fix: make error a structural variant — either an `ExprError` in ExprData
or a flag on Node — so downstream phases can test `is_error(node)` without
string parsing. Emit skips error nodes (or emits `compile_error!()`) instead
of translating fabricated values. Reconcile promotes `access_error` /
`inference_error` to Error severity so `compile_sources` gates correctly.

Parser LitNull recovery (23 sites in `02_parse.dag`) is a separate concern —
parser error recovery that produces dummy nodes with attached error
diagnostics is standard practice. The issue is that reconcile and emit don't
recognize these as error nodes and try to process them normally.

| # | Pattern | Sites | Where |
|---|---------|-------|-------|
| C-1 | LitNull sentinel for missing arguments | 5 | `05_emit_rust.dag:1751,1752,1760,1761,1786` |
| C-2 | LitNull sentinel for missing defaults/config | 9 | `04_reconcile.dag:3025,3053,3114,3158,3165,3172,3272,3293,3510` |
| C-3 | LitNull dummy for parser error recovery | 23 | `02_parse.dag` (throughout) |
| C-4 | `<error:*>` placeholder types | 4 production | `04_reconcile.dag:1531,1698,1861,2255` |
| C-5 | `<error:*>` detection via string_contains | 2 check | `04_reconcile.dag:900`, `05_emit_rust.dag:1473` |
| C-6 | `<error:unknown_*>` sentinels in emit | 2 | `05_emit_rust.dag:1766,2117` |
| C-7 | Dynamic as universal compatibility | multiple | `node_type_equals` in `04_reconcile.dag:901+`; `extend_scope_for_lambda` in `05_emit_rust.dag:1959` |
| C-8 | Warning severity for semantic errors | 2 helpers | `access_error` / `inference_error` at `04_reconcile.dag:1236,1245`; `compile_sources` gates on Error only |
| C-9 | Empty node / empty string fabrication | 2 | `05_emit_rust.dag:819` (empty Node for missing field), `05_emit_rust.dag:3368` (LitNull → "") |
| C-10 | `Rc::try_unwrap` clone fallback (v1) | 1 | `fn_codegen.rs:3783` — blocked on Track D ownership proof |

**Previously tracked as:** TD-7, SB-2, SB-3

---

### Cleanup

| # | Severity | Description |
|---|----------|-------------|
| F5 | LOW | `infer → reconcile` rename lacks documented contract justification. |
| SG-9 | LOW | .dag workarounds for force_clone (TokPos extraction, branch-aware use counting). Revert after verification at scale — may be redundant after R9. |
