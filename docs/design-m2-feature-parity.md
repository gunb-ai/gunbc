> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Unblocks: Lane 3 Stages 3a.2, 3a.3, 3a.4, 3a.5

# Design DB-10..DB-13 — M2 feature parity (3a.2–3a.5)

**Design blockers:** DB-10 (data value semantics), DB-11 (`where` refinement), DB-12 (surface generics), DB-13 (Disj dotted-path)
**Consumers:** Lane 3 Stage 3a.2 / 3a.3 / 3a.4 / 3a.5
**Status:** Design ready for implementer review.
**Companion:** [DB-9](./design-mutual-recursion-lowering.md) covers 3a.1 mutual recursion (the L of this set).

---

## Why one document

The four remaining 3a sub-stages share two things: they all close M2 feature-parity gaps that `compiler.dag` needs, and three of them (DB-10, DB-12, DB-13) are scope-clarifications of mechanisms that already exist in the compiler — substrate edges, parser AST fields, lens passes — that simply weren't wired through to the surface. DB-11 (`where` refinement) is the one where new semantics attach to an existing fact (a `where`-clause expression the parser currently discards). Keeping the four in one doc makes the "existing-mechanism → surface" through-line visible.

Each sub-stage below has its own Problem / Design / Rejected alternatives / Acceptance. The Associations section is shared at the bottom.

---

## DB-10 — `data` value semantics (3a.2, size S)

### Problem

`Declaration.value_body: Option<ValueBody>` (`src/v3/compiler/src/dag.rs:122`) already carries the lowered record-literal fields and scalar literals for `data foo: T = v` declarations. The parser lowers the body; `ValueBody::Structural { fields }` and `ValueBody::List` / `Map` / `Record` / `Variant` variants exist. But nothing downstream reads `value_body` back — emission treats every identifier reference as an opaque pointer to a `Declaration`, never inlines the carried value, and `resolve_static_field_project` (`lower.rs:1404`) walks a type's `Conj` without consulting whether the declaration also carries a value. The net effect: `data answer: Int = 42` compiles but is unreachable at every use site, and `data config: Config = { host: "h", port: 8080 }` supports `config.host` neither at compile time nor at emit time.

### Design

**One query function, zero substrate-shape change:**

```rust
// src/v3/compiler/src/dag.rs — impl Dag
fn data_value_at(&self, decl_id: DeclarationId) -> Option<&ValueBody> {
    self.declarations[decl_id].value_body.as_ref()
}
```

Reads the existing field. No new state. Consumed in two places:

**1. Emission.** At every identifier-resolution emit site (where the emitter today produces a reference to a declared name), the emitter asks: does the resolved `Declaration` carry a `value_body`? If yes, render the value directly via the existing `FieldValue`/`ValueBody` renderers:

- `ValueBody::Structural { fields }` — dispatch to `render_record_constructor`.
- `FieldValue::Literal(LiteralBits)` — target-native literal (Rust: `42`, Go: `42`, Python: `42`; strings quoted per language).
- `FieldValue::Reference(decl_id)` — recurse through `data_value_at`.
- `FieldValue::List(vs)` / `Map` / `Record` / `Variant` — existing nested renderers.

Three emit files today (`emit_rust.rs`, `emit_go.rs`, `emit_python.rs`); Lane 1e has not yet collapsed them. 3a.2 wires the check into all three at the identifier-resolution site. When Lane 1e lands, the three wirings collapse to one; no design change is needed at that point.

**2. Static field access.** `resolve_static_field_project` (`lower.rs:1404`) is extended so that when its input port resolves to a `Declaration` with `value_body: Some(ValueBody::Structural { fields })`, the requested field label is looked up in `fields` before walking the declared type's `Conj`. A record-valued data declaration resolves `config.host` to `"h"` at compile time; the `FieldProject` Transform is annotated with the literal. Scalar-valued data (`data answer: Int = 42`) has no fields; the existing "not a Conj" path handles the `config.host` analog on `answer.foo` and emits a diagnostic as before.

