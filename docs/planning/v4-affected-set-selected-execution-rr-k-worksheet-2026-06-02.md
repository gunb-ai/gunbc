# v4 Affected-Set / Selected-Execution Round-Robin Worksheet (RR-K)

> **Status:** RATIFIED FOR W2 DISPATCH — affected-set + selected-execution cross-cut (§2.9.2).
> **Work item:** `node://adhoc-3ff19e9f-818` — RR-K worksheet (`fierce-ram-162`) under CI Manager (`silent-crane-669`).
> **Gate:** Class 1 design closure only. The affected-set lens answers *what changed*; the selected-execution functions answer *what runs*. This worksheet states the single seam between them and the fail-closed law that seam must preserve. No new runtime, host lens, or scheduling behavior lands from this PR.

## §10.0-adapted worksheet

```text
Migration class:        K-AFFECTED-SET-SELECTED-EXECUTION-CROSSCUT
Representative failure:  Two authorities meet but their seam was never written down. The
                         affected-set authority (AffectedSet / RerunNodeSet / CiComponentAffected,
                         fail-closed B-4/B-5 frontier) computes *what changed*. The selected-
                         execution authority (ci_select_from_affected_set / ci_select_from_rerun_nodes
                         / ci_select_ci_jobs_from_affected_set, plus CiSelectionReceipt) computes
                         *what CI runs*. A reader cannot tell whether selection narrows ONLY through
                         the affected-set frontier, or whether a second "what changed" derivation
                         (a shell affected-detector, a per-job ad-hoc diff predicate) or a fail-OPEN
                         narrowing can silently skip work. Narrowing execution is a kill criterion:
                         get it wrong and CI is green on code it never ran.
Immediate local patch:   Add a job-local `changed?` predicate that re-reads the git diff; let a
                         selection path return the narrowed set directly when the affected-set is
                         uncertain; drop the job needs-closure pass to "save time"; or fork a second
                         affected-component detector (revive detect-affected-components.sh) parallel
                         to CiComponentAffected / tools/ci_affected_components.
Why forbidden:           P2/P5 — a second selection or "what changed" authority parallel to the
                         AffectedSet frontier re-introduces the divergent-derivation defect the
                         T-21/T-24 closure dissolved (MODELING M6 / M9 single-authority). Selection
                         that fails OPEN (skips when uncertain) inverts the load-bearing fail-closed
                         contract: affected-set uncertainty MUST widen execution, never shrink it.
                         CI selection is not a verifier; it may only subtract work the frontier
                         proves untouched, and only with its prerequisite closure intact.
DFS path:
  affected-set authority — "what changed" (CONSUME — do not fork):
    - v4.lens.affected_set — AffectedSet (Produced | FailClosed), RerunNodeSet
      (Produced | FailClosed), affected_set_rerun_nodes, re_exec_frontier_from_diff
    - v4.lens.edit_locus — GitDiffNameOnly, RepoPath (the diff is the only input)
    - v4.workflow.ci — CiComponentAffected, ci_component_affected_from_git_diff,
      ci_component_affected_is_fail_closed, ci_component_mask_intersects
    - tools/ci_affected_components/src/main.rs — `detect-ci-affected-components` host
      transport (dissolves detect-affected-components.sh; not a second authority)
  selected-execution authority — "what runs" (CONSUME — do not fork):
    - v4.workflow.ci — ci_select_from_rerun_nodes, ci_select_from_affected_set,
      ci_select_ci_jobs_from_affected_set, ci_select_ci_jobs_from_jobs,
      ci_select_ci_jobs_needs_closure(_pass), ci_job_selected_by_affected,
      CiSelectionReceipt + ci_selection_receipt_* shadow rows (NON-GATING)
    - v4.std.verification — test_claim_ci_selection_fail_closed (per-claim pin)
  receipts (CONSUME):
    - v4.test.claim.workflow.affected_set_ci_runner — RerunNodeSet roster-narrowing claims
    - v4.test.claim.workflow.ci_component_affected — git-diff path buckets -> flags claims
Deepest unsound boundary:
  The seam is the fail-closed coproduct arm. `affected_set_rerun_nodes` maps
  AffectedSetFailClosed -> RerunNodeSetFailClosed; `ci_select_from_rerun_nodes` maps
  RerunNodeSetFailClosed -> the FULL roster; `ci_component_affected_is_fail_closed` (all
  component flags set) makes `ci_job_selected_by_affected` true for every job. Drop or invert
  any one of these mappings and a "narrowed" run becomes false-green. Narrowing is sound ONLY
  inside the Produced arm, and even there two pins survive: per-claim
  test_claim_ci_selection_fail_closed (DiagnosticClaim always runs) and the job needs-closure
  re-add (a selected job drags in its prerequisites).
Systemic fix:
  Treat selected-execution as a pure projection of the affected-set frontier:
  (1) the ONLY input to "what changed" is the diff, through edit_locus/CiComponentAffected;
  (2) every fail-closed arm widens to the full set — selection never shrinks under uncertainty;
  (3) per-claim fail-closed pins and job needs-closure are invariants of every narrowing;
  (4) host transport (`ci_affected_components` bin, ci.yml job binding) projects these facts,
      it does not recompute them.
Non-goals:
  - No new affected-set or selection substrate; no second "what changed" detector.
  - No host Rust re-implementation of a ci_select_* lens; affected_set_ci_runner stays structural.
  - No gating promotion of the CiSelectionReceipt shadow path (feature:wave3-shadow-selection-
    receipt remains transport-blocked; live entrypoint is future CI work, not this PR).
  - AffectedSetCiReceipt kill-criterion instrumentation (#4271) is IN FLIGHT, not landed —
    consume its shape only once merged; do not re-author it here.
  - No detect-affected-components.sh revival; CiComponentAffected is the authority.
Falsification probe:
  §4 table (R1-R8) — mandatory before any RR-K implementation PR claims PROVEN.
Metric allowed only as secondary:
  Count of roster/jobs subtracted by a Produced frontier. Acceptance is the fail-closed-widens
  law plus needs-closure preservation, never a particular narrowing ratio.
```

