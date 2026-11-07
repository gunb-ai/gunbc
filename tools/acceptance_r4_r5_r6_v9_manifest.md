# R4, R5, R6 acceptance execution packets — v9: per-leg claim binding, hermetic fixtures, and a wall sum that admits every timeout (PREPARE ONLY)

Prepared by `fierce-wren-19` for `calm-eagle-670`, on keen-otter's hardened srv1 frame (templates: #15938's
`rdpo.sh` `aaf32964…` and rrc `a1de55aa`). **Nothing here has been executed.** No ssh, no claim run and no
partition write happened in the preparing container; the only modes run locally are `--classifier-fixtures`,
`--predict-only`, `--selfcheck` and the end-to-end gate refusals, which write nothing outside a scratch
directory and never touch a repository or a claim. Execution grants come only from keen-otter via
sleek-owl-397. **No row here is grantable by this document.** R6 stays HELD on keen-otter msg_3407a188.

## v8 → v9: what changed, and which assertion proves it (manager msg_6ad94d81, as extended by keen-otter msg_baaf8712; budgets per keen-otter msg_42c3c192)

| # | what keen-otter / the manager named | what landed in v9 | asserted by |
|---|---|---|---|
| 1 | **M1, `cs_check` did not bind a claim to its own leg.** A class file for leg A naming `a1 b1` and a file for leg B naming `b1` passed: the gate counted names across the whole set and per file, never `leg -> name` | every comparison is a **literal field comparison** (`awk -v l="$stem" -v n2="$n" '$1==l && $2==n2'`), never an interpolated regex. The gate now: checks the registration exists and is well formed (exactly two fields per line), that its leg names are exactly the booked legs, that every booked claim appears in it exactly once, that a classes file exists for each booked leg and none for an unbooked leg, that every line of every classes file is `<name> <STATE>`, that no name repeats **inside** a file or **across** files, that every name in a leg's file is registered **for that leg** (a claim registered elsewhere is `claim-wrong-leg`, a name never registered is `claim-unregistered`), and that each leg holds exactly the claims registered for it. On success it snapshots the validated files into `$OUT/cs-snapshot`, digests them, and re-reads the digests | packet fixtures `cs-claim-from-another-leg-exits-2`, `cs-claims-moved-between-legs-exit-2` (the literal `A='a1 b1', B='b1'` shape, both directions named), `cs-substituted-name-exits-2`, `cs-registration-naming-an-unbooked-leg-exits-2`, `cs-malformed-registration-line-exits-2`, `cs-malformed-classes-line-exits-2`, `cs-crossleg-*`, `cs-moved-*`, `cs-subst-*`, `cs-legset-*`, and the snapshot block (`cs-snapshot-validates-a-good-set`, `cs-snapshot-digests-match-the-validated-bytes`, `cs-tampered-live-set-refuses-revalidation`, `cs-snapshot-still-holds-the-validated-bytes`). The self-check inverts the three rules on copies of all three packets: `mut28` (per-leg binding relaxed to name-only), `mut29` (the cross-file duplicate scan removed), `mut30` (leg-set exactness relaxed to a count) — all bite |
| 2 | **M2, the fixture git needed a host identity.** `-c user.name/-c user.email` reached `git init` but not `git commit`; with an empty `HOME` the commit failed and two post-leg fixtures failed, so the run-path self-run reported NO_OBSERVATION on a machine with no git identity | one wrapper `gxf()` passes the identity on **every** call (`git -c user.name=fixture -c user.email=fixture@packet -c core.hooksPath=<empty dir> …`), runs with `HOME` pointed at an empty directory, `GIT_CONFIG_NOSYSTEM=1`, `GIT_CONFIG_GLOBAL=/dev/null`, and an empty `core.hooksPath`. Nothing reads or changes a srv1 git identity; the fixture repo lives under the scratch the self-run already owns | measured, not asserted: the three suites re-run under `env -i HOME=<fresh> PATH=/usr/bin:/bin` pass (R4 FIXTURES pass=292 fail=0, R5 FIXTURES pass=290 fail=0, R6 FIXTURES pass=294 fail=0), and the **inversion** — `gxf -C "$wt" commit` put back to plain `git -C "$wt" commit`, host config invisible — fails there, which the self-check requires. In the v8 packets the same empty-HOME run failed exactly the two predicted fixtures |
| 3 | **the wall sum was not the sum of the packet's own timeouts.** v8 counted neither the 300 s fixture self-run nor any kill grace: `timeout(1)` without `-k` sends TERM and then waits, so a child that ignores TERM left an unbounded tail outside the ceiling | every run-path timeout now carries `-k "$T_KILL"` (20 s), the run-path timeout count is a frozen constant (`N_TIMEOUTS_RUN`), and `WALL_SUM` adds `T_SELFRUN + N_TIMEOUTS_RUN*T_KILL`. **No booked number moved**: R4 stays 5400, R5 4500, R6 4800, and the per-step limits are byte-identical to v8 (keen-otter msg_42c3c192) | `ck wall-sum-arithmetic`, `ck wall-sum-counts-every-timeout-and-grace`, `ck wall-sum-counts-the-kill-graces`, `ck run-path-timeout-count`, `ck kill-grace-is-bounded`, `ck no-run-timeout-lacks-a-kill-grace`, `ck every-run-timeout-site-carries-a-kill-grace`, `ck wall-reserve-is-stated`, `ck wall-sum-inside-the-scope`, plus two `inv` controls that remove a `-k` from a leg and from the build; the self-check re-counts the timeouts per packet and refuses any run-path `timeout` with no grace; `mut32` (self-run and graces dropped from the sum) and `mut33` (every `-k` stripped) both bite on all three packets |
| 4 | **`final_verdict` re-read the live classes files** instead of what `cs_check` had just validated | `cs_check` snapshots each validated file and digests it; `final_verdict` verifies the snapshot digest set is complete, then classifies from `$OUT/cs-snapshot`. No snapshot: every claim NO_OBSERVATION, cause `snapshot-missing`, exit 2 | the snapshot fixtures above; static checks in the self-check that the `final_verdict` body contains the snapshot read and **no** `$OUT/*.classes` read; `mut31` (the snapshot write neutered) bites |
| 5 | the PY28 grep control **could not fail**: both directions grepped the mutated copy | replaced by `grepctl()`, which asserts the literal is present in the **shipped** packet and absent from the **mutated** copy, for all three packets | `grepctl` itself fails if the mutation does not apply, if the shipped packet lacks the literal, or if the copy still has it |
| 6 | minor: the `cs_check` comment described an enforcement it did not provide; the wall fixtures did not name the new terms; `final_verdict` re-hashed files | the comment states what the gate now enforces; the fixtures above; the verdict reads the snapshot | read in the shipped bytes; the harness pins the wording with `grep_in` needles |

## Bookings, bounds and wall arithmetic — no budget moved (keen-otter msg_42c3c192)

An increase over an approved timeout or budget needs operator sign-off; 7200 is the launcher's capability,
not an authorization; a never-granted proposal is not an approved ceiling; splitting, aggregating, retrying
or retuning a per-phase limit may not be used to get around a cap. v9 therefore keeps every booked number
exactly where it was — R4 `RUNTIME_S=5400`, R5 `4500`, R6 `4800`, and the per-step limits v8 froze (clone
300, fetch 300, checkout 120, build 2400, leg 900, self-run 300) are byte for byte unchanged — and makes
the two things v8 left out of the arithmetic into counted terms: the 300 s fixture self-run, and a 20 s
kill-after grace on every run-path timeout (without `-k`, `timeout(1)` sends TERM and waits, so a child
that ignores TERM made the "capped sum" a ceiling that was not closed).

| row | booked `RUNTIME_S` | the terms the packet admits | capped sum | reserve | launcher preflight cap | driver row `RuntimeMaxSec` |
|---|---|---|---|---|---|---|
| R4 | 5400 | 300 clone + 300 fetch + 120 checkout + 2400 build + 2 x 900 legs + 300 self-run + 7 x 20 kill graces | **5360** | 40 s | 7200 | 5700 |
| R5 | 4500 | 300 clone + 300 fetch + 120 checkout + 2400 build + 1 x 900 legs + 300 self-run + 6 x 20 kill graces | **4440** | 60 s | 7200 | 4800 |
| R6 (HELD) | 4800 | 300 clone + 300 fetch + 120 checkout + 2400 build + 1 x 900 legs + 300 self-run + 6 x 20 kill graces | **4440** | 360 s | 7200 | 5100 |

Nothing was shortened, so no `NO_OBSERVATION` risk is introduced by a shortened cap. The risk that the
stated reserve carries is written into each packet header next to the number: if the residual work after
the last timeout (startup, gate reads, evidence writes, receipt hashing) needs more than the reserve, the
scope's `RuntimeMaxSec` sends TERM and the driver row's `RuntimeMaxSec` (`RUNTIME_S` + 300 s) sends
SIGKILL; a row cut off mid-step is incomplete, which is `NO_OBSERVATION` (2) for that row's claims. R4's
reserve is the tightest at 40 s. If a row cannot fit inside its booked number, the answer is an operator
sign-off request with arithmetic, not a re-split or a retuned limit — and no row here needs one.

