# HANDOFF — belt credential-recovery receipt (node `credential-recovery-receipt-under-declared-instance-dir`)
Worker: wise-crane-498. Manager: wise-lark-694. Authority: Manager B (loyal-swift-572).
Purpose: complete enough that a fresh worker or the manager can execute the remaining steps WITHOUT me.

## 0. STATE / SCOPE
- Node id: `credential-recovery-receipt-under-declared-instance-dir` (in `dag/gunbc/roadmap/roadmap_authority.dag`, row at line 10144; handback at 10154).
- Fix PR **#17021** (production: receipt path via `belt_credential_recovery_receipt_path` -> `instance_receipts_dir(instance_root)` = `/opt/gunbc/receipts/credential-acquisition-recovery.json`) = MERGED.
- Planning PR **#17046** (rewrote the row's first_slice/red_control/out_of_scope/handback; deleted permanent-fail `old_path_witness.dag`; kept `dag/test/claim/belt_credential_recovery_receipt_home_witness.dag` as the single discriminator) = LANDED at **5611ac47e730427f6d19f9bf765c561b01c36de7** (= current main at handoff time). Final head e01ed87b3e93afba138b8d114abbe1313d094b0a; review 98191 APPROVE; B countersigned.
- G0 check: PASS — the fix touches only the receipt LOCATION + directory declaration; enables no belt tick, drain, or attempt launcher.
- **OPEN (gated, out of my hands):** host half — `/opt/gunbc/receipts/...json` written AND unit exit 0 — waits on the AUTHORIZED deploy-route apply (operator broker approval, manager-routed). Host NOT yet deployed: deployed model is `44808cbefd4d6dd464e0397b20102e2be3d6558c` (mtime 2026-11-24 05:49), pre-fix; live timer run 05:13:14Z->05:18:57Z still fails (see §3). I perform NO host writes.
- **Recording stays CLOSED**: no recorder run, no heal re-trigger.

