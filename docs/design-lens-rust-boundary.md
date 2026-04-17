> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Unblocks: Lane 1 Stage 1a (CostLens migration); future L2 lens migrations

# Design DB-10 — Lens Rust-boundary contract

**Design blocker:** DB-10 (added post-Half-A review)
**Consumers:** Lane 1 Stage 1a (migrates the one current offender: `CostLens`); every future `.dag` lens migration
**Status:** Design ready for implementer review.
**Origin:** Half A review feedback (2026-04-17), design question: *"What is the canonical public Rust boundary for migrated `.dag` lenses: direct exposure of the generated carrier types, or handwritten compatibility shims?"*

---

## Problem

Half A migrated two lenses. The two converged on different Rust-boundary patterns:

**Provenance** (canonical): re-exports generated carriers directly.
```rust
// src/v3/compiler/src/lib.rs
pub use lens_provenance::{origin_of, Origin};
```
Callers see `Origin::NoProducer`, `Origin::MissingPort`, `Origin::MissingBehavior`, etc. — typed miss states preserved. Fail-closed at the type level.

**Cost** (shim): handwritten adapter that collapses the typed carrier.
```rust
// src/v3/compiler/src/lens_cost.rs
pub struct CostLens<'a> { dag: &'a Dag }

impl<'a> CostLens<'a> {
    pub fn cost_of(&self, port: PortId) -> usize {
        match generated::cost_of(self.dag, &port) {
            generated::CostLookup::FoundCost { _0: cost } => usize::try_from(cost).expect(...),
            generated::CostLookup::MissingCost => panic!("malformed Dag ..."),
        }
    }
}
```
The `.dag` lens computes `MissingCost` as a legitimate typed fact; the wrapper throws it away and panics. Callers can't handle malformed references; they crash.

Reviewers flagged this (correctly) as debt:
- Fail-closed wins don't survive to the public Rust surface
- `MissingCost` is forbidden at the wrapper boundary — typed signal erased
- If this pattern repeats per lens, future L2 migrations accrete one-off adapters; the compiled-lens boundary diverges

**This design fixes the convention before more lenses land.**

---

## Design

### Invariant L-8

Add to `INVARIANTS.md`:

```markdown
**L-8 — Public Rust surfaces of migrated `.dag` lenses preserve the lens's typed failure carrier.**

When a `.dag` lens computes a typed result carrier (e.g., `CostLookup`,
`Origin`, `IdempotencyReport`), the public Rust API that exposes the
lens MUST NOT erase that carrier into a panicking shim or an opaque
primitive (`usize`, `bool`, etc.). Callers of the lens must be able
to distinguish `Found(T)` from `Missing(reason)` at the type level.

**Accepted patterns:**
1. Re-export the generated carrier and the compiled entry directly
   (Provenance pattern):
   ```rust
   pub use lens_<name>::{Carrier, query_fn};
   ```
2. Expose a typed wrapper whose public API uses the same carrier:
   ```rust
   pub struct CostLens<'a> { dag: &'a Dag }
   impl CostLens<'a> {
       pub fn cost_of(&self, port: PortId) -> CostLookup {
           lens_cost::cost_of(self.dag, &port)
       }
   }
   ```

**Forbidden patterns:**
1. Collapsing typed miss states to a primitive:
   ```rust
   // FORBIDDEN — `MissingCost` is a lens fact, not a programmer error
   pub fn cost_of(&self, port: PortId) -> usize { ... panic!(...) }
   ```
2. Suppressing via unwrap chains:
   ```rust
   // FORBIDDEN — same issue, different surface
   pub fn cost_of(&self, port: PortId) -> usize {
       match generated::cost_of(self.dag, &port) {
           generated::CostLookup::FoundCost { _0: c } => c as usize,
           _ => 0, // silent miss collapse — even worse than panic
       }
   }
   ```

**Why:** the `.dag` lens authored a typed failure carrier because the
miss state is a real outcome the lens computes. Erasing it at the
wrapper loses information that callers legitimately need. Panic is
fail-closed but not composable — the caller has no recourse besides
aborting.

**Mechanical gate:** CI grep check blocks new unwrap/panic patterns in
public lens wrappers:
```
grep -nE "fn.*Lens.*-> (usize|bool|i64)" src/v3/compiler/src/lens_*.rs
```
Zero matches required. Lens wrappers return typed carriers.

**Origin:** Half A review feedback 2026-04-17.
```

### Canonical shape: direct re-export

Prefer **direct re-export** of the generated types when the .dag lens's API shape is already the public contract:

```rust
// src/v3/compiler/src/lens_cost.rs (post-migration)
mod generated {
    #![allow(dead_code, unused_imports, unused_parens, unused_variables,
             clippy::clone_on_copy, clippy::collapsible_else_if)]
    use crate::dag::*;
    use crate::diagnostics::*;
    include!("lens_cost_generated.rs");
}

pub use generated::{CostLookup, cost_of};
```

