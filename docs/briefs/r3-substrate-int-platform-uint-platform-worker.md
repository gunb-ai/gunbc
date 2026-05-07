---
status: pre-authored per pre-authored-brief-queue discipline; dispatchable post-PR-#1914 close (valiant-ibex-312 freed-pool)
authority parent: R3 Substrate Manager (#1739)
ratification: Director Q1 + Q2 + Q3 + Q4 RATIFIED at gunbc#1739 #issuecomment-4393248961 (zesty-bear-812, 2026-05-07). Naming: IntPlatform/UIntPlatform per `feedback_reason_not_label`. Algebra: `Compose<Int, MachineWidth<PointerWidth>>` + `Compose<UInt, MachineWidth<PointerWidth>>` with Platform as substrate token (target-dependent at grounding). Substrate-concept layer (NOT target-only). Worker pin valiant-ibex-312.
roadmap row: §1.8 ledger row TBD slot — gate `int_platform_uint_platform_substrate_landed` (or canonical equivalent at PR-authoring time)
authority docs:
  - gunbc#1739 #issuecomment-4393248961 (Director Q1+Q2+Q3+Q4 RATIFICATION)
  - gunbc#1761 #issuecomment-4393222862 (valiant-ibex-312 STOP surfacing the substrate gap from Phase B)
  - gunbc#828 #issuecomment-4385530115 (Q-MachineConstraint sub-decision 3 — Compose<Algebra, MachineWidth<N>> shape; this brief extends N with substrate-token Platform)
  - dsl/std/integer.dag (consumer site for IntPlatform/UIntPlatform declarations)
  - src/v3/std/machine_constraints.dag (existing MachineWidth<bits> carrier; this brief extends to MachineWidth<PointerWidth>)
  - PR #1914 (T-Interval-Representation; closes at partial coverage; this brief is the follow-on enabling isize/usize row population)
gates:
  - `int_platform_uint_platform_substrate_landed` (proposed §1.8 row)
worker pin: valiant-ibex-312 (freed-pool post-PR-#1914 close; freshest substrate context from BigInt widening + Phase B coverage)
---

# R3 Substrate — `IntPlatform` / `UIntPlatform` substrate-fact-introduction worker brief

## Context

Per Director RATIFICATION at gunbc#1739 #issuecomment-4393248961 (Q1+Q2+Q3+Q4):

PR #1914 (T-Interval-Representation; bundled Rust-primitive-full-coverage) closes at u128 + Phase A coverage; isize/usize rows in `src/v3/spec/rust.dag` deferred to this follow-on. Substrate gap surfaced by valiant-ibex-312 STOP at gunbc#1761 #issuecomment-4393222862:

`TargetIntegerTypeInhabitance.kernel_integer: DeclarationRef` requires a std integer type. Existing rows reference `UInt32` / `Int32` / `Int` / `Int128` / `UInt128`. **`dsl/std/integer.dag` has no platform-sized integer carrier** (no IntPlatform / UIntPlatform / equivalent).

This brief lands the substrate concepts.

### Director ratifications absorbed

- **Q1 — Naming: `IntPlatform` / `UIntPlatform`** per `feedback_reason_not_label` ("encode the stable reason"). Platform-dependence is the structural distinction; size alone doesn't distinguish (Int<32> also has size).
- **Q2 — Algebra: same Int / UInt; platform-axis on MachineConstraint**:
  - `IntPlatform` = `Compose<Int, MachineWidth<PointerWidth>>` where `Int = AbelianGroup<GroupCompletion<Nat>>` (per Slice 3 #1466)
  - `UIntPlatform` = `Compose<UInt, MachineWidth<PointerWidth>>` where `UInt = CommutativeSemiring<Nat>` (per #1818)
  - `PointerWidth` is itself a substrate token (NOT a fixed numeric width); targets ground at emit time
- **Q3 — Substrate-concept layer (NOT target-only)** per `feedback_target_agnostic_ir`. Multiple targets have platform-sized integers; modeling them at substrate-concept layer keeps substrate target-agnostic
- **Q4 — Worker pin valiant-ibex-312** post-PR-#1914 close

## Scope

### Deliverable 1 — `PointerWidth` substrate token

Author `PointerWidth` substrate token in `src/v3/std/machine_constraints.dag` (or canonical adjacent location — worker greps for existing convention). Shape: substrate token consumed by `MachineWidth<PointerWidth>` similarly to existing `MachineWidth<bits>` form for fixed widths.

Two structural shapes worker DFS-decides at dispatch:

- **Option α (sum type)**: `type Platform = Pointer | TargetSpecific(String)` or similar enumeration of platform-axis classes
- **Option β (opaque token)**: `type Platform` (uninhabited or single-witness) — used purely as type-level marker; targets project to specific width at emit time
- **Option γ (typed parameter)**: `MachineWidth<PointerWidth>` parameterized over a Platform type-parameter that's bound by target spec at grounding

**Mgr recommendation**: γ (typed parameter) if the existing `MachineWidth<N>` shape parameterizes over numeric N — adds Platform as a sibling-shape kind alongside numeric. β if simpler / matches existing token discipline. α only if explicit platform-axis enumeration carries structural meaning.

Worker DFS catalogs current `MachineWidth<...>` parameterization shape at HEAD; chooses α/β/γ per existing convention.

### Deliverable 2 — `IntPlatform` / `UIntPlatform` declarations

Author in `dsl/std/integer.dag`:

```dag
type IntPlatform  = Compose<Int,  MachineWidth<PointerWidth>>
type UIntPlatform = Compose<UInt, MachineWidth<PointerWidth>>
```

Practice 4 classification: N/A — these are type aliases over existing `Compose<...>` carrier, not new sum types or coproducts. Worker confirms no new variant introduction.

### Deliverable 3 — `src/v3/spec/rust.dag` isize/usize rows + TargetIntegerTypeInhabitance population

Add rows:
- `rust_integer_inhabit_isize_platform`: `TargetIntegerTypeInhabitance` referencing `IntPlatform` via `kernel_integer` DeclarationRef + `bound: PlatformDependentFact`
- `rust_integer_inhabit_usize_platform`: parallel for `UIntPlatform`

Existing `TypeRealization` rows for `rust_isize` / `rust_usize` co-author per `spec/rust.dag` row population convention.

### Deliverable 4 — Cross-target signal (informational, not in-scope authoring)

Targets without native platform-sized integers (Python's arbitrary-precision `int`, etc.) handle ground-time via target-conditioned **lowering** — NOT target-conditioned substrate. Per Director Q3 ratification: substrate stays clean; Grounding-lane handles target-side projection.

This brief produces the substrate concepts; cross-target consumer wiring (Python / Go / etc.) is separate Grounding-tier work.

### Deliverable 5 — §1.8 ledger row receipt

Add `int_platform_uint_platform_substrate_landed` to §1.8 ledger; advance DECLARED → CONSUMER_LANDED on merge (consumer = Rust-target row population in same PR via Deliverable 3). Cluster: T-Numeric-Construction-adjacent (or new T-Platform-Integer cluster — Mgr discretion at PR-authoring time).

Co-receipt: PR #1914's `rust_primitive_full_coverage` gate advances PARTIAL → CONSUMER_LANDED on this PR's merge (full Rust integer primitive coverage closed via this slice's isize/usize rows).

## Slice — single PR

Phase ordering (PR-internal):
1. DFS-catalog `MachineWidth<...>` parameterization at HEAD; choose α/β/γ for `PointerWidth` token
2. Author `PointerWidth` substrate token (Deliverable 1)
3. Author `IntPlatform` / `UIntPlatform` declarations (Deliverable 2)
4. Author `src/v3/spec/rust.dag` isize/usize TargetIntegerTypeInhabitance rows + TypeRealization rows (Deliverable 3)
5. Bootstrap snapshot regen + parse corpus manifest refresh; integer-row ratchet update (10 → 12 in `int_literal_ranges.rs`)
6. §1.8 ledger row receipt (Deliverable 5) + co-receipt for `rust_primitive_full_coverage` gate advancement
7. Cross-program handoff receipt to Grounding Mgr (#1745) for G2 Phase 2 isize/usize-coverage dispatch unblock

## Acceptance

- `PointerWidth` substrate token landed in `src/v3/std/machine_constraints.dag` (or canonical equivalent) per α/β/γ shape worker chooses
- `IntPlatform` / `UIntPlatform` declarations landed in `dsl/std/integer.dag` consuming `Compose<Int|UInt, MachineWidth<PointerWidth>>` shape per Director Q2 ratification
- `src/v3/spec/rust.dag` has isize/usize TargetIntegerTypeInhabitance rows + TypeRealization rows; bound: PlatformDependentFact (existing variant per PR #1914 carrier-shape verification)
- Integer-row ratchet at `int_literal_ranges.rs` updated 10 → 12 rows (post-PR-#1914 baseline)
- Bootstrap snapshot + parse corpus manifest hold (semantic equivalence on existing 10 rows; 2 new rows added)
- §1.8 row `int_platform_uint_platform_substrate_landed` advances DECLARED → CONSUMER_LANDED upon merge
- Co-receipt: PR #1914's `rust_primitive_full_coverage` gate advances PARTIAL → CONSUMER_LANDED on this PR's merge
- Cross-program handoff receipt to Grounding Mgr (#1745) — G2 Phase 2 isize/usize coverage unblocked
- `cargo test --workspace --exclude v2-compiler-tests` green (3 pre-existing v2-compiler --lib failures verified unrelated)
- `cargo test -p v2-compiler-tests` green; strict-compile diagnostic ratchet at 0
- `cargo clippy --all-targets -- -D warnings` clean
- `cargo fmt --all --check` clean
- Citation discipline per `docs/briefs/brief-authoring-checklist.md`
- 5-question authority audit in PR body
- P1 substrate-fact-introduction receipt:
  - DFS-of-concept-DAG (no parallel platform-integer carrier already exists at HEAD)
  - Named consumer demand (Rust-target isize/usize rows; Grounding G2 Phase 2 partial-coverage cascade)
  - Carrier-shape rationale (α/β/γ decision)

## STOP-AND-ESCALATE

- **`MachineWidth<...>` parameterization shape doesn't accept `PointerWidth` token** (e.g., parser blocks non-numeric type arguments in second slot): STOP — surface to Substrate Mgr; parser/lowerer extension may be needed OR β opaque-token form needed instead of γ typed-parameter
- **`Compose<...>` interaction syntax encoding doesn't accept the proposed `IntPlatform` / `UIntPlatform` shape**: STOP — surface; parallel concern to S3 Phase-2 parser-grammar work; coordinate with valiant-ant-72 if relevant
- **Bootstrap snapshot drift** during Platform/IntPlatform/UIntPlatform/spec-rust-row authoring: root-cause; do NOT bridge with placeholder
- **Cross-target Grounding-lane consumers surface unforeseen blockers** (e.g., Python target requires substrate-side decision rather than Grounding-side projection): STOP — surface; Director Q3 ratified substrate-clean; if breaks, re-ratify
- **Bundled-scope drift**: do NOT bundle Grounding G2 Phase 2 lowering rules / emit verification into this PR. Per Director bundled-scope ratification at gunbc#1739 #issuecomment-4392225548 — parallel infrastructure DISALLOWED. This brief produces substrate; Grounding consumes follow-on

## Authority audit receipt

1. **Substrate exists?** At brief-author time:
   - `Compose<Algebra, MachineConstraint>` carrier landed (PR #1856) ✓
   - `MachineWidth<bits>` carrier landed (PR #1856) ✓ — for fixed numeric widths
   - `Int` / `UInt` algebraic-concept carriers landed (#1466 / #1818) ✓
   - `TargetIntegerInhabitanceBound::PlatformDependentFact` variant exists on origin/main HEAD (per PR #1914 Phase B ratification-state-grep) ✓
   - `PointerWidth` substrate token does NOT yet exist — this brief is producer
   - `IntPlatform` / `UIntPlatform` carriers do NOT yet exist — this brief is producer
2. **Existing brief?** No prior brief on this axis. PR #1914 (T-Interval-Representation; partial coverage) is the upstream brief that surfaced the gap; this brief is the named follow-on
3. **Design-doc match?** Director Q1+Q2+Q3+Q4 RATIFIED at gunbc#1739 #issuecomment-4393248961 + Q-MC sub-decision 3 (gunbc#828 #issuecomment-4385530115) name the shape verbatim
4. **Citations live?** Worker re-verifies at dispatch (post-PR-#1914 close)
5. **Carrier dissolves the bridge?** Yes — `PointerWidth` substrate token + `IntPlatform`/`UIntPlatform` declarations together dissolve the "isize/usize have no representable substrate concept" bridge. Substrate stays target-agnostic per `feedback_target_agnostic_ir`; targets ground at emit time

## Provenance

Drafted 2026-05-07 by Substrate Mgr (quick-crab-830) per Director Q1+Q2+Q3+Q4 RATIFICATION at gunbc#1739 #issuecomment-4393248961. Pre-authored per pre-authored-brief-queue discipline; dispatch fires post-PR-#1914 close (valiant-ibex-312 freed-pool from T-Interval-Representation; freshest context for follow-on per Director Q4).

Cross-references:
- PR #1914 (T-Interval-Representation partial-coverage; this brief is named follow-on)
- Q-MC sub-decision 3 (`Compose<Algebra, MachineWidth<N>>` shape; this brief extends N with substrate-token Platform)
- Grounding G2 Phase 2 (consumer; isize/usize coverage unblocks on this brief's PR merge)
- S3 Phase-2 parser-grammar (potential coordination point if `MachineWidth<PointerWidth>` non-numeric parameterization surfaces parser concerns)
