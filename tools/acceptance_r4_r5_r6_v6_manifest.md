# R4, R5, R6 acceptance execution packets — v6: one job per row, hardened srv1 frame (PREPARE ONLY)

Prepared by `fierce-wren-19` for `calm-eagle-670`, on keen-otter's hardened srv1 frame (templates: #15938's
`rdpo.sh` `aaf32964…` and rrc `a1de55aa`). **Nothing here has been executed.** No ssh, no claim run, no
partition write happened in the preparing container; the only modes run locally are `--classifier-fixtures`
and `--predict-only`, which write nothing outside a `mktemp -d` scratch and never touch a repository.
Execution grants come only from keen-otter via sleek-owl-397. **No row here is grantable by this document.**

v6 splits the R4+R5 packet into **two single-row jobs** because the installed launcher refuses the booking:
`calmcrab-udrv` v4 (`855920ac`) rejects any `RUNTIME_S` above 7200 in its own preflight, and the v5 combined
job needed `setup 720 + build 2400 + 4 legs x 900 + margin = 8940 s` — a `PRE_SCOPE_REFUSAL`, the same shape
as #15695 diag-v2 booked at 11700. Changing the launcher is out of scope, so the *rows* move, not the launcher.

## v5 -> v6: what changed, and which assertion proves it

