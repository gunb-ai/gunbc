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

- **No flags in codegen.** Boolean flags that change compilation behavior
  globally (like `force_clone`) are forbidden. Every compilation decision
  must be derived from the actual type and context of the expression
  being compiled, not from a global check. Flags silently degrade and
  are impossible to remove incrementally.

## Testing Invariants

- **Behavioral only.** Tests assert observable behavior — outputs given
  inputs, error messages, public API contracts. Never assert internal
  implementation details like which private functions were called, what
  order internal steps execute in, or how many times an internal helper
  runs.

- **Source-audit checks are a narrow exception.** Keep source-text
  architectural ratchets out of the main Rust test suite when possible.
  When a source-audit check intentionally reads source text, it must
  anchor on live syntax or declarations and ignore comments or
  historical notes. A comment match is not evidence that a boundary or
  implementation still exists.

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

### Status After Final Cleanup

This workboard is complete in this branch.

Closed in this branch:
- Root Cause A: A-1, A-2, A-3, A-4, A-5, A-6, A-7, A-8, A-9, A-10
- Root Cause B: B-1, B-2, B-3, B-4, B-5, B-6
- Root Cause C: C-1, C-2, C-3, C-4, C-5, C-6, C-7, C-8, C-9, C-10

The root-cause tables below are preserved as the historical problem statement
that motivated the refactor, not as a live backlog.

---

### Root Cause A: Reconcile→Emit Boundary is Information-Lossy (ADDRESSED)

**Status:** Design decision made, infrastructure landed. Gradual migration underway.

**Design decision (2026-03-21):** Split into two categories:

1. **Reconcile resolution bugs (A-4, A-5, A-8, A-9):** Reconcile fails to resolve
   facts it should. Fix: improve resolution, add `RefKind` and `ParamSource` types.

2. **Emit rendering decisions (A-1, A-2, A-3, A-6, A-7, A-10):** Emit owns these
   decisions but must compute them efficiently. Fix: `EmitContext` struct with 6
   cached indexes built once per emit call, O(1) lookups per expression. No
   precomputation in reconcile — rendering decisions stay with the renderer.

**Infrastructure landed:**
- `EmitContext` type + `build_emit_context` + `ctx_*` helpers in `05_emit.dag`
- `RefKind`, `ParamSource` types in `04_reconcile.dag`
- `build_intrinsic_index`, `build_primitive_set` pre-built at emit entry
- EmitContext wired into `emit_rust` entry point

**Remaining:** Migrate emit functions from individual map params to `EmitContext`
lookups. Mechanical — each function gets `ctx: EmitContext` parameter, replaces
ad-hoc scans with `ctx_*` helpers.

| # | What reconcile computes | Where it's lost | How emit compensates |
|---|------------------------|-----------------|---------------------|
| A-1 | Field access style (StoredField / EnumAccessor / OptionalUnwrap) — `build_field_summaries_*` at `04_reconcile.dag:1070-1175` | Not attached to ExprFieldAccess nodes | `emit_typed_field_access` calls `lookup_emit_field_summary_in_scope` at codegen time (redundant); `is_likely_optional_receiver` scans all type_summaries; `is_optional_field_in_any_type` / `is_enum_accessor_in_any_type` do global sweeps (`05_emit_rust.dag:1576-1601`) |
| A-2 | Known-method classification + result type — `resolve_known_method_node` in `04_reconcile.dag` | `ExprMethodCall` now carries `method_semantics`; remaining loss is that renderer leaf helpers still branch on `method` strings for target syntax | Complexity no longer compensates. Emit still has per-target method-name ladders and runtime helper tables. |
| A-3 | Call→MethodCall bridging — ExprCall handler rewrites bridged calls to `ExprMethodCall` | No longer lost after reconcile; bridged calls remain structurally distinct downstream | Emit no longer needs to rediscover bridged method shape, but Rust still carries target-specific runtime helper maps for ownership/rendering. |
| A-4 | Function-as-value reference — `lookup_in_scope` fallback to `lookup_func_sig` at `04_reconcile.dag:751-754` | ExprVar node gets return type only; callable-vs-value distinction lost | Emit cannot distinguish function reference from local binding (SB-1). Fabricates value type from callable's return type. |
| A-5 | Fold accumulator type — computed during method resolution | No longer lost on typed method nodes; carried in `IntrinsicMethodSemantics.fold_accumulator_type` | Downstream consumers can read it from `method_semantics`; remaining work is deleting renderer-local fallbacks. |
| A-6 | Rc-wrapping requirement — derivable from type summaries and scope types | Not attached per expression; Rust emit still re-derives it from a module-local `rc_types` map plus Rust-local match analysis | Emit now centralizes match probing through `RcPatternAnalysis`/`RcMatchAnalysis`; lookup-specific wrapping on data maps remains separate |
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