---

## §1 Landed Evidence Map

| Artifact | Landed state | RR-K disposition |
| --- | --- | --- |
| T-21/T-24 affected-set authority — `v4.lens.affected_set` (`AffectedSet`/`RerunNodeSet`, `affected_set_rerun_nodes`, fail-closed B-4/B-5 frontier) | MERGED — canonical frontier fold on modeled/hand-built graph inputs; real repo/source input is gated on the source-provenance producer design lane | **Consume as the only frontier source**; the `FailClosed` arm is load-bearing, not a placeholder |
| T-24 `CiComponentAffected` + `ci_component_affected_from_git_diff` + `tools/ci_affected_components` bin | MERGED — dissolves `detect-affected-components.sh`; git diff path buckets -> component flags | **Consume as the only component detector**; `ci_component_affected_is_fail_closed` = all-flags-set widens every job |
| `ci_select_from_rerun_nodes` / `ci_select_from_affected_set` / `ci_select_ci_jobs_from_affected_set` (`v4.workflow.ci`) | MERGED — roster + job selection over the frontier; needs-closure re-add | **Consume as the only selection authority**; narrowing lives only in the `Produced` arm |
| `test_claim_ci_selection_fail_closed` (`v4.std.verification`) | MERGED — per-claim pin (`DiagnosticClaim` always runs) | **Consume as a narrowing invariant**: pins survive frontier filtering |
| `CiSelectionReceipt` + `ci_selection_receipt_*` shadow rows | MERGED — NON-GATING shadow receipt surface | **Shadow only**; promotion to a live gating entrypoint is future CI work, not RR-K |
| #4271 `AffectedSetCiReceipt` — per-PR kill-criterion instrumentation | **OPEN (in flight)** | **Prerequisite-in-flight**: consume its receipt shape only after it merges; do not duplicate |

## §2 RR-K Authority Contract

### 2.1 Single "what changed" authority

The diff is the only input to affected detection, routed through `v4.lens.edit_locus`
(`GitDiffNameOnly` / `RepoPath`) into either the node frontier (`AffectedSet` / `RerunNodeSet`)
or the component flags (`CiComponentAffected` via `ci_component_affected_from_git_diff`).

**Accepted pattern:** add a path bucket / dependency edge inside the existing lens so a new
file class flows into the existing flags or frontier.

**Rejected pattern:** any second derivation of "what changed" — a job-local `changed?` predicate
re-reading the diff, a shell affected-detector, or a hand-Rust component classifier parallel to
`tools/ci_affected_components`.

### 2.2 Selection is a projection of the frontier, and it fails closed

