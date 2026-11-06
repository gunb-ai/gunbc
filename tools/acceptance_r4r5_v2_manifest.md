# R4/R5 acceptance claim-run packets, v2 — command manifest and self-test record

Packets (this directory):

    tools/acceptance_r4r5_claim_run_v2.sh        — rows R4 and R5, srv1, seed-only
    tools/acceptance_r6_live_read_claim_run_v2.sh — row R6, srv1, HELD (live-store read, gated)
    tools/acceptance_r4r5_selfcheck_v2.sh         — the fixture self-test for both

Shape follows sunny-bee-153's map-literal packet v12 (gunbc#15934 comment 6672689625,
sha256 d303846d25679de0af9be478883561c9904531ad34b3fdc4aaa4dd05d9bab447): the packet carries its
launch contract, freezes the commit and entry-file blobs it was written against, gates its run
environment, keeps the command strings it prints and the command strings it executes in the same
variables, and reads `claim_batch`'s output as bare verdict lines only.

## v2 changes, against the acceptance check of the v1 packets (both defects were real)

1. No literal `claim_batch` subcommand token after the binary path. The binary IS claim_batch;
   argv[1] = `claim_batch` falls to the parser's `other =>` arm ("unknown argument", exit 2) and
   every leg would have died before observing anything.
2. `--functions` is CSV: one comma-separated argument, no spaces (`csv.split(',')` then trim). v1
   passed space-separated names inside one quoted argument, which asks for one function whose name
   contains spaces.
3. One run-location convention: the launcher passes exactly one argument, `$HOME/runs/<JOB>/out`
   (`systemd-run --user --scope --unit=<JOB> bash <packet> "$HOME/runs/<JOB>/out"`). The packet
   refuses a run dir that does not end in `/out`, derives `<JOB>` as `basename(dirname(<out>))`,
   requires this process's own cgroup line to end in `/<JOB>.scope`, and refuses if a second `out`
   dir ever appears under the booked job dir. No instance root is assumed or created anywhere:
   `/opt/gunbc` appears only in a comment saying it is not assumed.
4. New self-test assertions: each `CMD_LEG_*` is tokenized the way `bash -c` tokenizes it (same
   quoting, same variable expansion, `BIN`/`BIN5` pointed at a sentinel path) and the tokens are
   checked against the accepted flag set scraped from the pinned parser
   (`--source-root/--entry/--function/--functions/--claim-run/--wet/--hermetic/--record/
   --eval-budget-ms/--fixture-store/--pre-push/--print-entry-closure/--outcomes-out`); argv[1] is
   asserted not to be `claim_batch`; every `--functions` field must split on `,`, match
   `^[a-z0-9_]+$`, contain no space, and be exactly the frozen row set; `--entry` must equal the
   frozen entry constant; the packet must not parse-fail and must not assign an instance root.
5. R6 stays HELD, and now carries the same fixes. Its `--execute` without a grant reference
   refuses with exit 3 and names the gate it is held behind.

