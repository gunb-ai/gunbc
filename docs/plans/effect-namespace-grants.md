# Effect grants over namespaces

**Design and ruling record.** The 2026-07-16 proposal separates input closure, output reach, handler binding and selection eligibility formerly conflated by Hermetic/Wet. The plan owns remaining decisions below; implementation standing comes from its named carriers. [Original analysis and receipts](https://github.com/gunb-ai/gunbc/blob/3e5a0f1f0dbc62e0d3589df44146858d3147c9e9/docs/plans/effect-namespace-grants.md) preserves dated PR/review receipts and handoff history.

## Contract

An effect target is a position in an existing containment tree. A frame grants a verb over a subtree, with a handler binding. Reuse filesystem/URI/proc/service/code namespaces and their prefix relation; do not invent a parallel scope tree. The interim path realization must converge with the naming index when it lands.

Grant = verb (Read/Write/Execute) × NamespacePosition root × (RealTransport or Replay fixture store). Execute derives from ExecuteEffect, as ruled after deploy-preflight supplied its consumer (2026-07-19); Create remains deferred. `std.materialization_ladder.Frame` owns nesting, lifetime and re-entry; child envelopes cannot exceed parent grants. Dispatch outside the covering verb/root refuses typed, located and counted EffectOutsideGrant. Diagnostic modes never widen permission.

Derive replayability from content-addressed or Replay-bound Reads; isolation from frame-owned scratch Writes; live-tree disposition from Read/repo-tree intersection. Hermetic/Wet/Record are temporary envelope presets, removed when every reader migrates. Grants reference effect shapes, never fork their semantics.

Frame containment requires BOTH target-under-controlled-namespace and target-lifecycle-within-frame. Preserve the grade: LifecycleByConstruction makes survival impossible; LifecycleByConvention depends on fallible cleanup. Netns-scoped loopback and ephemeral-container writes can be contained by construction; `/tmp` plus trap cleanup is convention only; BMC POSTs and repo writes are not contained. Quantitative budgets stay with CostAccount, identity stays distinct from reach, and mutable keyed state remains a database.

## Existing homes and sequence

Reuse `std.resources.ResourceHandle` for the capability convergence, `std.materialization_ladder.Frame`, existing replay fixtures, typed refusal discipline, and external resource trees. The Rust workspace_root path refusal is a proto-grant to dissolve. No new resource-kind enum or independent tree walk.

Unify operation semantics before layering grants: the 2026-07-17 ruling chooses `std.effects` for EffectShape/CreateCause and duplicate key/idempotency predicates; key/target KeySource is minimally moved then grounded on the containment tree. Do not invent a String bridge at the effects seam: its kernel String versus v2 text realization split resolves through staged key grounding. Keep nature, reach, target, idempotency and binding distinct. Faithfully modeled but unconsumed arms need named dissolution triggers.

1. P-A: model envelopes over Frame, derived axes and REDs (outside target; child exceeding parent), without behavior change; mark temporary mode presets.
2. P-B: enforce at Filesystem dispatch and derive workspace containment from grants; declare artifact-store scratch roots explicitly.
3. P-C: derive LiveTreeDisposition and select handlers per grant.
4. P-D: build admissibility becomes an envelope, retiring FLAG A as a global exception.

Every phase needs executed green/RED evidence; under-scope remains a counted frontier.

## Build envelope and enforcement

`v2.compiler.build_workspace_grant.build_workspace_grant_envelope` is the single build authority: Read substrate closure, pinned/version-keyed toolchain and own workspace; Write own workspace; no network grant. Workspace ownership includes reading generated sources/intermediates. Its scaffold trigger is enforcement through P-B, not a permanent mode exemption.

`v2.compiler.build_transport_admission.build_transport_admissible` projects descriptor reads/writes to positions and returns Admitted or located EffectOutsideGrant. `v2.compiler.emit_host.run_host_process_with_cache` consumes it BEFORE intrinsic dispatch; refused transports yield `^emit_host_transport_outside_grant` and no call. Check invocation arguments as well as program paths. Containment is segment-based: `..` escapes, `foo..bar` does not (reviews 39707/39684). Compiler placement covers every caller; concrete per-run roots remain workflow policy.

The named witnesses cover escaped paths/arguments, scoped writes, workspace reads, closure-versus-live-tree reads and the production dispatch RED. Three related write seams consume modeled grants: v2 effect_io_host, extdeps artifact_store_fs and cli_run repo_relative_path (`gunbc.cli_run_repo_grant`; Rust remains HAND-RUST until Chunk F).

An absent network grant alone is NOT enforcement. The seed's blanket hermetic host-execution refusal remains until a real network-isolation realization exists. The operator chose fail-closed Option 1: native hermetic execution requires LifecycleByConstruction and refuses the in-container case pending its target-environment trigger; netns/offline/sandbox must enforce the declared bound before relaxing the intrinsic.

## Remaining convergence and decisions

`std.effect_axes` owns derived regions/projections (#6987): unclassified reads count as live; Replay-bound repo reads are replayable and not live. `std.effect_grant.covering_grant` returns the actual covering grant, with admit_effect its reach projection; v2 effect_io_host selects RealTransport/Replay from that grant, not a second process-mode decision. The dated #7071 review state is historical, not a current merge claim.

The operator-approved 2026-07-22 residuals remain explicit: Replay's fixture_store String still needs per-grant resolution rather than the separately supplied EffectIoFixtureStore; seed eval_service_op retains mode-based transport on its own dissolution frontier. Stamp retirement needs the unresolved choice between per-witness envelope reach and std.live_tree G2 runtime-read closure; do not silently pick one because the documents disagree. Remove both v2 presets and seed std.execution_mode only after all readers and stamps migrate.

Open operator decisions: approve interim path-string-to-index convergence; decide AuthScope convergence only where a real reach-encoding consumer names its cost (identity-side for P-A). Q1 was amended to Execute; Q4 assigned one document home and is settled.

Reference/Publish verbs and AudienceScopeTree were an experiment, deleted by the 2026-08-01 ruling with std.visibility_grant. This retires their carrier, not publication capability. No replacement publication ontology is selected pending design review; reference-edge capability independently awaits a concrete consumer. See [original ruling and experiment references](https://github.com/gunb-ai/gunbc/blob/3e5a0f1f0dbc62e0d3589df44146858d3147c9e9/docs/plans/effect-namespace-grants.md).
