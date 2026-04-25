# T-ImpossibleBugs nested-optional flatten — design/scoping doc

> **Output of `bright-moth-390` per the redirected scoping brief**
> ([`t-impossiblebugs-nested-optional-flatten-worker.md`](t-impossiblebugs-nested-optional-flatten-worker.md)).
> No code changes. Investigation result + Director-actionable recommendation.

## TL;DR

**Recommendation: bypass-feasible.** v3 substrate already has `TypeConnective::Cardinality { element, bound: CardinalityBound }` as a first-class connective (`src/v3/compiler/src/dag.rs:395`); the cardinality-substrate gate THESIS:343 names is the **v2** state described in `docs/architecture.md:109`, and v3 is past it. `T??` parses today, lowers to nested `Cardinality { bound: AtMostOne, .. }`, and is structurally distinct from `T?` only because nothing collapses the outer wrap. A two-line guard in `lower.rs` closes the bug class without touching substrate. Implementation lane is **XS**.

## Q1 — Surface-upstream check: does `T??` parse?

**Yes, `T??` parses cleanly. No rejection at parse, desugar, or lower.**

Parser (`src/v3/compiler/src/parse_generated.rs:881-892`):

```rust
fn parse_type_expr(&mut self) -> Result<SurfaceType, Diagnostic> {
    let mut ty = self.parse_atom_type()?;
    while matches!(self.peek().kind, TokenKind::Question) {
        let q = self.bump().clone();
        let start = ty.span().byte_start;
        ty = SurfaceType::Optional {
            inner: Box::new(ty),
            span: SourceSpan::new(self.file, start, q.span.byte_end),
        };
    }
    Ok(ty)
}
```

The `while`-loop consumes any number of trailing `?` tokens. `T??` produces `SurfaceType::Optional { inner: SurfaceType::Optional { inner: T } }`. `T???` produces three nested layers. No diagnostic, no error.

Tokenizer (`src/v3/compiler/src/tokenize_generated.rs:240`) maps `b'?'` to `TokenKind::Question`; nothing else gates double-`?`.

**The dissolution does NOT live one layer upstream of algebra.** Surface accepts the nested form by construction.

## Q2 — Substrate-attachment: what's the cardinality state in v3?

**v3 substrate is past the cardinality bridge.** `architecture.md:109` describes the **v2** state — `return_cardinality` enum on Node with 142 construction sites, marked "Bridge — dissolve into edge existence." v3 has done that dissolution: `TypeConnective::Cardinality` is a first-class variant of the connective sum at `src/v3/compiler/src/dag.rs:395-398`:

```rust
Cardinality {
    element: DeclarationId,
    bound: CardinalityBound,
},
```

`CardinalityBound` (`src/v3/compiler/src/dag_scalar_generated.rs:21-25`):

```rust
pub enum CardinalityBound {
    Exact(u32),
    AtMostOne,
    Unbounded,
}
```

Lowering (`src/v3/compiler/src/lower.rs:1949-1968` and parallel arm at `:2044-2047`) maps `SurfaceType::Optional { inner }` to `TypeConnective::Cardinality { element: lower(inner), bound: AtMostOne }`. Recursive: `T??` becomes

```
Cardinality {
    element: <decl whose connective is> Cardinality {
        element: <decl for T>,
        bound: AtMostOne,
    },
    bound: AtMostOne,
}
```

No collapse step exists. The matching code path (`infer.rs:1918-1965`, `bind_expected_decl_to_actual_context`) walks both expected and actual `Cardinality` recursively; nested-optional binds to nested-optional and not to single-optional. So today `T?` and `T??` are *distinct* DAG types.

**Note on `OptionalOf` (algebra.dag:423).** `OptionalOf { inner: AlgebraTypeTemplate }` is a separate construct: it's a variant of the algebra-template enum used in std-method signatures (e.g., `first() -> OptionalOf<ReceiverElement>` at algebra.dag:558). It does *not* participate in user-surface `T?` lowering; user code never produces an `OptionalOf` directly. The two systems are parallel:

- User surface `T?` → `SurfaceType::Optional` → `TypeConnective::Cardinality { bound: AtMostOne }`.
- Std-algebra method declarations → `OptionalOf<X>` (algebra-template only; resolves through algebra-instantiation, not generic type lowering).

