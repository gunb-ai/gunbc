# T-FixedPoint unstaging worker brief — pre-staged for SG-0 zero arrival

> **Posture**: PRE-STAGED. Do NOT dispatch until **`sg0_non_test_zero` GREEN** under
> T-LensProducer-Retirement (#2086) — "SG-0 non-test = 0 + ≤1 first-time-bootstrap
> trampoline" per [`docs/r3-structure.md`](../r3-structure.md) line 428 (Director-locked
> 2026-04-28). This brief is mechanical-dispatch-ready at that point; PB Mgr issues
> the green-light comment on parent inbox #2074.
>
> Authored 2026-05-08 by bright-raven-819 (T-FixedPoint lane tracker, PB lane) at
> PB Mgr (warm-dove-618 / inbox #2074) request: pre-author so dispatch is mechanical
> when SG-0 zero arrives.

## Authority

- Closure gate: `pb_self_compile_fixed_point` — R3 horizon stronger interpretation
  ([`docs/r3-structure.md`](../r3-structure.md) lines 89-90, 200;
  [`docs/r3-program-plan.md`](../r3-program-plan.md) §1.8 row 16).
- Design authority: [`docs/design-fixed-point-ratchet.md`](../design-fixed-point-ratchet.md)
  (DB-8 — emit → rustc → run → byte-diff).
- SG-0 choreography authority: [`docs/design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md)
  §"First-time bootstrap" + DB-8 line 154.
- Sequencing: [`docs/r3-program-plan.md`](../r3-program-plan.md) §2.2 step 2 (PB Mgr
  poke-hole F1 corrected — T-LP-Retirement → T-FixedPoint, not reverse).

## Pre-author audit (per `brief-authoring-checklist.md`)

1. **Substrate exists?** YES. [`src/v3/compiler/src/bin/self_host_fixed_point.rs`](../../src/v3/compiler/src/bin/self_host_fixed_point.rs)
   already implements the full pipeline (lines 58-210): pipeline-snapshot pre-check,
   `compiler.dag` parse-probe, emit → rustc → run → byte-diff, `receipt.json` emission.
   Currently "staged": `compile_to_dag(compiler_dag_text)` errors on the cycle
   meta-model, so the bit-identical-diff path stays inert. **This brief is unstaging,
   not producer-landing.**

2. **Existing brief?** No prior brief covers fixed-point unstaging (grep
   `docs/briefs/` for `fixed.point|self.host|self.comp` returns empty 2026-05-08).

3. **Design-doc §recommendation match?** YES — `design-fixed-point-ratchet.md`
   line 271 explicitly stages on "v3 parses + emits a CLI-shaped crate." That stage
   trigger is exactly what this brief converts into a worker dispatch.

4. **File:line citations live?** Verified at HEAD 2026-05-08:
   - `self_host_fixed_point.rs:77` `compile_to_dag(&compiler_dag_text, ...)` — the staging gate
   - `self_host_fixed_point.rs:141-148` byte-equality diff — the R3 horizon assertion
   - `r3-structure.md:428` Director-locked SG-0 closure semantics
   - `design-fixed-point-ratchet.md:271` staging checklist row

5. **Two-horizon clarification respected?** YES. R1 horizon (`verification.dag` +
   `test_runner` evaluation) is already CONSUMER_LANDED per
   [`r3-program-plan.md`](../r3-program-plan.md) §1.8 row 16; this brief targets ONLY
   the R3 stronger interpretation. Worker MUST NOT modify R1-horizon Pass surface.

## Slice (mechanical, on dispatch)

### S1 — Verify SG-0 dispatch precondition

- Confirm `sg0_non_test_zero` GREEN: run the SG-0 census test
  (`cargo test -p v3-compiler --test integration -- sg0_census_test`) and verify
  `EXPECTED_HAND_AUTHORED_NON_TEST` resolves to ≤1 entry, where the single
  remaining entry (if any) is the trampoline path explicitly allocated under
  `design-pure-bootstrap-zero.md` §"First-time bootstrap."
- If census > 0 (and not the allocated trampoline): **STOP-AND-ESCALATE** to PB Mgr
  on inbox #2074 — premise violated, dispatch was premature.

### S2 — Verify `compiler.dag` v3-parses

- Run `cargo run -p v3-compiler --release --bin self_host_fixed_point` and read
  `target/self_host/receipt.json`.
- Required key state: `K_COMPILER_DAG_V3_PARSE == "ok"`. If still parse-error:
  **STOP-AND-ESCALATE** — Lane 3 Stage 3c (v3 grammar coverage of cycle
  meta-model) has not landed; this is a separate gate, not a fix-in-this-brief
  scope item.

### S3 — Verify two-cycle bit-identical diff engages

- After S2 passes, the binary writes `target/self_host/stage1.rs`, invokes `rustc`,
  runs the resulting stage1 binary against `dsl/gunbc/compiler.dag`. The stage1
  binary MUST itself emit a `stage2.rs`. Confirm presence of `stage2.rs` and read
  receipt key `fixed_point_diff`.
- Required: `fixed_point_diff == "ok"` (`self_host_fixed_point.rs:142`).
- If `mismatch`: surface byte-level diff (first divergence offset + ±64 bytes
  context) to PB Mgr; this is a non-determinism finding, NOT something to paper
  over. Per `design-fixed-point-ratchet.md:201` byte equality is the strictest
  contract by design.
- If `skipped_stage2_not_written`: stage1 binary did not produce stage2 — typically
  means the emitted CLI does not yet emit on a positional path argument. Gate is
  cross-coupled with T-LensProducer-Retirement bin-shim emit pattern; surface to
  PB Mgr for choreography.

### S4 — Promote R3 closure-gate state in canonical ledger

- Edit [`docs/r3-program-plan.md`](../r3-program-plan.md) §1.8 row 16
  `pb_self_compile_fixed_point` Status column from
  `CONSUMER_LANDED (R1 horizon; R3 stronger interpretation)` to
  `GREEN (R3 horizon — bit-identical fixed-point + SG-0 zero choreography per
  Director decisions)` with PR commit ref + `target/self_host/receipt.json` SHA.
- Do NOT touch the R1 row narrative (per two-horizon clarification — R1 close is
  not waiting on R3 and the predicate name is the same).

### S5 — CI ratchet

- Add `self_host_fixed_point` to the **release-mode** CI job (per
  `design-fixed-point-ratchet.md:97`) without `continue-on-error`. Until S3 GREEN,
  the slice was tolerated under workflow policy; on R3 closure that policy lifts.
- Verify via test that CI step has no `continue-on-error: true` on the
  `self_host_fixed_point` step (mechanical grep gate is sufficient — no need to
  author a `.dag` claim for CI YAML).

## Acceptance

- `target/self_host/receipt.json` shows `K_PIPELINE_FIXED_POINT_DEFAULT_SOURCE: "ok"`,
  `K_COMPILER_DAG_V3_PARSE: "ok"`, `fixed_point_diff: "ok"`, `K_STATUS: "completed"`.
- Binary exits 0; CI job is no-`continue-on-error` and green.
- §1.8 row 16 carries R3-horizon GREEN with receipt anchor.
- SG-0 census: `EXPECTED_HAND_AUTHORED_NON_TEST` ≤ 1 (trampoline allowance only).

## STOP-AND-ESCALATE

Surface to PB Mgr (parent #2074) when ANY of:

- S1 census check shows non-trampoline entries (premature dispatch).
- S2 `compiler.dag` parse fails (Lane 3 Stage 3c not landed).
- S3 byte-diff `mismatch` (non-determinism — Director decision likely needed on
  whether tooling change vs. emit-path fix is correct response).
- S3 `skipped_stage2_not_written` (lens-producer / bin-shim choreography gap).
- Any unexpected interaction with R1-horizon Pass surface
  (`verification.dag` + `test_runner` evaluation).

## Out of scope

- Authoring/editing `dsl/gunbc/compiler.dag` content (Lane 3 territory).
- Lifting any v3 grammar gap (Lane 3 Stage 3c territory).
- SG-0 census reductions (T-LensProducer-Retirement #2086 territory).
- R1-horizon Pass surface modifications (R1 closure already landed; do not
  silently rename or re-bind that predicate).
- `≤1 trampoline` allocation decisions — those are Director-territory per
  `design-pure-bootstrap-zero.md` §"First-time bootstrap"; worker accepts
  whichever allocation is in force at dispatch time.
