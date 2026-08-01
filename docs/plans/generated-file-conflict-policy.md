# Generated-file conflict policy — lane charter (operator-ruled 2026-08-01)

Status: RULED. This document records the operator ruling of 2026-08-01 on how generated
files, hand-authored authorities, and keyed rosters integrate under concurrent branches,
and charters four lanes to land it. The ruling's mechanism principle: **the authority is
the existing `.dag` model of artifacts, storage, commit workflow, and Git state —
`.gitattributes` and Git config are downstream projections of it, never the policy.**

```text
existing .dag authorities
  → path/integration-policy derivation
  → .gitattributes projection
  → repository-local Git configuration projection
  → commit-writer admission
```

Git is a compatibility realization downstream of the native integration model
(dag-scm-design.md), not the vocabulary the model derives from.

## The problem, with receipts

Three distinct failure classes were being experienced as "conflicts on generated files":

1. **Redundant artifact conflicts.** DESIGN.md / ROADMAP.md / ci.yml changed 46/73/49
   times in 3 weeks against 834 commits; DESIGN.md is 132 lines with lines up to 14KB,
   so git's line-grain merge conflicts on nearly any concurrent pair. In every observed
   case the `.dag` authority merged cleanly or conflicted resolvably — the artifact
   conflict carried zero information beyond it (clever-ferret-451, 3× in one day on one
   PR). The artifact is derivable; a human resolving its bytes is redundant work (§2).
2. **Commit-writer fail-open.** Commit `218dea61248d` (auto-heal, eager-seal's #7573
   branch) committed and pushed three literal `<<<<<<< HEAD` … `>>>>>>> origin/main`
   blocks inside `dag/gunbc/publication_grant.dag`. The writing automation committed
   *through* an unresolved merge instead of refusing — the §5 absorbing fallback at the
   commit boundary.
3. **Clean merges on keyed rosters — the expensive case is not a conflict.** #7565 and
   #7580 appended overlapping row sets to `publication_grant.dag`; both merged cleanly
   (git saw two list appends), main carried 11 duplicate paths, the placement gate went
   red, batch-1 stalled, and the fleet's witness corpus stopped while multiple sessions
   diagnosed it as their own regression (calm-badger-682). For append-only rosters a
   clean merge is the dangerous case and a conflict is the safe one; a conflict-only
   policy fixes the visible half and leaves the expensive half.

