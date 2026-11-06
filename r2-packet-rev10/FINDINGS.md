# r2 packet findings ledger (through rev10)

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
| J6 | Join law not yet checked against REAL retained claim_batch logs | silent-bee | OPEN: calm-crab holds the logs | none locally; asked of calm-crab |

## Containment

| # | Finding | Found by | Fix (rev) | Fixture |
|---|---|---|---|---|
| C1 | Harness used global pgrep/pkill as ownership | keen-otter | own-recorded pids + starttime (harness); jobs use their own cgroup | contain |
| C2 | Unreadable sub-cgroup read as "no survivors" | review thread | `_walk_cg` refuses an unreadable/unsearchable dir (rev10) | subtree-unreadable |
| C3 | Survivor check only proven on a fake file, never a real cgroup | sleek-owl | `contain_selftest` at the start of EVERY job, inside its real launcher scope: one child must be listed exactly, then gone (rev10). Not runnable here: this container's /proc/self/cgroup is `0::/`, /sys/fs/cgroup is not writable, no systemd-run | cst-exact, cst-stray, cst-blind (emulated listing); real-cgroup run = first job start |
| C4 | Self-test's own waits could be listed as survivors | self | builtin-only re-reads between waits (rev10) | cst-exact |

## Chain, markers, receipts

| # | Finding | Found by | Fix (rev) | Fixture |
|---|---|---|---|---|
| M1 | Tampered, unreviewed, ignored-file and printf-forged prior markers | review thread | REVIEWED_SCRIPTS map, chain walk to J1, ignored-set snapshot | chain-genuine, chain-old-tamper, chain-unreviewed, chain-ignored, chain-printf-forge |
| M2 | Marker OUT could be a foreign directory; receipts matched by basename | sleek-owl | OUT must equal `$RUNS_ROOT/<job>/out`; receipts at exact W:/O: paths; unknown marker lines refuse (rev10) | anchor-foreign-out, anchor-moved-receipt |
| M3 | Marker write failure was not checked | review thread | `mark` checks every write, `mv -f -T` | mark-write-fail |
| M4 | ign_snapshot failed every mark under pipefail when nothing matched | self | `{ grep ... \|\| true; }` | every chain case |
| M5 | Launcher OUT = `$RUNS_ROOT/<job>/out` is an ASSUMPTION | self | OPEN: needs calm-crab attestation | none |

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

## J1 / J1B1

| # | Finding | Found by | Fix (rev) | Fixture |
|---|---|---|---|---|
| B1 | Readable fabric writer key would send up to 4 signed POSTs without an explicit grant | review thread | arg 2 `FABRIC_POSTS_GRANT=msg_…` required when the key is readable (rev10) | fabric-gate |
| B2 | Untimed git / --version calls in J1 | review thread | every call under `timeout` (rev10) | none (timing) |
| B3 | Production probes (ssh git rev-parse, curl healthz) were not named in the grant | review thread | GRANT LINE REQUIRED header in J1R and J2 (rev10) | none (grant text) |

## Process findings (no fixture; recorded so they are not lost)

- P1 r1 was self-launched outside the keeper launcher: a breach, reported; settled by calm-crab as accounting only.
- P2 rev8 was re-frozen before sleek-owl's reviews landed.
- P3 The #15822 receipt for the earlier apply runs is still owed.

## Still unverified (need one keeper read or attestation)

InvocationID of an active .path unit; the real `sudo -ll` text; non-interactive `ssh srv1`; fabric key readability;
disabled vs static after reload; real claim_batch output lines (J6); launcher OUT path (M5).

## Inversion proof (J5)

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
