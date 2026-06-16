# Design: Self-Host Fixed Point (v2 bootstrap-from-self ratchet)

> **Status: DESIGN — map, not territory** (INVARIANTS.md "Map vs territory"). Explicitly
> **gated downstream of the emit ladder**: stages B–D below assume nothing about emit that
> hasn't landed, and only stage A is buildable today. Part of THESIS self-hosting facets 1–3;
> the no-compromise convergence gate is DB-8 per
> [`design-pure-bootstrap-zero.md`](design-pure-bootstrap-zero.md).
>
> Housekeeping: the governing prose lives at **`src/v3/SELF_HOSTING.md`** (not `src/v2/` — a
> commonly mis-cited path); THESIS describes a `hand_maintained_src` ratchet that
> `src/v2/compiler/self_host.dag` does not yet carry. This doc is the design that closes both
> gaps.

## 1. Problem

THESIS facet 2 commits to: *compiling `compiler.dag` produces bit-identical stage0; the
`.dag` graph is the source of truth.* What exists in v2 today is a scaffold-against-contract
(`src/v2/compiler/self_host.dag`): the witness shapes are landed (`StructurallyEqual`,
`BootstrapRoundtripEqual`, `FixedPointEqual`, `PromotionWitness`) but every body returns
`*_not_realized` `Violates`, and `self_host_fixed_point_validate` rejects unconditionally.
`workflow/bootstrap.dag` carries the richer vocabulary (stage hashes, `BootstrapHashPin`,
`FixptStage1Stage2`, `ConvergenceLadder`, `DiverseCompilationAgreement`,
`SeedHonestyDischarge`) — also ahead of any executing consumer.

Three things need designing, in dependency order:

1. **The oracle** — a precise definition of "fixed point reached" (what is compared, with
   what equality, normalized by what: answer — nothing; bit-identical is the gate).
2. **The ratchet** — the `hand_maintained_src` mechanism THESIS promises: the declared,
   monotonically-shrinking list of hand-maintained inputs the fixed point still rests on.
3. **The staging** — what can land *now* without pretending emit works (the E-10 trap this
   lane is most exposed to: a fixed-point validator with no compiler to validate is the
   canonical specification-without-execution).

## 2. What already exists (M9 DFS)

| Piece | Where | State |
|---|---|---|
| Witness shapes: `StageEmissionEquality = Witness<StructurallyEqual>`, roundtrip + fixed-point sub-checks, `PromotionWitness` | `src/v2/compiler/self_host.dag:25-64` | landed, bodies fail-closed `not_realized` |
| The dissolution marker for the equality body | `self_host.dag:51` — "dissolve-on: runner consumes **find_witness/coercion exact structural-equality** witness" | the wiring target is the landed `preservation_rule_exact_structural_equality_zip_fold` (`find_witness.dag`) — stage A below |
| Stage algebra + pins: `v2_stage{0,1,2}_hash`, `pinned_v4_fixed_point_hash`, `BootstrapHashPin`, `CompileStage`, `FixptStage1Stage2`, `ConvergenceLadder` | `src/v2/workflow/bootstrap.dag:30-133` | landed carriers, no executing runner |
| Trusting-trust carriers: `DiverseCompilationRun` / `IndependentCompilerPair` / `DiverseCompilationAgreement` | `bootstrap.dag:78-98` | landed shapes (stage D) |
| Claim + harness consumers | `src/v2/test/claim/self_host/claim_t15_self_host_fixed_point.dag`; `src/v3/compiler/tests/integration/v2_t15_self_host_fixed_point_harness_test.rs` | exist — the consumers stage A flips from shape-checks to executed comparisons |
| Governing prose: 0-floor, DB-8 convergence, N=0 runtime boundary | `docs/design-pure-bootstrap-zero.md`, `src/v3/SELF_HOSTING.md` | authority; this design implements, does not amend |
| Precedent: v2 census (2 hand-maintained of 62 stage0 files), v3 SG-0 census test | THESIS self-hosting section; `src/v3/compiler/tests/integration/sg0_census_test.rs` | the census *mechanism* to port (declared list + structural sweep ⊆ list) |

**Substrate target named (P1):** no new substrate. The design realizes bodies for declared
carriers (`self_host.dag`, `bootstrap.dag`), adds one declared data list
(`hand_maintained_src`, §4), and wires equality to the existing zip-fold predicate.

## 3. The oracle, stated precisely

Stage algebra (the classical ladder, over the carriers that already exist):

- **stage0** — the trusted seed: today the v2 `gunbc` binary (emitted-and-committed Rust,
  itself ~97% self-emitted per the v2 census).
