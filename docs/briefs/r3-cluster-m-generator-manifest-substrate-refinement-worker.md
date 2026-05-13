% R3 Cluster M — generator-manifest substrate refinement worker brief (AMENDED 2026-05-13)

**Status**: AMENDED — substrate-shape-only scope per Director re-ratification msg_606e0e50 (after worker warm-wren-479 STOP-AND-PING msg_1284363e surfaced three canvas-level shape concerns; original ratification msg_05837745 superseded on Q3 + Q-RegenCapability + Q-FixtureMapping).

**Owner parent**: warm-wolf-698 (R3 Substrate Mgr).
**Authoring date**: 2026-05-13.
**Amendment date**: 2026-05-13 (same day, post-STOP).
**Lane**: T-Tests-As-Data-Completeness / Cluster M Phase 3 substrate prereq for Gap 5 close (substrate-shape-only this PR; runtime + fixture-mapping split to follow-ups).
**Gates**: §1.8 row #86 `program_generator_carrier_landed` PASSING-evidence transitions in-place via shape-change-only; §1.8 row #84 `bulk_port_99_test_consumer_landed` positive-authority **substrate-readiness only** this PR — Gap 5 actual close fires when runtime PR lands (Evaluator-Mgr-owned follow-up).

---

## §0. Scope — substrate-shape-only (split from runtime + fixture-mapping per re-ratification)

Per Director msg_606e0e50 §"Q-RegenCapability RATIFIED: (β) SPLIT" + §"Q-FixtureMapping RATIFIED: deferred":

**IN-SCOPE this PR (Substrate-Mgr-owned)**:
- Substrate-shape change: `verification.dag` `GeneratedFromDag` refinement + new `GeneratedManifestEntry` type + `ContentHash` field
- 4-site `test_runner.rs` migration: lockstep field-rename only (minimal-shape; no new evaluator runtime capability)
- #86 PASSING transitions in-place via shape-change (existing predicate sites carry minimal-shape `manifest_entries` with trivial `dag_source` + trivial `source_hash`; gate satisfies on shape, not on byte-equality firing)
- Bootstrap regen via standard `regen.dag` flow (NOT hand-edit)

**OUT-OF-SCOPE this PR (split to follow-ups)**:
- 3-way byte-equality runtime assertion → **follow-up Evaluator-Mgr-owned PR** (blocked on R3 Evaluator Mgr lane re-spawn per operator escalation msg_acf78d37)
- Directory-walk orphan-output detection → same follow-up (also requires runtime capability extension)
- Per-file FixtureMapping enumeration (which DeclarationRef per parse_generated.rs, what ContentHash literal per surviving file) → **follow-up Verification-Mgr-owned integration slice** per still-moth-538 msg_6c50e646 "narrow follow-up integration/sweep" framing

**Gap 5 close impact** (Director-confirmed): gate #84 positive-authority predicate close STILL waits on the follow-up runtime PR, not this PR. This PR lands substrate-shape readiness; Gap 5 actual close fires when runtime lands. PR #3013 §Gap-5 narration unchanged.

Per `feedback_bundle_workstreams_per_pr` reading: bundling is the default WITHIN a lane; cross-lane work splits. Substrate-shape change (Substrate lane) + new evaluator runtime capability (Evaluator lane) + per-file fixture enumeration (Verification lane) are three lane-owned workstreams — splitting respects the discipline.

## §1. Substrate-shape refinement (Q1 §2.A ratified at msg_05837745; AMENDED Q3 per msg_606e0e50)

In `src/v3/std/verification.dag`:

### §1.1 New type `GeneratedManifestEntry`

```dag
type GeneratedManifestEntry {
  output_path: Path
  dag_source:  DeclarationRef           // TestClaim source authority (Q2 ratified, unchanged)
  source_hash: ContentHash              // AMENDED per Q3-(b): ContentHash, NOT SnapshotRef
}
```

**Why ContentHash, not SnapshotRef** (Director ratification msg_606e0e50): worker grep at `test_runner.rs:4742` verified `SnapshotRef` is a **sentinel string** (`"pipeline_stage_snapshots"`) driving registry-keyed-snapshot-lookup — NOT a byte-equality hash reference. Original Q3 framing overloaded the existing convention. `ContentHash` (per `core/infra::hash::ContentHash`, CLAUDE.md hash-unification) is the canonical hash type and names the actual fact per `feedback_naming_is_aliasing`. Aligns with `feedback_state_space_vs_behavioral_invariants`: type-system encodes the actual fact vs opaque-string-coupling to a registry.

Cardinality invariants (unchanged from msg_05837745):
- One manifest entry → exactly one source declaration (1:1).
- `dag_source` is DeclarationRef (per Q2 ratified).