A registration that can disagree with reality is a second authority (§3), and it did:
`generated_stage0_files` registered a projection whose file did not exist (#7559) and
five absent from main (#7515).

## The four-case policy

| Path semantics | Git integration treatment | Required result before commit |
| --- | --- | --- |
| **Deterministic generated projection** | Either textual side may be retained provisionally; no human resolves its bytes | Regenerate from the merged authorities and prove byte equality |
| **Hand-authored semantic authority** | Ordinary/native semantic integration; a conflict may carry information | All semantic choices resolved and index normalized |
| **Keyed set/map authority** | Never "take ours" and never rely on line merge | One value per key by construction; duplicate-key input refuses |
| **Roster naming generated outputs** | Delete the roster and derive membership from the producer | Produced set and observed output set agree exactly |

`publication_grant.dag`-shaped carriers are row 3, not row 1: hand-authored policy whose
key is `path`. A generated-artifact merge driver there would hide decisions.

## What the model must NOT be

- **No universal `FileKind` enum / path census.** `gunbc.generated_artifact.GeneratedArtifact`
  (repo generated-projection + commit-policy authority) and `v2.std.artifact.Artifact`/
  `ArtifactKind` (generic compiler artifact/provenance vocabulary) already exist with
  different scopes; `.dag` path⇄module facts belong to the derived `ModuleStorageBinding`
  authority. The conflict lane **joins** those facts; a third path census would be the
  dual representation this lane exists to remove.
- **No fused policy dimensions.** Linguist presentation, generated provenance, merge
  behavior, line-ending behavior, and diff presentation are orthogonal questions answered
  independently by the model. Generated ⇏ linguist-hidden.
- **Sparse projection over a total model.** The model may be total over tracked paths
  while `.gitattributes` stays sparse: ordinary files derive Git's default three-way
  behavior and the emitter writes only non-default attribute rows. No one-row-per-file
  census.

## Lane 1 — commit-writer admission (FIRST: the live safety hole)

One modeled admission predicate, consumed by **every** commit writer
(`HumanGitCommit | DashboardAutocommit | HealAutocommit | IntegrationCommit` — carrier
names indicative, not final). Two independent refusal arms:

1. **Unmerged-index refusal.** Every index entry must be `GitIndexStageNormal`
   (`extdeps.git.object_store` `GitIndexStage`); any `Base`/`Ours`/`Theirs` stage yields
   a typed, located refusal carrying the affected path and stage population. The Git
   model already carries the facts (`GitIndexStage`, `DiffUnmergedStatus`) — no stringly
   "conflicted tree" fact is minted.
2. **Resolved-to-marker-text refusal.** `git add` can collapse an unresolved file into a
   normal stage-0 blob with the conflict blocks still in the content. So the exact staged
   blobs about to be committed are inspected for the complete conflict-marker *grammar*
   (begin/separator/end structure, not one isolated token). Legitimate fixture content is
   exempted via typed test-artifact provenance, never a path allowlist. Not a
   repo-wide raw grep.

The generated `.githooks/pre-commit` hook is fast local feedback, **not enforcement**
(optional, bypassable — the repo's own hook notes already make the distinction). The
dashboard/heal writer that produced `218dea6` must call the same modeled predicate at its
actual write boundary; the repo lane lands the predicate + witnesses, and the infra
binding is reported for operator routing.

Acceptance: a witness reproducing `218dea6`'s exact shape (merge with unmerged entries;
and separately the staged-marker-text variant) refuses typed/located; positive control
(clean tree) admits; RED controls live.

## Lane 2 — path/integration-policy join, `.gitattributes` + Git config projections

`GitattributesArtifact` joins the registry with the meaning: *"the committed byte
projection through which Git receives the relevant subset of the modeled
repository-path policy."* Contents derive from existing authorities only:

```text
**/*.dag linguist-language=Rust        ← modeled as its own presentation-policy row
<each committed deterministic projection> merge=generated-artifact
```

The second population is `committed_generated_artifact_paths()` verbatim — adding or
deleting a generated artifact automatically adds/deletes its attribute row.
`.gitattributes` classifies **itself** through the same derivation; no handwritten
self-row. (`.gitignore` is the precedent inversion: it already consumes the registry.)

The merge-driver *definition* is repository-local Git state, not `.gitattributes`
content. It is modeled as desired state with: a generated config fragment or typed
reconciliation plan; an idempotent actuator installing the repo-local binding (driver +
`core.hooksPath`); observation/read-back proving the bindings present; typed refusal on
reconciliation or read-back failure (the membership-reconcile desired-state shape). A
documented manual `git config` command with no read-back is a declared-but-unrealized
configuration and does not satisfy the lane.

`merge=generated-artifact` with `driver=true` (take-ours) is a **transport
convenience, not the safety boundary**. The integration transaction is:

```text
integrate authored authorities without committing
  → prove no unmerged index entries          (lane 1 predicate)
  → regenerate affected committed projections
  → prove generated == committed/indexed bytes
  → run required admission checks
  → write commit
```

The commit writer consumes the regeneration receipt as a precondition — otherwise Git
can write a merge commit containing the provisional "ours" bytes and the drift gate
merely detects the bad transition after it exists.

**Documentation correction rides in the same change:** `docs/plans/invert-hand-maintained.md`
line-52 row says `.gitattributes` must NOT be inverted (sound under its premise — no
upstream authority existed then). The premise is superseded by this ruling; amend the row
to record that `.gitattributes` is now an emitted Git compatibility projection whose
non-default rows derive from repository artifact/storage/integration authorities, with no
independently authored classification.

## Lane 3 — derive stage0 output membership (gates the emitted-Rust extension)

`generated_stage0_files` (dag/gunbc/stage0_emit_model.dag) is a handwritten filename
list — the second-authority shape exactly. It becomes a derivation:

```text
stage0 emission plan → produced module/file identities → canonical output paths
  → generated projection membership → Git merge attributes
```

Only after this derivation lands do stage0 emitted `.rs` outputs join the
generated-projection merge policy (lane 2's rows). Extending `.gitattributes` from the
handwritten roster would propagate the existing dual representation into another
generated file. Resolution guidance for emitted `.rs` meanwhile stays: take either side,
rerun `regen_stage0`, verify divergence 0 — never hand-pick hunks.

## Lane 4 — keyed rosters get set/map construction semantics

For path-keyed grant/roster carriers (`PublicFilePublishGrant` key = `path`): the
writable carrier becomes a keyed set/map or a construction fold that refuses a repeated
key before a roster value exists; duplicate keys refuse at **per-PR admission**, not
only on main (the union-append is the writable bad state git cannot see). A line-union
merge driver remains optional convenience — it can still merge two branches into an
invalid duplicate-key population, so the construction/admission wall is the fix.
(Consistent with the recorded publication-grant re-land bar: set-semantics + subtree
grain + server-side.)

## Landing order and rejection bars

Order: **1** commit-writer admissibility (incl. the `218dea6` reproduction) → **2** the
policy join + generated `.gitattributes` + reconciled/read-back Git config → **3** stage0
membership derivation, then emitted-Rust merge-policy extension → **4** keyed-roster
set/map construction semantics.

Hard-reject bar for all lane PRs:

- no second generated-path roster;
- no hand-maintained `.gitattributes` policy rows;
- no raw marker grep as the sole commit check;
- no hook presented as enforcement;
- no "take ours" treatment for hand-authored policy carriers;
- no Git config mutation without modeled desired state and read-back;
- no commit written before generated projections are regenerated and verified.

## Out of scope, named

Merge-freshness — a branch authored before a roster/authority changed merging with
nothing re-checking it against the new state — is the same shape as the duplicate
incident but independent of conflict handling; it belongs to the merge-admission lane,
not here.
