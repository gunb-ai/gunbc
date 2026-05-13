---
status: dispatchable (worker brief; ratified shape per recursive Mgr-canvas chain + Director disposition relayed via PM msg_bc8c23f6 2026-05-13)
authority parent: R3 Substrate Manager (warm-wolf-698)
authoring date: 2026-05-13
gate: §1.8 ledger row #62 `substrate_gap_file_ingestion_closed`
parent canvases:
  - PR #2820 / `docs/briefs/r3-substrate-gate-62-file-ingestion-canvas.md` — macro shape (Candidate b RATIFIED 2026-05-13)
  - PR #2820 / `docs/briefs/r3-substrate-gate-62-file-attachment-carrier-sub-canvas.md` — carrier internals (Refined-B-1 RATIFIED 2026-05-13)
ratification anchor: PM msg_bc8c23f6 relaying Director msg_61e302c6 — full §8 Q1-Q6 dispositions + 7 anti-patterns + Director-grep-verified-at-HEAD audit
ledger refinement: PR #2821 commit 3fcd374e0 (DECLARED-with-ratified-shape-and-carrier-internals-ratified)
---

# Gate #62 — FileAttachment substrate-carrier worker brief

## §0. Status — DISPATCH-READY

All carrier-shape questions ratified. Worker authors carrier + ratchet + existence-proof use case. No further canvas required.

PR #2819 (Candidate A drift, compile-time `read_utf8_file`) is **CLOSED/HELD** as ratified anti-pattern; audit-trail preserved.

## §1. Ratified carrier (exact shape — Director verbatim)

```dag
type FileAttachment {
  subject_node:    NodeId
  content_digest:  ContentHash
  producer_id:     WorkflowProducerId
  workflow_run_id: WorkflowRunId
  attached_at_ns:  Nanoseconds
}
```

**5 fields, in this order, with these names**. Strict 5-of-7 subset of `WorkflowObservationAnchor` at `src/v3/std/timing_lens.dag:98-106` (dropping `observer_id` + `prover_id` as timing-specific roles).

**Placement**: `src/v3/std/` — same crate as `timing_lens.dag` sibling. Worker grep-verifies adjacency at authoring time.

## §2. Director-verified authority chain at HEAD (verbatim)

1. **#55 anchor 7 fields** at `src/v3/std/timing_lens.dag:98-106` (subject_node, artifact_digest, producer_id, observer_id, prover_id, attached_at_ns, workflow_run_id)
2. **All 5 needed branded nominals** at `dsl/std/types.dag:324-331`:
   - `ContentHash` :324
   - `WorkflowProducerId` :328
   - `WorkflowObserverId` :329 (unused by FileAttachment)
   - `WorkflowProverId` :330 (unused by FileAttachment)
   - `WorkflowRunId` :331
   - All follow `NonEmptyStr where brand("…")` pattern
3. **`dsl/std/encoding.dag`** exists with full `Encoding` lattice (`type Encoding` :23; 6-variant `ASCII | UTF8 | Latin1 | Text | Binary | Unknown` BoundedLattice)
4. **No `FileAttachment` / `AttachmentEncoding` / `WorkflowAssetPath` at HEAD** — names available

## §3. §8 Q1-Q6 dispositions (Director verbatim — load-bearing for PR framing)

| Q | Disposition |
|---|---|
| Q1 carrier shape | **Refined-B-1** — 5-field 5-of-7-subset-of-#55. B-2 path+digest REJECTED as parallel-rep; B-3 anchor+entry-pair REJECTED as over-engineered (no payload to split) |
| Q2 encoding field | **ABSENT default**. If consumer needs encoding, use `Encoding` from `dsl/std/encoding.dag`. **Never introduce `AttachmentEncoding`.** |
| Q3 `WorkflowAssetPath` | **DO NOT introduce**. No path field on carrier; digest is the identifier. |
| Q4 workflow-side coupling | **Defer to consumer evidence.** Per #55: `TimingObservationSet = List<TimingObservationEntry>` is OWN carrier, NOT embedded on Job/Step. Do NOT preemptively add `List<FileAttachment>` to Job/Step. |
| Q5 eager-vs-lazy | **EAGER** per #55 sibling. |
| Q6 `AttachmentEncoding` Practice-4 | 🔴 **RED** — DISSOLVE to existing `Encoding`. Practice-4 RED-zone (partitions consumers in way already modeled). |

## §4. 7 anti-patterns (downstream worker review MUST enforce)

1. Compile-time `read_utf8_file`-equivalent extern-func additions (Candidate A drift, rejected)
2. FileAttachment landed without #55/#53 sibling-carrier alignment
3. Bridge variants alongside FileAttachment (§P5 atomic-migration violation)
4. Any `AttachmentEncoding` or equivalent encoding sum-type duplicating `dsl/std/encoding.dag`
5. `path` field on FileAttachment (parallel-rep vs canonical digest)
6. `List<FileAttachment>` on Job/Step preemptively (consumer-evidence-required)
7. Carrier-pattern deviation from Refined-B-1 5-field structure (no novel field-name renames; `producer_id` stays `producer_id`, etc.)

PR body MUST cite this list verbatim and assert receipt-of-compliance per anti-pattern.

## §5. Scope

### Phase A — Carrier landing

1. Add `type FileAttachment { … }` to **`src/v3/std/`** in a new or existing file (worker decides per grep + sibling-carrier adjacency; `timing_lens.dag` is the structural neighbor)
2. Cite §1 verbatim in a header comment block referencing gate #62 + sub-canvas (PR #2820) + Director disposition (PM msg_bc8c23f6)
3. Re-validate all 5 branded nominals exist at `dsl/std/types.dag:324-331` at authoring time

### Phase B — Bootstrap ratchet test