| # | change (manager msg_384bca0f, from sleek-owl-397 msg_995f146b) | what landed | asserted by |
|---|---|---|---|
| 1 | split R4/R5 into two jobs, R4 first then R5; each its own clone/fetch/checkout/build/legs | three packets: `acceptance_r4_claim_run_v6.sh` (`fierce-wren-19-r4-db56530f`), `acceptance_r5_claim_run_v6.sh` (`fierce-wren-19-r5-dff57af7`), `acceptance_r6_live_read_claim_run_v6.sh` (`fierce-wren-19-r6-dff57af7`) | the self-check greps each packet for its own `J=` job token, and asserts no job clones or builds the other row's pin (`pin_clone "$WT5"` absent from R4, `pin_clone "$WT4"` absent from R5, `wt-r4` absent from R6) |
| 2 | `RUNTIME_S <= 7200` per job, with the capped sum stated in the header | R4 `RUNTIME_S=5400`, R5 `4500`, R6 `4800`; each header carries `WALL ACCOUNTING` with the per-step timeouts and `CAPPED SUM = <n> s` | the self-check recomputes `T_SETUP_CLONE+T_SETUP_FETCH+T_SETUP_CHECKOUT+T_BUILD+N_LEGS*T_LEG` from each packet's own frozen timeouts, requires it to equal the `wall_capped_sum_s=` line that packet prints, and re-states the three bookings independently (4920/4020/4020) so a packet cannot satisfy it by moving a constant |
| 3 | a self-check that the booking fits, before any fetch | new `wall_gate WALL_SUM RUNTIME_S LAUNCHER_CAP DRIVER_S` in **all three** packets, called in the run path immediately after the bounds gate and before any clone/fetch: refuses `RUNTIME_S > LAUNCHER_CAP` (`PRE_SCOPE_REFUSAL: RUNTIME_S <n> exceeds the installed calmcrab-udrv v4 preflight cap of 7200`), `sum > RUNTIME_S`, and a driver wall that is not `RUNTIME_S + 300` | per packet: 7 wall fixtures (`wall-gate-accepts-this-row`, `capped-sum`, `runtime-s-within-installed-launcher-cap`, `driver-wall-is-scope-plus-300`, and three refusals), plus self-structure needles that the gate is *used* as a gate (`WM=$(wall_ga` + `te`), and negative control `mut15` (rewrite `RUNTIME_S` to 9900) which makes each packet fail its own `wall-gate-accepts-this-row` |
| 4 | driver service = `RUNTIME_S + 300` | stated in each header, frozen as `DRIVER_S`, printed by `--predict-only` as `driver_service=calmcrab-drv-<JOB>.service RuntimeMaxSec=<n> (RUNTIME_S plus 300 s)` | header grep + `wall_gate`'s third comparison + the manifest line shape check in the self-check |
| 5 | command inventory byte-identical per row, or diff it | the per-row command strings are unchanged from v4/v5: R4 six commands (`f256a1da… 4a0ac69f… fdb0ba19… b9eb7b3c… 453568e5… 98211ed7…`), R5 five (`1c90e944… 1b03a85f… 220bd6bb… b9eb7b3c… c5f7fe87…`), R6 five (`995b533e… 4500968d… 85a78469… b9eb7b3c… 0053be34…`) | the self-check pins those digests and diffs them against what each packet prints now, per row; any changed command string fails the self-check |
| 6 | every v5 gate kept, and it has to bite in each packet | the stderr completion gate, the prefix-count-then-shape rule, the producer blob gate + render-site re-read, the end-to-end `leg()` controls, the empty-predicate refusal, the verdict exactness rules | the negative controls now run per packet: `mut8`-`mut13` (and `mut1`/`mut2` for R4, `mut7` for R6) are applied to R4, R5 and R6 copies; each mutated copy must fail its own embedded fixtures |
| 7 | one gate, three jobs — no dialects | the gate sections of all three packets are the same code | the self-check extracts each packet's `gates and helpers` region, drops the row-specific lines by name (pins, entry paths, CSVs, worktree names, runner variable) and R6's `grant_gate`, and requires R4 == R5 exactly, and R6 == R4 plus only the grant-gate lines (zero deletions, additions must mention the execution grant, and must not re-implement `pp_check`/`claim_classify`/`wall_gate`/`bounds_check`/`scope_gate`/`oom_*`) |
| 8 | cite the symbol, not the line number (sunny-bee-153 msg_bc02bb8c) | the packets no longer say `claim_batch.rs:1460`; they name `the single eprintln!("{}", partition.render(&timings)) call site in the run() path of src/v1/stage0/src/bin/claim_batch.rs (blob 12a71786…)` | `grep -q ":1460" over the three packets` finds nothing; the packet's `--predict-only` prints `render=the single eprintln partition.render(&timings) emission in run() blob=12a71786…` |
| 9 | merged capture is a design decision, not an accident | each packet header carries `STREAM DISCIPLINE IS A CONTRACT`: the leg capture is deliberately `> out 2> err`, and if anyone narrows it to stdout-only the gate reads zero completion records and a full-green row dies as NO_OBSERVATION (#15695 G1's shape, arrived at twice by two different routes) | the self-check greps for that sentence in each packet, and the `mut12` control points the gate back at `.out` in both the R5 and R6 packets, and must break them |
| 10 | R6 stays HELD | unchanged behaviour: it refuses with exit 2 unless its second argument names a cleared keen-otter grant for msg_3407a188; its booking is now checked against the same 7200 cap | its `grant_gate` fixtures (no argument / empty / other row) and the self-check's end-to-end probe: the shipped R6 packet refuses, creates nothing; the relaxed copy with a grant reference stops at the next gate |

## Files (in the draft PR, branch `session/fierce-wren-19`)

| path | bytes | sha256 |
|---|---|---|
| `tools/acceptance_r4_claim_run_v6.sh` | 66093 | `507fef33f8b8902232f9fbf6e7681ef87c9b3f50ea192b84f4a788852d1f6265` |
| `tools/acceptance_r5_claim_run_v6.sh` | 65059 | `563e5e51afda75630992ee134d3895c53ec9e4012cfb4cca2e3f15bf87fbf99d` |
| `tools/acceptance_r6_live_read_claim_run_v6.sh` | 62022 | `e8aba5e809e5134b2bee240edf3203ba7b89879325b345ea1f6071f76caf961b` |
| `tools/acceptance_r4_r5_r6_selfcheck_v6.sh` | 69792 | `b30d9dbc0768ddac919dac5f8091606d960e094e3e96e436aa75d2ab81b59621` |
| `tools/acceptance_r4_r5_r6_v6_manifest.md` | (this file) | stated in the PR comment and the message to calm-eagle-670, because a file cannot carry its own digest |

The launcher is handed the **staged script's** sha256 as argv[8]: that is the sha256 of the packet file above
(the one the launcher verifies against the staged copy), not a source-commit sha. `script_sha256_frozen=see-tools/…`
in `--predict-only` is deliberate: the packet states where its own digest is recorded.

## The three launcher lines, and the arithmetic behind each

All three use the calmcrab-udrv v4 argv in the launcher's order, `JOB RUNTIME_S MEMMAX_G MEMHIGH_G TASKS CPU% STAGED_SCRIPT SCRIPT_SHA OUT`,
memory in GiB, CPU with `%`, `OUT` as the literal token; memory bounds are the R row, the driver service is one task
(S row), and the D row is the scope itself (`cap 0`):

```
calmcrab-udrv.sh fierce-wren-19-r4-db56530f 5400 32 28 4096 3200% <STAGED_SCRIPT> <r4 packet sha256> OUT
calmcrab-udrv.sh fierce-wren-19-r5-dff57af7 4500 32 28 4096 3200% <STAGED_SCRIPT> <r5 packet sha256> OUT
calmcrab-udrv.sh fierce-wren-19-r6-dff57af7 4800 32 28 4096 3200% <STAGED_SCRIPT> <r6 packet sha256> OUT   # HELD
```

| job | clone | fetch | checkout | build | legs | capped sum | scope `RuntimeMaxSec` | launcher cap | driver `RuntimeMaxSec` |
|---|---|---|---|---|---|---|---|---|---|
| R4 `fierce-wren-19-r4-db56530f` | 300 | 300 | 120 | 2400 | 2 x 900 = 1800 | **4920** | 5400 | 7200 | 5700 |
| R5 `fierce-wren-19-r5-dff57af7` | 300 | 300 | 120 | 2400 | 1 x 900 = 900 | **4020** | 4500 | 7200 | 4800 |
| R6 `fierce-wren-19-r6-dff57af7` (HELD) | 300 | 300 | 120 | 2400 | 1 x 900 = 900 | **4020** | 4800 | 7200 | 5100 |

Every job satisfies `sum <= RUNTIME_S <= 7200` and `driver = RUNTIME_S + 300`; the margin left over the capped
sum is 480 s (R4), 480 s (R5), 780 s (R6). The R4 job runs first, then R5; each does its own clone, fetch,
checkout and `cargo build --locked -p claim_batch`, so nothing is shared between the two bookings.

Booked bounds, identical in all three (the packets refuse on any mismatch **before** any fetch or build):
`memory.max=34359738368 memory.high=30064771072 memory.swap.max=0 pids.max=4096 cpu.max=3200000_100000`
(32 GiB / 28 GiB / no swap / 4096 tasks / 3200%), derived from the same `MEMMAX_G MEMHIGH_G TASKS_MAX CPU_PCT`
constants that go into the launcher argv, so the packet and the launcher cannot disagree.

## What the rows do (and do not) show

R4 and R5 **discriminate nothing**: they are presence observations with no mutation leg, and no mutation leg is
proposed here. A pass says the witness claims exist and hold in the pinned tree; a bare `FAIL` would falsify
that. R6 asks whether the roadmap acceptance inbox projects on a live corpus; a refusal there is a FINDING,
and R6 is HELD (below).

## Row R4 — `tools/acceptance_r4_claim_run_v6.sh` (job `fierce-wren-19-r4-db56530f`)

* pin `db56530fcb186370e3dbf1aa3c69ded324daa25d` (the grant's R4 pin; tree `48be84d8b18a9530e3a5c42adf8c24b19e1aa4dc`);
  entry blobs `332cc981fd194bb9f02f16ff6e3c14d81b874e2b` (`src/v2/test/claim/stage0_partition_rebuild_scope_witness.dag`)
  and `abc6313eb8c4d49ad74a7bcafad36b895ea32f76` (`src/v2/test/claim/rust_crate_partition_witness.dag`).
* two legs, one per entry file: `R4/a` five witness claims in the scope-witness dag, `R4/b` `witness_r4_refuses_unit_graph_cycle_holds`
  in the partition-witness dag. Expect **all PASS**; the row is HELD only if every name has exactly one bare `PASS`.
* `--predict-only` prints exactly the six command strings (four setup + two legs), each with bytes and sha256:

```
PREDICT-ONLY row=R4 job=fierce-wren-19-r4-db56530f host_booked=srv1 script_sha256_frozen=see-tools/acceptance_r4_r5_r6_v6_manifest.md
THIS ROW DISCRIMINATES NOTHING: presence observations, no mutation leg, none proposed.
pin_r4=db56530fcb186370e3dbf1aa3c69ded324daa25d tree_r4=48be84d8b18a9530e3a5c42adf8c24b19e1aa4dc
blob src/v2/test/claim/stage0_partition_rebuild_scope_witness.dag 332cc981fd194bb9f02f16ff6e3c14d81b874e2b
blob src/v2/test/claim/rust_crate_partition_witness.dag abc6313eb8c4d49ad74a7bcafad36b895ea32f76
bounds=memory.max=34359738368 memory.high=30064771072 memory.swap.max=0 pids.max=4096 cpu.max=3200000_100000
launcher_units runtime_s=5400 memmax_gib=32 memhigh_gib=28 tasks=4096 cpu=3200%
launcher_call=calmcrab-udrv.sh fierce-wren-19-r4-db56530f 5400 32 28 4096 3200% <STAGED_SCRIPT> <this packet sha256> OUT
driver_service=calmcrab-drv-fierce-wren-19-r4-db56530f.service RuntimeMaxSec=5700 (RUNTIME_S plus 300 s)
launcher_preflight_cap=7200 (calmcrab-udrv v4, 855920ac): a RUNTIME_S above it is a PRE_SCOPE_REFUSAL, so this packet refuses it itself
ledger_rows=fierce-wren-19-r4-db56530f-res(cap 32GiB) fierce-wren-19-r4-db56530f-drv(cap 1) fierce-wren-19-r4-db56530f(cap 0)
completion_record_gate=exactly-one-prefix-count-then-shape sha_pinned=7ac24cf28973511b7902ce3ba621e8c46b060ad7855ea895b1f5a157bf1835b2
completion_record_stream=stderr(leg capture logs/<leg>.err) verdict_stream=stdout(<leg>.out) render=the single eprintln partition.render(&timings) emission in run() blob=12a71786bd92f683dbd0aec2ae5d6f426e651a44
verdict_rules=bare_only;one_outcome_line_per_name_any_keyword;qualified_is_NO_OBSERVATION
timeouts_s setup_clone=300 setup_fetch=300 setup_checkout=120 build=2400 leg=900
wall_capped_sum_s=4920 scope_RuntimeMaxSec=5400 driver_RuntimeMaxSec=5700 launcher_cap=7200 legs=2
claims_r4a=5 claims_r4b=1
exit_contract: 0=expectations_held 1=falsified_bare_FAIL 2=NO_OBSERVATION(every refusal)
cmd CMD_CLONE_4 bytes=24 sha256=f256a1da5244bfd6b62af2e05eb608b6b15e99fda2deda43df961af728917fe5
  git init -q "$OUT/wt-r4"
cmd CMD_FETCH_4 bytes=112 sha256=4a0ac69f9b81df39536466bb77e843488984137f853bbcf1b90f32dad9d78cab
  git -C "$OUT/wt-r4" fetch -q --depth 1 https://github.com/gunb-ai/gunbc db56530fcb186370e3dbf1aa3c69ded324daa25d
cmd CMD_CHECKOUT_4 bytes=51 sha256=fdb0ba1965609f46b72f4eb4ead4fb3a870a897c6bd01bb4bbdc88b6e4331911
  git -C "$OUT/wt-r4" checkout -q --detach FETCH_HEAD
cmd CMD_BUILD_4 bytes=69 sha256=b9eb7b3c3870d6bcc728886a54201223fa01f449790eec91d096ce35766769a5
  cargo build --locked --release -j 32 -p v1-compiler --bin claim_batch
cmd CMD_LEG_R4A bytes=387 sha256=453568e5420cb9f95c34f54a7ddc1c8b252a7a531d29e60b6d0b358bcd741e05
  "$BIN4" --source-root dag --source-root src/v1 --source-root src/v2 --entry "src/v2/test/claim/stage0_partition_rebuild_scope_witness.dag" --functions "witness_duplicate_owner_refuses_holds,witness_unowned_mirror_refuses_holds,witness_shell_partition_duplicate_owner_refuses_holds,witness_leaf_edit_compiles_owner_and_downstream_holds,witness_leaf_edit_excludes_unrelated_packages_holds"
cmd CMD_LEG_R4B bytes=184 sha256=98211ed70494ef8c9b85eed3b1e39e4cfd29b975d8b059cf185a11c92231d652
  "$BIN4" --source-root dag --source-root src/v1 --source-root src/v2 --entry "src/v2/test/claim/rust_crate_partition_witness.dag" --functions "witness_r4_refuses_unit_graph_cycle_holds"
```

## Row R5 — `tools/acceptance_r5_claim_run_v6.sh` (job `fierce-wren-19-r5-dff57af7`)

* pin `dff57af790f923066d61721e89ff751a91021278` (the #15936 squash; tree `356f2894a60edda51cdbe79879b2fb3b7aea75c4`);
  entry blob `adfeb29426eac9d4da67630a12d89f8eabe8754b` (`dag/test/claim/roadmap/roadmap_acceptance_inbox_witness.dag`).
* one leg, five inbox-witness claims. Expect **all PASS**.
* no mutation leg; the row discriminates nothing.

```
PREDICT-ONLY row=R5 job=fierce-wren-19-r5-dff57af7 host_booked=srv1 script_sha256_frozen=see-tools/acceptance_r4_r5_r6_v6_manifest.md
THIS ROW DISCRIMINATES NOTHING: presence observations, no mutation leg, none proposed.
pin_r5=dff57af790f923066d61721e89ff751a91021278 tree_r5=356f2894a60edda51cdbe79879b2fb3b7aea75c4
blob dag/test/claim/roadmap/roadmap_acceptance_inbox_witness.dag adfeb29426eac9d4da67630a12d89f8eabe8754b
bounds=memory.max=34359738368 memory.high=30064771072 memory.swap.max=0 pids.max=4096 cpu.max=3200000_100000
launcher_units runtime_s=4500 memmax_gib=32 memhigh_gib=28 tasks=4096 cpu=3200%
launcher_call=calmcrab-udrv.sh fierce-wren-19-r5-dff57af7 4500 32 28 4096 3200% <STAGED_SCRIPT> <this packet sha256> OUT
driver_service=calmcrab-drv-fierce-wren-19-r5-dff57af7.service RuntimeMaxSec=4800 (RUNTIME_S plus 300 s)
launcher_preflight_cap=7200 (calmcrab-udrv v4, 855920ac): a RUNTIME_S above it is a PRE_SCOPE_REFUSAL, so this packet refuses it itself
ledger_rows=fierce-wren-19-r5-dff57af7-res(cap 32GiB) fierce-wren-19-r5-dff57af7-drv(cap 1) fierce-wren-19-r5-dff57af7(cap 0)
completion_record_gate=exactly-one-prefix-count-then-shape sha_pinned=7ac24cf28973511b7902ce3ba621e8c46b060ad7855ea895b1f5a157bf1835b2
completion_record_stream=stderr(leg capture logs/<leg>.err) verdict_stream=stdout(<leg>.out) render=the single eprintln partition.render(&timings) emission in run() blob=12a71786bd92f683dbd0aec2ae5d6f426e651a44
verdict_rules=bare_only;one_outcome_line_per_name_any_keyword;qualified_is_NO_OBSERVATION
timeouts_s setup_clone=300 setup_fetch=300 setup_checkout=120 build=2400 leg=900
wall_capped_sum_s=4020 scope_RuntimeMaxSec=4500 driver_RuntimeMaxSec=4800 launcher_cap=7200 legs=1
claims_r5=5
exit_contract: 0=expectations_held 1=falsified_bare_FAIL 2=NO_OBSERVATION(every refusal)
cmd CMD_CLONE_5 bytes=24 sha256=1c90e94474e67011795be02913ba5cb6c341346f03365a96bb884aea6e82d586
  git init -q "$OUT/wt-r5"
cmd CMD_FETCH_5 bytes=112 sha256=1b03a85fa7cfa0244152c9a52bae422b10db9905ed5214bf92a1f8ab7f8ade19
  git -C "$OUT/wt-r5" fetch -q --depth 1 https://github.com/gunb-ai/gunbc dff57af790f923066d61721e89ff751a91021278
cmd CMD_CHECKOUT_5 bytes=51 sha256=220bd6bb5942cb5467d4099b7d8c749ede44667d978ac32d54099a4d2809fca4
  git -C "$OUT/wt-r5" checkout -q --detach FETCH_HEAD
cmd CMD_BUILD_5 bytes=69 sha256=b9eb7b3c3870d6bcc728886a54201223fa01f449790eec91d096ce35766769a5
  cargo build --locked --release -j 32 -p v1-compiler --bin claim_batch
cmd CMD_LEG_R5 bytes=397 sha256=c5f7fe87f6130cc3ad2cf1f60dc99be4457f0e81a419b7025f8bb81c674ceda7
  "$BIN5" --source-root dag --source-root src/v1 --source-root src/v2 --entry "dag/test/claim/roadmap/roadmap_acceptance_inbox_witness.dag" --functions "witness_inbox_load_refused_is_refused_not_empty,witness_inbox_unreplayable_history_is_refused_not_empty,witness_inbox_live_receipt_removes_node_from_rows,witness_inbox_revoked_receipt_reappears_with_prior,witness_inbox_stale_live_receipt_refuses"
```

## Row R6 — HELD — `tools/acceptance_r6_live_read_claim_run_v6.sh` (job `fierce-wren-19-r6-dff57af7`)

* same pin and tree as R5; entry blob `969b980b189dafa81df4045f62f7d2e70dcc50ad`
  (`dag/test/claim/roadmap/roadmap_acceptance_inbox_live_witness.dag`), one claim `witness_inbox_on_authority_projects_on_live_corpus`.
* **HELD behind keen-otter msg_3407a188** (existing rostered reader only). The read path was audited before packing:
  `roadmap_acceptance_event_history_load` -> `fabric_state_storage_binding` -> `roadmap_operator_acceptance_history_load`;
  the branch a bare srv1 checkout binds is the one that would read the live partition, so the row is gated, not seeded:
  the packet refuses with **exit 2** unless its second argument names a cleared grant, and creates nothing on the way.
* a refusal in the claim itself is a FINDING, not a tool status: the classifier records `VERDICT=` per row from the
  verdict lines, and only a bare `FAIL <fn>` falsifies.

```
PREDICT-ONLY row=R6 job=fierce-wren-19-r6-dff57af7 state=HELD gate=keen-otter-msg_3407a188 host_booked=srv1
pin=dff57af790f923066d61721e89ff751a91021278 tree=356f2894a60edda51cdbe79879b2fb3b7aea75c4
blob dag/test/claim/roadmap/roadmap_acceptance_inbox_live_witness.dag 969b980b189dafa81df4045f62f7d2e70dcc50ad
bounds=memory.max=34359738368 memory.high=30064771072 memory.swap.max=0 pids.max=4096 cpu.max=3200000_100000
launcher_units runtime_s=4800 memmax_gib=32 memhigh_gib=28 tasks=4096 cpu=3200%
launcher_call=calmcrab-udrv.sh fierce-wren-19-r6-dff57af7 4800 32 28 4096 3200% <STAGED_SCRIPT> <this packet sha256> OUT
ledger_rows=fierce-wren-19-r6-dff57af7-res(cap 32GiB) fierce-wren-19-r6-dff57af7-drv(cap 1) fierce-wren-19-r6-dff57af7(cap 0)
completion_record_gate=exactly-one-prefix-count-then-shape sha_pinned=7ac24cf28973511b7902ce3ba621e8c46b060ad7855ea895b1f5a157bf1835b2
completion_record_stream=stderr(leg capture logs/<leg>.err) verdict_stream=stdout(<leg>.out) render=the single eprintln partition.render(&timings) emission in run() blob=12a71786bd92f683dbd0aec2ae5d6f426e651a44
verdict_rules=bare_only;one_outcome_line_per_name_any_keyword;qualified_is_NO_OBSERVATION
timeouts_s setup_clone=300 setup_fetch=300 setup_checkout=120 build=2400 leg=900
wall_capped_sum_s=4020 scope_RuntimeMaxSec=4800 driver_RuntimeMaxSec=5100 launcher_cap=7200 legs=1
driver_service=calmcrab-drv-fierce-wren-19-r6-dff57af7.service RuntimeMaxSec=5100 (RUNTIME_S plus 300 s)
launcher_preflight_cap=7200 (calmcrab-udrv v4, 855920ac): a RUNTIME_S above it is a PRE_SCOPE_REFUSAL, so this packet refuses it itself
claims_r6=1
execute form: script.sh <OUT> <execution-grant-ref>   (no reference = refused, exit 2, nothing created)
exit_contract: 0=expectations_held 1=falsified_bare_FAIL 2=NO_OBSERVATION(every refusal, incl. the hold)
cmd CMD_CLONE bytes=24 sha256=995b533e654e1c8b6aa68add106d1df71e57699aa2be4dfd6516a793cf9e9de2
  git init -q "$OUT/wt-r6"
cmd CMD_FETCH bytes=112 sha256=4500968dca5ab55989dd67fb8468a8ebd3749074fc85674f964cdda4e68db56a
  git -C "$OUT/wt-r6" fetch -q --depth 1 https://github.com/gunb-ai/gunbc dff57af790f923066d61721e89ff751a91021278
cmd CMD_CHECKOUT bytes=51 sha256=85a784694b36535a3df3822daf36611aecca3a4d12163bb96910c6228a40eff0
  git -C "$OUT/wt-r6" checkout -q --detach FETCH_HEAD
cmd CMD_BUILD bytes=69 sha256=b9eb7b3c3870d6bcc728886a54201223fa01f449790eec91d096ce35766769a5
  cargo build --locked --release -j 32 -p v1-compiler --bin claim_batch
cmd CMD_LEG_R6 bytes=207 sha256=0053be34fb87d635cd5664cccd7880bd014a18417ca24bfcc1c9b61cd512be7a
  "$BIN6" --source-root dag --source-root src/v1 --source-root src/v2 --entry "dag/test/claim/roadmap/roadmap_acceptance_inbox_live_witness.dag" --functions "witness_inbox_on_authority_projects_on_live_corpus"
```

## The completion record every leg is gated on (unchanged from v4/v5)

`claim_batch` prints one `[process-partition] total_ms=…` record per process, to **stderr**, from the single
`eprintln!("{}", partition.render(&timings))` call site in the `run()` path of `src/v1/stage0/src/bin/claim_batch.rs`
(blob `12a71786bd92f683dbd0aec2ae5d6f426e651a44`, identical at both pins). The packets gate on:

* fixture `COMPLETION_LINE` = the real 259-byte record from calm-crab-645's srv1 run (`claims-cpr.log` line 66),
  pinned **without** a trailing LF: `sha256 7ac24cf28973511b7902ce3ba621e8c46b060ad7855ea895b1f5a157bf1835b2`.
  The with-LF variant is a different object (`d9e50eb2ec6352ed32d2cb4788057fb3f91f8da766e0ee2d999ae3c531b3fa50`, 260 B)
  and is **not** what these packets pin; a "259 B" claim and a trailing-newline fixture would be self-contradictory.
* the digest is re-verified at every call site, and an empty `COMPLETION_LINE`/predicate refuses the run: an empty
  pattern matches every line, which would silently turn a count-equals-one gate into "count everything".
* gate order per leg: count records by the **prefix** `^\[process-partition\] total_ms` (the `]` keeps the
  `[process-partition-exit] run_returned_ms=…` neighbour from being counted), require exactly one, then test the
  shape `^\[process-partition\] total_ms=[0-9]+( |$)`. A whole-line anchor without `( |$)` rejects the real line —
  that is what spent #15695's G1 attempt with 15/15 bare PASS.
* verdicts: only bare `^PASS <fn>$` / `^FAIL <fn>$` count; each expected name needs exactly one outcome line of any
  keyword (PASS/FAIL/UNOBSERVED/PANICKED, bare or qualified) — zero or two is NO_OBSERVATION; a qualified
  `FAIL <fn> (reason)` never falsifies.

## Inputs read, writes, and the exit contract (all three packets)

* read-only: the pinned commit and its blobs (`git ls-remote`/`clone --branch`/`fetch`/`checkout`/`rev-parse` gate,
  `HEAD == PIN` and `tree == TREE` before any build), `src/v1/stage0/src/bin/claim_batch.rs` (blob-gated, then
  re-read for the render site), the entry dag files (blob-gated), the cgroup files for the bounds and OOM snapshots,
  and the claim runner's own stdout/stderr captures. R6 additionally reads nothing beyond that: it refuses before any of it.
* writes: only under a fresh **exclusive-mkdir** output dir (`OUT`, `$HOME/runs/<JOB>/out` by the launcher convention;
  an existing `OUT` is a refusal, exit 2) — the receipt, the per-leg `.out`/`logs/*.err` captures, the raw OOM snapshots
  published noclobber *before* parsing, and the leg logs. No partition write, no operator state, no roster write.
* exit contract: `0` = every expectation held; `1` = a bare `FAIL` falsified a row; `2` = NO_OBSERVATION, which covers
  every setup or gate refusal (bounds mismatch, wall over cap, scope mismatch, reused `OUT`, missing tool, malformed
  completion record, missing/duplicate/qualified verdict evidence, and R6's un-cleared hold). There is no exit 3 and
  no "couldn't run" exit 1.
* OOM: one snapshot per boundary, raw bytes published before parsing, each key required exactly once as `^key [0-9]+$`;
  any read or write failure is UNKNOWN and therefore NO_OBSERVATION; rise/fall compared per boundary and final vs
  last-leg. No `grep '^oom_kill [1-9]'`, no silent `cat`, and `memory.peak` is printed but never judged.

## Self-test result (recorded, not asserted vaguely)

* `bash tools/acceptance_r4_r5_r6_selfcheck_v6.sh <repo>` (run in this container against this session's worktree,
  which carries both pins) → **699 passed, 0 failed, 0 skipped**.
* embedded packet fixtures: R4 **178 pass / 0 fail**, R5 **176 pass / 0 fail**, R6 **180 pass / 0 fail**.
* skips are named, not silent. There are exactly two conditional skips, both labelled in the script:
  `claim_batch.rs at the pin is not byte-identical to the working copy: parser checks skipped` and
  `no repo with the pinned commits: the pinned render-site grep was not run`. In the recorded run above neither
  fired (`0` skipped); a run **without** the repo argument takes the second one, and that number is the tell.
* the self-check tests the shipped code, not a copy: it extracts each packet's helper region verbatim and sources it,
  drives each packet's own `leg()` with a stub binary, runs each packet's `--predict-only` in two different cwds and
  requires byte-identical output, and requires `--predict-only` to create no file anywhere under the scratch root.
* end-to-end on this host: the shipped packets stop at the tooling gate (`/usr/bin/time` is absent here) with exit 2
  and create nothing — for all three jobs, R6 included with a grant reference in hand. A relaxed copy (that one check
  replaced) must reach the scope gate, write a receipt with `VERDICT=NO_OBSERVATION reason=scope`, publish zero memory
  snapshots, create no clone, and refuse a reused `OUT` with exit 2.

### Negative controls (each mutated copy must fail its own embedded fixtures, or the checks are decorative)

| control | first fixture failure |
|---|---|
| mutation 3 applied to the R4 job | FIXTURE FAIL self-structure: the booked bounds are gated before any work expected 1 got 0 / FIXTURES pass=176 fail=2 (rc=1) |
| mutation 4 applied to the R4 job | FIXTURE FAIL snap-parsed expected 0/0 got UNKNOWN/UNKNOWN / NO_OBSERVATION: leg E2E_STDERR: memory.events moved UNKNOWN/UNKNOWN -> UNKNOWN/UNKNOWN (re… |
| mutation 5 applied to the R4 job | FIXTURE FAIL self-structure: no fourth exit code exists anywhere in the script expected 0 got 1 / FIXTURES pass=177 fail=1 (rc=1) |
| mutation 6 applied to the R4 job | FIXTURE FAIL self-structure: each leg boundary pair is compared, not merely recorded expected 1 got 0 / FIXTURES pass=177 fail=1 (rc=1) |
| mutation 8 applied to the R4 job | FIXTURE FAIL self-structure: the completion gate reads the leg stderr capture, where the record is emitted expected 1 got 0 / FIXTURES pass=174 fail=4… |
| mutation 9 applied to the R4 job | FIXTURE FAIL verdict-bare-pass-plus-qualified-fail expected NO_OBSERVATION got HELD / FIXTURES pass=177 fail=1 (rc=1) |
| mutation 10 applied to the R4 job | FIXTURE FAIL verdict-qualified-fail-is-not-falsified expected NO_OBSERVATION got FAILED / FIXTURES pass=177 fail=1 (rc=1) |
| mutation 11 applied to the R4 job | FIXTURE FAIL completion-duplicate expected completion records: 2 matched the prefix, exactly 1 is required got OK [process-partition] total_ms=340188 … |
| mutation 12 applied to the R4 job | FIXTURE FAIL self-structure: the completion gate reads the leg stderr capture, where the record is emitted expected 1 got 0 / FIXTURES pass=172 fail=6… |
| mutation 13 applied to the R4 job | FIXTURE FAIL self-structure: the pin proves the pinned producer emits the record to stderr expected 1 got 0 / FIXTURES pass=177 fail=1 (rc=1) |
| mutation 3 applied to the R5 job | FIXTURE FAIL self-structure: the booked bounds are gated before any work expected 1 got 0 / FIXTURES pass=174 fail=2 (rc=1) |
| mutation 4 applied to the R5 job | FIXTURE FAIL snap-parsed expected 0/0 got UNKNOWN/UNKNOWN / NO_OBSERVATION: leg E2E_STDERR: memory.events moved UNKNOWN/UNKNOWN -> UNKNOWN/UNKNOWN (re… |
| mutation 5 applied to the R5 job | FIXTURE FAIL self-structure: no fourth exit code exists anywhere in the script expected 0 got 1 / FIXTURES pass=175 fail=1 (rc=1) |
| mutation 6 applied to the R5 job | FIXTURE FAIL self-structure: each leg boundary pair is compared, not merely recorded expected 1 got 0 / FIXTURES pass=175 fail=1 (rc=1) |
| mutation 8 applied to the R5 job | FIXTURE FAIL self-structure: the completion gate reads the leg stderr capture, where the record is emitted expected 1 got 0 / FIXTURES pass=172 fail=4… |
| mutation 9 applied to the R5 job | FIXTURE FAIL verdict-bare-pass-plus-qualified-fail expected NO_OBSERVATION got HELD / FIXTURES pass=175 fail=1 (rc=1) |
| mutation 10 applied to the R5 job | FIXTURE FAIL verdict-qualified-fail-is-not-falsified expected NO_OBSERVATION got FAILED / FIXTURES pass=175 fail=1 (rc=1) |
| mutation 11 applied to the R5 job | FIXTURE FAIL completion-duplicate expected completion records: 2 matched the prefix, exactly 1 is required got OK [process-partition] total_ms=340188 … |
| mutation 12 applied to the R5 job | FIXTURE FAIL self-structure: the completion gate reads the leg stderr capture, where the record is emitted expected 1 got 0 / FIXTURES pass=170 fail=6… |
| mutation 13 applied to the R5 job | FIXTURE FAIL self-structure: the pin proves the pinned producer emits the record to stderr expected 1 got 0 / FIXTURES pass=175 fail=1 (rc=1) |
| mutation 3 applied to the R6 job | FIXTURE FAIL self-structure: the booked bounds are gated before any work expected 1 got 0 / FIXTURES pass=178 fail=2 (rc=1) |
| mutation 4 applied to the R6 job | FIXTURE FAIL snap-parsed expected 0/0 got UNKNOWN/UNKNOWN / NO_OBSERVATION: leg E2E_STDERR: memory.events moved UNKNOWN/UNKNOWN -> UNKNOWN/UNKNOWN (re… |
| mutation 5 applied to the R6 job | FIXTURE FAIL self-structure: no fourth exit code exists anywhere in the script expected 0 got 1 / FIXTURES pass=179 fail=1 (rc=1) |
| mutation 6 applied to the R6 job | FIXTURE FAIL self-structure: each leg boundary pair is compared, not merely recorded expected 1 got 0 / FIXTURES pass=179 fail=1 (rc=1) |
| mutation 8 applied to the R6 job | FIXTURE FAIL self-structure: the completion gate reads the leg stderr capture, where the record is emitted expected 1 got 0 / FIXTURES pass=177 fail=3… |
| mutation 9 applied to the R6 job | FIXTURE FAIL verdict-bare-pass-plus-qualified-fail expected NO_OBSERVATION got HELD / FIXTURES pass=179 fail=1 (rc=1) |
| mutation 11 applied to the R6 job | FIXTURE FAIL completion-duplicate expected completion records: 2 matched the prefix, exactly 1 is required got OK [process-partition] total_ms=340188 … |
| mutation 12 applied to the R6 job | FIXTURE FAIL self-structure: the completion gate reads the leg stderr capture, where the record is emitted expected 1 got 0 / FIXTURES pass=175 fail=5… |
| mutation 13 applied to the R6 job | FIXTURE FAIL self-structure: the pin proves the pinned producer emits the record to stderr expected 1 got 0 / FIXTURES pass=179 fail=1 (rc=1) |
| defect (1) reintroduced: a literal claim_batch token after the binary | FIXTURE FAIL argv-R4a-sound expected  got R4a: argv carries a literal 'claim_batch' token after the binary / FIXTURES pass=176 fail=2 (rc=1) |
| defect (2) reintroduced: a space-separated name in the CSV | FIXTURE FAIL csv-r4b expected 1 got 3 / FIXTURES pass=175 fail=3 (rc=1) |
| defect (1) in the R6 packet as well | FIXTURE FAIL argv-R6-sound expected  got R6: argv carries a literal 'claim_batch' token after the binary / FIXTURES pass=178 fail=2 (rc=1) |
| the end-to-end leg control deleted from the packet fixtures | FIXTURE FAIL self-structure: the fixtures drive a real leg and require the stderr record to be an observation expected 1 got 0 / FIXTURES pass=165 fai… |
| the end-to-end leg control deleted from the R5 job too | FIXTURE FAIL self-structure: the fixtures drive a real leg and require the stderr record to be an observation expected 1 got 0 / FIXTURES pass=163 fai… |
| RUNTIME_S booked at 9900, above the installed launcher cap (v6) | FIXTURE FAIL wall-gate-accepts-this-row expected OK got RUNTIME_S 9900 exceeds the installed calmcrab-udrv v4 preflight cap of 7200; the launcher woul… |
| …the same over-cap booking in the R5 job | FIXTURE FAIL wall-gate-accepts-this-row expected OK got RUNTIME_S 9900 exceeds the installed calmcrab-udrv v4 preflight cap of 7200; the launcher woul… |
| …and in the R6 job | FIXTURE FAIL wall-gate-accepts-this-row expected OK got RUNTIME_S 9900 exceeds the installed calmcrab-udrv v4 preflight cap of 7200; the launcher woul… |

(%d negative controls ran; every one bit. The v1 argv defects (mut1, mut2) are applied to the R4 packet and mut7 to
the R6 packet, because those command strings no longer exist in the other jobs; mut14 to R4 and R5; the v5 stream and
producer controls, the verdict-rule controls, and the v6 over-cap booking (mut15) are applied to all three jobs.
Each control mutates a COPY; the shipped packets are never modified.)

## What was NOT done

* nothing was executed: no claim run, no ssh, no srv1 contact, no partition or operator write;
* the packets were not staged, the units were not created, and no `systemd-run` or launcher call was made —
  the launcher line is the contract, and it is executed by keen-otter/sleek-owl when a grant exists;
* no execution grant is requested here; grants come only from keen-otter via sleek-owl-397;
* R6 is not seedable and stays HELD until keen-otter clears msg_3407a188;
* the draft PR stays a draft (the packets are evidence, not a code change to merge);
* no mutation leg is proposed for any row, so R4/R5 presence observations cannot discriminate a regression.