### v2 Pipeline Audit (2026-03-22)

Full line-by-line audit of all 14 v2 .dag files (~16,000 lines). ~100 violations
across 7 structural themes. Root Causes A/B/C above were v1-focused; these are
the v2-native counterparts. Execution order minimizes total work — each theme
unblocks or cheapens the next.

**Execution order:** 4 → 6 → 3 → 5 → 1 → 2 → 7 (interleaved)

#### Why These Exist — Three Root Causes

The 7 themes compress to 3 root causes. Understanding them prevents recurrence.

**I. The IR conflates domain facts with rendering strategy.**

The clearest example was shared semantics carrying Rust policy.
`MethodSemantics` used to carry `wrap_result_in_rc` and `pass_receiver_by_ref`,
and `CallSemantics` used to carry `needs_rc_wrap` — fields that only made sense
if compiling to Rust. Those fields are now gone from `00_core.dag`, but the
lesson remains: once target policy enters shared semantics, every downstream
consumer starts destructuring facts it doesn't own. Python and Go previously had
to pattern-match Rust-only fields they did not use.

Once the IR carries rendering hints, the boundary between "what the program is" and
"how to render it" blurs. Reconcile starts computing Rc decisions. Emit starts
re-resolving types. Dynamic becomes a catchall because the type system serves two
masters. This is the origin of Themes 3, 5, and partially 7.

Prevention: for every field on a core type, ask "would this field make sense if we
were compiling to VHDL?" If no, it doesn't belong in core.

**II. No structural fold over ExprData.**

The DAG language has sum types but no generic visitor. Every consumer writes its own
20-arm match. 5 consumers × 20 variants = 100 match arms that must stay in sync.
Adding one ExprData variant means editing 10+ functions across 6 files.

The "shared dispatch" for Theme 2 is building a `fold_expr` by hand. The reconcile
fusion (Theme 1) is manually combining two handler tables into one match. Both are
workarounds for the language lacking parametric types — you can't write
`fold_expr(handler: ExprHandler<A>, acc: A, expr: Node) -> A` without generics.

Prevention: the language design choice to omit generics forced copy-paste parallelism.
Until/unless the language gains parametric types, the mitigation is to minimize the
number of walks (target: 5) and never add a new one without deleting an existing one.

**III. Define-at-use-site instead of import-from-authority.**

When reconcile needed kernel types, it defined `is_primitive_name()`. When emit needed
them, it defined `build_primitive_set()`. When complexity needed method costs, it
defined `classify_method_cost()`. Each file solved its local problem by copying. Dead
stubs (artifact, trace) are the flip side: code written speculatively, never connected
to an authority. This is Themes 4 and 6.

Prevention: import-first discipline. Before defining a list or classifier, check if an
upstream module already has one. If not, define it in the lowest shared module, then
import.

**All seven themes are symptoms of one thing: the v2 compiler was built bottom-up.**
Each file solved its local problem correctly. Nobody enforced that shared facts flow
downward from a single authority. The fix is to invert the direction: define authorities
first, then build consumers that import from them.

#### Acceptance Criteria — End State

All themes done = every item below is checked. Organized by file so nothing
gets missed during cleanup. Items marked DELETE must not exist; items marked
GONE mean the surrounding function/field no longer exists in that file.

**`00_core.dag`**
- [x] `kernel_types: List<String>` exists (canonical list, 8 entries)
- [x] `is_kernel_type(name: String) -> Bool` exists, uses `kernel_types`
- [x] `LookupCallSemantics` has no `needs_rc_wrap` field
- [x] `IntrinsicMethodSemantics` has no `wrap_result_in_rc` field
- [x] `RuntimeBridgeSemantics` has no `wrap_result_in_rc` or `pass_receiver_by_ref` fields
- [x] `expr_self_call_info(...)` exists and computes both recursion facts in one walk
- [ ] `expr_has_self_call` GONE (compat wrapper removable after downstream imports stop depending on it)
- [ ] `expr_has_non_tail_self_call` GONE (compat wrapper removable after downstream imports stop depending on it)

