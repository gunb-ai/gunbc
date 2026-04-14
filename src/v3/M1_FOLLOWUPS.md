# M1(2.5) Follow-ups

Deferred work discovered during the M1(2.5) substrate rework. Each item
is out of scope for M1(2.5) but needs to land before M1 is fully done.

## Review pickups applied in this PR

**Codex review (all four blockers resolved):**
- Variant allocation ordering in `lower_type_sum`: the variant
  declaration is now allocated AFTER its payload children, so the
  dense-sequential invariant on `Dag.declarations` holds for sum
  types with non-empty payloads (`Result<T, E> = Ok(T) | Err(E)`
  etc.).
- Unresolved Identifier stubs: `alloc_identifier_stub` defers
  diagnostic emission; `resolve_pending_identifiers` is a post-
  lowering sweep that either fills in each stub's `resolved` slot
  from the completed declaration table or emits a fail-closed
  ResolveError diagnostic via a phantom port. Called at the end
  of `bootstrap::bootstrap` and `lower::lower`.
- Canonical `Declaration.type_params` slot: added as a first-class
  field. All three generic-carrying lowering paths (record, sum,
  alias) populate it via `allocate_type_params`, which also routes
  type params off the `Conj.children` / `Disj.variants` axis.
  `template_param_id` reads from this slot.
- Dissolution ledger receipts: `TypeConnective`, `AtomPayload`,
  `ArrowBody`, `CardinalityBound`, `LiteralBits` all carry 4-pattern
  ledger comments in `dag.rs`. M1_DESIGN.md §Q7 entries are mirrored
  in the Rust source.

**ChatGPT review (partial pickup):**
- FAIL-CLOSED for declarations: resolved via the sweep above.
  `alloc_identifier_stub` and the template-arity fallback both
  emit diagnostics; compile_to_dag surfaces them through
  `Err(Semantic(dag))`.
- ILLEGAL STATES / `template_param_id(...).unwrap_or(value)`:
  replaced with `build_template_arguments`, which emits an
  ArityMismatch diagnostic on param-count mismatch AND a
  ResolveError on per-index miss, then still returns a
  structurally well-formed `TemplateArgument` list. The
  diagnostic means the compile fails at the boundary regardless
  of the fallback substitution.

**ChatGPT review (deferred — require scope decision):**
- **FACTS FLOW FORWARD** — bootstrap still parses fixture strings
  instead of real `dsl/std/*.dag` files, and
  `declaration_to_type_shape` still hardcodes `Int`/`Bool`/`String`.
  The fix requires parser extensions for `module`/`import`/`data`/
  `match`/record literals and a bridge removal — squarely M1(2.6)
  work. Tracked below.
