## SIBLING WITNESS LEG — green by execution at head 83c3c9ce117 (srv1, heavy slot)

**Runner:** `cc339_sibling_rerun3.sh`, md5 `0ebe141631cbcce3b1108c9943b55adb` (verified on srv1 before and during the run; this is the runner recorded by the lane manager). Superseded siblings remain on disk untouched; this receipt names only the runner that ran.

**Heavy-slot acquisition (one stage per slot):**
`heavy-slot: slot 4/12 acquired by eager-koi-371 at 2026-11-25T20:35:49Z: bash cc339_sibling_rerun3.sh`
Lane-holder count printed 0 before acquisition. Cgroup: `0::/user.slice/user-1000.slice/user@1000.service/app.slice/heavy-slot-902143.scope`. Slot released on exit.

**Host stamps:**
- `HOST=srv1`, `BOOT=c095c0cd`
- `HEAVY_OWNER=eager-koi-371`, `SESSION=clever-cat-339`

**Pre-acquisition gate (ran BEFORE any slot contact, refusing with rc 95 on failure):**
`PRECHECK_RC=0` — binary present, non-empty, sha256 match. This leg **builds nothing** (fetch + run only); cargo/rustc are never invoked, so the srv1 ssh-PATH trap does not apply.

**Tree identity:**
- `PR_HEAD_REF=83c3c9ce1171a499783b353db5de646f6ffb1feb` → `PR_HEAD_MATCH=yes`
- `TREE=/home/briansrls/scratch/clever-cat-339/pr17033-83c3c9ce`, `TREE_HEAD=83c3c9ce1171a499783b353db5de646f6ffb1feb`, `TREE_HEAD_MATCH=yes`, `TREE_DIRTY_COUNT=0`
- `WITNESS_BLOB_AT_HEAD=12dd42527aa9b58617d80d762eadd0fcb2d29b65` = the reviewed head's blob, computed independently in the worker container
- `ENTRY_MD5=b3d90c5c58bb25a89aed2ee3967eced0` = container-computed md5 of the entry module

**Binary provenance (Manager B's pre-ruled pre-built-base-engine route, with B's build-input tightening as an in-runner gate):**
- `BIN_PATH=/home/briansrls/scratch/clever-cat-339/main-tree/target/release/claim_batch`
- `BIN_SHA256=47614f9e0ca2fb914d1086d402285e1f28e3853e14680ce98742898c85af76ca`, `BIN_SHA_MATCH=yes` (re-verified from disk in-run)
- `BIN_BUILT_FROM_COMMIT=f990441e4aae05b83080f8c55142c41488f4b292` (the PR base) — built locally by cargo from a full clone in `~/scratch/clever-cat-339/main-tree` (git rev-parse HEAD = f990441e4aa; 152 cargo fingerprints; `.rustc_info.json` mtime == binary mtime Nov 24 15:45). **Not** from `~/rb-dl`, **not** a downloaded artifact (0 `release-bins*`/`*.manifest` files in my scratch).
- Whole build-input set gate (Manager B tightening — `src`, `.cargo`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, not just `.rs`): `BUILD_INPUT_DIFF_COUNT=0` over `f990441e4aa..83c3c9ce117`; sorted path+blob manifest identical both sides (`536ef8f1295b15491f47b7e2514e323f2c61d87d412ccb509531630ff37799e2`, 2308 files on each); **positive control fires** (entry-module path+blob list differs across the two commits: `e3b0c442…` vs `f6dade49…`), so the gate discriminates.
- `PR_BASE_TO_HEAD_CHANGED_FILES=1` — `dag/test/claim/shell/shell_dag_codex_app_server_trip_stream_separation_witness.dag` only; NO Rust changed → the base-built binary carries the same interpreter surface as the reviewed head.

**Exact invocation:**
`/home/briansrls/scratch/clever-cat-339/main-tree/target/release/claim_batch --source-root dag --source-root src/v2 --entry dag/test/claim/shell/shell_dag_codex_app_server_trip_stream_separation_witness.dag --functions <the 4 arms> --hermetic`

**Population (non-void):**
`claim_batch: 1 entry group(s), 4 witness(es) total`
`[resolve] …: 174504ms (199 modules, 6590 resolved items in closure)`

**Raw verdict lines (all four arms):**
```
PASS exit_policy_refusal_reports_exit_and_stderr_on_own_fields_never_a_merged_present
PASS channel_selection_admitted_run_presents_stdout_field_only_never_stderr_text
PASS channel_selection_empty_stdout_is_absent_not_a_refusal_and_not_stderr_content
PASS muxed_sentinels_stay_data_in_the_consumer
```
Per-witness evaluation records: eval_steps 35 / 39 / 30 / 2097, subject hashes 22e7512860994da5 / 8a1c734d90728203 / d4e5e36d739766cb / be7a894520bb676b.

**Real exit status, captured un-piped:**
`REAL_CLAIM_BATCH_EXIT_STATUS=0`, persisted `sibling_rerun3.rc value=0`; launcher `LAUNCH_RC=0`. The in-run binary gate (rc 95 refusal path) never fired.

**Resources:** per-shard-peak-rss 7.6 GiB, children-max-rss 6.8 GiB; full log `~/scratch/clever-cat-339/sibling_rerun3.log` (8038 B) incl. cost-partition.

**Head state at run time:** checks_state=passing, mergeable=MERGEABLE, merge_state_status=CLEAN, github_admissible=true, 1/1 distinct dashboard approval (claude), 0 request-changes.

**Rung statement (guarantee ladder):** this leg establishes, by execution at head 83c3c9ce117, that the four witness arms pass on the sealed classifier/parser surfaces named in the module header — exit-policy field selection, stdout/stderr channel selection, absent-vs-refusal distinction, and planted sentinel-muxed input staying data in the consumer. It is not a claim about production `RunArgvStdin` plumbing (covered by the pinned contract witness legs) nor a compiler-enforced guarantee (gap recorded in the integration-concern row with its climbs-when trigger).

— posted from clever-cat-339
