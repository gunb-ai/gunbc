### L-8: Lens Rust surfaces preserve typed failure carriers (2026-04-17)

Public Rust surfaces of migrated `.dag` lenses MUST preserve the
lens's typed failure carrier. When a `.dag` lens computes a typed
result carrier (e.g., `CostLookup`, `Origin`), the public Rust API
that exposes the lens MUST NOT erase that carrier into a panicking
shim or an opaque primitive (`usize`, `bool`, etc.). Callers of the
lens must be able to distinguish `Found(T)` from `Missing(reason)`
at the type level.

**Accepted patterns:**

1. Re-export the generated carrier and query function directly
   (Provenance / CostLens pattern):
   ```rust
   pub use generated::{Carrier, query_fn};
   ```
2. Typed wrapper whose public API uses the same carrier:
   ```rust
   impl CostLens<'a> {
       pub fn cost_of(&self, port: PortId) -> CostLookup { ... }
   }
   ```

**Forbidden patterns:**

1. Collapsing typed miss states to a primitive via panic:
   ```rust
   pub fn cost_of(&self, port: PortId) -> usize { ... panic!(...) }
   ```
2. Silent collapse to a sentinel (`_ => 0`, `_ => false`) — erases
   the distinction without even signaling.

**Why:** the `.dag` lens authored a typed failure carrier because
the miss state is a real outcome the lens computes. Erasing it at
the wrapper loses information callers legitimately need. Panic is
fail-closed but not composable — the caller has no recourse besides
aborting.

Earlier drafts allowed explicitly-named convenience helpers (e.g.
`cost_of_or_zero`) that collapse a carrier to a primitive at a
named boundary. **That exception is removed** (review round 1a.3):
the grep gate forbids every primitive-returning `pub fn` in
`lens_*.rs`, and the prose matches. If a specific caller needs a
collapse, it writes the `match` at its own call-site — three lines
that make the collapse visible in the call graph rather than hiding
behind a wrapper name. Alignment between prose and ratchet is
itself a single-authority concern.

**Mechanical gate:** CI grep check blocks every primitive-returning
public fn in a lens wrapper file:

```
grep -nE "pub fn .*-> (usize|bool|i64)" src/v3/compiler/src/lens_*.rs
```

Scope excludes `lens_depth.rs` (hand-written legacy lens, not
migrated from `.dag`; out of L-8 scope). Zero matches required.
Lens wrappers return typed carriers.

**Origin:** Half A review feedback (2026-04-17) + round 1a.3
prose/ratchet alignment. Full design:
[docs/design-lens-rust-boundary.md](docs/design-lens-rust-boundary.md).
Lane 1 Stage 1a migrated the one offender (`CostLens`); the gate
prevents regressions.