That's the whole file. No struct wrapper, no method, no panic.

### Alternative: typed wrapper struct (when stateful convenience needed)

If a wrapper struct is useful (caches, `'a` lifetimes, multi-method API), keep its methods typed:

```rust
pub struct CostLens<'a> {
    dag: &'a Dag,
}

impl<'a> CostLens<'a> {
    pub fn new(dag: &'a Dag) -> Self {
        Self { dag }
    }

    // Returns the typed carrier, not usize
    pub fn cost_of(&self, port: PortId) -> CostLookup {
        generated::cost_of(self.dag, &port)
    }

    // Convenience accessor that's EXPLICIT about the miss case
    pub fn cost_of_or_zero(&self, port: PortId) -> i64 {
        match self.cost_of(port) {
            CostLookup::FoundCost { _0: c } => c,
            CostLookup::MissingCost => 0,
        }
    }
}
```

Note: `cost_of_or_zero` is ALLOWED because its name makes the collapse explicit. It's not pretending to be the real lens result; it's a derived convenience. Callers choosing `cost_of_or_zero` are opting into zero-for-miss semantics; callers needing distinction use `cost_of`.

### Why Provenance already complies

Provenance follows the direct-re-export pattern (post-Half-A):

```rust
// src/v3/compiler/src/lib.rs (excerpt)
pub use lens_provenance::{origin_of, Origin};
```

`Origin` has variants `NoProducer`, `MissingPort`, `MissingBehavior`, `Source(...)`, `Computed(...)`, `Selected(...)`, `Accumulated(...)`. Callers see all miss states.

### Migration path for CostLens (Lane 1 Stage 1a)

**Today:**
```rust
pub struct CostLens<'a> { dag: &'a Dag }
impl<'a> CostLens<'a> {
    pub fn cost_of(&self, port: PortId) -> usize {
        match generated::cost_of(self.dag, &port) {
            generated::CostLookup::FoundCost { _0: cost } => usize::try_from(cost).expect("..."),
            generated::CostLookup::MissingCost => panic!("..."),
        }
    }
}
```

**After L-8 migration:**
```rust
mod generated { ... }
pub use generated::{CostLookup, cost_of};
```

Or if the wrapper struct is preferred for consistency with other APIs:
```rust
pub struct CostLens<'a> { dag: &'a Dag }
impl<'a> CostLens<'a> {
    pub fn new(dag: &'a Dag) -> Self { Self { dag } }
    pub fn cost_of(&self, port: PortId) -> CostLookup {
        generated::cost_of(self.dag, &port)
    }
}
```

Test callers update from `assert_eq!(lens.cost_of(port), 3)` to `assert_eq!(lens.cost_of(port), CostLookup::FoundCost { _0: 3 })` or match on the result.

### Interaction with the consolidated walker (Lane 1 Stage 1e)

When Lane 1e lands, the emitter itself is a lens-like consumer of substrate facts. The walker's public API (`emit(dag, target)`) returns `Result<EmittedSource, EmitError>` — typed error variants, NOT panic. Same principle as L-8.

Existing `EmitError` variants in `src/v3/compiler/src/emit_rust.rs` already follow this pattern. L-8 confirms it: don't regress during consolidation.

---

## Rationale

**Why the Provenance pattern as canonical (direct re-export)?** Because the `.dag` lens IS the authority. The generated Rust is a projection. A handwritten wrapper between the two creates a second authority (the wrapper's return type + error handling diverge from the `.dag` lens's semantics). Direct re-export means the public API IS the lens.

**Why allow the wrapper struct pattern at all?** Because some lenses benefit from stateful caching, lifetime-tying, or grouping methods under a namespace (`FooLens::analyze`, `FooLens::recommend`). Wrappers are fine — as long as their methods preserve the typed carrier.

**Why is `cost_of_or_zero` acceptable while `cost_of → usize` is not?** Because of naming honesty. `cost_of_or_zero` announces the collapse in its name; a caller picking it is agreeing to lose the miss information. `cost_of → usize` lies — the name suggests "the cost of this port" but the function will panic if no cost exists. Honest-collapse accessors compose; shim-panic accessors don't.

