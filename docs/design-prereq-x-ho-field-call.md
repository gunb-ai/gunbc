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
  span: ho_param.v3 [106, 107]
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

- **(L1.a) Statically-resolvable callee — reuse `Callable { callee: decl_id, args }`.**
  When the Arrow expression resolves at lowering time to a top-level
  `fn` declaration (e.g., `data v: WrapFn = { f: double }; v.f(x)`,
  where `v.f` projects a `FieldValue::Reference(double)` from the
  data binding's `ValueBody::Structural`), the projection is
  compile-time. Lowering walks `v` → `data v: WrapFn`'s
  `value_body` → `f: FieldValue::Reference(decl_id_of_double)`,
  resolves `decl_id_of_double` to `double`'s arrow signature, and
  emits `TransformDispatch::Callable(CallableDispatch { callee: decl_id_of_double, args })` directly (via `Dag::push_callable_transform`). No
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

  The substrate extension introduces a runtime-callee dispatch
  variant; the encoding question is how to carry the callee port
  alongside argument ports without admitting illegal cardinality
  states. Two competing constraints shape it:

  - **Facts Flow Forward / Every Dependency Is A Substrate Fact:**
    reflected consumers walk `TransformNode.inputs` to derive
    dependencies. A separate-field callee outside `inputs` would
    be invisible to that walk.
  - **Illegal states unrepresentable:** a positional convention
    (`inputs[0]` = callee, `inputs[1..]` = args) admits malformed
    states like `IndirectCall` with empty `inputs` or non-Arrow
    `inputs[0]`. Pushing enforcement to later type-checking
    violates modeling-discipline §"API-level enforcement."

  **Resolution: collapse `target` + `inputs` into a single typed
  `TransformDispatch` sum.** A tagged-element-type approach
  (`Vec<TransformInput>` where `TransformInput = Arg | Callee`)
  was considered and rejected: cardinality of `Callee` per
  `IndirectCall` transform stays a cross-field invariant
  (variant `Callee` is valid only inside `IndirectCall`,
  arbitrary other variants must not carry `Callee`), enforced by
  builder + debug assert rather than by the type. That fails the
  illegal-states-unrepresentable bar.

  The structurally-honest shape collapses
  `TransformNode.target: TransformTarget` and
  `TransformNode.inputs: Vec<PortId>` into one typed sum:

  ```rust
  pub struct TransformNode {
      pub id: NodeId,
      pub dispatch: TransformDispatch,
      pub output: PortId,
      pub span: SourceSpan,
  }

  /// Sum is public so consumers can match; per-variant payloads
  /// are tuple-wrapped structs with module-private fields (no
  /// visibility modifier — private to the dag module only, not
  /// `pub(crate)` which would still allow in-crate construction).
  /// Outside
  /// the dag module no caller can construct `Callable`,
  /// `FieldProject`, or `Indirect` directly — the only path is
  /// the Dag builder that validates against the target signature.
  /// Read access is through accessor methods on the payload structs.
  /// Field projection (pure value access, no call) and field
  /// invocation (project-then-call) are *different state families*
  /// — keep them as separate variants. A `FieldProject` payload
  /// has no `args` because there is no call; a `FieldCall` payload
  /// always has args (zero-arg calls still carry an empty list,
  /// but the *call vs. project* axis is structural).
  pub enum TransformDispatch {
      Callable     (CallableDispatch),
      FieldProject (FieldProjectDispatch),  // pure projection — current TransformTarget::FieldProject shape, preserved
      FieldCall    (FieldCallDispatch),     // project-then-call — new for X1
      Operator     (OperatorCall),
      Indirect     (IndirectDispatch),
  }

  pub struct CallableDispatch {
      /* private */ callee: DeclarationId,
      /* private */ args:   Vec<PortId>,
  }

  /// Pure field projection. No args — projecting a field of a
  /// carrier value is a value-access fact, not a dispatch.
  /// Preserves the current `TransformTarget::FieldProject` shape.
  pub struct FieldProjectDispatch {
      /* private */ field_label: String,
      /* private */ field_child: Option<DeclarationId>,
      /* private */ carrier:     PortId,
  }

  /// Field invocation: project an Arrow-typed field, then call it.
  /// Distinct from `FieldProject` because the carrier→field→call
  /// composition is a different state family than plain projection.
  pub struct FieldCallDispatch {
      /* private */ field_label: String,
      /* private */ field_child: Option<DeclarationId>,
      /* private */ carrier:     PortId,
      /* private */ args:        Vec<PortId>,
  }

  pub struct IndirectDispatch {
      /* private */ callee: ArrowPortRef,
      /* private */ args:   Vec<PortId>,
  }

  impl CallableDispatch {
      pub fn callee(&self) -> DeclarationId { self.callee }
      pub fn args(&self)   -> &[PortId]     { &self.args }
  }
  // (analogous accessors for FieldProjectDispatch / FieldCallDispatch / IndirectDispatch)

  /// Operators have fixed arity per op-kind; encode it in the sum.
  /// Unary/Binary cannot accidentally swap arities. `OperatorCall`
  /// remains a plain pub enum because both variants are
  /// constructable from primitives without target resolution —
  /// no signature is being witnessed, so there's no proof to
  /// protect.
  pub enum OperatorCall {
      Unary  { op: UnaryOp,  arg: PortId },
      Binary { op: BinaryOp, lhs: PortId, rhs: PortId },
  }

  /// **Atomic dispatch construction binds the args proof to the
  /// target.** The payload structs (`CallableDispatch`,
  /// `FieldProjectDispatch`, `IndirectDispatch`) have
  /// `/* private */` fields, so outside the dag module they cannot
  /// be constructed directly — the type system, not convention,
  /// blocks `CallableDispatch { callee, args }` literal
  /// construction. The only public path that yields a
  /// `TransformDispatch::Callable(_)` / `FieldProject(_)` /
  /// `Indirect(_)` is a Dag-level builder that takes
  /// `(target_identity, raw_ports)`, resolves the target's Arrow
  /// signature, and validates arity + per-position types in one
  /// step. The args proof and target are co-constructed; an args
  /// list checked against signature A cannot inhabit a dispatch
  /// built for target B because no public constructor exists
  /// that pairs pre-validated args with an arbitrary target.
  /// Same pattern as `NonSingletonList::from_vec` extended to a
  /// co-constructed pair.
  ///
  /// ```rust
  /// impl Dag {
  ///     pub fn push_callable_transform(
  ///         &mut self, callee: DeclarationId, raw_ports: Vec<PortId>, ...,
  ///     ) -> Result<NodeId, CallShapeError> { ... }
  ///
  ///     pub fn push_indirect_transform(
  ///         &mut self, callee: ArrowPortRef, raw_ports: Vec<PortId>, ...,
  ///     ) -> Result<NodeId, CallShapeError> { ... }
  ///
  ///     pub fn push_field_project_transform(
  ///         &mut self, field_label: String, field_child: Option<DeclarationId>,
  ///         carrier_port: PortId, ...,
  ///     ) -> Result<NodeId, CallShapeError> { ... }
  ///
  ///     pub fn push_field_call_transform(
  ///         &mut self, field_label: String, field_child: Option<DeclarationId>,
  ///         carrier_port: PortId, raw_ports: Vec<PortId>, ...,
  ///     ) -> Result<NodeId, CallShapeError> { ... }
  /// }
  /// ```
  ///
  /// Args therefore carry no separate type — the binding to the
  /// target's signature is established at construction and
  /// expressed by the absence of any public constructor that
  /// could split them.

  /// Track-9 typed handle — wraps a PortId with proof that the
  /// referenced port carries an Arrow-typed value. Only constructable
  /// via a Dag-level validator that resolves the port's producer
  /// behavior signature and verifies it's an Arrow connective:
  ///
  /// ```rust
  /// impl Dag {
  ///     pub fn resolve_arrow_port(&self, p: PortId)
  ///         -> Result<ArrowPortRef, NonArrowPortError> { ... }
  /// }
  /// ```
  ///
  /// `ArrowPortRef`'s constructor is private to the dag module; outside
  /// callers must go through `Dag::resolve_arrow_port`. Same pattern
  /// as `NonSingletonList::from_vec` — the type carries the proof.
  pub struct ArrowPortRef(/* private */ PortId);

  impl TransformDispatch {
      /// Single-authority dependency walk for Facts Flow Forward.
      /// Yields *every* runtime `PortId` the dispatch depends on,
      /// across all variants. Reflected consumers (lenses,
      /// schedulers, dataflow analyses) iterate this without
      /// knowing which variant they have.
      ///
      /// Per-variant enumeration (callee identities resolved by
      /// `DeclarationId` at lowering time are not runtime ports
      /// and are *not* yielded; runtime ports always are):
      ///
      /// - `Callable(d)`              — `d.args`.
      /// - `FieldProject(d)`          — `d.carrier`.
      /// - `FieldCall(d)`             — `d.carrier`, then `d.args`.
      /// - `Operator(Unary{arg})`     — `arg`.
      /// - `Operator(Binary{lhs,rhs})` — `lhs`, then `rhs`.
      /// - `Indirect(d)`              — `d.callee` (the wrapped
      ///                                `ArrowPortRef`'s port),
      ///                                then `d.args`.
      pub fn input_ports(&self) -> impl Iterator<Item = &PortId> { ... }
  }
  ```

  **Arity enforcement.** `Operator` arity is fixed by op-kind and
  encoded directly in `OperatorCall`'s variants — a `Unary` cannot
  carry two operands, a `Binary` cannot carry one, by type. The
  call-shapes (`Callable` / `FieldCall` / `Indirect`) have
  signature-dependent arity that only the lowerer knows. For those,
  the args proof is bound to the target by **atomic dispatch
  construction**: variant fields are module-private to `dag` (no
  visibility modifier; not `pub(crate)`), and the only
  public surface that yields a `Callable` / `FieldProject` /
  `Indirect` is a Dag builder (`push_callable_transform`,
  `push_field_project_transform`, `push_indirect_transform`) that
  takes `(target_identity, raw_ports)`, resolves the target's
  Arrow signature, and validates arity + per-position types against
  it in one step. There is no public constructor that takes a
  pre-validated args list and pairs it with an arbitrary target —
  the proof and target are co-constructed, so args validated against
  signature A cannot be re-attached to a dispatch built for target B.

  Both invariants now hold structurally:

  - **Facts Flow Forward:** `dispatch.input_ports()` is the single
    authority that yields every runtime-port dependency for any
    variant — including the `Indirect.callee`'s wrapped `PortId`.
    Reflected consumers (lenses, schedulers, dataflow analyses)
    walk this iterator without knowing which variant they have.
  - **Illegal states unrepresentable at the type level:**
    - `Callable` / `FieldProject` / `FieldCall` / `Operator` cannot accidentally
      carry a runtime callee port (no `callee` field of any kind).
    - `Indirect` cannot omit its callee (single field, not a
      `Vec`; not `Option`).
    - Multi-callee `Indirect` is impossible (single field, not a
      `Vec`).
    - `Callable.callee: DeclarationId` (compile-time) and
      `Indirect.callee: ArrowPortRef` (runtime) are incompatible
      types; the type system separates them.
    - **Arrow-typed callee proof:** `Indirect.callee` is
      `ArrowPortRef`, not raw `PortId`. A non-Arrow port cannot
      inhabit `ArrowPortRef` because the only constructor
      (`Dag::resolve_arrow_port`) validates the port's producer
      signature is Arrow-typed and returns `Err(NonArrowPortError)`
      otherwise. Constructing `Indirect { callee: ..., ... }` with
      a non-Arrow port is therefore unrepresentable —
      type-checking happens at the type-handle's construction
      boundary, not behaviorally inside the lowerer.

  No builder + debug-assert ratchet; cardinality and target/callee
  compatibility are both expressed in `TransformDispatch`'s shape.
  The constructor APIs (`Dag::push_callable_transform(...)`,
  `Dag::push_indirect_transform(...)`, etc.) become
  ergonomic helpers, not invariant-defenders — the type rejects
  malformed combinations at construction by definition.

  **Dissolution ledger.** Per modeling-discipline coproduct
  classification, each variant is tagged with its dissolution
  status (🟢 keep / 🟡 future-dissolve / 🔴 dissolve-now):

  - 🟡 **`Operator { op: OperatorCall }`** — future-dissolve.
    Per modeling-discipline Practice 4, the canonical example
    of an algebraic-form coproduct that should dissolve is
    `ArithOp::{Add,Sub,Mul,Div}` → `Apply { function: FunctionRef }`
    pointing at the corresponding `std::int::add` / `std::int::sub`
    function references. `OperatorCall::{Unary,Binary}` is
    structurally that case: each operator has a richer source —
    the algebra-witness function in `std/int/`, `std/bool/`, etc.
    — so absence of a *current* `DeclarationId` is not the same
    as "no richer source exists." Held as its own variant today
    because (a) the std-side algebra-witness functions are not
    yet declared as resolvable `DeclarationId`s for unary `!` /
    binary `+` etc., and (b) the parser still emits operator
    tokens distinct from call syntax. **Tracking gate:** dissolve
    when `std/{int,bool,float}/` declares the operator-algebra
    witness functions and the parser desugars operator tokens
    to `Call(FunctionRef)` at parse time. At that point
    `Operator` collapses into the same `Call { callee: CalleeRef
    = Decl(...) }` shape as the rest of the call-shapes, and the
    🟡 set above absorbs it.

  - 🟢 **`FieldProject`** — keep. Pure value-access fact
    preserved from the current `TransformTarget::FieldProject`
    shape; no dispatch (no args). Distinct state family from
    `FieldCall` — collapsing them would conceal the
    projection-vs-invocation axis behind args-emptiness, which
    is the conflation the reviewer flagged.

  - 🟡 **`Callable` / `FieldCall` / `Indirect`** — future
    dissolution to a single `Call { callee: CalleeRef, args }`
    variant where `CalleeRef = Decl(DeclarationId) | Field {
    label, child: Option<DeclarationId>, carrier: PortId } |
    Port(ArrowPortRef)`. The three variants share the same
    dispatch shape (target + args validated against target's
    Arrow signature); they differ only in callee-identity source.
    Held as separate today because (a) emitter rendering
    currently dispatches on callee-source, and (b) the args proof
    is bound per-variant by the atomic builders described above —
    collapsing requires `CalleeRef` itself to carry the same
    co-construction guarantee. **Tracking gate:** dissolved when
    the emitter splits callee-rendering from the dispatch match
    (same shape as `lens_*_emitter_split` work). Not a blocker
    for X1; a follow-up modeling slice.

  - 🔴 **None** — no variant is in dissolve-now state because
    nothing in `TransformDispatch` admits a malformed-state shape
    that a downstream lens would have to defend against; the
    invariants (Facts Flow Forward, illegal-states-unrepresentable,
    args-bound-to-target) all hold under the 🟡 set above.

  No variant is added speculatively: each axis (built-in op vs
  user-defined call vs runtime-port callee) has a present
  consumer in the lens framework's expected dispatch surface.

  **Migration cost.** The collapse is a substantial refactor of
  `TransformNode` and every consumer that walks `target` /
  `inputs` separately (`emit_*_target.rs`, lens reads, lowerer
  call-site emit, bootstrap-generated). All consumers move from
  `(t.target, t.inputs)` to `t.dispatch`, with the
  `dispatch.input_ports()` iterator replacing direct
  `t.inputs.iter()` walks. **Implementation worker scopes the
  migration**; the audit only locks the target shape.

  **Emitter contract.** Emitters match on `TransformDispatch`
  variant:

  ```rust
  match &t.dispatch {
      TransformDispatch::Indirect(d) => {
          render_indirect_call(d.callee(), d.args(), ...)
      }
      TransformDispatch::Callable(d) => {
          render_callable(d.callee(), d.args(), ...)
      }
      // ... etc.
  }
  ```

  No partition/lookup/fail-closed defense needed; the variant
  branch carries the right fields. `EmitError::MalformedIndirectCall`
  retires — the malformed state is unrepresentable.

  Per-target `SubstrateAccessorBinding`-style rendering is not
  required because the call is structural (no per-accessor
  carrier), only the call-syntax template per target.

  **Dissolution / ratchet receipt.** Two distinct claims, kept
  separate to avoid the muddle the reviewer flagged:

  - **HO dispatch capability is permanent.** Higher-order dispatch
    over Arrow-typed runtime values is a real long-term language
    surface, not staging; some variant must encode it. No SCAFFOLD
    lifecycle on the *capability*.
  - **The specific variant spelling `Indirect(IndirectDispatch)` is
    transitional.** Per the 🟡 ledger entry above, `Callable` /
    `FieldCall` / `Indirect` future-collapse into a single
    `Call { callee: CalleeRef, args }` variant where
    `CalleeRef = Decl | Field | Port`. After that collapse, HO
    dispatch is expressed as `Call { callee: CalleeRef::Port(_), .. }`
    — same capability, different spelling. The variant *name*
    `Indirect` retires; the *capability* it carries does not.

  The `(target, inputs)` → `dispatch` collapse is permanent —
  single-authority dispatch encoding is the long-term shape, not a
  transitional bridge.

**Sequencing for the implementation slice:**
1. (L1.a) statically-resolvable case lands FIRST against the
   existing static-callee dispatch path (renamed to
   `TransformDispatch::Callable` post-collapse, but no new
   variant). Covers
   `data v: WrapFn = { f: double }; v.f(x)`. This is enough to
   unblock `data complexity_lens: Lens<Int> = { ... }` consumers
   when `complexity_lens.read(d, b)` is called from `fold_lens<C>`
   if and only if `complexity_lens` is a `data` binding (which it
   is — Lens instances are top-level data).
2. (L1.b) runtime-sourced case lands SECOND with the
   `TransformDispatch::Indirect(IndirectDispatch { callee: ArrowPortRef, args: Vec<PortId> })` (constructed atomically via `Dag::push_indirect_transform`)
   variant + the `TransformNode.target/inputs` collapse described
   above.
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
to scope Prereq-X1 (and Prereq-X3 if Director confirms it is
required for `fold_lens<C>` — X3's syntax is already locked to
explicit block markers; only the *need* for X3 in this slice
remains an open call) as a separate parser/lowerer slice.

---

## Cross-references

- `docs/design-lens-fold-prerequisites.md` — original lens-fold
  audit; this Prereq-X is a follow-up that audit didn't catch.
- `src/v3/std/lens.dag` — Lens<C> 6-field carrier (#1186).
- `src/v3/std/dimensions.dag:82-87` — `AnalysisDimension<Carrier>`
  precedent for Arrow-typed Conj fields. Field assignment landed
  via Prereq-1; field invocation never exercised because
  `analyze_symbolic_cost_dimension` data binding was deferred per
  `src/v3/lenses/cost.dag:268-302`.
- `src/v3/std/substrate.dag` — `workflow_root_port` accessor + `WorkflowRoot`
  sum (Prereq-3a, #1232).
- Prereq-1: PR #1230 + #1239.
- Prereq-2: PR #1248.
- Prereq-3a: PR #1232.
