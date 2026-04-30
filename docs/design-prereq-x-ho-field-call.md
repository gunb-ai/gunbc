# Prereq-X — call-on-field-access prerequisite for `fold_lens<C>`

**Status:** AUDIT (Director-approved 2026-04-30 on parent inbox #1130).
Authored at the stop-and-ping point of the `fold_lens<C>` core slice
after the HO field-call grammar smoke confirmed the surface is not in
v3 today. Names the exact missing parser/lowerer surfaces required
before any `Lens<C>` instance can be invoked from `.dag`.

This audit does not author code. It records the smoke evidence,
splits the prerequisite into implementation slices, and maps each to
the consumer-side dispatch shapes `fold_lens<C>` and lens-instance
authoring need.

---

## Smoke evidence

Four shapes tested via `cached_compile_to_dag(...)` against current
`origin/main` post-Prereq-1 (#1230 / #1239), Prereq-2 (#1248), and
Prereq-3a (#1232). All four fail. The fixtures and tested error
messages are recorded verbatim so the implementation slice can use
them as regression cases.

### S1 — direct call-on-field-access `w.f(x)`

```dag
type WrapFn { f: fn(Int) -> Int }
fn invoke(w: WrapFn, x: Int) -> Int = w.f(x)
```

Result: parser failure.

```
Parse(ParseError {
  message: "expected `let`, `fn`, `type`, `module`, `import`, or `data`, got LParen",
  span: ho_param.v3 [76, 77]
})
```

The parser consumes `w.f` as a field-access expression, then sees `(`
with no expression-call grammar rule to apply, and falls through to
the top-level decl parser which expects keywords. The `(` is
interpreted as the start of a malformed top-level item.

**Conclusion:** call-on-field-access (`<expr>.<ident>(<args>)`)
where `<expr>.<ident>` resolves to an Arrow-typed value is not in
the v3 surface grammar.

### S2 — parenthesized callee `(w.f)(x)`

```dag
fn invoke(w: WrapFn, x: Int) -> Int = (w.f)(x)
```

Result: parser failure.

```
Parse(ParseError {
  message: "expected primary expression, got LParen",
  span: ho_param.v3 [...]
})
```

Parenthesized expressions are not in the primary-expression position
grammar; `(...)` cannot be used to bracket the callee.

**Conclusion:** the call-on-arbitrary-expression dispatch position
is constrained to identifier callees, with no parenthesization
escape.

### S3 — top-level let + call-on-Var

```dag
type WrapFn { f: fn(Int) -> Int }
fn double(n: Int) -> Int = n + n
data wrap_double: WrapFn = { f: double }
let g = wrap_double.f
let result = g(5)
```

Result: two semantic diagnostics.

```
ResolveError {
  name: "dotted path \"wrap_double.f\" is not a local field access; \
         expression-position dotted paths currently require a local-variable \
         head or a `data` declaration with a compile-time value",
  span: ho_call_let.v3 [117, 130]
}
```

```
ResolveError {
  name: "g",
  span: ho_call_let.v3 [144, 148]
}
```

Two layered failures:

1. `wrap_double.f` does not project as an expression-position
   field access even though `wrap_double` is a `data` declaration.
   The error message hints "compile-time value" is required; the
   `data` body's `value_body: Some(ValueBody::Structural { ... })`
   apparently does not satisfy the projector's requirement, so
   the projection bails before producing a value.
2. `g` cannot resolve at `let result = g(5)`'s call site because
   `g` was never bound (the prior `let g = ...` aborted at
   resolve time per #1).

**Conclusion:** even setting aside grammar gaps, expression-position
field access on a `data` binding does not lower today. The "local
variable head" path is also not available because top-level `let` is
not a function-scope binding.

### S4 — brace-block let-then-call inside `=` body

```dag
fn invoke(w: WrapFn, x: Int) -> Int = {
  let g = w.f
  g(x)
}
```

Result: parser failure.

```
Parse(ParseError {
  message: "expected field label, got KwLet",
  span: ho_field_call.v3 [151, 154]
})
```

The parser treats `{` after `=` as a record literal opening, not a
block expression. The first token after `{` must be a field label
(record discipline); `let` is rejected. Prereq-2 (#1248) added
**brace-bodied function parsing** for top-level fn definitions, but
the brace-block-as-expression grammar inside `=` bodies is a
distinct surface that did not land.

**Conclusion:** `fn name(...) -> T = { let ...; <expr> }` is not
expressible. Block expressions with intermediate `let` bindings
cannot be inlined into expression-bodied function definitions.

---

## Implementation prerequisites — three slices

The four smoke failures collapse to three independent prerequisite
slices. Each can land separately; the implementation worker picks
sequencing.

### Prereq-X1 — call-on-field-access dispatch

**Scope:** extend the surface call-position grammar so that any
expression evaluating to an Arrow type can occupy the callee
position. Concretely, generalize the call rule from
`<ident>(<args>)` to `<expr>(<args>)` where `<expr>` resolves
through inference to an Arrow type. Field projection in callee
position (`<expr>.<ident>(<args>)`) is the primary motivating case.

**Lowerer + substrate impact:** call-site lowering today dispatches
on the resolved decl-id of the head identifier and emits a
`TransformTarget::Callable(DeclarationId)` (`src/v3/compiler/src/dag.rs:1695-1712`).
The current `TransformTarget` enum has three variants — `Callable`,
`FieldProject`, `Operator` — and **none** carries a runtime-port-
sourced Arrow value. Two cases per call-site need lowering, and
one of them requires a substrate extension:

- **(L1.a) Statically-resolvable callee — reuse `Callable(decl_id)`.**
  When the Arrow expression resolves at lowering time to a top-level
  `fn` declaration (e.g., `data v: WrapFn = { f: double }; v.f(x)`,
  where `v.f` projects a `FieldValue::Reference(double)` from the
  data binding's `ValueBody::Structural`), the projection is
  compile-time. Lowering walks `v` → `data v: WrapFn`'s
  `value_body` → `f: FieldValue::Reference(decl_id_of_double)`,
  resolves `decl_id_of_double` to `double`'s arrow signature, and
  emits `TransformTarget::Callable(decl_id_of_double)` directly. No
  substrate extension; the carrier identity is preserved through
  field projection at the lowering boundary.

- **(L1.b) Runtime-sourced callee — substrate extension required.**
  When the callee is a function parameter (`fn invoke(w: WrapFn, x: Int)
  = w.f(x)`) or a let-bound projection from a runtime-source value,
  the callee Arrow is not statically resolvable to a top-level decl
  and must be sourced from a port. Today's `TransformTarget` has
  no variant that takes a `PortId` for the dispatch target;
  `Callable(DeclarationId)` requires a static decl, `FieldProject`
  is for projecting Conj children at the type-substitution boundary
  (not for invoking Arrow values), `Operator` is for built-in
  primitives.

  The substrate extension is `TransformTarget::IndirectCall` with
  the callee port carried in `TransformNode.inputs`, alongside the
  argument ports. Two competing constraints shape the encoding:

  - **Facts Flow Forward / Every Dependency Is A Substrate Fact:**
    reflected consumers walk `TransformNode.inputs` to derive
    dependencies. A separate-field callee outside `inputs` would
    be invisible to that walk.
  - **Illegal states unrepresentable:** a positional convention
    (`inputs[0]` = callee, `inputs[1..]` = args) admits malformed
    states like `IndirectCall` with empty `inputs` or non-Arrow
    `inputs[0]`. Pushing enforcement to later type-checking
    violates modeling-discipline §"API-level enforcement."

  **Resolution: structurally-tagged input element type.** Refine
  `TransformNode.inputs` from `Vec<PortId>` to
  `Vec<TransformInput>` where:

  ```rust
  pub enum TransformInput {
      Arg(PortId),       // ordinary argument port; existing semantics
      Callee(PortId),    // Arrow-typed dispatch source; valid only inside IndirectCall transforms
  }
  ```

  This preserves both invariants: `inputs.iter()` still walks every
  dependency port (Facts Flow Forward), and the `Callee`/`Arg`
  distinction is structural rather than positional (illegal states
  unrepresentable at the variant level). For `Callable`,
  `FieldProject`, `Operator` transforms, every element is
  `TransformInput::Arg(_)`. For `IndirectCall`, exactly one
  element is `TransformInput::Callee(_)`; the rest are `Arg`.

  **Constructor-API enforcement (Track 9 named-typed-handle):** a
  dedicated builder `Dag::push_indirect_call_transform(callee:
  PortId, args: Vec<PortId>) -> NodeId` is the only way to
  construct an `IndirectCall` transform. The builder validates at
  construction time that `callee`'s port has an Arrow type and
  that `args.len()` matches the Arrow's declared arity, then emits
  `inputs = [Callee(callee), Arg(args[0]), Arg(args[1]), ...]`.
  Direct field construction (e.g., `TransformNode { target:
  IndirectCall, inputs: vec![Arg(...), Arg(...)] }` with no
  `Callee`) is impossible without bypassing the builder; reviewers
  enforce builder usage in code review the same way `push_node`
  call discipline is enforced today.

  **Cardinality discipline:** the variant requires exactly one
  `Callee` element and zero-or-more `Arg` elements. Without a
  refinement primitive in v3 today, the cardinality is enforced by
  the builder + a debug assertion in the constructor. Future
  refinement: when v3 supports refined enum payload (`{ inputs:
  Vec<TransformInput> | inputs has exactly one Callee }`), the
  cardinality dissolves into the type. For now: builder + assert.

  Emitters render the variant as the target language's first-class
  function-call surface (Rust closure call, Python `()` on a
  callable, etc.) using `inputs[0]` as the callee and `inputs[1..]`
  as args. Per-target `SubstrateAccessorBinding`-style rendering
  is not required because the call is structural (no per-accessor
  carrier), only the call-syntax template per target.

  Adding this variant is the load-bearing substrate change in X1's
  implementation slice. **Dissolution / ratchet receipt:** the
  `IndirectCall` variant is permanent (HO dispatch is a real
  long-term language surface, not staging); no SCAFFOLD lifecycle.
  The variant attaches to existing `TransformTarget` rather than
  introducing a parallel carrier; arity-checking and type-checking
  reuse the current path with the callee's port type as the Arrow
  source.

**Sequencing for the implementation slice:**
1. (L1.a) statically-resolvable case lands FIRST against existing
   `TransformTarget::Callable` — no substrate change. Covers
   `data v: WrapFn = { f: double }; v.f(x)`. This is enough to
   unblock `data complexity_lens: Lens<Int> = { ... }` consumers
   when `complexity_lens.read(d, b)` is called from `fold_lens<C>`
   if and only if `complexity_lens` is a `data` binding (which it
   is — Lens instances are top-level data).
2. (L1.b) runtime-sourced case lands SECOND with the
   `TransformTarget::IndirectCall` extension.
   Required for `fn invoke(lens: Lens<Int>, ...) -> ... = lens.read(...)`
   patterns where the Lens value flows through a parameter rather
   than a static binding. `fold_lens<C>` itself is parametric over
   `Lens<C>`, so its body's `lens.read(...)` dispatch is L1.b —
   `lens` is a function parameter, not a static binding.

**`fold_lens<C>` dependency on this split:** L1.a alone does NOT
unblock `fold_lens<C>`. The body is `fn fold_lens<C>(lens: Lens<C>,
d: Dag) -> DimensionReport<C>` — `lens` is a parameter, so every
`lens.read(...)` / `lens.sequential.op(...)` / `lens.branch(...)` /
`lens.iterate(...)` / `lens.validate(...)` call site is L1.b. The
`IndirectCall` substrate extension is the actual unblocker.

**Test matrix** (acceptance):
- `T1.1` — call on field projection: `data v: WrapFn = { f: double }; fn r(x: Int) -> Int = v.f(x)`. Bootstraps; emit-Rust roundtrip computes `double(x)`.
- `T1.2` — call on parameter field: `fn invoke(w: WrapFn, x: Int) -> Int = w.f(x)`. Same as S1 above; should lower clean.
- `T1.3` — nested field call: `data wraps: { outer: WrapFn } = { outer: { f: double } }; fn r(x: Int) -> Int = wraps.outer.f(x)`. Multi-level field projection in callee.
- `T1.4` — diagnostic on non-Arrow callee: `data v: { x: Int } = { x: 5 }; fn r() -> Int = v.x(7)`. Fails with type error, not parse error.

### Prereq-X2 — call-on-Var (Arrow-typed local) dispatch

**Scope:** if X1 generalizes call-callee to "any Arrow-typed
expression," X2 is implicit in X1 and adds nothing. If the X1
implementation special-cases field projection only (preserving the
identifier-only call grammar elsewhere), X2 is the parallel
extension for Var nodes — `let g = ...; g(x)` where `g` resolves to
an Arrow-typed value.

**Recommendation:** treat as part of X1. The cleanest grammar
generalization (callee = any Arrow-typed expression) covers both;
splitting into X1 (field-call) and X2 (var-call) creates parallel
representations of the same dispatch path.

**Test matrix** (covered if X1 generalizes):
- `T2.1` — call on let-bound name: `fn r(x: Int) -> Int = do { let g = double; g(x) }`. Requires X3 for the explicit `do { ... }` block (per X3 lock above), but the call-site dispatch is X2.
- `T2.2` — call on function parameter: `fn r(g: fn(Int) -> Int, x: Int) -> Int = g(x)`. Pure X2 without block-expression dependency.

### Prereq-X3 — block expressions with let inside `=` bodies

**Scope:** `fn name(...) -> T = do { let v = ...; <expr> }` (or
similar explicit block marker) where the marked block is a
**block expression** (sequence of let-bindings followed by a
final expression) distinct from `{ ... }` record literals.

**Disambiguation strategy (Director-locked 2026-04-30, parent
inbox #1130):** **explicit block syntax.** Reasons:

1. `{ ... }` already has live record literal AND map literal
   meanings in v3 surface; #1248 (Prereq-2) tightened the
   fallback contract around exactly this ambiguity. Adding a
   third "block expression" interpretation behind a heuristic
   first-token lookahead would re-introduce the same parser-
   disambiguation surface area #1248 just stabilized.
2. Heuristic lookahead (parse as block iff first non-whitespace
   token is `let` or another keyword) makes the parse rule
   non-local — adding a future record-literal field syntax that
   starts with a keyword would silently break previously-record
   programs.
3. An explicit marker (e.g., `do { ... }`) is verbose but
   unambiguous and cost-of-change-zero for the parser when new
   block-internal forms land.

Concrete proposal: `do { ... }` keyword. Implementation worker
may pick a different keyword if Director surfaces one — the audit
locks the **explicit-marker discipline**, not the specific token.

**Test matrix:**
- `T3.1` — `fn r(x: Int) -> Int = do { let g = double; g(x) }`. Parses as block, lowers, evaluates correctly.
- `T3.2` — `data v: SomeRecord = { f: ... }`. Continues to parse as record literal (no regression).
- `T3.3` — `fn r(x: Int) -> Int = { let g = double; g(x) }` (without `do`). Fails fail-closed with a parser diagnostic naming the explicit-block requirement; suggests `do { ... }` as the fix surface.

X3 **may not be required** if X1 + X2 land in a way that allows
inlining everything (e.g., `fn r(x: Int) -> Int = double(x)` directly,
without the let-binding intermediate). For `fold_lens<C>` specifically,
the body needs `match workflow_root_port(d) { ... }` and one or
more intermediate values from per-Behavior dispatch — could be
expression-only with chained calls if X1 + X2 are sufficiently
expressive. **Director call:** is X3 required for `fold_lens<C>`,
or is "all-expression body with no let" the target shape?

---

## Mapping to `fold_lens<C>` and lens-instance consumers

Every Lens instance dispatch path the framework requires is a
call-on-field-access:

- **`lens.read(d, b)`** at the per-Behavior fold step (X1).
- **`lens.sequential.op(a, b)`** when accumulating two BindNode
  cost values (X1 with two-level field projection — `lens.sequential`
  is the `Monoid<C>` Conj, `.op` is its Arrow field).
- **`lens.branch(a, b)`** at BranchNode arms (X1).
- **`lens.iterate(body, bound)`** at LoopNode (X1).
- **`lens.validate(d, composed)`** for the aggregate side-condition
  (X1).

`fold_lens<C>` body cannot be authored without X1. Lens instance
authoring (e.g., `data complexity_lens: Lens<Int> = { ... }`) does
not need X1 because Prereq-1 already lowers the field assignments
themselves (`read: complexity_read` resolves the fn-ref). X1 is
strictly the consumer-side gap.

**Updated audit-doc cross-reference:** `docs/design-lens-fold-prerequisites.md`
treated Prereq-1 as the unblocker for "Lens<C> field assignment AND
any consumer dispatch through those fields." That conflated two
distinct surfaces. Prereq-1 unblocked **assignment**; Prereq-X
unblocks **invocation**. Both are required before `fold_lens<C>`
ships.

The lens-fold-prerequisites audit's Prereq-3b (`fold_lens<C>`
machinery) becomes blocked on Prereq-X. The accessor (Prereq-3a,
landed) and the Lens<C> carrier (#1186, landed) are unaffected.

---

## What this audit does NOT do

- Does not modify the parser, lowerer, or emitter.
- Does not author `fold_lens<C>` (blocked on Prereq-X).
- Does not author Lens instances or migrate the four PROXY/STUB
  lenses (independent of Prereq-X — see audit
  `docs/design-lens-fold-prerequisites.md` Prereq-1 + Prereq-2).
- ~~Does not commit to (a) vs (b) for X3 disambiguation — flagged for
  Director.~~ **Updated 2026-04-30:** Director-locked explicit
  block syntax (proposed `do { ... }` keyword); see Prereq-X3
  scope above for rationale.
- Does not size X1 / X2 / X3 implementation effort beyond a rough
  "similar shape to Prereq-2 / #1248." The implementation worker
  scopes precisely.

---

## Acceptance for this audit PR

This document is the deliverable. No code changes; no test
additions; no parser edits. The next dispatch consumes this audit
to scope Prereq-X1 (and X3 if Director confirms) as a separate
parser/lowerer slice.

---

## Cross-references

- `docs/design-lens-fold-prerequisites.md` — original lens-fold
  audit; this Prereq-X is a follow-up that audit didn't catch.
- `src/v3/std/lens.dag` — Lens<C> 6-field carrier (#1186).
- `src/v3/std/dimensions.dag:72-78` — `AnalysisDimension<Carrier>`
  precedent for Arrow-typed Conj fields. Field assignment landed
  via Prereq-1; field invocation never exercised because
  `analyze_symbolic_cost_dimension` data binding was deferred per
  `src/v3/lenses/cost.dag:268-302`.
- `src/v3/std/substrate.dag` — `workflow_root_port` accessor + `WorkflowRoot`
  sum (Prereq-3a, #1232).
- Prereq-1: PR #1230 + #1239.
- Prereq-2: PR #1248.
- Prereq-3a: PR #1232.
