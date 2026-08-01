# Sole modeled publisher — staged migration design

Status: DRAFT for operator review (slice 1 landed as model + shadow only, 2026-08-01).

Origin: priority-assessment directive section 8 — publication safety outranks reveal UX. Repeated unmodeled auto-push incidents (three specimens on 2026-08-01) pushed unverified session work to the public remote because `git.Core` had no modeled `Push`/`Commit` write surface and publication was raw argv nobody's model saw.

## Evidence specimens (2026-08-01)

Recorded motivation receipts for the sole-publisher lane; no action items beyond citation.

### Unmodeled session pushes (opening specimens)

Three dashboard worker auto-push incidents the same day: session branches reached the public GitHub remote with no modeled `Push`/`Commit` in `extdeps.git` and no `PublicationAuthority` gate on **who** may write.

### Publication roster races (placement-gate landing, #7560 / repair #7580)

While the placement gate was rolling out, merge refs raced the roster:

1. **Missing rows** — branches cut before the gate landed merged to `main` carrying post-cutover `Added` paths with no `PublicFilePublishGrant` row; every subsequent PR merge ref refused `publication_placement_gate_passes` until repair #7580 stamped eleven paths.
2. **Duplicated rows** — parallel lane fixes merged without textual conflict markers but duplicated grant rows in `gunbc.publication_grant` (same path twice, valid syntax, invalid policy).
3. **Evaluate-then-merge TOCTOU** — the gate observes cutover→HEAD at merge-ref evaluation time; concurrent merges can change which paths are `Added` between evaluation and landing. An **admitted projection** model closes this by binding publication to a receipt over the exact source head and path set being published, not a one-shot gate run that later merges assume still hold.

### CI auto-heal unmodeled push (purest specimen yet)

During the roster races, CI auto-heal machinery **committed and pushed an unresolved merge** to worker branch `eager-seal-327` (commit `218dea6`): literal `<<<<<<<` / `=======` / `>>>>>>>` conflict markers remained in `dag/gunbc/publication_grant.dag`, producing parse errors downstream. An automated actor with push capability published syntactically invalid content nobody reviewed.

Under the sole-publisher architecture this incident class is **unwritable twice over**: the heal bot would hold no `PublicationAuthority` (refused `NotTheSolePublisher` before any placement check), and an admitted `PublishGuardLocalAccepted` projection could never contain conflict markers — invalid syntax is not an admissible public write.

## Problem statement

Today every dashboard worker and the autocommit machinery push session branches directly to the public GitHub remote. Hooks and CI placement gates are fast diagnostics and merge protection, but they do not constrain **who** may write to public storage. Confidentiality requires that ordinary sessions cannot hold public-write capability at all.

Target architecture (operator sketch): a separate bucket/server is source of truth; the public GitHub repo is a **downstream mirror**. Under that shape, bypassing the wall stops being possible because sessions never touch the public remote.

## Slice 1 boundary (this PR)

**Hard rule:** no live push path, credential custody, or remote mutation. The fleet continues pushing session branches to the public repo.

Delivered:

| artifact | role |
|---|---|
| `extdeps.git.publication_transport` | Modeled `PushRefUpdates` and `RecordCommit` interface shapes; argv is one transport handler |
| `tools.publication_publisher` | `PublicationAuthority` — sole publisher + admitted projection via `std.authorization_profile.PublicationAdmissionRequest`; typed refusals |
| `tools.publication_push_shadow` | Read-only stopped-line-audit replay; reports, does not green or gate |
| witness tests | Green-by-execution with discriminating REDs |

## WHO × WHAT composition

- **WHAT (landed P-B):** `tools.publication_placement_gate` decides which paths may inhabit the universal audience projection.
- **WHO (this slice):** `tools.publication_publisher` decides which session may bind `git.PublicationTransport` write operations against an admitted projection.

Ordinary sessions are refused `NotTheSolePublisher` before any placement outcome is consulted. The sole publisher without `PublishGuardLocalAccepted` is refused `ProjectionNotAdmitted`.

## PublicationAdmissionRequest consumer contract

Fields the sole-publisher consumer needs from proud-wolf-486's carrier (coordinate, do not fork):

- `subject` — `ModeledSessionId` / principal attempting the write
- `source.head` — exact commit sha under judgment
- `source.pushed_refs` — `List<PrePushStdinRow>` from `extdeps.git.hooks`
- `resource.public_root` + `resource.cutover` — pinned public mount policy
- `audience` — `UniversalAudience` (`AudienceSet<GunbcPrincipal>`)
- `evidence` — baseline marker + file markers + added paths from the placement receipt
- `context` — publisher write stage marker (`PublisherWriteContext`)

## Staged migration (fleet stays working at every step)

### Stage 0 — model + shadow (this PR)

- Model write operations and publisher authority.
- Shadow observer replays observed pushes; ledger counts ordinary-session refusals.
- **Trigger to Stage 1:** shadow receipt shows stable refusal taxonomy on live specimens; proud-wolf admission profile landed.

### Stage 1 — staging remote / bucket writes

- Sessions push to a **private staging remote** (or bucket) only; public remote credentials removed from worker reach.
- Autocommit seam retargeted to staging; fleet keeps working on session branches.
- Publisher process (separate session/host) mirrors admitted refs to public.
- **Rollback:** restore worker push credentials to public remote; disable publisher mirror.

### Stage 2 — enforcement gate

- Bind `git.PublicationTransport` dispatch: refuse unless `PublicationAuthority` is present.
- Shadow witness becomes regression control; live gate refuses instead of only reporting.
- **Rollback:** feature flag returns to shadow-only; staging remote remains.

### Stage 3 — downstream mirror architecture

- Public GitHub repo is generated projection only; sessions never receive public-write credentials.
- Autocommit machinery dissolved into staging + publisher pipeline.
- **Rollback:** not instant — requires credential reprovision and mirror pause (operator migration).

## Autocommit seam fate

The dashboard auto-committer currently snapshots multi-file edits mid-task and pushes session branches to the public remote. Stage 1 retargets it to staging only. Stage 3 replaces "push" with "submit to staging + request mirror" where the sole publisher is the only actor that can advance content to the public remote.

## Open coordination

- **proud-wolf-486 (#7578):** `std.authorization_profile.PublicationAdmissionRequest` landed; provisional `tools.publication_admission` bridge dissolved.
- **P-C composed-graph wall:** independent; does not block Stage 1 staging remote work.
