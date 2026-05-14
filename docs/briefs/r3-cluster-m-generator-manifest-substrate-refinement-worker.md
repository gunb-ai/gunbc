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
- Substrate-shape change: `verification.dag` `GeneratedFromDag` refinement + new `GeneratedManifestEntry = PendingFact | ResolvedFact` sum-variant (RE-RATIFIED msg_3b99a90f; `ResolvedFact.source_hash: NonEmptyStr` per Q3-RE-AMEND β msg_8423d468)
- 4-site `test_runner.rs` migration: lockstep field-rename only (minimal-shape; no new evaluator runtime capability)
- #86 PASSING transitions in-place via shape-change (existing predicate sites carry `PendingFact { output_path }` only — no fabricated `dag_source` / `source_hash` per INVARIANTS P3 / C-9; `ResolvedFact` materialisation defers to the follow-up Evaluator-Mgr-owned runtime PR at its hash-derivation construction boundary; gate satisfies on shape, not on byte-equality firing)
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
// Q-FixtureMapping RE-RATIFIED per Director msg_3b99a90f: sum-variant Pending|Resolved
type GeneratedManifestEntry =
  | PendingFact  { output_path: Path }
  | ResolvedFact { output_path: Path, dag_source: DeclarationRef, source_hash: NonEmptyStr }
```

**Why sum-variant** (Director ratification msg_3b99a90f, third amend on this canvas):

Worker's third STOP-AND-PING (msg_e6829094) surfaced codex BLOCKING: prior "minimal-shape with trivial stubs" framing (msg_606e0e50 §Q-FixtureMapping deferral + my brief §4 "empty/sentinel/TBD values") **forces P3/C-9 violation** at fixture sites — required-field discipline admits no honest values pre-runtime. INVARIANTS.md:366-374 P3 load-bearing across C-1/C-2/C-3/C-5/C-9.

Sum-variant `PendingFact | ResolvedFact`:
- **P3 preserved**: PendingFact carries no fabricated values; ResolvedFact carries the full byte-fidelity contract once runtime materialization lands
- **Typed-state explicit** per `feedback_state_space_vs_behavioral_invariants`: illegal states (fabricated values where data isn't materialized) become unrepresentable
- **Coproduct preserved** per `feedback_coproduct_dissolution` 4-pattern audit: Pending vs Resolved IS the meaningful coordinate axis (existence of resolution data); won't dissolve to coordinates
- **`feedback_construction_over_ratchets`**: models the actual fact rather than bridging via stubs
- Substrate-shape forward-readiness PRESERVED: PR lands sum-variant; all fixture-sites migrate to `PendingFact { output_path }` (lockstep field-rename + sum-variant-wrap); follow-up Evaluator-Mgr-owned runtime PR adds PendingFact → ResolvedFact migration as runtime materializes `dag_source` + `source_hash` via hash-derivation

**Why bare `NonEmptyStr`** (Director re-ratification msg_8423d468, second amend on Q3):

The Q3-amend at msg_606e0e50 attempted `source_hash: ContentHash` but worker grep verified `dsl/std/types.dag:324` declares `ContentHash = NonEmptyStr where brand("ContentHash")` — branded-refinement type. `lower.rs:6094` fail-closes when scalar literal can't statically discharge where-refinement; bootstrap regen emits spurious diagnostics for branded NonEmptyStr from string literal. Zero existing .dag fixtures construct branded NonEmptyStr from literal — fresh substrate ground; would require lowerer narrowing-branch substrate-fact-introduction (path α).

Bare `NonEmptyStr` aligns with:
- `SnapshotRef = NonEmptyStr` precedent at verification.dag:31 + `FixedPointConverges.expected: SnapshotRef` at verification.dag:431 (existing branded-free precedent for the same shape-class).
- Q-RegenCapability β SPLIT semantics: `source_hash` has ZERO runtime consumer this PR — field is pure shape-stub carrying forward to follow-up Evaluator-Mgr-owned runtime PR. Brand provides nominal-disambiguation with no runtime enforcement during this shape-only window.

**Brand re-introduction at follow-up runtime PR consumption boundary** (Director-flagged): when Evaluator-Mgr-owned runtime PR lands, hash-derivation at runtime consumption site produces ContentHash (via canonical `core/infra::hash::ContentHash::from_str` or similar). Construction at runtime doesn't hit the literal-construction lowerer gap — different construction path. Brand semantic preserved at runtime use boundary; deferred from substrate-shape window.

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
3. **SUPERSEDED per msg_3b99a90f**: all fixture-sites migrate to `PendingFact { output_path: <path> }` — NO fabricated `dag_source` / `source_hash` values; sum-variant `GeneratedManifestEntry` removes the P3/C-9 violation. ResolvedFact materialization is follow-up runtime PR scope.
4. Run `cargo test --release` and verify existing `program_generator_carrier_landed` predicate is green under the refined shape. STOP-AND-PING per §5 (i) if PASSING regresses.

## §5. STOP-AND-PING triggers (revised per Director msg_606e0e50)

- **(i)** Shape-change-only causes #86 PASSING regression — substrate-fact-introduction surfaces something unaccounted for in the field-rename. Indicates the canvas underspecified migration paths even at shape-only level.
- **(ii)** 4-site `test_runner.rs` migration surfaces >4 call-sites — scope-broadening risk; Director's pre-dispatch grep count audit was wrong.
- **(iii) [RESOLVED via Q3-RE-AMEND]** ContentHash branded-NonEmptyStr literal-construction gap — superseded by bare NonEmptyStr per msg_8423d468.
- **(iv) NEW per Director msg_8423d468 — partially superseded by (vi)**: bare `NonEmptyStr` literal ALSO fails to discharge at fixture sites — N/A now: PendingFact carries no NonEmptyStr field per sum-variant ratification.
- **(v) Q4 scan-root ambiguity during follow-up runtime PR — N/A this PR.**
- **(vi) NEW per Director msg_3b99a90f**: cursor / codex re-review surfaces another invariant-conformance concern I haven't anticipated — STOP-AND-PING per 4-axis discipline (type-shape / semantic / constructability / invariant-conformance) maturity.

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
