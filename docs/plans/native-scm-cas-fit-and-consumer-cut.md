# Native SCM: CAS semantic fit and first consumer cut

Phase 0 ruling, 2026-09-13. This records the dependency contract for Phase 1; it declares no SCM carriers and implements no transition. The consumer is the Phase 1 implementation and its composition evidence. This record remains the semantic boundary for subsequent authoring and landing routes.

## Shared authority and payload fit

The leasing/locking home is `std.durable_compare_and_set`, realized by `gunbc.durable_cas_file_store::file_compare_and_set`. `CasOutcome<T>` needs no additional arm for stale generation, absent target, or unexpectedly occupied target:

- `CasPreconditionFailed { expected: ExpectSlotGeneration { generation }, observed: CasReadableAbsent }` preserves a missing expected slot.
- `CasPreconditionFailed { expected: ExpectSlotAbsent, observed: CasReadablePresent { version } }` preserves an unexpectedly occupied slot.
- `CasPreconditionFailed { expected: ExpectSlotGeneration { generation }, observed: CasReadablePresent { version } }` preserves a generation disagreement. `CasSlotVersion` carries the observed generation, content hash, and value.

The successful arm retains the committed version. `CasStoreRefused` retains `CasUnreadableSlot`, including malformed state, missing referenced content, and refused reading. `cas_decide` preserves the readable/unreadable distinction before evaluating the expectation. No SCM-local synonym sum may replace this authority.

Exact native parent identity and slot generation are different facts. A generation expectation alone does not identify the native parent: the consumer must bind the proposed native transition to the parent held by the observed slot version. Git SHA, PR number, and CAS generation are not native commit identity.

## Realization findings: payload sufficiency is not execution sufficiency

Inspection of `file_compare_and_set` confirms that its initial eligibility branches preserve all three precondition cases above. `cas_probe_as_readable` supplies generation, recomputed content hash, and value for `ProbedHead`.

The realization does **not** preserve the full refusal partition on every path:

1. `cas_probe_from` treats every unsuccessful `Filesystem.Read` as the chain end. Permission or I/O refusal can therefore become absence or an earlier apparent head. Its comment names typed read failure as unavailable, but `extdeps.filesystem.filesystem_io::Filesystem.Read` now exposes `error_kind`, and `filesystem_exact_read` distinguishes absent, unreadable, and unrecognized failure kind. The adoption capability exists; this consumer has not adopted it.
2. `cas_commit_at` maps every unsuccessful `Filesystem.WriteOwnerOnly` to `CasPreconditionFailed`. It does not establish that another writer won. An operational write refusal can therefore be mislabeled as a precondition failure, even when the re-observed state still satisfies the expectation.
3. That same failed-write branch calls `cas_probe_as_readable`, whose `ProbedBeyondBound` arm deliberately returns `CasReadableAbsent` for its documented reporting use. The defect is consuming that projection to construct a decision payload without establishing a lost race. In contrast, the initial `file_compare_and_set` branch and `observe_cas_slot_state` preserve a bound refusal as `CasUnreadableMalformed`.

These are defects in consumption of the existing authority, not evidence for a fourth CAS outcome. Phase 1 must repair the realization at its existing home and preserve transport refusal through the shared outcome. `WriteOwnerOnly` currently omits the host `error_kind` output that `Read` and `WriteCreateNew` expose; its consumer must obtain typed write classification without parsing error prose.

No new wet execution was performed for this ruling. The findings above are source-contract findings, not a claimed rung or proof of durable publication. Exclusive creation must not be confused with atomic visibility of complete bytes or crash durability. The Phase 1 composition evidence must establish those publication properties before a mutable slot may name an immutable object.

The transport distinction is concrete: `v1_compiler.v1_interpreter::write_file_owner_only` opens the final path exclusively and then writes its content. A post-open failure can leave partial bytes at the slot path. The adjacent create-new contract explicitly treats owner-only exclusion as incidental to permissions, whereas `extdeps.filesystem.filesystem_io::Filesystem.WriteCreateNew` promises create-only publication and its realization publishes complete content through a sibling file. Phase 1 must bind the CAS realization to contractual exclusive, complete publication and establish durability; correcting error classification alone is insufficient. The transport realization home is `extdeps.filesystem.rust_realization`, not an SCM-specific filesystem algorithm.

## Read-back is a separate relation

Independent read-back consumes a committed receipt and observes the same target. It cannot rewrite `CasCommitted` into a claim that the CAS did not commit. Store refusal, precondition failure, committed-with-read-back-unavailable, and committed-with-read-back-disagreement remain distinguishable. Only the last two require composition over a committed receipt; neither extends or replaces `CasOutcome`.

A later target generation also cannot erase the receipt: read-back must distinguish subsequent advancement from disagreement about the committed version. A verified transition requires evidence bound to the exact committed subject. A caller-authored expected value is not independent observation.

## Exact Phase 1 dependency and consumer cut

Existing producer symbols to consume:

Before either SCM slot consumer relies on the store, repair these prerequisites in order:

