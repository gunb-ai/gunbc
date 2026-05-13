# R3 Gate 87 Lens-Cementing Test-Discipline Dispatch — 2026-05-13

**Owner:** Verification Mgr session `lively-raven-354`.

**Purpose:** decompose `lens_cementing_test_discipline_complete` into concrete child work that preserves the lens-completeness invariant while gate #87 stays tied to the `regen.dag` registry corpus.

This is a dispatch artifact, not a second gate authority. The acceptance authority remains `docs/r3-structure.md` §"T-Tests-As-Data-Completeness" and `docs/r3-program-plan.md` §1.8 row #87. Pattern details live in `docs/briefs/r3-v-cluster-m-87-cementing-worker.md` §7 and `docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md`.

## Dispatch Invariant

For every `LensRegistryEntry` in `src/v3/compiler/regen.dag`, the gate-87 corpus must have one visible cementing path:

- a `.dag` receipt under `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_<lens>.dag`;
- runner inclusion through `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`;
- dispatch coverage through `src/v3/compiler/tests/dag/cementing_dispatch.dag`;
- any temporary Rust pin named in `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs` with an explicit dissolution trigger.

Rows outside `regen.dag` remain Band-C / #84 bulk-port scope. Do not use them to prove or reopen gate #87.

## Child Work Items

### G87-D1 — Registry-Invariant Audit

Audit `src/v3/compiler/regen.dag`, `docs/v3-lens-capability-register.md`, `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`, and `src/v3/compiler/tests/dag/cementing_dispatch.dag` for exact row alignment.

Acceptance:

- Every registry lens name appears in the runner inventory.
- Every runner inventory lens has a corresponding `t_r3_gate_87_cementing_regen_<lens>.dag` file.
- `cementing_dispatch.dag` covers the same registry names and does not introduce a parallel hand list.
- Any mismatch is fixed in the same PR or escalated with the specific missing surface.

Verification:

```bash
cargo test -p v3-compiler r3_gate_87
```

### G87-D2 — COMPLETE-Flip Same-PR Checklist

**Status:** normative reviewer checklist (G87-D2 landed 2026-05-13). Expanded rationale and predicate notes remain in [`TESTING.md`](../../TESTING.md) (*Cementing tests (Band C)* → *Same-PR checklist — promoting a row to `BEHAVIORALLY COMPLETE`*) and predicate classes in [`r3-cementing-discipline-pattern-2026-05-12.md`](r3-cementing-discipline-pattern-2026-05-12.md) §2.

Use this list whenever work **(a)** promotes `data lens_capability_register_rows` / the prose table to `LensCapabilityBehavioralComplete` for a `regen.dag` lens, or **(b)** adds a new `LensRegistryEntry` consumer that should participate in gate #87. The two triggers share one **receipt stack**: skipping any step below breaks the **lens-completeness invariant** for the `regen.dag` corpus and is **non-mergeable** — CI may not catch every prose-only drift, but reviewers should treat an incomplete stack as the same class of defect as a failing `CementingDispatchMatchesProjection` / `r3_gate_87` ratchet.

#### Atomic same-PR edit set (all in one change)

1. **`src/v3/std/verification.dag`** — `data lens_capability_register_rows`: structural register row (`lens_basename`, structural axis, behavioral axis, `v2_counterpart`) matches the promotion.
2. **`docs/v3-lens-capability-register.md`** — capability table row: behavioral marker, structural marker, v2 column, and “What v2 has that v3 drops” cleared to `N/A` before claiming `COMPLETE`.
3. **`src/v3/compiler/regen.dag`** — `LensRegistryEntry` for the generated consumer (new row on add; unchanged registry name only if the change is pure status — still touch this file if the entry’s contract text must track the promotion).
4. **`src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_<lens_stem>.dag`** — per-lens gate-#87 harness (`<lens_stem>` matches the `regen.dag` / file-stem convention used across gate #87).
5. **`src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`** — append or update `R3_GATE_87_CEMENTING_REGEN_SUITES` (path, suite name, claim names) so `t_pb_b_1_dag_runner_test` stays the single merge-visible inventory.
6. **`src/v3/compiler/tests/dag/cementing_dispatch.dag`** — add or extend the Band-C receipt list so `CementingDispatchMatchesProjection` stays fail-closed against `lens_capability_register_rows`.
7. **`src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`** — extend **when** a temporary Rust pin remains required; each pin carries an explicit **dissolution trigger** (carrier, runner capability, or lane) in the module or harness doc, same PR as the placeholder.