**`03_resolve.dag`**
- [x] `kernel_type_names()` DELETE — callers import `kernel_types` from core

**`04_reconcile.dag`**
- [x] `is_primitive_name()` DELETE — callers import `is_kernel_type` from core
- [x] `build_type_env` kernel list (lines 3157-3164) replaced with `kernel_types` import
- [x] `build_type_env_unresolved` kernel list (lines 3260-3264) replaced with `kernel_types` import
- [x] `node_is_named_ref` inline kernel exclusion (lines 968-978) uses `is_kernel_type`
- [x] `type_needs_rc` GONE (moved to emit_rust)
- [x] `type_needs_rc_seen` GONE (moved to emit_rust)
- [x] `data_lookup_needs_rc_wrap` GONE (moved to emit_rust as `rust_lookup_receiver_needs_rc_wrap`)
- [x] `rc_wrapped: Bool` GONE from TypeSummary
- [x] `rc_wrapped_types: Map<String, Bool>` GONE from EmitGraphInfo
- [x] `rc_wrapped_types: Map<String, Bool>` GONE from EmitStateAccum
- [x] `emit_info_is_rc_wrapped_type` GONE (moved to emit_rust)
- [x] All `rc_wrapped_types` accumulation logic GONE from `build_emit_graph_info`
- [ ] `infer_expr` and type resolution fused into single walk (`infer_and_resolve_expr`)
- [ ] `collect_calls_in_expr` + `expr_has_self_call` + `expr_has_non_tail_self_call` fused into `analyze_expr_calls` returning `CallAnalysis`
- [ ] Dynamic sites audited: each classified as Correct/Lazy/Fixed, ≤5 justified remaining
- [ ] No string-based method dispatch downstream of the classifiers in reconcile

**`05_emit.dag`**
- [x] `build_primitive_set()` DELETE — callers import `kernel_types` from core
- [x] `ctx_is_rc_wrapped` GONE (Rc concern moved to emit_rust)
- [ ] Shared `emit_typed_expr` dispatch exists (single 20-arm match, target parameter)
- [ ] Shared TCO dispatcher with `TcoSyntax` config exists
- [ ] Shared service/transport traversal exists

**`05_emit_rust.dag`**
- [x] `build_module_vtoe` stub DELETE
- [x] `emit_record_lit` compat wrapper DELETE (tests updated to call `emit_record_lit_full`)
- [x] `resolve_expr_type_node` DELETE
- [x] Rc decision map is derived once in Rust emit entry/module wrappers from `type_summaries`
- [x] `type_needs_rc`, `type_needs_rc_seen` live here (moved from reconcile)
- [x] `rust_lookup_receiver_needs_rc_wrap` lives here (moved from reconcile)
- [x] All 6 Rc-probing heuristics consolidated into the Rust-side Rc pre-pass
- [ ] `emit_typed_expr` 20-arm match GONE (replaced by leaf functions called from shared dispatch)
- [ ] `emit_typed_tco_expr` parallel walk GONE (replaced by shared TCO dispatcher)
- [x] All 18 intrinsic methods handled (no fallback arms)

**`05_emit_python.dag`**
- [ ] `_unimplemented` placeholders (lines 1063, 1076) GONE — real emission or compile error
- [ ] `emit_py_typed_expr` 20-arm match GONE (replaced by leaf functions)
- [ ] All 18 intrinsic methods handled (currently 7)
- [ ] No silent fallback for unhandled methods

**`05_emit_go.dag`**
- [ ] `/* unhandled expr */` wildcard (line 592) GONE — match is exhaustive
- [x] Dead `if wrap_result_in_rc` identity branch (line 659) GONE
- [ ] `emit_go_typed_expr` 20-arm match GONE (replaced by leaf functions)
- [ ] All 18 intrinsic methods handled (currently 7)
- [ ] No silent fallback for unhandled methods

**`06_pipeline.dag`**
- [x] Artifact computation block (lines 170-179) DELETE — `_artifact_output`, `plan`, `artifact` locals all gone
- [x] Go arm wired: `Go => emit_go(typed: typed)` (not error diagnostic)
- [x] `import v2.compiler.emit_go { emit_go }` exists
- [x] `resolve_sources` refactored — shared tokenize→parse→resolve helper with `compile_sources`
- [x] Header comment (line 10) matches reality (mentions Go alongside Rust/Python)

