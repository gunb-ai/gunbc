# T-Ground-Pilot — Receipt

**Status**: COMPLETE. Pilot lane closed via [PR #765](https://github.com/gunb-ai/gunbc/pull/765) (merged commit `2909f9e05`, 2026-04-25).

**Purpose**: capture the lessons the pilot established empirically, so they carry forward into Engine + full-reference + Dissolve scopes without requiring future workers to re-derive them from the PR thread. Manager-authored synthesis per the [grounding-manager brief](grounding-manager.md) ("Pilot deliverable #4 — manager scope, not worker scope").

**Audience**: T-Ground-Engine workers (any phase), T-Ground-Rust / -Python / -Go workers (when substrate unblocks), T-Ground-Tests workers, R2 Grounding Manager (working-state inputs), Director (program-scope review).

---

## Framing question the pilot answered

*Does inhabitance-search routing — consuming structural target-primitive declarations and selecting by algebra-homomorphism — produce the same target-primitive selection as today's name-keyed table lookup, on a small Rust pilot set?*

**Answer**: Yes, with a finding. Routing parity stratifies into two strata:

- **Stratum A** (3 of 10 pilot types — `Int(=Int64)→i64`, `Bool→bool`, `Unit→()`): exact match against today's `rust_type_checkpoints`.
- **Stratum B** (7 of 10 — `Int8/16/32`, `UInt8`–`UInt64`): engine produces canonical width-correct primitive that the name-keyed table cannot reach. The fallback `OrderedRing→i64` algebra-inhabitant is width-blind (would mis-route `Int8` to `i64`); `Semiring` has no inhabitant declared at all (unsigned types fail-closed today).

The proposal's central architectural claim — *structural `(algebra, carrier)` matching strictly subsumes name-keyed lookup* — was empirically demonstrated. The proposal did not need amendment.

---

## The five lessons

These are the carry-forwards. Each cites its empirical source (the PR commit or comment that produced it) and names where the lesson routes forward.

### Lesson 1 — Stratum-B finding: full-reference scope is larger than implicit baseline

**What we learned.** Today's table-driven coercion surface (`dsl/extdeps/languages/rust/types.dag`'s `rust_type_checkpoints` + `rust_algebra_inhabitants`) covers far less than the implicit baseline assumed. Of the 10 pilot types, only 3 have name-keyed routing entries; 7 either mis-route via width-blind fallbacks or fail-closed entirely.

**Source.** [PR #765 body — "Parity finding (pilot-success signal, not an escalation)"](https://github.com/gunb-ai/gunbc/pull/765); [LGTM comment 4317308089](https://github.com/gunb-ai/gunbc/pull/765#issuecomment-4317308089).

**Forward routing.** T-Ground-Rust / -Python / -Go workers and brief authors must internalize: *the full-reference lanes' true scope is "model the language reference's primitives" + "extend routing onto previously fail-closed surface."* The latter is invisible from the existing tables — it only becomes visible by enumerating the language reference and discovering what's missing. Brief authors for the full-reference lanes should not size scope by counting existing table rows.

**Forward routing (T-Ground-Dissolve).** Dissolution doesn't just swap table-routing for engine-routing on covered surface — it also extends routing onto previously fail-closed surface. The dissolution PR is a strict-superset behavior change, not a like-for-like swap.

### Lesson 2 — Mirroring is forbidden; eliminating it is a substrate ask

**What we learned at pilot time.** The pilot's Rust-constant mirror of `dsl/extdeps/languages/rust/primitives.dag` (and of `dsl/std/integer.dag`'s type-mappings) is a load-bearing concession with hand-sync risk. The lesson, as originally written: *Engine's load-bearing scope is to consume `.dag` declarations directly, eliminating the mirror.*

