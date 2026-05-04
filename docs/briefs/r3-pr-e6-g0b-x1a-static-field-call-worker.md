# R3 PR-E E6-G0b — Prereq-X1.a Static Call-on-Field-Access Worker Brief

**Status:** docs-only worker brief decomposing the E6-G0b implementation slice
into a single landable PR. Authored after Substrate E6-G0a
(`lens_value_generic_conj_field_substitution_lands`, #1640) merged.

**Parent authorities:**
[`r3-pr-e6-g0-first-gate-narrowing.md`](r3-pr-e6-g0-first-gate-narrowing.md),
[`../design-prereq-x-ho-field-call.md`](../design-prereq-x-ho-field-call.md)
§Prereq-X1.a + L1.a + T1.1,
[`r3-evaluator-dispatch.md`](r3-evaluator-dispatch.md) §E6.

**Non-goals:** does not implement `fold_lens`, does not author lens
instances, does not edit `Lens<C>` / `Witness<C>` / `OptionalDiagnostic` /
`DimensionReport<C>` / `TransformTarget`, does not collapse
`TransformNode.target+inputs` into `TransformDispatch`, does not introduce
`Indirect` / `ArrowPortRef`, does not unblock parameter-callee (Prereq-X1.b),
does not unblock call-on-Var (Prereq-X2), does not unblock block-expression
bodies (Prereq-X3), does not change the evaluator (E6-G0c), and does not
narrow or remove `lens_apply.rs::fold_lens_over_reflected_program`.

**#1532 debt receipt:** Per-PR dissolution gate required (the slice touches
hand Rust under `src/v3/`); no census shift expected (modifies existing
parser/lowerer pathways and adds tests).

---

## Why this is its own PR

