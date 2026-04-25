# T-Ground-Engine — Phase 1 (Pilot-Scope Production Walker)

**Status**: PROPOSAL — same formal gates as the parent program (R1 all-gates-green closure + R2 promotion of [`docs/r2-structure.md`](../r2-structure.md)). Pilot merge ([PR #765](https://github.com/gunb-ai/gunbc/pull/765), commit `2909f9e05`) is a necessary-not-sufficient signal. Director-discretionary dispatch is honored under the same posture the pilot dispatched under.

**Lane**: T-Ground-Engine (M) per ROADMAP.md §"Post-R1 Grounding lanes". This brief covers **Phase 1 only** — the pilot-scope subset. Engine grows in subsequent phases as full-reference lanes land (cross-type coercion paths, container types, Python/Go targets).

**Manager**: R2 Grounding Manager ([`docs/briefs/grounding-manager.md`](grounding-manager.md)).

**Parent context**: pilot brief inheritance lives in [`PR #765`](https://github.com/gunb-ai/gunbc/pull/765) (merged commit `2909f9e05`). Read its merged state, not the original brief — the structural fix from the codex P2 adjudication landed and changes the inherited shape.

---

## Orient before working

1. **PR #765 — the pilot.** Read [`src/v3/grounding_pilot/src/lib.rs`](../../src/v3/grounding_pilot/src/lib.rs) end-to-end. **You are replacing this crate with the production walker.** The pilot's contract is your inheritance.
2. **[`dsl/extdeps/languages/rust/primitives.dag`](../../dsl/extdeps/languages/rust/primitives.dag).** The structural authority. Note: `RustPrimitive` is a **sum type** (`IntegerPrimitive | NonIntegerPrimitive`), partitioned so overflow-bearing state-space is structurally separated from non-overflow. State-space discipline is load-bearing — see "Inherited from pilot" below.
3. **[`dsl/std/integer.dag`](../../dsl/std/integer.dag), [`dsl/std/algebra.dag`](../../dsl/std/algebra.dag), [`dsl/std/bit.dag`](../../dsl/std/bit.dag).** The .dag-side authority for unfolding user types (`Int8`..`UInt64`, `Bool`, `Unit`) into `(algebra, carrier)` facts. The pilot's `dag_type_facts` function mirrors these — your job is to consume them directly.
4. **[`docs/thesis/target-grounding-proposal.md`](../thesis/target-grounding-proposal.md).** Minimum-satisfier discipline, fail-closed tie-breaking with structured diagnostics, the three-way L4 split.
5. **[`docs/single-emitter-design.md`](../single-emitter-design.md).** The architectural design Engine operationalizes: "the mapping should fall out from the algebra, not from a hand-maintained table."
6. **[`MODELING.md`](../../MODELING.md)** (especially M9: DFS the concept DAG) and **[`INVARIANTS.md`](../../INVARIANTS.md).** Same governing rules as pilot.
7. **Pilot receipt.** Manager-authored synthesis of the four headline lessons Engine inherits. **Primary source**: `docs/briefs/grounding-pilot-receipt.md` once authored. **Fallback if not yet on main at the time you read this**: the same lessons are captured in [PR #765's merged body](https://github.com/gunb-ai/gunbc/pull/765) and the LGTM comment thread ([`comment 4317308089`](https://github.com/gunb-ai/gunbc/pull/765#issuecomment-4317308089) for the four-lesson enumeration; [`comment 4317414841`](https://github.com/gunb-ai/gunbc/pull/765#issuecomment-4317414841) for the state-space-discipline lesson from the codex P2 adjudication). Read both — receipt is synthesis, the PR thread is the empirical record.

---

## Framing question this lane (Phase 1) answers

Can the production walker consume `.dag` declarations directly via the v3 substrate, retain the pilot's parity guarantees (zero mismatches on Stratum A + canonical extension on Stratum B across the 10 pilot types), and inherit the pilot's fail-closed contract (`Ambiguous` + `NoInhabitant`) — **without** mirroring facts as Rust constants?

A "yes" greenlights the pilot crate's deletion (T-Ground-Dissolve scope expansion already committed). A "no" routes the substrate gap to manager → Director.

---

## Inherited from pilot (carry-forwards Engine must honor)

These are not optional. The pilot established them empirically; Engine inherits them as design contracts.

1. **Mirroring is forbidden.** The pilot's `dag_type_facts` and `RUST_PILOT_PRIMITIVES` mirror `.dag` facts as Rust constants. This was a pilot-scoped concession with explicit hand-sync risk flagged in three places. **Engine must consume the `.dag` declarations directly.** Eliminating mirroring is the load-bearing scope of this lane — without it, Engine is just "pilot at scale."
2. **State-space discipline.** `RustPrimitive`'s sum-type partition (`IntegerPrimitive | NonIntegerPrimitive`) makes illegal combinations (`bool: Some(TwoComplementWrap)`, `i64: None`) unrepresentable. Engine must preserve this — variant-aware walker handling, no widening to a flat record. Per the codex P2 adjudication on PR #765 ([`comment 4317414841`](https://github.com/gunb-ai/gunbc/pull/765#issuecomment-4317414841)).
3. **Fail-closed by construction.** `GroundingError::{Ambiguous, NoInhabitant}` plus `pilot_primitives_have_unique_algebra_carrier_keys` and `missing_inhabitant_fails_closed` tests. Pilot deferred minimum-satisfier discipline to Engine; Engine ships it. Bare-vector candidates list is the floor; Engine produces a real diagnostic per the proposal's L4 surface.
4. **SG-0 ratchet untouched.** Pilot lived as a sibling crate (`src/v3/grounding_pilot/`) to avoid bumping the SG-0 hand-Rust ratchet on `src/v3/compiler/`. Engine inherits the same constraint — see the Phase 0 audit below for where Engine lives.

---

## Phase 1 scope

**Same routing surface as pilot — 10 Rust primitive types.** Engine-Phase-1 is not a scope expansion; it's a substrate-integration upgrade.

### Phase 0 — substrate audit + design decision

Before implementation, audit where Engine lives. Three options, ordered by project-thesis alignment:

- **(a) `.dag`-defined walker** (e.g., `dsl/grounding/engine.dag`). Cleanest, no SG-0 ratchet issue, fully aligned with the project's "compiler operations are `.dag` programs" thesis. **Requires audit**: can the v3 substrate today model:
  - Loading `rust_pilot_primitives: List<RustPrimitive>` as a value to walk?
  - Pattern-matching `RustPrimitive`'s sum variants to extract `algebra` and `carrier` fields (the variant-tagged algebra means matching has to dispatch on variant)?
  - Filtering by structural equality on `(algebra, carrier)` — note algebra is variant-typed (`IntegerAlgebra` vs `NonIntegerAlgebra`), so equality is heterogeneous?
  - Returning a `Result<RustPrimitive, GroundingError>` (or the structural equivalent — sum type carrying the structured diagnostic)?

  If yes, this is the destination.

- **(b) Sibling crate** (e.g., `src/v3/grounding_engine/`). Extends the pilot's pattern; fastest path; defers the `.dag`-walker question. Acceptable **only if** (a)'s substrate audit fails — flag the gap that blocks (a) so manager can coordinate substrate work with Director.

- **(c) `src/v3/compiler/`.** **REJECTED** — SG-0 hand-Rust ratchet. Do not propose.

The PR description must lead with the audit result and the resulting choice. **Do not skip the audit** — defaulting to (b) without auditing (a) is the missing-modeling pattern called out in `feedback_model_before_fixing.md`.

Audit deliverable: write up findings in `docs/briefs/t-ground-engine-substrate-audit.md` (≤ 300 lines). Conclude with (a) or (b), with substrate-gap flags if (b). The `docs/briefs/` location matches existing convention for manager/worker work-product docs (audits, receipts) — see siblings like `ci-ratchet-architecture-audit.md` and `complexity-v2-v3-comparison-receipt.md`.

If (b), the substrate gaps surfaced are escalation-class — flag to manager, do not invent workarounds.

### Phase 1 — production walker

Implement the walker per the audit's choice. Required behavior:

- **Direct `.dag` consumption.** No Rust-constant mirror of `rust_pilot_primitives`, no Rust-constant mirror of std-side type-mappings. The pilot's `RustPrimitive` and `dag_type_facts` mirroring must not survive into Engine.
- **Variant-aware walker.** `RustPrimitive`'s `IntegerPrimitive | NonIntegerPrimitive` partition means routing must dispatch on variant to extract the algebra field. The variant tag is not itself a routing key — search across both variants and match on `(algebra, carrier)` agreement, where algebra equality is heterogeneous across the variant-typed enums (`IntegerAlgebra` vs `NonIntegerAlgebra`).
- **Inherit fail-closed contract.** `Ambiguous` and `NoInhabitant` with structured diagnostic naming candidates (when ambiguous). Pilot's bare-vector candidates list is the floor; Engine produces a real diagnostic per the proposal's L4 surface.
- **Minimum-satisfier selection.** Pilot deferred this; Engine ships it. For pilot scope every key has exactly one satisfier so this is never exercised — but the discipline must be in place for full-reference lanes (containers, cross-type coercion, etc.).

### Phase 2 — parity tests

Re-run the pilot's test suite against the production walker. All 10 routing-parity assertions must pass (Stratum A: `Int64→i64`, `Bool→bool`, `Unit→()`; Stratum B: signed widths to `i8`/`i16`/`i32`, unsigned widths to `u8`..`u64`). Plus:

- **Mirroring-elimination assertion.** Structural test that verifies the walker reads from the `.dag` source — e.g., changing `primitives.dag` changes walker behavior without code edits. This is the load-bearing test for the mirroring-lesson carry-forward.
- **State-space-discipline assertion.** Variant-aware walker correctness: structural test that asserts `IntegerPrimitive`-only fields (`overflow`) are not accessed when routing through a `NonIntegerPrimitive` (or the structural equivalent in the chosen substrate).
- **Diagnostic-quality test.** The `Ambiguous` path produces a structured diagnostic naming all candidates, not a bare vector.

### Phase 3 — pilot-crate retirement signal

This PR **does not** delete `src/v3/grounding_pilot/` — that's T-Ground-Dissolve scope per the dissolution-expansion line in the pilot's PR body. But it does add a one-line deprecation note to the pilot crate's `lib.rs` header pointing at Engine, so future readers know the pilot's role transitioned to "historical reference."

---

## Out of scope (do NOT do)

- **Cross-type coercion paths** (UTF-8 String↔FreeMonoid<Char>, etc.) — these need full-reference scope; later Engine phase.
- **Container types** — block on cardinality-substrate.
- **Other targets** (Python, Go, dag) — separate full-reference lanes, blocked on substrate.
- **Emit-pipeline wiring.** Escalate to manager. Per program brief: emit-pipeline boundary closes in R1 T-Emit; any post-close amendment routes via Director. **Do not absorb emit-pipeline work into Engine-Phase-1.**
- **Removing the existing table-lookup call sites in emit** — T-Ground-Dissolve scope.
- **Touching `dsl/std/coercion.dag`** — T-Ground-Dissolve scope.
- **Deleting `src/v3/grounding_pilot/`** — T-Ground-Dissolve scope. This PR adds a deprecation note only.
- **Widening `RustPrimitive` back to a flat record** — locked structural decision per codex P2 adjudication on PR #765.

---

## Hand-off discipline

Escalate to manager (do **not** absorb in lane) if:

- **Phase 0 audit concludes (b) is required** because the v3 substrate can't model the walker in `.dag`. This is a substrate-capability question manager routes to Director — substrate work may need to pre-empt or run in parallel with Engine implementation.
- **Phase 1 reveals a `.dag`-substrate gap not visible in the audit** (e.g., loading data-tables-as-values blows up at integration; pattern-matching variants of `RustPrimitive` works in the audit but not when integrated with the rest of the substrate).
- **Phase 2 reveals a parity divergence from pilot** — same escalation as pilot's brief: routing-parity failure routes to Director for proposal amendment.
- **Anything would require touching the emit pipeline.**
- **DSL-side type unfolding requires a new concept in `dsl/std/`** — escalate; concept additions to std should be reviewed at the std-authority level, not absorbed in Engine scope.

Per `feedback_root_causes_over_quick_fixes.md`: no quick fixes. If Engine is blocked, escalate; do not patch.

---

## Acceptance

PR lands with:

- `docs/briefs/t-ground-engine-substrate-audit.md` documenting the (a) vs (b) decision and any substrate-gap flags.
- Production walker in the chosen location, consuming `.dag` directly, no Rust-constant mirror.
- Variant-aware routing across `IntegerPrimitive | NonIntegerPrimitive`.
- 10/10 routing-parity tests passing against the production walker.
- Mirroring-elimination test (changes to `primitives.dag` change walker behavior without code edits).
- State-space-discipline test (variant-aware walker correctness).
- Diagnostic-quality test (`Ambiguous` produces structured diagnostic, not bare vector).
- One-line deprecation note in `src/v3/grounding_pilot/src/lib.rs` pointing at Engine.
- PR body covers: audit conclusion, design choice, substrate-gap flags (if any), parity confirmation, mirroring-elimination demonstration.
- `cargo test --workspace --exclude v2-compiler-tests`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all --check` all clean.
- Per `feedback_test_timeout_2s.md`, all tests sub-second.
- Per `TESTING.md`, hermetic, behavior-driven, unit-first.

---

## What unblocks on merge

- **Manager** updates working-state checklist; T-Ground-Dissolve checklist gains a confirmed entry to delete `src/v3/grounding_pilot/`.
- **T-Ground-Tests** dispatch becomes possible once Engine + at least one full-reference lane reaches parity. Engine alone is necessary-not-sufficient for Tests dispatch (Tests' L4 witness-based certification needs full-reference scope to assert against).
- **No automatic full-reference unblocking** — T-Ground-Rust / -Python / -Go still block on DB-11 closure (refinement substrate) and cardinality-substrate. Engine-Phase-1 merge does not change those gates.

---

## Notes (informational, no action required)

- Pilot's `selection_is_by_algebra_homomorphism_not_name` test is the assertion that future receipt readers should look at to understand what pilot proved. Engine should keep an equivalent test alive — the architectural claim under validation is unchanged.
- Pilot's `pilot_primitives_have_unique_algebra_carrier_keys` test is the structural pre-condition for single-satisfier match. At pilot scope it's always true; full-reference scope may introduce intentional ambiguity (e.g., `i64` and `isize` both inhabit OrderedRing<Word64> on 64-bit platforms). When that lands, minimum-satisfier discipline is what disambiguates. Engine ships the discipline now even though pilot scope doesn't exercise it.
- Variant naming in `RustPrimitive` (`IntegerPrimitive`/`NonIntegerPrimitive`) was discussed but not blocked at pilot time. If full-reference scope (Float, Decimal) makes `IntegerPrimitive` feel mis-named, renaming is worker discretion at that future scope, not Engine-Phase-1 scope.