Flattening user-surface nested-optional therefore does not touch `OptionalOf`. (If `OptionalOf<OptionalOf<X>>` ever appears in std method declarations as an authoring mistake, that's a separate issue scoped to `dsl/std/algebra.dag`; out of scope here.)

## Q3 — Bypass feasibility: per-construction-site flatten

**Yes — clean two-line guard at `SurfaceType::Optional` lowering arms.**

The algebraic property: `AtMostOne ∧ AtMostOne = AtMostOne`. In partial-function terms: `Option<Option<T>>` is observationally equivalent to `Option<T>` because the cardinality semantics of `AtMostOne` is idempotent under self-composition (`min(1, min(1, n)) = min(1, n)`). This is the math of `AtMostOne`, not invented vocabulary.

**Two arms to edit** (`src/v3/compiler/src/lower.rs`):

1. `:1949-1968` — `type_to_declaration_id` `SurfaceType::Optional` arm: after lowering `inner`, check if `dag.declaration(element).connective` matches `TypeConnective::Cardinality { bound: CardinalityBound::AtMostOne, .. }`. If yes, return `element` directly without wrapping.
2. `:2044-2047` — `type_to_connective` `SurfaceType::Optional` arm: same guard; if inner already AtMostOne-wrapped, return the inner's connective (or its element-as-connective) instead of constructing a fresh wrap.

The guard is **specific to AtMostOne ∧ AtMostOne**. The other shapes are not idempotent and stay distinct:

| Surface | Lowered | Flatten? | Reason |
|---|---|---|---|
| `T??` | `Cardinality<Cardinality<T, AtMostOne>, AtMostOne>` | **YES → `T?`** | `min(1, min(1, n)) = min(1, n)` |
| `List<T>?` | `Cardinality<Cardinality<T, Unbounded>, AtMostOne>` | **NO** | Optional-list ≠ list (`None` ≠ empty list) |
| `T?[]` (`List<T?>`) | `Cardinality<Cardinality<T, AtMostOne>, Unbounded>` | **NO** | List of optionals — outer is Unbounded |
| `[T; 3]?` | `Cardinality<Cardinality<T, Exact(3)>, AtMostOne>` | **NO** | Optional-fixed-array; `None` ≠ array of zeros |

The guard predicate is exactly: outer bound `AtMostOne` *and* inner bound `AtMostOne`. All other combinations stay distinct shapes.

## Q4 — Recommendation: bypass implementation brief shape

**Outcome: (a) bypass-feasible.** Director can fast-track an implementation brief (XS, not S).

### Implementation-brief shape

**Title:** `feat(v3): T-ImpossibleBugs — `T??` flattens to `T?` at lower (closes nested-optional flatten class)`

**Reqs:**

1. **Idempotent flatten in lower.** `SurfaceType::Optional` lowering arms in `src/v3/compiler/src/lower.rs:1949` and `:2044` add a guard: when the lowered inner element's connective is `TypeConnective::Cardinality { bound: AtMostOne, .. }`, return the inner element directly without constructing an outer wrap. Specific to `AtMostOne ∧ AtMostOne`; other bound combinations untouched.
2. **Span policy.** The flattened declaration uses the inner declaration's existing span (the user wrote `T??` syntactically; the type they got is structurally the inner `T?`'s declaration). No info-level "flattened to" hint — *"impossible by construction"* per discipline; the second `?` is silently absorbed at lower. Document this in PR body.
3. **Test fixture.** A v3 compiler test asserting `T??` and `T?` produce the same `DeclarationId` (or the same `TypeConnective` shape modulo declaration identity); pattern-matching against `T??` requires only one level of unwrap. Place under `src/v3/compiler/tests/integration/`.

**STOPs:**

- **Any consumer matches on outer-`AtMostOne` over inner-`AtMostOne` as a meaningful distinct type** (audit `infer.rs` `Cardinality` arms before editing) — if so, that consumer carries a hidden assumption that flatten breaks. STOP and surface.
- **Lowering arm has a hidden caller path that needs the outer wrap for span/identity reasons** — STOP.
- **Generalization pressure to flatten `List<List<T>>` or `Set<Set<T>>` falls out of the same guard** — it does not (those are `Unbounded ∧ Unbounded`, not idempotent in the same algebraic sense; they're legitimately distinct). STOP if the implementer is tempted.
- **DB-8 fixed-point drifts** — STOP immediately.

**Dispatch profile:** XS, single PR, single worker. ~10-line code diff + 1 test fixture + PR-body doc. Not gated on any other lane. Independent of the other two T-ImpossibleBugs classes.

**Acceptance:**

- `T??` and `T?` produce the same lowered shape (test).
- `List<T>?`, `T?[]`, `[T; 3]?` all preserve their distinct shapes (test).
- `cargo test --workspace --exclude v2-compiler-tests` clean.
- `cargo test -p v2-compiler-tests` clean.
- `clippy --all-targets -- -D warnings` clean.
- `cargo fmt --all --check` clean.
- DB-8 fixed-point converges bit-identically.
- SG-0 census deltas as needed.

### Why not "needs upstream substrate design"

The cardinality substrate THESIS:343 names as the gate is the **v2** state (`return_cardinality` enum, 142 sites). v3 has already promoted Cardinality to a first-class connective. There is no upstream substrate work to do — the substrate is in place; only the idempotent-meet rule at the construction site is missing. Adding that rule is not substrate redesign; it's a one-site invariant attached to existing substrate.

### Why not "park"

The bypass is small (XS), well-scoped (specific to `AtMostOne ∧ AtMostOne`), structurally grounded (algebraic idempotence of `AtMostOne`), and the substrate prerequisite is already met. Parking would be conservative beyond what the evidence supports.

## Cross-manager note

- **Zero-Floor Manager:** no current overlap. The bypass is a lower-stage rule, not substrate-extension; SG-0 census may show a delta if the test fixture lands.
- **Grounding Manager:** no overlap.

## Closing signal

`bright-moth-390` recommends Director author the bypass implementation brief described in §Q4 and dispatch to the next idle worker. No upstream blockers identified. The cardinality-substrate gate THESIS:343 names is closed (in v3) by virtue of `TypeConnective::Cardinality` being a first-class connective, not a bridge enum.
