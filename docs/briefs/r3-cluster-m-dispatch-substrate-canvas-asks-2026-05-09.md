# R3 Cluster M Phase 1 — Substrate Carrier Landing Asks (2026-05-09)

**Owner**: Substrate Mgr (warm-wolf-698 / gunbc#2068)
**Authority**: PM-tier dispatch coordination per Director ratification at gunbc#846 #issuecomment-4412309986 (Director answered Ask 2 with "Substrate Mgr standing authority"). Sequencing structure lives at [`docs/audit/r3-cluster-m-sequencing-plan-2026-05-09.md`](../audit/r3-cluster-m-sequencing-plan-2026-05-09.md) §3 (in-flight via concurrent PR #2361; this brief is the dispatch overlay — substantive content here is self-contained and grounded in locked-design authority `docs/design-tests-as-data-completeness.md` §2.1/§2.2 cited below). This brief is a light-touch dispatch surface.

---

## §0. Scope

Two parallel substrate carrier landings — Phase 1 of Cluster M sequencing per (γ) hybrid dispatch shape. Per locked design `docs/design-tests-as-data-completeness.md` §6 line 344, Phase 1 introduces **5 carriers** to `src/v3/std/verification.dag`:

1. **#85** `forall_exists_quantifier_substrate_landed` — `Quantifier` + `QuantifiedTestClaim` + `SuiteClaim` carriers (per design §2.2). `SuiteClaim` has `Enumerated`/`Quantified` variants — existing `TestSuite.claims` sites migrate to wrap in `Enumerated(...)` per design §6 step 1.
2. **#86** `program_generator_carrier_landed` — `ProgramGenerator` + `ProgramShape` carriers (per design §2.1). `ProgramShape` is the element type of `ProgramGenerator`'s body (`List<ProgramShape>`); generator types reference it structurally per design §2.1 (Rust signature in design doc).

Both #85 + #86 carrier landings parallel-authorable; no mutual substrate dep. **Full carrier list per design §6 line 344**: `ProgramGenerator`, `ProgramShape`, `Quantifier`, `QuantifiedTestClaim`, `SuiteClaim` — all five extensions to `src/v3/std/verification.dag` (split #85/#86 follows the gate-id partition: #85 = quantifier surface; #86 = generator surface).

## §1. Authority correction 2026-05-09 (codex BLOCKING on PR #2361 sha `c6c3fb96`)

Prior framing of this brief as "substrate canvas asks" + "surface for Director ratification" was a duplicate-authority anti-pattern. The substrate carriers are **already canonically defined** in [`docs/design-tests-as-data-completeness.md`](../design-tests-as-data-completeness.md) §2.1 + §2.2.

Per locked-design §1 Authority discipline:
> "All §8 design questions resolved in-doc per `feedback_design_before_implement` — **no Director ratification required before lane dispatch** (only standard cascade gates: R2-Evaluator landed; existing TestClaim infrastructure from DB-15 R2)."

Single canonical authority per INVARIANTS P2: locked design owns shape; this brief owns dispatch coordination only.

## §2. Dispatch disposition (cite-and-execute)

- **Dispatch shape**: (γ) hybrid — Substrate Mgr lands carriers → Verification discipline → Verification bulk-port coordinator
- **Authoring authority**: Substrate Mgr standing authority (worker brief authoring + dispatch under standing authority; no Director ratification needed — locked design resolves carrier shape)
- **Pattern**: substrate-shape canvases for novel substrate (e.g., T-WAD Slice 2); migration / locked-design carrier landings dispatch directly. #85/#86 fall in the latter category.

## §3. Substantive guidance per carrier (per locked design §2.1/§2.2)

### §3.1 #85 — `Quantifier` + `QuantifiedTestClaim` carriers (per locked design §2.2)

**Locked carrier shape** (`docs/design-tests-as-data-completeness.md` §2.2):

```
type Quantifier = ForAll | Exists  // closed two-variant sum

type QuantifiedTestClaim {
  generator: ProgramGenerator
  quantifier: Quantifier
  // ... see design doc for full Rust signature
}

type SuiteClaim
  | Enumerated(TestClaim)            // existing single-source claim
  | Quantified(QuantifiedTestClaim)  // generator-driven family claim
```

`Quantifier` is a closed two-variant sum exhausting structurally meaningful quantifications over a `ProgramGenerator`'s output. `QuantifiedTestClaim` covers the property-based axis where the existing `TestClaim` covers single-source enumerated tests. `SuiteClaim` is the **wrapper sum** that lets `TestSuite.claims` hold both — existing claims migrate to `Enumerated(TestClaim)`; quantified claims land as `Quantified(QuantifiedTestClaim)`.

**Worker scope**: extend `src/v3/std/verification.dag` to add `Quantifier` + `QuantifiedTestClaim` + `SuiteClaim` per design §2.2 spec. Migrate existing `TestSuite.claims: List<TestClaim>` sites to `TestSuite.claims: List<SuiteClaim>` with claims wrapped in `Enumerated(...)` (mechanical migration; backward-compatible at the predicate level). No Director ratification needed.

### §3.2 #86 — `ProgramGenerator` + `ProgramShape` carriers (per locked design §2.1)

**Locked carrier shape** (`docs/design-tests-as-data-completeness.md` §2.1):

```
type ProgramShape
  | LiteralProgram(String)  // bootstrap variant per design §8.2
  // richer shapes deferred per design §8.2 dissolution trigger

type ProgramGenerator {
  // structural reference to a generator declaration whose body
  // produces a List<ProgramShape> (or iterator-shaped value)
  // ... see design doc for full Rust signature
}
```

`ProgramShape` is the element type — the "shape" a generator emits. Bootstrap variant `LiteralProgram(String)` per design §8.2 (richer shapes attach later via the same generator carrier; substrate-shape stable). `ProgramGenerator` is a structural reference to a generator declaration whose body returns `List<ProgramShape>` — **not** a roster of "shape kinds" (which would replicate the closed-roster failure flagged by `lens-library-design.md` §1.5).

**Worker scope**: extend `src/v3/std/verification.dag` to add `ProgramGenerator` + `ProgramShape` per design §2.1 spec. Composition with §3.1 #85: `QuantifiedTestClaim.generator: ProgramGenerator`; `ProgramGenerator` body returns `List<ProgramShape>`. No Director ratification needed.

## §4. STOP-and-PING posture

If during landing the worker discovers an unexpected shape question that the locked design §2.1/§2.2 does not resolve, STOP-and-PING via Substrate Mgr inbox per `feedback_construction_over_ratchets` rather than authoring a canvas mid-port. Surface the question; Director ratifies the surfaced shape; worker resumes.

## §5. Dispatch trigger

Substrate Mgr (warm-wolf-698) dispatches worker brief(s) for #85/#86 carrier landings under standing authority once PR #2361 lands (sequencing plan ratified by Director ratification at #846 #issuecomment-4412309986). Workers can be queued ahead of merge per pre-authored brief discipline.

## §6. Receipt

- `src/v3/std/verification.dag` extended with `Quantifier`, `QuantifiedTestClaim`, `ProgramGenerator` per design §2.1/§2.2 specs
- §1.8 ledger Status moves DECLARED → CONSUMER_LANDED on substrate carrier landing PR merges
- Phase 2 (#87) Verification Mgr discipline-pattern brief unblocks once #85/#86 carriers land

## §7. Velocity context

Per [`docs/audit/r3-pb0-velocity-walk-2026-05-09.md`](../audit/r3-pb0-velocity-walk-2026-05-09.md): Cluster M is **critical-path-load-bearing** for PB-0 closure. Phase 1 (this brief) gates Phase 2 (#87) gates Phase 3 (#84 bulk-port). Total Cluster M close target: 4-8 weeks per sequencing plan §6; fits 8-12 week R3 window with parallel dispatch per operator "staffing is not a concern" directive.

---

**End of brief.**