The launcher lines, verbatim, with the v9 packet digests (the v4 argument order `JOB RUNTIME_S MEMMAX_G
MEMHIGH_G TASKS CPU% STAGED_SCRIPT SCRIPT_SHA OUT`; memory in GiB, CPU with `%`, the sha of the packet
itself, and the literal `OUT` the launcher maps to `$HOME/runs/<JOB>/out`):

```
calmcrab-udrv.sh fierce-wren-19-r4-db56530f 5400 32 28 4096 3200% <STAGED_SCRIPT> 4cd48a2789795b1032efb5a2076166f9c2144c424289461747ef269c41005c4e OUT
calmcrab-udrv.sh fierce-wren-19-r5-dff57af7 4500 32 28 4096 3200% <STAGED_SCRIPT> 23d6a38fef81e2a77b4f807d6c2e7423ac57b3e44e9a46df62efd62858d17fc0 OUT
calmcrab-udrv.sh fierce-wren-19-r6-dff57af7 4800 32 28 4096 3200% <STAGED_SCRIPT> defd30b63dbace665bfb2f06a21c86cc7b4cbf5d87b9c2fab165fe78dce8b317 OUT   # HELD
```

The bounds the driver row must carry (the R ledger row cap in GiB, per keen-otter msg_c9d1df56):
`MemoryMax=32G MemoryHigh=28G TasksMax=4096 CPUQuota=3200%`. The packet re-reads `memory.max`,
`memory.high`, `memory.swap.max`, `pids.max` and `cpu.max` **before any fetch or build** and refuses on
any mismatch with those numbers; dumping them is not a gate.

## Pins and pinned objects

| row | pin (`HEAD` must equal this after clone+fetch) | what is blob-gated at that pin | blob availability in the measuring repo |
|---|---|---|---|
| R4 | `db56530fcb186370e3dbf1aa3c69ded324daa25d` | src/v2/test/claim/stage0_partition_rebuild_scope_witness.dag = `332cc981fd19` | local |
| R4 | `db56530fcb186370e3dbf1aa3c69ded324daa25d` | src/v2/test/claim/rust_crate_partition_witness.dag = `abc6313eb8c4` | local |
| R5 | `dff57af790f923066d61721e89ff751a91021278` | dag/test/claim/roadmap/roadmap_acceptance_inbox_witness.dag = `adfeb29426ea` | local |
| R6 | `dff57af790f923066d61721e89ff751a91021278` | dag/test/claim/roadmap/roadmap_acceptance_inbox_live_witness.dag = `969b980b189d` | local |
| ALL | `both pins (identical blob at each)` | src/v1/stage0/src/bin/claim_batch.rs = `12a71786bd92f683dbd0aec2ae5d6f426e651a44` | local |

Ancestry, as the v9 harness reports it in this measuring repo (a shallow `blob:none` partial clone):

* `the #15936 squash precedes the R4 freeze pin - dff57af790f923066d61721e89ff751a91021278..db56530fcb186370e3dbf1aa3c69ded324daa25d reads as a chain of 8 commit(s) in this object store`
* `the R4 freeze pin precedes origin/main - db56530fcb186370e3dbf1aa3c69ded324daa25d..7d062bda1b6dab67bc72e0c49d0eb635c45ca7f9 reads as a chain of 43 commit(s) in this object store`
* `the R5/R6 pin precedes origin/main - dff57af790f923066d61721e89ff751a91021278..7d062bda1b6dab67bc72e0c49d0eb635c45ca7f9 reads as a chain of 51 commit(s) in this object store`

calm-eagle-670 measured the same link independently in a full clone (`/tmp/gunbc`, `is-shallow=false`,
`origin/main 928966fc`): `git merge-base --is-ancestor dff57af7 db56530f` returned 0 (TRUE) and
`git rev-list --count dff57af7..db56530f` returned 8, and both pins are ancestors of `origin/main`.
The measuring repo here agrees: the first bullet above is a chain of **8** commits over exactly that
range, which is the number they obtained. The clone here is shallow and `blob:none`, which costs it
*objects*, not the commit graph between the pins - and where an object really is missing the v9 harness
reports UNRESOLVED (a skip, a non-zero exit), never a pass. That distinction is what v9 added: v8 read
pinned bytes out of a working copy and silently skipped when they were absent.

## Refusal-position audit (computed from the shipped bytes at manifest time)

Every function below is called as `X=$(fn …) || nox …` or `OG=$(fn …) || { … }`, so it decides by **exit
code**. Each has at least one non-zero return arm and no bare `return` arm; the self-check recomputes this
per packet, so the table cannot rot. Functions that decide by **value** (`oom_key`, `oom_pair_verdict`,
`pp_check`, `claim_classify`, `leg_rc_expect`, `stray_verdict_lines`, `leg_state`, `cs_check`) are compared
by their printed value at the call site and are not in this set — which is also why an empty predicate or
an empty verdict file is refused rather than counted.

| packet | functions in refusal position | non-zero arms / bare arms (computed from the shipped file) |
|---|---|---|
| R4 | `bin_sha`, `blob_gate`, `bounds_check`, `cs_check`, `names_gate`, `out_gate`, `producer_gate`, `register_claims`, `scope_gate`, `wall_gate` | `bin_sha` 2/0, `blob_gate` 2/0, `bounds_check` 3/0, `cs_check` 9/0, `names_gate` 2/0, `out_gate` 5/0, `producer_gate` 3/0, `register_claims` 8/0, `scope_gate` 3/0, `wall_gate` 3/0 |
| R5 | `bin_sha`, `blob_gate`, `bounds_check`, `cs_check`, `names_gate`, `out_gate`, `producer_gate`, `register_claims`, `scope_gate`, `wall_gate` | `bin_sha` 2/0, `blob_gate` 2/0, `bounds_check` 3/0, `cs_check` 9/0, `names_gate` 2/0, `out_gate` 5/0, `producer_gate` 3/0, `register_claims` 8/0, `scope_gate` 3/0, `wall_gate` 3/0 |
| R6 | `bin_sha`, `blob_gate`, `bounds_check`, `cs_check`, `grant_gate`, `names_gate`, `out_gate`, `producer_gate`, `register_claims`, `scope_gate`, `wall_gate` | `bin_sha` 2/0, `blob_gate` 2/0, `bounds_check` 3/0, `cs_check` 9/0, `grant_gate` 3/0, `names_gate` 2/0, `out_gate` 5/0, `producer_gate` 3/0, `register_claims` 8/0, `scope_gate` 3/0, `wall_gate` 3/0 |

## Evidence, reported exactly as obtained

| artifact | bytes | sha256 |
|---|---|---|
| `acceptance_r4_claim_run_v9.sh` | 119544 | `4cd48a2789795b1032efb5a2076166f9c2144c424289461747ef269c41005c4e` |
| `acceptance_r5_claim_run_v9.sh` | 118587 | `23d6a38fef81e2a77b4f807d6c2e7423ac57b3e44e9a46df62efd62858d17fc0` |
| `acceptance_r6_live_read_claim_run_v9.sh` | 112843 | `defd30b63dbace665bfb2f06a21c86cc7b4cbf5d87b9c2fab165fe78dce8b317` |
| `acceptance_r4_r5_r6_selfcheck_v9.sh` | 105949 | `0818f94b6a58160eb72f9bf7e9e7cf17e5b2d707d884456aa203874cea7b95d5` |

Embedded fixture suites, normal environment: R4 `FIXTURES pass=292 fail=0`, R5 `FIXTURES pass=290 fail=0`, R6 `FIXTURES pass=294 fail=0`.
Embedded fixture suites, `env -i HOME=$(mktemp -d) PATH=/usr/bin:/bin`: R4 `FIXTURES pass=292 fail=0`, R5 `FIXTURES pass=290 fail=0`, R6 `FIXTURES pass=294 fail=0`.
The self-check (`acceptance_r4_r5_r6_selfcheck_v9.sh`, with the repo argument): **1003 passed, 0 failed, 0 skipped, VERDICT=COMPLETE, rc=0**.
Pinned-blob materializations the self-check read from the object store in this run: 23, and no pinned blob was missing, so the UNRESOLVED path never fired here. What IS driven end-to-end by a fixture is the rule that any skip at all makes the summary read INCOMPLETE and exit non-zero (the fixture calls the shipped `skip()` and `report_state()` and asserts both). The unreadable-blob branch itself has no fixture - it needs an object store that lacks the blob, which this repo is not - and that gap is stated here rather than counted as coverage.
The `--predict-only` output, v8 to v9: 2 line(s) differ on R4, 2 on R5, 1 on R6 — the manifest path the header prints and the `wall_capped_sum_s=` value, nothing else. The command inventory itself is identical: 6 command strings and their digests match line for line on every row (asserted, not eyeballed). R5 carries 5 commands and R6 5, also identical.
End-to-end refusals on the measuring host (no clone, no build, no claim, nothing written): a missing tool gate stops R4 and R5 with rc 2; R6 refuses with rc 2 naming keen-otter msg_3407a188; a relative `OUT`, a non-existent `OUT`, and an `OUT` naming another job all refuse with rc 2 before any work.