Selected-execution narrows ONLY through `ci_select_from_rerun_nodes` /
`ci_select_ci_jobs_from_affected_set`. Every fail-closed arm widens to the full set:

- `affected_set_rerun_nodes(AffectedSetFailClosed) -> RerunNodeSetFailClosed`
- `ci_select_from_rerun_nodes(RerunNodeSetFailClosed) -> full roster`
- `ci_component_affected_is_fail_closed(...) -> ci_job_selected_by_affected = true` for every job
- `ci_select_ci_jobs_from_affected_set` returns all jobs when the pipeline is cyclic or any
  `needs` fails to resolve

A selection path that returns a narrowed set under uncertainty (fails OPEN) is forbidden:
affected-set uncertainty must always *widen* execution.

### 2.3 Narrowing invariants

Inside the `Produced` arm, two pins are preserved on every narrowing:

1. **Per-claim fail-closed pins** — `test_claim_ci_selection_fail_closed` forces a claim into the
   selected roster regardless of the frontier (`DiagnosticClaim` always runs).
2. **Job needs-closure** — `ci_select_ci_jobs_needs_closure` re-adds the transitive `needs`
   prerequisites of every selected job; a job is never run without its prerequisites.

### 2.4 Host transport projects, it does not recompute

`tools/ci_affected_components` and the `.github/workflows/ci.yml` affected-job binding are
transport that *projects* the modeled facts. They may not recompute the affected set or
re-implement a `ci_select_*` lens. CI authority stays in `v4.workflow.ci` + `dsl/gunbc`
carrier; editing only `ci.yml` for a modeled selection fact is forbidden (splits authority).

## §3 Implementation Lanes

| Lane | Allowed work | Required receipt |
| --- | --- | --- |
| K.1 frontier/component widening | New path buckets or dependency edges feeding the existing `AffectedSet`/`CiComponentAffected` authority | `ci_component_affected` claim row + an affected_set_ci_runner row proving the new class narrows/widens correctly |
| K.2 selection consumers | New consumers of `ci_select_*` over an existing roster/job set | Structural claim that fail-closed arms widen and pins/needs-closure survive |
| K.3 receipt instrumentation | Consume #4271 `AffectedSetCiReceipt` shape once merged | Per-PR kill-criterion receipt; no duplicate authority |
| K.4 host transport | `ci_affected_components` bin / `ci.yml` job binding wiring | Modeled CI smoke + `dsl/gunbc` carrier pin update; no recomputation |

## §4 Falsification Table (Implementation PROVEN)

| ID | Probe | Receipt |
| -- | ----- | ------- |
| R1 | "What changed" has exactly one authority: every affected fact traces to `edit_locus` -> `AffectedSet`/`CiComponentAffected` | Diff review; no second diff reader |
| R2 | `AffectedSetFailClosed` / `RerunNodeSetFailClosed` widen to the full roster | Selection test on a fail-closed frontier returns the whole roster |
| R3 | `ci_component_affected_is_fail_closed` selects every job | `affected_set_ci_runner`/job-selection claim with all-flags-set |
| R4 | Cyclic pipeline or unresolved `needs` returns all jobs | `ci_select_ci_jobs_from_affected_set` cyclic/unresolved fixture |
| R5 | Per-claim `test_claim_ci_selection_fail_closed` pins survive a `Produced` narrowing | Roster-narrowing claim with a `DiagnosticClaim` present |
| R6 | A selected job drags in its `needs` prerequisites | `ci_select_ci_jobs_needs_closure` closure-pass test |
| R7 | Host transport (`ci_affected_components`, `ci.yml`) projects modeled facts, recomputes nothing | Diff review + `dsl/gunbc` carrier pin match |
| R8 | No fail-OPEN narrowing path exists (selection never shrinks under uncertainty) | Diff review over every `ci_select_*` caller |

## §5 Forbidden Patterns