Mirror `timing_lens_substrate_carrier_test.rs` shape — single test asserting:
- `FileAttachment` type declared with 5 fields (exact names + types per §1)
- All 5 field types resolve via cross-module reference (not parallel declarations)
- Test placement: `src/v3/compiler/tests/integration/cementing/file_attachment_substrate_carrier_test.rs` (or sibling location per existing pattern)

### Phase C — Existence-proof use case (forward-looking)

Per §1.8 row #62 criterion (".dag program ingests external file w/o `include_str!`"), one concrete `.dag` program must construct a `FileAttachment` value as existence proof. Worker enumerates the candidate use case:

- Suggested: timing-observation-set ingest (cf. existing `TimingObservationSet` in `timing_lens.dag`) where observation data originates from an external file
- Alternative: any `.dag` workflow that needs to attach an asset by digest

Worker proposes the use case in PR body; can be a minimal demo module (cf. `t_ci_workflow_as_data_demo.dag` shape).

### Phase D — §1.8 row #62 ledger update

After Phase A+B+C land and pass, update `docs/r3-program-plan.md` §1.8 row #62 (line 290) from **DECLARED-with-ratified-shape-and-carrier-internals-ratified** (per PR #2821) to **CONSUMER_LANDED** with cite to this PR.

## §6. STOP conditions

1. **Branded-nominal grep mismatch** at HEAD — if `dsl/std/types.dag:324-331` no longer contains the 5 nominals exactly as Director-verified (e.g., rename, removal), **STOP** and surface — substrate state has drifted.
2. **Sibling-carrier #55 drift** — if `WorkflowObservationAnchor` shape at `src/v3/std/timing_lens.dag:98-106` no longer matches the 7-field structure (e.g., renamed, reshaped), **STOP** — the 5-of-7-subset rationale needs re-anchoring.
3. **Carrier-name collision** — if `FileAttachment` / `AttachmentEncoding` / `WorkflowAssetPath` appear at HEAD via parallel landing, **STOP** — surface to Mgr (warm-wolf-698) for de-duplication.
4. **Consumer-evidence use case unbuildable** — if no candidate `.dag` site can construct a `FileAttachment` value at digest-resolution-time (e.g., no `ContentHash` source in `.dag` surface), **STOP** — sub-canvas-2 (blob-store substrate) is prerequisite, not optional. Surface to Mgr.
5. **Practice-4 RED violation surfaces** — if Phase A authoring tempts an `AttachmentEncoding` sum-type or path field, **STOP** — anti-patterns #4 + #5 fire; revise to use existing `Encoding` + drop path.

## §7. Verification

- `cargo test --workspace` green
- Hermetic test `file_attachment_substrate_carrier_test.rs` asserts:
  - Field count = 5
  - Field names + types match §1 exactly
  - Cross-module reference resolution for all 5 nominals
- Demo module (Phase C) compiles + constructs a `FileAttachment` value
- PR body cites:
  - Gate #62 closure (Phase D ledger update)
  - Director disposition (PM msg_bc8c23f6) verbatim Q1-Q6
  - 7 anti-patterns receipt-of-compliance
  - Sibling-carrier #55 5-of-7-subset alignment receipt
  - Branded-nominal grep audit at HEAD

## §8. Out of scope

- Workflow blob-store substrate (`content_digest → bytes` resolution) — **separate canvas; Wave-2 sub-canvas-2 trigger per Director "not blocking this ratification"**. If consumer evidence (Phase C) requires blob resolution unavailable at HEAD, surface STOP-4 instead of inventing inline.
- `List<FileAttachment>` on `Job` / `Step` (anti-pattern #6) — consumer-evidence-required, not preemptive
- Migration of any (hypothetical) legacy `include_str!` paths — Director-verified no `include_str!` in `dsl/` at HEAD; gate is forward-looking
- Encoding-tag introduction — anti-pattern #4

## §9. PR body framing template

```
Closes gate #62 substrate_gap_file_ingestion_closed.

Carrier landed exactly per Refined-B-1 ratification (PR #2820 sub-canvas;
Director disposition relayed via PM msg_bc8c23f6 2026-05-13):

[paste §1 type definition verbatim]

5-of-7-subset of #55 WorkflowObservationAnchor (src/v3/std/timing_lens.dag:98)
— drops observer_id + prover_id (timing-specific roles).

Q1-Q6 dispositions enforced (verbatim per Director):
[paste §3 table]

7 anti-patterns receipt-of-compliance:
[enumerate each + cite that the implementation does not violate it]

Existence proof: [Phase C demo module path] constructs a FileAttachment
value via digest-resolution at [site].

§1.8 row #62 updated: DECLARED-... → CONSUMER_LANDED.
```

## §10. Reference

- Parent canvases: PR #2820 (top-level + sub-canvas as two commits on same branch)
- Director disposition relay: PM msg_bc8c23f6 (relaying Director msg_61e302c6)
- Ledger refinement: PR #2821 commit 3fcd374e0
- Sibling-carrier #55: `src/v3/std/timing_lens.dag:98-106`
- Sibling-carrier #55 ratchet: `src/v3/compiler/tests/integration/cementing/timing_lens_substrate_carrier_test.rs`
- Branded nominals: `dsl/std/types.dag:324-331`
- Encoding lattice (do-not-duplicate authority): `dsl/std/encoding.dag:23`
- §4.3 framing: `docs/r3-program-plan.md:499-506`
- bright-otter-731 audit-trail: msg_e85224dc

---

**Authored by**: warm-wolf-698 (R3 Substrate Mgr)
**Date**: 2026-05-13
**Sub-canvas-2 trigger** (forward-looking, NOT blocking): workflow blob-store substrate canvas authoring to Wave-2 queue.