## Reporting-honesty state of the self-check (this is a status, not a pass claim)

What has been measured, in which environment, and what has not been settled:

| run | environment | result |
|---|---|---|
| v8 packets, embedded fixtures | this container, full environment | R4 FIXTURES pass=260 fail=0, R5 FIXTURES pass=258 fail=0, R6 FIXTURES pass=262 fail=0 (rc 0) |
| v8 packets, embedded fixtures | `env -i HOME=$(mktemp -d) PATH=/usr/bin:/bin` | R4 FIXTURES pass=258 fail=2, R5 FIXTURES pass=256 fail=2, R6 FIXTURES pass=260 fail=2 (rc 1) — the two failures are `pg-ok-exits-0` and `pg-short-log-refuses-nonzero`, the M2 defect |
| v9 packets, embedded fixtures | full environment | R4 FIXTURES pass=292 fail=0, R5 FIXTURES pass=290 fail=0, R6 FIXTURES pass=294 fail=0 (rc 0) |
| v9 packets, embedded fixtures | `env -i HOME=$(mktemp -d) PATH=/usr/bin:/bin` | R4 FIXTURES pass=292 fail=0, R5 FIXTURES pass=290 fail=0, R6 FIXTURES pass=294 fail=0 (rc 0) |
| v9 self-check | full environment, repo argument given | 1003 passed, 0 failed, 0 skipped -> VERDICT=COMPLETE (rc 0) |
| v9 self-check | re-run inside itself under `env -i` | 1002 passed, 0 failed, 0 skipped -> VERDICT=COMPLETE (rc 0) |
| v9 self-check | the whole harness launched under `env -i HOME=$(mktemp -d) PATH=/usr/bin:/bin`, repo argument given | 1003 passed, 0 failed, 0 skipped -> VERDICT=COMPLETE (rc 0) |

sleek-owl-397 measured the **v8** self-check at **903 passed / 1 failed / 19 skipped — INCOMPLETE**, in
a shallow clone whose head was `99e16ed1` fetched by sha, under `env -i PATH=/usr/bin:/bin` with a fresh
empty `HOME` and `TMPDIR`, no network, and the repo argument not recorded. The one failure was the
ancestry row (`the R5 pin (the #15936 squash) is an ancestor of the R4 pin (main at freeze)`), and the 19
skips were, exactly as they listed them: 1 comparison of pinned bytes against the **working copy**
(`claim_batch.rs`, the shape v9 removes), 5 pinned bytes of `stage0_partition_rebuild_scope_witness.dag`
unreadable offline, 5 of `roadmap_acceptance_inbox_witness.dag`, 1 of `roadmap_acceptance_inbox_live_witness.dag`,
and 7 from the R6 read-path audit (`roadmap_dashboard_instance.dag` x2, `roadmap_operator_acceptance.dag`,
`fabric_event_log.dag` x2, `fabric_storage_client.dag`, `roadmap_acceptance_inbox_live_witness.dag`).
**That is the state of their run and it is reported as INCOMPLETE, not smoothed into a pass.** In this
measuring repo those same pinned-byte comparisons run against pinned blobs (23 materializations, 0 skips).
The v9 harness changes two things about that shape:
pinned bytes are compared blob-to-blob (`git rev-parse <pin>:<path>` then `cat-file`, never the working
copy, so a branch checkout cannot make a check vanish), and any skip at all makes the summary say
INCOMPLETE and exit non-zero - a fixture drives that rule end-to-end through the shipped `skip()` and
`report_state()`. A blob the object store cannot produce offline is reported UNRESOLVED; unavailable
content is never reported as a pass, and this manifest does not claim one.

Still UNRESOLVED in the measuring repo, with the reason: nothing in this run: every pinned blob and both ancestry links were readable here (the repo attestation line above says exactly which repo and what kind of clone it is), so the skips did not fire.
Not settled by anything in this document: everything that only a granted srv1 run can answer - whether the R4/R5 claims actually hold, and whether the R6 read path touches the live partition (which is why R6 stays HELD on keen-otter msg_3407a188). Nothing in this file is evidence about those questions.

## History: what v8 changed (manager msg_8ed9b50b: one fail-open in `final_verdict`)

*The block below is the v8 front matter, kept as history. Its file names are the **v9** names,
because every freeze renames the files; its digests and its quoted measurements are the v8 ones,
and where those numbers differ from the v9 table above, the v9 table is the current one.*

# R4, R5, R6 acceptance execution packets — v8: the exit decision is anchored to a registered claim set (PREPARE ONLY)

Prepared by `fierce-wren-19` for `calm-eagle-670`, on keen-otter's hardened srv1 frame (templates: #15938's
`rdpo.sh` `aaf32964…` and rrc `a1de55aa`). **Nothing here has been executed.** No ssh, no claim run, no
partition write happened in the preparing container; the only modes run locally are `--classifier-fixtures`
and `--predict-only`, which write nothing outside a `mktemp -d` scratch and never touch a repository.
Execution grants come only from keen-otter via sleek-owl-397. **No row here is grantable by this document.**

v8 changes ONE thing, and changes nothing else: the v7 packets summed whatever `$OUT/*.classes` files
happened to exist and could reach `VERDICT=AS_EXPECTED` with `claims=0` — a green verdict from an empty
run. v8 registers the booked claim set into `$OUT/claims-registered.txt` from the frozen constants before
the first leg, and `final_verdict` refuses unless the classes files are exactly that set. The three
bookings, their bounds, their wall arithmetic and their command inventory are byte-identical to v7:
the `--predict-only` diff v7 → v8 is empty except for the manifest file name it prints.

## v7 -> v8: what changed, and which assertion proves it (manager msg_8ed9b50b)

| # | the fail-open | what landed | asserted by |
|---|---|---|---|
| 1 | `final_verdict` looped over whatever `$OUT/*.classes` files existed. With none at all — or one leg's file missing or short — it summed nothing, printed `VERDICT=AS_EXPECTED` and exited 0. `nclaims=0` made every-claim-held true vacuously, so a run that produced no evidence at all was reported as as-expected | the booked claim set is declared once from the frozen claim-name constants (`LEGS_BOOKED`, `CLAIMS_<leg>`, `CLAIMS_ALL`, `N_CLAIMS_BOOKED`), written into `$OUT/claims-registered.txt` by `register_claims` in the run path **before** the first `leg` call (and the writer refuses a registration that disagrees with its own constants), and `final_verdict` consults `cs_check` **before** it reads any classes file: one file per booked leg, no file for an unbooked leg, every booked claim named exactly once, no unregistered name, and the registration re-checked against the packet's frozen constants. Any breach: every claim NO_OBSERVATION, `claim-set refused:<tokens>` in the receipt, exit 2 |
|   |  |  | packet fixtures `cs-complete-set-is-countable` (0), `cs-no-classes-files-exits-2`, `cs-one-leg-file-missing-exits-2`, `cs-claim-missing-from-a-file-exits-2`, `cs-empty-classes-file-exits-2`, `cs-duplicate-claim-exits-2`, `cs-extra-unbooked-claim-exits-2`, `cs-extra-classes-file-exits-2`, `cs-tampered-registration-exits-2`, `cs-missing-registration-exits-2` (all 2, each naming its reason token), `cs-complete-set-with-a-fail-is-still-1` (a real refutation is not swallowed), `cs-writer-*` (the writer refuses sets it cannot produce); the self-check re-runs the shipped `cs_check` on synthetic states (`csx` controls) and runs three mutation controls per packet — neutering `cs_check`, removing the classes-file count, and never writing the registration — each of which must break the packet's own fixtures |