The E6-G0 narrowing brief sequences E6-G0b strictly between Substrate G0a
(now landed in #1640) and Evaluator G0c. The X1.a slice touches four files
crossing two carriers (parse surface schema, parse algorithm, lowerer
dispatch, ratchet test) and goes through `regen_parse` ceremony. Bundling
this with G0c evaluator work would mix substrate-schema regen with runtime
behavior changes; bundling with G0a would have hidden the grammar/lowering
work behind a substrate diff. Keep them separate.

## Scope: only X1.a (static `data`-binding callee)

After this slice, exactly the following shape lowers to a static
`TransformTarget::Callable(decl_id_of_function)` call site:

```dag
fn double(n: Int) -> Int = n + n
type WrapFn { f: fn(Int) -> Int }
data v: WrapFn = { f: double }
fn r(x: Int) -> Int = v.f(x)
```

Lowering walks:

1. The callee path `v.f` resolves: head `v` is a top-level `data` binding
   with `ValueBody::Structural`; field `f` is `FieldValue::Reference(decl_id_of_double)`.
2. The call site emits `TransformTarget::Callable(decl_id_of_double)` with
   the existing static-callee dispatch path (no new substrate variant).

After this slice, **all** of the following continue to fail closed exactly
as before:

- Parameter-callee `fn invoke(w: WrapFn, x: Int) -> Int = w.f(x)` — pinned
  by `prereq_x_call_on_field_access_ratchet_test::x1_direct_field_call_blocked`.
  This is Prereq-X1.b territory; lowering must produce a typed
  diagnostic (not a parser error) once X1.a parses the syntax. See
  "Ratchet update" below.
- Block-expression body `fn invoke(w, x) -> Int = { let g = w.f; g(x) }`
  — pinned by `x3_brace_block_with_let_head_blocked`. Untouched.
- Parametric `fold_lens<C>` body `fn fold_lens<C>(lens: Lens<C>, …) =
  lens.read(…)`. Untouched (still blocked by Prereq-X1.b).
- Non-Arrow callee `data v: { x: Int } = { x: 5 }; fn r() -> Int = v.x(7)`
  — must lower-fail with a typed diagnostic naming the field as non-Arrow,
  not pass silently and not surface as a parse error.

## Implementation plan

### 1. Parse-surface schema (`src/v3/std/parse_surface.dag`)

Add one variant alongside the existing `Path` and `Call`:

```text
| PathCall {
    segments: List<String>
    segment_spans: List<SourceSpan>
    args: List<SurfaceExpr>
    span: SourceSpan
  }
```

Rationale: collapsing `Path` + LParen into the existing `Call { target:
String, … }` would either smuggle a dotted path into a single `String`
target (an opaque-string anti-pattern flagged by
`feedback_opaque_strings_attract_heuristics`) or require widening
`Call.target` to a path-shape (a wider blast radius than this slice
warrants). A sibling variant is the smallest honest carrier.

Regenerate `src/v3/compiler/src/parse_surface_generated.rs` via the
existing parse-surface emitter.

### 2. Parser algorithm (`src/v3/compiler/parse_parser_body.txt`)

In `parse_ident_expr`, after the `while matches!(self.peek().kind,
TokenKind::Dot)` segment-collection loop, peek for `TokenKind::LParen`. If
present, parse `parse_call_args()` and emit `SurfaceExpr::PathCall { … }`
instead of `SurfaceExpr::Path { … }`. Otherwise keep the existing `Path`
emission.

Regenerate `src/v3/compiler/src/parse_generated.rs` via `regen_parse`.

Touch points outside `parse_ident_expr`:

- `parse_pipe_call`'s `target_expr` match (`parse_parser_body.txt:1471-1505`)
  must add a `SurfaceExpr::PathCall { span, .. }` arm. Decision: `|>` does
  not currently support path-targets, so emit the same parse-error shape as
  the existing `SurfaceExpr::Path` arm — "dotted paths are not callable in
  the current surface grammar." A future X2 or pipe-target slice can lift
  this; not in scope for X1.a.
- Any `unreachable!("parse_ident_expr only returns Var, Call, Path, or
  VariantRecord")` claim must be updated to include `PathCall`.

### 3. Lowerer (`src/v3/compiler/src/lower.rs`)

Add a `SurfaceExpr::PathCall` lowering arm. The arm:

1. Resolves `segments[0]` as a top-level decl. If it is not a `data`
   binding with `ValueBody::Structural`, emit a typed
   `Diagnostic::ResolveError` naming the unsupported head and noting that
   parameter-callee dispatch (X1.b) requires the substrate
   `TransformDispatch::Indirect` collapse.
2. Walks remaining `segments[1..]` through nested structural-body field
   projection. For each segment:
   - if the field's typed value is `FieldValue::Reference(decl_id)` to a
     top-level function, the walk continues toward call-site emission;
   - if the field's typed value is a non-Arrow value, emit a typed
     `Diagnostic::TypeError` naming the segment and field type;
   - if the field is missing, emit the existing structural-field-missing
     diagnostic.
3. For the resolved callee `decl_id`, validate arity and per-position types
   against the callee's Arrow signature, then emit
   `TransformTarget::Callable(decl_id)` via the existing static-callee
   dispatch entry point (`Dag::push_callable_transform`-equivalent on
   current `main`).
4. No new `TransformTarget` variant. No `TransformDispatch` collapse.

### 4. Ratchet update (`src/v3/compiler/tests/integration/prereq_x_call_on_field_access_ratchet_test.rs`)

Convert exactly one ratchet (`x1_direct_field_call_blocked`) to a positive
parse-and-lower assertion **for the static-data form**, and add a new
ratchet pinning the parameter-callee form red:

- New `x1a_static_data_field_call_lowers_to_callable`: `data v: WrapFn = {
  f: double }; fn r(x: Int) -> Int = v.f(x)` parses, lowers, and the
  resulting `Dag` contains a `TransformTarget::Callable(decl_id_of_double)`
  transform — assert by structural inspection of `Dag` nodes, not by
  emit-roundtrip output. (Roundtrip-evaluation is E6-G0c scope.)
- New `x1a_non_arrow_field_call_diagnostic`: `data v: { x: Int } = { x: 5
  }; fn r() -> Int = v.x(7)` lowers to a typed diagnostic naming `v.x` as
  non-Arrow. Asserts on diagnostic kind/message, not parse error.
- Renamed `x1b_parameter_field_call_blocked` (was
  `x1_direct_field_call_blocked`): `fn invoke(w: WrapFn, x: Int) -> Int =
  w.f(x)` parses cleanly **but** lowering returns a typed
  `Diagnostic::ResolveError` whose message names the parameter-callee /
  X1.b prerequisite. The diagnostic-shape assertion replaces the existing
  parse-error `LParen` assertion.
- `x3_brace_block_with_let_head_blocked` and
  `control_arrow_typed_field_decl_parses` are unchanged.

### 5. `complexity_lens.read(d, b)` follow-up fixture

If the post-#1640 `Lens<Int>` structural-value path supports a minimal
`data complexity_lens_seed: Lens<Int> = { … }` fixture authoring at HEAD,
add an `x1a_complexity_lens_read_lowers_to_callable` integration test that
parses and lowers `complexity_lens_seed.read(d, b)` to
`TransformTarget::Callable(decl_id_of_complexity_read)`. **If #1640 still
defers a fully-typed `Lens<Int>` value (e.g., one or more of `read`,
`branch`, `iterate`, `validate`, `sequential` cannot yet be authored as
top-level function references because of an unrelated lens-corpus gap), do
NOT scaffold a placeholder lens** — instead, in the PR body name the
remaining lens-authoring blocker separately from X1.a, link the relevant
lens file (`src/v3/lenses/complexity.dag`), and ship X1.a with the
`WrapFn`/`double` synthetic fixture only. The X1.a parser/lowerer slice is
honest on its own; the lens-corpus integration is a downstream follow-on.

## Acceptance

- `cargo test -p v3-compiler` and `cargo test -p v3-compiler-tests` green.
- `cargo fmt --all --check` and `cargo clippy --all-targets -- -D warnings`
  green.
- Ratchet test changes match §4 above exactly.
- Stage-0 strict-compile diagnostic ratchet (`v2_strict_compile_diagnostic_count`)
  unchanged.

## STOP+PING

- if the slice would require collapsing `TransformNode.target+inputs` →
  `TransformDispatch` or introducing `Indirect` / `ArrowPortRef`;
- if the slice would require widening `TransformTarget` with a new variant;
- if the lowerer would require `eval_node` / `eval_port` / evaluator
  changes (E6-G0c);
- if the parameter-callee form (X1.b) would parse-and-lower to anything
  other than a typed lowering diagnostic;
- if the `complexity_lens` fixture forces authoring a non-honest lens value
  (e.g., a placeholder `read` returning `Witness<Int>::Inhabits(0)` from a
  host default).

## Cross-references

- `src/v3/std/parse_surface.dag` around `SurfaceExpr` (variant addition).
- `src/v3/compiler/src/parse_surface_generated.rs` (regen target).
- `src/v3/compiler/src/parse_generated.rs` (regen target).
- `src/v3/compiler/parse_parser_body.txt` around `parse_ident_expr` and
  `parse_pipe_call`'s `target_expr` match (parser-body splice).
- `src/v3/compiler/src/lower.rs` (PathCall arm; static field projection).
- `src/v3/compiler/src/dag.rs` around the existing static-callee dispatch
  builder (no shape change).
- `src/v3/compiler/tests/integration/prereq_x_call_on_field_access_ratchet_test.rs`
  (split into x1a-positive, x1a-non-arrow-diagnostic, x1b-parameter-blocked).
- `src/v3/lenses/complexity.dag` (downstream consumer; not authored here).