### Rejected alternatives

- **New `Value` substrate node wrapping the literal** — duplicates `value_body`, creates parallel authority. Any fact the substrate can compute from `value_body` should not also live as a node. Rejected.
- **Special-casing data values as anonymous `Bind { body: Value(...) }`** — rewrites the substrate shape for a non-problem. `value_body` is an edge on `Declaration` precisely to avoid this. Rejected.
- **Inlining values at parse time (no dedicated accessor)** — couples parser and emitter; emitter cannot see the declaration's name in diagnostics. `data_value_at` as a substrate-level accessor preserves name-for-diagnostic and is a single-line function. Rejected as a non-improvement.

### Acceptance

- `data answer: Int = 42` compiles; a use of `answer` emits the literal `42` in all three target languages.
- `data config: Config = { host: "h", port: 8080 }` compiles; `config.host` resolves statically; emission produces the target-native string literal `"h"` at the use site (not a struct-field read).
- `fn f() -> Int = answer + 1` emits `42 + 1` (inlined) in all three targets.
- Rejected: `data x: Int = 42` + `x.foo` → diagnostic: "Int has no field foo" (scalar data has no static field path).

---

## DB-11 — `where` refinement predicates (3a.3, size M)

### Problem

The parser consumes `where <clause>` at `parse.rs:985` (`skip_where_clause`) but discards the content. `fn div(n: Int, d: Int where d != 0) -> Int = n / d` compiles identically to `fn div(n: Int, d: Int) -> Int = n / d`; `div(1, 0)` is accepted despite the author's declared intent to reject it. The 28+ `where` clauses already present in `dsl/std/` (in `types.dag`, `string_type.dag`, `iteration.dag`, `induction.dag`, `binding.dag`, `coercion.dag`) are documentation today, not compiler-enforced. Making them enforceable is the 3a.3 mandate.

### Design

The design has three parts, each grounded in a mechanism the compiler already has.

#### Q1 — Where refinement lives

Refinement has two flavors and they get different homes:

- **Declared refinement** (`fn div(n: Int, d: Int where d != 0)`): the predicate is part of the signature and visible globally. `Int where d != 0` is a distinct **nominally refined type**, not a subtype of `Int`. It attaches to the parameter's type declaration as a structural edge:

```rust
// src/v3/compiler/src/dag.rs — Declaration
pub struct Declaration {
    pub id: DeclarationId,
    pub name: Option<String>,
    pub connective: TypeConnective,
    pub type_params: Vec<DeclarationId>,
    pub meta_tag: Option<DeclarationId>,
    pub inhabits: Option<DeclarationId>,
    pub value_body: Option<ValueBody>,
    pub refinement: Option<DeclarationId>,  // NEW — points at the predicate expression declaration
    pub span: SourceSpan,
}
```

The predicate is itself a `Declaration` whose connective resolves to a `Bool`-typed expression DAG (e.g., `gt(x, 0)` → resolved `Transform` referencing `OrderedRing<Int>::gt`). The parent type declaration points at it via `refinement`. Two distinct refinements on `Int` (e.g., `Int where x > 0` and `Int where x != 0`) are two distinct declarations.

- **Inferred refinement** inside a Branch arm (`if d != 0 then div(n, d)`): already solved. M1(2.8)'s infer-time pattern resolution narrows the arm-scoped port type when a Branch discriminates on a predicate. Same machinery that narrows `Some(s)` in `match opt { Some(s) => ... }`. `where` refinement *consumes* this existing narrowing — it does not add a parallel `(Port, BranchContext)` side table.

**Why declared-on-declaration, not on-port:** a function signature's parameter port carries a type reference. The refinement is a property of the type, not the port instance. Two functions with `Int where d != 0` share the same refined declaration; two port instances at the same signature site share it via the type edge. Keeping the refinement on the `Declaration` avoids duplicating it across every port that references that type.

