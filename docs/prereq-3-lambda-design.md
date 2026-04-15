# Prereq 3 — Lambda parser + lowering (design note)

> **Parent doc:** `docs/substrate-reflection-design.md` §11
> (the prerequisite slate that names Prereq 3). This note is the
> self-contained design for the lambda-specific work, intended
> to be picked up directly by warm-elk (or whoever implements
> Prereq 3) without further back-and-forth.
>
> **Prior art:** v3-spec.md §Principle 5 ("Lambda = Bind + Define
> transform. Not a special behavior."), `docs/v3-validation-experiments.md`
> Experiment 1 (v2-level validation of the approach and the
> closure/binding semantics discovery).

---

## §1. What Prereq 3 is (and isn't)

**Prereq 3 is a parser and lowering extension.** It adds the
surface form `|params| body` to v3's grammar and lowers each
lambda to an ordinary `Bind` declaration with captures as
explicit additional input edges.

**Prereq 3 is NOT a substrate extension.** Per v3-spec.md
§Principle 5 and Experiment 1's validated partial pass, the
substrate already has everything lambdas need: `Bind` carries
parameters and a body sub-DAG; captures are typed edges to
values in the enclosing scope. There is no "lambda" concept in
the substrate — there is only `Bind` with carefully-chosen
parameters and input edges. Prereq 3 is entirely grammar-to-Bind
translation work.

**Prereq 3 is NOT runtime closure semantics.** v3 has no
runtime environment, no heap-allocated closure cells, no
capture-by-reference. Every capture is captured **by value**
at lambda-construction time. The captured value flows through
the substrate as a typed edge; the lambda body reads it as an
ordinary input. This matches how v3 already handles every
other non-lambda function's inputs.

**First-landing implementation scope.** Lambdas lower when the
construction site already provides an expected function type
(for example a function-typed argument position or an annotated
`let` binding). An unannotated standalone lambda like
`let f = |x| x` is not guessed into a function type implicitly;
it fails closed until function-value inference is designed
explicitly.

**Prereq 3 does NOT land the callback rule tests** for
Experiment 1's closure-in-Loop semantics. That's a lens-level
concern (ownership lens treats captures as fan-out=N when the
Bind's result flows into a Loop body; termination lens treats
self-calls as bounded by the Loop, not the enclosing
recursion). The callback rule is tested in a follow-up PR
once v3 has ownership + termination lenses; Prereq 3 lands the
syntax and the lowering, not the callback-rule enforcement.

---

## §2. Surface form

**Syntax: `|ident (, ident)*| expr`**. Rust-style. Chosen
because:

1. `|` is currently unused in v3's token set; no parse ambiguity
   with anything else.
2. Shorter than `(x, y) => body`, which overlaps with the
   `fn(T, T) -> T` syntax in type position — picking `|x| body`
   keeps expression-position lambdas grammatically distinct
   from type-position function types.
3. Avoids C++ `[](x) { body }` which would collide with v3's
   existing block-body `{ … }` syntax for function bodies.

**Parser additions:**

```rust
// In parse.rs SurfaceExpr enum
pub enum SurfaceExpr {
    // … existing variants …
    Lambda {
        params: Vec<String>,           // declared parameter names (by name, resolved at lower time)
        body: Box<SurfaceExpr>,        // the lambda body — an ordinary expression
        span: SourceSpan,
    },
}
```

**Tokenizer status:** `|` already tokenizes as a standalone
token (used by the `|>` pipe operator in Prereq 5, and by sum-
type declarations at the item level). No tokenizer change
needed; the parser disambiguates `|` based on position.

**Parser rule location:** inside `parse_primary`, before the
existing identifier / literal dispatch. When the parser sees
`|` in expression position with a following identifier, it
enters the lambda parse path:

```
LAMBDA ::= `|` PARAMS `|` EXPR
PARAMS ::= (IDENT (`,` IDENT)*)?
```

**Empty parameter list:** `||body` is a zero-argument lambda.
Parses as `Lambda { params: [], body: <body>, span }`. Useful
for thunks / nullary callbacks.

**Parse ambiguity notes:**

- `|` inside an expression (`a | b` as bitwise-or) is NOT
  currently a v3 operator. If v3 grows bitwise operators in
  the future, the lambda rule needs lookahead — "if the next
  token after `|` is an identifier followed by `,` or `|`,
  it's a lambda; otherwise it's bitwise-or." For now, no
  ambiguity.
- `|` as a sum-type delimiter appears only in item position
  (`type Foo = A | B | C`), not in expression position. No
  collision.

---

## §3. Free-variable analysis

**Goal.** Given a parsed `SurfaceExpr::Lambda { params, body }`,
identify which variables referenced in `body` are NOT bound by
`params` and NOT bound by the built-in scope (functions,
declared types, etc.). Those are the captures.

**Algorithm (recursive walk):**

```
free_vars(expr, bound_set) -> Set<String>

  match expr {
    Literal(_)                 => {}
    Var(name)                  => if name not in bound_set then { name } else {}
    Lambda { params, body }    => free_vars(body, bound_set ∪ params)
    Call { target, args }      => free_vars(target, bound_set) ∪ ⋃ free_vars(args, bound_set)
    If { cond, then, else }    => free_vars(cond) ∪ free_vars(then) ∪ free_vars(else)
    Match { scrutinee, arms }  => free_vars(scrutinee) ∪ ⋃ free_vars(arm.body, bound_set ∪ arm.binding)
    Let { name, value, body }  => free_vars(value, bound_set) ∪ free_vars(body, bound_set ∪ { name })
    // … one arm per SurfaceExpr variant …
  }
```

**Input:** the lambda's body expression, plus the lambda's
own declared params as the initial `bound_set`.

**Output:** the set of variable names that are free in the
body relative to the lambda's params. These are candidate
captures.

**Resolution step:** for each candidate, look it up in the
**outer scope** at the lambda's construction site. Three
outcomes:

1. **Resolves to a declared function / type / constant.** Not
   a capture — it's a top-level reference. No input edge
   needed; the lambda's body references the declaration
   directly.
2. **Resolves to a local variable (let-binding or enclosing
   parameter).** THIS IS A CAPTURE. Must be materialized as
   an additional input edge on the lambda's Bind declaration.
3. **Does not resolve.** Fail-closed diagnostic: "unresolved
   identifier in lambda body."

**Nested lambdas and transitive captures.** When a lambda body
contains another lambda, the inner lambda's captures may
reference variables bound in the outer lambda. Example:
`|x| (|y| x + y)`. The inner `|y| x + y` has `x` as a free
variable; `x` is bound by the outer lambda's params.

Handling: free-variable analysis is recursive. When processing
the outer lambda, the inner lambda's free variables (minus its
own params) become part of the outer's body's free variables.
The outer lambda captures `x` if `x` is free in ITS body after
accounting for the inner lambda's bindings. In the example,
the outer lambda captures nothing (its body is a lambda
expression that binds `x` via the outer's params); the inner
lambda captures `x` via its parameter inheritance.

**Scope walking.** Variables in Rust's closest-scope-wins
pattern. If a lambda shadows an outer variable (`|x| (|x|
x)(x)` — the inner `|x|` shadows the outer's `x`), the
innermost binding wins. The free-variable walk tracks
`bound_set` as a set of names; shadowing is handled by adding
the shadowing name to the set.

**Determinism.** The capture list must be in a **stable order**
across compilations. Use alphabetical order of captured names,
or order-of-first-occurrence in the body walk. Either is fine
as long as it's deterministic. Pick one and commit.

---

## §4. Lowering

**Goal.** Transform `SurfaceExpr::Lambda { params, body }` into
an ordinary `Bind` declaration whose parameter list includes
both the declared params AND the captures, and whose body is
the lowered body expression.

**Two outputs from lowering a lambda:**

1. **A new `Declaration`** representing the lambda's function
   shape. The declaration's `connective` is an `Arrow` with
   `inputs = [declared params + captures]` and `output = body's
   type`. The declaration has a synthetic name (e.g.,
   `__anon_lambda_<span>` or similar — name doesn't matter
   because everything is `DeclarationId` at consumption time).
2. **A new `Bind` at the construction site** (the place where
   the lambda expression appears in the enclosing function's
   body). The Bind's `target` references the new declaration
   from step 1; the Bind's `inputs` carry the captured values
   (ports from the enclosing scope that the free-variable
   analysis identified).

**Example:**

```
// Surface:
fn outer(x: Int, y: Int) -> Int {
  let f = |z| x + z   // lambda captures x (y is unused)
  f(y)
}

// After lowering:
//
// Declaration 1 — the lambda's function shape:
// __anon_lambda_4_12: Arrow { inputs: [x_port_from_capture, z_param], output: Int }
//   body: Bind { value: <lowered x + z> }
//
// Bind at the lambda's construction site:
// f = Bind { target: __anon_lambda_4_12, inputs: [x_port] }
//     // only x_port is an input; z_param comes from the call site
//
// Call site `f(y)`:
// Transform { target: Callable(__anon_lambda_4_12_bind), inputs: [y_port] }
//     // the Bind carrying the captured x is callable with the remaining declared params
```

**The capture-port vs call-arg distinction.** This is the
subtle part. The lambda's declaration has inputs =
`[captures + declared_params]` (captures come first by
convention — pick and commit). At the lambda's construction
site, the Bind provides the capture inputs. At the lambda's
call site, the caller provides the declared-param inputs.
**Both contribute to the same Arrow's input list**, but at
different times and from different sources.

**Why this works.** The substrate doesn't distinguish "capture
slots" from "declared parameter slots" — they're both ordinary
Arrow inputs. A lens walking the resulting Bind sees params =
`[all of them]` with no way to tell which are captures and
which are declared. That's exactly what the thesis wants: "a
lambda is a function; no special handling."

**Implementation sketch:**

```rust
fn lower_lambda(
    params: Vec<String>,
    body: Box<SurfaceExpr>,
    span: SourceSpan,
    lowering_ctx: &mut LoweringContext,
) -> Result<PortId, Diagnostic> {
    // 1. Compute captures via free-variable analysis
    let captures: Vec<String> = free_vars(&body, params.iter().cloned().collect());

    // 2. Resolve each capture to a PortId in the enclosing scope
    let capture_ports: Vec<PortId> = captures
        .iter()
        .map(|name| resolve_in_enclosing_scope(name, lowering_ctx))
        .collect::<Result<_, _>>()?;

    // 3. Allocate parameter ports for each declared param
    let param_ports: Vec<PortId> = params
        .iter()
        .map(|name| allocate_param_port(name, lowering_ctx))
        .collect();

    // 4. Lower the body with both captures and declared params in scope
    let mut body_ctx = lowering_ctx.new_child_scope();
    for (name, port) in captures.iter().zip(capture_ports.iter()) {
        body_ctx.bind(name.clone(), *port);
    }
    for (name, port) in params.iter().zip(param_ports.iter()) {
        body_ctx.bind(name.clone(), *port);
    }
    let body_port = lower_expr(*body, &mut body_ctx)?;

    // 5. Build the synthetic Arrow declaration
    //    inputs: capture_ports + param_ports
    //    output: type of body_port
    //    body: Bind { value: body_port }
    let lambda_decl_id = allocate_synthetic_arrow_declaration(
        &capture_ports,
        &param_ports,
        body_port,
        span.clone(),
    );

    // 6. Allocate the Bind at the construction site
    //    target: lambda_decl_id
    //    inputs: capture_ports (the declared params are NOT inputs here;
    //            they're filled in at call time)
    let bind_node_id = allocate_bind_at_construction_site(
        lambda_decl_id,
        capture_ports,
        lowering_ctx,
    );

    // 7. Return the PortId of the Bind's output (the lambda value)
    Ok(bind_port(bind_node_id))
}
```

**The call site.** When the lambda's result is called via
`f(y)` or `f(y, z)`, the call site is a normal `Transform`
with `target` pointing at the lambda's Bind, and `inputs`
being the declared-param values from the call. The captures
are already baked into the Bind; the call site only supplies
the remaining inputs.

**Interaction with template instantiation (Prereq 0, 1c).**
When a lambda is passed to a higher-order function like
`filter(xs, |x| x > 0)`, the template instantiation mechanism
from Prereq 0 binds `P := <lambda's DeclarationId>` as a
template argument. Inside `filter`'s body, `p(x)` resolves
through the SubstStack to the lambda's declaration, and the
call becomes a normal Transform with `target: Callable
(lambda_decl_id), inputs: [x_port]`. The captures already
flowed into the lambda's Bind at the call-site-that-created-
the-lambda (the `filter(xs, ...)` call); inside `filter`'s
body, only the declared params are supplied.

---

## §5. Test plan

Five categories of tests, each exercising a specific aspect:

### §5.1 Basic lambda — no captures

```
fn test_identity() -> Int {
  let f = |x| x
  f(42)
}
// Expected: returns 42
```

Assert: compiles, evaluates, returns the correct value.
Assert: the lowered Bind has no input edges from the enclosing
scope (zero captures).

### §5.2 Single capture

```
fn test_capture(x: Int) -> Int {
  let add_x = |y| x + y
  add_x(5)
}
// Called as test_capture(10), expected: returns 15
```

Assert: compiles, evaluates, returns 15.
Assert: the lowered `add_x` Bind has exactly one input edge
(the `x` port from the enclosing scope).

### §5.3 Multiple captures — deterministic order

```
fn test_multi(a: Int, b: Int, c: Int) -> Int {
  let f = |d| a + b + c + d
  f(4)
}
// Called as test_multi(1, 2, 3), expected: returns 10
```

Assert: compiles, evaluates, returns 10.
Assert: the lowered `f` Bind has three input edges in
**deterministic order** (alphabetical, or first-occurrence —
whichever is committed). Tests are order-sensitive; the commit
message names the ordering choice.

### §5.4 Nested lambdas with transitive capture

```
fn test_nested(x: Int) -> Int {
  let outer = |y| {
    let inner = |z| x + y + z
    inner(1)
  }
  outer(2)
}
// Called as test_nested(3), expected: returns 6
```

Assert: compiles, evaluates, returns 6.
Assert: the `inner` lambda captures both `x` (from the
enclosing function) and `y` (from the outer lambda). The walk
through nested lambdas correctly threads transitive captures.

### §5.5 Lambda passed to higher-order function (integration with Prereq 0)

```
fn test_higher_order() -> Int {
  fold([1, 2, 3], 0, |acc, x| acc + x)
}
// Expected: returns 6
```

Assert: compiles, evaluates, returns 6.
Assert: the lambda is template-instantiated correctly inside
`fold`'s body via the Prereq 0 mechanism. The `p(acc, head)`
call inside fold resolves through SubstStack to the lambda's
declaration.

**This test depends on both Prereq 3 AND Prereq 0 being
complete**, so it's an integration test rather than a Prereq
3 unit test. Include it in Prereq 3's test suite so the
dependency is visible and the test fails with a clear message
until Prereq 0 lands.

### §5.6 Shadowing

```
fn test_shadow(x: Int) -> Int {
  let f = |x| x * 2  // inner x shadows outer x
  f(5)
}
// Called as test_shadow(100), expected: returns 10 (NOT 200)
```

Assert: compiles, evaluates, returns 10.
Assert: the lambda's `x` parameter shadows the enclosing
scope's `x`, so there is no capture. The lowered Bind has zero
input edges from the enclosing scope.

### §5.7 Fail-closed — unresolved identifier in lambda body

```
fn test_unresolved() -> Int {
  let f = |x| x + nonexistent_variable
  f(5)
}
// Expected: parse or lowering fails with "unresolved identifier" diagnostic
```

Assert: the diagnostic names the specific identifier
(`nonexistent_variable`) and points at the span inside the
lambda body.

---

## §6. Open questions

These are implementation-detail questions that should be
answered during the implementation pass, not blockers.

### §6.1 Capture ordering

Alphabetical vs first-occurrence-in-body-walk. Either is
deterministic; pick one. My weak preference: **first-occurrence-
in-body-walk**, because it mirrors the natural reading order
of the lambda body and makes the capture list stable relative
to source edits that don't change the body's traversal order.
Alphabetical is simpler and arguably more robust under body
rewrites. Either is fine.

### §6.2 Synthetic declaration naming

Every lambda produces a new Declaration. It needs a name (or at
least an identifier) for diagnostic purposes. Options:

- **(a) `__anon_lambda_<line>_<col>`** — based on source position
- **(b) `__anon_<counter>`** — based on allocation order
- **(c) Unnamed, identified only by `DeclarationId`** — no name
  field populated; diagnostics use `SourceSpan` for display

My weak preference: **(a)**. Gives readable diagnostics ("lambda
at line 42 column 15") without depending on a global counter.
Sensitive to source-position shifts but that's OK because the
declaration's identity doesn't depend on the name; only
diagnostics do.

### §6.3 Lambda body as single expression vs block

v3's existing `fn` form allows both `fn f(x) = expr` (expression
body) and `fn f(x) { statements; expr }` (block body with let-
bindings). Lambdas should support both.

- Single-expression form: `|x| x + 1`
- Block form: `|x| { let y = x + 1; y * 2 }`

Both are reasonable. Block form lets lambda bodies contain
let-bindings and multiple statements, matching what
block-bodied functions can do. Parsing adds a `{ ... }`
branch to the body parse rule.

**Recommendation:** support both. Block form is straightforward
if the parser already handles block bodies in `fn`
declarations (which it does in v3 today — see `parse_fn_item`).
Reuse that machinery.

### §6.4 Zero-argument lambdas

`|| body` — a thunk. Should parse. Captures work the same way
as any other lambda (free-variable analysis identifies which
enclosing variables the body references). No declared params;
the call site passes no arguments.

**Recommendation:** support. It's free once the empty-params
case is handled in the parser, and thunks are legitimately
useful for deferred evaluation.

### §6.5 Lambda return type

v3 type inference should deduce the lambda's return type from
the body expression's type. No surface syntax for explicit
return-type annotation is needed in Prereq 3 (can be added
later if required).

---

## §7. Acceptance criteria

- [x] `SurfaceExpr::Lambda { params, body, span }` added to
      `parse.rs`
- [x] Parser rule `|ident (, ident)*| expr` accepts in
      expression position, produces `SurfaceExpr::Lambda`
- [ ] Zero-argument lambda (`|| body`) accepts
- [ ] Block-body lambdas (`|x| { let y = ...; y }`) accept
      if §6.3 is addressed
- [x] `free_vars` walker in `lower.rs` identifies captures
      correctly for all test cases in §5
- [x] `lower_lambda` produces a synthetic Arrow declaration
      with correct input ordering (captures + declared params)
- [x] `lower_lambda` produces a Bind at the construction site
      with captures as input edges
- [ ] Tests §5.1 through §5.7 all pass (modulo §5.5 which
      depends on Prereq 0)
- [ ] Clippy clean
- [x] No regressions on existing v3 tests
- [ ] Commit message names the capture-ordering choice (§6.1)
      and the synthetic-naming choice (§6.2) for future review

---

## §8. What Prereq 3 does NOT include

Explicitly out of scope:

- **Callback rule enforcement.** The ownership-fan-out = N and
  termination-bounded-by-Loop semantics from Experiment 1 are
  lens concerns. Prereq 3 lowers lambdas correctly; the lenses
  check the Loop-body flow property in a separate PR.
- **Lambda return type annotations.** Explicit `-> Type` syntax
  on lambdas can be added later if needed.
- **`move` captures or explicit capture lists.** Rust's
  `move |x| body` and `|x: &T| body` forms are NOT supported.
  v3 captures are always by-value at construction time; no
  capture mode selection is needed.
- **Recursive lambdas.** `let rec f = |x| if x == 0 then 1 else
  x * f(x - 1)` — self-referential lambdas. v3's substrate
  supports recursion via `Loop`, not via self-reference. If a
  recursive function is needed, declare it as `fn rec_f(x)
  = ...` instead of as a lambda. Lambdas are strictly
  non-recursive in Prereq 3.
- **The `|>` pipe operator.** That's Prereq 5. Independent of
  lambda work.

---

## §9. Dependency map

**Prereq 3 depends on:**

- Nothing in the prereq slate. It can land first, in parallel,
  or after other prereqs. Its scope is entirely self-contained
  within parser + lowering changes.

**Prereq 3 is depended on by:**

- **Prereq 4 (`list.dag` ships)** — `list.dag` uses lambdas in
  `fold`/`map`/`filter` bodies (e.g., `fold(list, 0, |acc, x|
  acc + x)`). Prereq 4 cannot load cleanly without lambdas.
- **L2 consumer migrations (complexity, ownership, effects,
  trace)** — v2's analysis code uses lambdas extensively. The
  `.dag` versions will use lambdas at the same sites.
- **L3 pipeline stages in `.dag`** — lowering, inference, and
  emit are all likely to use lambdas.

**Integration with Prereq 0:** Prereq 3 doesn't depend on
Prereq 0, but test §5.5 exercises the integration. Land them
in either order; the integration test becomes green when both
have landed.

---

## §10. Estimated scope

**Parser:** ~50-100 lines. New variant + new parse rule + a
handful of test-source snippets.

**Free-variable analysis:** ~150-250 lines. Recursive walker
over SurfaceExpr, tracking `bound_set`, returning a deterministic
capture list. Tests for each SurfaceExpr variant.

**Lowering:** ~200-300 lines. Scope management for the lambda
body, allocation of the synthetic Arrow declaration, wiring
captures + declared params into the Bind's inputs, and the
return port.

**Tests:** ~200-400 lines. Six to ten test cases covering §5's
categories, plus edge cases that surface during implementation.

**Total: ~600-1000 lines.** Medium scope. Comparable to Prereq
2 in size but with more subtle design questions (free-variable
analysis, capture ordering, scope walking).

---

## §11. Open questions for reviewer input

Before implementation starts, the implementer should confirm
the following calls with a reviewer:

1. **§6.1 capture ordering choice.** First-occurrence or
   alphabetical? Either is defensible.
2. **§6.2 synthetic naming.** `__anon_lambda_<line>_<col>`,
   `__anon_<counter>`, or unnamed? The doc recommends position-
   based naming.
3. **§6.3 block-body lambdas.** Support at Prereq 3 time, or
   defer to follow-up? Recommendation: support at Prereq 3 time,
   it's free given v3's existing block-body machinery.
4. **§6.4 zero-argument lambdas.** Support? Recommendation:
   yes, it's free.
5. **§8 `move` semantics.** Confirm that by-value capture is
   the only supported mode. Recommendation: yes, matches v3's
   immutable-ports substrate.
6. **§8 recursive lambdas.** Confirm that recursion via lambda
   self-reference is NOT supported, and the substitute is
   `Loop` via named `fn` declarations. Recommendation: yes.

If any of these need redirection, make the call before
implementation; changing course mid-implementation is much
more expensive than deciding upfront.

---

**This design note is self-contained.** An implementer should
be able to pick it up, confirm the §11 open questions, and
start implementation without further design discussion. Open
questions that surface during implementation graduate to
comments on the implementation PR for reviewer attention.
