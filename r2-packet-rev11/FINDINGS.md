# r2 packet findings ledger (through rev11)

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
| J6 | Join law not yet checked against REAL retained claim_batch logs | silent-bee | CLOSED for rev10's patterns (calm-crab msg_6627419e: rev10 decision body verbatim over 5 retained logs; all-PASS accepted, 4 bad logs + forced-rc control refused). rev11 changed the boundaries (J7, J8): re-check of rev11's exact patterns against the same 5 logs requested | none locally |
| J7 | rev10's ALLCAPS scan `[ \t:]` is space/backslash/'t'/colon under GNU grep: `Started ...` counted, `WEDGED<TAB>x` missed. My spot check ran under ugrep (my tool shell's `grep`), not GNU grep | self (from silent-bee's rev10 pass) | non-word boundary (rev11) | unknown-caps-tab, noise-capital-t (both red with the rev10 pattern restored) |
| J9 | The same `\t`-in-brackets trap in bash patterns / parameter expansions strips a final `t` from real names (false refusal); `IGNORED<TAB>x` slipped the scan | silent-bear-373 via silent-bee (msg_440d1499), sleek-owl (msg_ca59a79d) | every `\t` in all 8 rev11 scripts checked by context: the only remaining ones are a comment and a fixture line that `printf` turns into a real TAB; no bash pattern or expansion carries `\t`; `classify` uses `[[:space:]]`, `verdict` requires a literal space | ignored-tab, pass-tab (refuse); long-t: an all-PASS roster of two long real-style names ending in `t` (incl. `witness_teardown_dependency_arm_refuses_golden_independent`) must be ACCEPTED. Real long-name log control: requested from fierce-bee-693 (emit-leg log sha 26a91b18...) |
| J8 | Keyword/ALLCAPS boundary was whitespace/colon/EOL only: `TIMED-OUT a`, `FAIL(a)`, `FAIL-a`, `FAILED.` passed | sleek-owl (rev10 review) | boundary `[^A-Za-z0-9_]` or EOL for both scans (rev11). Not refused by design: a mixed-case unknown token (`Wedged a`), which no outcome form uses; `error: x` / `INFO: x` refuse (fail-closed) | timed-hyphen, fail-paren, fail-hyphen, failed-dot |

## Containment

| # | Finding | Found by | Fix (rev) | Fixture |
|---|---|---|---|---|
| C1 | Harness used global pgrep/pkill as ownership | keen-otter | own-recorded pids + starttime (harness); jobs use their own cgroup | contain |
| C2 | Unreadable sub-cgroup read as "no survivors" | review thread | `_walk_cg` refuses an unreadable/unsearchable dir (rev10) | subtree-unreadable |
| C3 | Survivor check only proven on a fake file, never a real cgroup | sleek-owl | `contain_selftest` at the start of EVERY job, inside its real launcher scope: one child must be listed exactly, then gone (rev10). Not runnable here: this container's /proc/self/cgroup is `0::/`, /sys/fs/cgroup is not writable, no systemd-run | cst-exact, cst-stray, cst-blind (emulated listing); real-cgroup run = first job start |
| C4 | Self-test's own waits could be listed as survivors | self | builtin-only re-reads between waits (rev10) | cst-exact |
| C5 | `_walk_cg` skipped dot-named sub-cgroups (`*/` without dotglob) | sleek-owl (rev10 review) | explicit `.[!.]*/` and `..?*/` patterns inside `_walk_cg`; no global dotglob (rev11) | subtree-dotdir (nested `sub/.deep` and top-level `.hide`) |
| C6 | J4 `cg_procs` gave each `find`/`cat` its own timeout, so N sub-cgroups multiplied it (budget 1 s returned success at 2.12 s) | keen-otter (msg_266a9a2f) | the whole traversal is ONE subprocess under ONE timeout (rev11). Reproduction, same stub (3 files x 0.7 s): rev10 budget 1 -> rc 0 at 2.13 s; rev11 budget 1 -> rc 1 at 1.00 s; rev11 budget 5 -> rc 0 at 2.12 s | j4-slowcg-listen, j4-slowcg-stop |
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

## J1 / J1B1

| # | Finding | Found by | Fix (rev) | Fixture |
|---|---|---|---|---|
| B1 | Readable fabric writer key would send up to 4 signed POSTs without an explicit grant | review thread | arg 2 `FABRIC_POSTS_GRANT=msg_…` required when the key is readable (rev10) | fabric-gate |
| B2 | Untimed external calls. rev10's claim "every call timed" was FALSE (keen-otter msg_266a9a2f, sleek-owl): J1's red-worktree git, J2's three `git -C $LAB/source` reads, J1R's `sha256sum -c` were untimed | review thread; keen-otter; sleek-owl | rev11 times every git, sha256sum, cp, mv, rm, ssh, curl, systemctl, ss, find, sudo, gunbc and claim_batch call on a job's real path. Remaining untimed, by inspection (pattern search over the bodies, not a proof): the self-tests' own operations inside their private mktemp directory, and bash builtins/pipeline utilities on in-memory or small local data | none (timing) |
| B3 | Production probes (ssh git rev-parse, curl healthz) were not named in the grant | review thread | GRANT LINE REQUIRED header in J1R and J2 (rev10) | none (grant text) |

## Source roots

| # | Finding | Found by | Fix (rev) | Fixture |
|---|---|---|---|---|
| R2 | `closure_gate` called `budget c ...`, and `budget`'s own `local c` shadowed it, so the closure query ran under an EMPTY timeout and failed on every leg (caught by the suite before publishing, every job's join self-test refused) | self | `budget`'s local renamed `_bc` at the root, so no caller's variable can be shadowed (rev11) | every join self-test case through the real run_entry |
| R1 | run_entry passed roots [dag, src/v1, src/v2]; `cli_run::resolve_virtual_source_with_imports` demands `witness_layer_roots` [dag, src/v2] on the shared slot claim_batch fills first, and a different key refuses (SharedIndexSecondResidentPool) -> a possible deterministic NO_OBSERVATION | calm-crab (msg_6627419e), keen-otter (msg_8cdaf17a) | rev11: claim runs use EXACTLY [dag, src/v2]. Before each claim run `closure_gate` (a) requires the pinned authority row `gunbc.ci.ci_layer_roots` `witness_layer_roots` to read exactly that list, and (b) runs `claim_batch --print-entry-closure` (non-evaluating) with those roots and requires one complete summary, matching row count, and every file under dag/ or src/v2/: omitted-root independence by EXECUTION per entry, at run time. Static trace at 26fe4b8f (source only, nothing run): no selected roster function, J1 OFF/RED included, reaches a slot-demanding builtin; 0 src/v1 modules in any entry's import+qualified closure; J1R/J2 `gunbc run` already used [dag, src/v2] | closure-v1, closure-nosummary (join self-test) |

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
