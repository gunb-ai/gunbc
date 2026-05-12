# Decomposition Algebra — ctrl/ migration scoping doc

**Status**: DRAFT (2026-05-12). Authored per operator directive: migrate ctrl/ processes into .dag substrate; algebra is authoritative, ctrl/ TS becomes projected emission.

**Authority**: ctrl/ PRs #1192 / #1193 / #1195 / #1197 (decomposition-algebra series, stacked, all OPEN as of 2026-05-12T18:30Z). These four PRs land the algebra in TS; this doc proposes the .dag substrate that makes the algebra source-of-truth.

**Out of scope**: implementation of emission targets (HTTP REST / SQL migrations / audit-event emission). Those are Phase 3 — substrate-prerequisites that gate the ctrl/ replace. This doc is Phase 0 (readiness audit + integration sketch).

---

## §1. Concept

The operator wants `ctrl/` (dashboard / planning / replanning) workflows expressed as projections of a typed .dag substrate, with pedantic algebra underneath and a friendly CLI on top (`dashboard-ops open-task` / `replan` / `close` / `archive` / `escalate` / `pause`). The decomposition algebra defined in ctrl PRs #1192-#1197 is the candidate substrate: nodes have a structural type (leaf / composite / bucket / NULL); closure is emergent for composites; replan is more-decomposition; buckets capture algebraic remainder.

The .dag migration makes that algebra the **single authority**. The TS implementation in `ctrl/` becomes one emission target (REST handlers + SQL schema + CLI surface), but no longer authors the rules. Cost-of-change for a new Mode variant or Operation arm collapses to 1 file (the substrate). Per `feedback_lenses_not_passes.md`: the algebra is a lens over graph state, not a heuristic pass.

---

## §2. The ctrl/ algebra — concrete reference

### §2.1 Schema (ctrl PR #1192)

Adds to `nodes` table:
- `mode TEXT CHECK IN ('leaf', 'composite', 'bucket')` (or NULL) — declared shape
- `mode_declared_at`, `mode_declared_by_session_id` — audit fields
- `bucket_drained_at`, `bucket_drained_by_session_id`, `bucket_drain_note` — drain receipt
- `dashboard_migrations` row `decomposition_algebra.cutoff.v1` — cutoff timestamp; pre-cutoff rows grandfathered

`NODE_MODES` exported as single authority; DB CHECK enforces at boundary.

### §2.2 Operations (ctrl PR #1193)

API endpoints:
- `POST /api/nodes/:id/declare { mode, session_id? }`
- `POST /api/nodes/:id/bucket/drain { note, session_id? }`
- `POST /api/internal-work-items` (existing; now auto-flips parent to `composite` on first child via `autoFlipParentModeForChildAddition`)

Pure helpers in `lib/dag_writes.mjs`:
- `declareNodeMode` — idempotent on same value; refuses redeclare to different mode (caller must replan)
- `drainBucket` — refuses non-bucket nodes + empty notes; mandatory witness string
- `autoFlipParentModeForChildAddition` — "the act of authoring a child IS the declaration of the parent"

`DecompositionError` with `code` field for HTTP status mapping.

### §2.3 Closure rule (ctrl PR #1195)

`canCloseNode(db, nodeId, { cutoff }) → { ok, reason }` — single source of truth.

Reasons:
- `NODE_NOT_FOUND`
- `ALREADY_CLOSED` (idempotent path)
- `MODE_NOT_DECLARED` (post-cutoff node with mode=NULL)
- `BUCKET_NOT_DRAINED` (mode='bucket' without bucket_drained_at)
- `COMPOSITE_HAS_OPEN_CHILDREN` (mode='composite' with ≥1 open blocks-child)

`closeNode(db, nodeId, { closedAt, force, cutoff })` wraps the guard; `force: true` is operator override; `node_close_refused` events logged on refusal.

### §2.4 Replan (ctrl PR #1197)

`dashboard-ops replan` thin client authors a reconcile work-item under the caller's parent. No new schema; walk-back is operator-driven recursion through normal work-item flow. **Replan IS more decomposition, not a state revert** — the meta-work is itself work, expressed with the same primitives.

---

## §3. Mapping to existing gunbc primitives (grep-verified)

