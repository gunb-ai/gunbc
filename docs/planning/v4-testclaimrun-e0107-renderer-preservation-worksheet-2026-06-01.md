# v4 TestClaimRun E0107 Renderer Preservation Worksheet - 2026-06-01

> **Status:** DRAFT Section 10.0 WORKSHEET - PATH B blocker for strict M1.
> **Authority request:** silent-crane-669 urgent PATH B brief, PR #4179 / run 26748897345, and main strict M1 family on `fc7930ee4`.
> **Existing authority:** SG-2 `TargetTypeExpressionProjection` worksheet remains the substrate authority for generic instantiation. This worksheet narrows a Rust-emitter consumer boundary that strips already-authored type arguments at generated test-claim cache/signature sites.

---

## Problem Shape

Strict M1 cargo check is blocked by emitted Rust that erases `TestClaimRun<S, A>` type arguments in generated `v4_test_claim_*` modules:

```rust
CACHED.with(|c: &Rc<Vec<Rc<TestClaimRun>>>| c.clone())
```

The source .dag carries the instantiation:

```dag
data run_phase1_nat_semiring_rung6_rust_add_left_identity_emit_equals_eval:
  TestClaimRun<Node, RuntimeValue> = ...
```

Representative source evidence:

- `src/v4/test/claim/nat_semiring/rung_6.dag` imports `TestClaimRun`.
- The rung 6 run declarations use `TestClaimRun<Node, RuntimeValue>`.
- The emitted failure family reports `TestClaimRun` missing two generic arguments.

The v2 compile step is diagnostic-clean; the failure appears when rustc checks the emitted Rust tree. The fix therefore belongs at the Rust type-rendering consumer boundary, not as a source .dag edit or emitted-file patch.

---

## Section 10.0 Worksheet

```text
SG class:
  SG-2 residual / Rust emitter consumer coverage.

Representative emitted failure:
  CACHED.with(|c: &Rc<Vec<Rc<TestClaimRun>>>| c.clone())
  // Rust requires TestClaimRun<Node, RuntimeValue>.

Immediate local patch:
  Add an emitter branch:
    if type_name == "TestClaimRun":
      print TestClaimRun<Node, RuntimeValue>
  or add a generic-arity table:
    TestClaimRun -> 2

Why forbidden:
  - Duplicates generic instantiation facts already carried by the authored type expression.
  - Creates a Rust-emitter-only authority for a source type argument fact.
  - Fails the next generic carrier unless the emitter table grows again.
  - Violates the SG-2 rule forbidding name-keyed Outcome/TestClaimRun/Witness arity patches.

DFS path:
  Source authority:
    - Authored test-claim run declarations in src/v4/test/claim/* use
      TestClaimRun<Node, RuntimeValue>.
    - TestClaimRun<S, A> is the carrier declaration in the evaluation model.
  Existing target-type authority:
    - TargetTypeExpressionProjection remains the modeled generic-instantiation authority.
  Rust emitter consumer boundary:
    - src/v2/05_emit_rust.dag:
      render_rust_decl_type
      render_rust_type_with_applied_binding
      render_node_type
      generated cache/static/function-signature render paths
    - src/v2/stage0/src/v2_compiler_emit_rust.rs must mirror the .dag behavior.

Deepest unsound boundary:
  Helper refactors split generic function-signature rendering from applied-binding
  rendering. When a function or generated cache site has type params, the path can
  switch to render_node_type / raw render_rust_type and drop the __applied_type_args
  or authored child type expression that still contains TestClaimRun<Node, RuntimeValue>.

Single-authority fact:
  Canonical Rust type rendering for a type position must preserve authored/applied
  generic arguments across helper boundaries. A function-signature/cache/static
  consumer may compose with render_rust_type_with_applied_binding,
  render_rust_decl_type, and render_node_type, but it may not replace an instantiated
  type expression with the bare carrier name.

Stack-safety companion fact:
  Any render_rust_decl_type recursion repair must either keep the recursive branch
  inside the bounded stack-growth receipt or provide an equivalent modeled bounded
  stack-growth receipt. Removing stacker coverage to fix generics is not an
  acceptable tradeoff.

Systemic fix:
  Refactor the Rust type-rendering helper boundary so generated cache/statics and
  function signature type positions always render the canonical instantiated type
  expression first, then layer Rust ownership/decl syntax. Stage0 must be regenerated
  or kept in exact parity with src/v2/05_emit_rust.dag.

Non-goals:
  - TestClaimRun-specific arity, name, or module branches.
  - Broad SG-2 implementation beyond this helper-boundary consumer slice.
  - Editing emitted v4 Rust files.
  - Treating E0107 count reduction as acceptance without falsification.
  - Removing stacker / maybe_grow coverage from recursive type rendering.

Falsification probe:
  Add a new generic test carrier, e.g. ProbeClaimRun<X, Y>, and use it in:
    (a) generated cache/static type,
    (b) function parameter,
    (c) function return,
    (d) closure annotation if the path exists in the same generator.
  The emitted Rust must preserve ProbeClaimRun<Node, RuntimeValue> in all covered
  sites without an emitter branch for ProbeClaimRun.

Required local receipt:
  V4_M1_RUST_EMIT_PROBE_STRICT=1 cargo/check path passes locally before push.

Metric allowed only as secondary:
  E0107 TestClaimRun movement in strict M1.
```

---

## Dispatch Gate

No implementation worker should land changes for this PATH B family until the worksheet is ratified by the Modeling DFS arbiter. After ratification, an implementation worker may restructure PR #4179 or open a new PR, but the implementation must stay inside the single-authority boundary above.

Worker must:

- Preserve generic type arguments by composing with the canonical type renderer instead of special-casing `TestClaimRun`.
- Keep `src/v2/05_emit_rust.dag` and `src/v2/stage0/src/v2_compiler_emit_rust.rs` in parity.
- Include a non-`TestClaimRun` generic falsification probe.
- Run strict M1 locally before pushing.

Worker must not:

- Add name-keyed generic arity tables.
- Patch generated Rust output.
- Broaden into FreeMonoid E0391, E0308 collection projection, or other M1 residual bands without a separate worksheet.

---

## Arbiter Checklist

- [ ] Single-authority fact accepted: generic type rendering preserves authored/applied instantiation across helper boundaries.
- [ ] Spot-fix forbidden: no `TestClaimRun` arity/name branch.
- [ ] Stack-safety condition accepted for `render_rust_decl_type` recursion.
- [ ] Falsification probe accepted.
- [ ] Implementation dispatch authorized.