Temporary Rust-only cementing remains same-PR Band-C only when the Rust module, `tests/integration.rs` `#[path]`, any `EXPECTED_HAND_AUTHORED_TEST` census line, and the named blocker/dissolution path land **together** with the `.dag` / register edits — not as a follow-on ticket.

#### Predicate / receipt class (pick before coding)

| Register / row shape | Required Band-C evidence (`.dag` by default) |
|---|---|
| **Real v2 counterpart** (`LensCapabilityV2RealV2`) | `DifferentialEquals` and/or an existing reviewed frozen-v2 oracle predicate used for sibling regen lenses — not prose parity. |
| **v3-native `N/A` / no v2 counterpart** (`LensCapabilityV2NoneV3Native` or `LensCapabilityV2NotApplicable` with behavioral `COMPLETE`) | `LensOutputEquals` on minimal programs or constructed `Dag` shapes, **or** `SymbolicCostExprEquals` when the published contract is symbolic-cost-shaped — not “structurally TERMINAL only.” |
| **Helper / intentionally partial registry surface** | Narrow `.dag` claims (often `Compiles`) **plus** explicit dissolution trigger and, when required, the paired Rust pin in `r3_gate_87_lens_cementing_regen_receipts_test` in the **same** change. |

#### Reviewer one-pass

Before approving: confirm steps 1–7 above for this lens name, confirm the predicate row in the table matches the v2 axis, and confirm `cargo test -p v3-compiler r3_gate_87` is green on the PR. A `BEHAVIORALLY COMPLETE` promotion (or new `LensRegistryEntry`) without the full receipt stack violates the gate-#87 lens-completeness invariant and must not merge.

Verification:

```bash
rg -n "COMPLETE|LensRegistryEntry|R3_GATE_87_CEMENTING_REGEN_SUITES|cementing_dispatch|non-mergeable|receipt stack|lens_capability_register_rows" \
  docs/briefs src/v3/compiler/regen.dag src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs
```

### G87-D3 — Placeholder-Dissolution Ledger

Audit all gate-87 receipts that still use `Compiles` or a host-side Rust pin because the exact expected carrier cannot yet be authored as `.dag` data.

Acceptance:

- Each placeholder names the missing carrier or runner capability.
- Each placeholder names the owning lane that can unblock it.
- No placeholder is treated as a silent exception to Band-C; it is either a temporary receipt or a non-gate-87 residual.
- The result updates the existing pattern/closure-audit docs rather than creating a new independent inventory.

Verification:

```bash
rg -n "Compiles|dissolve|dissolution|placeholder|Rust pin|blocked" \
  docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md \
  docs/briefs/r3-gate-87-lens-cementing-closure-audit.md \
  src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_*.dag \
  src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs
```

### G87-D4 — Runner and SG-0 Ratchet Receipt

Verify that the executable tests enforce the gate-87 inventory rather than relying on prose.

Acceptance:

- `t_pb_b_1_dag_runner_test` executes the gate-87 suites through the shared runner table.
- `r3_gate_87_lens_cementing_regen_receipts_test` rejects registry / receipt drift.
- `sg0_census_test.rs` comments continue to point workers at the single cementing inventory and forbid parallel hand lists.
- Any failing or stale ratchet is fixed in the same PR.

Verification:

```bash
cargo test -p v3-compiler r3_gate_87
cargo test -p v3-compiler sg0_census
```

### G87-D5 — Band-C / #84 Handoff Classification

Refresh the post-#87 handoff table for remaining hand-Rust cementing-looking tests so #84 workers do not consume gate-87 registry receipts incorrectly.

Acceptance:

- Every `src/v3/compiler/tests/integration/cementing/*.rs` row that remains in `EXPECTED_HAND_AUTHORED_TEST` is classified as gate-87 residual, Band-C bulk-port candidate, host-wrapper retirement, or T-LAS demonstration scope.
- The classification names the owning lane and the expected SG-0 census delta.
- The table stays in `docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md`; no duplicate inventory is introduced.

Verification:

```bash
rg -n "src/v3/compiler/tests/integration/cementing/|r3_gate_87_lens_cementing_regen_receipts_test|wiring_scanner_test" \
  src/v3/compiler/tests/integration/sg0_census_test.rs \
  docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md
```

## Dispatch Order

D1 and D4 are the fail-closed invariant checks and can run first. D2 and D3 can run in parallel once D1 confirms the current registry surface. D5 is the handoff slice for #84 and should consume D3's placeholder classifications where they overlap.

Completion of these children means gate #87 has a concrete, reviewable discipline package: the registry corpus stays complete, future COMPLETE flips have same-PR receipt requirements, placeholders have named dissolution paths, executable ratchets guard drift, and broader Band-C work is handed to #84 without duplicating authority.