## Pins and blobs

    R4  pin db56530fcb186370e3dbf1aa3c69ded324daa25d   (gunbc main at freeze)
    R5  pin dff57af790f923066d61721e89ff751a91021278   (the #15936 squash; ancestor of the R4 pin)
    R6  pin dff57af790f923066d61721e89ff751a91021278   (same)

    332cc981fd194bb9f02f16ff6e3c14d81b874e2b  src/v2/test/claim/stage0_partition_rebuild_scope_witness.dag   (R4/a)
    abc6313eb8c4d49ad74a7bcafad36b895ea32f76  src/v2/test/claim/rust_crate_partition_witness.dag            (R4/b)
    adfeb29426eac9d4da67630a12d89f8eabe8754b  dag/test/claim/roadmap/roadmap_acceptance_inbox_witness.dag    (R5)
    969b980b189dafa81df4045f62f7d2e70dcc50ad  dag/test/claim/roadmap/roadmap_acceptance_inbox_live_witness.dag (R6)

Those four blob ids are identical at the R4 pin, at the R5 pin and at origin/main as of this run
(`origin/main` had advanced to 799ff9134ba509095b2dccc7912c36ead806d3bc; the self-test asserts both
that the pin is an ancestor of origin/main and that the four entry blobs are unchanged there, so
main moving does not change what this packet reads). The packet fetches `$PIN` by sha and refuses
unless `HEAD == $PIN`.

## What each row runs, and the mutation question

    R4/a  5 claims  --entry src/v2/test/claim/stage0_partition_rebuild_scope_witness.dag
    R4/b  1 claim   --entry src/v2/test/claim/rust_crate_partition_witness.dag
    R5    5 claims  --entry dag/test/claim/roadmap/roadmap_acceptance_inbox_witness.dag
    R6    1 claim   --entry dag/test/claim/roadmap/roadmap_acceptance_inbox_live_witness.dag   (HELD)

No row has a mutation leg, and none is proposed: the packet defines no `mutation` boundary, the row
sets name none, and `raw_verify` exits 2 if an `ev-mutation.raw` — or any R6-shaped boundary in the
R4/R5 packet, or any mutation/R4/R5-shaped boundary in the R6 packet — ever appears in the raw
evidence tree. Expected verdict for every claim: bare `PASS`.

## Command manifest (printed by `--predict-only`; the same variables are what the run evals)
```
PREDICT-ONLY: the R4/R5 seed-only packet (no mutation leg). Nothing runs.
pin_r4=db56530fcb186370e3dbf1aa3c69ded324daa25d
pin_r5=dff57af790f923066d61721e89ff751a91021278
blob src/v2/test/claim/stage0_partition_rebuild_scope_witness.dag 332cc981fd194bb9f02f16ff6e3c14d81b874e2b
blob src/v2/test/claim/rust_crate_partition_witness.dag abc6313eb8c4d49ad74a7bcafad36b895ea32f76
blob dag/test/claim/roadmap/roadmap_acceptance_inbox_witness.dag adfeb29426eac9d4da67630a12d89f8eabe8754b
launch: systemd-run --user --scope --unit=<JOB> bash acceptance_r4_r5_claim_run_v2.sh "$HOME/runs/<JOB>/out"
job_hint=acceptance-r4r5-srv1  (the packet derives <JOB> as basename(dirname(<out>)) and requires this process's cgroup to end in /<JOB>.scope)
expansion points: RUN_DIR=<the launcher's single argument>  BIN=<RUN_DIR>/wt-r4/target/release/claim_batch  BIN5=<RUN_DIR>/wt-r5/target/release/claim_batch
the bytes/sha256 below are over the LITERAL frozen string, before those are expanded
rows: R4/a=5 claims, R4/b=1 claim, R5=5 claims; expected verdict: PASS for all 11
no mutation leg in any row; R6 is split into its own HELD packet (live-store read, gated)
cmd CMD_CLONE_R4 bytes=68 sha256=2beca154480ec7b2082af9e3a371e1a9ba711e8bbd0a0e5ceea1723654b62470
  git clone --quiet --no-checkout --depth 1 "$ORIGIN" "$RUN_DIR/wt-r4"
cmd CMD_FETCH_R4 bytes=64 sha256=0e23acc3d909b789ddfbba6b2a703ff46a02f63cdaa58bb06e4f317c902248e9
  git -C "$RUN_DIR/wt-r4" fetch --quiet --depth 1 origin "$PIN_R4"
cmd CMD_CHECKOUT_R4 bytes=59 sha256=5b470c1e33e75c73cc7c55fb195b44bc8334698cac6f7a531cf64e520f583fc5
  git -C "$RUN_DIR/wt-r4" checkout --quiet --detach "$PIN_R4"
cmd CMD_BUILD_R4 bytes=73 sha256=8224e40c87d1b8af40db510d1891f3c2e9d14fd0897afa5dfbe6683a281f8fd5
  cargo build --locked --release -p v1-compiler --bin claim_batch --jobs 32
cmd CMD_LEG_R4A bytes=386 sha256=8768fc209294a47160ad6cf39999fcb6bac2e699abbf0b7f99513d40cf645a79
  "$BIN" --source-root dag --source-root src/v1 --source-root src/v2 --entry "src/v2/test/claim/stage0_partition_rebuild_scope_witness.dag" --functions "witness_duplicate_owner_refuses_holds,witness_unowned_mirror_refuses_holds,witness_shell_partition_duplicate_owner_refuses_holds,witness_leaf_edit_compiles_owner_and_downstream_holds,witness_leaf_edit_excludes_unrelated_packages_holds"
cmd CMD_LEG_R4B bytes=183 sha256=aa5801e74b689bf1352391d25070f7bb3d806a135f977eae485bbdc0c8164c4f
  "$BIN" --source-root dag --source-root src/v1 --source-root src/v2 --entry "src/v2/test/claim/rust_crate_partition_witness.dag" --functions "witness_r4_refuses_unit_graph_cycle_holds"
cmd CMD_CLONE_R5 bytes=68 sha256=eb8d7d848e55e3a5eb8fa6f37a0291ed3e1e433ac9c6a6d02bd409ebb8474b3a
  git clone --quiet --no-checkout --depth 1 "$ORIGIN" "$RUN_DIR/wt-r5"
cmd CMD_FETCH_R5 bytes=64 sha256=4bf7bc7e96d27c3355dbd3d43c39f518f45b1637631f5acbb5e0ce4c139e78aa
  git -C "$RUN_DIR/wt-r5" fetch --quiet --depth 1 origin "$PIN_R5"
cmd CMD_CHECKOUT_R5 bytes=59 sha256=23bd9badd8927d9c7e7c4dfb255197cef6073f01aa4b76efd741340ccb3e330a
  git -C "$RUN_DIR/wt-r5" checkout --quiet --detach "$PIN_R5"
cmd CMD_BUILD_R5 bytes=73 sha256=8224e40c87d1b8af40db510d1891f3c2e9d14fd0897afa5dfbe6683a281f8fd5
  cargo build --locked --release -p v1-compiler --bin claim_batch --jobs 32
cmd CMD_LEG_R5 bytes=397 sha256=c5f7fe87f6130cc3ad2cf1f60dc99be4457f0e81a419b7025f8bb81c674ceda7
  "$BIN5" --source-root dag --source-root src/v1 --source-root src/v2 --entry "dag/test/claim/roadmap/roadmap_acceptance_inbox_witness.dag" --functions "witness_inbox_load_refused_is_refused_not_empty,witness_inbox_unreplayable_history_is_refused_not_empty,witness_inbox_live_receipt_removes_node_from_rows,witness_inbox_revoked_receipt_reappears_with_prior,witness_inbox_stale_live_receipt_refuses"
```
### the held R6 packet
```
PREDICT-ONLY: the held R6 packet. Nothing is executed and the live store is not touched.
pin=dff57af790f923066d61721e89ff751a91021278
blob dag/test/claim/roadmap/roadmap_acceptance_inbox_live_witness.dag 969b980b189dafa81df4045f62f7d2e70dcc50ad
claim set: witness_inbox_on_authority_projects_on_live_corpus (1 claim, 1 leg; expected PASS; a refusal is a FINDING)
launch: systemd-run --user --scope --unit=<JOB> bash acceptance_r6_live_read_claim_run_v2.sh --execute <execution-grant-ref> "$HOME/runs/<JOB>/out"
job_hint=acceptance-r6-srv1
expansion points: RUN_DIR=<the launcher's single argument>  BIN=<RUN_DIR>/wt-r6/target/release/claim_batch
the bytes/sha256 below are over the LITERAL frozen string, before those are expanded
cmd CMD_CLONE bytes=68 sha256=04ac6423f9c48840f74d33d2fb3ca17221f1e3997b2201177e18368685658ac3
  git clone --quiet --no-checkout --depth 1 "$ORIGIN" "$RUN_DIR/wt-r6"
cmd CMD_FETCH bytes=61 sha256=84ce5ec42e01384bcfb3f42a486cae943c5ddc4bc34f42bbf87c1985fd6930c3
  git -C "$RUN_DIR/wt-r6" fetch --quiet --depth 1 origin "$PIN"
cmd CMD_CHECKOUT bytes=56 sha256=39c447782a96ceb5c6aeed8e4c9fbc27e9b4b5b6f388c8a031f37311e1b0c407
  git -C "$RUN_DIR/wt-r6" checkout --quiet --detach "$PIN"
cmd CMD_BUILD bytes=73 sha256=8224e40c87d1b8af40db510d1891f3c2e9d14fd0897afa5dfbe6683a281f8fd5
  cargo build --locked --release -p v1-compiler --bin claim_batch --jobs 32
cmd CMD_LEG_R6 bytes=206 sha256=cf0ddab1db69e3d23675b7572be33c3bf82bb52842eba0e821c925d8ece8b45b
  "$BIN" --source-root dag --source-root src/v1 --source-root src/v2 --entry "dag/test/claim/roadmap/roadmap_acceptance_inbox_live_witness.dag" --functions "witness_inbox_on_authority_projects_on_live_corpus"
```

Bytes and sha256 are over the literal frozen string, before `RUN_DIR`/`BIN`/`ORIGIN`/pin expansion;
expansion points are named in the header of each block. `CMD_BUILD_R4`, `CMD_BUILD_R5` and the R6
`CMD_BUILD` are the identical string, hence the identical sha256.

## Inputs read

    origin https://github.com/gunb-ai/gunbc.git — one --depth 1 clone per pin, then fetch of the pinned sha
    the pinned checkout: dag/, src/v1/, src/v2/ (claim_batch --source-root dag|src/v1|src/v2)
    srv1 cgroup files: memory.current, memory.peak, memory.min, memory.high, memory.max,
                       memory.events, memory.stat, cgroup.controllers, cgroup.subtree_control
    /proc/self/cgroup (the scope gate), hostname + /etc/machine-id (the bound-host gate)
    dmesg and `journalctl -k -q` (OOM lines; unreadable is recorded as such, not assumed clean)

## Writes

Every write is inside the run dir created by `mkdir -- "$RUN_DIR"` (no `-p`: an existing dir is
refused, exit 3), mode 700. Nothing is written outside it: no instance root is created, no
partition or store path is opened for writing, and no git push/commit/checkout-mutate happens.

    $RUN_DIR/raw/ev-<boundary>.raw      raw memory evidence, copied before any verdict is printed
    $RUN_DIR/raw/ev-<leg>.out           a copy of the leg output the classifier judged
    $RUN_DIR/raw/mem.events             the memory-evidence roll-up (read by the OOM gate)
    $RUN_DIR/logs/<leg>.log, build.log  stderr and build diagnostics
    $RUN_DIR/blob-gate.txt              the blob ids the gates observed
    $RUN_DIR/binding.txt                the bound-host gate's observation
    $RUN_DIR/wt-r4, wt-r5, wt-r6        the pinned clones (build target trees live inside them)
    $RUN_DIR/grant.txt                  R6 only: the grant reference the run was named with
    $RUN_DIR/tmp                        TMPDIR

## Exit codes

    0  every leg produced exactly the expected bare PASS/FAIL lines
    1  a leg could not run at all, or a build failed — nothing observed; one attempt, never retried
    2  NO_OBSERVATION: a non-verdict line, too few verdicts, an empty log, a missing or stowaway
       raw boundary, an entry blob that is not the frozen id, or an unusable argument
    3  an environment gate refused before or during the run: run dir reused, wrong scope, wrong
       host, an entry path untracked, HEAD off the pin, memory decided the run — or, for R6,
       `--execute` without a grant reference that has cleared the live-read gate

## Self-test

    $ bash tools/acceptance_r4r5_selfcheck_v2.sh <gunbc-checkout>
    SELF-TEST: 266 passed, 0 failed, 0 skipped        (exit 0)

Nothing from either repository is executed: the fixture feeds synthetic claim logs, synthetic run
dirs, synthetic cgroup and memory.events files and synthetic command lines to the packets' own gate
functions (extracted verbatim from between the FUNCTIONS markers) and to the packets themselves.
Constants are read out of the packets as VALUES, never restated, so the test cannot pass against a
packet it is not looking at.

Content checks are byte-identity gated: a working-tree file stands in for the pinned bytes only when
`git hash-object` of it equals the blob id at the pin, otherwise the check reports SKIPPED, never
passed. (This checkout is a blob:none partial clone: `git cat-file blob <pin>:<path>` lazily
fetches, which hangs on a credential prompt — pinned bytes are reached through local tree objects
plus hash-equality, and every git call gets /dev/null on stdin.)

Negative controls, run against deliberately mutated copies of the packet:

    defect (1) reintroduced (a literal `claim_batch` argv[1]) + a misspelled claim name
      -> 261 passed, 3 failed, exit 1
         "R4/a argv: argv[1] is the literal token 'claim_batch' (the parser's other => arm exits 2)"
         "R4/a: 'witness_leaf_edit_excludes_unrelated_packagez_holds' is not a test fn in …"
    defect (2) reintroduced (space-separated names in one --functions argument)
      -> 255 passed, 8 failed, exit 1
         "R5 argv: --functions name '…_not_empty …_not_empty' is outside ^[a-z0-9_]+$"
         "the R5 name list from the packet is empty or short (1)"
         "R5: '…' is not a test fn in dag/test/claim/roadmap/roadmap_acceptance_inbox_witness.dag …"

The R6 hold, in one line: at this pin a bare srv1 checkout binds the SERVED store branch, so
`witness_inbox_on_authority_projects_on_live_corpus` reads the live fabric acceptance partition over
the network from a NEW reader; keen-otter's msg_3407a188 admits an existing rostered reader only.
The audit and the claim-by-claim argument are in the header of the R6 packet. There is no seed-only
form of this claim without editing the claim or the store binding, which no grant in this set
authorizes.
