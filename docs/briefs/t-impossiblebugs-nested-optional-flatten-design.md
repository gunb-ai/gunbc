# T-ImpossibleBugs nested-optional flatten — design/scoping doc

> **Output of `bright-moth-390` per the redirected scoping brief**
> ([`t-impossiblebugs-nested-optional-flatten-worker.md`](t-impossiblebugs-nested-optional-flatten-worker.md)).
> No code changes. Investigation result + Director-actionable recommendation.

## TL;DR

**Recommendation: bypass-feasible.** v3 substrate already has `TypeConnective::Cardinality { element, bound: CardinalityBound }` as a first-class connective (`src/v3/compiler/src/dag.rs:395`); the cardinality-substrate gate THESIS:343 names is the **v2** state described in `docs/architecture.md:109`, and v3 is past it. `T??` parses today, lowers to nested `Cardinality { bound: AtMostOne, .. }`, and is structurally distinct from `T?` only because nothing collapses the outer wrap. The fix is (a) one predicate `cardinality_idempotent_target` owning the `AtMostOne ∧ AtMostOne` rule, (b) one allocator `alloc_cardinality_decl` that consults it, and **(c) API closure on `TypeConnective::Cardinality`'s payload** so the variant cannot be constructed outside the allocator — mechanical enforcement of "illegal states unrepresentable," not by-convention. Implementation lane is **M** (revised from S after API-closure requirement landed; see §Q3 *API closure*). (Original draft proposed lower-only XS, then constructor-helper-by-convention S; revised post-PR #798 review — see §Q3 *Revision*.)

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

## Q3 — Bypass feasibility: substrate-constructor invariant

**Yes — single helper enforced at every `TypeConnective::Cardinality` construction site.**

The algebraic property: `AtMostOne ∧ AtMostOne = AtMostOne`. In partial-function terms: `Option<Option<T>>` is observationally equivalent to `Option<T>` because the cardinality semantics of `AtMostOne` is idempotent under self-composition (`min(1, min(1, n)) = min(1, n)`). This is the math of `AtMostOne`, not invented vocabulary.

### Revision per PR #798 review (2026-04-25)

The initial framing of this section proposed a lower-only guard at the two `SurfaceType::Optional` arms. **A reviewer (gpt-5-5-pro, blocking) correctly flagged that this leaves the illegal state representable through other DAG construction paths**, violating "illegal states unrepresentable" and reducing the rule to API-level enforcement.

Audit of all hand-Rust `TypeConnective::Cardinality` construction sites confirms multiple paths:

| File:line | Path | Hand-Rust? |
|---|---|---|
| `src/v3/compiler/src/lower.rs:1949-1968` | `type_to_declaration_id` `SurfaceType::Optional` arm | yes |
| `src/v3/compiler/src/lower.rs:2044-2047` | `type_to_connective` `SurfaceType::Optional` arm | yes |
| `src/v3/compiler/src/infer.rs:2902-2916` | `concretize_decl_with_subst` substitutes through Cardinality | **yes — killer case** |
| `src/v3/compiler/src/dag/builder.rs:920` | `push_test_declaration` (test scaffolding) | test-only |
| `src/v3/compiler/src/bootstrap_std_generated.rs` (~22 sites) | regenerated from std/ declarations | generated |

The `infer.rs:2902` path is the substantive blocker: when `fn foo<T>(x: T?) -> T?` is instantiated with `T = Int?`, substitution produces `Cardinality<Cardinality<Int, AtMostOne>, AtMostOne>` *here*, never passing through `lower.rs`. A lower-only guard misses every generic-instantiation case.

### Corrected design: single substrate-level helper (canonical sketch)