**What the audit revealed (post-pilot).** The mirror existed precisely because the v3 substrate doesn't load `dsl/extdeps/languages/*` into the bootstrap Dag at all (`bootstrap.rs:16-19` — *"Production bootstrap does not inject target-language realizations"*). The pilot's choice to mirror was forced by the substrate, not by pilot-scope convenience. Eliminating the mirror is a **substrate-capability question**, not an Engine-internal question.

**Source.** Pilot mirroring discipline first articulated in [PR #765 LGTM comment 4317308089](https://github.com/gunb-ai/gunbc/pull/765#issuecomment-4317308089); substrate dependency confirmed in [the Phase 0 audit](t-ground-engine-substrate-audit.md) (PR #768 merged 2026-04-25).

**Forward routing.** Engine cannot honor "no mirroring" without the extdeps loader closing. See `docs/briefs/t-ground-engine-substrate-escalation.md` for the routing decision.

**Forward routing (full-reference lanes).** Same constraint: T-Ground-Python / -Go cannot consume their respective `dsl/extdeps/languages/*/primitives.dag` files without the extdeps loader. Their dispatch shape depends on the same substrate close.

### Lesson 3 — Fail-closed by construction is the contract baseline

**What we learned.** The pilot proactively shipped `GroundingError::{Ambiguous, NoInhabitant}` plus structural tests (`pilot_primitives_have_unique_algebra_carrier_keys`, `missing_inhabitant_fails_closed`). The original pilot brief allowed deferring fail-closed discipline to Engine; the worker chose to lock it at pilot scope per `feedback_fail_closed_discipline.md`.

**Source.** Pilot crate `src/v3/grounding_pilot/src/lib.rs`, fail-closed shape per [PR #765](https://github.com/gunb-ai/gunbc/pull/765).

**Forward routing.** Engine doesn't need to *introduce* fail-closed selection — only generalize it (minimum-satisfier selection, structured diagnostics naming candidates). Pilot's bare-vector `Ambiguous { candidates }` is the floor; Engine produces a real diagnostic per the proposal's L4 surface. **This is a contract, not a suggestion** — Engine that drops fail-closed shape regresses against pilot.

**Forward routing (T-Ground-Tests).** L4 witness-based certification can rely on the contract being load-bearing in the engine, not just decorative. Tests can assert against `Ambiguous` / `NoInhabitant` outcomes as first-class behaviors.

### Lesson 4 — SG-0 ratchet drives Engine location decision

**What we learned.** The pilot's manager-brief candidate location (`src/v3/compiler/src/pilot/grounding_pilot.rs`) would have ratchet-bumped the SG-0 hand-Rust ratchet on `src/v3/compiler/`. Worker restructured to a sibling crate (`src/v3/grounding_pilot/`) — strictly better than the manager-brief default because workspace-member removal is a cleaner "deletable as a unit" than module deletion.

**Source.** [PR #765 PR body — "File-location decisions"](https://github.com/gunb-ai/gunbc/pull/765); manager endorsement in [LGTM comment 4317308089](https://github.com/gunb-ai/gunbc/pull/765#issuecomment-4317308089).

**Forward routing.** If Engine remains hand-Rust temporarily (sharpened (b.i) per the substrate-escalation), it must inherit the same SG-0 constraint — sibling-crate or `.dag`-defined, never under `src/v3/compiler/`. The Engine-Phase-1 brief codified this in its Phase 0 audit options ((c) `src/v3/compiler/` is REJECTED on SG-0 grounds).

**Forward routing (T-Ground-Dissolve).** Dissolve scope already expanded to delete `src/v3/grounding_pilot/`. If Engine ships in a sibling crate, Dissolve scope expands further to delete that crate too. Both deletions are workspace-member removals (clean).

### Lesson 5 — State-space discipline: precedent shape for full-reference

**What we learned.** The original `RustPrimitive` declaration carried `overflow: IntegerOverflow?` with a doc-comment convention ("none for non-integer primitives"). Codex P2 review caught the gap: the type admitted illegal state combinations (`bool: Some(TwoComplementWrap)`, `i64: None`). Manager called for a structural fix per `feedback_state_space_vs_behavioral_invariants` and `feedback_no_validation_passes`. Worker partitioned `RustPrimitive` into `IntegerPrimitive | NonIntegerPrimitive` — overflow lives only on the integer variant; structurally absent from non-integer.

**Source.** Codex P2 finding + manager adjudication in [comment 4317414841](https://github.com/gunb-ai/gunbc/pull/765#issuecomment-4317414841); structural-fix commit on the pilot branch.

**Forward routing.** `RustPrimitive`'s sum-type partition is the **precedent shape** that `PythonPrimitive` / `GoPrimitive` inherit. T-Ground-Python / -Go briefs must require state-space discipline at the type-shape level (not via doc-comment convention or `Option`-with-validation). Specifically:

- Identify per-target the analog axes where overflow / Copy-trait / mutability / GIL-affinity / etc. only apply to a subset of primitives.
- Partition the per-target `Primitive` type structurally so subset-only fields live only on the variants where they're meaningful.

**Forward routing (Engine).** Variant-aware walker handling is a contract: `find_inhabitant` and equivalents must dispatch on `IntegerPrimitive | NonIntegerPrimitive` to extract the variant-typed `algebra` field. The Engine-Phase-1 brief codified this via the state-space-discipline test requirement.

**Forward routing (T-Ground-Dissolve).** Widening `RustPrimitive` back to a flat record is locked out of scope per the codex P2 adjudication. Dissolve cannot simplify the partition away.

---

## What this receipt does NOT contain

- **Worker-level implementation details.** The pilot's engine code, test names, and per-line decisions live in [PR #765](https://github.com/gunb-ai/gunbc/pull/765) and `src/v3/grounding_pilot/src/lib.rs`. This receipt is lessons, not history.
- **Brief amendments.** Lessons that should reshape future briefs are flagged; the actual amendments happen in the relevant brief PRs.
- **Director-routed substrate decisions.** The mirroring lesson's substrate ask is documented in [`t-ground-engine-substrate-escalation.md`](t-ground-engine-substrate-escalation.md), not here.

---

## Cross-program implications

- **Pure Bootstrap to Zero program.** The extdeps-loader gap (Lesson 2 forward routing) overlaps directly with PB-1 / PB-Bootstrap-Process scope (`bootstrap.rs` data-driven loader). See substrate-escalation doc for concrete coordination ask.
- **R1 Substrate Manager (archived on R1 close).** The pilot's lessons about state-space discipline, fail-closed contract, and structural over conventional are general-purpose and should inform DB-11 + cardinality-substrate design decisions if those sub-lanes haven't yet finalized their type shapes. The lessons are framework-level, not Grounding-specific.
- **R2 Grounding Manager working state.** Lane checklist updated in `grounding-manager.md`. Pilot lane ✅. Engine-Phase-1 audit complete + parked pending loader-close PR (Director routed Route 1; substrate ask is in flight as ad-hoc Director dispatch).

---

## Lineage

- Parent program: ROADMAP.md §"Post-R1 Program — Grounding Completeness" (current); §"Release R2 Program" → T-Ground (post-promotion).
- Parent thesis claim: THESIS.md Tier 1 → "Grounding completeness."
- Parent architectural design: [`docs/single-emitter-design.md`](../single-emitter-design.md).
- Parent worked-examples doc: [`docs/thesis/target-grounding-proposal.md`](../thesis/target-grounding-proposal.md).
- Sibling artifacts: [`grounding-manager.md`](grounding-manager.md), [`t-ground-engine-phase-1.md`](t-ground-engine-phase-1.md), [`t-ground-engine-substrate-audit.md`](t-ground-engine-substrate-audit.md), [`t-ground-engine-substrate-escalation.md`](t-ground-engine-substrate-escalation.md).