- **stage1** = `stage0(compiler.dag)` — the artifact set produced by compiling the v2
  compiler sources with the seed.
- **stage2** = `stage1(compiler.dag)` — the same sources compiled by the stage1 artifact.

**Fixed point reached ⇔ stage1 ≡ stage2 bit-identically over the declared artifact set.**

Three commitments inside that sentence:

1. **Bit-identical, no quotient.** Unlike the round-trip relation
   ([`design-bidirectional-coercion.md`](design-bidirectional-coercion.md) §4.3), which is
   identity-up-to-declared-quotient over *source*, fixed-point equality is over *emitted
   artifacts* and admits no normalization. DB-8 (deterministic emission) is therefore a hard
   prerequisite: any nondeterminism (map iteration order, timestamp, path leakage) found by
   this gate is a root-cause defect fixed upstream, never normalized away here (Root-Cause
   Depth; C-8).
2. **The artifact set is declared, not implied.** A `FixedPointArtifactSet` data row in
   `bootstrap.dag` names exactly which outputs participate (emitted stage0 Rust; generated
   claims/tests if and when they are emitted). Comparing "whatever came out" is how silent
   scope-shrink hides.
3. **Divergence is a located diagnostic.** stage1 ≠ stage2 reports the first diverging
   artifact and node path (the zip-fold equality already produces located mismatch evidence)
   — not a boolean. Fail-closed reporting is most of the debugging value of the gate.

