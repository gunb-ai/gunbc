# T-Ground-Engine — Phase 1 (Type-Structure Validation, Sharpened-(b))

**Status**: PROPOSAL — dispatchable when [PR #776](https://github.com/gunb-ai/gunbc/pull/776) (loader-close) merges. This brief supersedes [`t-ground-engine-phase-1.md`](t-ground-engine-phase-1.md) for live worker dispatch; the original brief is preserved as decision-history anchor for structural contracts.

**Lane**: T-Ground-Engine (M) per ROADMAP.md §"Post-R1 Grounding lanes". This brief covers the **rescoped Phase 1** per Director signal on [PR #768 (2026-04-25T03:27:15Z)](https://github.com/gunb-ai/gunbc/pull/768) — type-structure validation only. Phase 2 (full pilot-list enumeration; mirror retirement) deferred to [`t-ground-engine-phase-2-enumeration.md`](t-ground-engine-phase-2-enumeration.md) (forward-planned; blocks on R2 T-Substrate 4th sub-lane).

**Manager**: R2 Grounding Manager ([`grounding-manager.md`](grounding-manager.md)).

**Lineage**:
- Phase 0 audit: [`t-ground-engine-substrate-audit.md`](t-ground-engine-substrate-audit.md) ([PR #768](https://github.com/gunb-ai/gunbc/pull/768), merged commit `4afc0d794`).
- Routing decision: [`t-ground-engine-substrate-escalation.md`](t-ground-engine-substrate-escalation.md) §"Decision" — Director chose Route 1 (small loader-close, ad-hoc Director dispatch) on 2026-04-25.
- In-flight loader-close ([PR #776](https://github.com/gunb-ai/gunbc/pull/776)) ran a probe and discovered a second substrate gap (top-level `ValueBody::Unparsed(SourceSpan)` for the pilot list); Director routed Path 2 (re-scope Engine to type-structure-only Phase 1; defer enumeration to Phase 2).

---

## Orient before working

1. **PR #776 — the loader-close.** Read the public accessor signature in `src/v3/compiler/src/lib.rs` and `src/v3/compiler/src/bootstrap.rs` once #776 merges (or against the branch state if not yet merged). The accessor exposes the `RustPrimitive` type-structure declarations as `Declaration` — you walk these. **The `rust_pilot_primitives` data table itself lowers as `ValueBody::Unparsed(SourceSpan)`** per the loader's Path 2 scoping; do not attempt to walk it symbolically (Phase 2 territory).
2. **[`dsl/extdeps/languages/rust/primitives.dag`](../../dsl/extdeps/languages/rust/primitives.dag).** Source authority. The `RustPrimitive` sum (`IntegerPrimitive | NonIntegerPrimitive`), `IntegerAlgebra | NonIntegerAlgebra`, `TargetCarrier`, `IntegerOverflow` — your walker validates the type structure of these against expected shape. The pilot-set declarations (lines 196+) are out of scope for Phase 1.
3. **[`src/v3/grounding_pilot/src/lib.rs`](../../src/v3/grounding_pilot/src/lib.rs).** The pilot crate stays in place. Its `RUST_PILOT_PRIMITIVES` Rust constants remain authoritative for routing in Phase 1 (mirror persists). **Do not delete or modify the mirror** — that's Phase 2 scope.
4. **[`t-ground-engine-phase-1.md`](t-ground-engine-phase-1.md)** (original, pre-rescope brief). Read for the **structural contracts that carry forward verbatim**: no Rust-constant mirror of `.dag` *for the types being validated*, state-space discipline (variant-aware walker), fail-closed by construction (Ambiguous / NoInhabitant), SG-0 ratchet untouched.
5. **[`grounding-pilot-receipt.md`](grounding-pilot-receipt.md)** — the five pilot lessons. Especially Lesson 2 (mirroring as substrate-ask): Phase 1 closes the *type-structure* half of mirroring; Phase 2 closes the *enumeration* half.
6. **[Director's Phase 1/2 split signal](https://github.com/gunb-ai/gunbc/pull/768)** (PR #768 comment, 2026-04-25T03:27:15Z) — names the consumer convergence on R2 T-Substrate's 4th sub-lane.
7. **[`MODELING.md`](../../MODELING.md)** (especially M9: DFS the concept DAG) and **[`INVARIANTS.md`](../../INVARIANTS.md)**.

---

## Framing question this lane (Phase 1) answers

Can Engine consume the loader-close's public accessor to validate the **type structure** of `RustPrimitive` symbolically — sum partition, variant fields, algebra/carrier shape — without walking the pilot-list data table (which remains `ValueBody::Unparsed`)?

**Why this question is meaningful even with the mirror still in place**: it proves the structural-validation half of the mirroring elimination works against the loader-close accessor. When R2 T-Substrate's 4th sub-lane closes the `ValueBody::List`/aggregate gap, Phase 2 graduates to the enumeration half against the same accessor surface. Phase 1 establishes the consumer pattern; Phase 2 extends scope.

A "yes" greenlights Phase 2 dispatch when the substrate sub-lane lands. A "no" routes back to manager → Director with a re-scoping ask.

---

## Inherited from pilot + original Engine Phase 1 brief (carry-forward contracts)

These remain non-optional. Same as the pre-rescope brief inherited from pilot.

1. **No Rust-constant mirror of the validated types.** The `RustPrimitive` / `IntegerAlgebra` / `NonIntegerAlgebra` / `TargetCarrier` / `IntegerOverflow` *type definitions* must not be mirrored as Rust constants for routing logic to read. Walk the `Declaration` from the loader's accessor. **Exception preserved**: the pilot's `RUST_PILOT_PRIMITIVES` Rust constants for the **pilot-list data** stay in place — Phase 2 retires them, not Phase 1.
2. **State-space discipline preserved.** The walker dispatches on `IntegerPrimitive | NonIntegerPrimitive` variants. Validation logic that distinguishes integer-specific fields (`overflow`) from non-integer fields must respect the partition; flat-record handling is out of scope.
3. **Fail-closed by construction.** If the `Declaration` walker encounters a `RustPrimitive` shape mismatch (missing variant, wrong field, etc.) it returns a structured diagnostic (`StructureMismatch { expected, actual }` or equivalent). No silent fallbacks.
4. **SG-0 ratchet untouched.** Engine lives in a sibling crate (`src/v3/grounding_engine/` or equivalent — worker discretion per pilot's `src/v3/grounding_pilot/` precedent). Not in `src/v3/compiler/`.
5. **Variant-aware walker dispatching.** `IntegerAlgebra` vs `NonIntegerAlgebra` are heterogeneous variant-typed enums; routing/validation must dispatch on variant rather than treating algebra as a flat string.

---

## Phase 1 scope

### Phase 0 — substrate audit (NOT REQUIRED)

The Phase 0 audit was completed in [PR #768](https://github.com/gunb-ai/gunbc/pull/768) and Director routed Route 1 + Path 2. Do not re-run. The substrate state is captured in the orient-list above.

### Phase A — Engine sibling crate scaffold

Create the Engine sibling crate. SG-0 ratchet handling: same as pilot — sibling-crate isolation, workspace-member entry, deletable as a unit. Suggested location: `src/v3/grounding_engine/` (parallels `src/v3/grounding_pilot/`); worker discretion.

The crate depends on:
- The loader's public accessor surface (whatever signature ships in #776; check `src/v3/compiler/src/lib.rs` post-merge).
- The pilot crate's `RUST_PILOT_PRIMITIVES` constants for routing logic that requires actual pilot enumeration (Phase 1 keeps these consumers; Phase 2 retires).

### Phase B — Type-structure walker

Implement a walker that consumes the loader's accessor to extract the `RustPrimitive` type-structure declaration and validates:

- The top-level type is a sum of two variants (`IntegerPrimitive`, `NonIntegerPrimitive`).
- `IntegerPrimitive` has fields `target_name: String`, `algebra: IntegerAlgebra`, `carrier: TargetCarrier`, `is_copy: Bool`, `overflow: IntegerOverflow`.
- `NonIntegerPrimitive` has fields `target_name: String`, `algebra: NonIntegerAlgebra`, `carrier: TargetCarrier`, `is_copy: Bool` (no overflow).
- `IntegerAlgebra = OrderedRingAlgebra | SemiringAlgebra`.
- `NonIntegerAlgebra = BooleanAlgebraAlgebra | TerminalAlgebra`.
- `TargetCarrier` has six variants per `primitives.dag` (Bit, Byte, Word16, Word32, Word64, Terminal).
- `IntegerOverflow` has three variants (TwoComplementWrap, Saturating, Trap).

Each validation produces either an `Ok(())` or a structured `StructureMismatch` diagnostic naming the expected vs actual shape.

### Phase C — Wire structural validation into the routing path

The pilot's existing routing logic in `src/v3/grounding_pilot/src/lib.rs` (functions like `find_inhabitant`, `ground`) walks the **mirrored** `RUST_PILOT_PRIMITIVES`. Phase 1 doesn't replace this. Instead:

- Engine's type-structure validation runs at Engine-crate startup or as part of an explicit "validate" entry point.
- Validation result asserts that the loader's `Declaration` agrees with the pilot's mirror — i.e., the mirror is still consistent with the source-of-truth `.dag` file.
- This **proves the consumer pattern works** (Engine consumes the loader's accessor; routing/enumeration is a separate pre-existing surface).
- When Phase 2 lands, the mirror retires and routing migrates to walk `rust_pilot_primitives` directly via the same accessor surface.

### Phase D — Tests

- **Type-structure parity test**: walker against the loaded `Declaration` matches the expected `RustPrimitive` shape (per Phase B's enumeration). 100% pass.
- **Mirror-consistency test**: walker's extracted type structure matches the Rust-constant mirror's implicit shape. Detects drift between `primitives.dag` and the mirror. (This test would have caught the original mirroring discipline lesson empirically.)
- **State-space-discipline test**: walker correctly distinguishes `IntegerPrimitive` from `NonIntegerPrimitive` and refuses to access `overflow` on the latter (or returns `StructureMismatch` if asked to).
- **Diagnostic-quality test**: `StructureMismatch` carries enough information to pinpoint the divergence (variant name, field name, expected vs actual shape).

Per `feedback_test_timeout_2s.md`, sub-second. Per `TESTING.md`, hermetic, behavior-driven, unit-first.

### Phase E — Pilot-crate deprecation note (deferred to Phase 2)

The original Phase 1 brief required adding a deprecation note to `src/v3/grounding_pilot/src/lib.rs` pointing at Engine. **Defer to Phase 2** — the pilot's `RUST_PILOT_PRIMITIVES` is still a load-bearing routing source in Phase 1 (mirror persists). Adding "this crate is being retired" before retirement is misleading.

---

## Out of scope (do NOT do)

- **Walking `rust_pilot_primitives` (the data declaration) symbolically.** It lowers as `ValueBody::Unparsed(SourceSpan)`. Phase 2 territory; blocks on R2 T-Substrate 4th sub-lane.
- **Retiring the pilot's `RUST_PILOT_PRIMITIVES` mirror.** Phase 2 territory.
- **Replacing the pilot's routing functions** (`find_inhabitant`, `ground`). Phase 1 ADDS structural validation alongside; doesn't replace routing surface.
- **Cross-type coercion paths**, container types, other targets — all later phases / lanes.
- **Touching `src/v3/compiler/`** — SG-0 ratchet.
- **Touching `dsl/std/coercion.dag` or `dsl/extdeps/languages/*/types.dag`** — T-Ground-Dissolve scope.
- **Pilot crate deletion** — T-Ground-Dissolve scope (and only after Phase 2 retires the mirror).
- **Widening `RustPrimitive` back to a flat record** — locked structural decision.
- **Re-running Phase 0 audit** — settled in PR #768.

---

## Hand-off discipline

Escalate to manager (do **not** absorb in lane) if:

- The loader's public accessor signature in #776 differs materially from what the manager-side input section in [`t-ground-engine-substrate-escalation.md`](t-ground-engine-substrate-escalation.md) anticipated.
- The walker can't extract the variant structure from the `Declaration` shape (substrate Gap 1's accessor is insufficient for type-structure walking — would mean the Path 2 scope is still too narrow).
- A second substrate gap surfaces during walker authoring (similar to how the original Phase 1 found the loader gap).
- Mirror-consistency test fails, indicating `primitives.dag` and the Rust mirror have actually drifted (this would be a real bug to surface, not a workaround opportunity).
- Anything would require touching the emit pipeline.

Per `feedback_root_causes_over_quick_fixes.md`: no quick fixes.

---

## Acceptance

PR lands with:

- Engine sibling crate at the chosen location (e.g., `src/v3/grounding_engine/`), workspace-member entry, deletable as a unit.
- Type-structure walker consuming the loader-close public accessor.
- Variant-aware structural validation per Phase B's enumeration.
- Mirror-consistency assertion linking the walker's extracted shape to the pilot's `RUST_PILOT_PRIMITIVES` implicit shape.
- Type-structure-parity, mirror-consistency, state-space-discipline, and diagnostic-quality tests — all sub-second.
- PR body covers: scope (Phase 1 type-structure-only), what stays mirrored (pilot list), what gets validated (RustPrimitive type structure), Phase 2 forward-pointer.
- `cargo test --workspace --exclude v2-compiler-tests`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check` all clean.

---

## What unblocks on merge

- **Manager** updates working-state checklist: Phase 1 ✅; Phase 2 status updated to "blocked on R2 T-Substrate 4th sub-lane only" (one fewer dependency).
- **Phase 2 brief** authored/refined (forward-planned at [`t-ground-engine-phase-2-enumeration.md`](t-ground-engine-phase-2-enumeration.md) — sketch may exist; will be hardened against the actual loader accessor + Phase 1 walker shape).
- **No automatic full-reference unblocking** — DB-11 + cardinality-substrate gates unchanged.

---

## Lineage chain

| Step | Artifact | Status |
|---|---|---|
| Pilot | [PR #765](https://github.com/gunb-ai/gunbc/pull/765) | ✅ merged |
| Pilot receipt | [`grounding-pilot-receipt.md`](grounding-pilot-receipt.md) | ✅ landed in PR #767 |
| Original Phase 1 brief | [`t-ground-engine-phase-1.md`](t-ground-engine-phase-1.md) | ✅ landed; superseded by this brief for live dispatch |
| Phase 0 audit | [`t-ground-engine-substrate-audit.md`](t-ground-engine-substrate-audit.md) | ✅ merged in [PR #768](https://github.com/gunb-ai/gunbc/pull/768) |
| Substrate escalation + Director routing | [`t-ground-engine-substrate-escalation.md`](t-ground-engine-substrate-escalation.md) | ✅ landed in PR #767 |
| Loader-close + Path 2 split | [PR #776](https://github.com/gunb-ai/gunbc/pull/776) | 🔄 in flight |
| **Phase 1 (this brief)** | this file | 📋 dispatchable on #776 merge |
| Phase 2 enumeration | [`t-ground-engine-phase-2-enumeration.md`](t-ground-engine-phase-2-enumeration.md) | 📋 forward-planned; blocks on R2 T-Substrate 4th sub-lane |
| Tests / Dissolve | (briefs not authored) | ⏸️ blocked further downstream |