**Audit scope correction (2026-05-12, post-codex Finding #1)**: prior version scoped audit to `dsl/std/` only and missed the v3 lens substrate. Corrected audit covers `dsl/std/` + `src/v3/std/` + `src/v3/lenses/`. The v3 substrate is the live home for many primitives I previously marked ABSENT.

| Decomp-algebra concept | Gunbc primitive | Citation | Reuse status |
|---|---|---|---|
| Node (work item) | Recursive type pattern; concrete `Node` is compiler-AST | `dsl/std/node.dag:55-196` | **Concept reuses; carrier needs new domain-specific declaration** (compiler Node is for AST, not work items) |
| Mode (open enum) | Open enum + Practice 4 dissolution framework | `dsl/std/computation.dag:133,192,246` (`SizeBound` / `CallPattern` / `IterationPrimitive`) | ✓ Direct reuse of pattern |
| Operation (closed sum) | Closed enum with payloads | `dsl/std/effects.dag:71-76` (`EffectShape`); `dsl/std/computation.dag:246-249` (`IterationPrimitive`) | ✓ Direct reuse of pattern |
| Closure projection `canCloseNode` | Projection function pattern (Slice 4 substrate) | `dsl/gunbc/ci.dag:29-32` documents shape; `dsl/gunbc/ci_emission.dag` not yet authored (Slice 4 in flight) | ✓ Pattern documented; concrete projection function pattern lands soon |
| Cardinality (single/optional) | `Required \| Optional` | `dsl/std/constructors.dag:74-75` | ✓ Direct reuse |
| Bounded multiplicity (0..N) | List<T> + SetCardinality (size primitive) | `dsl/std/termination.dag:198` (`SetCardinality`) | ◐ Partial — no `Bounded<min, max>` carrier yet |
| **Witness<C>** (Inhabits \| Violates) | First-class typed witness; the substrate is live in v3 | `src/v3/std/dimensions.dag:35` (`Witness<Carrier> = Inhabits(Carrier) \| Violates { reason, at }`) | ✓ **EXISTS — corrects prior audit miss**. Decomp-algebra Witness can reuse this carrier. |
| OptionalDiagnostic | Optional break channel for `break_diagnostic` | `src/v3/std/dimensions.dag` (`OptionalDiagnostic = NoDiagnostic \| SomeDiagnostic`) | ✓ Reuse |
| DimensionReport<C> | Aggregate result from analysis fold (pass/fail partition) | `src/v3/std/dimensions.dag` (`DimensionReport<Carrier> = DimensionOk { composed, witnesses } \| ...`) | ✓ Reuse (closure-decision shape candidate) |
| Timestamp | Narrowed String (ISO 8601) | `dsl/std/types.dag:299` | ✓ Reuse |
| Clock resource | `Clock` with `now()` | `dsl/std/resources.dag:83-95` | ✓ Reuse |
| Algebraic structures | Magma / Semigroup / Monoid / Group / Ring / Field / FreeMonoid | `dsl/std/algebra.dag:99-320` | ✓ Reuse — closure rules can compose under appropriate structure |
| Graph / edges | Field-based on Node; explicit `GraphEdge` in `CallGraph` | `dsl/std/node.dag` (children: List<Node>); `dsl/std/graph.dag:15-30` | ◐ Partial — no labeled edges (parent→child semantic relationship vs dependency vs constraint) |
| Coproduct dissolution receipts | 🟡 MIXED / 🟢 TERMINAL emoji + dissolution-trigger blocks | `dsl/std/computation.dag:126,189,228,244`; `docs/v3-modeling-analysis.md:217-229` (ledger rule); `src/v3/std/coproduct_projection.dag` (substrate dispatch for Practice 4) | ✓ Direct reuse of pattern |
| Effects model | `EffectShape` for idempotency; `IterationPrimitive` for computation | `dsl/std/effects.dag:71-76`; `dsl/std/computation.dag:246-249`; `src/v3/std/effects.dag` (v3 expansion) | ✓ Reuse — algebra operations are EffectShape-class graph mutations |
| **Lens<C>** (compositional analysis carrier) | 🟡 TRANSITIONAL 6-field shape (Director-locked); reuses Witness/OptionalDiagnostic/DimensionReport/Monoid | `src/v3/std/lens.dag` (`Lens<C> { name, read, sequential, branch, iterate, validate }`); `src/v3/lenses/` (~16 worked instances: complexity, cost, parallelism, idempotency, ...) | ✓ **EXISTS — corrects prior audit miss**. Decomposition-algebra closure/state projections fit this carrier shape. |
| Audit log / EventLog<T> | Domain-only at `dsl/gunbc/workflow/types.dag:327-335` (`AuditEntry`); no std/ primitive | — | **✗ GAP — needed for replay-as-truth (see §5 Gap 1)** |
| Task / WorkItem / Workflow | Domain-specific types in `dsl/gunbc/workflow/types.dag` (`ImplementationTask`, `TrackedIssue`, `IssueLifecycleStage`, `StageOutcome`) | `dsl/gunbc/workflow/types.dag:46-194` | **OVERLAP — see §4 (axis-conflation correction post-codex Finding #2)** |

---

## §4. Workflow domain overlap — significant finding

The audit surfaced an existing workflow type system in `dsl/gunbc/workflow/types.dag` (~340 lines, gunbc-domain):

| Type | Lines | Decomp-algebra analog |
|---|---|---|
| `ImplementationTask { title, description, file_paths?, dependencies?, done: Bool }` | 106-112 | Leaf-mode Node |
| `IssueLifecycleStage = Idea \| Design \| DesignReview \| Accepted \| Implementing \| CodeReview \| Testing \| Done \| TerminalFailed` | 46-54 | State-machine-residue parallel to Mode |
| `TrackedIssue { id, title, body, stage, url?, author?, labels, created_at?, updated_at? }` | 57-67 | Composite Node with stage tag |
| `StageOutcome { run_key, stage, status, payload?, error?, attempt_count, retry_budget_remaining, ... }` | 183-194 | Operation outcome / execution receipt |
| `AuditEntry { timestamp, actor, action, entity_type, entity_id, before?, after? }` | 327-335 | Single audit-log event |
| `Signal { signal_type, idempotency_key, payload, produced_at, consumed_at? }` | 245-250 | Idempotent operation event |

These types pre-date the decomposition-algebra work and overlap with it. The key question for this scoping doc:

**Do the workflow types REPLACE the decomp-algebra carriers, EXTEND them, or DISSOLVE INTO them?**

**Initial proposal**: DISSOLVE INTO decomp-algebra Mode + closure rules.

**CORRECTION (post-codex Finding #2 2026-05-12T19:08Z)**: the initial proposal **conflated two orthogonal axes**:

1. **Decomposition axis** (Mode = Leaf / Composite / Bucket / NULL) — STRUCTURAL; describes how a node closes
2. **Workflow phase axis** (Idea / Design / Implementing / CodeReview / Testing / Done) — TEMPORAL/PROCESS; describes where in the workflow lifecycle a node is

These are **orthogonal coordinates**. A Composite node can be in "Implementing" OR "CodeReview" phase. A Leaf node can be in "Design" OR "Done" phase. The naive collapse of stages to modes drops information that real consumers use (e.g., review-scheduler decides "trigger review at CodeReview stage transition" — that's not a Mode property).

**Corrected dissolution shape — preserve both axes as structural coordinates** (per Practice 4 "dimensional" dissolution pattern; M-dimensional space hidden in flat N-variant enum):

```
type Node {
  mode: Mode           // Leaf | Composite | Bucket | NULL  (decomposition axis)
  phase: Phase         // workflow lifecycle axis
  ...
}

type Phase
  = Pre              // pre-decomposition: idea / design / design-review / accepted
  | Active           // decomposition committed: implementing
  | InReview         // composite-with-children-in-witness-collection: code-review / testing
  | Closed           // canCloseNode passed: done / terminal-failed (with closure-witness)
```

`Phase` is itself an open enum subject to Practice 4 — variants will dissolve further as consumers prove out (e.g., `InReview` may split if review-vs-test diverge structurally).

**ALTERNATIVE — prove every existing stage consumer dissolves**: enumerate every `ctrl/` site that reads `IssueLifecycleStage` and show its question is answerable from `(mode, phase, closure_witnesses)` instead of from raw stage. This is the rigorous dissolution proof; it's deferred to Phase 1.5 work and is the closure predicate for the workflow-types dissolution PR.

**Until either proof lands**: workflow-types stay extant; decomp-algebra is co-located not replacing. The dissolution claim is **staged with trigger**: trigger = per-consumer enumeration proves no information loss.

**Pipeline-coordinate facts MUST be preserved as separate axes** (post-codex inline BLOCKING #2 2026-05-12T19:08Z): the grep audit of `dsl/gunbc/workflow/types.dag` surfaces facts USED downstream as pipeline coordinates, that the initial dissolution proposal would have dropped:

| Fact | Citation | Pipeline role | Preservation strategy |
|---|---|---|---|
| `StageRunKey` | `dsl/gunbc/workflow/types.dag:159` | Unique per-stage-invocation identifier; threads through `StageOutcome:184`, `PipelineArtifact:213`, `MetricRecord:225`, `RetryDue:239`, `TerminalStateReached:243` | KEEP as first-class fact; `RunKey` is a node-decomposition-independent coordinate (work-item can have N runs across its lifecycle). NOT dissolved into Mode/Phase. |
| `ClaimLease` | `dsl/gunbc/workflow/types.dag:166` | Lease-generation + expiry for stage-execution claims | KEEP. Lease-claim semantics are pipeline-coordinate, not decomposition-state. |
| `SignalType` | `dsl/gunbc/workflow/types.dag:235` | Idempotency-keyed signal payload kind | KEEP. Signals are events on the EventLog (Gap 1); SignalType is the event-payload tag, structurally distinct from Mode/Phase. |
| `PipelineArtifact` | `dsl/gunbc/workflow/types.dag:120, 212-214` | Stage-output artifact with type tag (`artifact_id`, `artifact_type`) | KEEP. Artifacts are stage-produced facts; flow forward from run to run. |
| `ArtifactType` | `dsl/gunbc/workflow/types.dag:214, 226` | Artifact taxonomy | KEEP as open enum subject to Practice 4. |
| Metrics (`artifacts_stored: Int` at 318, `MetricRecord` at 225) | `dsl/gunbc/workflow/types.dag:225, 318` | Per-stage telemetry; aggregated upstream | KEEP. Metrics are observability facts, structurally separate. |

**Per `feedback_projections_must_compose_facts.md`** + **INVARIANTS P2 facts-flow-forward**: facts MUST flow forward through projections. Dissolving stage to mode/phase WITHOUT preserving stage-bound facts violates P2. The corrected dissolution treats `IssueLifecycleStage` as a stage-tag whose VALUE collapses into `(Mode, Phase)` BUT the run-keyed facts (RunKey / ClaimLease / SignalType / Artifacts / Metrics) remain structurally distinct.

**Other workflow-type mappings** (refined):
- `ImplementationTask.done: Bool` → leaf-Mode + Phase=Closed case (Mode × Phase axes)
- `TrackedIssue.stage` (the stage-value) → collapses into `(currentMode, currentPhase)` projection lens
- `StageOutcome` → KEEP as run-keyed pipeline fact; `StageOutcome.status` → Operation outcome on a specific run; `run_key` threads forward
- `StageRunKey` → KEEP as first-class run-identifier; orthogonal to decomposition
- `ClaimLease` → KEEP as lease-execution coordinate
- `Signal` → audit-log-position model for idempotency; `SignalType` enum tag stays
- `PipelineArtifact` → KEEP; artifacts are stage-output facts forward-flowing
- `AuditEntry` → per-event payload inside `EventLog<Operation>` primitive (Gap 1 from §5)

Per `feedback_checkpoint_dissolution_default.md`: at C-checkpoints, dissolution is default — but dissolution requires both structural-coordinate preservation (Mode × Phase axes) AND fact-preservation (run-keyed facts flow forward unaffected). The dissolution proof is bounded: only stage-VALUE collapses; stage-bound facts persist.

---

## §5. Four gaps as extension proposals

### Gap 1 — EventLog<T> primitive

**Need**: replay-as-truth model. Current node state is a projection over the audit log; closure is `canCloseNode(replay(log_at(now)))`.

**Sketch**:
```
type EventLog<T> {
  events: List<TimestampedEvent<T>>
  // List has monotonic timestamp invariant per event
}

type TimestampedEvent<T> {
  position: MonotonicSeq   // strictly increasing
  timestamp: Timestamp
  payload: T
}
```

**Algebra**: `EventLog<T>` forms a `FreeMonoid<TimestampedEvent<T>>` under append (already in `dsl/std/algebra.dag:390`). Replay is `fold` (already in `IterationPrimitive`).

**Consequences if landed**:
- `node_close_refused` event in PR #1195 becomes structurally emitted via `EventLog<NodeOperation>`, not hand-implemented logging call
- Stale-RC parser bug class **dissolves**: review-tally is `latest_per_provider_at_HEAD = fold(log, by_provider, take_latest_for_current_HEAD)`. Old-SHA RC is structurally not-at-HEAD.
- Cutoff timestamp (PR #1192) becomes `log_position` instead of `dashboard_migrations.created_at`

### Gap 2 — RETRACTED (post-codex inline BLOCKING 2026-05-12T19:08Z)

**Original proposal**: introduce `Lens<S, A> { view, update }` as new bidirectional state-projection carrier.

**Retracted because**: `src/v3/std/lens.dag` already declares a Director-locked `Lens<C>` substrate (6-field shape: name / read / sequential / branch / iterate / validate). Introducing a separate `Lens<S, A>` creates parallel authority — violates INVARIANTS P2 (single-authority) + MODELING.md M9 (DFS the concept DAG before defining).

**Corrected approach**: state-projection in decomp-algebra reuses existing substrate. Specifically:
- Mode-as-projection = `fold` over `FreeMonoid<TimestampedEvent<Operation>>` (FreeMonoid already in `dsl/std/algebra.dag:390`); no Lens carrier needed for one-way projection
- Closure-decision = `canCloseNode` as a typed projection function (pattern per `dsl/gunbc/ci.dag:29-32` `project_github_actions` shape)
- If the v3 Lens<C> 6-field shape (read / sequential / branch / iterate / validate) fits a future ctrl-domain analysis, reuse-as-instantiation; substrate-parameterization is a downstream concern, not this canvas

**Open question**: if decomposition-algebra surfaces a genuine bidirectional-update use case that doesn't fit v3 Lens<C> (which is fold-only over DAG-with-behaviors), surface to Substrate Mgr for shape audit. Until then, no new lens carrier proposed.

### Gap 3 — Bounded multiplicity

**Need**: bucket-mode wants "0..N children with semantic-remainder property." `Cardinality = Required | Optional` is too coarse.

**Sketch**:
```
type Bounded<N: Nat> {
  min: Nat
  max: Optional<Nat>   // None = unbounded
  current_count: Nat
}
```

OR — reuse `SetCardinality` from `dsl/std/termination.dag:198` if it generalizes.

**Lower priority** than Gaps 1+2 — workable with current List<T> for now.

### Gap 4 — Attestation carrier (RENAMED from "Witness" to avoid v3 collision)

**Carrier-name collision audit** (post-codex inline BLOCKING 2026-05-12T19:08Z): v3 substrate already has `Witness<Carrier> = Inhabits(Carrier) | Violates { reason, at }` at `src/v3/std/dimensions.dag:35`. That carrier is per-Behavior cost-basis-inhabitance proof — STRUCTURALLY DISTINCT from decomp-algebra's human-attestation-of-intent (drain note, replan reason, escalate debt-string, operator-override).

**Per `feedback_self_hosting_md_authority_audit_before_substrate_naming.md`**: same-name carriers across namespaces invite confusion. Rename decomp-algebra's "Witness" to **`Attestation`** to preserve namespace clarity.

**Need**: typed attestation record for human-authored intent attached to Operation events.

**Sketch**:
```
type Attestation {
  author: SessionId
  text: String
  timestamp: Timestamp
  evidence: List<AttestationEvidence>
}

type AttestationEvidence =                 // 🟡 MIXED — will become TERMINAL when locked
    StructuralLensReceipt { lens_name: String }   // reuses v3 Lens<C> instance result
  | IntegrationTest { name: String, passes: Bool }
  | ProseAttestation { text: String }
```

**Reuse**: where decomp-algebra carriers reference v3 substrate (e.g. an `AttestationEvidence::StructuralLensReceipt` whose lens-instance is a real `Lens<C>` from `src/v3/lenses/`), the existing `Witness<C>` carrier is the inhabitance proof — `Attestation` and v3 `Witness<C>` compose; they don't conflict.

**Lower priority** — small extension, can land alongside Phase 1 substrate. The `Witness` → `Attestation` rename cascade across §6/§9/§13 in this doc applied below.

---

## §6. Operations as effects

**Operations** (graph mutations on the typed node-graph):

```
type Operation =
    Declare { node: NodeId, mode: Mode, attestation: Attestation }
  | Decompose { parent: NodeId, children: List<Node>, attestation: Attestation }
  | Drain { bucket: NodeId, attestation: Attestation }
  | Replan { node: NodeId, reason: Attestation }
  | Escalate { from: NodeId, to_parent: NodeId, debt: Attestation }
  | Pause { node: NodeId, reason: Attestation }
  | Reopen { node: NodeId, reopen: ReopenAttestation }            // closed → open
  | Regress { node: NodeId, retracted_child_ids: List<NodeId>, regression: RegressionAttestation }
  | AttestedOverride { rule: ClosureRule, attestation: Attestation }   // force: true
```

(Renamed: `AttestedOverride` → `AttestedOverride` to match the `Attestation` carrier naming; per Gap 4 carrier-rename.)

**Reopen + Regress added (post-codex Finding #3 2026-05-12T19:08Z)**: real workflows have closure-reversal cases:
- PR merge reverted post-CI failure
- Gate ratchet retroactively determined wrong
- Subtree decomposition retracted (re-merge-into-parent)

Without explicit `Reopen` / `Regress` operations, the monotonicity claim was unfounded. Each has explicit witness requirements:

```
type ReopenAttestation {
  reason: ReopenReason
  attestation: Attestation   // why reopen is justified
}

type ReopenReason =
    PostMergeRevert { revert_commit: Sha }
  | RetroGateInvalidation { gate_id: GateId }
  | OperatorIntervention { directive: String }

type RegressionAttestation {
  reason: String
  retracted_subtree_signature: String   // structural proof of what's being merged back
  attestation: Attestation
}
```

Each operation is a **pure function** `(EventLog<Operation>, Operation) → EventLog<Operation>` (append). Graph state is `state: EventLog → NodeGraph` (projection lens).

**Closure-decision lattice — REPLACES prior monotonicity claim**: closure decisions are NOT monotonic across the full Operation set. They are monotonic only across `{Declare, Decompose, Drain, Pause, AttestedOverride}` — the "forward-stable" subset. `Reopen` + `Regress` + `Replan` + `Escalate` explicitly RETRACT closure state with witnessed cause.

The lattice over closure decisions:
- `canCloseNode(state(log + forward_stable_op)) ≥ canCloseNode(state(log))` (preserved-or-improved)
- `canCloseNode(state(log + Reopen{node, ...})) < canCloseNode(state(log))` IF the prior state had `node` closed (explicit regression with witness)
- `canCloseNode(state(log + Regress{node, ...})) < canCloseNode(state(log))` similarly (decomposition retraction may reopen closure questions on the parent)

This is `feedback_state_space_vs_behavioral_invariants.md` applied: the type captures the regression possibility explicitly. There is no silent regression — every closure-reversal carries a typed witness, and every consumer of `canCloseNode` sees the lattice value rather than "stable boolean."

`AttestedOverride` is the structurally-modeled `force: true` from PR #1195. It's a typed operation, not a special-case API flag.

---

## §7. Dissolution receipts

### `Mode = Leaf | Composite | Bucket | NULL` — 🟡 MIXED

**Dissolution trigger**: when `Mode` becomes derivable from graph state.

**Derivation candidates**:
- `Leaf` = node has no children AND no decomposition declared
- `Composite` = node has ≥1 child OR explicit composite declaration
- `Bucket` = node has explicit `is_remainder` marker + bound drain rule
- `NULL` = pre-cutoff (grandfathered; dissolves entirely when all grandfathered nodes close)

Mode dissolution trigger: when `currentMode: EventLog<Operation> → Mode` projection function lands (a `fold` over `FreeMonoid<TimestampedEvent<Operation>>` per Gap 2 retraction), Mode collapses from stored field → derived projection. The `mode_declared_at` audit becomes a tag on the fold result (provenance, not state). No new lens carrier needed — reuses existing FreeMonoid + fold substrate.

### `Operation = Declare | Decompose | ... | AttestedOverride` — 🟢 TERMINAL

Closed sum over the friendly-CLI vocabulary; payload is structural. No dissolution proposed — each variant is structurally distinct (different effect shape per `EffectShape` mapping).

### `Attestation` — 🟡 MIXED → 🟢 TERMINAL pending AttestationEvidence enum closure

When `AttestationEvidence` enum (StructuralLensReceipt / IntegrationTest / ProseAttestation) is locked, `Attestation` becomes terminal. Strength ordering: `StructuralLensReceipt > IntegrationTest > ProseAttestation`. Distinct from v3 `Witness<C>` (`src/v3/std/dimensions.dag:35`) — `Attestation` is human-intent attestation; `Witness<C>` is per-Behavior inhabitance proof. They compose: a `StructuralLensReceipt` may reference a `Lens<C>` instance whose result is a `Witness<C>` chain.

---

## §8. Cost-of-change contract

**Adding a new Mode variant** (e.g. `Periodic`, `Replicated`) should touch **exactly 1 file**: `dsl/std/process_algebra.dag` (or wherever Phase 1 lands).

Derived emissions:
- CLI vocabulary (`dashboard-ops open-task --periodic`) — derived from `Mode` enum projection
- REST endpoint validation (`POST /api/nodes/:id/declare { mode: 'periodic' }`) — derived from `Mode` schema projection
- SQL `CHECK IN (...)` constraint — derived from `Mode` enum coordinate dissolution
- Audit-event variant — derived from `Operation` enum payload

Verification approach: a test that asserts `count(Mode variants) = count(CLI declare flags) = count(SQL CHECK values) = count(REST schema enum values)`. If hand-edit required anywhere, cost-of-change > 1 — failure.

This is the same shape as `EmissionTarget = YamlStatic | BinaryShim` deciding `project_github_actions` arm dispatch in T-WAD substrate.

---

## §9. Phase 1 substrate-file outline

`dsl/std/process_algebra.dag` (proposed; new file):

```
// Process algebra — workflow planning/replanning as composition of typed
// operations on a node-graph. Source authority for ctrl/ planning state.
//
// Per feedback_lenses_not_passes.md: this substrate is a lens over event-log,
// not a heuristic pass. Current state is a projection of the audit log; closure
// is canCloseNode applied to the projection.

// === Mode (open enum, Practice 4 MIXED) ===
type Mode = Leaf | Composite | Bucket | NULL_TRANSITIONAL
  // 🟡 MIXED — dissolution trigger: when `currentMode` projection function lands,
  // Mode collapses from stored field → lens projection. NULL dissolves
  // entirely when all grandfathered nodes (pre-cutoff) close.

// === Witness ===
// `Attestation` is the decomp-algebra attestation carrier (renamed from
// "Witness" to avoid collision with v3.std.dimensions::Witness<C>).
type Attestation {
  author: SessionId
  text: String
  timestamp: Timestamp
  evidence: List<AttestationEvidence>
}

type AttestationEvidence =          // 🟡 MIXED — will become TERMINAL when locked
    StructuralLensReceipt { lens_name: String }   // refs v3 Lens<C> instance result
  | IntegrationTest { name: String, passes: Bool }
  | ProseAttestation { text: String }

// === Operation (closed sum, TERMINAL) ===
type Operation =                   // 🟢 TERMINAL
    Declare { node: NodeId, mode: Mode, attestation: Attestation }
  | Decompose { parent: NodeId, children: List<NodeId>, attestation: Attestation }
  | Drain { bucket: NodeId, attestation: Attestation }
  | Replan { node: NodeId, reason: Attestation }
  | Escalate { from: NodeId, to_parent: NodeId, debt: Attestation }
  | Pause { node: NodeId, reason: Attestation }
  | AttestedOverride { rule: ClosureRule, attestation: Attestation }

// === EventLog<T> primitive (Gap 1) ===
type EventLog<T> = FreeMonoid<TimestampedEvent<T>>

type TimestampedEvent<T> {
  position: MonotonicSeq
  timestamp: Timestamp
  payload: T
}

// === Closure rule (lens projection) ===
type CloseDecision =
    OK
  | NotFound
  | AlreadyClosed
  | ModeNotDeclared
  | BucketNotDrained
  | CompositeHasOpenChildren

projection canCloseNode(
  graph: NodeGraph,
  node_id: NodeId,
  cutoff: Cutoff
) -> CloseDecision

// === Graph projection over event log ===
projection state(log: EventLog<Operation>) -> NodeGraph

projection currentMode(
  graph: NodeGraph,
  node_id: NodeId
) -> Mode    // derived, not stored
```

This is ~50 lines of substrate. CLI / REST / SQL / audit emissions derive.

---

## §10. First-cut process to migrate

**Recommendation: review-verdict-parser** (today's pain).

**Why**:
- 7 parser-class operator-tier surfaces in ~8h validated the cost of the heuristic-pass model (per `feedback_lenses_not_passes.md`)
- Migration is well-bounded: define `ReviewVerdict = Approve | RequestChanges(findings) | Comment` at reviewer-output boundary; review-tally becomes `latest_per_provider_at_HEAD` lens over `EventLog<ReviewEvent>`
- Self-validating: when parser-lag disappears, the substrate works

**Alternative: the decomposition algebra itself** (the ctrl PRs #1192-#1197).

- Larger scope (replaces an in-flight TS implementation)
- But more strategic: every other ctrl/ migration consumes the algebra
- Requires Phase 3 emission targets (HTTP REST + SQL) before cut-over

**Suggested ordering**: review-verdict-parser FIRST (proves substrate emission patterns; small win), decomposition algebra SECOND (replaces the foundational TS-side implementation).

---

## §11. Open questions

1. **Workflow types dissolution scope** — do we dissolve `IssueLifecycleStage` / `ImplementationTask` / `TrackedIssue` etc. fully into the decomp-algebra, or keep them as gunbc-domain refinements on top? Proposed: dissolve, with the existing types deprecated and removed once consumers migrate. Operator confirm.

2. **Home placement** — `dsl/std/process_algebra.dag` (universal) vs `dsl/ctrl/process_algebra.dag` (application-specific). Operator confirmed compositional split: universal → `std/`. Confirmed.

3. **Phase 2 CLI emission target** — Rust binary replaces bash `dashboard-ops`? Or stays as bash shim wrapping a generated client? Proposed: Rust binary (gunbc's strongest emission target), bash shim only for transition period.

4. **Emission target priority** — HTTP REST first, SQL second, audit-event third? Or different order? Proposed: HTTP REST first (smallest), enables full Phase 4 cut-over without needing SQL emission yet (ctrl/ keeps SQL hand-authored short-term).

5. **Force-override modeling** — `AttestedOverride` as a structured Operation variant (this doc proposes) vs API flag (ctrl PR #1195 implements). Proposed: structured operation; eliminates the special-case "force: true" boolean flag.

6. **Phase 1 substrate landing strategy** — single PR for `dsl/std/process_algebra.dag` skeleton (~50 lines), or stacked PRs per type (Mode → Operation → EventLog → projection)? Proposed: single bundled PR (algebra is one substrate; per `feedback_bundle_workstreams_per_pr.md`).

---

## §12. Cross-references

- ctrl PRs #1192 / #1193 / #1195 / #1197 — decomposition-algebra implementation (TS-side authority until Phase 4 cut-over)
- `dsl/std/algebra.dag:99-320` — algebraic structure precedent (Magma / Semigroup / Monoid / Group / Ring / Field / FreeMonoid)
- `dsl/std/computation.dag:133,192,246` — open-enum + Practice 4 dissolution receipt patterns (`SizeBound` / `CallPattern` / `IterationPrimitive`)
- `dsl/std/effects.dag:71-76` — `EffectShape` closed sum (precedent for `Operation` shape)
- `dsl/std/node.dag:55-196` — compiler-AST Node (recursive-type discipline; carrier reuses pattern, not concrete type)
- `dsl/std/graph.dag:15-30` — `GraphEdge` (precedent for labeled edge types if Gap 7 advances)
- `dsl/gunbc/workflow/types.dag` — pre-existing workflow types (dissolution scope per §4)
- `dsl/gunbc/ci.dag:29-32` — projection-function pattern (`project_github_actions`); concrete implementation in flight via Slice 4
- `docs/v3-modeling-analysis.md:217-229` — Practice 4 dissolution-receipt ledger rule
- `INVARIANTS.md` P1 (single-authority), P2 (illegal-states-unrepresentable), P5 (Pure Bootstrap)
- `MODELING.md` M9 (DFS the concept DAG before defining)
- Memory `feedback_lenses_not_passes.md` — lenses over physics, not heuristic passes
- Memory `feedback_coproduct_dissolution.md` — 4 dissolution patterns
- Memory `feedback_checkpoint_dissolution_default.md` — at C-checkpoints, dissolution is default
- Memory `feedback_construction_over_ratchets.md` — model first, violations dissolve

---

## §13. Validation case study — PR #2745 misread (2026-05-12)

The structural algebra proposed in this doc would have caught the PR #2745 misread at PR-close time, not at PM-audit-time. Walked through:

**Decomposition (implicit, as it stood)**:
- Root: operator intent "T-WAD FULL R3-close"
- → PM scope doc (PR #2744 §0)
- → Criterion 3 "WorkflowRuntime + project_github_actions in gunbc-substrate"
- → WI-2 brief: claim "NEW file `dsl/gunbc/ci_emission.dag` declaring enum + signature"
- → cool-carp-720 worker → PR #2745

**The contradiction**:
- WI-2 brief claim: PR #2745 delivers `dsl/gunbc/ci_emission.dag` with `WorkflowRuntime` + `project_github_actions`
- Actual delivery: PR #2745 created `dsl/extdeps/github/ci.dag` (platform substrate) + modifications to `actions.dag` + comment additions to `dsl/gunbc/ci.dag`. The claimed file `dsl/gunbc/ci_emission.dag` was never authored.

**Walk-back trace (under structural algebra)**:
- Start at PR #2745 leaf node; check parent (WI-2 brief composite node)
- WI-2 brief claim unchanged (meaning intact)
- Composite check: do children's deliveries cover the parent's stated children-set? **NO** — `dsl/gunbc/ci_emission.dag` is listed but missing
- → `COMPOSITE_HAS_OPEN_CHILDREN` would fire at PR-close-event
- → `Replan` operation authored under WI-2 brief; reconcile work-item created

**What the structural algebra catches that prose-driven workflow missed**: file-list-intersection between brief's declared deliverables and PR's actual file changes is a `StructuralLens` projection over `EventLog<NodeOperation>`. Mismatch raises `canCloseNode → COMPOSITE_HAS_OPEN_CHILDREN` automatically. Saves the wrong-claim cycle PM hit (which required Director questioning + grep-discovery + correction relay).

**Mapping to operations**:
- `WalkBackEvent` → sequence of `Replan` operations in the event log
- `StableAncestor` → parent node where `Replan` was authored
- `RootContested` → `Escalate` operation routed to root-asker (operator)
- `force: true` resolution path → `AttestedOverride` operation

---

— Authored by deep-wolf-155 (PM) 2026-05-12 per operator directive for ctrl/ migration scoping. Replaces a prior procedural walk-back draft with this structural-integration scope; validation case study (§13) inlined.