`PromotionWitness` then has exact semantics: a **candidate** stage1 is promotable to the
pinned reference iff (a) bootstrap-roundtrip holds (`stage0(compiler.dag)` reproduces the
*pinned* stage1 — proving the pin is still derivable from source) and (b) fixed point holds
(candidate(compiler.dag) ≡ candidate's own output). Promotion **writes the new
`BootstrapHashPin` and is operator-gated, never automatic** — the pin is the trust anchor,
and rotating a trust anchor is a decision, not a side effect of CI going green.

## 4. The ratchet: `hand_maintained_src` as declared data + census sweep

THESIS's promised mechanism, made concrete (ports the v2/SG-0 census pattern):

- `self_host.dag` (or a sibling data module) declares
  `data hand_maintained_src: List<HandMaintainedEntry>` where each entry carries: the path
  pattern, the capability it still hand-provides (seed runtime, host transport, CI shim …),
  and a **named dissolution trigger** (P5 — an entry without a trigger is the
  scaffold-without-dissolution problem shape).
- For v2 the list's character differs from v2's: `src/v2/**` is already 0 hand-maintained
  `.rs` — the hand-maintained surface the v2 fixed point *rests on* is the **seed and
  bridges**: v2 stage0 (the runtime executing v2 today), the v3 harness residuals
  (ROADMAP's T-22 bridge row, PB-Runtime deferral), and the hand-synced workflow shims
  (`release.yml` until YamlStatic emission). The list makes that dependency surface explicit
  instead of ambient.
- **Census sweep (the teeth):** a structural check — every file matching the hand-maintained
  patterns is ⊆ the declared list (sweep fails closed on an undeclared entrant), and the
  list itself only shrinks (each removal PR cites the capability's emitted replacement —
  same discipline as the SG-0 receipts). The monotonic-shrink check compares against the
  pinned previous list, alongside the hash pins in `bootstrap.dag`.
- Terminal state: `hand_maintained_src == []` per the 0-floor
  (`design-pure-bootstrap-zero.md`) — at which point facet-2's "the `.dag` graph is the
  source of truth" has no asterisk.

This piece is deliberately **emit-independent**: the list, the entry triggers, and the sweep
are about *today's* dependency surface and can land now (stage A).

## 5. Staging (the gate, honored)

The lane's constraint is "don't design the fixed point assuming emit works until it does."
Each stage names what it consumes and refuses to reach past it:

- **Stage A — land now (no emit assumption).**
  1. Wire `StageEmissionEquality` to the landed exact-structural-equality zip-fold predicate
     (the marker at `self_host.dag:51` names exactly this). The T-15 claim flips from
     shape-contract to **executed comparison on fixture stages**, with the discriminating
     red: a perturbed candidate node ⇒ `Violates` with a located diff.
  2. Land `hand_maintained_src` + census sweep + shrink check (§4).
  3. Keep `self_host_fixed_point_validate` fail-closed for the whole-compiler path — its
     `not_realized` rejection is *correct* today and stands until stage C's inputs exist.
- **Stage B — per-module convergence ladder (consumes emit T3–T6 as it lands).**
  `ConvergenceLadder` gets its semantics: a monotonically growing list of compiler/std
  modules for which `stage0`-emitted artifact ≡ pinned reference, one claim row per module.
  This is "fixed point on a slice" — it exercises determinism (commitment 1) and the
  comparison machinery long before whole-compiler emission, and every emit-ladder tier
  promotion adds rows. The ladder list shrinking-complement is the honest progress metric
  (it cannot be gamed by typecheck — every row is an executed artifact comparison).
- **Stage C — whole-compiler stage1/stage2 (consumes T6–T8).** Realize
  `self_host_fixed_point_validate`: run the stage algebra, produce `PromotionWitness`,
  operator-gated pin rotation. Acceptance is THESIS's sentence verbatim: the v2 binary
  compiles `compiler.dag` and produces bit-identical stage0 Rust plus bit-identical emitted
  artifacts.
- **Stage D — diverse double-compilation (post-fixed-point, optional).** The
  `DiverseCompilationAgreement` carriers get a runner: two independent seeds (v2 binary;
  prior-pin binary) compile the same sources and must agree bit-identically — the
  trusting-trust mitigation (Wheeler's DDC). Designed-for now (the carriers exist), built
  only when stage C is real and a second seed exists.

## 6. Consumers and minimal slice (E-10 / seesaw)

- **Consumers (exist):** `claim_t15_self_host_fixed_point.dag` (claim corpus);
  `v2_t15_self_host_fixed_point_harness_test.rs` (v3 harness — itself a `hand_maintained_src`
  entry with PB-Runtime's deferral as its trigger); the bounded-bridge rows in ROADMAP.
- **Minimal slice = stage A**, nothing more: equality wiring + census data + the T-15 claim
  executing green on fixtures with the perturbed-candidate red. The slice's risk is the
  comparison-and-report machinery (located diffs, fail-closed sweep) — which is precisely
  what stages B/C reuse unchanged. Anything that needs the emit ladder stays a roadmap row
  bound to its tier.

## 7. Dissolution receipts (P5)

- `self_host.dag`'s `stage_emission_equality_not_realized` scaffold body deletes at stage A
  (its marker's named arrival).
- Each `hand_maintained_src` entry deletes with its trigger; the v3 harness entry dissolves
  per the existing `PB-Runtime-External-Toolchain-TestClaims` deferral row (ROADMAP).
- The `bootstrap_roundtrip_not_realized` / `fixed_point_not_realized` bodies delete at stage
  C; the ROADMAP T-22 eval-host bridge dissolves on its own row's trigger, independently.
- Forbidden: promoting a pin without a `PromotionWitness`; adding an artifact to the compare
  set silently; normalizing any divergence inside the fixed-point gate.

## 8. Open questions — escalate, don't improvise

- **Q-S1 — what stage1 *is* during the interpreted era.** Today v2 runs interpreted by the
  v2 binary; whole-compiler Rust emission is the ladder's far end. Recommended framing:
  stage B rows compare *emitted artifacts per module* (meaningful immediately), and "stage1
  as a runnable binary" only becomes a concept at stage C — do not invent an intermediate
  "interpreted stage1 binary" notion; it has no consumer.
- **Q-S2 — artifact-set scope. RESOLVED wave-1 (operator 2026-06-09):** emitted `.rs` —
  with the operator noting the committed-`.rs` era itself is ending soon. That transition
  (stop committing emitted Rust; the N=0 runtime-boundary options in
  `design-pure-bootstrap-zero.md` — shipped binary / runtime crate) is **its own lane**,
  and it interacts with this design only through the declared `FixedPointArtifactSet` row:
  when `.rs` stops being committed, the compare set re-declares against wherever the
  emitted artifacts then live (build output vs tree), and "bit-identical" certifies the
  same relation over the new location. Each widening/relocation of the set remains an
  operator call.
- **Q-S3 — pin rotation authority. RESOLVED (operator 2026-06-09): confirmed** —
  operator-GO per rotation, recorded as the `BootstrapHashPin` update commit; no
  auto-promotion ever.
- **Q-S4 — prose authority location.** `src/v3/SELF_HOSTING.md` governs but lives in the
  frozen tree and is cited (incorrectly) as `src/v2/SELF_HOSTING.md` in planning material.
  Either move it to `docs/` or fix the citations; one authority, one path (P2).

## 9. Non-goals

- No emit-ladder design (that is ctrl#1489 / the T-tier lane; this doc only *consumes* its
  tiers).
- No normalization layer for artifact comparison (bit-identical or diverged — by design).
- No automatic trust-anchor rotation; no CI-driven pin promotion.
- No new bootstrap carriers — `bootstrap.dag`'s existing vocabulary is sufficient until
  stage C teaches us otherwise (E-10: bodies before more shapes).