The self-check also proves, computed rather than grepped, that each packet's booked claim set is exactly
the set its leg commands carry, that the registration call precedes the first leg in the run path, and
that `cs_check` is consulted before any summation (`lsum` is taken after `final_verdict`'s own line, so
`cs_check`'s own loop cannot satisfy the ordering check).

## The exit contract, verbatim as pre-registered by the acceptance owner (calm-eagle-670, msg_ac20c593)

> A leg is **OBSERVED** iff a completion record is present AND every claim in the leg has exactly one bare
> verdict line AND there are no stray verdict-shaped lines AND `rc == 0` when every claim PASSes, else
> `rc == 1`. Within an observed leg a bare PASS is HELD and a bare FAIL is FALSIFIED, per claim. Anything
> else (an rc inconsistent with the verdicts, a missing or duplicate verdict, a reasoned FAIL, a timeout,
> an OOM) makes EVERY claim in that leg NO_OBSERVATION.
> Packet exit: **1** if any claim is FALSIFIED, else **2** if any claim is NO_OBSERVATION, else **0**.

FALSIFIED therefore outranks NO_OBSERVATION: a bare FAIL inside a well-formed leg is a real result, so an
uninformative second leg does not cancel it. Classification is per CLAIM: a leg that is not observed marks
its own claims NO_OBSERVATION and lets the other legs stand. The same block is quoted verbatim in all three
packet headers, and the fixtures drive it end to end (see the `contract-*` fixtures and the self-check's
`fv_*` controls, which assert the exit code, not a description of it).

## History — v6 -> v7: what changed, and which assertion proved it (carried unchanged into v8)

| # | change (manager msg_ac20c593, from sleek-owl-397 msg_9a42d63c) | what landed | asserted by |
|---|---|---|---|
| 1 | the verdict named another packet's state variables (`S5` in the R4 job, `S4A`/`S4B` in the R5 job) — under `set -u` that is exit 127 arriving in the middle of the verdict, after every probe was spent | the verdict is one function, `final_verdict`, that reads only this packet's `*.classes` files and its own `post-leg.cause`; no non-comment line in any packet names `S4A`, `S4B`, `S5` or `S6` at all | the self-check greps every packet for those names outside comments (must be 0), and drives each packet's own `final_verdict` end to end: all-held → 0, one falsified → 1, one unobserved → 2, falsified+unobserved → 1, post-leg cause → 2 |
| 2 | `wall_gate` and `producer_gate` printed a refusal and returned 0, so `WM=$(wall_gate …) || nox …` never fired — the refusal existed only as a string in a variable | every refusal arm in every gate returns non-zero | packet fixtures assert the exit status of each arm (`wall-gate-refuses-*`, `producer-gate-*-refuses-nonzero`, `out_gate-refusals-exit-nonzero`); the self-check audits **every** function used in refusal position (`X=$(fn …) || nox …` / `|| { … }`) and fails if any has a bare-return arm; mutation 17 and 18 restore the v0 defect in `wall_gate` / `producer_gate` and must break the fixtures |
| 3 | multi-claim legs must be classified per claim, not per leg | `leg_rc_expect` (exit code vs the leg's whole bare-verdict set), `claim_classify` (per claim, only inside an observed leg), and `leg()` which records the reasons a leg was not observed and marks every claim in it NO_OBSERVATION | fixtures `contract-pass-and-fail-at-rc1` (a HELD, b FALSIFIED, exit 1), `contract-all-pass-at-rc1` (NOOBS), `contract-pass-and-fail-at-rc0` (NOOBS), plus reasoned-fail / missing / duplicate / timeout cases; mutations 16 and 20 weaken the rc agreement and must break them, mutation 21 inverts the precedence and must break `falsified-outranks-no-observation` |
| 4 | post-leg checks | `post_leg_gate <worktree> <runner> <pre-leg sha256>`: `git status --porcelain` must be empty and the runner must hash to the bytes hashed before the legs; a cause turns **every** claim in the run into NO_OBSERVATION | packet fixtures `post-leg-clean-*`, `post-leg-dirty-worktree-refuses`, `post-leg-runner-changed-refuses`, `post-leg-dirty-forces-every-claim-nood` (a bare FAIL becomes exit 2); self-check `plg_*` controls on the shipped helpers; mutations 23 and 24 disable each half and must break them |
| 5 | the six non-blocking items | the one argument is gated by `out_gate` (absolute, `<JOB>/out` shape, the booked job name); the fixture self-run runs under `timeout $T_SELFRUN` (300 s, inside the wall margin); `CARGO_HOME=$OUT/cargo-home`; `stray_verdict_lines` fails closed on an unreadable file; the gate/build/teardown OOM labels carry the verdict they got (`gate-read-UNKNOWN`, `…-COUNTER_INCOHERENT`) instead of guessing | `out_gate` behavioural controls in both the packets and the self-check; structural greps for `timeout "$T_SELFRUN"`, `export CARGO_HOME=$OUT/cargo-home`, `gate-read-$GV`; mutation 22 removes the relative-OUT refusal and must break the fixtures |
| 6 | keep the command inventory byte-identical, same wall arithmetic | `--predict-only` output is **identical** to v6 for all three packets (0 differing lines); `RUNTIME_S` 5400/4500/4800, capped sums 4920/4020/4020, drivers 5700/4800/5100 | `diff` of the `--predict-only` output v6 vs v7 is empty, asserted in the build; `wall_gate` is called with those constants and refuses on any mismatch |

## Refusal-position audit (computed from the shipped bytes at manifest time; includes v8's two new gates)

Every function below is called as `X=$(fn …) || nox …` or `OG=$(fn …) || { … }`, so it decides by **exit
code**. Each has at least one non-zero return arm and no bare `return` arm; the self-check recomputes this
per packet, so the table cannot rot. Functions that decide by **value** (`oom_key`, `oom_pair_verdict`,
`pp_check`, `claim_classify`, `leg_rc_expect`, `stray_verdict_lines`, `leg_state`) are compared by their
printed value at the call site and are not in this set — which is also why an empty predicate or an empty
verdict file is refused rather than counted.

| packet | functions in refusal position | non-zero arms / bare arms (computed from the shipped file) |
|---|---|---|
| R4 | `bin_sha`, `blob_gate`, `bounds_check`, `cs_check`, `names_gate`, `out_gate`, `producer_gate`, `register_claims`, `scope_gate`, `wall_gate` | `bin_sha` 2/0, `blob_gate` 2/0, `bounds_check` 3/0, `cs_check` 9/0, `names_gate` 2/0, `out_gate` 5/0, `producer_gate` 3/0, `register_claims` 8/0, `scope_gate` 3/0, `wall_gate` 3/0 |
| R5 | `bin_sha`, `blob_gate`, `bounds_check`, `cs_check`, `names_gate`, `out_gate`, `producer_gate`, `register_claims`, `scope_gate`, `wall_gate` | `bin_sha` 2/0, `blob_gate` 2/0, `bounds_check` 3/0, `cs_check` 9/0, `names_gate` 2/0, `out_gate` 5/0, `producer_gate` 3/0, `register_claims` 8/0, `scope_gate` 3/0, `wall_gate` 3/0 |
| R6 | `bin_sha`, `blob_gate`, `bounds_check`, `cs_check`, `grant_gate`, `names_gate`, `out_gate`, `producer_gate`, `register_claims`, `scope_gate`, `wall_gate` | `bin_sha` 2/0, `blob_gate` 2/0, `bounds_check` 3/0, `cs_check` 9/0, `grant_gate` 3/0, `names_gate` 2/0, `out_gate` 5/0, `producer_gate` 3/0, `register_claims` 8/0, `scope_gate` 3/0, `wall_gate` 3/0 |

## History: why the rows are one job each (v5 -> v6, manager msg_384bca0f)

v6 splits the R4+R5 packet into **two single-row jobs** because the installed launcher refuses the booking:
`calmcrab-udrv` v4 (`855920ac`) rejects any `RUNTIME_S` above 7200 in its own preflight, and the v5 combined
job needed `setup 720 + build 2400 + 4 legs x 900 + margin = 8940 s` — a `PRE_SCOPE_REFUSAL`, the same shape
as #15695 diag-v2 booked at 11700. Changing the launcher is out of scope, so the *rows* move, not the launcher.

## v5 -> v6: what changed, and which assertion proves it

| # | change (manager msg_384bca0f, from sleek-owl-397 msg_995f146b) | what landed | asserted by |
|---|---|---|---|
| 1 | split R4/R5 into two jobs, R4 first then R5; each its own clone/fetch/checkout/build/legs | three packets: `acceptance_r4_claim_run_v9.sh` (`fierce-wren-19-r4-db56530f`), `acceptance_r5_claim_run_v9.sh` (`fierce-wren-19-r5-dff57af7`), `acceptance_r6_live_read_claim_run_v9.sh` (`fierce-wren-19-r6-dff57af7`) | the self-check greps each packet for its own `J=` job token, and asserts no job clones or builds the other row's pin (`pin_clone "$WT5"` absent from R4, `pin_clone "$WT4"` absent from R5, `wt-r4` absent from R6) |
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
| `tools/acceptance_r4_claim_run_v9.sh` | 119544 | `4cd48a2789795b1032efb5a2076166f9c2144c424289461747ef269c41005c4e` |
| `tools/acceptance_r5_claim_run_v9.sh` | 118587 | `23d6a38fef81e2a77b4f807d6c2e7423ac57b3e44e9a46df62efd62858d17fc0` |
| `tools/acceptance_r6_live_read_claim_run_v9.sh` | 112843 | `defd30b63dbace665bfb2f06a21c86cc7b4cbf5d87b9c2fab165fe78dce8b317` |
| `tools/acceptance_r4_r5_r6_selfcheck_v9.sh` | 105949 | `0818f94b6a58160eb72f9bf7e9e7cf17e5b2d707d884456aa203874cea7b95d5` |
| `tools/acceptance_r4_r5_r6_v9_manifest.md` | (this file) | stated in the PR comment and the message to calm-eagle-670, because a file cannot carry its own digest |

The launcher is handed the **staged script's** sha256 as argv[8]: that is the sha256 of the packet file above
(the one the launcher verifies against the staged copy), not a source-commit sha. `script_sha256_frozen=see-tools/…`
in `--predict-only` is deliberate: the packet states where its own digest is recorded.

## The three launcher lines, and the arithmetic behind each

All three use the calmcrab-udrv v4 argv in the launcher's order, `JOB RUNTIME_S MEMMAX_G MEMHIGH_G TASKS CPU% STAGED_SCRIPT SCRIPT_SHA OUT`,
memory in GiB, CPU with `%`, `OUT` as the literal token; memory bounds are the R row, the driver service is one task
(S row), and the D row is the scope itself (`cap 0`):

```
calmcrab-udrv.sh fierce-wren-19-r4-db56530f 5400 32 28 4096 3200% <STAGED_SCRIPT> 4cd48a2789795b1032efb5a2076166f9c2144c424289461747ef269c41005c4e OUT
calmcrab-udrv.sh fierce-wren-19-r5-dff57af7 4500 32 28 4096 3200% <STAGED_SCRIPT> 23d6a38fef81e2a77b4f807d6c2e7423ac57b3e44e9a46df62efd62858d17fc0 OUT
calmcrab-udrv.sh fierce-wren-19-r6-dff57af7 4800 32 28 4096 3200% <STAGED_SCRIPT> defd30b63dbace665bfb2f06a21c86cc7b4cbf5d87b9c2fab165fe78dce8b317 OUT   # HELD
```

| job | clone | fetch | checkout | build | legs | self-run | kill graces | capped sum | scope `RuntimeMaxSec` | launcher cap | driver `RuntimeMaxSec` |
|---|---|---|---|---|---|---|---|---|---|---|---|
| R4 `fierce-wren-19-r4-db56530f` | 300 | 300 | 120 | 2400 | 2 x 900 = 1800 | 300 | 7 x 20 = 140 | **5360** | 5400 | 7200 | 5700 |
| R5 `fierce-wren-19-r5-dff57af7` | 300 | 300 | 120 | 2400 | 1 x 900 = 900 | 300 | 6 x 20 = 120 | **4440** | 4500 | 7200 | 4800 |
| R6 `fierce-wren-19-r6-dff57af7` (HELD) | 300 | 300 | 120 | 2400 | 1 x 900 = 900 | 300 | 6 x 20 = 120 | **4440** | 4800 | 7200 | 5100 |

Every job satisfies `sum <= RUNTIME_S <= 7200` and `driver = RUNTIME_S + 300`. The two costs v8 did not count
- the bounded `--classifier-fixtures` self-run, and the 20 s `timeout -k` grace that belongs at each run-path
`timeout` call site - are absorbed **inside the unchanged bookings**, per keen-otter msg_42c3c192: no booked
number moved, no per-phase limit was shortened, and the job was not re-split or given retries. The margin
over the capped sum is therefore **40 s (R4), 60 s (R5), 360 s (R6)**, down from 480/480/780. R4’s 40 s is
the binding constraint and its risk is stated in the packet header beside the number: work still running when
it expires is cut short by the scope’s `RuntimeMaxSec` (`TERM`) and by the driver row’s
(`RUNTIME_S`+300, `SIGKILL`), and an incomplete row is NO_OBSERVATION (2), never a guess.
The R4 job runs first, then R5; each does its own clone, fetch, checkout and
`cargo build --locked -p claim_batch`, so nothing is shared between the two bookings.

Booked bounds, identical in all three (the packets refuse on any mismatch **before** any fetch or build):
`memory.max=34359738368 memory.high=30064771072 memory.swap.max=0 pids.max=4096 cpu.max=3200000_100000`
(32 GiB / 28 GiB / no swap / 4096 tasks / 3200%), derived from the same `MEMMAX_G MEMHIGH_G TASKS_MAX CPU_PCT`
constants that go into the launcher argv, so the packet and the launcher cannot disagree.

## What the rows do (and do not) show

R4 and R5 **discriminate nothing**: they are presence observations with no mutation leg, and no mutation leg is
proposed here. A pass says the witness claims exist and hold in the pinned tree; a bare `FAIL` would falsify
that. R6 asks whether the roadmap acceptance inbox projects on a live corpus; a refusal there is a FINDING,
and R6 is HELD (below).

## Row R4 — `tools/acceptance_r4_claim_run_v9.sh` (job `fierce-wren-19-r4-db56530f`)

* pin `db56530fcb186370e3dbf1aa3c69ded324daa25d` (the grant's R4 pin; tree `48be84d8b18a9530e3a5c42adf8c24b19e1aa4dc`);
  entry blobs `332cc981fd194bb9f02f16ff6e3c14d81b874e2b` (`src/v2/test/claim/stage0_partition_rebuild_scope_witness.dag`)
  and `abc6313eb8c4d49ad74a7bcafad36b895ea32f76` (`src/v2/test/claim/rust_crate_partition_witness.dag`).
* two legs, one per entry file: `R4/a` five witness claims in the scope-witness dag, `R4/b` `witness_r4_refuses_unit_graph_cycle_holds`
  in the partition-witness dag. Expect **all PASS**; the row is HELD only if every name has exactly one bare `PASS`.
* v9: `cs_check` binds every booked claim to the **leg** that booked it. The classes file for `R4/a` must hold
  exactly the five names registered under `R4/a` and the file for `R4/b` the one registered under `R4/b`;
  a name registered under another leg is `claim-wrong-leg:<leg>:<name>`, a name no leg registers is
  `claim-unregistered:<name>`, a registered name its own file does not carry exactly once is
  `claim-count:<leg>`, a name in two leg files is `claim-duplicated-across-files`, and a registration naming
  a leg that was never booked is `registration-leg-count` - even when the counts match. All comparisons are
  literal awk field comparisons (`$1==leg && $2==name`); no claim name is ever interpolated into a regex.
  When it passes, `cs_check` copies and digests the files it validated into `$OUT/cs-snapshot`, and
  `final_verdict` counts **that snapshot** - the bytes the gate saw are the bytes the verdict counts.
* v9: the fixture git is hermetic. Every git call in the fixture scope carries the identity on **init and
  commit**, runs with `HOME` on an empty directory, `GIT_CONFIG_NOSYSTEM=1`, `GIT_CONFIG_GLOBAL=/dev/null`,
  `core.hooksPath` on an empty dir, under `env -i` with only `PATH`/`TMPDIR`. Nothing reads or writes an
  srv1 git identity. Without it the post-leg fixtures cannot commit on a host with no git identity, and a
  run-path self-check there reports NO_OBSERVATION - which is why v8’s suite fails 2 fixtures per packet
  under `env -i` and v9’s does not.
* `--predict-only` prints exactly the six command strings (four setup + two legs), each with bytes and sha256:

```
PREDICT-ONLY row=R4 job=fierce-wren-19-r4-db56530f host_booked=srv1 script_sha256_frozen=see-tools/acceptance_r4_r5_r6_v9_manifest.md
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
wall_capped_sum_s=5360 scope_RuntimeMaxSec=5400 driver_RuntimeMaxSec=5700 launcher_cap=7200 legs=2
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

## Row R5 — `tools/acceptance_r5_claim_run_v9.sh` (job `fierce-wren-19-r5-dff57af7`)

* pin `dff57af790f923066d61721e89ff751a91021278` (the #15936 squash; tree `356f2894a60edda51cdbe79879b2fb3b7aea75c4`);
  entry blob `adfeb29426eac9d4da67630a12d89f8eabe8754b` (`dag/test/claim/roadmap/roadmap_acceptance_inbox_witness.dag`).
* one leg, five inbox-witness claims. Expect **all PASS**.
* no mutation leg; the row discriminates nothing.
* v7: the legs are classified per claim; a leg whose exit code disagrees with its own verdict lines, or whose
  completion record is missing, marks its claims NO_OBSERVATION without touching the other leg;
  `post_leg_gate` then requires the worktree to be clean and the runner to still hash to the bytes hashed
  before the legs, or every claim in the row becomes NO_OBSERVATION. (This bullet appeared three times in
  the v7 and v8 manifests - an artifact of the assemble passes; it is stated once here.)
* v9: the five inbox claims are bound to the one booked leg by name-and-leg pairs, with the same literal
  field comparisons, the same cross-file uniqueness rule, and the same snapshot that `final_verdict` counts.
* v9: the fixture git is hermetic (see R4); the suite runs twice in the self-check, normally and under
  `env -i HOME=$(mktemp -d) PATH=/usr/bin:/bin`, and both runs must pass.

```
PREDICT-ONLY row=R5 job=fierce-wren-19-r5-dff57af7 host_booked=srv1 script_sha256_frozen=see-tools/acceptance_r4_r5_r6_v9_manifest.md
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
wall_capped_sum_s=4440 scope_RuntimeMaxSec=4500 driver_RuntimeMaxSec=4800 launcher_cap=7200 legs=1
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

## Row R6 — HELD — `tools/acceptance_r6_live_read_claim_run_v9.sh` (job `fierce-wren-19-r6-dff57af7`)

* same pin and tree as R5; entry blob `969b980b189dafa81df4045f62f7d2e70dcc50ad`
  (`dag/test/claim/roadmap/roadmap_acceptance_inbox_live_witness.dag`), one claim `witness_inbox_on_authority_projects_on_live_corpus`.
* **HELD behind keen-otter msg_3407a188** (existing rostered reader only). The read path was audited before packing:
  `roadmap_acceptance_event_history_load` -> `fabric_state_storage_binding` -> `roadmap_operator_acceptance_history_load`;
  the branch a bare srv1 checkout binds is the one that would read the live partition, so the row is gated, not seeded:
  the packet refuses with **exit 2** unless its second argument names a cleared grant, and creates nothing on the way.
* v9: R6 carries the same per-leg claim binding, the same snapshot rule, the same hermetic fixture git and
  the same wall arithmetic including the self-run and the kill graces (4440 s inside the booked 4800 s,
  360 s of margin). Nothing about the hold changed: it refuses with **exit 2**, naming keen-otter
  msg_3407a188, and creates nothing on the way. It stays HELD and is not proposed for execution.
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
wall_capped_sum_s=4440 scope_RuntimeMaxSec=4800 driver_RuntimeMaxSec=5100 launcher_cap=7200 legs=1
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
* exit contract (v7 wording): **1** if any claim is FALSIFIED (a bare FAIL inside an OBSERVED leg), else **2** if
  any claim is NO_OBSERVATION, else **0** when every claim in every leg held. `2` also covers every setup or gate
  refusal: bounds mismatch, wall over the launcher cap, scope mismatch, reused `OUT`, relative or mis-booked `OUT`,
  missing tool, malformed or absent completion record, missing/duplicate/qualified verdict evidence, a leg whose
  exit code disagrees with its own verdict lines, a dirty worktree or a changed runner after the legs, and R6's
  un-cleared hold. There is no exit 3 and no "couldn't run" exit 1: a packet that never ran reports
  NO_OBSERVATION, while a packet that ran and saw a falsification reports 1 even if another leg produced nothing.
* OOM: one snapshot per boundary, raw bytes published before parsing, each key required exactly once as `^key [0-9]+$`;
  any read or write failure is UNKNOWN and therefore NO_OBSERVATION; rise/fall compared per boundary and final vs
  last-leg. No `grep '^oom_kill [1-9]'`, no silent `cat`, and `memory.peak` is printed but never judged.

## Self-test result (recorded, not asserted vaguely)

* `bash tools/acceptance_r4_r5_r6_selfcheck_v9.sh <repo>` (run in this container against this session's worktree,
  which carries both pins) → **1002 passed, 0 failed, 0 skipped**.
* embedded packet fixtures: R4 **292 pass / 0 fail**, R5 **290 pass / 0 fail**, R6 **294 pass / 0 fail**.
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
| mutation 3 applied to the R4 job | FIXTURE FAIL self-structure: the booked bounds are gated before any work expected 1 got 0 / FIXTURES pass=290 fail=2 (rc=1) |
| mutation 4 applied to the R4 job | FIXTURE FAIL snap-parsed expected 0/0 got UNKNOWN/UNKNOWN / FIXTURES pass=283 fail=9 (rc=1) |
| mutation 5 applied to the R4 job | FIXTURE FAIL self-structure: no fourth exit code exists anywhere in the script expected 0 got 1 / FIXTURES pass=291 fail=1 (rc=1) |
| mutation 6 applied to the R4 job | FIXTURE FAIL self-structure: each leg boundary pair is compared, not merely recorded expected 1 got 0 / FIXTURES pass=291 fail=1 (rc=1) |
| mutation 8 applied to the R4 job | FIXTURE FAIL self-structure: the completion gate reads the leg stderr capture, where the record is emitted expected 1 got 0 / FIXTURES pass=287 fail=5… |
| mutation 9 applied to the R4 job | FIXTURE FAIL verdict-bare-pass-plus-qualified-fail expected NO_OBSERVATION got HELD / FIXTURES pass=291 fail=1 (rc=1) |
| mutation 10 applied to the R4 job | FIXTURE FAIL bare-pass expected HELD got NO_OBSERVATION / FIXTURES pass=282 fail=10 (rc=1) |
| mutation 11 applied to the R4 job | FIXTURE FAIL completion-duplicate expected completion records: 2 matched the prefix, exactly 1 is required got OK [process-partition] total_ms=340188 … |
| mutation 12 applied to the R4 job | FIXTURE FAIL self-structure: the completion gate reads the leg stderr capture, where the record is emitted expected 1 got 0 / FIXTURES pass=281 fail=1… |
| mutation 13 applied to the R4 job | FIXTURE FAIL self-structure: the pin proves the pinned producer emits the record to stderr expected 1 got 0 / FIXTURES pass=291 fail=1 (rc=1) |
| mutation 16 applied to the R4 job | FIXTURE FAIL rc1-pass-plus-fail expected OK got MISMATCH rc=1 but no claim printed a bare FAIL, so 0 was required / FIXTURES pass=282 fail=10 (rc=1) |
| mutation 17 applied to the R4 job | FIXTURE FAIL wall-gate-refuses-over-cap-nonzero expected 1 got 0 / FIXTURES pass=289 fail=3 (rc=1) |
| mutation 18 applied to the R4 job | FIXTURE FAIL producer-gate-stdout-render-refuses-nonzero expected 1 got 0 / FIXTURES pass=289 fail=3 (rc=1) |
| mutation 19 applied to the R4 job | FIXTURE FAIL post-leg-dirty-forces-every-claim-nood expected [a=NO_OBSERVATION b=NO_OBSERVATION] inside [claims classified: a=HELD b=FAILED post_leg_c… |
| mutation 20 applied to the R4 job | FIXTURE FAIL contract-all-pass-at-rc1 expected NO_OBSERVATION NO_OBSERVATION NOOBS 2 got HELD HELD AS_EXPECTED 0 / FIXTURES pass=290 fail=2 (rc=1) |
| mutation 21 applied to the R4 job | FIXTURE FAIL cs-complete-set-with-a-fail-is-still-1 expected 1 got 2 / FIXTURES pass=281 fail=11 (rc=1) |
| mutation 22 applied to the R4 job | FIXTURE FAIL out_gate-refusals-exit-nonzero expected 1 got 0 / FIXTURES pass=291 fail=1 (rc=1) |
| mutation 23 applied to the R4 job | FIXTURE FAIL post-leg-dirty-worktree-refuses expected [the worktree] inside [ exit=1] / FIXTURES pass=289 fail=3 (rc=1) |
| mutation 24 applied to the R4 job | FIXTURE FAIL post-leg-runner-changed-refuses expected [the runner changed] inside [ exit=1] / FIXTURES pass=290 fail=2 (rc=1) |
| mutation 3 applied to the R5 job | FIXTURE FAIL self-structure: the booked bounds are gated before any work expected 1 got 0 / FIXTURES pass=288 fail=2 (rc=1) |
| mutation 4 applied to the R5 job | FIXTURE FAIL snap-parsed expected 0/0 got UNKNOWN/UNKNOWN / FIXTURES pass=281 fail=9 (rc=1) |
| mutation 5 applied to the R5 job | FIXTURE FAIL self-structure: no fourth exit code exists anywhere in the script expected 0 got 1 / FIXTURES pass=289 fail=1 (rc=1) |
| mutation 6 applied to the R5 job | FIXTURE FAIL self-structure: each leg boundary pair is compared, not merely recorded expected 1 got 0 / FIXTURES pass=289 fail=1 (rc=1) |
| mutation 8 applied to the R5 job | FIXTURE FAIL self-structure: the completion gate reads the leg stderr capture, where the record is emitted expected 1 got 0 / FIXTURES pass=285 fail=5… |
| mutation 9 applied to the R5 job | FIXTURE FAIL verdict-bare-pass-plus-qualified-fail expected NO_OBSERVATION got HELD / FIXTURES pass=289 fail=1 (rc=1) |
| mutation 10 applied to the R5 job | FIXTURE FAIL bare-pass expected HELD got NO_OBSERVATION / FIXTURES pass=280 fail=10 (rc=1) |
| mutation 11 applied to the R5 job | FIXTURE FAIL completion-duplicate expected completion records: 2 matched the prefix, exactly 1 is required got OK [process-partition] total_ms=340188 … |
| mutation 12 applied to the R5 job | FIXTURE FAIL self-structure: the completion gate reads the leg stderr capture, where the record is emitted expected 1 got 0 / FIXTURES pass=279 fail=1… |
| mutation 13 applied to the R5 job | FIXTURE FAIL self-structure: the pin proves the pinned producer emits the record to stderr expected 1 got 0 / FIXTURES pass=289 fail=1 (rc=1) |
| mutation 16 applied to the R5 job | FIXTURE FAIL rc1-pass-plus-fail expected OK got MISMATCH rc=1 but no claim printed a bare FAIL, so 0 was required / FIXTURES pass=280 fail=10 (rc=1) |
| mutation 17 applied to the R5 job | FIXTURE FAIL wall-gate-refuses-over-cap-nonzero expected 1 got 0 / FIXTURES pass=287 fail=3 (rc=1) |
| mutation 18 applied to the R5 job | FIXTURE FAIL producer-gate-stdout-render-refuses-nonzero expected 1 got 0 / FIXTURES pass=287 fail=3 (rc=1) |
| mutation 19 applied to the R5 job | FIXTURE FAIL post-leg-dirty-forces-every-claim-nood expected [a=NO_OBSERVATION b=NO_OBSERVATION] inside [claims classified: a=HELD b=FAILED post_leg_c… |
| mutation 20 applied to the R5 job | FIXTURE FAIL contract-all-pass-at-rc1 expected NO_OBSERVATION NO_OBSERVATION NOOBS 2 got HELD HELD AS_EXPECTED 0 / FIXTURES pass=288 fail=2 (rc=1) |
| mutation 21 applied to the R5 job | FIXTURE FAIL cs-complete-set-with-a-fail-is-still-1 expected 1 got 2 / FIXTURES pass=279 fail=11 (rc=1) |
| mutation 22 applied to the R5 job | FIXTURE FAIL out_gate-refusals-exit-nonzero expected 1 got 0 / FIXTURES pass=289 fail=1 (rc=1) |
| mutation 23 applied to the R5 job | FIXTURE FAIL post-leg-dirty-worktree-refuses expected [the worktree] inside [ exit=1] / FIXTURES pass=287 fail=3 (rc=1) |
| mutation 24 applied to the R5 job | FIXTURE FAIL post-leg-runner-changed-refuses expected [the runner changed] inside [ exit=1] / FIXTURES pass=288 fail=2 (rc=1) |
| mutation 3 applied to the R6 job | FIXTURE FAIL self-structure: the booked bounds are gated before any work expected 1 got 0 / FIXTURES pass=292 fail=2 (rc=1) |
| mutation 4 applied to the R6 job | FIXTURE FAIL snap-parsed expected 0/0 got UNKNOWN/UNKNOWN / FIXTURES pass=285 fail=9 (rc=1) |
| mutation 5 applied to the R6 job | FIXTURE FAIL self-structure: no fourth exit code exists anywhere in the script expected 0 got 1 / FIXTURES pass=293 fail=1 (rc=1) |
| mutation 6 applied to the R6 job | FIXTURE FAIL self-structure: each leg boundary pair is compared, not merely recorded expected 1 got 0 / FIXTURES pass=293 fail=1 (rc=1) |
| mutation 8 applied to the R6 job | FIXTURE FAIL self-structure: the completion gate reads the leg stderr capture, where the record is emitted expected 1 got 0 / FIXTURES pass=290 fail=4… |
| mutation 9 applied to the R6 job | FIXTURE FAIL verdict-bare-pass-plus-qualified-fail expected NO_OBSERVATION got HELD / FIXTURES pass=293 fail=1 (rc=1) |
| mutation 10 applied to the R6 job | FIXTURE FAIL bare-pass expected HELD got NO_OBSERVATION / FIXTURES pass=284 fail=10 (rc=1) |
| mutation 11 applied to the R6 job | FIXTURE FAIL completion-duplicate expected completion records: 2 matched the prefix, exactly 1 is required got OK [process-partition] total_ms=340188 … |
| mutation 12 applied to the R6 job | FIXTURE FAIL self-structure: the completion gate reads the leg stderr capture, where the record is emitted expected 1 got 0 / FIXTURES pass=284 fail=1… |
| mutation 13 applied to the R6 job | FIXTURE FAIL self-structure: the pin proves the pinned producer emits the record to stderr expected 1 got 0 / FIXTURES pass=293 fail=1 (rc=1) |
| mutation 16 applied to the R6 job | FIXTURE FAIL rc1-pass-plus-fail expected OK got MISMATCH rc=1 but no claim printed a bare FAIL, so 0 was required / FIXTURES pass=284 fail=10 (rc=1) |
| mutation 17 applied to the R6 job | FIXTURE FAIL wall-gate-refuses-over-cap-nonzero expected 1 got 0 / FIXTURES pass=291 fail=3 (rc=1) |
| mutation 18 applied to the R6 job | FIXTURE FAIL producer-gate-stdout-render-refuses-nonzero expected 1 got 0 / FIXTURES pass=291 fail=3 (rc=1) |
| mutation 19 applied to the R6 job | FIXTURE FAIL post-leg-dirty-forces-every-claim-nood expected [a=NO_OBSERVATION b=NO_OBSERVATION] inside [claims classified: a=HELD b=FAILED post_leg_c… |
| mutation 20 applied to the R6 job | FIXTURE FAIL contract-all-pass-at-rc1 expected NO_OBSERVATION NO_OBSERVATION NOOBS 2 got HELD HELD AS_EXPECTED 0 / FIXTURES pass=292 fail=2 (rc=1) |
| mutation 21 applied to the R6 job | FIXTURE FAIL cs-complete-set-with-a-fail-is-still-1 expected 1 got 2 / FIXTURES pass=283 fail=11 (rc=1) |
| mutation 22 applied to the R6 job | FIXTURE FAIL out_gate-refusals-exit-nonzero expected 1 got 0 / FIXTURES pass=293 fail=1 (rc=1) |
| mutation 23 applied to the R6 job | FIXTURE FAIL post-leg-dirty-worktree-refuses expected [the worktree] inside [ exit=1] / FIXTURES pass=291 fail=3 (rc=1) |
| mutation 24 applied to the R6 job | FIXTURE FAIL post-leg-runner-changed-refuses expected [the runner changed] inside [ exit=1] / FIXTURES pass=292 fail=2 (rc=1) |
| defect (1) reintroduced: a literal claim_batch token after the binary | FIXTURE FAIL argv-R4a-sound expected  got R4a: argv carries a literal 'claim_batch' token after the binary / FIXTURES pass=290 fail=2 (rc=1) |
| defect (2) reintroduced: a space-separated name in the CSV | FIXTURE FAIL csv-r4b expected 1 got 3 / FIXTURES pass=289 fail=3 (rc=1) |
| defect (1) in the R6 packet as well | FIXTURE FAIL argv-R6-sound expected  got R6: argv carries a literal 'claim_batch' token after the binary / FIXTURES pass=292 fail=2 (rc=1) |
| the end-to-end leg control deleted from the packet fixtures | FIXTURE FAIL self-structure: the fixtures drive a real leg and require the stderr record to be an observation expected 1 got 0 / /tmp/r4r5r6-selfcheck… |
| the end-to-end leg control deleted from the R5 job too | FIXTURE FAIL self-structure: the fixtures drive a real leg and require the stderr record to be an observation expected 1 got 0 / /tmp/r4r5r6-selfcheck… |
| RUNTIME_S booked at 9900, above the installed launcher cap (v6) | FIXTURE FAIL wall-reserve-is-stated expected 40 got 4540 / FIXTURES pass=282 fail=10 (rc=1) |
| …the same over-cap booking in the R5 job | FIXTURE FAIL wall-reserve-is-stated expected 60 got 5460 / FIXTURES pass=280 fail=10 (rc=1) |
| …and in the R6 job | FIXTURE FAIL wall-reserve-is-stated expected 360 got 5460 / FIXTURES pass=284 fail=10 (rc=1) |
| a shipped gate neutered (mutation 25 applied to the R4 packet) | FIXTURE FAIL cs-shipped-set-registers-and-is-countable expected 0 got 2 / FIXTURES pass=260 fail=32 (rc=1) |
| a shipped gate neutered (mutation 25 applied to the R5 packet) | FIXTURE FAIL cs-shipped-set-registers-and-is-countable expected 0 got 2 / FIXTURES pass=258 fail=32 (rc=1) |
| a shipped gate neutered (mutation 25 applied to the R6 packet) | FIXTURE FAIL cs-shipped-set-registers-and-is-countable expected 0 got 2 / FIXTURES pass=262 fail=32 (rc=1) |
| a shipped gate neutered (mutation 26 applied to the R4 packet) | FIXTURE FAIL cs-shipped-set-registers-and-is-countable expected 0 got 2 / FIXTURES pass=274 fail=18 (rc=1) |
| a shipped gate neutered (mutation 26 applied to the R5 packet) | FIXTURE FAIL cs-shipped-set-registers-and-is-countable expected 0 got 2 / FIXTURES pass=272 fail=18 (rc=1) |
| a shipped gate neutered (mutation 26 applied to the R6 packet) | FIXTURE FAIL cs-shipped-set-registers-and-is-countable expected 0 got 2 / FIXTURES pass=276 fail=18 (rc=1) |
| a shipped gate neutered (mutation 27 applied to the R4 packet) | FIXTURE FAIL cs-writer-produces-the-booked-registration expected 0 got 1 / FIXTURES pass=263 fail=29 (rc=1) |
| a shipped gate neutered (mutation 27 applied to the R5 packet) | FIXTURE FAIL cs-writer-produces-the-booked-registration expected 0 got 1 / FIXTURES pass=261 fail=29 (rc=1) |
| a shipped gate neutered (mutation 27 applied to the R6 packet) | FIXTURE FAIL cs-writer-produces-the-booked-registration expected 0 got 1 / FIXTURES pass=265 fail=29 (rc=1) |
| a shipped gate neutered (mutation 28 applied to the R4 packet) | FIXTURE FAIL cs-crossleg-names-the-claim-cause expected [claim-wrong-leg:LEGY:b] inside [NO_OBSERVATION:claim-set claim-count:LEGY claim-duplicated-ac… |
| a shipped gate neutered (mutation 28 applied to the R5 packet) | FIXTURE FAIL cs-crossleg-names-the-claim-cause expected [claim-wrong-leg:LEGY:b] inside [NO_OBSERVATION:claim-set claim-count:LEGY claim-duplicated-ac… |
| a shipped gate neutered (mutation 28 applied to the R6 packet) | FIXTURE FAIL cs-crossleg-names-the-claim-cause expected [claim-wrong-leg:LEGY:b] inside [NO_OBSERVATION:claim-set claim-count:LEGY claim-duplicated-ac… |
| a shipped gate neutered (mutation 29 applied to the R4 packet) | FIXTURE FAIL cs-crossleg-names-the-cross-file-cause expected [claim-duplicated-across-files] inside [NO_OBSERVATION:claim-set claim-count:LEGY claim-w… |
| a shipped gate neutered (mutation 29 applied to the R5 packet) | FIXTURE FAIL cs-crossleg-names-the-cross-file-cause expected [claim-duplicated-across-files] inside [NO_OBSERVATION:claim-set claim-count:LEGY claim-w… |
| a shipped gate neutered (mutation 29 applied to the R6 packet) | FIXTURE FAIL cs-crossleg-names-the-cross-file-cause expected [claim-duplicated-across-files] inside [NO_OBSERVATION:claim-set claim-count:LEGY claim-w… |
| a shipped gate neutered (mutation 30 applied to the R4 packet) | FIXTURE FAIL cs-legset-names-the-leg-cause expected [registration-leg-count] inside [NO_OBSERVATION:claim-set claim-count:LEGY claim-wrong-leg:LEGY:c … |
| a shipped gate neutered (mutation 30 applied to the R5 packet) | FIXTURE FAIL cs-legset-names-the-leg-cause expected [registration-leg-count] inside [NO_OBSERVATION:claim-set claim-count:LEGY claim-wrong-leg:LEGY:c … |
| a shipped gate neutered (mutation 30 applied to the R6 packet) | FIXTURE FAIL cs-legset-names-the-leg-cause expected [registration-leg-count] inside [NO_OBSERVATION:claim-set claim-count:LEGY claim-wrong-leg:LEGY:c … |
| a shipped gate neutered (mutation 31 applied to the R4 packet) | FIXTURE FAIL cs-shipped-set-registers-and-is-countable expected 0 got 2 / FIXTURES pass=275 fail=17 (rc=1) |
| a shipped gate neutered (mutation 31 applied to the R5 packet) | FIXTURE FAIL cs-shipped-set-registers-and-is-countable expected 0 got 2 / FIXTURES pass=273 fail=17 (rc=1) |
| a shipped gate neutered (mutation 31 applied to the R6 packet) | FIXTURE FAIL cs-shipped-set-registers-and-is-countable expected 0 got 2 / FIXTURES pass=277 fail=17 (rc=1) |
| a shipped gate neutered (mutation 32 applied to the R4 packet) | FIXTURE FAIL wall-sum-arithmetic expected 5360 got 4920 / FIXTURES pass=287 fail=5 (rc=1) |
| a shipped gate neutered (mutation 32 applied to the R5 packet) | FIXTURE FAIL wall-sum-arithmetic expected 4440 got 4020 / FIXTURES pass=285 fail=5 (rc=1) |
| a shipped gate neutered (mutation 32 applied to the R6 packet) | FIXTURE FAIL wall-sum-arithmetic expected 4440 got 4020 / FIXTURES pass=289 fail=5 (rc=1) |
| a shipped gate neutered (mutation 33 applied to the R4 packet) | FIXTURE FAIL no-run-timeout-lacks-a-kill-grace expected 1 got 0 / FIXTURES pass=288 fail=4 (rc=1) |
| a shipped gate neutered (mutation 33 applied to the R5 packet) | FIXTURE FAIL no-run-timeout-lacks-a-kill-grace expected 1 got 0 / FIXTURES pass=286 fail=4 (rc=1) |
| a shipped gate neutered (mutation 33 applied to the R6 packet) | FIXTURE FAIL no-run-timeout-lacks-a-kill-grace expected 1 got 0 / FIXTURES pass=290 fail=4 (rc=1) |
| the v8 git shape (identity on init only) inverts under env -i | FIXTURE FAIL post-leg-clean-no-cause expected [exit=1] inside [the worktree /tmp/tmp.eJ5TIj exit=2] (rc=1) |

(Every control above bit. mut1/mut2 (the v1 argv defects) are applied to the R4 packet and mut7 to the R6
packet, because those command strings no longer exist in the other jobs; mut14 to R4 and R5; mut15 (the
over-cap 9900 booking), the v5 stream/producer controls, the verdict-rule controls, and every v7 control
(mut16-24) are applied to all three jobs, and so are the three v8 controls (mut25-27) and the six v9 controls
(mut28-33). v8’s manifest listed only mut16-24 here - the three v8 controls were measured and reported in
the message but never made it into that manifest; they are listed below with their v9 successors.

| v7 control | the defect it reintroduces | what must fail |
|---|---|---|
| mut16 | `leg_rc_expect` stops counting bare FAILs, so any exit code looks consistent | the contract fixtures and the rc controls |
| mut17 | `wall_gate` prints its refusal and returns 0 (the exact v6 defect) | `wall-gate-refuses-*` exit-status fixtures |
| mut18 | `producer_gate` prints its refusal and returns 0 | `producer-gate-*-refuses-nonzero` |
| mut19 | `final_verdict` ignores the post-leg cause | `post-leg-dirty-forces-every-claim-nood`, `fv_*` |
| mut20 | `leg()` stops asking whether the exit code agrees with the leg’s verdicts | `contract-all-pass-at-rc1`, `contract-pass-and-fail-at-rc0` |
| mut21 | the exit precedence inverted: an unobserved leg would bury a real falsification | `falsified-outranks-no-observation` |
| mut22 | the argument gate stops refusing a relative OUT | the `out_gate-*` fixtures and controls |
| mut23 | the post-leg worktree check disabled | `post-leg-dirty-worktree-refuses` |
| mut24 | the post-leg runner-identity check disabled | `post-leg-runner-changed-refuses` |
| mut25 | `cs_check` neutered: the classes files are never anchored to a booked set | the `cs-*` fixtures and every contract fixture |
| mut26 | the booked-leg count removed from `cs_check` | `cs-shipped-set-registers-and-is-countable` |
| mut27 | `register_claims` never writes the registration, so the decision is anchored to nothing | the `cs-writer-*` fixtures and every contract fixture that registers |
| mut28 | `cs_check` stops binding a claim to the leg it is registered under (a booked name anywhere passes) | `cs-crossleg-names-the-claim-cause` |
| mut29 | the cross-file duplicate scan is removed | `cs-crossleg-names-the-cross-file-cause` |
| mut30 | the registration may name legs that were never booked (a count instead of the exact set) | `cs-legset-names-the-leg-cause` |
| mut31 | `cs_check` validates but records no snapshot, so the verdict counts bytes it never saw | `cs-shipped-set-registers-and-is-countable` |
| mut32 | the self-run and the kill graces drop out of `WALL_SUM` | `wall-sum-arithmetic` |
| mut33 | every `-k` grace is stripped from the run-path timeouts | `no-run-timeout-lacks-a-kill-grace` |

Each control mutates a COPY; the shipped packets are never modified.)

## What was NOT done

* nothing was executed: no claim run, no ssh, no srv1 contact, no partition or operator write;
* the packets were not staged, the units were not created, and no `systemd-run` or launcher call was made —
  the launcher line is the contract, and it is executed by keen-otter/sleek-owl when a grant exists;
* no execution grant is requested here; grants come only from keen-otter via sleek-owl-397;
* R6 is not seedable and stays HELD until keen-otter clears msg_3407a188;
* the draft PR stays a draft (the packets are evidence, not a code change to merge);
* no mutation leg is proposed for any row, so R4/R5 presence observations cannot discriminate a regression.