**Why panic-at-wrapper not panic-at-substrate?** Because the substrate's `dag.port(id)` panics on unknown `PortId` to enforce "malformed substrate is an invariant violation." That IS fail-closed: reaching an unknown id means the Dag was malformed at construction. But the LENS'S `MissingCost` is NOT that — the lens explicitly computes that state for legitimate cases (reference to a port whose producer isn't in `d.nodes`). The lens says "this is a legal state I care about." Erasing it at the wrapper violates the lens's own semantics.

---

## Rejected alternatives

**Make panic-on-miss the canonical pattern** — loses caller composability; user programs can't recover. Rejected.

**Return `Option<T>` instead of typed carriers** — drops the REASON for the miss. `MissingCost` vs `MissingPort` vs `MissingBehavior` carry different semantics; collapsing to `None` throws that away. Rejected.

**Require every lens to return `Result<T, LensError>`** — imposes a uniform error type where each lens has its own taxonomy. The `.dag` lens already declares the carrier; just re-export it. Rejected.

**Keep the existing inconsistency (provenance re-exports, cost shims) as "implementer choice"** — that's the current state; reviewer correctly flagged it as drift. Rejected.

**Add a lint / macro that auto-generates the wrapper** — premature tooling for a 2-line API. Rejected; direct re-export is simpler.

---

## Implementation notes

### Lane 1 Stage 1a checklist addition

1. Rewrite `src/v3/compiler/src/lens_cost.rs` to re-export `CostLookup` + `cost_of` directly
2. Update test callers that used `CostLens::cost_of`:
   - `src/v3/compiler/tests/m1_3_lens_cost_test.rs` — update assertions to match on `CostLookup`
   - Any other consumer
3. Remove the `CostLens` struct if no methods beyond `cost_of` remain (or keep as empty wrapper if consumers expect the name; all methods typed)
4. Verify: `grep -n "-> usize" src/v3/compiler/src/lens_cost.rs` returns zero
5. Verify: `cargo test -p v3-compiler` passes

### CI gate

Add to CI workflow (pairs with L-7 gate):

```yaml
- name: L-8 lens boundary check
  run: |
    OFFENDERS=$(grep -rnE "fn.*Lens.*-> (usize|bool|i64)" src/v3/compiler/src/lens_*.rs || true)
    if [ -n "$OFFENDERS" ]; then
      echo "L-8 violation — lens wrappers must return typed carriers:"
      echo "$OFFENDERS"
      exit 1
    fi
```

### Documentation of decision per lens

Each lens's `lens_<name>.rs` file should carry a short header explaining which pattern it follows and why:

```rust
// lens_cost.rs
//
// Public Rust API: direct re-export of the generated carrier (L-8).
// Callers receive `CostLookup = MissingCost | FoundCost(Int)` — they
// must handle the miss case explicitly. No panic path at this
// boundary.
```

### Pattern for future lenses (Lane 2)

Lane 2 adds `idempotency`, `symbolic_cost`, `parallelism`, and (potentially) `side_effects`, `space_bounds`. All new lenses follow L-8 from day one:

- `lens_idempotency.rs` re-exports `idempotency_report` + `WorkflowIdempotencyReport`
- `lens_symbolic_cost.rs` re-exports `symbolic_cost_of` + `SymbolicCost`
- `lens_parallelism.rs` re-exports `analyze_parallelism` + `ParallelizationOpportunity`

No shim shims.

---

## Associations

- **Lane 1 Stage 1a** ([phase1-lane1-l15-tail.md](./phase1-lane1-l15-tail.md)) — implements this for the cost lens (the one current offender)
- **Lane 1 Stage 1e** (walker) — same principle applies; `emit()` returns typed `Result<T, EmitError>`, not panic
- **Lane 2 all stages** ([lane2-compile-time-proofs.md](./lane2-compile-time-proofs.md)) — new lenses follow L-8 from inception
- **DB-5 Substrate keyed-lookup** ([design-substrate-keyed-lookup-api.md](./design-substrate-keyed-lookup-api.md)) — note: substrate accessors (`dag.port(id) -> &DagPort`) panic on unknown id — that's a DIFFERENT contract (invariant-level). L-8 governs LENS wrappers, not substrate accessors.
- **Update `src/v3/compiler/src/lens_cost.rs`** — migrate
- **Update `INVARIANTS.md`** — add L-8
- **Update `.github/workflows/ci.yml`** — grep gate
- **Origin** — Half A review feedback (2026-04-17 APPROVE_WITH_COMMENTS)

---

## Acceptance

- [ ] `INVARIANTS.md` L-8 entry added
- [ ] `src/v3/compiler/src/lens_cost.rs` re-exports `CostLookup` + `cost_of` directly (or keeps a typed wrapper that returns `CostLookup`, not `usize`)
- [ ] No `fn *Lens* -> usize` in `src/v3/compiler/src/lens_*.rs`
- [ ] CI grep gate passes
- [ ] Test callers updated to match on `CostLookup`
- [ ] Header comment in each `lens_*.rs` file documenting the L-8 pattern

---

## Open questions

1. **What about `DiagnosticReport` carriers?** Lens diagnostic emission already returns `Vec<Diagnostic>`; L-8 applies consistently (don't collapse diagnostic reasons). Confirmed: Diagnostic vector stays typed.

2. **Are there cases where a lens genuinely doesn't have a miss state?** E.g., a pure count lens that always returns a number. Yes — those don't need typed carriers because there's no failure mode. L-8 applies only when the `.dag` lens computes a typed miss state.

3. **What if a user-declared Dimension (Lane 2 Stage 2f / Lane 4 Stage 4b-4c) has complex carrier?** User-declared dimensions use the generic `DimensionReport<Carrier>` surface from DB-3. Public Rust API re-exports that directly. L-8 upheld by construction.