- **SINGLE AUTHORITY** — `inject_primitive_operators` still
  registers `"+"`, `"-"`, etc. as named Arrow declarations
  alongside the algebra fixture. The proper fix is SubstStack +
  §8.9 inhabitance walks (M1(2.6) priority #1 below), at which
  point `inject_primitive_operators` dissolves entirely.
- **AtomPayload `Identifier { resolved: Option<_> }` phase
  coproduct** — the None/Some split is functionally a pre- vs
  post-resolution phase flag hidden inside an Option. Splitting
  into `UnresolvedIdentifier(String)` vs
  `ResolvedIdentifier(DeclarationId)` is a substrate-shape
  change that would surface the phase distinction structurally.
  Deferred until the lowering pipeline stabilizes (M1(2.6) or
  later).

## M1(2.6) priority #1 — SubstStack + §8.9 inhabitance walks

**Status at M1(2.5):** the substrate shape is in place
(`TypeConnective::Instantiation` carries `template` +
`TemplateArgument` bindings, `OrderedRing<T>` declares `add:
fn(T, T) -> T`), and `parse_std_algebra_and_walk_int_add`
validates that the shape is correct — by doing manual
substitution inside the test itself. Production user code
(`1 + 2`) still routes through `inject_primitive_operators`'s
hardcoded `"+"` Arrow, not through
`Int → OrderedRing → add → substitute T := Word64`.

**What's missing:**
- `SubstStack` in `infer.rs` — the walker data structure that
  pushes bindings on entering an `Instantiation` and pops on
  exit, with linear lookup against the frame stack.
- §8.9 inhabitance walk in `resolve_arrow` — given a Transform
  target that's an operator identifier (`"+"`), walk the
  inferred LHS type into its `Instantiation` chain, find the
  matching algebra field by name (via an operator → field map
  like `"+" → "add"`), and substitute the template arguments
  against the SubstStack to yield a concrete Arrow signature.

**Dissolution outcome:** once the walk lands,
`inject_primitive_operators` can be deleted from `bootstrap.rs`
entirely — operators stop being a separate substrate concept
and become a naming convention that resolves through the
inhabitance chain. That's the bridge-removal the reviewer
flagged as M1(2.6) priority #1.

**Prereq for this:** none, strictly — the substrate shape is
already correct. It's a pure inference-side change.

## PR #444 pickup — adapted vs deferred items

## PR #444 pickup — adapted vs deferred items

PR #444 ("thesis + M1: static-only scope commitment + review cleanup")
is open and doc-only. M1(2.5) picks up its load-bearing code changes
and defers parser-dependent items:

**Picked up (code):**
- `Declaration.inhabits` split into `meta_tag: Option<DeclarationId>` +
  `inhabits: Option<DeclarationId>`. `meta_tag` is the value-construction
  meta-type tag; `inhabits` is the secondary algebra-inhabitance edge.
  Both stay `None` across the bootstrap set except for the §6.5 stub's
  `Int64_add_rust` instance, which sets `meta_tag = Some(Realization)`.
- §6.5 realization smoke test — adapted to bypass the
  `dsl/extdeps/languages/rust.dag` file and construct the Realization
  declaration chain directly in `bootstrap::inject_realization_stub`.
  Validates `ArrowBody::ExternalRealization(DeclarationId)` walks through
  inference without panicking. See
  `tests/m1_substrate_test.rs::smoke_int_add_external_realization`.

**Deferred:**
- **`dsl/extdeps/languages/rust.dag` parsed fixture** — blocked on
  parser support for record literals (`realization Int64_add { for: ...,
  target: ..., body: "..." }`) and a `realization` item keyword. M1(2.6)
  swaps the Rust-constructed stub in `inject_realization_stub` for
  fixture parsing once the parser handles this shape. The substrate
  shape is identical; only the data source changes.
- **§8.11 Pending-elimination CI ratchet** — doc-only spec; the actual
  CI wiring is M1(3) work. See §8.11 in `M1_DESIGN.md` (post-#444).
- **Dissolution ledger entries for `ArrowBody` and `AtomPayload`** —
  these are doc-only in `M1_DESIGN.md` and land when PR #444 merges.
  No code action.

## v3 CI coverage — NOT YET WIRED

`.github/workflows/ci.yml` currently runs only the v2 stage0 pipeline
(`cargo run -p v2-compiler --release -- run --source-root dsl
--function run_ci_pipeline`). It does NOT run `cargo test -p
v3-compiler` or `cargo clippy -p v3-compiler --all-targets`. This means
v3 regressions are caught only by local developer test runs, not CI.

This PR adds a separate `v3` job to ci.yml that runs tests and clippy
against the v3 crate. If that change is not desired in this PR, revert
the ci.yml edit and track the gap as a dedicated CI PR.

## Parser — production std/ parity (M1(2.6))

M1(2.5) uses fixture strings (logic/bit/algebra/types subsets) embedded
in `src/v3/compiler/src/bootstrap.rs` because the v3 parser cannot yet
parse `dsl/std/logic.dag`, `dsl/std/bit.dag`, `dsl/std/algebra.dag`, or
`dsl/std/types.dag` directly. The fixtures are semantically narrower —
only the declaration-tree shapes the M0 and substrate tests exercise.

Blockers for full parity:

- `module std.algebra` — `module` / `import` directives
- `fn f(...) { match x { A => ..., B => ... } }` — block-body functions
  with `match` expressions (M0's fn body is `=` expression-form only)
- `data kernel_type_set: Map<String, Bool> = { ... }` — `data`
  declarations with record literals
- `fn f() -> List<X> = [ ... ]` — list literals
- Field-level `Map<K, V>` with string-keyed record literals

None of these block M1(2.5) semantics, but blocking M1(3) Rust emission
against real std files means they must land before M1(3) consumes
`dsl/std/*`. M1(2.6) is the proper place.

## `ArrowBody::Pending` → resolved CI ratchet (M1(3))

`ArrowBody::Pending` is the scaffolded "signature valid, body not yet
loaded" state. At M1(2.5), every algebra field in the bootstrap fixtures
lands with `body: Pending`. Per M1_DESIGN.md §8.10 the CI ratchet that
enforces `Pending → UserDefined | ExternalRealization` before M3 is
deferred to M1(3).

Implementation sketch:
- `rg 'ArrowBody::Pending' src/v3/compiler/src/bootstrap.rs | wc -l`
  pins the current count.
- Phase-0 of M1(3) snapshots the count as the initial ratchet ceiling.
- Each new extdeps language spec that resolves a Pending arrow reduces
  the count; CI rejects increases.

## `dsl/std/meta.dag` (deferred until second consumer)

Not needed at M1(2.5). The one meta-type consumer in the synthetic
service test (`SyntheticService`, `SyntheticOperation`) is inline in
the test. A first-class `meta.dag` is waiting for a second consumer per
the construction-over-speculation principle.

## §8.9 full inhabitance walks (M1+)

M1(2.5) short-circuits operator dispatch by pre-registering `+`, `-`,
`*`, `/`, `==`, `!=`, `<`, `<=`, `>`, `>=` as named Arrow declarations
during `bootstrap::inject_primitive_operators`. The proper §8.9 path —
walking the LHS type's inhabitance chain (`Int` → `Instantiation(OrderedRing,
[T := Word64])` → find field `add`) — works for the substrate test's
direct walks but is not exercised in user-code operator dispatch.

Triggers for promoting operators to inhabitance dispatch:
- A second inhabitance chain (`Float = Field<Word64>`) needs the same
  operators with different signatures.
- `dsl/std/algebra.dag` parses directly (M1(2.6) above), at which point
  the named-operator bridge duplicates information.

## `fixtures/` as real files (optional)

Bootstrap fixtures are currently embedded as `const` strings in
`bootstrap.rs`. Moving them to on-disk fixture files under
`src/v3/compiler/tests/fixtures/` and `include_str!`-ing them gives
syntax highlighting and easier editing without runtime cost. Skipped at
M1(2.5) because it adds one layer of indirection for no semantic gain.

## `Node.name` field lifetime (cross-project)

Unrelated to M1(2.5) substrate. Tracking in
`project_node_name_deletion.md` — v2 project, separate PR cadence.

## Test 2 nesting depth

`parse_synthetic_service_all_layers` asserts a 3-level nesting (CmdExec
→ operations container → Run record). M1_DESIGN.md §6 describes a 5-level
nesting that additionally demands `Cardinality { element: String, bound:
Exact(3) }` for the `argv` field. M1(2.5) parser has no surface syntax
for fixed-length arrays; `[T; n]` is M2. Test 2 can be tightened to the
full §6 nesting once that syntax lands.