**`07_complexity.dag`**
- [x] `intrinsic_method_cost_shape` is the only method→`CostShape` authority — no parallel `classify_method_cost` or inline string classifier remains
- [ ] `intrinsic_cost_shape` is exhaustive match on IntrinsicMethod — no Option, no None, no wildcard
- [x] `is_size_preserving_method(mname: String)` DELETE — replaced by `is_size_preserving(intrinsic: IntrinsicMethod) -> Bool`
- [x] `is_size_preserving` is exhaustive match — no string comparison
- [ ] `count_self_calls` (lines 1253-1323) DELETE — fused into `cost_of_expr` or uses `CallAnalysis` from reconcile
- [x] `cost_of_expr` reads `method_semantics` from Node, never matches on method name strings

**`07_ownership.dag`**
- [ ] Match arm patterns walk `VariantPattern` bindings (currently skipped)
- [ ] Destructuring patterns updated for any MethodSemantics field changes from Theme 5

**`08_artifact.dag`**
- [x] `plan_artifacts` ModuleBased stub arm (lines 86-88) DELETE
- [x] `plan_artifacts` ServiceBased stub arm (lines 89-91) DELETE
- [x] Only `Explicit` arm remains, or function deleted entirely

**`09_trace.dag`**
- [x] `import std.types { SourceSpan }` fixed to `import v2.std.core { SourceSpan }`
- [x] Interpreter-oriented header/comments reconciled with `src/v2/DESIGN.md` (compiler is a pure transform; no interpreter in the compiler)
- [x] Module connected to pipeline (called from `compile_sources`) or explicitly marked as future work

**Cross-cutting invariant: `00_core.dag` is target-agnostic.**

Every field on every type in `00_core.dag` must satisfy: "this field would make sense
if we were compiling to VHDL, C, or a hardware description language." Fields that
encode a specific target's memory model (Rc, borrow, GC, pointer), execution model
(async, coroutine), or syntax (indentation, braces) do not belong in core.

Rendering decisions are *computed* by emit from domain facts — never *stored* on core
types. If `type_needs_rc` is derivable from cycle detection on the type graph (it is),
it should never have been a field. If `pass_receiver_by_ref` is derivable from Rust's
borrow rules applied to the method's receiver type (it is), it should never have been
a field. The domain model records the facts; each backend derives its strategy.

After cleanup, this grep returns zero results:
```
rg -n '\b(needs_rc_wrap|wrap_result_in_rc|pass_receiver_by_ref|Rc|borrow)\b' src/v2/00_core.dag
```

**Cross-cutting invariant: no new ExprData walks without deleting an existing one.**

Until the language gains parametric types (enabling a generic `fold_expr`), the number
of full ExprData walks is capped at 5. Adding a 6th walk requires justification and
consolidation of an existing pair. The 5 allowed walks are:
1. `infer_and_resolve_expr` (reconcile) — type inference + resolution
2. `analyze_expr_calls` (reconcile) — call graph + recursion detection
3. `walk_expr` (ownership) — binding consumption classification
4. `cost_of_expr` (complexity) — symbolic cost computation
5. `emit_typed_expr` (emit, shared) — target-language rendering

**Cross-cutting invariant: import-from-authority, never define-at-use-site.**

Before defining a list, classifier, or predicate, check if an upstream module already
defines the same concept. If not, define it in the lowest shared module that all
consumers import, then import it. A fact defined at the use site will be copied to the
next use site.

**Cross-cutting verification:**
- [x] `rg -n '"String", "Int", "Bool"' src/v2/*.dag` returns only `00_core.dag`
- [x] `rg -n 'wrap_result_in_rc' src/v2/*.dag` returns 0 results
- [x] `rg -n 'pass_receiver_by_ref' src/v2/*.dag` returns 0 results
- [x] `rg -n 'needs_rc_wrap' src/v2/*.dag` returns only `05_emit_rust.dag`
- [x] `rg -n '\b(needs_rc_wrap|wrap_result_in_rc|pass_receiver_by_ref|Rc|borrow)\b' src/v2/00_core.dag` returns 0 results
- [ ] `grep -r '"Dynamic"' src/v2/` returns ≤5 results in `04_reconcile.dag`, all with justification comments
- [ ] `grep -r '_unimplemented\|/\* unhandled' src/v2/` returns 0 results
- [ ] `grep -rn 'ExprLiteral.*ExprVar.*ExprCall' src/v2/` — full 20-arm ExprData matches exist only in: `infer_and_resolve_expr`, `analyze_expr_calls`, `walk_expr` (ownership), `cost_of_expr`, `emit_typed_expr` (shared). Total: 5 walks, down from 11+.
- [ ] Adding a new ExprData variant requires editing ≤5 match arms (one per walk above)
- [ ] Adding a new intrinsic method requires editing 4 files: core (enum), reconcile (classifier), and one leaf function per target renderer

