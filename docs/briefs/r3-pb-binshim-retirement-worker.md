# R3 PB — BinShim instances + emit pattern + retirement dispatch (PB-owned planning brief)

**Status:** PROPOSAL (planning artifact, dispatch-gated). Authored 2026-04-29 by PB Manager continuation per the BinShim instances + emit pattern row in [`docs/briefs/r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md#program-scope-t-pb-post-r1-only) and the Item 5 ownership lock from [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) §5.4 (LANDED via #1176/#1186).

**Substrate-disposition update 2026-05-09**: §7.3 `CensusListConstant` / filter-predicate question (formerly an open Substrate gap) was **RATIFIED by Substrate Manager** (warm-wolf-698) at gunbc#2068 [c#4411574142](https://github.com/gunb-ai/gunbc/issues/2068#issuecomment-4411574142). Locked shape: **(b) `CensusSubsetCount` filter** with closed predicate `BinShimFilesSubsetPredicate` mirroring the existing `LensProducerFilesSubsetPredicate` precedent at [`src/v3/std/verification.dag:41-43`](../../src/v3/std/verification.dag). Carriers + consumer wire-up enumerated below in §"Substrate landings (locked shape)".

**Item-4 partial-landing update 2026-05-09**: PR #2282 (closes gunbc#2227) landed the **Item-4 carrier substrate** (`runtime.dag` PB-Runtime types + regenerated bootstrap surface) only — it did **not** author the §7.1 `pb_runtime_equivalent_to_evaluator_on_corpus` equivalence claim, the bin-shim emitter, the §7.2 BinShim equivalence fixture, or any per-shim `BinShim` instance rows. Those remain dispatch preconditions for this lane.

**This brief stays in PROPOSAL.** This 2026-05-09 amendment is a **substrate-disposition absorption** (encoding §7.3 ratification per #2068 [c#4411574142](https://github.com/gunb-ai/gunbc/issues/2068#issuecomment-4411574142)) — it does **not** flip the lane to ready. Dispatch remains gated on §"Dispatch preconditions" below, which still includes §7.1 Item-4 sub-gate green + §7.2 equivalence fixture authoring + the emitter / instance-row landings. Per `INVARIANTS.md` [P5](../../INVARIANTS.md), the dispatch gate must continue to fail-closed against incomplete Item-4 / Item-5 deliverables.

**Owning manager:** Pure Bootstrap Manager (R2 → R3 continuation per `r3-structure.md` §"Manager structure" Item 1).

**Lane size:** S (per `r2-pure-bootstrap-manager.md` R3 continuation table).

**This is a planning artifact — not a dispatch order.** Worker dispatch is gated; see §"Dispatch preconditions" + §"STOP conditions". PB Manager re-reads this brief at gate-clear to issue worker dispatch.

## Scope

PB owns three deliverables for the bin-shim class of T-LensProducer-Retirement sub-gate iii (`regen_lens_dot_rs_retired`):

1. **Per-shim `BinShim` instance declarations** under `dsl/std/runtime/bin_shims/` — one `data <name>_shim: BinShim = { ... }` per existing PB-owned hand-Rust bin (§4.1 of the design doc names `regen_lens.rs` as the canonical first target; §4.1 also names other regen drivers as the broader class). Each instance is a pure-data witness of the existing pipeline's load/compile/write composition.
2. **The bin-shim emit pattern** — a `.dag` emitter program (analogous in shape to `dsl/extdeps/languages/rust/emit.dag` per design doc §4.2) that translates `BinShim` declarations to Rust files headed `// AUTO-GENERATED from <path> — DO NOT EDIT.`.
3. **Retirement dispatch** — once (1) and (2) land, dispatch the per-bin retirement workers in the order called out by §4.3, starting with `regen_lens.rs` as canonical first cut (sub-gate iii of T-LensProducer-Retirement; see §5.1).

PB does **not** own and **must not** edit:

- The `BinShim` carrier-type shape (additional fields, signature refinement of `entry: () -> std.process.ProcessExit`, etc.) — that's Substrate Manager territory per §5.4 boundary; generalized carrier-shape evolution escalates via [`INVARIANTS.md#p1-modeling-faithfulness`](../../INVARIANTS.md#p1-modeling-faithfulness) substrate-fact-introduction procedure.
- The PB-Runtime interpreter-as-data implementation (Item 4 / sub-gates i + ii — `lens_apply.rs` and `lens_testgen.rs` retirement). Those are separate and gated separately.
- The §7.3 `CensusListConstant` / filter-predicate substrate **shape question** (Substrate Mgr disposition territory). Per the 2026-05-09 ratification at #2068 [c#4411574142](https://github.com/gunb-ai/gunbc/issues/2068#issuecomment-4411574142), the Substrate Mgr authorized PB-lane authoring of the four strict-mirror artifacts enumerated in §"Substrate landings (locked shape)" (mirror of the existing `LensProducerFilesSubsetPredicate` precedent — no novel substrate fact). PB lane MUST NOT introduce a *novel* substrate shape; it MAY land the strict-mirror declarations as part of the retirement work.

## First slice — `regen_lens.rs`

Retirement-readiness checklist (owners, SG-0 / `REGEN_OUTPUTS`, STOP routing, dispatch row): [`r3-pb-regen-lens-first-binshim-target-retirement-readiness.md`](r3-pb-regen-lens-first-binshim-target-retirement-readiness.md).

Instance-declaration framework + naming convention (PB-owned authoring surface): [`dsl/std/runtime/bin_shims/README.md`](../../dsl/std/runtime/bin_shims/README.md). Per-shim `.dag` files land here once the `BinShim` carrier is live on main; the README locks the `<bin_name>.dag` / `data <bin_name>_shim` convention so per-shim retirement workers have a consistent target.

Per design doc §4.3 the dissolution path is:

1. Author `data regen_lens_shim: BinShim = { name: "regen_lens", entry: regen_lens, ... }` in `dsl/std/runtime/bin_shims/regen_lens.dag`.
2. Land the `BinShim` emitter (deliverable 2). Emitter generates `src/v3/compiler/src/bin/regen_lens.rs` from the `.dag` declaration.
3. Verify behavioral equivalence vs the current hand-Rust shim (per §7.2 fixture; **not** byte-identity — the emitted form carries an `AUTO-GENERATED` header and may differ in formatting / comment shape).
4. Retire the hand-Rust file.
5. SG-0 census drops one entry (`regen_lens.rs`); ratchet `EXPECTED_HAND_AUTHORED_NON_TEST` count goes down by one.

Subsequent bin-shims (other `regen_*.rs` drivers, `self_host_fixed_point.rs` to the extent it's bin-shim-shaped, etc.) follow the same template, dispatched serially by PB Manager. The lane closes when every PB-owned hand-Rust bin in `src/v3/compiler/src/bin/` is either retired via this pattern or has a Director-approved continuing exception.

## Dependencies / gates

Per design doc §5.4 + §7 + the broader R3 Evaluator-gated lane discipline in [`docs/r3-structure.md`](../r3-structure.md):

1. **R2-Evaluator landed.** §5.4 calls out Evaluator Manager as co-author of the convergence path. PB-Runtime mirrors the Evaluator's runtime-value model; without Evaluator the bin-shim emitter and the §7.2 equivalence fixture have no runtime to verify against.
2. **Item 4 (PB-Runtime interpreter-as-data) landed.** Item 5 (this lane) inherits Item 4's runtime-value vocabulary; the bin-shim emitter is "one of many `.dag` emitters" per §6 anti-bridge invariant #4, sharing the same fold-over-`Lens<C>` substrate.
3. **Substrate-owned `BinShim` carrier type live.** The `type BinShim { ... }` sketch in design doc §4.2 is the locked design; Substrate Manager lands the carrier (in `dsl/std/runtime/bin_shim.dag` or wherever Substrate picks per their dispatch). PB cannot author instance declarations against a carrier that doesn't exist.
4. **`std.process.ProcessExit` substrate live.** Design doc §4.2 names this as the structural contract for translating `.dag` program return values into host process exit codes. Already declared per design doc reference; verify on dispatch.
5. **No-new-bin-shim-hand-Rust fixture's substrate prerequisite — RESOLVED 2026-05-09.** Substrate Manager (warm-wolf-698) ratified shape (b) `CensusSubsetCount` reuse with closed predicate `BinShimFilesSubsetPredicate` at gunbc#2068 [c#4411574142](https://github.com/gunb-ai/gunbc/issues/2068#issuecomment-4411574142). See §"Substrate landings (locked shape)" below for required carrier + consumer wire-up.

These dependencies are cumulative: R2-Evaluator → Item 4 (PB-Runtime) → BinShim carrier + ProcessExit live → bin-shim instances/emitter (this lane) → per-shim retirement.

## Substrate landings (locked shape)

Authoritative source: Substrate Mgr (warm-wolf-698) disposition at gunbc#2068 [c#4411574142](https://github.com/gunb-ai/gunbc/issues/2068#issuecomment-4411574142). Per `feedback_strict_mirror_vs_novel_substrate_fact`, this is a strict mirror of the existing `LensProducerFilesSubsetPredicate` precedent (no novel substrate fact; ratifies directly).

The retirement worker MUST land the following four artifacts (atomic with the first per-shim retirement, or as a preceding scaffolding PR):

1. **Substrate marker type** at [`src/v3/std/verification.dag`](../../src/v3/std/verification.dag) (adjacent to the `LensProducerFilesSubsetPredicate` block at `:41-43`):
   ```dag
   // Closed PB bin-shim predicate follows the same declaration-ref pattern
   // as the lens-producer subset.
   type BinShimFilesSubsetPredicate {}
   ```
2. **Substrate marker value**, co-located:
   ```dag
   data bin_shim_files_subset_predicate: BinShimFilesSubsetPredicate = {}
   ```
3. **Runtime predicate body** in [`src/v3/compiler/src/test_runner.rs`](../../src/v3/compiler/src/test_runner.rs), adjacent to `is_lens_producer_census_path`:
   ```rust
   fn is_bin_shim_census_path(path: &str) -> bool {
       path.starts_with("src/v3/compiler/src/bin/")
   }
   ```
4. **Dispatch branch** in `eval_census_subset_count_shape` (parallel to the existing `LensProducerFilesSubsetPredicate` branch around `test_runner.rs:3273-3290`) that resolves the `subset_predicate` field as `BinShimFilesSubsetPredicate` and counts entries via `is_bin_shim_census_path`.

Bin-shim file inventory at main `5a13ed800` (initial count = **9**; closure at 0):

```
src/v3/compiler/src/bin/emit_method_template_projection.rs
src/v3/compiler/src/bin/r1c_e_emit_gates.rs
src/v3/compiler/src/bin/regen_bootstrap.rs
src/v3/compiler/src/bin/regen_lens.rs
src/v3/compiler/src/bin/regen_parse.rs
src/v3/compiler/src/bin/regen_parse_tables.rs
src/v3/compiler/src/bin/regen_tokenize.rs
src/v3/compiler/src/bin/regen_v3.rs
src/v3/compiler/src/bin/self_host_fixed_point.rs
```

All 9 are members of `expected_hand_authored_non_test`; per-shim retirement net-decreases the subset count by one and atomically grows `REGEN_OUTPUTS` / `GENERATED_FILES` by one.

## Acceptance — what a future implementation PR must prove

Per design doc §7. The locked TestClaim names are authoritative; do not rename:

### `pb_runtime_equivalent_to_evaluator_on_corpus` (§7.1)

Owner: Item 4 lane (out of this brief's scope, but cited because §5.1 makes Item 4 a precondition). PB-Runtime evaluation of every `.dag` program in the certification corpus equals R2-Evaluator's evaluation of the same program. Substrate the bin-shim lane consumes; not authored here.

### `regen_lens_bin_shim_emits_behaviorally_equivalent_to_hand_rust` (§7.2)

Behavioral equivalence (NOT byte-identity per anti-bridge invariant #1) between the emitted Rust from `data regen_lens_shim: BinShim` and the current hand-Rust `regen_lens.rs`. Authoring shape per design doc:

```dag
test_claim {
  name: "regen_lens_bin_shim_emits_behaviorally_equivalent_to_hand_rust"
  ...  // shape locked in design doc §7.2
}
```

If the existing `TestPredicate` envelope at `src/v3/std/verification.dag:108-235` cannot express "behavioral equivalence between emitted and hand-Rust binaries," this is a **substrate gap** — escalate per `INVARIANTS.md` [P1](../../INVARIANTS.md#p1-modeling-faithfulness) to Substrate Manager. PB does not invent a new `TestPredicate` variant from this lane.

### `no_new_bin_shim_hand_rust` (§7.3) — substrate-shape locked 2026-05-09

Closed-set retirement gate: hand-Rust bin-shim count never exceeds the documented retirement schedule. **Substrate disposition resolved** per §"Substrate landings (locked shape)" above. Authorable acceptance shape:

```dag
data census_subset_claim: TestClaim = {
  name: "no_new_bin_shim_hand_rust",
  source: "let x: Int = 1",
  file_name: "no_new_bin_shim_hand_rust.v3",
  predicate: CensusSubsetCount {
    authority: census_authority,
    list_constant: expected_hand_authored_non_test,
    subset_predicate: bin_shim_files_subset_predicate
  },
  requires: []
}
```

**TestClaim verdict semantics**: the runtime predicate is **zero-only** (`Pass` iff count==0; otherwise `Fail` with the observed count). The TestClaim is **not** a schedule-bound gate; it does not encode an interim bound and cannot pass with a non-zero count. "Per-PR shrink receipts" referenced in the disposition are **PR-level milestones tracked outside this TestClaim** (e.g., R3 BinShim retirement lane progress reports), not TestClaim verdicts. Closure of §1.8 sub-gate #7 is gated on this TestClaim flipping to Pass at count==0.

### Other behavioral acceptance bullets (per dispatch directive)

- **Behavioral equivalence to hand Rust.** Covered by §7.2 above.
- **No new hand-Rust bin shim.** Covered by §7.3 above (substrate-gated).
- **Generated file is not editable authority.** Emitted bin-shim files carry the `// AUTO-GENERATED from <path> — DO NOT EDIT.` header per design doc §4.2; SG-0 census already enforces "hand-authored `// AUTO-GENERATED` header does not slip through" (per `sg0_census_test.rs` header rule). The retirement PR must add the per-shim emit path to `REGEN_OUTPUTS` in `src/v3/compiler/build.rs` so the SG-0 partition counts the file as generated.
- **SG-0 delta.** For each retired bin-shim, `EXPECTED_HAND_AUTHORED_NON_TEST` shrinks by one; `GENERATED_FILES` (via `REGEN_OUTPUTS`) grows by one. The retirement PR must update both atomically.

## Non-goals

This lane explicitly does NOT cover:

- **T-FixedPoint implementation** (#1169 lane; gated separately).
- **`lens_apply.rs` / `lens_testgen.rs` retirement** — Item 4 sub-gates i + ii. Different runtime substrate (PB-Runtime interpreter-as-data); this lane consumes Item 4 but does not implement it.
- **PB-Runtime interpreter-as-data implementation** — Item 4. This lane assumes Item 4 has landed.
- **Substrate-owned `BinShim` carrier-shape edits** — additional fields, refining `entry: () -> std.process.ProcessExit` beyond the locked signature, etc. Escalate to Substrate Manager via [P1](../../INVARIANTS.md#p1-modeling-faithfulness) if the retirement work surfaces a real shape gap.
- **§7.3 `CensusListConstant` / filter-predicate substrate *shape* authoring** — Substrate Manager territory per [P1](../../INVARIANTS.md#p1-modeling-faithfulness) disposition. **As of 2026-05-09 (#2068 c#4411574142) the disposition is RATIFIED** as a strict-mirror of `LensProducerFilesSubsetPredicate`; PB lane authoring of the four enumerated mirror artifacts is authorized per §"Substrate landings (locked shape)". PB lane MUST NOT introduce *novel* shape (extra fields on `CensusSubsetCount`, alternative coproducts, etc.) — that escalates back to Substrate Mgr.

## Dispatch preconditions

PB Manager dispatches per-shim retirement workers when **a single readiness check** confirms all of:

1. R2 close signal (R2 Release Manager closure ledger) — same precondition as the other 7 Evaluator-gated R3 lanes per `docs/r3-structure.md` §"R3 worker dispatch precondition".
2. R2-Evaluator landed and stable.
3. **Item 4 (PB-Runtime interpreter-as-data) sub-gate green** — `pb_runtime_equivalent_to_evaluator_on_corpus` evaluates true.
4. **Substrate-owned `BinShim` carrier type live on main** — `type BinShim { ... }` declared per design doc §4.2 (verify by grep at dispatch).
5. **Substrate-owned `CensusListConstant` / filter disposition for §7.3 picked** — **RESOLVED 2026-05-09** per gunbc#2068 [c#4411574142](https://github.com/gunb-ai/gunbc/issues/2068#issuecomment-4411574142); locked-shape carriers + consumer wire-up enumerated in §"Substrate landings (locked shape)".

If any of (1)-(5) is unmet, this brief stays in PROPOSAL state; PB Manager does not dispatch.

## STOP conditions

Worker MUST STOP and escalate to PB Manager (which escalates to Director if cross-program) when:

- **`BinShim` carrier shape pressure.** Authoring a `data <name>_shim: BinShim = { ... }` instance reveals that the locked carrier shape (per design doc §4.2) cannot express the bin's actual entry-function signature or pipeline composition. That's a Substrate-owned carrier-shape evolution per §5.4 boundary; signal Substrate Manager via [P1](../../INVARIANTS.md#p1-modeling-faithfulness) — do not extend the carrier from this lane.
- **No fitting `TestPredicate` variant for §7.2 equivalence.** If "behavioral equivalence between emitted Rust and hand-Rust binary" cannot compose from existing variants at `src/v3/std/verification.dag:108-235`, that's a substrate gap — [P1](../../INVARIANTS.md#p1-modeling-faithfulness) to Substrate Manager. Do not invent a new variant here.
- **Emit-pattern divergence from `dsl/extdeps/languages/rust/emit.dag` shape.** Per anti-bridge invariant #4 the `BinShim` emitter "is one of many `.dag` emitters; its shape mirrors `dsl/extdeps/languages/rust/emit.dag`." If the retirement worker finds itself authoring parallel emit logic, STOP — that's a sign the carrier or pattern is wrong.
- **§7.3 substrate disposition drift.** The locked shape is recorded in §"Substrate landings (locked shape)" above; if a worker grep finds the substrate marker types/values already exist on main but with a different shape than enumerated, STOP — surface the drift to PB Mgr + Substrate Mgr rather than authoring against the stale brief encoding.
- **SG-0 census drift the wrong way.** Per `feedback_ratchet_only_down`: if the retirement PR does not net-decrease the hand-Rust bin-shim count, that's a defect in the work, not a ratchet to relax.

## Cross-program signals

- **Evaluator Manager** — runtime-value model convergence (§5.4 cross-program coordination); PB-Runtime mirrors Evaluator's value model. Per-shim authoring confirms the mirror; deviation is a co-design escalation, not unilateral PB authority.
- **Substrate Manager** — `BinShim` carrier-shape evolution (§5.4 boundary), §7.3 `CensusListConstant` / filter-predicate disposition (§7.3 substrate prerequisite), any new `TestPredicate` variant proposed by §7.2's equivalence requirement.
- **R3 Release Manager** (when authored) — sub-gate progress reporting per `r2-pure-bootstrap-manager.md` §"Reporting cadence" (closure-ledger reports T-LensProducer-Retirement sub-gate progress within the one-program lane).
- **Director** — scope-change escalations (e.g., a bin-shim that needs Director-approved continuing exception rather than retirement); [P1](../../INVARIANTS.md#p1-modeling-faithfulness) escalation arbitration if Substrate disposition stalls.

## Cross-refs

- Parent manager brief: [`docs/briefs/r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md#program-scope-t-pb-post-r1-only) — R3 continuation row "BinShim instances + emit pattern + retirement dispatch" + [`Locked design decisions consumed`](r2-pure-bootstrap-manager.md#locked-design-decisions-consumed-per-1078-dialogue--cascade) §"PB-Runtime interpreter-as-data + PB-1 bin-shim emit pattern (LANDED via #1176)".
- Design lock: [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) — §4.1 bin-shim class enumeration; §4.2 `BinShim` carrier-shape sketch + emitter shape; §4.3 dissolution path; §4.4 first-time-bootstrap compatibility; §5.1 R3-T-LensProducer-Retirement sub-gate decomposition; §5.4 PB/Substrate/Evaluator boundary; §6 anti-bridge invariants; §7.2 BinShim equivalence fixture; §7.3 No-new-bin-shim-hand-Rust fixture (with substrate prerequisite call-out).
- T-LensProducer-Retirement parent lane: [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure" T-LensProducer-Retirement row.
- Sibling R3-PB lane (separate scope, gated separately): T-FixedPoint planning brief — [`docs/briefs/r3-pb-t-fixedpoint-worker.md`](r3-pb-t-fixedpoint-worker.md).
- Substrate-fact-introduction procedure (escalation path): [`INVARIANTS.md#p1-modeling-faithfulness`](../../INVARIANTS.md#p1-modeling-faithfulness) (Procedure).
- SG-0 census authority + generated-file partition: `src/v3/compiler/tests/integration/sg0_census_test.rs` (`EXPECTED_HAND_AUTHORED_NON_TEST` + `GENERATED_FILES`); `src/v3/compiler/build.rs` `REGEN_OUTPUTS`.
- First-shim source (unchanged until retirement): `src/v3/compiler/src/bin/regen_lens.rs`.
- First-target readiness (planning only): [`r3-pb-regen-lens-first-binshim-target-retirement-readiness.md`](r3-pb-regen-lens-first-binshim-target-retirement-readiness.md).
- Existing emitter shape reference (per anti-bridge invariant #4): `dsl/extdeps/languages/rust/emit.dag`.
