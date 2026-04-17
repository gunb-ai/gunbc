> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Unblocks: Scheduled deletion of `declaration_by_name` + NameKeyedReference bridges | Consumes: compiler-as-dependency-analyzer framing

# Design DB-17 — Reference-resolution provenance

**Design blocker:** DB-17 (substrate amendment that records HOW a resolved identifier was resolved — structural walk vs name-lookup fallback)
**Consumer:** `lens_structural_resolution.dag` (joint enforcement for `declaration_by_name` and NameKeyedReference bridges)
**Status:** Proposed. Awaiting review.
**Origin:** scheduled-deletions reclassification pass (2026-04-17). Every scaffold in the "Needs substrate amendment" column needed a concrete DB; `declaration_by_name` + NameKeyedReference bridges cluster as one class, unlocked by this DB.

---

## Problem

Today, when lowering resolves an identifier, the resulting `AtomPayload::ResolvedIdentifier(DeclarationId)` carries the *result* of resolution (the `DeclarationId`) but not the *provenance* — the path by which resolution succeeded. Two very different code paths produce the same substrate shape:

1. **Structural walk** — lowering follows parent-chain / scope-chain edges from the reference site to a declaration reachable through structural parentage. This is the authoritative path: the reference is grounded in the compiler's dependency DAG.
2. **Name fallback** — lowering calls `declaration_by_name(name)` (or an analogous helper that maps a `String` to a `DeclarationId` via a name table). This works today for std/bootstrap lookups where cross-module structural resolution isn't yet wired, but it's a bridge: the reference reaches a declaration by **string identity**, not by structural edge.

Post-resolution, both produce `AtomPayload::ResolvedIdentifier(id)`. A lens walking `d.nodes` cannot distinguish them. Any bridge that fell back through `declaration_by_name` becomes invisible.

This blocks the scheduled deletion of `declaration_by_name`: to dissolve it, we need to enumerate its uses, audit that every legitimate use has a structural-walk alternative, and enforce that no new name-fallback resolutions enter user-range code. Without provenance in the substrate, "enumerate uses" collapses to grep over compiler source — which is exactly the heuristic-over-source failure mode the dependency-analyzer framing rules out.

More broadly: the **dependency graph is incomplete until every reference is grounded by a structural edge**. Name-keyed resolution is a bridge that bypasses the edge. The compiler-as-dependency-analyzer thesis requires that this bridge be either visible or absent.

---

## Design

### Split `AtomPayload::ResolvedIdentifier` into two variants

Today (substrate schema, simplified):

```rust
pub enum AtomPayload {
    UnresolvedIdentifier(String),
    ResolvedIdentifier(DeclarationId),
    TypeParam(String),
    Literal(LiteralBits),
    // ...
}
```

After DB-17:

```rust
pub enum AtomPayload {
    UnresolvedIdentifier(String),
    ResolvedByStructure(DeclarationId),    // NEW — resolved via structural walk
    ResolvedByName(DeclarationId),          // NEW — resolved via name fallback
    TypeParam(String),
    Literal(LiteralBits),
    // ...
}
```

`ResolvedIdentifier` is removed; every call site that produced it picks which variant applies. Lowering that walks a structural edge produces `ResolvedByStructure`; lowering that calls `declaration_by_name` (or any name-keyed helper) produces `ResolvedByName`.

### Consumer migration

Every consumer that currently matches on `ResolvedIdentifier(id)` updates to match both variants. For consumers that don't care about provenance, this is mechanical:

```rust
// Before
AtomPayload::ResolvedIdentifier(id) => use(id),

// After
AtomPayload::ResolvedByStructure(id) | AtomPayload::ResolvedByName(id) => use(id),
```

A small helper (`resolved_id(payload) -> Option<DeclarationId>`) can reduce boilerplate for consumers that uniformly ignore provenance. But the default is explicit pattern-matching — the provenance fact is visible at every read site, which is the whole point.

### Lens enforcement unlocks

Once DB-17 lands, `lens_structural_resolution.dag` gets a second variant:

```dag
type StructuralResolutionViolation
  = UnresolvedArrowBody { arrow: DeclarationRef }
  | NameKeyedReference { site: DeclarationRef, resolved_to: DeclarationRef }

// A NameKeyedReference violation fires whenever ResolvedByName is reachable
// from a user-range root. Bootstrap/std ranges are exempt until M2 module
// scoping lands (at which point all resolutions are structural).
```

The lens walks `d.nodes`, traverses Atom payloads, and reports any `ResolvedByName` found in user-range code. **No grep, no allowlist, no string heuristic** — the substrate fact carries the information the lens queries.

### Dissolution of `ResolvedByName`

`ResolvedByName` is itself a scheduled-deletion row in the ROADMAP. Dissolution trigger: M2 module scoping lands. At that point every reference resolves structurally; the variant becomes unreachable; a final PR removes it from the substrate.

Between DB-17 landing and M2 module scoping landing, `ResolvedByName` is the explicit bridge — visible, countable, lens-enforced. No silent drift.

---

## Rejected alternatives

- **Add `provenance: ResolutionProvenance` metadata to a single `ResolvedIdentifier` variant.**
  ```rust
  ResolvedIdentifier { decl: DeclarationId, provenance: ResolutionProvenance }
  ```
  Rejected because the fact belongs at the variant level, not as a metadata field. The substrate's coproduct axis captures "distinct kinds of reference"; folding provenance inside loses structural visibility. Also: consumers that care about provenance must unwrap the metadata; consumers that don't care read `decl` directly, which silently ignores provenance. The variant split makes every read site acknowledge provenance explicitly.

- **Side table: map of `(Port, DeclarationId) → ResolutionProvenance`.**
  Parallel authority (violates D-2). The substrate's Atom is the authority for resolution; a side table forks the fact. Rejected.

