# Generated-file conflict policy

**Ruled 2026-08-01.** Existing `.dag` artifact, storage, commit and Git authorities own integration policy. `.gitattributes` and repository-local Git config project it; Git remains a compatibility realization of [native integration](dag-scm-design.md).

The motivating failures were redundant projection conflicts, auto-heal `218dea61248d` committing conflict blocks in `publication_grant.dag`, and clean overlapping appends (#7565/#7580) producing eleven duplicate paths. The last case requires construction semantics, not conflict handling. [Original analysis and receipts](https://github.com/gunb-ai/gunbc/blob/3e5a0f1f0dbc62e0d3589df44146858d3147c9e9/docs/plans/generated-file-conflict-policy.md) preserves measurements and lane history.

## Policy and authority

| Subject | Integration | Before commit |
|---|---|---|
| Deterministic projection | Either side provisional; no manual byte resolution | Regenerate merged authority; prove byte equality |
| Hand-authored semantic authority | Resolve semantic choices | Normalize index |
| Keyed set/map authority | Neither take-ours nor line merge decides | Refuse duplicate keys before construction |
| Roster of generated outputs | Derive from producer; delete duplicate roster | Exact produced/observed membership agreement |

Join `gunbc.generated_artifact`, `v2.std.artifact` and derived ModuleStorageBinding; do not mint a universal FileKind or path census. Provenance, merge, line-ending, diff and Linguist presentation are independent axes. A total path model may emit sparse nondefault attributes; generated does not imply hidden.

## Lane 1: every commit writer consumes admission

Human, dashboard, heal and integration writers share one predicate at the actual write boundary. A generated pre-commit hook is optional feedback, never enforcement; infra bindings are countable work requiring operator routing.

Refuse any non-normal `extdeps.git.object_store.GitIndexStage`, carrying located path/stages. Also inspect the exact staged blobs for complete conflict-marker grammar: `git add` can normalize the index while retaining markers. An isolated token or repository-wide grep is insufficient. Test exemptions require typed provenance.

Admission consumes the COMPLETE staged-index observation, with independent refusals for missing/extra observations, duplicates, index/blob identity disagreement, content not matching the indexed object, and unavailable decode/classification. Text classification is observed, not caller-selected. A fixture exemption is receipted from tracked artifact/module-storage authority and bound to the same path/blob; a freely constructed Artifact is not evidence.

REDs reproduce `218dea6` both as unmerged index and staged marker content; clean input admits. Also refuse source paired with test provenance, provenance/staged-path mismatch and a fabricated artifact with matching path. Heal and dashboard must consume this wall, not merely mention it.

## Lane 2: policy projections and clone realization

GitattributesArtifact projects only existing policy: the `.dag` language presentation row is independent; generated merge rows derive from committed deterministic projections, including `.gitattributes` itself without a special self-row. The driver definition is repo-local desired Git state. Model its config and core.hooksPath, reconcile idempotently, then read back; absence, wrong value and read failure have distinct typed refusals. A manual config command does not discharge this.

The transaction is: integrate authored authorities without committing → refuse unmerged index → regenerate affected committed projections → prove equality to indexed bytes → required admission → commit. A take-ours/driver=true merge is provisional transport convenience. The writer MUST consume the regeneration receipt before writing.

Config converges before each clone's FIRST Git consumer, including heal's first merge. Acceptance exercises a fresh clone with absent config, convergence, a real projection conflict, provisional resolution, regeneration and equality. Deferred session/developer clones need a countable rollout carrier with population/converged/missing counts and retirement when every clone-creation path invokes convergence.

Stage0 policy must join this population with collision refusal, not remain a follow-up named only in prose; the landed pure realization below supersedes the originally proposed live path-list join.

## Lane 3: stage0 projection coverage

The handwritten generated_stage0_files roster is deleted. The planned stage0_emit_plan route died in #8406. Its replacement, `gunbc.stage0.stage0_rust_source_lifecycle_scaffold.derived_generated_stage0_repo_paths`, joins emitted population and tracked files at a revision, but live git observations cannot determine a pure committed projection: clones at one commit must not disagree.

The landed merge-policy producer is `gunbc.generated_projection_paths.deterministic_generated_projection_merge_rows`: cover direct children of the stage0 source root, then unset `v2.compiler.self_host.stage0_crate_layout.hand_maintained_stage0_filenames` members not themselves registry artifacts. Last matching attribute wins, so exceptions MUST follow the glob. This covers newly emitted files without another roster; the former prose claim that the live derivation already fed attributes was false.

Recovery uses `claim_executor --required-regen` and generated-artifact registry `gunbc.instruments.generated_artifact_gate.main_wet`; `regen_stage0` was deleted. Take either provisional side, regenerate merged authority and verify zero divergence.

Unavailable/Refused source supply must remain refusing through emission, artifact generation and writer. No non-host or test-helper `Refused => []` fallback may yield writable bytes. Canonical OUTPUT paths, not merely module identities, require unique keys and located collision refusals because flattening can collide. The merge-policy consumer must actually cover this output population.

## Lane 4: live keyed construction

A keyed roster refuses repeats before a value exists, at per-PR admission. Consumers accept only Built from a typed Built|Refused outcome and preserve located key/first/duplicate evidence; a Bool frontier discards the required refusal. Optional line-union transport cannot establish uniqueness.

Generic kernel tests alone do not close this lane. Exercise a real roster end to end and the exact #7565/#7580 overlap. `public_file_publish_grants` was deleted by operator ruling #7591 and its re-land frozen for design review; keep incident fixtures and the lane open until a live carrier consumes the construction. Any re-land must satisfy set semantics, subtree grain and server-side admission.

## Order and rejection bars

The charter precedes implementation: lane 1 with complete observation/bound provenance → lane 2 with pre-consumer convergence → lane 3 stacked on lane 2 with refusal-preserving generation → lane 4, integrating commit_workflow semantically after lane 1.

Reject a second generated-path roster, handwritten attribute policy, marker grep as sole check, hooks claimed as enforcement, take-ours on authored policy, config mutation without desired state/readback, or commits before regeneration/equality. Merge-freshness against changed authority is separate merge-admission work.
