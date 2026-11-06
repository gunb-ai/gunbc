# r2 packet findings ledger (through rev12)

One row per finding. "Fixture" is the case in `fixtures/run-all.sh` that goes red if the defect returns; "none" means no
local fixture can express it, and the row says what covers it instead. Finder is given where the thread records it;
"self" = found by loyal-deer-449 while building or testing. Attribution not recorded here is in the review thread on #15987.

## Outcome join (run_entry)

| # | Finding | Found by | Fix (rev) | Fixture |
|---|---|---|---|---|
| J1 | Foreign, duplicate, missing, reasoned and rc-mismatched outcome lines must not join | review thread | exact-roster join, rc == own verdicts (rev8) | join_fixture: foreign-pass, foreign-fail, foreign-skipped, duplicate-pass, reasoned-fail, unobserved, missing-name, mixed-rc-* |
| J2 | `fail a`, ` FAIL a`, `FAIL: a` beside a valid log were accepted (scan was `^[A-Z]+ ` only) | keen-otter (msg_b5063276), via silent-bee | case-insensitive keyword scan with leading space / colon; non-canonical outcome-like line refuses; CR refuses (rev10) | lower-fail, space-fail, colon-fail, timedout, cr-byte |
| J3 | Normal diagnostics must still pass the tightened scan | silent-bee (msg_b43e5154) | keyword scan only is -i; ALLCAPS scan stays case-sensitive (rev10) | noise-pass |
| J4 | `TIMEOUT: a`, `ABORTED`, `IGNORED: b` accepted (ALLCAPS test needed space after token) | sleek-owl, via silent-bee (msg_feb3e7d3) | `^[A-Z][A-Z0-9_]*([ \t:]|$)`; keyword list widened to the 17-word minimum (rev10) | timeout-colon, aborted-bare, ignored-colon, unknown-caps-eol |
| J5 | A self-test must be shown able to fail | keen-otter + tidy-ant (msg_feb3e7d3) | proof run on a scratch copy, NOT shipped (rev10) | see "Inversion proof" below |
| J6 | Join law not yet checked against REAL retained claim_batch logs | silent-bee | CLOSED for rev11's patterns too (calm-crab msg_a019e137: rev11 post-run body verbatim over 6 retained logs, incl. a mixed PASS/FAIL rc-1 all-bare log ACCEPTED with 0 deviations; panic, reasoned, closure-refused logs and a forced-rc control REFUSED; 0 TAB bytes, no "Started"-type lines, no real name ending in t, so J7/J9's TAB and t cases rest on synthetic fixtures only). Earlier: CLOSED for rev10's patterns (calm-crab msg_6627419e: rev10 decision body verbatim over 5 retained logs; all-PASS accepted, 4 bad logs + forced-rc control refused). rev11 changed the boundaries (J7, J8): re-check of rev11's exact patterns against the same 5 logs requested | none locally |
| J7 | rev10's ALLCAPS scan `[ \t:]` is space/backslash/'t'/colon under GNU grep: `Started ...` counted, `WEDGED<TAB>x` missed. My spot check ran under ugrep (my tool shell's `grep`), not GNU grep | self (from silent-bee's rev10 pass) | non-word boundary (rev11) | unknown-caps-tab, noise-capital-t (both red with the rev10 pattern restored) |
| J9 | The same `\t`-in-brackets trap in bash patterns / parameter expansions strips a final `t` from real names (false refusal); `IGNORED<TAB>x` slipped the scan | silent-bear-373 via silent-bee (msg_440d1499), sleek-owl (msg_ca59a79d) | every `\t` in all 8 rev11 scripts checked by context: the only remaining ones are a comment and a fixture line that `printf` turns into a real TAB; no bash pattern or expansion carries `\t`; `classify` uses `[[:space:]]`, `verdict` requires a literal space | ignored-tab, pass-tab (refuse); long-t: an all-PASS roster of two long real-style names ending in `t` (incl. `witness_teardown_dependency_arm_refuses_golden_independent`) must be ACCEPTED. Real long-name log control DONE (fierce-bee-693 msg_a18ca48c): rev11 decision body verbatim over its retained 56-PASS emit-leg log (sha 26a91b18...): ACCEPTED, exit 0, 0 deviations; CR 0, outcome-like 56, canonical 56, ALLCAPS 56, malformed residue 0. rev12 changes only how the malformed check is computed (a count instead of `\| grep -q`, same patterns). Whether any of those 56 names ends in `t` was not reported, so the t case still rests on the synthetic long-t fixture |
| J8 | Keyword/ALLCAPS boundary was whitespace/colon/EOL only: `TIMED-OUT a`, `FAIL(a)`, `FAIL-a`, `FAILED.` passed | sleek-owl (rev10 review) | boundary `[^A-Za-z0-9_]` or EOL for both scans (rev11). Not refused by design: a mixed-case unknown token (`Wedged a`), which no outcome form uses; `error: x` / `INFO: x` refuse, and so does a prose line starting with a capital letter and a space (`A thing`), since `A` is an ALLCAPS token (fail-closed; calm-crab's 6 real logs contain none) | timed-hyphen, fail-paren, fail-hyphen, failed-dot |

## Containment

| # | Finding | Found by | Fix (rev) | Fixture |
|---|---|---|---|---|
| C1 | Harness used global pgrep/pkill as ownership | keen-otter | own-recorded pids + starttime (harness); jobs use their own cgroup | contain |
| C2 | Unreadable sub-cgroup read as "no survivors" | review thread | `_walk_cg` refuses an unreadable/unsearchable dir (rev10) | subtree-unreadable |
| C3 | Survivor check only proven on a fake file, never a real cgroup | sleek-owl | `contain_selftest` at the start of EVERY job, inside its real launcher scope: one child must be listed exactly, then gone (rev10). Not runnable here: this container's /proc/self/cgroup is `0::/`, /sys/fs/cgroup is not writable, no systemd-run | cst-exact, cst-stray, cst-blind (emulated listing); real-cgroup run = first job start |
| C4 | Self-test's own waits could be listed as survivors | self | builtin-only re-reads between waits (rev10) | cst-exact |
| C5 | `_walk_cg` skipped dot-named sub-cgroups (`*/` without dotglob) | sleek-owl (rev10 review) | explicit `.[!.]*/` and `..?*/` patterns inside `_walk_cg`; no global dotglob (rev11) | subtree-dotdir (nested `sub/.deep` and top-level `.hide`) |
| C6 | J4 `cg_procs` gave each `find`/`cat` its own timeout, so N sub-cgroups multiplied it (budget 1 s returned success at 2.12 s) | keen-otter (msg_266a9a2f) | the whole traversal is ONE subprocess under ONE timeout (rev11). Reproduction, same stub (3 files x 0.7 s): rev10 budget 1 -> rc 0 at 2.13 s; rev11 budget 1 -> rc 1 at 1.00 s; rev11 budget 5 -> rc 0 at 2.12 s | j4-slowcg-listen, j4-slowcg-stop |
| C8 | `mark` built its body as `"$(printf ... "$(shas_id)" ...)"`, so a failing `shas_id`/`hash_t` was masked by printf's success: rc 0 and a marker with an EMPTY SHAS field (J4 could then report DONE) | keen-otter (rev11 review, executed the real mark) | every id is computed and shape-checked in the job shell before the body is built (rev12) | mark-shas-fail (refuses), mark-shas-nomarker (no marker written) |
| C9 | OPDL did not bound every post-listen operation: mark's mkdir, file write, readback and final cat, and ign_snapshot's sort and write, were untimed | keen-otter (rev11 review) | each is a timed external command under OPDL (`wfile` via timed tee; timed cat/mkdir/mv/sort) (rev12) | mark-deadline + mark-deadline-bound (readback hanging 20 s under an 8 s deadline: refused in < 15 s, no marker; rev11's untimed readback would take >= 20 s) |
| C10 | `... \| grep -q ... && die2` under `set -o pipefail`: an early-exiting grep -q SIGPIPEs its writer and fails the pipeline, so the refusal never runs on a large input. sleek-owl DEMONSTRATED it on rev11's closure gate (1000 dag + 1 src/v1 + 1500 src/v2 rows, 279 KB: PIPESTATUS 141 0, gate returned 0, 3/3); the 4-row closure-v1 fixture could not see it. Same shape in run_entry's malformed-line check and need's unknown-marker-line check (silent pass), and J4's listener membership (spurious refusal) | sleek-owl (rev11 review B1, m2); self | closure: ONE pass, no pipe: rows of the admitted shape must number exactly path_rows. The others: `grep -c` counts compared, never `\| grep -q` (rev12). Inspected: every `\| grep ...q` in body-common and the 7 bodies | closure-big-v1 (4001 rows, the src/v1 row followed by 3000 rows, ~150 KB: refuses), closure-big-ok (4000 rows: accepted) |
| C11 | `cg_procs` dropped non-numeric cgroup.procs lines instead of refusing | sleek-owl (rev11 review m3) | any non-numeric line makes the read fail (rev12) | none (unit-level) |
| C7 | The J4 post-listen phase was bounded only by the sum of per-command caps, and `mark`'s receipt hashing was untimed | keen-otter (msg_266a9a2f) | one operation deadline OPDL = JOB_END - 10 for every bookkeeping operation (`optc`); `hash_t` for mark/shas_id; failure stays an honest serve-UP UNRESOLVED (rev11) | j4-slowhash, opdl-phase |

## Chain, markers, receipts

| # | Finding | Found by | Fix (rev) | Fixture |
|---|---|---|---|---|
| M1 | Tampered, unreviewed, ignored-file and printf-forged prior markers | review thread | REVIEWED_SCRIPTS map, chain walk to J1, ignored-set snapshot | chain-genuine, chain-old-tamper, chain-unreviewed, chain-ignored, chain-printf-forge |
| M2 | Marker OUT could be a foreign directory; receipts matched by basename | sleek-owl | OUT must equal `$RUNS_ROOT/<job>/out`; receipts at exact W:/O: paths; unknown marker lines refuse (rev10) | anchor-foreign-out, anchor-moved-receipt |
| M3 | Marker write failure was not checked | review thread | `mark` checks every write, `mv -f -T` | mark-write-fail |
| M4 | ign_snapshot failed every mark under pipefail when nothing matched | self | `{ grep ... \|\| true; }` | every chain case |
| M5 | Launcher OUT = `$RUNS_ROOT/<job>/out` was an ASSUMPTION | self | CLOSED: calm-crab attestation (msg_6627419e) from retained launcher calmcrab-udrv v4 sha 855920ac...: OUT = $HOME/runs/<JOB>/out; the launcher does NOT create it, and the frame creates it exclusively (`mkdir`, no -p) | none |

## J2 (install) and J4 (start)

| # | Finding | Found by | Fix (rev) | Fixture |
|---|---|---|---|---|
| S1 | `sudo -n -l <argv>` exits 0 for wrong runas / PASSWD / prefix / other-args rules | review thread | parse `sudo -n -ll` per rule: root, !authenticate, exact argv / bare program / ALL (rev10) | sudo-wrong-runas, sudo-mixed-tags, sudo-prefix, sudo-other-args |
| S2 | ssh srv1 must resolve to loopback, as briansrls, batchmode | review thread | `ssh -G` precheck | j2-ssh |
| S3 | Unit enable / kill paths | review thread | inventory before/after, diff before daemon-reload | j2-happy, j2-enable, j2-kill |
| S4 | units-pre-reload.txt was not bound as a receipt; identity.txt write unchecked; PATH-resolution compare; RESERVE not fixed | sleek-owl (delayed rev7 review) | all four fixed (rev10) | j2-happy (receipt set) |
| S5 | `$(tc …)` inside timeout argv died in a subshell only | self | time budget computed in the main shell | j4-deadline |
| S6 | J4 loop variable clobbered LoadState | self | renamed `sa` (J2 and J4) | j4-happy |
| S7 | Late listener + slow bookkeeping must still finish before JOB_END | review thread | WORK_END = JOB_END - stop and post reserves | j4-late, j4-stall |
| S8 | Listener pid must be inside the unit's cgroup subtree; receipt must be the exact 3-line J3 form | review thread | live re-read of InvocationID + ActiveEnterTimestampMonotonic | j4-subtree, j4-nopid, j4-receipt |
| S9 | `sudo_rule_ok` accepted `ALL` with a negated `!/usr/bin/x` on the same entry | sleek-owl (rev10 review) | a rule carrying ANY negated command never counts (J2 and J4) (rev11) | sudo-negated |
| S10 | still accepted: RunAsUsers `ALL, !root`; a negated command in a DIFFERENT rule (sudoers is last-match-wins); Options `!authenticate, authenticate` | sleek-owl (rev11 review m1) | a negated command in ANY rule refuses the whole listing; a negated RunAsUsers item or a positive `authenticate` option makes the rule not count (rev12). Still a precheck only: `sudo -n` refuses at start regardless | sudo-runas-neg, sudo-other-rule-neg, sudo-auth-both |

## J1 / J1B1

| # | Finding | Found by | Fix (rev) | Fixture |
|---|---|---|---|---|
| B1 | Readable fabric writer key would send up to 4 signed POSTs without an explicit grant | review thread | arg 2 `FABRIC_POSTS_GRANT=msg_…` required when the key is readable (rev10) | fabric-gate |
| B2 | Untimed external calls. rev10's claim "every call timed" was FALSE (keen-otter msg_266a9a2f, sleek-owl): J1's red-worktree git, J2's three `git -C $LAB/source` reads, J1R's `sha256sum -c` were untimed | review thread; keen-otter; sleek-owl | rev11 times every git, sha256sum, cp, mv, rm, ssh, curl, systemctl, ss, find, sudo, gunbc and claim_batch call on a job's real path. Remaining untimed, by inspection (pattern search over the bodies, not a proof): the self-tests' own operations inside their private mktemp directory, and bash builtins/pipeline utilities on in-memory or small local data | none (timing) |
| B2b | rev11's B2 was ALSO false: 11 real-path hashes still used the frame's untimed `hash_of` (need x2, J2 x8, J4 x1), and J1 called the frame's untimed `tree_clean` | self (rev12, while answering keen-otter); sleek-owl (rev11 review M2, same sites) | every real-path hash uses `hash_t` (timed, under OPDL); J1 checks the checkout with a timed `git status` in its own body instead of `tree_clean`. The frame's `hash_of` and `tree_clean` (kept byte-for-byte) remain only in the frame's own self-fixtures and the start-time SELF_SHA; those, like any blocked syscall, are bounded only by the launcher's runtime cap (rev12). Inspected: every `hash_of` occurrence in body-common and all 7 bodies, by /usr/bin/grep | none (timing) |
| B3 | Production probes (ssh git rev-parse, curl healthz) were not named in the grant | review thread | GRANT LINE REQUIRED header in J1R and J2 (rev10) | none (grant text) |

## Source roots

| # | Finding | Found by | Fix (rev) | Fixture |
|---|---|---|---|---|
| R2 | `closure_gate` called `budget c ...`, and `budget`'s own `local c` shadowed it, so the closure query ran under an EMPTY timeout and failed on every leg (caught by the suite before publishing, every job's join self-test refused) | self | `budget`'s local renamed `_bc` at the root, so no caller's variable can be shadowed (rev11) | every join self-test case through the real run_entry |
| R3 | closure_gate added up to 930 s PER ENTRY before the run caps; rev11's budgets were rev10's totals | keen-otter (rev11 review) | ONE closure query per job and worktree over all its entries (CQCAP 600; observed 130-160 s per query, calm-crab msg_a019e137); run_entry refuses an entry the query did not admit; FULL-CAP-OR-REFUSE: a claim run or closure query starts only if its whole cap fits, otherwise it refuses BEFORE starting; J1B3 RT 6600 -> 7200 (launcher max). Budget table below (rev12) | join self-test (every case admits its entry first) |
| R1 | run_entry passed roots [dag, src/v1, src/v2]; `cli_run::resolve_virtual_source_with_imports` demands `witness_layer_roots` [dag, src/v2] on the shared slot claim_batch fills first, and a different key refuses (SharedIndexSecondResidentPool) -> a possible deterministic NO_OBSERVATION | calm-crab (msg_6627419e), keen-otter (msg_8cdaf17a) | rev11: claim runs use EXACTLY [dag, src/v2]. Before each claim run `closure_gate` (a) requires the pinned authority row `gunbc.ci.ci_layer_roots` `witness_layer_roots` to read exactly that list, and (b) runs `claim_batch --print-entry-closure` (non-evaluating) with those roots. rc 0 plus one complete summary for exactly the job's entries IS the evidence: a module findable only under src/v1 makes the loader fail with rc 1. NOT executed by the gate (sleek-owl rev11 M1): that no selected function reaches a SLOT-demanding builtin rests on the static trace only; what the gate executes is that every closure resolves with the authority roots, so the claim run's slot key equals witness_layer_roots and the mismatch cannot arise in it (calm-crab msg_a019e137 verified this CLI contract at 26fe4b8f). The dag/src/v2 path filter is defence in depth only: with only those roots every printed path is under them, so it cannot otherwise fail. rev12: one query per job and worktree, see R3. Static trace at 26fe4b8f (source only, nothing run): no selected roster function, J1 OFF/RED included, reaches a slot-demanding builtin; 0 src/v1 modules in any entry's import+qualified closure; J1R/J2 `gunbc run` already used [dag, src/v2] | closure-v1, closure-nosummary (join self-test) |

## Budget (rev12; keen-otter rev11 review)

W = RT - 60 (frame exit margin) - 300 (RESERVE): the window in which tcap lets work start. "Caps" are the per-step timeouts.
Every claim run and closure query is FULL-CAP-OR-REFUSE, so a step that does not fit refuses before starting (typed
NO_OBSERVATION) and nothing is cut off before its roster. Small checks (git reads, hashes, marks) are bounded by OPDL and not
itemised; the join self-test measured 3.2 s locally.

| Job | RT | W | closure query cap | claim-run caps | other large caps | worst-case sum | margin |
|---|---|---|---|---|---|---|---|
| J1 | 7200 | 6840 | 2 x 600 (WT, WR) | 1500 + 1500 | git 1560, build 1800, SHAS 120 | 7680 | **-840** |
| J1R | 4200 | 3840 | none | none | 3 renders x 900 + preflight 900 | 3600 | 240 |
| J1B1 | 6600 | 6240 | 600 | 1500 + 1500 + 1200 | - | 4800 | 1440 |
| J1B2 | 6600 | 6240 | 600 | 2000 + 900 + 1200 | - | 4700 | 1540 |
| J1B3 | 7200 (was 6600) | 6840 | 600 | 1500 + 1200 + 1200 + 1500 | - | 6000 | 840 |
| J1B4 | 6600 | 6240 | 600 | 600 + 600 + 1200 + 1500 | - | 4500 | 1740 |
| J2, J4 | 6000, 2400 | - | none (no claim runs) | - | unchanged from rev10 | - | - |

J1's git caps: clone 300, fetch 300, checkout 120, fetch-main 300, 2 merge-base 240, worktree-add 120, red checks 180.

**J1 cannot hold every cap at once**, and RT 7200 is the launcher maximum. The RED leg (R, second) starts only if 1500 s
remain, so it refuses before starting when git + build + SHAS + both closure queries + the G leg exceed 5340 s. With the
OBSERVED closure cost (130-160 s each, ~320 s for two), even every git step and the build at their caps fit: 3480 + 320 + 1500
= 5300 <= 5340 (40 s to spare). It does not fit only if the closure queries also run far past their observed cost. The build's
real duration on srv1 at this pin is NOT measured here. If J1 runs long, it reports the R leg as a typed pre-start refusal and
must be rerun; it never reports a partial roster.

## Section Y (APPLY-EFFECTS)

| # | Finding | Found by | Status | Fixture |
|---|---|---|---|---|
| Y1 | Section Y is NOT closed: placement tests #2-21 and #34-45 were classified by their constructors, not traced line by line, and the live-effect fields were checked by symbol search only | sleek-owl (rev10 review) | OPEN. The spot checks sleek-owl ran match the witness files at 26fe4b8f; the remaining tests need a line-by-line trace before Section Y can be called complete | none |

## Process findings (no fixture; recorded so they are not lost)

- P1 r1 was self-launched outside the keeper launcher: a breach, reported; settled by calm-crab as accounting only.
- P2 rev8 was re-frozen before sleek-owl's reviews landed.
- P3 The #15822 receipt for the earlier apply runs is still owed.
- P4 rev10's FINDINGS B2 stated "every call under timeout"; it was false. Claims of completeness in this ledger now state the population inspected.
- P5 My tool shell's `grep` is a ugrep wrapper; the job scripts and harness run GNU grep 3.11. Regex spot checks must use /usr/bin/grep, and helper debugging must run in a clean `env -i bash --norc` (a sourced helper in my shell still sees the wrapper; it misled one debugging step during rev11).

## Still unverified (need one keeper read or attestation)

InvocationID of an active .path unit; the real `sudo -ll` text; non-interactive `ssh srv1`; fabric key readability;
disabled vs static after reload; rev11's join patterns over the retained logs (J6 re-check); contain_selftest and the
dot-named walk on a REAL cgroup (incl. whether an unprivileged child can mkdir a dot-named sub-cgroup); the cost of one
`--print-entry-closure` query per leg (R1: it builds the shared index once more per entry; bounded by a 900 s budget, fails closed).

## Inversion proof (J5)

rev12 FINAL bytes (the published ones): same inversion in a scratch copy; J1, J1B1, J1B2, J1B3, J1B4 each `startup exit=2;
join fixture all-pass: exit 0, required 2`. Earlier runs below are kept for the record.

Repeated on rev11's FINAL bytes (the published ones): the same inversion (`all-pass` 0 -> 2) in a scratch copy, the five
run_entry jobs started under the harness: J1, J1B1, J1B2, J1B3, J1B4 each `startup exit=2; join fixture all-pass: exit 0,
required 2`. rev10's run is kept below for the record. Separately, on an EARLIER rev11 draft (before the
J8/J9/R1/R2 edits, not these final bytes): restoring rev10's ALLCAPS pattern `[ \t:]` in a scratch copy made all five exit 2
on `unknown-caps-tab`.

Scratch copy only, never shipped: `join_fixture`'s `all-pass` case expectation changed from 0 to 2, packet regenerated,
the five jobs that call run_entry started under the local harness. Each exits 2 at startup:

```
INVERTED ld449-r2-j1.sh:   startup exit=2; join fixture all-pass: exit 0, required 2
INVERTED ld449-r2-j1b1.sh: startup exit=2; join fixture all-pass: exit 0, required 2
INVERTED ld449-r2-j1b2.sh: startup exit=2; join fixture all-pass: exit 0, required 2
INVERTED ld449-r2-j1b3.sh: startup exit=2; join fixture all-pass: exit 0, required 2
INVERTED ld449-r2-j1b4.sh: startup exit=2; join fixture all-pass: exit 0, required 2
(each last line: NO OBSERVATION (exit 2): REFUSED_SETUP: join-law fixture failed (run_entry would mis-join outcome lines))
```

J1R, J2 and J4 never call run_entry (the one textual match in each is the call inside join_fixture's own body, which they
never invoke), so they carry no join self-test.
