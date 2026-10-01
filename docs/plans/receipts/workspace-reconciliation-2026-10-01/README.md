# Allocation stack reconciliation

Integration starts from commissioning checkpoint `77557f3717ae2854cb514f69464dc0359b28c7ee` and reconciles `main@c95f906f0ff6f21b00db7edb84ae09c41eff5863` in an isolated worktree. No serving checkout, runner service or host allocation state is changed.

The source conflicts are overlapping declaration moves. Main's canonical `gunbc.floor_demand` owns `floor_execution_requirements`, `floor_isolation_requirement` and `gunbc_internal_fleet_trust_domain`. Their duplicate definitions are removed from this stack's lightweight `fabric_floor_policy`; pricing/grant terms remain there to preserve its acyclic dependency boundary. All affected callers and declaration references use the canonical owner. The generic resource-parameterized reservation path remains intact; only the floor wrapper supplies floor requirements.

The retired auth declaration and its retirement standing remain as authored on main. No retirement check is weakened. The conflicted srv1 sudoers projection must be regenerated from the merged model, not selected from a merge parent.

Compiler build, generated artifacts and qualification results are recorded below as they complete. No commissioning, reservation, VM boot or live acceptance is claimed.

## Completed local checks

The reconciled Rust sources built `gunbc` and `claim_executor` successfully with two build jobs in a 12 GiB/no-swap user scope. Binary SHA-256 values are `71f79c1f82268376fde1a8784b8620ce6a057af0ac72fcc4f20fb505793c4a31` and `08ee6aa4b04917e42210d26ad86b6bd4db91d3d099f3ba1840ec27ad26959b3c` respectively. These binaries were built from the pending merge, before its commit.

The floor-demand entry passed 29 controls and the fabric-isolation entry passed six using byte-identical import-closure snapshots and the rebuilt compiler, each under 6 GiB/no swap.

Main changed `argv_command` to consume `ProgramIdentity`. The owned-file installer now uses the existing `install_program()` authority; the SSH forwarding builder now has an explicitly admitted uncataloged SSH program identity, preserving its prior PATH lookup semantics. Neither command constructor seal is bypassed.

The registry-wide artifact invocation exceeded 12 GiB before writing. The narrower invocation preserved here calls the registry's actual `runner_host_sudoers_content` producer, admitted srv1 deployment and `artifact_path` authority over 689 unchanged dependency modules. It completed successfully (reported pre-entry peak RSS 4,247,480 KiB), regenerated `provisioning/srv1/gunbc-ghrunner.sudoers`, and `visudo -cf` accepted the result. The generated changes remove main's retired srv1-10 runner permissions and derive srv1-13 CPUQuota=400% from the merged model. This is producer qualification, not a passing registry-wide gate.

The reconciled fleet-plan entry passed all 86 assertions under 12 GiB/no swap; reported pre-entry peak RSS was 8,859,460 KiB. Its retained log records the complete assertion names. The separate workflow entry passed its unchanged assertion (pre-entry peak RSS 11,890,780 KiB), and the dispatch/fleet/installer entry passed all four controls (8,929,468 KiB). Total: 29 + 6 + 86 + 1 + 4 = 126 passing controls.

## Regeneration and integrated CI follow-up

Commit-bound required regeneration at `b83a547de305b73d6af3e9583b43c7a00deec0fa` passed all 161 planned/executed/adjudicated outputs with `first_generation_equal=true`. The second-generation check also passed with `fixed_point_equal=true`, referencing that same commit. No compiler mirror needed replacement. Both used 12 GiB/no swap. An earlier fixed-point attempt correctly refused a receipt started before the merge commit; that receipt was replaced by the fresh commit-bound run rather than edited.

CI run 36800627115 reached the unimported-provider gate and refused 15 names across three files. The host-assimilation module gained explicit imports from its existing upsert and temporal-effect authorities (13 names). The two primitive calls now use the already-supported `fields.get(index:)` and `text.ends_with(suffix:)` forms, avoiding bare-name collisions with unrelated corpus declarations. No test-provider import or debt-roster exemption was added. These application-only follow-up edits were present during the compiler fixed-point check; its compiler authority/mirror inputs were unchanged.

The generation-publication entry passed three controls and host-assimilation passed eleven after this repair, each under 6 GiB/no swap. The dispatch entry passed all four controls again after the string-method repair under 12 GiB/no swap (pre-entry peak RSS 8,285,124 KiB). CI's complete floor remains a separate required gate; no local focused result is substituted for it.
