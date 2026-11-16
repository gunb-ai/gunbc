# deploy-artifact-closure stack — working notes (fierce-stag-781)

## Done-when (red control 2)
A DEPLOY whose intent omits an artifact a unit reads REFUSES before any enable leg runs,
proven THROUGH the deploy route (apply_legs `deploy_artifact_closure_gate` at EffectsReady arm).
NOT authorized to merge. Manager crisp-dove-117 merges after external review.

## Topology (linear stack warm-ibex built): exec-extractor ← path-units ← no-read ← roster ← oidc ← broker ← gate
- fold = path-units+no-read+roster+oidc = main + these feature commits: 4023 c076 2b17 | d1a6 b0dd | 6a71 | a124 b153
- broker (#16550) = ebd8 863a f324 096e
- gate (#16554) = b691 891c 80ca 6fea c33e (+ tip 0ccf 4e9a) + ReadKind rename 656b (from validate-gate-cascade)

## My 3 PRs (STACKED, one merge commit each, tree = base+delta, NON-force push)
- #16538 branch session/warm-ibex-158-path-units : tree=main+fold ; parents [884f141d754, origin/main] ; base main
- #16550 branch session/warm-ibex-158-broker-path-prep : tree=#16538head+broker ; parents [3f030179557, A] ; base #16538
- #16554 branch gate-cb8 : tree=#16550head+gate ; parents [4e9a9026c4a, B] ; base #16550
Method: build tree from MAIN's tree, hand-edit only the delta regions (keep main's reviewed content),
then git commit-tree with the two parents, update-ref, push (non-force; FF since first parent = current tip).
NEVER rebase/force-push. NEVER naive merge origin/main.

## KEY FACTS about main (already on main, do NOT re-add)
- read_sets.dag imports service_directive_required_reads_of_unit_file + classifier arms from unit_standing.
- unit_standing HAS: type RequiredDirectiveRead(205), service_directive_required_reads(211), service_directive_required_reads_of_unit_file(271). NO exec_start_binary_of_unit_file/working_directory_of_unit_file (removed by #16535). NO watched_paths_of_unit_file (fold adds).
- deploy_artifact_closure.dag ArtifactReadSet = Declared | NotYetModeled | InstalledArtifact (3 arms, InstalledArtifact ALREADY there). NO NoReadOfRecord (fold adds). ArtifactInventoryRow has InstalledArtifactRow (227).
- emit.dag HAS live_deploy_publication_path_unit_file(999), live_deploy_belt_exit_path_unit_file(1262). NO path_read_paths fns, NO live_deploy_tree_remote_unit_file_for_closure (fold adds).
- apply_legs.dag has NO deploy_artifact_closure_gate (gate adds at EffectsReady=1500: EffectsReady{effects:es}=>deploy_apply_family_plans).
- placement = dag/gunbc/live_deploy/repository_convergence_placement.dag. Has ConvergenceSliceAdmission(682), convergence_slice_admission(705), convergence_oneshot_unit_text(746), convergence_oneshot_rendering(754), convergence_oneshot_text_in (~761, builds ServiceUnitFile inline).

## FOLD deltas (port onto main)
### deploy_artifact_read_sets.dag
- import emit: add live_deploy_belt_event_path_read_paths, live_deploy_publication_path_read_paths, live_deploy_tree_remote_unit_file_for_closure
- import closure: add NoReadOfRecord
- BeltEventPathUnit => Declared{paths: live_deploy_belt_event_path_read_paths(spec)}
- PublicationHelperPathUnit => Declared{paths: live_deploy_publication_path_read_paths(spec)}
- TailscaleServeMapping => NoReadOfRecord  (nullary)
- DeployedTreeRemoteUnit => unit_file_read_set(live_deploy_tree_remote_unit_file_for_closure(spec: spec))
- OidcConfigArtifact => InstalledArtifact
### deploy_artifact_closure.dag
- ArtifactReadSet += `| NoReadOfRecord` (nullary). Also update any total-match over ArtifactReadSet (verdict fns) to handle NoReadOfRecord => Allowed (a declared no-read is satisfied).
- deploy_closure_verdict / unit_closure_verdict / first_missing_artifact / inventory must handle NoReadOfRecord.
### unit_standing.dag
- add watched_paths_of_unit_file(file) -> List<String>  (fold over PathUnitFile DirectoryNotEmpty dirs) — use main's helpers. 6a71 fold-tip text uses list_append(left:, right:). CHECK fold tip for exact.
### emit.dag
- add live_deploy_publication_path_read_paths(spec) = spool_role_dirs(SpoolPublication,...)
- add live_deploy_belt_event_path_read_paths(spec) = spool_role_dirs(SpoolAttemptExit,...)
- refactor both renderers to consume those fns
- add live_deploy_tree_remote_unit_file_for_closure(spec) -> SystemdUnitFile? BUT per review 95814: factor ONE placement helper convergence_oneshot_service_unit_file_if_admitted(spec, oneshot)->SystemdUnitFile? and emit just calls it. emit must NOT match convergence_slice_admission; DROP ConvergenceSliceAdmission/Admitted/InsideServingSlice (+ convergence_slice_admission, gunbc_repository_convergence_slice_unit_name if unused) imports from emit.
### repository_convergence_placement.dag (per 6a71 + review 95814)
- extract convergence_oneshot_service_unit_file(slice, oneshot)->SystemdUnitFile from convergence_oneshot_text_in (which then = serialize(that)).
- add convergence_oneshot_service_unit_file_if_admitted(spec, oneshot)->SystemdUnitFile? doing the single slice_admission match -> Present/None. Use it in convergence_oneshot_rendering admitted arm too (no second construction), keep refusal reason.
- tree_remote_convergence_oneshot stays in emit (imported by placement? no). CHECK who owns tree_remote_convergence_oneshot.
### tailscale_serve_scope.dag (d1a6/b0dd)
- add derived no-read guard from typed service (tailscale serve no-read). See fold tip.
### witness deploy_artifact_closure_witness.dag
- add the_belt_path_unit_reads_its_watched_dirs, the_publication_path_unit_reads_its_watched_dirs, the_tree_remote_read_set_matches_the_emitted_host_text, tailscale derive control, oidc control, etc. Copy from fold tip b1537069271 verbatim, adapted to classifier names.
- live_deploy_emit_witness.dag: ComposeOwned import (b153) — check if main already has it.

## GATE (review 95811 REQUESTS CHANGES) fold into #16554 rebuild
- TWO deciders: production calls deploy_gate / deploy_closure_halts. DELETE old verdict-fold path: unit_closure_verdict, combine_closure_verdicts, deploy_closure_verdict, deploy_gate_decision, first_missing_artifact — AT THE ROOT (gunbc.design_argument::replacement_migration). Point EVERY witness control at deploy_gate/deploy_closure_halts.
- CAUTION: main controls (#16513/#16525/#16535: parent-alone + broker env) use first_missing_artifact/unit_closure_verdict. MIGRATE to gate route keeping assertions (Refused naming artifact => Halt{MissingArtifact} naming it). Don't drop.
- Fix file header (says gate "calls deploy_closure_verdict"). Fix dangling all_halt_causes reference.
- Reader-ordering obligation claimed in comments (closure ~197, apply_legs ~1689) as "filed as a gunbc.integration_concern row" but no such row. FILE it: capability gap naming reader units + tree converger. (manager choice: file it)
- Reply on #16554 citing review 95811 in plain text when I push the gate.

## Evidence: run each changed control per fn:
gunbc run --claim-run --entry <file> --function <fn> --source-root dag --source-root src/v2
(one at a time; NEVER whole-file/claim_batch — OOM). Gate required controls must pass on PR HEAD:
- srv1 PROCEEDS control
- the_deploy_refuses_when_a_read_binary_is_not_installed
- the_deploy_refuses_when_the_tree_converger_is_not_brought_up
- allowed_reads_proceed_and_missing_read_halts
- unclosed_reads_halt_the_deploy_gate_never_proceeds
Seed binary at /tmp/fierce-stag-781-target/release/gunbc (build j4a880016). Seed needs main>=1ea81b14.

## Seed traps: `if x == Variant {` parses as record literal (use match). list_push(xs:,item:) append one. list_cover = set-equality.

## STATUS
- [x] fold #16538 built + 12 controls PASS + head 138f161bc PUSHED to session/warm-ibex-158-path-units + retitled/commented; #16541/#16544/#16546 CLOSED pointing at it. head sent to manager.
  - local branch fs781-fold = 138f161bc (parents [884f141d754, origin/main=8d4210c4524]). working branch fold-build.
- [ ] broker #16550 built + controls + head  (build on fs781-fold; 3 files: host_effect_realize(needs main-merge, main deleted 18 lines), release_locus(clean), approval_broker_tree_install_witness(clean))
  - FINAL broker: ReleaseLocus{revision,spine,locus} (NO base_root, f324); release_locus_record(base_root,rev); helper_release_locus_record(locus,spine,rev) (863a); one approval_broker_tree_install(owner,locus,source_root,executables,transport); dark_helper_snapshot_at/verify_at take tree:ReleaseLocus (096e); board_function nets out (NOT changed).
- [ ] gate #16554 built + 5 controls + head + reply review 95811 (build on broker; review 95811 rewrite of closure deciders + apply_legs gate + reader-ordering row)
- GATE branch = gate-cb8 (remote tip 4e9a9026c4a). Its commits: b691 891c 80ca 6fea c33e 0ccf 4e9a + 656b(ReadKind rename from validate-gate-cascade).
- STACKED: #16550 base = #16538 branch (fs781-fold). #16554 base = #16550 branch. So each head self-contained/compiles.

## SEED: /tmp/fierce-stag-781-target/release/gunbc (built from 1ea81b14). Controls ~3min each (reconcile+resolve). Run one file's controls sequentially in ONE bash (safe).