## 1. PRE-STAGED RECEIPT — NOT COMPUTED (provenance gate). Hand off RECIPE instead.
Per manager order (msg_b80272cc): the pre-staged `roadmap_criteria_digest` is **NOT** computed here, because the only digest binary available has an **unrecorded built-from commit** and the fnv1a64 algorithm lives IN the binary (`src/v1/stage0/src/std_content_hash.rs`) — so a computed digest would inherit the provenance gap B flagged. Compute it ONLY via the §4 single-pass recipe on a SELF-BUILT gunbc with printed built-from HEAD.
- Target value: `roadmap_model::node_criteria_digest(rn)` — tag `roadmap-acceptance-criteria-v1`, digests **brief + red_control + handback** only (NOT first_slice/out_of_scope). Render `Fnv1a64Structural` as `.digest as String` (16-char lowercase hex).
- Row text is read from the tree via `roadmap_authority::declared_roadmap_nodes() -> List<RoadmapNode>` (roadmap_authority.dag:809), find the node by id, call `node_criteria_digest`.
- Label: **PRE-STAGED / ACCEPT-PENDING** (host half open until authorized apply). Author principal left BLANK.
- Receipt must carry: node id; both PRs + heads (#17021 head; #17046 head e01ed87 landed 5611ac47); evidence pointers (§ below); the criteria digest + the gunbc binary sha256 + **built-from commit printed**; controls_ok baseline.
- **PENDING-PLANNING-PR CHECK (must re-run before labelling):** any OPEN PR that edits THIS row's brief/red_control/handback flips the label to **PENDING <PR#>**. I executed it (result below): `gh pr diff <n> --repo gunb-ai/gunbc | grep -c credential-recovery-receipt-under-declared-instance-dir` over the open roadmap-touching PRs.
  - RESULT at handoff: PRs 17071 / 16635 / 16988 / 15574 / 15602 — **0 node-id hits each** (17071/16635/16988 touch roadmap_authority.dag only for their OWN rows; branched before my merge so they don't contain my row). => No open PR changes this row => label stays **PRE-STAGED/ACCEPT-PENDING**, NOT PENDING. Re-check at execution time (new PRs may open).

PRIOR (unproven-binary) values, for context ONLY, superseded by §4:
- node-digest TOOL full-row digest (NOT criteria_digest), via nd-crane+gunbc, controls_ok=yes (master.log): BEFORE(before-lab@07b3ec16)=`3445edc63a0cf995` rc0; AFTER(green@e01ed87)=`8e305b17f44c5755` rc0; REDAT(redat@56d460)=`MISSING cause: no_such_roadmap_node_id` rc1. Binary built-from unrecorded -> re-run via §4 for authoritative values.
- criteria_digest: NEVER computed.

## 2. SRV1 PATHS / BINARIES / LOGS (all read-only artifacts)
Trees (all `git clone` from mirror `/home/briansrls/gunbc`, except main which is a `git worktree` of `/home/briansrls/gunbc`, origin https://github.com/gunb-ai/gunbc):
- `~/scratch/wise-crane-498-green`  HEAD e01ed87b3e93... (tree 88d8a005...) — the FIX head.
- `~/scratch/wise-crane-498-redat`  HEAD 56d460116d6252f6ec8dfe889ea358c804685861 (tree 9096caef...) — red_at (pre-fix).
- `~/scratch/before-lab`            HEAD 07b3ec1635f591253ac2f2614352dc6c23f13c95 (tree 6b1be53f...) — "BEFORE" row-at-rest tree (its roadmap row set to match main).
- `~/scratch/wise-crane-498-main`   HEAD 5611ac47e730427f6d19f9bf765c561b01c36de7 (tree 57f1f89b...) — current main. NO tree named "regen" exists; before-lab is the BEFORE-role tree.
Logs:
- `/tmp/master.log` (32868 B) — AUTHORITATIVE re-run set at the cited heads.
- Superseded (earlier heads 923fed99aa2a / b7e0c37): `/tmp/three_run2.log`, `/tmp/three_run.log`. Also `/tmp/nd_before.log`, `/tmp/nd_after.log` (earlier digest attempts; nd_after ran at LAB_HEAD=570ca5e5, superseded by master AFTER at e01ed87).
- Host RED archive: `~/scratch/wise-crane-498/host_red_0513_14Z.txt` sha256 `8140b5e66d336f6520c8f9cae32ec7c4ace886af0b36b889254730880ab518c9` (BOOT c095c0cd).
Binaries:
- claim_batch `~/tmp-wise-crane-498-claim_batch` sha256 `8ab7bd7753448ca5c49463f2aae2d62f913c9a02f95d8b3740a4e5be5824cf1d`. FOREIGN: built by session merry-deer-252 2026-11-23T01:29:33Z from `~/scratch/merry-deer-252/gunbc` via `cargo build --release --bin claim_batch` (see `~/build_cb.log`); built-from COMMIT UNRECORDED; predates #17021. DO NOT rely on it for provenance — self-build per §4.
- node-digest wrapper `~/scratch/nd-run/nd-crane` (2114 B shell) sha256 `360e79e42925f73a87b18848441176fccf7f2808ceeb02b5150f09f01d929859`. Uses env `NDC_LAB` (tree), `NDC_GUNBC` (gunbc binary; default `/home/briansrls/shared/node-digest/bin/gunbc`), `NDC_REV` (required LAB_HEAD). Emits `HOST=/BOOT=/LAB=/LAB_HEAD=/GUNBC_SHA256=`, runs control/control-third/missing probes then the target `probe(id:)` per node. See §5.
- probe template `~/scratch/nd-run/probe_template.dag` (module `probe_node_digest_batch`, fn `probe(id:)`); a patched variant also at `~/scratch/nd-crane-498/node-digest-patched.sh` (1461 B).
- relocated gunbc used by nd-crane: `GUNBC_SHA256=82e9a74a134dbd6715231e914a2069cfd0b4e7154b5f63665831378cfe89d876` (RELOCATED COPY, NOT the shared gate at `/home/briansrls/shared/node-digest/bin/gunbc`; built-from commit unrecorded — per B, print BOTH shas and label "relocated copy, not the shared gate").
- parse binary `~/scratch/wise-crane-498-main/target/release/v1_src_dag_parse` sha256 `278e44543e7367fa3998ea6d9eeff374f37e92dd3acc51d1c1c4c6e11a4522dd` — SELF-BUILT from current main (BUILD_RC 0); reproduction handoff for the 212-citation-diagnostics finding already sent to silent-swift-73.

## 3. HOST-HALF OBSERVATION (read-only ONLY; run ONLY after the authorized apply)
ssh form: `ssh -n -o BatchMode=yes -o StrictHostKeyChecking=no -o HostName=srv1.tailecbe08.ts.net briansrls@srv1.tailecbe08.ts.net '<cmd>'` (fallback `-F /dev/null -i ~/.node-orch/id_ed25519 -o IdentitiesOnly=yes`, never -F /dev/null alone). NO writes, no systemctl start/stop, no service touches.
Commands (single observation window, not per-tick):
- `systemctl show gunbc-roadmap-belt-credential-recovery.service -p ActiveState -p SubState -p Result -p ExecMainStatus -p ExecMainExitTimestamp -p NRestarts`
- `systemctl status gunbc-roadmap-belt-credential-recovery.service --no-pager`
- `ls -la /opt/gunbc/receipts/credential-acquisition-recovery.json`  and read it: `cat /opt/gunbc/receipts/credential-acquisition-recovery.json`
- `journalctl -u gunbc-roadmap-belt-credential-recovery.service --since "<run start>" --until "<run start + ~8min>" --no-pager`
**EXPECTED-GREEN (B's definition — BOTH required):** (i) unit reaches **exit 0** (`ExecMainStatus=0`, `Result=success`, not `failed/exit-code`); AND (ii) receipt present at `/opt/gunbc/receipts/credential-acquisition-recovery.json`. If exit 0 but the receipt describes a FAILED recovery, that alone is not the host half — read the content. If the unit then fails for another reason, the host half is NOT met: report the journal line, do NOT narrow the row.
Reference (deployed-PRE-fix state, host_red archive): run 05:13:14Z->05:18:57Z, `ExecMainStatus=1 Result=exit-code ActiveState=failed`; journal `Filesystem.Write /opt/gunbc/credential-acquisition-recovery.json (115 bytes)` + `belt_credential_recovery_cli: receipt write failed`; recovery payload succeeded; both receipt paths ABSENT.
Separate, OUT OF SCOPE: lab unit `gunbc-srv1-lab-codex-feedback-v0-belt-credential-recovery.service` (instance_root=/opt/gunbc-lab/codex-provider-feedback-v0) writes its receipt to a lab dir already; any other lab failure is not this row.

## 4. SINGLE-PASS RE-RUN RECIPE (authoritative; fixes the §c binary gap)
Prereqs: heavy runs via `HEAVY_OWNER=wise-crane-498 ~/bin/heavy-slot bash -lc '<cmd>'` (strict FIFO ~/.heavy-slots; if not acquired, report QUEUE CONTENTS not "saturation"). Real rc via `echo $?` into a file, never a pipe's rc. srv1 boot_id prefix c095c0cd. claim_batch home-witness resolve ~25-35 min.
STEP A — SELF-BUILD both binaries from a PRINTED HEAD, tree clean:
```
cd <tree at target HEAD>          # e.g. fresh clone of current main
git rev-parse HEAD                # print built-from HEAD (REQUIRED in build log)
git status --porcelain            # MUST be empty before building; if not, STOP
BUILD_FROM=$(git rev-parse HEAD)
HEAVY_OWNER=wise-crane-498 ~/bin/heavy-slot bash -lc 'cd <tree> && cargo build --release --bin gunbc --bin claim_batch 2>&1 | tail -5; echo BUILD_RC=$?'
sha256sum target/release/gunbc target/release/claim_batch    # record + built-from=$BUILD_FROM
```
STEP B — pre-stage criteria digest over 5611ac47 row text, using the SELF-BUILT gunbc via nd-crane (NDC_GUNBC=.../target/release/gunbc, NDC_LAB=wise-crane-498-main, NDC_REV=5611ac47e730...): record `LAB_HEAD=`, `BOOT=`, gunbc sha + built-from, controls_ok=yes, then the criteria-digest value (or add a tiny probe calling `node_criteria_digest` on the node). PRE-STAGED/ACCEPT-PENDING, author blank.
STEP C — the four audit pairs; for EACH: print `TREE_HEAD=$(git -C <tree> rev-parse HEAD)` + `BOOT=` + `BIN_SHA256=` (the SELF-BUILT binary) + heavy-slot acquisition line IMMEDIATELY before the run, and `echo rc>file` for the real rc:
1. GREEN home witness @ e01ed87 (`~/scratch/wise-crane-498-green`): `~/.../claim_batch --source-root dag --source-root src/v2 --entry dag/test/claim/belt_credential_recovery_receipt_home_witness.dag --functions the_receipt_sits_in_the_instance_receipts_directory,the_receipt_path_is_the_expected_bytes,the_receipt_is_not_homed_in_the_credential_acquisition_slots,the_receipt_path_contains_no_traversal`. Expect 4x PASS, rc 0.
2. MUTATION RED @ e01ed87: in the green tree (or a scratch copy), edit ONLY `belt_credential_recovery_receipt_path`'s body (`roadmap_belt_tick_cli.dag` ~line 828) back to the top-level `join([instance_root as String,"/credential-acquisition-recovery.json"],"")`. SAVE the wrapper script that does edit->run->restore as the RESTORE ATTESTATION. Print `PROD_BEFORE`+`PROD_AFTER` (sha of the file) and assert equality + `git status --porcelain` empty after restore; the two value claims must FAIL, two structural PASS, rc 1. (Prior run flagged `restored_clean=0` with an unsaved wrapper — the fresh run MUST save the wrapper and show restored_clean=1.)
3. IMPORT-REFUSAL RED @ red_at 56d460 (`~/scratch/wise-crane-498-redat`): the home witness .dag is an OVERLAY (it does not exist at red_at). Print `TREE_HEAD=56d460...` AND the overlay file's `sha256sum` of `dag/test/claim/belt_credential_recovery_receipt_home_witness.dag` beside it. Expect FAIL `the_receipt_sits_in_the_instance_receipts_directory` "name 'belt_credential_recovery_receipt_path' not found in module gunbc.roadmap.roadmap_belt_tick_cli", rc 1.
4. NODE-DIGEST TRIO: BEFORE(before-lab@07b3ec16)/AFTER(green@e01ed87)/REDAT(redat@56d460) via nd-crane with the SELF-BUILT gunbc (NDC_GUNBC=); require `controls_ok=yes` (control=`f1c76111c83a0a60`, third non-empty & != control, missing_rc!=0 & missing_out empty). Expect BEFORE rc0 digest, AFTER rc0 digest, REDAT rc1 MISSING.
NOTE: claim_batch resolves the witness entry + closure FROM THE TREE (--source-root dag/src/v2), but PASS/FAIL and fnv1a64 are PRODUCED by the BINARY's interpreter+host-builtins -> the binary is NOT neutral; hence the self-build-from-printed-HEAD requirement.

## 5. NODE-DIGEST TOOL details + controls_ok baseline
- Wrapper `~/scratch/nd-run/nd-crane` sha `360e79e4...`; calls `probe(id:)` from `probe_template.dag` via gunbc `run --entry <probe> --function <fn> --source-root dag --source-root src/v2`.
- Controls (baked): control node `v2-emitter-producer-provenance` -> must equal `f1c76111c83a0a60` rc0; third `2-scm-git-upstream-model` -> non-empty rc0 and != control; missing `no_such_node_zzz_not_a_real_id` -> rc!=0 and empty out. `controls_ok=yes` gates all digests (else PROBE_BROKEN exit 2).
- Baseline observed good (master.log): control=`f1c76111c83a0a60`, third=`b3747b95baf099cc`, missing_rc=1, empty out, controls_ok=yes on all three runs.
- Print BOTH the relocated gunbc sha and the shared `~/shared/node-digest/bin/gunbc` sha; if they differ, state "relocated copy, not the shared gate" (B's rule). Block-gate counts only when byte-identical to shared.

## 6. WHAT IS PROVEN vs UNPROVEN (report to B verbatim)
- PROVEN by on-host logs (master.log), modulo binary provenance: sync = git clone/worktree (no rsync); TREE_HEAD printed immediately before each pair; the 4 home-witness claims, mutation 2-value-FAIL, import-refusal, node-digest trio BEFORE/AFTER/REDAT all with controls_ok=yes.
- UNPROVEN (B's (c)): every prior claim/digest run used a FOREIGN binary (`8ab7bd7753` by merry-deer-252, commit unrecorded) or relocated gunbc `82e9a74a` (commit unrecorded). => run §4 to close.
- HOST HALF: not met (deploy not run); gated on authorized apply; then run §3 once.
- Do NOT `gh pr merge` anything; recording CLOSED; no fleet-converge dispatch; no /opt writes.