#### Q2 — Proof theory: structural equality

A call's argument satisfies the callee's refinement iff its port type declaration carries a refinement whose resolved predicate expression DAG is **structurally equal on resolved predicate expression DAGs** to the callee's — walk both trees, compare operator resolution and operand structure node-by-node. No interning table, no reliance on shared `DeclarationId`s; two independently-constructed `Int where d != 0` declarations check equal because their resolved predicate DAGs walk the same. Discharge mechanisms:

- **Same-declaration discharge:** caller's port type IS the callee's refined type (identity). A trivial special case of the structural walk.
- **Branch-arm discharge:** inside an arm of a Branch that narrowed the port to the refined type, the arm-scoped port carries a refinement whose predicate DAG walks equal to the callee's via M1(2.8)'s narrowing.

No SMT. No algebra entailment. No implication reasoning. `x > 1` does **not** automatically discharge `x > 0`: they are distinct refinement declarations with distinct predicate expression DAGs. If users want that, they write a Branch that checks `x > 0` inside the `x > 1` scope (trivial, source-level); or, in a follow-up stage, `algebra.dag` declares entailment rules and the structural-equality check consults them.

This is a weak proof theory. That is the point. It is decidable, it preserves the bounded-kernel invariant, and it forces the user to write the narrowing Branch they would have needed to prove the implication anyway.

#### Q3 — Algebra resolution: no new machinery

The predicate `x > 0` parses as `gt(x, 0)` (via existing expression parser). Lowering resolves `gt` against `OrderedRing<T>` through `resolve_operator_arrow` (M1(2.7) R9) — the same path every other operator takes. The resolved predicate is a well-typed `Bool` expression DAG. Structural equality on predicates is structural equality on resolved expression DAGs.

### Implementation shape

- Parser: `parse.rs:985` — replace `skip_where_clause` with `parse_where_clause`. Return the parsed expression (reuse the expression parser already used for function bodies). The `where` clause is parsed *after* the parameter's type but within the parameter's scope; the parameter name resolves to the parameter's port.
- Lowering: for each parameter with a parsed `where` expression, lower the expression against a scope where the parameter name is bound to its port. Resolve operators via `resolve_operator_arrow`. Create a new anonymous `Declaration` whose connective wraps the resulting expression DAG. Create a new `Declaration` for the refined type (clone of the base type declaration) with its `refinement` edge set to the predicate declaration. Assign the parameter's port type to the refined declaration.
- Call-site check: at call lowering, for each argument, compare structurally:
  1. Callee parameter port's type declaration's `refinement: Option<DeclarationId>`.
  2. Argument port's type declaration's `refinement: Option<DeclarationId>`.
  - If callee has no refinement → unconditionally OK.
  - If callee has refinement R, and argument's port type declaration also carries a refinement whose resolved predicate expression DAG is **structurally equal on resolved predicate expression DAGs** to R's (structural walk — same operator resolution at each node, same operand structure; no interning, no shared-id requirement) → OK.
  - Otherwise → Diagnostic with DB-1 Correction.