1. Bind CAS to contractual exclusive, complete, durable publication at the existing homes `extdeps.filesystem.rust_realization` and `gunbc.durable_cas_file_store`. The finding is `gunbc.recurring_failure_mode.consumer_relies_on_incidental_realization_property::consumer_relies_on_incidental_realization_property`. An SCM-local safe-write helper would fork the existing authority. A correctly classified refusal over torn slot bytes is still unsafe, so this prerequisite comes first.
2. Repair refusal-as-contention and unreadable-as-absence at `gunbc.durable_cas_file_store`, consuming the existing typed filesystem observations. The finding is `gunbc.recurring_failure_mode.store_refusal_reported_as_contention::store_refusal_reported_as_contention`.

These are source-verified readings of contract mismatches, not executed reproductions. The repair lane owes the discriminating REDs. Both defects affect the existing compute control-plane consumer; absence of an SCM caller does not make them prospective.

- `std.durable_compare_and_set::{CasAttempt, CasExpectation, CasGeneration, CasSlotVersion, CasOutcome, cas_attempt, cas_decide}`: the shared conditional-publication vocabulary and decision.
- `gunbc.durable_cas_file_store::{admit_cas_attempt, VerifiedCasAttempt, file_compare_and_set, observe_cas_slot_state}`: verified attempt admission, store-owned conditional publication, and independent slot observation. `VerifiedCasAttempt` verifies the encoded payload/hash pair; it does not verify the transitive SCM objects named by that payload.
- `extdeps.filesystem.filesystem_io::{filesystem_exact_read, filesystem_create_new}`: existing typed transport-result classification. Upstream filesystem facts remain at that authority, conforming to `extdeps.external_authority::ExternalModelScope`; SCM policy belongs downstream.
- `gunbc.scm.object_store::{ScmObject, CorpusManifestObjectRef}` and `gunbc.scm.ancestry::RepositoryCommitRef`: existing SCM object and commit reference subjects. Their current locator semantics must not be silently promoted into durable cross-party content identity.

The first consumer carriers to be declared **in Phase 1**, not here, are a repository-target generation slot and a workspace-scoped stage/base slot. These are semantic names for the cut, not claims that declarations already exist.

The repository-target slot binds one repository and target to its native parent/commit reference and CAS version. Its conditional advance consumes the exact observed generation and a candidate bound to that parent. The workspace slot binds one independently mutable workspace to its selected base and staged manifest, with its own CAS version. Two workspaces share immutable objects and repository targets, but never one mutable stage/base slot. Workspace identity is independent of author identity.

For each slot, the consuming route is immutable object put-if-absent publication, durable independent verification of the referenced closure, slot payload encoding, `admit_cas_attempt`, `file_compare_and_set`, independent read-back, then composed transition standing. Any unavailable capability refuses at its own boundary. The first two steps cannot be substituted with an in-memory object-table insertion.

`gunbc.scm.repository_envelope::RepositoryEnvelope` currently carries one document's stage selection; it is not the multi-workspace slot authority. `gunbc.scm.repository_save::save_repository` persists a checked document unconditionally and must not become the conditional publisher. Phase 1 consumes the shared CAS home instead of cloning it or hiding mutable workspace selection inside that document.

The cut closes only when wet evidence executes the entire object-to-verification-to-CAS-to-read-back composition for both slot subjects, including contention and independent workspaces. Re-proving the isolated linearizer, declaring these carriers, or landing a ticket does not satisfy that trigger.

## Mandatory Phase 1 provider gate

2A, publication integrity, precedes 2B, outcome fidelity. Success means one exclusively published final generation containing the complete proposed value. Failure before publication means no final generation path ever became visible. An occupied target leaves its existing complete generation untouched. `WriteCreateNew` is the nearest surviving construction, but fsync of a private staging file does **not** establish durability of the final directory entry: either the contract and implementation cover the publication metadata, or the claimed durability rung stays lower.

For 2B, real occupancy or a genuine race yields `CasPreconditionFailed` with the re-observed winner. Permission, storage, IO, staging, sync, or publication failure yields `CasStoreRefused`. Error prose is never parsed to choose between them. Finding no readable winner on a second observation cannot widen a store refusal into contention.

The repair lane owes these wet controls:

- Inject failure after staging writes begin: no final generation exists.
- Inject failure before publication completes: no committed CAS outcome is returned.
- Race two eligible writers: exactly one commits; the other reports precondition failure naming the observed winner.
- Cause a non-contention store refusal: preserve `CasStoreRefused`, never precondition failure.
- Restart and independently read back: exact generation, content identity, and value agree.
- Mutate the realization back to direct final-path open-then-write: the partial-publication control turns red.
- Mutate the fold to classify every failed publication as contention: the store-refusal control turns red.

Phase 1 may not declare the CAS realization suitable for native SCM until **both 2A and 2B pass their wet controls**. This gate is a prerequisite to SCM consumption, not an assumption its tests may make. The three discovery records remain source-verified findings, not executed closures.