- **Grep over compiler source for `declaration_by_name` call sites.**
  Heuristic over source text; can't distinguish sanctioned bootstrap path from user-range bridge; falls to renames and helper indirection. The exact failure mode the dependency-analyzer framing rules out. Rejected.

- **Leave `ResolvedIdentifier` unified; rely on audit cadence.**
  "Audit every few PRs" has the same drift problem as grep — invisible new uses land in the interval. Substrate fact is the only non-heuristic signal. Rejected.

- **Add provenance at the `Port` level instead of `AtomPayload`.**
  Port carries resolved type info, not reference provenance; the fact lives where the resolution happens (the Atom identifier), not where the type lives (the Port). Rejected.

---

## Implementation scope

1. **Substrate change.** `src/v3/compiler/src/dag.rs` — split `AtomPayload::ResolvedIdentifier` into `ResolvedByStructure` and `ResolvedByName`. Delete the old variant.
2. **Lowering.** `src/v3/compiler/src/lower.rs` — every call site that currently produces `ResolvedIdentifier` updates. Structural-walk code paths produce `ResolvedByStructure`; `declaration_by_name` and analogous helpers produce `ResolvedByName`. This is mechanical per-call-site.
3. **Infer.** `src/v3/compiler/src/infer.rs` — match arms on `ResolvedIdentifier` update to handle both variants. A helper `resolved_id(payload) -> Option<DeclarationId>` reduces repetition where provenance is uniformly ignored, but match sites are explicit.
4. **Reflection.** `src/v3/std/substrate.dag` — the reflected `AtomPayload` type gains both variants. Existing lenses that match on `ResolvedIdentifier` update.
5. **Lens.** Write `lens_structural_resolution.dag` as the DB-17 consumer. Initial variants: `UnresolvedArrowBody`, `NameKeyedReference`. Acceptance test fires on any user-range DAG with `ResolvedByName` reachable.
6. **Scheduled-deletions update.** Flip the `declaration_by_name` + NameKeyedReference rows from "Needs DB-17" to the live lens path when the lens lands.

Size: **M** for DB-17 substrate + lowering + consumer migration; **S** additional for the lens itself. Together roughly one lane-stage of work.

---

## Open questions

1. **Bootstrap range vs user range — how is the boundary represented?** The lens needs to exempt name-keyed resolutions inside std/bootstrap files while firing on user-range references. Today `user_start` / declaration-id comparisons serve this purpose in Rust-side code, but the lens needs a substrate-visible fact. Option: a `Declaration.source: DeclarationSource` edge or similar that distinguishes bootstrap/std/user ranges. This might be a sibling DB (DB-18?) or part of DB-17's scope. Current proposal: separate DB if the fact isn't already visible; otherwise cite the existing substrate edge.

2. **Does DB-17 need to handle `ResolvedIdentifier` inside type declarations, or just in Behavior-emitted references?** The intuition says "all resolution sites." The implementation scope for "all sites" is larger. Recommendation: handle all, since half-coverage means the lens has blind spots.

3. **Does the split affect `authored_name_at` or related span-based APIs?** Those resolve names to spans, not to declarations directly. Likely orthogonal — `authored_name_at` is already name-based by design (it's about source position). DB-17's scope is resolved references, not span queries. Confirm at implementation time.

---

## Acceptance

- [ ] `AtomPayload::ResolvedIdentifier` removed from `dag.rs`; `ResolvedByStructure` and `ResolvedByName` added
- [ ] Every lowering call site updated to produce the appropriate variant
- [ ] Every consumer (infer, lenses, emit) updated to handle both variants
- [ ] `src/v3/std/substrate.dag` reflects the new shape
- [ ] `lens_structural_resolution.dag` written with at least `NameKeyedReference` variant; acceptance test fires on a fixture that triggers `declaration_by_name`-style fallback in user-range code
- [ ] Scheduled-deletions table entry for `declaration_by_name` flipped from "Needs DB-17" to the live lens path
- [ ] Existing bootstrap tests green; the split should be semantically invisible to correct consumers
- [ ] A regression test: a fixture that deliberately uses name fallback in user-range code produces a `NameKeyedReference` diagnostic via the lens

---

## Associations

- **Compiler-as-dependency-analyzer thesis** (tonight's framing) — DB-17 is the substrate consequence: every reference must be structural; name-fallback bridges are explicitly marked when they exist.
- **Scheduled-deletions ROADMAP section** (same PR as this DB) — DB-17 unblocks the `declaration_by_name` and NameKeyedReference rows.
- **`lens_structural_resolution.dag`** (separate implementation work; other session) — DB-17 is a prerequisite for the second lens variant (`NameKeyedReference`).
- **M2 module scoping** — dissolves `ResolvedByName` entirely; DB-17's new variant is itself scheduled for deletion.
- **`src/v3/compiler/src/dag.rs::AtomPayload`** — the substrate type being split.
- **`src/v3/compiler/src/lower.rs::declaration_by_name`** — the bridge DB-17 makes structurally visible.
- **`src/v3/std/substrate.dag`** — reflected substrate that lenses walk; gains the two new variants.

---

## Why this is worth a DB

Without DB-17, `declaration_by_name`'s scheduled deletion has **no structural enforcement path**. The scaffold persists because: (a) grep is heuristic and can't tell good fallback from drift, (b) audit cadence misses interval drift, (c) the dependency DAG is incomplete until every reference is structural. DB-17 makes the bridge visible in the substrate so the lens can walk it. Once visible, the scaffold counts down to zero under structural enforcement — not under discipline, not under audit.

This is the compiler-as-dependency-analyzer thesis cashing out: **every named scaffold gets a structural enforcement path, or the substrate amendment that unlocks one is filed.**
