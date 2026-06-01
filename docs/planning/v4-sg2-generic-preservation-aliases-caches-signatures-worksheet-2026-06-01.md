# v4 SG-2 Generic Preservation Across Aliases / Caches / Signatures — Next-Wave Worksheet — 2026-06-01

> **Status:** DRAFT Section 10.0 WORKSHEET — SG-2 residual next-wave RCA, downstream of #4183.
> **Authority request:** vivid-lynx-81 (Rust RCA Manager) SG-2 residual route per #4140 catalog (E0107/E0282 SG-2 residual band).
> **Existing authority:** The SG-2 `TargetTypeExpressionProjection` worksheet remains the substrate authority for generic instantiation. The #4183 `TestClaimRun` E0107 worksheet established the single-authority fact for the **function-signature / generated cache / static** consumer slice. This worksheet does **not** re-litigate either; it extends the same single-authority fact to the **type-alias declaration** consumer, which routes through a *different* renderer entry point and is therefore not covered by #4183.

---

## Why a second worksheet (scope delta vs #4183)

#4183 named four consumers: `render_rust_decl_type`, `render_rust_type_with_applied_binding`, `render_node_type`, and "generated cache/static/function-signature render paths." Those consumers thread a `generic_param_names: List<String>` context (the enclosing function/decl's type-parameter header) so a generic argument that *is* an in-scope type parameter renders as that parameter rather than being dropped or re-resolved.

The type-alias declaration path does **not** go through that context. In `src/v2/05_emit_rust.dag`, `emit_typed_item` renders an alias RHS as:

```dag
concat(rust_visibility_prefix(), rust_items().type_alias_keyword, " ", item_text, " = ",
       render_rust_type(n: resolved_type(n: item), shared_types: shared_types,
                        source_indices: env.source_indices), ";")
```

`render_rust_type` (line ~150) is the only public renderer that takes **no** `generic_param_names` argument. Internally it always calls the applied-type renderer with an empty context:

```dag
render_rust_applied_type_shared(n: applied, generic_param_names: [], ...)
```

So the alias consumer is structurally distinct from every consumer #4183 covered: it has no path to thread an alias's own generic header, and it cannot distinguish "this child token is a bound type parameter of the alias" from "this child token is a nominal carrier." The same generic-instantiation fact that #4183 protected at signature/cache sites is unprotected here. This is the residual next wave.

---

## Problem Shape

Strict M1 residual in the SG-2 band reports generic carriers losing their type arguments (E0107: "wrong number of generic arguments") and, on the alias side, unresolved/empty type parameters (E0282/E0412 neighbors) at emitted Rust **type alias** declarations whose source `.dag` carried a generic instantiation or a generic alias header.