- Branch-arm narrowing integration: verify (via reading `infer.rs`'s pattern-resolution pass) that a Branch checking a predicate expression E narrows `then`-arm-scoped ports that share E with a refined type declaration. If the pass already narrows variant-payload types but does not narrow refinement-predicate types, extend it with the predicate case. This is a small extension to an existing pass; it is not a new pass.

### Diagnostic shape

On refinement discharge failure at a call site:

```
ERROR at <call span>: argument `d` does not satisfy required refinement
  expected: Int where d != 0
  argument: Int  (no narrowing branch in scope)

FIX: guard the call with the refinement check:
    if d != 0 then div(n, d) else <fallback>
```

Uses DB-1 Correction shape (source-level).

### Rejected alternatives

- **`(Port, BranchContext)` side table** — the existing Branch-arm narrowing pass already produces per-arm port typings; a side table is parallel authority. Rejected.
- **SMT-style semantic entailment** — violates decidability invariant; unbounded reasoning. Out of scope for 3a.3.
- **Explicit algebra-declared entailment rules** (`x > 1 ⇒ x > 0`) — requires `algebra.dag` entailment declarations that do not exist. Legitimate follow-up stage; not 3a.3.
- **Refinement as a separate lens (not part of the type)** — makes refinement invisible at call-site type matching, forcing every lens that cares about refinement to walk the Branch-arm table separately. Rejected — refinement is a property of the type, not a secondary fact.
- **Refinement stored on every port instance** — duplicates the refinement across every port that references the refined type. Rejected in favor of the declaration-level edge.

### Acceptance

- `fn div(n: Int, d: Int where d != 0) -> Int = n / d` compiles.
- `div(1, 0)` is rejected at compile time with a refinement diagnostic carrying DB-1 Correction.
- `if d != 0 then div(n, d) else 0` compiles — Branch-arm narrowing discharges the refinement inside `then`.
- Distinct refinements do not entail each other: `fn f(x: Int where x > 1) -> Int = g(x)` where `g: fn(x: Int where x > 0) -> Int` is rejected (structural equality on predicate expression DAGs; `gt(x, 1)` ≠ `gt(x, 0)`).
- Substrate integrity: `Declaration.refinement` is the only new edge. `type Behavior` remains at five variants.

---

## DB-12 — surface generics (3a.4, size S)

### Problem

The substrate already supports generic type parameters: `Declaration.type_params: Vec<DeclarationId>` (`dag.rs:119`), `AtomPayload::TypeParam(String)` (`dag.rs:352–376`), `LowerSubstStack` for substitution (`lower.rs:1251–1268`), and inference already walks TypeParams correctly. The parser already recognizes `<T>` after the *type-declaration* identifier (`type Foo<T> { ... }`). `SurfaceItem::Fn` already has a `type_params: Vec<String>` field declared.

The gap is one parser arm: the `fn` item parsing path doesn't consume `<T, U, ...>` between the identifier and `(`. So `fn id<T>(x: T) -> T = x` fails to parse, even though every downstream stage already handles generics.

### Design

Parser extension only. No substrate-shape change, no inference change, no emission change.

- In `parse.rs`'s `parse_item` function, after the `fn` keyword and identifier, optionally consume `< ident (, ident)* >` before `(`. Store each consumed identifier in the existing `SurfaceItem::Fn.type_params` field.
- In `lower.rs`'s fn lowering path, when `SurfaceItem::Fn.type_params` is non-empty, create `Atom(AtomPayload::TypeParam(name))` declarations for each and push their `DeclarationId`s to `Declaration.type_params`.
- Inside the fn's scope, each type parameter name resolves to its `TypeParam` atom declaration — the parameter types (`x: T`) use existing identifier-resolution to find the `TypeParam` atom by name.

**Bare bindings only.** `<T>`, `<T, U>`. No explicit bounds (`<T: Ord>`). Constraints come from use-site `inhabits` resolution (same as Prereq 0.5's inferred generics). This matches gunbc's "walk inhabitance, not a bootstrap pre-registration table" principle: an algebra is required by a function if the function uses one of the algebra's operators, which is an already-computed fact.

### Rejected alternatives

- **Explicit bounds in generic declaration** (`fn f<T: Ord>(...)`) — parallel to `inhabits` resolution. Two authorities for "what algebra must T satisfy." Rejected. Parse-time diagnostic with Correction: "drop `: Ord` — algebra constraints are inferred from operator usage."
- **New `Generic` substrate variant** — `TypeParam` atom already exists and inference already handles it. No substrate extension needed. Rejected.
- **Making type params implicit at the call site (`fn id(x: T) = x` with T inferred at definition)** — the bare-param form already exists and works; `<T>` surface syntax is for *explicitness* in `compiler.dag` and doc-style code where the author wants the binding visible. Implicit-at-definition remains available; explicit is additive.

### Acceptance

- `fn id<T>(x: T) -> T = x` compiles.
- `fn pair<A, B>(a: A, b: B) -> Pair<A, B> = mk_pair(a, b)` compiles.
- `fn bound<T: Ord>(x: T) -> T = x` is rejected at parse time with Correction: drop `: Ord`.
- Substrate integrity: no new variant, no new field; `Declaration.type_params` already existed.

---

## DB-13 — Disj dotted-path (3a.5, size S)

### Problem

Inside `match opt { Some(s) => s.field, None => ... }`, the match-arm lowering binds `s` to a port whose type is the narrowed variant-payload type (the `Some` variant's payload; already computed by M1(2.8)'s infer-time pattern resolution). Accessing `s.field` fails because `lower_field_path_expr` (`lower.rs:3034–3083`) gates expression-position dotted paths on a local-variable head lookup that doesn't include pattern bindings. The error at `lower.rs:3050–3059` explicitly says so: "expression-position dotted paths currently require a local-variable head." This unblocks Half B's B13.

### Design

Two mechanical wirings:

- **Parser:** match-arm body parsing currently accepts expression-position identifiers and literals but not dotted paths. Extend the match-arm body expression grammar to accept `ident (. ident)+`, producing `SurfaceExpr::Path { segments }` (a shape that already exists — `parse_dotted_path` in `parse.rs:766–800` handles it for other contexts).
- **Lowering:** extend `lower_field_path_expr`'s `scope` lookup at `lower.rs:3049` to consult the match-arm's pattern-binding scope in addition to the local-variable scope. M1(2.8) already produces the arm-scoped binding's narrowed port type; the lookup just needs to find it. Once the head port is found, the existing `for field_label in rest { ... }` loop already walks the fields via `resolve_static_field_project`, which uses `walk_to_conj_decl_with_subst_lower` (`lower.rs:1342`). The error at `lower.rs:3050–3059` is dropped (replaced by the successful lookup) for the arm-scoped case.

No new walk. No new substrate. No new lens. The machinery is all present; the parser-to-lowering path just wasn't wired through for match-arm bodies.

### Rejected alternatives

- **Introducing a separate `ArmField` substrate node** — the existing `FieldProject` `Transform` already handles the access; the gap is upstream of the transform. Rejected.
- **Type-narrowing as a new lens** — M1(2.8) infer-time pattern resolution already narrows; we consume its output. Rejected.
- **Match-arm scopes merged into the enclosing local-variable scope at parse time** — scope leaking. Arm bindings are scoped to the arm. Rejected.

### Acceptance

- `match opt { Some(s) => s.field, None => default }` parses and lowers.
- Emission in Rust / Go / Python produces the target-native pattern-match with field access.
- Nested dotted paths in arm bodies work: `match wrapper { Wrap(inner) => inner.a.b }`.
- Half B's B13 unblocked.

---

## Consolidated implementation sequence

Size classifications per `lane3-self-hosting-cycle.md:36–43` rows. Sequence chosen so that 3a.3 (the M-sized item with the most substrate-adjacent change) lands after 3a.4 (which also touches `SurfaceItem::Fn`), to avoid churn on the same code.

1. **DB-12 / 3a.4 surface generics** — parser + lowering plumbing. Smallest, fully self-contained.
2. **DB-13 / 3a.5 Disj dotted-path** — parser + lowering plumbing. Smallest.
3. **DB-10 / 3a.2 data value semantics** — substrate accessor + emit wiring in three files (Lane 1e not yet done) + static field resolution.
4. **DB-11 / 3a.3 `where` refinement** — substrate edge on `Declaration` + parser extension + call-site check + Branch-arm narrowing integration. Largest of this set.

3a.1 mutual recursion (DB-9, size L) is independent and can proceed in parallel with any of the above.

**Escalation:** any sub-stage that materially exceeds its size classification (S → M, M → L) stops and escalates. Per lane3-self-hosting-cycle.md:54.

---

## Associations

- **Lane 3 Stage 3a.2 / 3a.3 / 3a.4 / 3a.5** ([lane3-self-hosting-cycle.md](./lane3-self-hosting-cycle.md)) — this is that stage's consolidated design.
- **DB-9 Mutual recursion** ([design-mutual-recursion-lowering.md](./design-mutual-recursion-lowering.md)) — 3a.1's design; shares Stage 3a acceptance ceiling.
- **DB-1 Correction shape** ([design-correction-shape.md](./design-correction-shape.md)) — refinement diagnostics (DB-11), bounded-generic diagnostics (DB-12), and dotted-path diagnostics (DB-13) emit Corrections.
- **M1(2.7) R9** — `resolve_operator_arrow` is the resolution path for `where`-clause predicates (DB-11 §Q3).
- **M1(2.8)** — infer-time pattern resolution is the narrowing machinery that DB-11 consumes for Branch-arm discharge and DB-13 consumes for arm-scoped field access.
- **`src/v3/compiler/src/dag.rs`** — `Declaration.value_body` (DB-10 authority), `Declaration.refinement` (DB-11 new edge), `Declaration.type_params` (DB-12 consumer), `ValueBody`/`FieldValue` renderers (DB-10).
- **`src/v3/compiler/src/parse.rs`** — `skip_where_clause` (DB-11: becomes `parse_where_clause`), `parse_item` fn arm (DB-12: consume `<T, U>`), match-arm body grammar (DB-13: accept dotted path).
- **`src/v3/compiler/src/lower.rs`** — `lower_field_path_expr` (DB-13), `resolve_static_field_project` (DB-10), `walk_to_conj_decl_with_subst_lower` (DB-10/DB-13), fn-item lowering (DB-11/DB-12).
- **`src/v3/compiler/src/infer.rs`** — Branch-arm narrowing pass (DB-11 integration), generic inference (DB-12 consumer).
- **`src/v3/compiler/src/emit_rust.rs` / `emit_go.rs` / `emit_python.rs`** — DB-10 wires data-value rendering into each; when Lane 1e collapses the three, the wiring collapses with them.
- **Thesis anchors** — THESIS.md:604-629 (five behaviors, not violated), decidability invariant (DB-11 proof theory bounded to structural equality).

---

## Open questions

1. **`refinement` on `Declaration` vs. on the parameter port** (DB-11). The doc proposes declaration-level. The alternative — per-port — is rejected for duplication reasons but may matter if two calls to the same function with differently-refined arguments need to participate in the same refinement check. Verify by reading the port/declaration split in `infer.rs` before committing to the edge location; if ports carry their own type declarations (not references to a shared type declaration), the distinction dissolves.

2. **Branch-arm predicate narrowing already exists?** (DB-11). The design assumes M1(2.8)'s pattern-resolution pass narrows arm-scoped ports not only for variant payloads but also for predicate-checked values (e.g., `if d != 0` narrows `d`'s type inside `then`). If it does not, DB-11 extends it; that extension is within 3a.3's scope. If the extension turns out to be larger than expected, size 3a.3 to M is preserved by keeping the structural-equality proof theory — *not* by weakening acceptance.

3. **Lane 1e dissolution status** (DB-10). Three emit files still exist at the time of writing. DB-10 wires into all three. When Lane 1e collapses them, the three wirings collapse with no design change.

4. **Cross-module refinement / cross-module generic parameter unification** — out of scope for 3a.3 and 3a.4. `compiler.dag` work predominantly lives within a single module; cross-module is handled by follow-up stages if `compiler.dag`'s growth forces the issue.