---

#### Theme 4: Kernel/Primitive Lists → Single Source of Truth

**Invariant:** No duplicate representations. Single-authority metadata.

**Problem:** 4+ copies of the same 6-8 type names (`kernel_type_names`,
`is_primitive_name`, `build_primitive_set`, `build_type_env` hardcoded list),
already drifting (`build_primitive_set` adds `"Char"`).

**Design:** Add to `00_core.dag`:
```
data kernel_types: List<String> = ["String", "Int", "Bool", "Float", "Secret", "Json", "Unit", "Bytes"]
fn is_kernel_type(name: String) -> Bool { kernel_types |> any(t => t == name) }
```

Delete `kernel_type_names()` in 03_resolve, `is_primitive_name()` in 04_reconcile,
`build_primitive_set()` in 05_emit, hardcoded list in `build_type_env`. All import
from core.

**Effort:** ~30 min. Zero risk.

---

#### Theme 6: Dead/Disconnected Infrastructure → Delete or Connect

**Invariant:** No fallbacks that fabricate. No parallel implementations.

| Dead code | Action |
|-----------|--------|
| Pipeline artifact stage (`_artifact_output`, lines 170-179) | Delete |
| `plan_artifacts` stub arms (ModuleBased/ServiceBased) | Delete |
| Artifact/Boundary types | Keep — forward-looking, types are cheap |
| Go pipeline dispatch (returns error despite emit_go existing) | Add `import emit_go`, wire `Go => emit_go(typed: typed)` |
| Trace `import std.types` | Fix to `import v2.std.core` |
| `build_module_vtoe` stub | Deleted |
| `resolve_sources` duplication | Extract shared tokenize→parse→resolve helper |
| `emit_record_lit` compat wrapper | Update tests, delete wrapper |
| Pipeline header comment | Fix to match reality |

**Effort:** ~1 hour. Low risk — mostly deletion.

---

#### Theme 3: String-Keyed Method Dispatch → Enum Everywhere

**Invariant:** No case enumeration for open sets. Single-authority metadata.

**Problem:** The enum pipeline is now mostly in place. `cost_of_expr` and
`receiver_size_var` dispatch on reconcile-provided `MethodSemantics`, and
reconcile now resolves known method semantics/result types once via
`resolve_known_method_node`. Residual string-based method logic still lives
in the source-to-semantics classifiers (`classify_reconciled_intrinsic_method`,
`classify_runtime_bridge_method`) and in per-target renderer leaf dispatch.

**Design:** After reconcile, every method call carries `MethodSemantics` with
`IntrinsicMethodSemantics { intrinsic: IntrinsicMethod, ... }`. Downstream phases
dispatch on the enum, never on strings.

Changes:
1. Keep `intrinsic_method_cost_shape(intrinsic) -> CostShape` as the single cost-shape authority
2. `resolve_known_method_node(...)` is the single reconcile authority for known method semantics and result types
3. `receiver_size_var(...)` reads `MethodSemantics`, not method-name strings
4. Delete the remaining string-based method dispatch downstream of the reconcile classifiers

Single authority chain:
```
string (source) → reconcile → IntrinsicMethod (enum)
                                  ↓
                    emit: rendering per intrinsic
                    complexity: CostShape per intrinsic
                    ownership: edge classification per intrinsic
```

**Effort:** ~2 hours. Medium risk — touches reconcile/emit/complexity.

---

#### Theme 5: Target-Specific Leakage → Ownership as Rendering Concern

**Invariant:** DAG nodes are facts, rendering is separate.

**Problem:** Rc wrapping still spans multiple places inside the Rust renderer
(`type_needs_rc`, pattern-deref heuristics, lookup-specific wrapping, DryRunMode),
even though the Rust-only fields and shared Rc indexes have now been removed from
core/reconcile/shared emit.

