---
status: draft (worker brief; dispatchable now post-#1856 merge — S3 `Compose<Algebra, MachineConstraint>` carrier landed)
authority parent: R3 Substrate Manager (#1739)
ratification: dispatchable per Q-MachineConstraint sub-decisions RATIFIED at gunbc#828 #issuecomment-4385530115; cross-program with Grounding Mgr (#1745)
roadmap row: T-Numeric-Construction (S9 Phase-1 step 3) + §1.8 ledger rows #17-#24
authority docs:
  - docs/briefs/r3-substrate-s9-t-numeric-construction-worker.md (parent S9 brief; Phase-1 step 3 scope)
  - PR #1856 (S3 Q-MachineConstraint substrate — `dsl/std/machine_constraints.dag` MERGED)
  - PR #1818 (S9 Phase-1 step 2 — `type UInt = Nat` MERGED)
  - PR #1466 / Q6 audit (Slice 3 — `type Int = AbelianGroup<GroupCompletion<Nat>>` MERGED earlier)
  - gunbc#828 #issuecomment-4385530115 (Q-MachineConstraint sub-decisions ratification)
gates:
  - §1.8 ledger row #17 (int_construction_landed)
  - §1.8 ledger row #18 (uint_construction_landed)
  - §67 (numeric_construction_demonstration — Acceptance bullet)
worker pin: proud-lynx-311 (#1746) — S9 worker pin held; brief queues post-Slice-2.5-completion
---

# R3 Substrate S9 Phase-1 Step 3 — Concrete emission entries via parametric `Compose<Algebra, MachineConstraint>`

## Context

### T-V2-Retirement coordination gate (precondition framing)

T-Numeric-Construction historically carried an internal cascade gate
on T-V2-Retirement landing (per S9 parent brief STOP-AND-ESCALATE
bullet at `r3-substrate-s9-t-numeric-construction-worker.md` —
"T-Numeric-Construction has named T-V2-Retirement-landing-first
dependency for path-(a) v2-refinement-syntax-blocker resolution").

**Director ratification at gunbc#828 #issuecomment-4385530115
implicitly supersedes** that gate via Q-MachineConstraint
sub-decision 6 (UNIVERSAL substrate posture) — the coordination
moves from v2-side cascade to Grounding-side discharge. PR #1856
landing the carrier substrate without T-V2-Retirement landing
first confirms supersession in practice. This brief inherits
the same supersession; T-V2-Retirement coordination is **not**
a hard precondition.

If the supersession is contested at dispatch (e.g., reviewer
challenges that v2-refinement-syntax must land first), surface
to Substrate Mgr (#1739) as STOP — Director re-ratification
needed.

### Substrate state

S3 substrate landed at PR #1856 (squash-merged 2026-05-06; commit on origin/main):
- `dsl/std/machine_constraints.dag` carries `MachineWidth<bits>` (sole R3 machine-axis carrier) + `Compose<Algebra, MachineConstraint>` (parametric type-level interaction shape; unary phantom sum)

S9 Phase-1 step 1+2 already landed:
- `Int = AbelianGroup<GroupCompletion<Nat>>` at PR #1466 / Q6 audit (Slice 3)
- `UInt = Nat` at PR #1818 (Slice 2 partial-ship)

This brief lands **Phase-1 step 3 concrete emission entries**: parametric `Compose<...>` instantiations naming the algebra × machine-axis pairs that emit to target Rust primitives. Per S9 brief Phase-1 step 3 + Q-MachineConstraint sub-decision 5 ("≥3 algebra × constraint pairs is **minimum, not target**").

## Scope (Phase-1 step 3 concrete entries)

### Deliverable 1 — `Int<N>` emission entries

Author parametric instantiations in `dsl/std/integer.dag` (or canonical equivalent — worker greps existing convention at dispatch):

- `Int<32>` ≡ `Compose<Int, MachineWidth<32>>` → emits Rust `i32`
- `Int<64>` ≡ `Compose<Int, MachineWidth<64>>` → emits Rust `i64`
- `Int<128>` ≡ `Compose<Int, MachineWidth<128>>` → emits Rust `i128`

Algebra-side spelling: **first slot is the fully-applied carrier+witness composite `Int`** (= `AbelianGroup<GroupCompletion<Nat>>` per #1466), NOT bare `AbelianGroup`. Per Q-MC sub-decision 3 critical correction at gunbc#828 #issuecomment-4385530115: *"prior phrasing `Compose<AbelianGroup, MachineWidth<64>>` was wrong — `AbelianGroup<T>` is a witness shape over carrier `T`, not a carrier constructor; bare `AbelianGroup` in slot-1 composes witness rather than concept."* `Int` IS the algebraic-concept name (canonical-instance form preserving group-completion edge per `feedback_compositional_not_templating`).

### Deliverable 2 — `UInt<N>` emission entries

Per PR #1818 / Codex P2 review: UInt = Nat (Nat IS the CommutativeSemiring with both additive and multiplicative monoid structure preserved).

- `UInt<32>` ≡ `Compose<UInt, MachineWidth<32>>` → emits Rust `u32`
- `UInt<64>` ≡ `Compose<UInt, MachineWidth<64>>` → emits Rust `u64`
- `UInt<128>` ≡ `Compose<UInt, MachineWidth<128>>` → emits Rust `u128`

Algebra-side spelling: **first slot is the fully-applied algebraic-concept name `UInt`** (= `Nat`; Nat carries the `CommutativeSemiring<Magnitude>` surface per #1818), NOT bare `CommutativeSemiring`. Same Q-MC sub-decision 3 correction applies — witness shape doesn't go in slot-1; the named concept does.

### Deliverable 3 — Cross-program emission consumer wiring (Grounding G2)

Per S3 brief Phase 3 + Q-MachineConstraint sub-decision 6 (UNIVERSAL substrate):
- Grounding Mgr (#1745) consumes the parametric `Compose<...>` instantiations to emit Rust target primitives
- Targets without native machine-width semantics (Python `int`/`float` / etc.) handle omission at Grounding-level discharge — target-conditioned **lowering**, NOT target-conditioned substrate

This brief produces the substrate emission entries. Grounding follow-on (G2 / T-Ground-Rust) consumes them. Cross-program handoff receipt documented in PR body.

### Deliverable 4 — Producer-side substrate-axis verification (NOT end-to-end Rust emission)

Per Director bundled-scope discipline (gunbc#1739 #issuecomment-4392225548): Grounding consumer wiring (per-pair lowering rules + emitted-Rust-primitive verification) is parallel infrastructure DISALLOWED same-PR with substrate emission entries. The full §1.8 #67 `numeric_construction_demonstration` end-to-end Int<32> → Rust i32 round-trip is therefore a **Grounding G2 PR receipt**, NOT this brief's Acceptance.

This brief lands producer-side verification only:
- Each Int<N>/UInt<N> instantiation type-checks under landed `Compose<Algebra, MachineConstraint>` carrier (parametric resolution succeeds; algebraic-concept-name + MachineWidth<N> arguments accepted by the carrier)
- Bootstrap snapshot + parse corpus manifest hold post-instantiation landing (no parser/lowerer regressions)
- Cross-program handoff receipt to Grounding G2 names which entries are now consumable for lowering-rule authoring

§1.8 #67 demonstration receipts split across both PRs:
- Producer half (this brief): substrate emission entries exist; correct algebraic-concept slot-1 spellings; carrier accepts the parametric instantiations
- Consumer half (Grounding G2 PR): Int<32> source DSL → lower → execute → emit Rust i32 → numeric value correct (full round-trip)

If same-PR cross-program co-author is preferred at dispatch (e.g., to land §1.8 #67 in one PR), surface to Substrate Mgr to coordinate with Grounding Mgr; the bundled co-author shape is a separate ratification per bundled-scope discipline.

## Slice — single PR

Phase ordering (PR-internal):
1. Author Int<N> instantiations (Deliverable 1)
2. Author UInt<N> instantiations (Deliverable 2)
3. Cross-program handoff receipt to Grounding Mgr (#1745) for G2 consumer wiring
4. Producer-side substrate-axis verification (Deliverable 4 — type-check + carrier acceptance; full end-to-end demo deferred to Grounding G2 PR per bundled-scope discipline)
5. Bootstrap snapshot regen + parse corpus manifest refresh

## Acceptance

- 6 concrete emission entries landed: `Int<32>` / `Int<64>` / `Int<128>` + `UInt<32>` / `UInt<64>` / `UInt<128>` (≥3 minimum per sub-decision 5; 6 actual)
- Algebra-side spellings correct per Q6 audit + #1818 Codex review:
  - `Int<N>` first slot = `Int` (fully-applied `AbelianGroup<GroupCompletion<Nat>>`)
  - `UInt<N>` first slot = `UInt` (alias of `Nat`, whose carrier is `CommutativeSemiring<Magnitude>`)
- Machine-axis spellings consistent: `MachineWidth<bits>` per S3 ratified shape
- Cross-program handoff receipt to Grounding Mgr (#1745) in PR body for G2 consumer wiring
- Producer-side verification: each instantiation type-checks under `Compose<...>` carrier; bootstrap snapshot + parse corpus manifest hold (NO end-to-end Rust emission demonstration in this PR — Grounding G2 owns lowering rules + emit verification per bundled-scope discipline)
- `Real<64>` demonstration EXPLICITLY OUT-OF-SCOPE (gates on S8 Float migration; Phase 2 absorbs)
- §1.8 ledger rows #17 + #18 advance from DECLARED → PRODUCER_LANDED upon merge (not CONSUMER_LANDED — Grounding G2 follow-on PR carrying per-pair lowering rules + emitted-Rust-primitive verification is what advances the rows to CONSUMER_LANDED; this brief produces substrate emission entries, the consumer is owned by Grounding Mgr #1745). If same-PR Grounding consumer wiring is preferred at dispatch (cross-program co-author), surface to Substrate Mgr to coordinate with Grounding Mgr; default is split-PR producer-then-consumer per bundled-scope discipline (Director ratification at gunbc#1739 #issuecomment-4392225548 — parallel infrastructure DISALLOWED in same PR)
- §1.8 ledger row #67 (`numeric_construction_demonstration`) Acceptance bullet partial — producer half (substrate emission entries exist) landed; consumer half (full Int<32> → Rust i32 round-trip) via Grounding G2 PR; Real<64> half via S8 cascade + Phase-2 brief
- `cargo test --workspace --exclude v2-compiler-tests` green (3 pre-existing v2-compiler --lib failures verified unrelated)
- `cargo test -p v2-compiler-tests` green; strict-compile diagnostic ratchet at 0
- `cargo clippy --all-targets -- -D warnings` clean
- `cargo fmt --all --check` clean
- Citation discipline per `docs/briefs/brief-authoring-checklist.md`: section anchors / rule-text quotes only; no bare `:NNN`
- 5-question authority audit in PR body

## STOP-AND-ESCALATE

- **`Int` / `UInt` algebraic-concept spellings not landed at HEAD** (e.g., types not yet declared in `dsl/std/integer.dag` per #1466 / #1818): worker re-greps; if naming has drifted, surface as substrate-state-divergence STOP and align brief to actual spellings before authoring
- **`Compose<...>` interaction syntax encoding doesn't accept the proposed emission-entry shape** (e.g., parser rejects `type Int<N> = Compose<Int, MachineWidth<N>>` or generic-type-parameter-on-decl is structurally blocked per the limitation valiant-ant-72 noted): STOP — surface to Substrate Mgr; may need parser/lowerer extension or alternative emission-entry encoding
- **Grounding emission rule shape not yet defined** for `Compose<...>` consumption (i.e., G2 doesn't have per-pair lowering rules yet): STOP — coordinate with Grounding Mgr (#1745); cross-program handoff receipt needs concrete consumer surface, not aspirational
- **End-to-end demonstration fails at evaluator or emission stage**: STOP — root-cause; do NOT bridge with placeholder lowering. Per `feedback_fail_closed_discipline`: production code shouldn't ship demo-stub-shaped "numeric_construction_demonstration" Pass-bullets

## Authority audit receipt

1. **Substrate exists?** Per memory + draft-time grep:
   - `MachineWidth<bits>` + `Compose<Algebra, MachineConstraint>` landed at #1856 ✓
   - `Int` algebraic-concept landed (= `AbelianGroup<GroupCompletion<Nat>>` per Slice 3 / #1466)
   - `UInt` algebraic-concept landed (= `Nat`; Nat carries `CommutativeSemiring<Magnitude>` per #1818) — worker confirms at dispatch
   - `GroupCompletion<Nat>` substrate landed per Slice 3 #1466 / Q6 audit
   - Concrete emission entries (Int<N> / UInt<N> parametric instantiations) do NOT yet exist; this brief is producer
2. **Existing brief?** S9 parent brief (`r3-substrate-s9-t-numeric-construction-worker.md`) names Phase-1 step 3 in its slice section. This brief is the worker dispatch packet for that step; not a competing authority
3. **Design-doc match?** Q-MachineConstraint ratification (gunbc#828 #issuecomment-4385530115) names sub-decisions 1-6 verbatim per shape this brief consumes. Slice 3 (#1466) + #1818 + #1856 cement the algebra-side and machine-side carriers
4. **Citations live?** Verified at HEAD post-#1856 merge: `dsl/std/machine_constraints.dag` + algebra-side carriers landed. Worker re-verifies at dispatch
5. **Carrier dissolves the bridge?** Yes — concrete emission entries are the end-state of the substrate-carrier port program for numeric primitives. The "bridge" is the gap between abstract algebra/machine carriers and concrete target-Rust-primitives; parametric `Compose<...>` instantiations dissolve via emission-rule lookup at Grounding lowering layer

## Provenance

Drafted 2026-05-06 post-#1856 merge per Tier-1 brief-queue commitment at gunbc#846 #issuecomment-4390098574 + standing R3 Substrate Mgr cycle assignment at gunbc#1858.

Cross-references S3 (`MachineConstraint<C>` carrier — landed at #1856) and S8 (`ApproximateField<F>` Float migration — pending; Phase 2 of S9 absorbs Real<N> emission). S9 worker pin proud-lynx-311 holds; this brief queues post-Slice-2.5-completion (currently rung-3 in flight) as natural follow-on.
