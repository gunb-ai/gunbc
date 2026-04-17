> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) | Unblocks: Scheduled deletion of user-range `ResolvedByName` AtomPayload bridges (NOT compiler-internal `declaration_by_name` call sites) | Consumes: compiler-as-dependency-analyzer framing

# Design DB-17 — Reference-resolution provenance (user-range AtomPayload scope)

**Design blocker:** DB-17 (substrate amendment that records HOW a user-range resolved identifier was resolved — structural walk vs name-lookup fallback)
**Consumer:** `lens_structural_resolution.dag` — walks user-range AtomPayload edges, reports `ResolvedByName` occurrences reachable from user-range roots.
**Status:** Proposed. Awaiting review.
**Scope is narrow.** DB-17 addresses ONE class of `declaration_by_name`-shaped debt: user-range references whose lowering fell back to name-keyed resolution when a structural walk failed. Compiler-internal call sites of `declaration_by_name` (bootstrap `substrate_markers` caches, pipeline-authority wiring, emitter algebra lookups) are a SEPARATE class with a different dissolution path — see §"What DB-17 does NOT cover" below.
**Origin:** scheduled-deletions reclassification pass (2026-04-17). Codex review on PR #507 correctly flagged that keying the scheduled deletion to the helper name conflates multiple classes; this revision narrows DB-17's scope to one specific class.

---

## Problem

Today, when lowering resolves an identifier **in user-range code**, the resulting `AtomPayload::ResolvedIdentifier(DeclarationId)` carries the *result* of resolution (the `DeclarationId`) but not the *provenance* — the path by which resolution succeeded. Two very different code paths produce the same substrate shape:

1. **Structural walk** — lowering follows parent-chain / scope-chain edges from the reference site to a declaration reachable through structural parentage. This is the authoritative path: the reference is grounded in the compiler's dependency DAG.
2. **Name fallback** — lowering calls `declaration_by_name(name)` (or an analogous helper that maps a `String` to a `DeclarationId` via a name table). This works today for std/bootstrap lookups where cross-module structural resolution isn't yet wired, but it's a bridge: the reference reaches a declaration by **string identity**, not by structural edge.

Post-resolution, both produce `AtomPayload::ResolvedIdentifier(id)`. A lens walking `d.nodes` cannot distinguish them. Any user-range bridge that fell back through name-keyed resolution becomes invisible in the substrate.

This blocks the scheduled deletion of **user-range name-fallback bridges**: to dissolve them, we need to enumerate them structurally, audit that every legitimate case has a structural-walk alternative, and enforce that no new name-fallback resolutions enter user-range code. Without provenance in the substrate, "enumerate" collapses to grep — the heuristic-over-source failure mode the dependency-analyzer framing rules out.

More broadly: the **dependency graph is incomplete until every user-range reference is grounded by a structural edge**. Name-keyed resolution is a bridge that bypasses the edge. The compiler-as-dependency-analyzer thesis requires that this bridge be either structurally visible or absent.

## What DB-17 does NOT cover

`declaration_by_name` (the Rust helper at `dag.rs:1459`) has ~83 call sites across the compiler. DB-17 covers exactly one class of them — the user-range lowering fallback. The other classes are NOT unblocked by this DB:

1. **Bootstrap substrate-marker caches** (`dag.rs:1616-1654+`). The compiler looks up `"Int"`, `"Bool"`, `"String"`, `"Value"`, `"Transform"`, `"Branch"`, `"Loop"`, `"Bind"`, `"Main"`, `"DeclarationRef"`, `"TypeRealization"`, `"OperatorRealization"`, `"BehaviorRealization"` by name to populate `substrate_markers`. This is a compiler-internal cache, not a resolution fallback at a user-range declaration. Dissolves via a separate substrate amendment (marker field becoming a typed edge rather than a name-keyed cache) — NOT via DB-17.

2. **Pipeline-authority wiring** (`bootstrap.rs:216, 362, 448`; `pipeline_authority.rs:35, 65, 162`). Bootstrap walks the Dag by name for `PIPELINE_REALIZATION_META`, `"parse"`, `"PipelineStageBinding"`, `"PipelineSnapshotKind"` to wire realization edges. This is bootstrap-time machinery, not user-range lowering. Dissolves when self-hosting makes the compiler's own pipeline.dag lens-able as ordinary user code.

3. **Emitter algebra lookups** (`emit_go.rs:2167, 2378`; `emit_python.rs:1233, 1243, 1390, 1522`). Emitters look up algebra declarations (`"OrderedRing"`) or behavior parents by name at emission time. This is emission-layer dispatch, not substrate resolution. Dissolves when Lane 1e consolidates emitters and target-specific dispatch becomes spec-declared.

**DB-17's lens is NOT a joint dissolver for all of these.** The ROADMAP scheduled-deletions table splits them into separate rows with separate dissolution paths. The "DB-17 covers ALL `declaration_by_name` uses" framing (from an earlier revision of this doc) was wrong.

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

`ResolvedByName` is itself a scheduled-deletion row in the ROADMAP (the user-range AtomPayload row). Dissolution trigger: M2 module scoping lands. At that point every user-range reference resolves structurally; the variant becomes unreachable; a final PR removes it from the substrate.

Between DB-17 landing and M2 module scoping landing, `ResolvedByName` is the explicit bridge for user-range fallback — visible, countable, lens-enforced.

**This does not dissolve compiler-internal `declaration_by_name` call sites.** Those dissolve via separate paths (bootstrap substrate_markers via substrate amendment; pipeline/emitter sites via self-hosting). See §"What DB-17 does NOT cover" above.

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
2. **Lowering (user-range paths only).** `src/v3/compiler/src/lower.rs` — call sites that currently produce `ResolvedIdentifier` at user-range resolution points update. Structural-walk paths produce `ResolvedByStructure`; user-range name-fallback paths produce `ResolvedByName`. Compiler-internal `declaration_by_name` call sites (bootstrap substrate_markers, pipeline authority, emitter algebra) do NOT produce AtomPayload — they consume the helper directly for their own purposes and are out of scope for this change.
3. **Infer.** `src/v3/compiler/src/infer.rs` — match arms on `ResolvedIdentifier` update to handle both variants. A helper `resolved_id(payload) -> Option<DeclarationId>` reduces repetition where provenance is uniformly ignored, but match sites are explicit.
4. **Reflection.** `src/v3/std/substrate.dag` — the reflected `AtomPayload` type gains both variants. Existing lenses that match on `ResolvedIdentifier` update.
5. **Lens.** Write `lens_structural_resolution.dag` as the DB-17 consumer. Variants: `UnresolvedArrowBody` (lens-able today against existing substrate), `NameKeyedReference` (DB-17-enabled). Acceptance test: `NameKeyedReference` lens fires on any user-range DAG with `ResolvedByName` AtomPayload reachable from a user-range root.
6. **Scheduled-deletions update.** Flip the user-range `ResolvedByName` AtomPayload row from "Needs DB-17" to the live lens path when the lens lands. The compiler-internal `declaration_by_name` row stays on its separate compiler-source-ratchet path.

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