**Design:** Reconcile produces target-agnostic facts only. Rust-specific ownership
decisions move to a Rust-specific pre-pass within emit_rust.

Move FROM reconcile to emit_rust:
- `type_needs_rc`, `type_needs_rc_seen` → Rust renderer pre-pass [done]
- `data_lookup_needs_rc_wrap` → Rust renderer (`rust_lookup_receiver_needs_rc_wrap`) [done]
- `rc_wrapped` on TypeSummary → deleted from shared summaries; Rust derives Rc status locally [done]

Move FROM emit shared to emit_rust:
- `resolve_expr_type_node` → deleted; emitter now trusts typed nodes plus narrow local fallback [done]
- All 6 Rc-probing heuristics → compute once in a Rust-side pre-pass via `RcPatternAnalysis` / `RcMatchAnalysis` [done]

Clean up MethodSemantics:
- `wrap_result_in_rc` on IntrinsicMethodSemantics/RuntimeBridgeSemantics → Rust-only context [done in core semantics]
- `pass_receiver_by_ref` on RuntimeBridgeSemantics → same [done in core semantics]
- `needs_rc_wrap` on LookupCallSemantics → same [done in core semantics]

Target-agnostic facts reconcile SHOULD provide:
- "This type is recursive" (structural via children/connective)
- "This type participates in a cycle of depth N" (SCC analysis)
- "This value has N semantic consumers" (ownership analysis)

**Prerequisite:** None, but unblocks Theme 2.
**Effort:** ~3-4 hours. Higher risk — restructures reconcile/emit boundary.

---

#### Theme 1: Parallel ExprData Walks → Fuse Reconcile Passes

**Invariant:** No parallel implementations. No duplicate representations.

**Problem:** 8+ complete 20-arm ExprData walks. Reconcile still has 4. Adding one
expression kind edits 10+ match arms.

**Current reconcile walks:**
1. `infer_expr` — type inference (20 arms)
2. `resolve_expr_types` — type resolution (20 arms)
3. `collect_calls_in_expr` — call graph edges (20 arms)
4. `expr_self_call_info` — shared self-recursion + TCO eligibility walker in `00_core.dag` (20 arms, wrapped by `expr_has_self_call` / `expr_has_non_tail_self_call`)

**Fused design:**

Walk A (`infer_and_resolve_expr`): Combines (1) and (2). Infer each subexpression's
type, resolve it in the same traversal. Single 20-arm match.

Walk B (`analyze_expr_calls`): Combines (3) and the shared self-call analysis. Returns
`CallAnalysis { all_calls, has_self_call, has_non_tail_self_call }`.

Complexity module: `cost_of_expr` + `count_self_calls` → fuse into one walk.

Current reduction: the separate self-call/TCO walks have already been collapsed to one shared core walk. Final target remains 4 reconcile walks → 2. Cost of adding ExprData variant drops from 10+ to ~4.

**Effort:** ~4-5 hours. Medium risk — reconcile is the largest file.

---

#### Theme 2: Triple Renderer Parallelism → Shared Dispatch + Per-Target Leaves

**Invariant:** No parallel implementations.

**Problem:** 3× expression dispatch (60 match arms), 3× TCO, 3× services,
3× resources, 3× data. Python/Go only handle 7/18 intrinsic methods with
silent fabricating fallbacks.

**Design:** One shared expression dispatch in `05_emit.dag`, per-target leaf
functions in emit_rust/emit_python/emit_go.

Shared dispatch:
```
fn emit_typed_expr(texpr: Node, target: RenderTarget, ctx: EmitContext, ...) -> String {
  match texpr.expr_data {
    ExprLiteral { value: v } => target_literal(v, target)
    ExprMethodCall { ... } => target_method_call(recv_str, method, arg_strs, ms, target, ...)
    ...
  }
}
```

Per-target files shrink to leaf renderers only (`target_literal`, `target_call`,
`target_method_call`, `target_match`, etc.). TCO uses shared dispatcher with
per-target syntax config (`TcoSyntax { loop_open, break_prefix, continue_kw }`).

After Theme 3, each renderer provides `render_intrinsic(intrinsic, recv, args) -> String`
covering all 18 intrinsics.

**Prerequisite:** Theme 5 (move Rc to emit_rust) — otherwise shared dispatch
must thread Rust-specific state that Python/Go ignore.