Representative source shapes that route through the alias consumer (not the #4183 signature/cache consumer):

```dag
# (a) alias whose RHS is an instantiated carrier
type ClaimRunRow = TestClaimRun<Node, RuntimeValue>

# (b) generic alias header forwarding its own params
type Pair<S, A> = TestClaimRun<S, A>

# (c) alias to an instantiated container of a carrier
type ClaimRunBatch = List<TestClaimRun<Node, RuntimeValue>>
```

Emitted Rust that erases instantiation at the alias site, e.g.:

```rust
pub type ClaimRunRow = TestClaimRun;        // requires TestClaimRun<Node, RuntimeValue>
pub type Pair = TestClaimRun;               // S, A dropped from both sides
```

The v2 compile step is diagnostic-clean; the failure surfaces only when rustc checks the emitted Rust. As in #4183, the fix belongs at the Rust type-rendering consumer boundary — here specifically the alias entry point — **not** as a source `.dag` edit or an emitted-file patch.

---

## Section 10.0 Worksheet

```text
SG class:
  SG-2 residual / Rust emitter consumer coverage — type-alias entry point.
  Sibling slice to #4183 (signature/cache/static), same single-authority fact.

Representative emitted failure:
  pub type ClaimRunRow = TestClaimRun;
  pub type Pair = TestClaimRun;
  // Rust requires the authored instantiation / forwarded params.

Immediate local patch (FORBIDDEN):
  - Add a name-keyed arity table consulted by render_rust_type
    (TestClaimRun -> 2, Outcome -> 1, ...).
  - Branch on item_text == "ClaimRunRow" / specific alias names.
  - Special-case "alias RHS == TestClaimRun" in emit_typed_item.

Why forbidden:
  - Duplicates generic instantiation facts already carried by the authored
    (resolved) type expression of the alias RHS.
  - Creates a Rust-emitter-only, alias-only authority for a source fact.
  - Fails the next generic carrier or the next generic alias header unless the
    table grows again.
  - Violates the SG-2 rule forbidding name-keyed Outcome/TestClaimRun/Witness
    arity patches — and #4183 already forbade exactly this for the sibling slice.

DFS path:
  Source authority:
    - Authored alias declarations in src/v4/** : `type T = Carrier<Args...>` and
      `type T<P...> = Carrier<P...>` carry instantiation in the resolved type
      expression and the alias param header.
    - TestClaimRun<S, A> remains the carrier declaration in the evaluation model.
  Existing target-type authority:
    - TargetTypeExpressionProjection remains the modeled generic-instantiation
      authority (unchanged).
  Established consumer authority (#4183):
    - Signature / generated cache / static sites preserve instantiation via the
      render_rust_decl_type / render_rust_type_with_applied_binding /
      render_node_type helper boundary, threading generic_param_names.
  Uncovered consumer boundary (this worksheet):
    - src/v2/05_emit_rust.dag:
      emit_typed_item  (is_type_alias_item arm) -> render_rust_type(...)
      render_rust_type                          -> always generic_param_names: []
      render_rust_type_without_applied_binding  -> render_node_type / applied-shared
    - src/v2/stage0/src/v2_compiler_emit_rust.rs must mirror the .dag behavior.

Deepest unsound boundary:
  render_rust_type is the one public renderer with no generic_param_names
  parameter, and it hardwires generic_param_names: [] into
  render_rust_applied_type_shared. For an alias whose RHS references the alias's
  own generic header (case b) the param tokens have no scope to resolve against;
  for an alias whose RHS is an instantiated nominal carrier (cases a/c) the
  empty-context applied path can collapse to the bare carrier name when the
  __applied_type_args child is absent/empty and the fallback reaches
  render_node_type without the authored instantiation.

Single-authority fact (extends #4183, does not replace it):
  Canonical Rust type rendering for a type position must preserve authored/applied
  generic arguments REGARDLESS OF ENTRY POINT. A type-alias declaration is a type
  position: its RHS must render the canonical instantiated type expression, and an
  alias with a generic header must carry that header onto the emitted alias so the
  forwarded parameters stay in scope on both sides. The alias consumer may compose
  with render_rust_decl_type / render_rust_type_with_applied_binding /
  render_node_type, but it may not render an instantiated type expression as the
  bare carrier name, and it may not drop the alias's own generic parameters.

Stack-safety companion fact (inherited from #4183):
  Any render_rust_decl_type recursion repair reached via the alias path must keep
  the recursive branch inside the bounded stack-growth receipt (stacker /
  maybe_grow) or provide an equivalent modeled bounded receipt. Removing stack
  coverage to fix alias generics is not an acceptable tradeoff.

Systemic fix (direction, not prescription):
  Route the alias RHS through the same canonical instantiated-type renderer the
  signature/cache slice uses, supplying the alias's own param header as
  generic_param_names, and emit the alias's generic header on the left side
  (`pub type Pair<S, A> = ...`). The narrowest faithful form is to give
  render_rust_type a generic_param_names context (defaulting to the alias header
  at the alias call site) and converge its applied path with
  render_rust_decl_type, rather than keeping a second empty-context renderer.
  Stage0 must be regenerated or kept in exact parity with src/v2/05_emit_rust.dag.

Non-goals:
  - TestClaimRun-specific or alias-name-specific arity/name/module branches.
  - Re-implementing the #4183 signature/cache/static slice; that fact stands.
  - Broad SG-2 implementation beyond the alias consumer slice + its shared renderer.
  - Editing emitted v4 Rust files.
  - Treating E0107/E0282 count reduction as acceptance without falsification.
  - Removing stacker / maybe_grow coverage from recursive type rendering.

Falsification probe:
  Add a new generic carrier ProbeClaimRun<X, Y> (reuse the #4183 probe if present)
  and exercise the alias entry point specifically:
    (a) type AliasInst   = ProbeClaimRun<Node, RuntimeValue>
    (b) type AliasFwd<X, Y> = ProbeClaimRun<X, Y>
    (c) type AliasNested = List<ProbeClaimRun<Node, RuntimeValue>>
  Emitted Rust must preserve full instantiation and the forwarded header in all
  three WITHOUT any ProbeClaimRun- or alias-name-keyed branch. Cross-check that
  the #4183 signature/cache probe still passes (no regression at the sibling slice).

Required local receipt:
  V4_M1_RUST_EMIT_PROBE_STRICT=1 cargo/check path passes locally before push,
  AND the emitted alias declarations above type-check.

Metric allowed only as secondary:
  E0107 / E0282 movement in strict M1 at alias sites (never the acceptance gate).
```

---

## Dispatch Gate

No implementation worker should land changes for this slice until the worksheet is ratified by the Modeling DFS arbiter (same gate discipline as #4183). After ratification, an implementation worker may open a PR, but the implementation must stay inside the single-authority boundary above and must converge with — not fork — the renderer the #4183 slice uses.

Worker must:

- Preserve generic arguments at the alias entry point by composing with / converging on the canonical type renderer, threading the alias's own generic header as `generic_param_names`.
- Emit the alias's generic header on the left-hand side when the source alias is generic.
- Keep `src/v2/05_emit_rust.dag` and `src/v2/stage0/src/v2_compiler_emit_rust.rs` in parity.
- Include a non-`TestClaimRun`, non-alias-name-keyed generic falsification probe and re-run the #4183 probe to prove no sibling-slice regression.
- Run strict M1 locally before pushing.

Worker must not:

- Add name-keyed generic arity tables or alias-name branches.
- Patch generated Rust output.
- Re-open the #4183 signature/cache/static fact or broaden into FreeMonoid E0391, E0308 collection projection, or other M1 residual bands without a separate worksheet.

---

## Arbiter Checklist

- [ ] Scope delta accepted: the type-alias entry point is a genuinely uncovered consumer, not a restatement of #4183.
- [ ] Single-authority fact accepted: generic preservation is entry-point-independent; the alias RHS is a type position.
- [ ] Generic-header forwarding (left-hand side) accepted as part of the fact.
- [ ] Spot-fix forbidden: no arity table, no `TestClaimRun`/alias-name branch.
- [ ] Stack-safety condition carried over for `render_rust_decl_type` recursion reached via aliases.
- [ ] Falsification probe accepted, including the #4183 no-regression cross-check.
- [ ] Implementation dispatch authorized.