### §1.2 `GeneratedFromDag` variant refinement

Replace `generated_paths: List<Path>` at `verification.dag:437-440` with `manifest_entries: List<GeneratedManifestEntry>`:

```dag
| GeneratedFromDag {
    authority:        DeclarationRef                       // unchanged
    manifest_entries: List<GeneratedManifestEntry>         // REPLACES generated_paths
  }
```

Variant doc-comment update:
- Reference the substrate-shape readiness for bidirectional integrity contract (the contract itself fires when follow-up runtime lands, not this PR).
- Document that this PR is shape-only; runtime byte-equality + orphan detection in Evaluator-Mgr-owned follow-up PR.
- Q4 scan-root remains future-refinement flag per original ratification.

## §2. Runner consumer migration (minimal-shape lockstep rename)

In `src/v3/compiler/src/test_runner.rs`:

### §2.1 `eval_generated_from_dag_shape` migration — field-rename only

Per Q-RegenCapability (β) SPLIT: this PR does **NOT** add runtime regen-from-DeclarationRef capability, 3-way byte-equality assertion, or directory-walk. Those land in the follow-up Evaluator-Mgr-owned PR.

This PR's evaluator-side change is **purely a field-rename + type-destructure update**. The existing one-direction set-membership check + outside-paths hand-count logic is preserved, just reading `manifest_entries[i].output_path` instead of `generated_paths[i]`:

1. Payload destructure: `[authority, FieldValue::List(generated_paths)]` → `[authority, FieldValue::List(manifest_entries)]`
2. Per-entry iteration reads `entry.output_path` for the existing set-membership check; `entry.dag_source` and `entry.source_hash` are present but unused this PR (carried forward for follow-up runtime PR consumption).
3. Error messages update to reference `(DeclarationRef, List<GeneratedManifestEntry>)`.
4. The outside-paths hand-count check (currently at `:5051`) preserves existing behavior — comparing on-disk paths to the manifest_entries' output_path set.

### §2.2 Existing `generated_paths` call-site migrations (4 sites)

Confirmed 4 sites in `test_runner.rs`; lockstep field-rename:

- `test_runner.rs:5018` — payload destructure update (see §2.1.1).
- `test_runner.rs:5020` — error string update.
- `test_runner.rs:5028` — loop over `manifest_entries`, destructure `output_path` for the existing logic.
- `test_runner.rs:5033` — type-check error message update.
- `test_runner.rs:5040` — set-membership error message: minor wording update (still on `output_path`).
- `test_runner.rs:5051` — outside-paths hand-count: preserves behavior; reads `entry.output_path` set.

If grep audit at implementation time surfaces >4 call-sites, **STOP-AND-PING** per §5 trigger (ii) below.

### §2.3 Bootstrap regen

Per CLAUDE.md `all_tools() elimination` discipline + Cost-of-Change: regenerate via `cargo run -p gunbc-codegen --bin gunbc-testgen`. `src/v3/compiler/src/bootstrap_generated.rs:43453` + `bootstrap_generated_without_parse_surface.rs:42945` `label: "GeneratedFromDag"` reflection auto-regenerates to the new field shape. NOT hand-edit.

## §3. Q4 scan-root (UNCHANGED + still deferred to follow-up runtime)

Q4 directory-walk-in-existing-evaluator ratification still stands per msg_05837745, BUT the directory-walk itself moves to the follow-up Evaluator-Mgr-owned runtime PR (orphan-output detection is part of the 3-way assertion runtime capability split).

This PR documents in the variant doc-comment that the well-known `tests/` scan-root will be consumed by the follow-up runtime PR. The `scan_root: Path` field future-refinement flag stays as a Q4-amendment trigger for the follow-up PR, not this one.

## §4. PASSING-evidence preservation (shape-change-only)

Director-mandated per msg_606e0e50 §"Revised worker scope": existing #86 `program_generator_carrier_landed` PASSING evidence transitions in-place via the shape-change. Worker must:

1. Identify existing predicate sites carrying `GeneratedFromDag { authority, generated_paths: [...] }` payloads.
2. Migrate each site to `manifest_entries: [GeneratedManifestEntry { output_path: <prior path>, dag_source: <minimal-shape stub>, source_hash: <minimal-shape stub> }]` shape.
3. Minimal-shape stub for `dag_source` + `source_hash` per Director: these fields are SHAPE-CARRIED-FORWARD this PR; the follow-up integration slice (Verification-Mgr-owned) enumerates real values. Worker MAY use empty / sentinel / TBD values that the gate's shape check accepts; the runtime byte-equality assertion that would consume real values doesn't exist this PR.
4. Run `cargo test --release` and verify existing `program_generator_carrier_landed` predicate is green under the refined shape. STOP-AND-PING per §5 (i) if PASSING regresses.

