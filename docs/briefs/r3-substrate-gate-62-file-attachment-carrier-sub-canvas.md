---
status: Mgr sub-canvas (carrier-internals substrate-shape question; surfaced per recursive feedback_substrate_shape_belongs_in_mgr_canvas after PR #2820 Candidate (b) ratification 2026-05-13)
authority parent: R3 Substrate Manager (warm-wolf-698)
authoring date: 2026-05-13
gate: §1.8 ledger row #62 `substrate_gap_file_ingestion_closed`
parent canvas: PR #2820 / `docs/briefs/r3-substrate-gate-62-file-ingestion-canvas.md` — Candidate (b) RATIFIED
sibling-carrier precedent: gate #55 `WorkflowObservationAnchor` (src/v3/std/timing_lens.dag:98) + gate #53 WorkflowSecret (PR #2160 / dsl/extdeps/github/actions.dag:114)
---

# Gate #62 — FileAttachment carrier-shape sub-canvas

## §0. Status

Macro shape ratified: workflow-substrate FileAttachment carrier extending #53 per parent canvas PR #2820 + Director disposition relayed via PM msg_52c4a707. This sub-canvas surfaces **carrier internals** for Director-or-Mgr-tier ratification before worker brief authoring proceeds.

PR #2819 (Candidate A) stands closed/held per ratification; audit-trail (msg_e85224dc) preserved as diagnostic input.

## §1. Authority + anti-patterns (verbatim per Director disposition)

### Authority chain at HEAD

1. §4.3 line 505: "Closes via: T-Workflow-As-Data file-ingestion substrate (`workflow_substrate_carriers_landed` extended to file-attachment)"
2. §1.8 row #53 sibling-carrier precedent (WorkflowSecret + CronExpression β-ratified via PR #2160)
3. §1.8 row #55 (`shared_external_attachment_pattern_documented`) **already CONSUMER_LANDED** — `WorkflowObservationAnchor` + `TimingObservationEntry` typed in `src/v3/std/timing_lens.dag:98-116`; pattern + 6 invariants documented in `docs/design-timing-lens.md` §2; branded provenance nominals in `dsl/std/types.dag`; bootstrap ratchet `timing_lens_substrate_carrier_test.rs`. **Load-bearing**: this is the precedent FileAttachment must align with.

### Director-enumerated anti-patterns (downstream worker review must enforce)

1. **No compile-time `read_utf8_file`-equivalent extern-func substrate additions** — Candidate A drift, rejected
2. **FileAttachment must align with #53 + #55 sibling-carrier pattern** — no novel shape
3. **No bridge variants** — legacy `include_str!` paths preserved alongside FileAttachment violates §P5 atomic-migration

## §2. Sibling-carrier pattern (#55 precedent, verbatim)

```dag
type WorkflowObservationAnchor {
  subject_node:    NodeId
  artifact_digest: ContentHash
  producer_id:     WorkflowProducerId      // branded nominal
  observer_id:     WorkflowObserverId      // branded nominal
  prover_id:       WorkflowProverId        // branded nominal
  attached_at_ns:  Nanoseconds
  workflow_run_id: WorkflowRunId           // branded nominal
}
```

Shape rules visible in this precedent:
- **Workflow-context coupling** via `subject_node: NodeId` + `workflow_run_id: WorkflowRunId`
- **Artifact identity** via `artifact_digest: ContentHash` (not bare path; content-addressed)
- **Branded provenance** for distinct provenance roles (three role-IDs, not three `String`s)
- **Typed time** (`Nanoseconds`, not `Int`)
- **Boundary Discipline**: anchor + payload separated in `TimingObservationEntry` (one product fact, no join drift)

This is the pattern FileAttachment must mirror.

## §3. Carrier-shape question — three candidate shapes

### Candidate B-1 — content-addressed minimal (strictest #55 mirror)

```dag
type FileAttachment {
  subject_node:     NodeId
  content_digest:   ContentHash
  workflow_run_id:  WorkflowRunId
  attached_at_ns:   Nanoseconds
}
```

Rationale: ContentHash IS the file (content-addressed); path is derivable from a separate lookup (workflow-substrate stores blobs by digest). Maximally aligned with #55's content-addressed posture.

Pros:
- Strictest mirror of #55; no novel fields
- Path/encoding/mime become **derivable** from digest + workflow blob store, not stored on the carrier
- Cost-of-change-1 satisfied (one digest reference per ingest site)

Cons:
- Requires a workflow blob-store substrate to resolve digest→bytes; that substrate may not exist at HEAD
- Doesn't give consumers a self-contained file handle

### Candidate B-2 — path-keyed with provenance (richer)

```dag
type FileAttachment {
  subject_node:     NodeId
  path:             WorkflowAssetPath       // NEW branded nominal (cross-provider authority?)
  content_digest:   ContentHash             // for change-detection / pinning
  encoding:         AttachmentEncoding      // UTF-8 | Binary | ...
  workflow_run_id:  WorkflowRunId
  attached_at_ns:   Nanoseconds
}
```

Rationale: path AND digest carried; encoding declared up-front so consumers don't decode-by-guess.

Pros:
- Self-contained: any consumer can resolve to bytes without external blob store
- Encoding-declared avoids consumer-side parse-shape drift
- Brand-typed path (`WorkflowAssetPath`, new nominal) keeps boundary discipline

Cons:
- Introduces `AttachmentEncoding` sum-type (Practice 4 application — see §5)
- Introduces `WorkflowAssetPath` nominal (cross-provider authority grep required per `feedback_self_hosting_md_authority_audit_before_naming`)
- Slightly larger surface than #55 baseline; question whether encoding-declaration is load-bearing

### Candidate B-3 — anchor+entry-pair (closest structural mirror of #55+`TimingObservationEntry`)

```dag
type FileAttachmentAnchor {
  subject_node:     NodeId
  workflow_run_id:  WorkflowRunId
  attached_at_ns:   Nanoseconds
}

type FileAttachment {
  anchor:          FileAttachmentAnchor
  path:            WorkflowAssetPath
  content_digest:  ContentHash
  encoding:        AttachmentEncoding
}
```

Rationale: structurally mirrors `WorkflowObservationAnchor` + `TimingObservationEntry` split — anchor (provenance/identity) and payload (file fact) are distinct product facts, joined explicitly.

Pros:
- **Structurally isomorphic to #55** — exact pattern-match for the locked sibling-carrier shape
- Boundary Discipline: anchor vs payload separation
- Future-proof: if a non-file attachment (e.g. inline-string attachment) needs the same anchor pattern, the anchor type composes

Cons:
- Two types vs one; slightly more authoring overhead per ingest site
- Per-ingest-site cost-of-change is still 1 (build one `FileAttachment { anchor: …, … }` value)

## §4. Lazy-vs-eager evaluation

Orthogonal axis to carrier shape. Three sub-options:

- (i) **Eager**: `FileAttachment` value is constructed with `content_digest` pre-computed; bytes are workflow-substrate-resolved by digest at consumer-time
- (ii) **Lazy**: `content_digest` is a deferred-thunk evaluated on first read
- (iii) **Eager with cache**: eager construction, consumer-side memoized

Recommendation: **(i) eager**, consistent with #55 `TimingObservationEntry` (no lazy thunks in sibling carriers). Lazy semantics not justified by current consumer evidence.

## §5. Practice 4 (coproduct dissolution) check — `AttachmentEncoding`

Candidates B-2 and B-3 introduce `AttachmentEncoding` sum-type. Per `feedback_canvas_finding_taxonomy`:

- 🟢 **GREEN if** encoding distinctions drive type-level behavior differences at consumer sites (e.g., UTF-8 consumers vs Binary consumers have different APIs); named-dissolution-trigger applies if encoding can be inferred from `path` extension or `content_digest` magic-bytes
- 🟡 **YELLOW if** encoding is a tag without behavioral payload — likely vacuous, should be derivable
- 🔴 **RED** would only apply if encoding partitions consumers in a way that's already modeled elsewhere (e.g., `dsl/std/encoding.dag` already has carriers)

**Existing precedent check** (per `feedback_self_hosting_md_authority_audit_before_naming`): `dsl/std/encoding.dag` exists; the canvas worker brief MUST grep that file before naming `AttachmentEncoding` — if the encoding partition is already modeled there, FileAttachment field references the existing nominal rather than introducing a new sum-type.

## §6. Workflow-context coupling — how does a Workflow node reference an attached file?

Three sub-questions:

- (a) **Carrier-side**: `FileAttachment.subject_node: NodeId` (per #55 mirror) — the node it's attached to is on the attachment record
- (b) **Workflow-side**: does `Workflow` / `Job` / `Step` carry a `List<FileAttachment>`? Or is the attachment table indexed externally?
- (c) **Consumer-evidence**: which ingest call sites need this (forward-looking; gate is "model for future use", not retire-existing per Director audit)

PR #2820 §2 cites `rg include_str! dsl/` → no matches at HEAD — gate is forward-looking. So consumer-evidence brief is **forward-looking** (existence-proof use cases) not **migration-shaped** (no legacy sites to migrate).

## §7. Consumer evidence — forward-looking existence proof

Director's anti-pattern #3 ("no bridge variants") + Class-1 5-criteria-pass for §1.8 row #62 imply: at least 1 concrete `.dag` program that ingests an external file via FileAttachment must land as existence-proof.

Sub-canvas defers to worker brief: enumerate ≥1 candidate use case (e.g., timing observation set ingest as workflow attachment; cf. timing_lens.dag's existing `TimingObservationSet` shape but where the observation data itself comes from a file).

## §8. Open questions for ratification

1. **Carrier shape**: B-1 (content-addressed minimal) / B-2 (path-keyed with provenance) / B-3 (anchor+entry-pair mirror of #55)?
2. **`AttachmentEncoding` field present** (B-2/B-3) **or absent** (B-1) — derivable-from-path or from-digest, OR carried explicitly?
3. **`WorkflowAssetPath` branded nominal**: introduce new nominal or strict-mirror an existing `dsl/std/types.dag` brand? (Worker brief grep required.)
4. **Workflow-side coupling**: does `Job` / `Step` carry `attachments: List<FileAttachment>`, or external indexing?
5. **Eager-vs-lazy**: confirm (i) eager per #55 precedent.
6. **Practice 4 disposition for `AttachmentEncoding`** if introduced: 🟢/🟡/🔴 per §5.

## §9. Cost-of-change accounting

| Candidate | Files edited to add a new file-ingest site |
|---|---|
| B-1 (content-addressed minimal) | 1 (one `FileAttachment` value referencing digest) |
| B-2 (path-keyed) | 1 (one `FileAttachment` value with path+digest) |
| B-3 (anchor-entry pair) | 1 (one `FileAttachment { anchor: …, … }` value) |

All three satisfy cost-of-change-1. Discriminator is **#55 structural-mirror fidelity** + **encoding-declaration load-bearing-ness**, not cost.

## §10. Ratification ask

Director or Substrate-Mgr-tier ratification on §8 questions 1-6. On ratification, worker brief authors:
- Type definition + sibling-carrier alignment receipt
- Brand-nominal grep audits (encoding + path)
- Practice 4 disposition for any new sum-types
- Workflow-side coupling site
- Existence-proof use case
- Bootstrap ratchet test (mirror of `timing_lens_substrate_carrier_test.rs`)

## §11. Reference

- Parent canvas: PR #2820 / `docs/briefs/r3-substrate-gate-62-file-ingestion-canvas.md`
- Director disposition: PM msg_52c4a707
- #55 precedent type: `src/v3/std/timing_lens.dag:98-116`
- #55 design doc: `docs/design-timing-lens.md` §2 (6 invariants)
- #55 bootstrap ratchet: `timing_lens_substrate_carrier_test.rs`
- #53 sibling: `dsl/extdeps/github/actions.dag:114` (WorkflowSecret)
- §4.3 framing: `docs/r3-program-plan.md:499-506`
- bright-otter-731 audit: msg_e85224dc-7c7e-495d-911f-8bace75a7bcd

---

**Authored by**: warm-wolf-698 (R3 Substrate Mgr)
**Date**: 2026-05-13
**Source authority**: PR #2820 Director-ratified macro shape + #55 sibling-carrier precedent