| Pattern | Why forbidden |
| ------- | ------------- |
| Second "what changed" detector (shell, job-local diff predicate, hand-Rust classifier) | Divergent derivation parallel to the `AffectedSet`/`CiComponentAffected` authority (M9) |
| Fail-OPEN narrowing (return narrowed set under uncertainty) | Inverts the load-bearing fail-closed contract; CI false-green |
| Dropping the job needs-closure pass | Runs a job without its prerequisites |
| Dropping per-claim fail-closed pins | `DiagnosticClaim` coverage silently lost |
| Reviving `detect-affected-components.sh` | Re-forks the dissolved detector |
| Host Rust re-implementation of a `ci_select_*` lens | Selection authority fork |
| Editing only `.github/workflows/ci.yml` for a modeled selection fact | Splits CI authority from `ci.dag` + carrier pin |
| Promoting the `CiSelectionReceipt` shadow to gating from this worksheet | Shadow path is transport-blocked; out of RR-K scope |

## §6 Landing Order

```text
1. RR-K merged (this doc) — CI Manager may dispatch Class 2 affected-set/selection workers.
2. K.1/K.2 implementation PRs: frontier/component widening and selection consumers with
   fail-closed + needs-closure receipts.
3. K.3 receipt instrumentation: consume #4271 AffectedSetCiReceipt shape only after it merges.
4. K.4 host transport: ci_affected_components / ci.yml binding with carrier pin update.
5. Follow-up dissolution: promote the CiSelectionReceipt shadow to a live entrypoint only when
   workflow/TestClaim projection executes the modeled selection facts.
```

## §7 Handoffs

- **CI Manager (`silent-crane-669`)**: owns selection-gating policy and the #4271 receipt lane.
  RR-K does not authorize promoting the shadow receipt to gating.
- **Runtime / TestClaim (RR-I)**: owns roster authority and per-claim verdicts; RR-K consumes
  `test_claim_ci_selection_fail_closed` and the Wave-3 shadow roster, it does not author claims.
- **Branch H / Source Authority**: the git diff is input, not source authority; affected-set
  selection may not treat the diff or CI artifacts as canonical `.dag` source. The live
  affected-testgen gate is a fixture-wiring slice over hand-built graph/provenance rows;
  real-input coverage is blocked on a separate design-first source-provenance producer lane
  that derives `NodeArtifactProvenance` from the compiler/source-authority ingest, not from
  snapshots or a parallel scanner.

## §8 Modeling DFS Arbiter Checklist

- [x] Single-authority: one "what changed" source (`edit_locus` -> `AffectedSet`/`CiComponentAffected`); one selection authority (`ci_select_*`).
- [x] Fail-closed seam recorded: every `FailClosed`/cyclic/unresolved arm widens to the full set.
- [x] Narrowing invariants recorded: per-claim pins + job needs-closure survive every `Produced` narrowing.
- [x] Host transport projects modeled facts; it does not recompute or fork the detector.
- [x] #4271 treated as in-flight prerequisite, not duplicated.
- [x] Falsification R1-R8 accepted.
- [x] **READY-FOR-WORKER-DISPATCH** (RR-K Class 1 closure — implementation workers may proceed under §3).

---

## Related Artifacts

- `src/v4/lens/affected_set.dag` — `AffectedSet`, `RerunNodeSet`, `affected_set_rerun_nodes`, fail-closed B-4/B-5 frontier
- `src/v4/lens/edit_locus.dag` — `GitDiffNameOnly`, `RepoPath`
- `src/v4/workflow/ci.dag` — `CiComponentAffected`, `ci_component_affected_from_git_diff`, `ci_component_affected_is_fail_closed`, `ci_select_from_rerun_nodes`, `ci_select_from_affected_set`, `ci_select_ci_jobs_from_affected_set`, `ci_select_ci_jobs_needs_closure`, `ci_job_selected_by_affected`, `CiSelectionReceipt`
- `src/v4/std/verification.dag` — `test_claim_ci_selection_fail_closed`
- `tools/ci_affected_components/src/main.rs` — `detect-ci-affected-components` host transport
- `src/v4/test/claim/workflow/affected_set_ci_runner.dag` — roster-narrowing claims
- `src/v4/test/claim/workflow/ci_component_affected.dag` — git-diff path-bucket claims
- gunb-ai/gunbc#4271 — `AffectedSetCiReceipt` per-PR kill-criterion instrumentation (in flight)
- `docs/planning/v4-runtime-testclaim-rr-i-worksheet-2026-06-02.md` — roster/claim authority (RR-I)
- gunb-ai/gunbc#4333 — sibling CI cross-cut worksheet (RR-L), **OPEN / not yet on main**; `docs/planning/v4-incremental-bootstrap-ci-perf-rr-l-worksheet-2026-06-02.md` lands with that PR
