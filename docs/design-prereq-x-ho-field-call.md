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

**Lowerer impact:** the call-site lowering currently dispatches on
the resolved decl-id of the head identifier (Callable target). It
must also handle Arrow-typed value sources — field projection,
let-bound names, function-parameter values — by lowering the
callee to a port and dispatching through a Transform whose target
is `Callable(<resolved-fn-decl-id>)` if the Arrow's underlying decl
is a top-level fn, or via a higher-order Transform target if the
callee is computed at runtime.

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
- `T2.1` — call on let-bound name: `fn r(x: Int) -> Int = { let g = double; g(x) }`. Requires X3 for the `{ let ... }` block, but the call-site dispatch is X2.
- `T2.2` — call on function parameter: `fn r(g: fn(Int) -> Int, x: Int) -> Int = g(x)`. Pure X2 without block-expression dependency.

### Prereq-X3 — block expressions with let inside `=` bodies

**Scope:** `fn name(...) -> T = { let v = ...; <expr> }` where `{ ... }`
is a **block expression** (sequence of let-bindings followed by a
final expression) rather than a record literal. The parser
disambiguation today sees `{` after `=` and commits to record
literal; X3 needs to introduce block-expression-vs-record-literal
disambiguation, likely by:

- **(a)** Looking ahead to the first non-whitespace token after
  `{`: if `let` (or a future `return` / non-record keyword), parse
  block expression; else parse record literal.
- **(b)** Requiring an explicit block syntax (e.g., `do { ... }`)
  to distinguish blocks from record literals.

(a) is the smaller surface change but adds parser look-ahead
complexity. (b) is more verbose for users but unambiguous. Director
should pick.

**Test matrix:**
- `T3.1` — `fn r(x: Int) -> Int = { let g = double; g(x) }`. Parses as block, lowers, evaluates correctly.
- `T3.2` — `data v: SomeRecord = { f: ... }`. Continues to parse as record literal (no regression).
- `T3.3` — disambiguation diagnostic when ambiguous (if (a) chosen): `fn r() -> Int = { let: ... }`. Should parse as block (recovers `let` as keyword) or fail-closed with a clear "block-vs-record" diagnostic.

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
- Does not commit to (a) vs (b) for X3 disambiguation — flagged for
  Director.
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
