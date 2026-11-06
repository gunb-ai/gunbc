# R4–R6 acceptance execution packets — v3, hardened srv1 frame (PREPARE ONLY)

Prepared by `fierce-wren-19` for `calm-eagle-670`, re-framed on keen-otter's hardened srv1 frame
(templates: #15938's `rdpo.sh` aaf32964184a30d66f6320de707bfb03975f0df683cccd1133dcf2d024ed9366 and rrc a1de55aa).
**Nothing here has been executed.** No ssh, no claim run, no partition write was performed in the preparing
container; the only modes run locally were `--classifier-fixtures` and `--predict-only`, which write nothing
outside a `mktemp -d` scratch dir and never touch a repository. Execution grants come only from keen-otter
via sleek-owl-397.

## What v3 changed, item by item (msg_e42861ba)

| # | requirement | what landed in both packets | asserted by |
|---|---|---|---|
| 1 | launch via the calmcrab-udrv v4 launcher, full argv, driver service, R/S/D ledger rows, stated bounds; no bare systemd-run | header `LAUNCHER (calmcrab-udrv v4)` block: the argv tuple `<JOB> <RUNTIME> <MEMMAX> <MEMHIGH> <TASKS> <CPUQ> <STAGED> <SHA> <OUT>` with each value spelled out; three ledger rows `<JOB>-res` / `<JOB>-drv` / `<JOB>`; units `<JOB>.scope` + `calmcrab-drv-<JOB>.service`; the script books nothing and refuses (2) if it was not started under that booking | self-check "the launcher's full argv is stated", "the row ledger rows are stated", "the driver unit is stated", "no bare systemd-run launch form"; packet self-structure `no bare systemd-run launch is spelled out` = 0 |
| 2 | pre-work bounds gate that COMPARES, not dumps | `BOOKED='memory.max=34359738368 memory.high=30064771072 memory.swap.max=0 pids.max=4096 cpu.max=3200000_100000'`; `bounds_check` reads `/sys/fs/cgroup$CG/{memory.max,memory.high,memory.swap.max,pids.max,cpu.max}` and compares each read value against the booked value (`max`/literal text/unparsable = mismatch); called as `BM=$(bounds_check "/sys/fs/cgroup$CG") || nox "booked bounds: $BM" bounds` before the first fetch and before any build; any mismatch → exit 2 | self-check runs the packet's own `bounds_check` against a synthesized cgroup dir with each key mismatched in turn, the literal `max` refused, an unreadable file refused, and an ordered check that the gate's call site precedes the first fetch; packet self-structure `the booked bounds are gated before any work` = 1 and `bounds-precedes-fetch`; negative control mut3 (gate deleted) → the packet fails its own fixtures |
| 3 | strict OOM as in rdpo: one snapshot per boundary, noclobber publication of raw bytes BEFORE parsing, each key exactly once as `^key [0-9]+$`, any read/write failure → UNKNOWN → NO_OBSERVATION, rise/fall per boundary plus final vs last-leg; no `grep '^oom_kill [1-9]'`, no silent cat | `oom_snap <events-file> <tag>` performs ONE read and publishes the raw bytes with `( set -C; printf '%s\n' "$t" > "$f" )` into `$OUT/mem/<NNN>-<ts>-<tag>.txt` before anything parses them; a read or publication failure writes `unreadable` with the same exclusive create and returns UNKNOWN; `oom_parse` demands every key appears exactly once matching `^key [0-9]+$` (duplicate, missing, tab, trailing field → UNKNOWN); `oom_pair_verdict` gives unchanged/rise/fall/UNKNOWN per boundary; boundaries: gate-before/gate-after, build-before/build-after, and per leg leg-before/leg-after; the final snapshot is compared against the LAST leg's after-snapshot (`LASTOOM` threaded) | self-check re-derives the helpers verbatim from the packet text and checks exclusive publication, byte-identical published file, unreadable/RO-dir/missing-dir → UNKNOWN, duplicate/tab/trailing-field → UNKNOWN, rise/fall/unknown verdicts, distinct sequence names per boundary; packet self-structure pins `set -C; printf` = 1, the unreadable marker = 1, `grep -c ^oom_kill` = 0, `dmesg`/`journalctl` = 0, `memory.events` ≥ 1; negative controls mut4 (noclobber removed) and mut6 (pair faked unchanged) → both fail the packet's own fixtures |
| 4 | exit contract: 0 expectations held, 1 bare FAIL (falsified), 2 NO_OBSERVATION; every setup or gate refusal is 2; no `exit 1 couldn't run`, no `exit 3` | `nox(){ ...; exit 2; }` is the only refusal path; usage, scope, bounds, clone, fetch, HEAD/tree, blob, name, build, timeout (124/137), OOM UNKNOWN, counter UNKNOWN, classifier shape and return-code disagreement are all 2; the single `exit 1` is the bare-FAIL falsification line; R6's no-grant refusal is now `exit 2` | self-check counts `exit 3` = 0 and `exit 1` = 1 on comment-stripped text of both packets, and requires ≥ 8 `exit 2` paths; packet self-structure repeats the same counts; negative control mut5 (a refusal switched to `exit 3`) → the packet fails its own fixtures |
| 5 | header states plainly that R4/R5 discriminate nothing | header line: `THIS ROW DISCRIMINATES NOTHING: presence observations, no mutation leg, none proposed.` plus the CLASSIFIER section's expectation list | self-check greps the header statement, and rejects any executable mention of `mutation` other than `mutation_legs=0` in the verdict line |
| 6 | wall accounting: capped sum of per-step timeouts vs the outer wall; size RuntimeMaxSec for the scope and the driver from it | R4/R5: clone 300 + fetch 300 + checkout 120 + build 2400 (×2 pins) + 3 legs × 900 = **8940 s capped sum**; scope `RuntimeMaxSec=9900`, driver `RuntimeMaxSec=10500`. R6: 300+300+120+2400+900 = **4020 s**; scope 4800, driver 5400. Every step runs under its own `timeout` | self-check recomputes the sum from the packets' timeout constants and compares against the header's stated scope/driver walls (arithmetic, not prose); packet self-structure requires the `timeout "$T_LEG"` / `"$T_BUILD"` / `"$T_SETUP_FETCH"` call sites to be present |

## Files

| path | bytes | sha256 |
|---|---|---|
| `tools/acceptance_r4_r5_claim_run_v3.sh` | 47666 | `176234a92458c5a6bd8c0b5d41fb56139e704d590a57944344399d7fd136cd41` |
| `tools/acceptance_r6_live_read_claim_run_v3.sh` | 42313 | `92da138e068ae525b51ee3100026ad9d9f7ea4c12090f9e0744ff1bfd9d1aa0e` |
| `tools/acceptance_r4r5_selfcheck_v3.sh` | 46362 | `32caff2fe702df3805ab695684f3538cb3b68f5a5613e821a1aa3ac24e890903` |

A script cannot contain its own hash: the launcher stages the file and compares its sha256 against this
manifest, and each packet prints `script_sha256=$(sha256sum "$0")` into its receipt as its first act.

## Frame shared by both packets

* **Launch.** `calmcrab-udrv` v4 with the full argv above. The packet takes exactly one argument,
  `OUT`, which must end in `/out`; `JOB` is derived as `basename(dirname(OUT))`, so `OUT` must be
  `$HOME/runs/<JOB>/out`. The scope gate refuses unless this process's cgroup path ends in `/<JOB>.scope`
  (read from `/proc/self/cgroup`), which is true only when the launcher put it there. No instance root such
  as `/opt/gunbc` is assumed anywhere: the run root is the single argument.
* **Bounds, as read.** `memory.max 34359738368` (32 GiB) and `memory.high 30064771072` (28 GiB) are the
  numbers rdpo booked on srv1 for a cold release build of this crate with `-j 32`; `memory.high` is a
  **ceiling, not a measurement** — claim_batch's own peak is not measured in this packet. `memory.swap.max 0`,
  `pids.max 4096` (threads, not processes), `cpu.max "3200000 100000"` = CPUQuota 3200 % **paired with
  `-j 32`** (srv1 reports nproc 128). The five files are read separately, so the quintuple is not an atomic
  snapshot; the gate compares each read value to its booked value and refuses on any mismatch.
* **Classifier.** `claim_batch` (blob `12a71786bd92f683dbd0aec2ae5d6f426e651a44`, byte-identical at both pins)
  prints `PASS <fn>` / `FAIL <fn>` bare for the counted outcomes, and `FAIL <fn> (runtime error …)`,
  `FAIL <fn> (returned ProcessExit::ExitFailure, code N: …)`, `UNOBSERVED <fn> (…`, `PANICKED <fn> (…`
  for the uncountable ones. Only a bare `PASS n` line with process exit 0 counts as held; a bare `FAIL n`
  with exit 1 counts as falsified; anything else — qualified shapes, extra lines, empty output, 124/137
  timeouts, missing file, PASS-but-exit-1 (the cost-accounting partition refused while every claim passed) —
  is NO_OBSERVATION and refuses 2. A leg whose own `--functions` names are not all present as `test fn <name>`
  in the checked-out entry file is refused before the claim is attempted.
* **Writes.** Everything lands under the fresh, exclusive `mkdir` of `OUT`: `receipt.txt`,
  `mem/NNN-<ts>-<tag>.txt`, `logs/{selfcheck.log,<leg>.out,<leg>.classes,<leg>.time}`, `tmp/`,
  `wt-r4|wt-r5|wt-r6/` (the pinned clones). `OUT` must not exist; an existing `OUT` refuses with 2 and
  leaves it untouched. No ledger, no partition, no repo file outside `OUT` is written.
* **One attempt.** Each pin is cloned, fetched, checked out, built and legged exactly once; there is no retry
  path in either packet.

## Row R4 + R5 (one packet, two pins) — `acceptance_r4_r5_claim_run_v3.sh`

* R4 pin `db56530fcb186370e3dbf1aa3c69ded324daa25d`, tree `48be84d8b18a9530e3a5c42adf8c24b19e1aa4dc`.
  Chosen as current `gunbc` main when the brief was issued. `origin/main` has since moved to
  `799ff9134ba509095b2dccc7912c36ead806d3bc`; the self-check verifies byte-identity of all four entry files
  and of `src/v1/stage0/src/bin/claim_batch.rs` at the frozen pin, at R5's pin and at current main, so
  re-pinning R4 to 799ff913 is a one-token edit if the grant set wants it. R5's pin is an ancestor of R4's.
* R5 pin `dff57af790f923066d61721e89ff751a91021278` (the #15936 squash), tree `356f2894a60edda51cdbe79879b2fb3b7aea75c4`.
* Entry blobs: `src/v2/test/claim/stage0_partition_rebuild_scope_witness.dag` `332cc981fd194bb9f02f16ff6e3c14d81b874e2b`;
  `src/v2/test/claim/rust_crate_partition_witness.dag` `abc6313eb8c4d49ad74a7bcafad36b895ea32f76`;
  `dag/test/claim/roadmap/roadmap_acceptance_inbox_witness.dag` `adfeb29426eac9d4da67630a12d89f8eabe8754b`.
* Legs: R4/a (5 names), R4/b (1 name, separate `--entry`), R5 (5 names). Expectation: every leg prints a bare
  `PASS n` and exits 0.
* **Class of the row: presence observations.** These claims assert that the partition/rebuild/inbox machinery
  refuses or admits the right things in a fresh read-only workspace; nothing in the row mutates a fixture, a
  partition or a receipt. **R4 and R5 discriminate nothing between a working and a broken implementation of
  the acceptance machinery beyond "the claims still hold at this pin"; no mutation leg is run or proposed in
  this packet.** The same statement is in the packet's header.

### Command inventory (printed by the packet itself with `--predict-only`; 11 commands)

```
PREDICT-ONLY row=R4+R5 job=fierce-wren-19-r4r5-db56530f host_booked=srv1 script_sha256_frozen=see-tools/acceptance_r4r5_v3_manifest.md
THIS ROW DISCRIMINATES NOTHING: presence observations, no mutation leg, none proposed.
pin_r4=db56530fcb186370e3dbf1aa3c69ded324daa25d tree_r4=48be84d8b18a9530e3a5c42adf8c24b19e1aa4dc
pin_r5=dff57af790f923066d61721e89ff751a91021278 tree_r5=356f2894a60edda51cdbe79879b2fb3b7aea75c4
blob src/v2/test/claim/stage0_partition_rebuild_scope_witness.dag 332cc981fd194bb9f02f16ff6e3c14d81b874e2b
blob src/v2/test/claim/rust_crate_partition_witness.dag abc6313eb8c4d49ad74a7bcafad36b895ea32f76
blob dag/test/claim/roadmap/roadmap_acceptance_inbox_witness.dag adfeb29426eac9d4da67630a12d89f8eabe8754b
bounds=memory.max=34359738368 memory.high=30064771072 memory.swap.max=0 pids.max=4096 cpu.max=3200000_100000
timeouts_s setup_clone=300 setup_fetch=300 setup_checkout=120 build=2400 leg=900
wall_capped_sum_s=8940 scope_RuntimeMaxSec=9900 driver_RuntimeMaxSec=10500
claims_r4a=5 claims_r4b=1 claims_r5=5
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

## Row R6 — HELD — `acceptance_r6_live_read_claim_run_v3.sh`

The read-path audit at pin `dff57af7…` (its header, lines 14–40):
`roadmap_acceptance_event_history_load` → `fabric_state_storage_binding`
(`dag/gunbc/fabric/fabric_storage_binding.dag:36`) → `roadmap_operator_acceptance_history_load` (same file:197);
`fabric_storage_placement` returns `FabricStoragePlaced` and `fabric_state_storage_binding` maps EVERY placed
branch to the live storage selection, so the branch a bare srv1 checkout binds is the live acceptance
partition, not a scratch one. That is exactly the read keen-otter's **msg_3407a188** restricts to the
*existing rostered reader*; a claim runner starting a NEW read of the live partition is not that reader.
So the row is **split out into its own packet and HELD**: R6 is not seed-only (the row's subject is the live
corpus), and it is not runnable as a presence observation until keen-otter clears msg_3407a188.

* Execution form differs by one argument: `script.sh <OUT> [<execution-grant-ref>]`.
  With no grant reference the packet refuses with **exit 2** (NO_OBSERVATION per the v3 contract) and does so
  **before anything is created** — no `OUT`, no clone, no receipt — while recording the reference it was given
  in `OUT/grant-ref.txt` when one is named. `grant_gate` refuses an empty reference, one padded with
  whitespace, and anything that looks like another flag.
* Pin `dff57af790f923066d61721e89ff751a91021278`, tree `356f2894a60edda51cdbe79879b2fb3b7aea75c4`; entry
  `dag/test/claim/roadmap/roadmap_acceptance_inbox_live_witness.dag` blob `969b980b189dafa81df4045f62f7d2e70dcc50ad`;
  one leg, `witness_inbox_on_authority_projects_on_live_corpus`. Expect PASS; a bare FAIL is a FINDING (exit 1)
  for the grant set, not a retry trigger.

### Command inventory (`--predict-only`; 5 commands)

```
PREDICT-ONLY row=R6 job=fierce-wren-19-r6-dff57af7 state=HELD gate=keen-otter-msg_3407a188 host_booked=srv1
pin=dff57af790f923066d61721e89ff751a91021278 tree=356f2894a60edda51cdbe79879b2fb3b7aea75c4
blob dag/test/claim/roadmap/roadmap_acceptance_inbox_live_witness.dag 969b980b189dafa81df4045f62f7d2e70dcc50ad
bounds=memory.max=34359738368 memory.high=30064771072 memory.swap.max=0 pids.max=4096 cpu.max=3200000_100000
timeouts_s setup_clone=300 setup_fetch=300 setup_checkout=120 build=2400 leg=900
wall_capped_sum_s=4020 scope_RuntimeMaxSec=4800 driver_RuntimeMaxSec=5400
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

## Inputs read

`/proc/self/cgroup`; `/sys/fs/cgroup/<scope-path>/{memory.max,memory.high,memory.swap.max,pids.max,cpu.max}`
and the same five plus `memory.events` under the run scope; `https://github.com/gunb-ai/gunbc` at the two
frozen pins (shallow, `--depth 1`, by sha); the pinned tree's `Cargo.lock`/workspace manifests for
`cargo build --locked --release -j 32 -p v1-compiler --bin claim_batch`; the three (R4/R5) / one (R6) entry
`.dag` files, byte-checked against the frozen blob ids and re-checked for the `test fn <name>` lines before
any claim is attempted. Nothing else: no live partition, no operator state, no roster.

## Self-test result

* `bash tools/acceptance_r4r5_selfcheck_v3.sh <repo>` → **417 passed, 0 failed, 0 skipped** (exit 0). It
  reads both packets' constants by name, runs each packet's embedded fixtures, extracts the helper region
  verbatim from the packet text (so it tests the shipped code, not a copy), and checks every v3 item in both
  directions — the correct behaviour and a deliberate wrong input for each. It also runs the shipped packets
  end-to-end up to the first gate (they refuse here at the tooling gate, creating nothing) and requires a
  relaxed copy to reach the scope gate, write a receipt with `VERDICT=NO_OBSERVATION reason=scope`, publish
  zero memory snapshots, create no clone, and refuse a reused `OUT` with exit 2.
* Embedded fixtures: **123 pass / 0 fail** in the R4/R5 packet, **125 pass / 0 fail** in the R6 packet
  (classifier shapes, stray-line detection, leg-state precedence, name gate, CSV shape, argv vs the pinned
  parser, OOM publication and parse contract, bounds comparison, scope gate, wall arithmetic, self-structure).
* Negative controls (each mutated copy must fail its own fixtures, or the checks are decorative):

| mutation | first fixture failure |
|---|---|
| a literal `claim_batch` token put back after the binary (v1 defect 1) | `FIXTURE FAIL argv-R4a-sound … argv carries a literal 'claim_batch' token` |
| a space-separated name in the CSV (v1 defect 2) | `FIXTURE FAIL csv-r4b …` |
| the booked-bounds gate deleted from the run path (item 2) | `FIXTURE FAIL self-structure: the booked bounds are gated before any work` |
| noclobber removed from the OOM publication (item 3) | `FIXTURE FAIL snap-parsed expected 0/0 got UNKNOWN/UNKNOWN` |
| a refusal switched from exit 2 to a fourth code (item 4) | `FIXTURE FAIL self-structure: no fourth exit code exists anywhere in the script` |
| the last-leg OOM pair faked as unchanged (item 3) | `FIXTURE FAIL self-structure: each leg boundary pair is compared, not merely recorded` |
| defect (1) reintroduced in the R6 packet | `FIXTURE FAIL argv-R6-sound …` |

## What was NOT done

No packet was executed. In the preparing container only `--classifier-fixtures` and `--predict-only` were
invoked (they never reach the run path; the run path itself refuses immediately here because `/usr/bin/time`
and `/<JOB>.scope` are absent). No ssh, no claim execution, no gunbc run, no partition write, no ledger write.