**Size reduction estimate:**
- emit_rust: 3,571 → ~1,200 (leaves + Rust pre-pass)
- emit_python: 1,169 → ~400 (leaves only)
- emit_go: 1,196 → ~400 (leaves only)
- emit shared: 819 → ~1,500 (shared dispatch + helpers)
- Total: 6,755 → ~3,500 (~48% reduction)

**Effort:** ~8-10 hours. Highest risk — largest restructuring. Do last.

---

#### Theme 7: Fabricating Fallbacks → Fail Loud or Implement

**Invariant:** No fallbacks that fabricate. Correctness by construction.

**Problem:** Silent fallbacks produce valid-looking but wrong output.

| Fabrication | Action |
|-------------|--------|
| `Dynamic` as permissive wildcard (~25 reconcile sites) | Audit each: keep justified, fix lazy inference, convert error-masking to diagnostic. Target: <5 justified. |
| Python/Go intrinsic fallthrough (7/18) | Implement all 18 per language (Theme 2). Until then, emit `raise NotImplementedError` / `panic` with context. |
| Go `/* unhandled expr */` wildcard | Make match exhaustive — add remaining ExprData arms. |
| Transport panic/unimplemented | Emit language-specific compile error with context. |
| `plan_artifacts` ignoring config | Delete stubs (Theme 6). |

**Dynamic audit classification per site:**
- **Correct:** genuinely polymorphic position → keep
- **Lazy:** type could be inferred but isn't → fix inference
- **Error-masking:** inference failed silently → convert to diagnostic

**Effort:** Ongoing, ~1 hour per batch of 5 Dynamic sites. Interleave with Themes 1 and 5.

---

### Inference produces incomplete type structures that emit compensates for

**Invariant violated:** Correctness by construction, not by validation.

**Observation (2026-03-25):** `bare_map_node()` and `bare_list_node()` in
`04_types.dag` create container type nodes with zero children. These are
structurally incomplete — a `Map` without key/value children is not a fully
resolved type. Inference hands them to emit unchanged (via `empty_map()`,
`map_insert()`, `map_merge()` in `04_infer.dag` and `04_method.dag`).

The old per-backend emitters compensated with hardcoded fallbacks:
`"Map"` → `"BTreeMap<_, _>"`, `"List"` → `"Vec<_>"`. When the shared emitter
was extracted (P4.2), these compensations were initially lost. The shared
emitter now restores them (`emit_node_type_leaf_rc` bare container branch),
but the fix is in the wrong layer — emit shouldn't need to know that
inference might produce incomplete containers.

| # | Severity | Where | What |
|---|----------|-------|------|
| IV-1 | MED | `04_types.dag:76-78` | `bare_map_node()` creates Map with 0 children — structurally incomplete |
| IV-2 | MED | `04_infer.dag:1912-1921` | `empty_map()` returns bare container without resolving type params |
| IV-3 | MED | `04_method.dag:153,176` | `map_insert()`/`map_merge()` return bare_map_node |

**Direction:** Either inference resolves container type parameters from context
(bidirectional inference), or bare containers carry an explicit "unresolved
parameters" marker that emit can handle uniformly rather than per-backend.

---

### Silent type fabrication in emit

**Invariant violated:** No fallbacks that fabricate.

**Observation (2026-03-25):** Several emit code paths produce valid-looking
but wrong output instead of failing. The `"String"` fallback was the
canonical case — a multi-field anonymous product with a missing `return_type`
emitted `(String, SomeType)` as valid Rust that compiles but has the wrong
type. Single-field products correctly used `compile_error!`.

Fixed: multi-field anonymous product now uses `compile_error!` (2026-03-25).
CLI param type mapping (`05_emit_rust.dag:3584-3591`) still fabricates
`"String"` for structured/unknown types — left as-is because CLI surface
is P4.5 scope, but tracked here.

| # | Severity | Where | What |
|---|----------|-------|------|
| IV-4 | FIXED | `05_emit.dag:952` | Multi-field anonymous product: `"String"` → `compile_error!` |
| IV-5 | LOW | `05_emit_rust.dag:3584-3591` | CLI param type mapping fabricates `"String"` for unknown types |

---

### Cleanup

| # | Severity | Description |
|---|----------|-------------|
| F5 | LOW | `infer → reconcile` rename lacks documented contract justification. |
| SG-9 | LOW | .dag workarounds for force_clone (TokPos extraction, branch-aware use counting). Revert after verification at scale — may be redundant after R9. |