## §5. STOP-AND-PING triggers (revised per Director msg_606e0e50)

- **(i)** Shape-change-only causes #86 PASSING regression — substrate-fact-introduction surfaces something unaccounted for in the field-rename. Indicates the canvas underspecified migration paths even at shape-only level.
- **(ii)** 4-site `test_runner.rs` migration surfaces >4 call-sites — scope-broadening risk; Director's pre-dispatch grep count audit was wrong.
- **(iii)** `ContentHash` existing type doesn't compose cleanly with substrate types (e.g., `verification.dag` cannot import/reference `ContentHash`; substrate-side ContentHash declaration unclear).
- **(iv) (preserved from original brief)** Q4 scan-root ambiguity during follow-up runtime PR — Q4-amendment trigger, not this PR.

Removed triggers (no longer apply per split):
- Original §5 trigger #4 (Q3 SnapshotRef doesn't fit) — RESOLVED via Q3-amend to ContentHash.
- Original §5 trigger #2 (3-way assertion finds snapshot drift) — N/A this PR (runtime moved to follow-up).

## §6. Closure receipt

- §1.8 row #86 `program_generator_carrier_landed` continues PASSING under refined shape; no PASSING regression.
- §1.8 row #84 substrate-shape-readiness landed (positive-authority carrier now has structural form; runtime evaluation deferred to follow-up).
- `dashboard-ops reviews` mergeable=CLEAN, ≥2 distinct approving providers, no REQUEST_CHANGES per standing merge policy.
- Post-merge §1.8 ledger-receipt sync per `feedback_post_merge_ledger_receipt_sync` — Substrate Mgr (warm-wolf-698) updates #84 substrate-shape-readiness status + adds explicit gating note that Gap 5 close requires Evaluator-Mgr-owned follow-up runtime PR.

## §7. Follow-up workstreams (split per re-ratification)

These do NOT block this PR; tracked for sequencing visibility.

### §7.1 Evaluator-Mgr-owned follow-up: runtime regen + 3-way byte-equality assertion

- Scope: extend `test_runner.rs` with general regen-from-DeclarationRef capability; implement 3-way assertion (`regen_bytes_hash == manifest_entry.source_hash == output_path_disk_bytes_hash`); implement directory-walk orphan-output detection.
- Blocker: R3 Evaluator Mgr lane re-spawn (operator escalation msg_acf78d37 pending).
- Authoring: separate worker brief authored once Evaluator Mgr lane re-spawns; consumes this PR's substrate-shape readiness.

### §7.2 Verification-Mgr-owned follow-up: per-file FixtureMapping enumeration

- Scope: enumerate the DeclarationRef↔file mapping for existing predicate sites + populate real `source_hash: ContentHash` values per surviving file.
- Dependency: §7.1 runtime capability landed (so byte-equality assertions actually fire).
- Owner: still-moth-538 / Verification Mgr "narrow follow-up integration/sweep" per msg_6c50e646.

## §8. References

- Director ratification (original): msg_05837745 (§2.A + Q2 DeclarationRef + Q4 directory-walk + Q5 per-class same-shape preserved — all UNCHANGED).
- Director re-ratification (amendment): msg_606e0e50 (Q3-amend ContentHash + Q-RegenCapability split + Q-FixtureMapping deferred).
- Worker STOP-AND-PING that surfaced amendments: warm-wren-479 msg_1284363e.
- Canvas authority: [`docs/design-cluster-m-generator-manifest-substrate-canvas-2026-05-13.md`](../design-cluster-m-generator-manifest-substrate-canvas-2026-05-13.md) (commit 7c762c500).
- Gap 5 close-criterion source: [`docs/r3-actual-close-plan.md`](../r3-actual-close-plan.md) §Gap-5 lines 204-223 (PR #3013).
- Existing substrate: `src/v3/std/verification.dag:437` (`GeneratedFromDag`).
- Existing runner consumer: `src/v3/compiler/src/test_runner.rs:5015-5060` (`eval_generated_from_dag_shape`).
- ContentHash carrier: `core/infra::hash::ContentHash` per CLAUDE.md hash-unification (canonical hash type).
- INVARIANTS §P1 (carrier introduction) + §P2 (Single Authority) + §P5 (Progress is Dissolution).
- §1.8 rows: #84 (substrate-shape-readiness this PR; runtime close follow-up), #86 (PASSING in-place transition).

---

**End of brief (AMENDED).**