Introduce one helper in `src/v3/compiler/src/dag/builder.rs` (or adjacent — implementer's call) enforcing the rule at allocation, plus one predicate that owns the rule itself. All construction paths — including the codegen emitter for `bootstrap_std_generated.rs` (~22 sites) — route through the helper. Nested `AtMostOne ∧ AtMostOne` literally cannot be constructed, satisfying "illegal states unrepresentable" structurally rather than by-convention.

One predicate, one allocator helper, three (+ codegen) call sites:

```rust
// The predicate. The ONLY place the AtMostOne∧AtMostOne rule lives.
fn cardinality_idempotent_target(
    dag: &Dag, element: DeclarationId, bound: CardinalityBound,
) -> Option<DeclarationId>;
// Returns Some(inner_decl) when the rule fires (outer == AtMostOne AND
// element's connective is Cardinality { bound: AtMostOne, .. }); else None.

// The allocator. Single substrate-constructor authority for Cardinality
// declarations.
fn alloc_cardinality_decl(
    dag: &mut Dag, element: DeclarationId, bound: CardinalityBound, span: SourceSpan,
) -> DeclarationId {
    if let Some(inner) = cardinality_idempotent_target(dag, element, bound) {
        return inner;
    }
    // ... allocate fresh declaration with TypeConnective::Cardinality { element, bound }
}
```

**All call sites consume `alloc_cardinality_decl`** (not the predicate directly, not literal `TypeConnective::Cardinality { ... }`):

- `lower.rs:1949` (`type_to_declaration_id` Optional arm) — direct call, returns `DeclarationId`.
- `lower.rs:2044` (`type_to_connective` Optional arm) — restructure the caller (`type_to_declaration_id`'s enclosing wrap-and-allocate) to call `alloc_cardinality_decl` instead of constructing a `TypeConnective::Cardinality` and pushing a declaration around it. The `:2044` arm's literal connective construction is the path of last resort and is replaced.
- `infer.rs:2902` (`concretize_decl_with_subst`) — direct call, returns `DeclarationId`.
- `regen_bootstrap_emit.rs` (codegen for `bootstrap_std_generated.rs`) — emit calls to `alloc_cardinality_decl` instead of literal `TypeConnective::Cardinality { ... }` struct-init in regen output.

**Single-authority requirement (per PR #798 Codex review #1):** the rule lives in exactly one place — `cardinality_idempotent_target`. No sibling helper that re-applies the rule. No literal `TypeConnective::Cardinality` construction outside `alloc_cardinality_decl`'s body. A second authority violates INVARIANTS.md P2 Boundary Discipline.

### API closure: mechanical, not conventional (per PR #798 Codex review #2)

A second Codex review correctly flagged that "all call sites consume `alloc_cardinality_decl`" is **convention-level** — a future caller can still write literal `TypeConnective::Cardinality { element, bound }` directly. Per `docs/modeling-discipline.md` practice 6 (API-level enforcement over convention), the substrate invariant must be mechanically enforced: the constructor API must be closed so the variant **cannot be constructed** outside `alloc_cardinality_decl`.

Concretely, in Rust this means refactoring `TypeConnective::Cardinality`'s payload from a struct-variant with public fields:

```rust
// BEFORE — payload is publicly constructible anywhere TypeConnective is in scope.
enum TypeConnective {
    Cardinality { element: DeclarationId, bound: CardinalityBound },
    ...
}
```

…to a tuple-variant carrying a payload struct whose constructor is module-private:

```rust
// AFTER — payload's only constructor is `pub(super) fn new` reachable
// only from the module that owns alloc_cardinality_decl. Callers outside
// the module CANNOT construct CardinalityPayload, therefore CANNOT
// construct TypeConnective::Cardinality. Pattern-matching destructure
// stays public via accessor methods.
mod cardinality_payload {
    pub struct CardinalityPayload {
        element: DeclarationId,
        bound: CardinalityBound,
    }
    impl CardinalityPayload {
        pub(super) fn new(element: DeclarationId, bound: CardinalityBound) -> Self {
            Self { element, bound }
        }
        pub fn element(&self) -> DeclarationId { self.element }
        pub fn bound(&self) -> &CardinalityBound { &self.bound }
    }
}
enum TypeConnective {
    Cardinality(CardinalityPayload),
    ...
}
```

`alloc_cardinality_decl` becomes the **only** function that calls `CardinalityPayload::new`. Nested `AtMostOne ∧ AtMostOne` is then mechanically unrepresentable — no caller, present or future, can bypass the predicate. This is "illegal states unrepresentable" by type-system enforcement, not by-convention/by-review.

**Scope impact:** every existing destructure of `TypeConnective::Cardinality { element, bound }` (currently struct-variant style, ~30 sites grepped) becomes `TypeConnective::Cardinality(p) => { let element = p.element(); let bound = p.bound(); ... }` or equivalent. Mechanical, but pervasive.

**Lane re-size:** API closure bumps the lane from S → M (~30-line core diff plus pattern-match migration across the compiler). The reviewer offered an alternative — keep the public-literal surface bounded with a dissolution trigger — but this contradicts modeling-discipline practice 6 and `feedback_construction_over_ratchets` (model first; violations dissolve). The recommendation is mechanical closure (M lane), not bounded scaffold. **Director-call** if M is too large for this slot — explicitly noted, not deferred to implementer.

The guard is **specific to AtMostOne ∧ AtMostOne**. The other shapes are not idempotent and stay distinct:

| Surface | Lowered | Flatten? | Reason |
|---|---|---|---|
| `T??` | `Cardinality<Cardinality<T, AtMostOne>, AtMostOne>` | **YES → `T?`** | `min(1, min(1, n)) = min(1, n)` |
| `List<T>?` | `Cardinality<Cardinality<T, Unbounded>, AtMostOne>` | **NO** | Optional-list ≠ list (`None` ≠ empty list) |
| `T?[]` (`List<T?>`) | `Cardinality<Cardinality<T, AtMostOne>, Unbounded>` | **NO** | List of optionals — outer is Unbounded |
| `[T; 3]?` | `Cardinality<Cardinality<T, Exact(3)>, AtMostOne>` | **NO** | Optional-fixed-array; `None` ≠ array of zeros |

The guard predicate is exactly: outer bound `AtMostOne` *and* inner bound `AtMostOne`. All other combinations stay distinct shapes.

## Q4 — Recommendation: bypass implementation brief shape

**Outcome: (a) bypass-feasible.** Director authors an implementation brief at lane size **M** (see §Dispatch profile below; sized to include API closure per §Q3 *API closure*. Earlier drafts sized XS / S correspond to rejected designs — lower-only sugar and helper-by-convention respectively).

### Implementation-brief shape

**Title:** `feat(v3): T-ImpossibleBugs — `T??` flattens to `T?` via Cardinality-constructor invariant (closes nested-optional flatten class)`

**Reqs:**

1. **Substrate-level idempotence helper + API closure.** Add `alloc_cardinality_decl` (or equivalent) in `src/v3/compiler/src/dag/builder.rs` enforcing `AtMostOne ∧ AtMostOne = AtMostOne` at allocation time. When `bound == AtMostOne` and `element`'s connective is `Cardinality { bound: AtMostOne, .. }`, return `element` directly without allocating a fresh outer wrap. Specific to `AtMostOne ∧ AtMostOne`; `Unbounded`, `Exact(n)`, and mixed-bound combinations untouched. **Do NOT** peel to `element`-as-element (would collapse `T??` to `T` instead of `T?` and contradict acceptance below) — the helper returns the existing inner *declaration* (its connective is already `Cardinality<T, AtMostOne>`), not its element-of-element. **Additionally close the `TypeConnective::Cardinality` constructor API** per §Q3 *API closure* — refactor the payload to a module-private struct so the variant cannot be constructed outside `alloc_cardinality_decl`. Pattern-match destructure stays public via accessor methods on the payload.
2. **Route ALL Cardinality construction paths through `alloc_cardinality_decl`** per the §Q3 canonical sketch: `lower.rs:1949`, `lower.rs:2044` (via caller restructure), `infer.rs:2902`, and `regen_bootstrap_emit.rs` (codegen emits helper calls, not literal `TypeConnective::Cardinality { ... }` struct-init in `bootstrap_std_generated.rs`). The rule lives only in `cardinality_idempotent_target`. **No sibling helper. No literal `TypeConnective::Cardinality` construction outside `alloc_cardinality_decl`'s body** (including in regen output). Single authority for the substrate invariant.
3. **Span policy.** When flatten triggers, reuse the inner declaration's span (the user wrote `T??`; the type they got is the inner `T?`'s declaration). No info-level "flattened to" hint — *"impossible by construction"* per discipline; the second `?` is silently absorbed at the substrate constructor. Verify no diagnostic / hover / error-printer round-trips `T??` back to the user as `T?` confusingly (per gpt-5-5-pro non-blocking observation on PR #798).
4. **Test fixtures.**
   - **Shape assertion (not identity).** `T??` lowered shape contains no nested `Cardinality { bound: AtMostOne, .. }` whose `element`'s connective is also `Cardinality { bound: AtMostOne, .. }` — i.e. a single `AtMostOne` wrap around the underlying type, equivalent to `T?`'s shape. Assert via structural walk over the resulting declaration, **not** raw `DeclarationId` equality with an independently lowered `T?` (declaration identity is an interning concern; the invariant is shape, per `TESTING.md` behavior-driven discipline).
   - Generic-instantiation case: `fn foo<T>(x: T?) -> T?` instantiated with `T = Int?` produces a return-type shape with one `AtMostOne` wrap around `Int`, **not** two nested `AtMostOne` wraps (test that exercises `concretize_decl_with_subst`; this is the path the original lower-only design missed). Same shape-level assertion, not `DeclarationId`-level.
   - `List<T>?`, `T?[]`, `[T; 3]?` preserve their distinct shapes (test).
   - Place under `src/v3/compiler/tests/integration/`.

**STOPs:**

- **Any consumer matches on outer-`AtMostOne` over inner-`AtMostOne` as a meaningful distinct type** (audit `infer.rs` `Cardinality` arms before editing) — if so, that consumer carries a hidden assumption that flatten breaks. STOP and surface.
- **A construction path was missed in the audit** — if `cargo test` reveals nested `Cardinality<_, AtMostOne>` produced by a path other than the three hand-Rust sites named, STOP and audit before extending the helper rollout.
- **Diagnostic/hover/error-printer surface confuses the user** — if any user-facing surface re-prints types and round-trips `T??` ambiguously, STOP and surface (per gpt-5-5-pro observation).
- **Generalization pressure to flatten `List<List<T>>` or `Set<Set<T>>` falls out of the same helper** — it does not (those are `Unbounded ∧ Unbounded`, not idempotent in the same algebraic sense; they're legitimately distinct). The helper's predicate is exactly `outer == AtMostOne && inner == AtMostOne`. STOP if the implementer is tempted to widen.
- **`OptionalOf<OptionalOf<X>>` in std-method authoring** is a separate brief (algebra-template lint), not in scope here. STOP if the implementer is tempted to fold it in.
- **DB-8 fixed-point drifts** — STOP immediately.

**Dispatch profile:** M. Single PR, single worker. Core diff: one predicate + `alloc_cardinality_decl` helper + `CardinalityPayload` module-private payload + accessor methods + 4 routed construction paths (`lower.rs:1949`, `lower.rs:2044` via caller restructure, `infer.rs:2902`, `regen_bootstrap_emit.rs` codegen). Plus pattern-match migration: every existing `TypeConnective::Cardinality { element, bound }` destructure (~30 sites grepped) → `TypeConnective::Cardinality(p) => { ... p.element() ... p.bound() ... }`. Mechanical but pervasive. 3 test fixtures + PR-body doc. Not gated on any other lane. Independent of the other two T-ImpossibleBugs classes. **Director-call**: if M is too large for this slot, the alternative is bounded-scaffold convention with a tracked dissolution trigger to API closure — but that contradicts modeling-discipline practice 6 and `feedback_construction_over_ratchets`. Recommendation is M lane, mechanical closure.

**Acceptance:**

- `T??` lowers to a shape with no nested `Cardinality<_, AtMostOne>` (test; structural walk, not `DeclarationId` equality).
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

The bypass is small (S; see §Dispatch profile), well-scoped (specific to `AtMostOne ∧ AtMostOne`), structurally grounded (algebraic idempotence of `AtMostOne`), and the substrate prerequisite is already met. Parking would be conservative beyond what the evidence supports.

## Cross-manager note

- **Zero-Floor Manager:** no current overlap. The bypass is a lower-stage rule, not substrate-extension; SG-0 census may show a delta if the test fixture lands.
- **Grounding Manager:** no overlap.

## Closing signal

`bright-moth-390` recommends Director author the bypass implementation brief described in §Q4 and dispatch to the next idle worker. No upstream blockers identified. The cardinality-substrate gate THESIS:343 names is closed (in v3) by virtue of `TypeConnective::Cardinality` being a first-class connective, not a bridge enum.
