# r2 packet findings ledger (through rev16)

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
| J6 | Join law not yet checked against REAL retained claim_batch logs | silent-bee | CLOSED through rev12 (calm-crab msg_51f80679: rev12 post-run body verbatim over the same 6 logs, identical results). rev13 does not change run_entry's post-run body (its changes are mark, need, the closure query and J1's git reads), so this carries to rev13. Earlier: CLOSED for rev11's patterns too (calm-crab msg_a019e137: rev11 post-run body verbatim over 6 retained logs, incl. a mixed PASS/FAIL rc-1 all-bare log ACCEPTED with 0 deviations; panic, reasoned, closure-refused logs and a forced-rc control REFUSED; 0 TAB bytes, no "Started"-type lines, no real name ending in t, so J7/J9's TAB and t cases rest on synthetic fixtures only). Earlier: CLOSED for rev10's patterns (calm-crab msg_6627419e: rev10 decision body verbatim over 5 retained logs; all-PASS accepted, 4 bad logs + forced-rc control refused). rev11 changed the boundaries (J7, J8): re-check of rev11's exact patterns against the same 5 logs requested | none locally |
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
| C12 | `mark`'s readback `[ "$(timeout $c cat -- $t)" = "$body" ]` ignored the read's rc: a cat that prints the exact temp file and then fails (rc 1, or timeout's 124) still published the marker, rc 0 | keen-otter (rev12 review, executed failure injection on the real mark) | readback assigned and rc-checked BEFORE comparing (rev13) | mark-readback-rc1, mark-readback-rc124 (refuse), mark-readback-nomarker1/124 (no marker) |
| C13 | the dual of C8 and C12 in `need`: `[ "$(mfield $m SHAS)" = "$(shas_id)" ]` is TRUE when shas_id fails and the marker's SHAS field is empty | self (rev13, generalizing C12) | both sides assigned, rc- and shape-checked, then compared (rev13). Rule applied: an acceptance comparison against a command's output checks that command's rc first. Inspected every `[ "$(...)" = ...]` in body-common and the 7 bodies: also converted J1's three `git rev-parse` comparisons. Left as is: `grep -c` counts (a read error prints no count, so it cannot equal the expected value) and `hash_t` comparisons (failure prints nothing, which never equals a 64-hex digest) | need-shas-fail |
| G1 | rc-before-content was not applied everywhere: J1's built identity and J2's served identity read `--version 2>&1 \| head -1` with no rc check (a stub printing the right line and exiting 1 or 124 was ACCEPTED); C13's "every [ \"$(...)\" = ... ]" claim missed the assign-then-compare form | sleek-owl (rev13 review M1, M2) | `capt` (whole bounded command, exact rc, no head) for both identity gates (rev15) | version-j1-rc1/rc124/rc0, j2-version-rc1/rc124 |
| G2 | producer status lost at authority/verdict gates: need_one's mfield and exp/got pipelines, verdict's grep, roster_gate's process substitution, fns_csv, oomread/classify/completed | sleek-owl (rev13 review m1, m2), keen-otter (msg_9669a3e4: SUBSTANTIVE) | mfield and need_one rc-checked, exp/got built with builtins from captured data; r2 overrides of verdict, roster_gate, fns_csv, oomread, classify, completed with the same law and captured status (the frame's originals stay byte-for-byte for its own fixtures); override self-test at every startup (rev15) | inj-mfield, inj-verdict, inj-fns-roster, fns-control, inj-completed, oom_selftest |
| G3 | grep tests read rc >= 2 (error) as "no match": the CR gate, the closure/resolve gate, the closure checks; the frame's CG via an unchecked sed | sleek-owl (rev13 review m3) | `gcnt`/`gtest` everywhere (0 match, 1 no match, >= 2 refuse); an independent CG re-check (rev15) | inj-cr-gate, inj-closure-diag, inj-closure-summary, inj-cost-partition, inj-control (success), cg-recheck |
| G4 | the J2 pre-reload gate diffed two process substitutions of an awk filter: an awk failure made both sides empty and the gate PASS before daemon-reload; J4's listener attribution compared two `grep -c` pipelines (an error made both sides empty and equal) | self (rev15, while building the inventory) | `not_dest_to` writes files with awk's rc checked, then `cmp` 0/1/other; J4 parses listener lines with builtins (rev15) | j2-notdest-awk, j4-nopid, j4-happy |
| G5 | the startup self-tests had no bound ("~20 s" was an observation; the join self-test's caps were 35 x (600 + 60)) | sleek-owl (rev13 review m4) | the join self-test runs under ONE aggregate deadline (240 s + one 30 s kill grace); contain_selftest's loops are fixed (about 10 s) (rev15) | join-bound (every call slowed by 8 s: gave up in < 300 s) |
| GATES | a comprehensive inventory of every capture/compare site | sleek-owl (msg_1b45cc1a) | GATES.md: 144 source sites classified (H/A/P/B/V/R/F) with generated job:line and the regression fixture; frame excerpt listed separately | see GATES.md |
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

## rev16 structure: the root acts leave the jobs (keen-otter msg_bc8bc508 / msg_e41d8062)

| # | Finding / decision | Source | Change (rev16) | Fixture |
|---|---|---|---|---|
| K0 | every sudoers-interpreter round found another mishandled shape (S1-S12, overlap, last-match, globs, groups, options); interpreting sudoers is a second authority for what sudo decides | sleek-owl msg_af194434, keen-otter msg_bc8bc508 | the parser and every sudo read are DELETED (S1-S12 are superseded, not fixed); J2 and J2's successors run no sudo at all; the root acts are two standalone reviewed scripts, K1 and K2, whose holder is named by the operator's reply to escalation msg_e8888e14 (see "K1/K2 authorization record"; capability UNVERIFIED) | the deleted sudo-* cases go with the parser |
| K1 | install + daemon-reload as one root act with a disposition at every boundary, bound to J2's request by hash | keen-otter msg_bc8bc508 | K1.sh: B0 binding (request sha; J2_READY binds exactly this request; J2_PARTIAL absent; a private root-owned copy of the unit text hashed against the reviewed sha and installed FROM that copy), B1-B7 with stated dispositions (B5: dest removed and the inventory VERIFIED equal to pre, else UNRESOLVED; B6/B7: UNRESOLVED, no further action), one whole-act deadline; receipt OUT/k1-receipt.txt | k1-happy, k1-b1-install, k1-b2-stage-readback, k1-b3-rename, k1-b4-dest-readback, k1-b5-verified, k1-b5-unresolved, k1-b6-reload, k1-text-moved, k1-unbound, k1-partial |
| K2 | start/observe/stop as root with a bounded cleanup that does NOT depend on an asynchronously available keeper | keen-otter msg_bc8bc508, sleek-owl msg_e530c6f9 | K2.sh (replaces J4): binds the K1_OK receipt and J2_READY, re-reads the J3 receipt and the installed bytes live, starts, observes with one deadline, and on ANY failure stops THIS start's invocation in the same run (STOP_RESERVE 360 s); the unit cgroup must exist BEFORE the stop (a wrong path never reads as "gone") | k2-happy, k2-deadline, k2-subtree, k2-cg-missing, k2-nopid, k2-receipt, k2-dest-moved, k2-stall, k2-late, k2-slowcg-listen, k2-slowcg-stop |
| K3 | the K scripts' capability and frame | keen-otter msg_e41d8062 / msg_560eeac5, calm-crab msg_e90e4723 | both carry CAPABILITY / PROVENANCE (escalation msg_e8888e14 2026-11-06T20:12:59Z with its exact question and options; the operator's exact reply ~20:15Z; keen-otter's resolution: K1/K2 by loyal-deer-449 as briansrls, NOT J3, not to be re-asked; sudo capability UNVERIFIED; frame UNDECIDED; all holds remain) and the EXECUTION FRAME as review material. See "K1/K2 authorization record" | none (a review decision) |
| J2R | J2 restructured: no root acts; admit -> apply -> identity -> the exact 3-line K1 request; J2_READY marker binds it | keen-otter msg_bc8bc508 | FULL BUDGET BEFORE EACH MUTATION (NEED_ADMIT 6570 before admit, NEED_APPLY 5450 before apply); itemized budget and honest partial-state disposition in the header (no rollback claimed); the false "each capped at 300 s" claim deleted; inventories are one bounded find + one bounded hash pass per root (no per-entry untimed stat; no die2 inside a redirect); the unit-config inventory must be IDENTICAL before and after apply (step=apply-units) | j2-happy, j2-budget-admit, j2-budget-apply, j2-inv-fail, j2-enable, j2-live-unit-drain, j2-live-unit-failed-ok, j2-version-rc1/rc124, j2-kill, j2-ssh, inv_fixture |
| J4R | J4 retired (K2 replaces it): removed from gen.py, REVIEWED_SCRIPTS, MARKER_JOB, EXPECTED_RECEIPTS, CHAIN_PRIORS | - | - | - |

## K1/K2 authorization record (review record; keen-otter msg_e41d8062 / msg_560eeac5)

- **Escalation:** msg_e8888e14, created 2026-11-06T20:12:59Z.
  - Question: "Who performs the srv1 lab restoration's root acts (install serve unit, daemon-reload, start/stop)? keen-otter requires a named root holder."
  - Options: "you run reviewed K1/K2 scripts with sudo yourself" (recommended) | "you attest existing exact NOPASSWD argv rules on srv1".
- **Operator's direct chat reply, about 20:15Z, exact:** "You have srv1 access to briansrls, I give you permission to run those"
- **keen-otter's resolution (msg_560eeac5):**
  - The permission covers loyal-deer-449, as briansrls, performing the described reviewed K1/K2 acts.
  - It does NOT include J3.
  - It must not be re-asked.
  - Sudo capability stays UNVERIFIED (no probe; a `sudo -n` refusal at run time is a stop).
  - The execution frame stays UNDECIDED (the EXECUTION FRAME section in K1.sh and K2.sh: one attempt; serialized with calm-crab's launcher, START/END announced; receipt and reconcile).
  - All review, chain and exact-packet holds remain: nothing runs before K1/K2 are byte-reviewed, and before J1 -> J2 has been granted and run through calm-crab's launcher.
- **J3 (the .path units):** a separate act, with execution authorization NONE (J3.md).

## rev16 fixes (independent reviews of rev15, calm-crab, keen-otter, sleek-owl)

| # | Finding | Found by | Fix (rev16) | Fixture |
|---|---|---|---|---|
| L1 | J2's live-unit check skipped any line containing belt/exit/drain (name OR description): a live attempt unit read as absent and the mutation proceeded (BLOCKER, demonstrated) | independent gate review | every matching unit in any state (--all); only ACTIVE inactive/failed do not count; no substring exclusion | j2-live-unit-drain, j2-live-unit-failed-ok |
| L2 | J1R could publish a leftover `.render.tmp` from an aborted run and report DONE (BLOCKER, demonstrated) | independent gate review | a leftover .render.tmp refuses; `cp -a -T` | none locally (the J1R body has no fixture run; stated) |
| L3 | tmp_list's rc leaked the LAST [ -e ]/[ -O ] result: a dangling/foreign last /tmp match refused a wet leg | calm-crab M1, sleek-owl, keen-otter | explicit filter status; only the sort can fail | tmp-list-dangling, tmp-list-sort-fail |
| L4 | the join self-test accepted ANY exit 2: deleting most single checks went undetected; budget exhaustion could pass a want=2 case | independent gate review, keen-otter msg_dc650b95 | every negative case asserts ITS reason; any budget/deadline refusal fails the self-test; OPDL shadowed so containment waits are inside the 240 s deadline; a mutation test deletes each check in turn. The canonical-count check is implied (malformed = 0 and outcome-like = roster), so no input can reach it first: kept, stated not discriminable | 36 join cases (incl. new cr-gate), mutation[...] x 10 |
| L5 | the PATH could resolve the witnesses' bare tools through a user-writable directory first on PATH | independent Y-SP review | path_guard: every listed tool must resolve to /usr/bin or /bin | path-guard |
| L6 | the positive Reconciled gate used grep -q (early success is not a complete read) | keen-otter msg_dc650b95 | full-read count (gcnt) == 1 | inj-cost-partition |
| L7 | J1's #15175 absence gate accepted merge-base rc 1 in a shallow clone (traversal stops at the shallow boundary: a hidden ancestor reads as absent); lazy fetching in a partial clone was unguarded; the conditional fetch's own rc was hidden by a later cat-file | keen-otter msg_cf03a808 | promisor_guard (git config rc 1 ok / 0 refuse / other refuse) right after the clone and again at the gate; shallow_guard (exact `false` from rev-parse --is-shallow-repository, rc checked); no15175_gate accepts rc 1 only after both; fetch-main's own rc decides | anc-shallow-hidden (rev15's exact line ACCEPTS, rev16 refuses), anc-full-ancestor, anc-full-nonancestor, anc-mergebase-error, anc-shallow-read-fail, anc-shallow-empty, anc-promisor, anc-config-error |
| L8 | J1B1's empty-mktemp bound missed `/attempt-slots-cutover-*` | independent Y-SP review | added | j1b1-emptymktemp (same mechanism) |
| L10 | J1B1's fabric receipt was written through an unchecked `\| tee` and was not a bound receipt | sleek-owl | one rc-checked write + read-back compare; O:fabric-posts.txt added to J1B1_GREEN's EXPECTED_RECEIPTS and passed to J1B1's mark | fabric-gate, fabric-receipt |
| L11 | J3 (the .path units) is a SEPARATE operational act, not covered by the operator's K1/K2 permission | keen-otter msg_e41d8062 / msg_218a93f9 | J3.md: exact units, argv, expected effects (enable; the triggered services), pre-state, readback, the 3-line J3_OK receipt K2 consumes; authorization for execution NONE; it needs its OWN execution decision; the executor is undecided (calm-crab holds no separate root). J3-PLAN.txt is unchanged | k2-receipt (the receipt law K2 enforces) |
| L9 | GATES.md's job:line cells were a string slice; bare `|| die` gates were not listed | independent gate review | generator fixed (locs as a list; bare `|| die`, `&& die`, `case $?` sites; K1/K2 included): 306 rows, every one classified | GATES.md |

## J2 (install) and J4 (start) — HISTORY: superseded by K1/K2 in rev16

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
| S12 | the overlap check recognised only exact, bare or ALL commands: a PASSWD rule with a glob (`/usr/bin/r*`), a `%wheel` runas, a comma list or a directory was MISSED and ACCEPTED | sleek-owl (rev13 review M3) | ANY rule the parser cannot classify exactly (runas other than ALL or a plain name; a command that is not ALL or an exact absolute path; RunAsGroups items likewise) refuses the whole precheck (rev15). No effective-permission claim is made from a partial parser. COST: a host whose listing carries the default `%sudo ALL=(ALL:ALL) ALL` PASSWD rule for this user makes J2/J4 refuse even if a later NOPASSWD rule would win in real sudo | sudo-glob, sudo-runas-group, sudo-comma-list, sudo-directory (+ the rev13 overlap cases) |
| S11 | the precheck ignored overlap: rule A (root, NOPASSWD, ALL) then rule B (root, PASSWD, /usr/bin/install) was ACCEPTED, but real sudo (last-match-wins) would prompt for install MID-MUTATION. MATERIAL to the pre-mutation guarantee | sleek-owl, keen-otter (rev12 reviews) | FAIL CLOSED ON OVERLAP (rev13): every rule that runs as root and lists the exact argv must be NOPASSWD with no negations; one matching rule that is not makes the precheck refuse before any mutation, whatever its listing position (the order of `-ll` is not relied on). The probe creates no authorization | sudo-last-match (A then B), sudo-overlap-reversed (B then A); both refuse pre-mutation |

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
| R3 | closure_gate added up to 930 s PER ENTRY before the run caps; rev11's budgets were rev10's totals | keen-otter (rev11 review) | ONE closure query per job and worktree over all its entries (CQCAP 600; observed 130-160 s per query, calm-crab msg_a019e137); run_entry refuses an entry the query did not admit; FULL-CAP-OR-REFUSE: a claim run or closure query starts only if its whole cap fits, otherwise it refuses BEFORE starting; J1B3 RT 6600 -> 7200 (launcher max). Budget table below (rev12; J1 itemised in full in rev13) | join self-test (every case admits its entry first); fullcap-run and fullcap-closure exercise both refusal branches and show claim_batch is never invoked (rev13) |
| R4 | per-job batching changed the CLI usage from one --entry to several | keen-otter (rev12 review) | the gate now also refuses a duplicated requested entry, and requires the per-entry row counts to SUM to path_rows, so no row can belong to an unrequested entry (rev13). The multi-entry output contract (one row group per entry, a single summary with entries=N and the total path_rows) is read from claim_batch.rs at 26fe4b8f: confirmation from calm-crab requested. Loader success is NOT typecheck or eval | closure-foreign-entry |
| E1 | GUNBC_WORKSPACE_ROOT overrides the CLI's workspace walk; a set override would make every gunbc/claim_batch call read another tree | keen-otter (msg_57af3d65) | rev13: refuse before any CLI call if GUNBC_WORKSPACE_ROOT, or ANY exported GUNBC_* name, is set; names via the builtin `compgen -e`, values never read or printed. Generalized because the CLI at the pin reads at least 26 GUNBC_* names by literal and others via generated constants (GUNBC_WORKSPACE_ROOT is one). The GUARD MECHANISM is closed (keen-otter executed it, msg_49296e3f). Whether the host environment exports any GUNBC_* name is UNKNOWN: calm-crab's retained launcher source exports none (only PATH), but that says nothing about the inherited user-manager environment, and one past panic evidences only GUNBC_WORKSPACE_ROOT unset at one moment. calm-crab's proposed show-environment launch guard was reverted and is unapproved, so it is NOT relied on. The in-job guard decides at run time | env-GUNBC_WORKSPACE_ROOT, env-GUNBC_EVAL_MEMO (refuse; the value never appears in output) |
| Y2 | the Y-SP trace took the .dag service-verb declarations at face value, and the empty-mktemp branch was untraced | keen-otter (msg_aaaec02c) | Rust realizations traced (APPLY-EFFECTS Y-SP "Rust side"); the empty-mktemp branch roots paths at `/` (string concatenation, never worktree-relative); J1B1 refuses up front if root, if `/` is writable, or if any of those top-level paths exists (rev15). The witness's own fresh_dir (ignores mktemp's status) and the unenforced `"" as FilePath` are corpus defects outside this packet | j1b1-emptymktemp |
| R5 | the closure path shape accepted `file="dag/../src/v1/x.dag"` | sleek-owl (rev12 review) | any path with a `..` segment refuses (rev13). The omitted-root evidence remains the loader rc and the complete summary, not the regex | closure-dotdot |
| R1 | run_entry passed roots [dag, src/v1, src/v2]; `cli_run::resolve_virtual_source_with_imports` demands `witness_layer_roots` [dag, src/v2] on the shared slot claim_batch fills first, and a different key refuses (SharedIndexSecondResidentPool) -> a possible deterministic NO_OBSERVATION | calm-crab (msg_6627419e), keen-otter (msg_8cdaf17a) | rev11: claim runs use EXACTLY [dag, src/v2]. Before each claim run `closure_gate` (a) requires the pinned authority row `gunbc.ci.ci_layer_roots` `witness_layer_roots` to read exactly that list, and (b) runs `claim_batch --print-entry-closure` (non-evaluating) with those roots. rc 0 plus one complete summary for exactly the job's entries IS the evidence: a module findable only under src/v1 makes the loader fail with rc 1. NOT executed by the gate (sleek-owl rev11 M1): that no selected function reaches a SLOT-demanding builtin rests on the static trace only; what the gate executes is that every closure resolves with the authority roots, so the claim run's slot key equals witness_layer_roots and the mismatch cannot arise in it (calm-crab msg_a019e137 verified this CLI contract at 26fe4b8f). The dag/src/v2 path filter is defence in depth only: with only those roots every printed path is under them, so it cannot otherwise fail. rev12: one query per job and worktree, see R3. Static trace at 26fe4b8f (source only, nothing run): no selected roster function, J1 OFF/RED included, reaches a slot-demanding builtin; 0 src/v1 modules in any entry's import+qualified closure; J1R/J2 `gunbc run` already used [dag, src/v2] | closure-v1, closure-nosummary (join self-test) |

## Budget (rev16; every on-path cap, the startup self-tests and need's chain walk included)

W = RT - 60 (exit margin) - 300 (RESERVE): the window in which tcap lets a claim run or closure query START. All figures are
CEILINGS (per-step timeouts, kill graces included); they are not predictions. Startup per job: the frame's own fixtures
(bounded only by RT; local git init/commit), contain_selftest ~10 s, and, in J1/J1B1-4, the join self-test under ONE aggregate
deadline of 240 s + one 30 s grace (~280 s with contain_selftest). need's chain walk is per-receipt hash_t (60 s ceilings on small
files) + shas_id + the prior hash per marker, + wt_ok 240 + shas_ok 120 + the ignored-set snapshot 150, all under OPDL.

| Job | RT | W | startup | need chain | closure | claim-run caps (+40 contain each) | worst-case sum | margin |
|---|---|---|---|---|---|---|---|---|
| J1 | 7200 | 6840 | 280 | none (first job) | 2 x 640 | 1540 + 1540 | 7610 before R + 1540 | **refuses R in the worst case** |
| J1R | 4200 | 3840 | 10 | 990 | none | renders 3 x 900 + preflight 900 (budget-capped, a cut render refuses) | 4600 | **-760** |
| J1B1 | 6600 | 6240 | 280 | 1530 | 640 | 1540 + 1540 + 1240 | 6770 | **-530** |
| J1B2 | 6600 | 6240 | 280 | 1950 | 640 | 2040 + 940 + 1240 | 7090 | **-850** |
| J1B3 | 7200 | 6840 | 280 | 2370 | 640 | 1540 + 1240 + 1240 + 1540 | 8850 | **-2010** |
| J1B4 | 6600 | 6240 | 280 | 2850 | 640 | 640 + 640 + 1240 + 1540 | 7830 | **-1590** |
| J2 | 7200 | - | 10 + inventory fixture 1440 | 3330 | none | admit 930, apply 2430, post 2960 (header itemization) | pre-admit caps ~6280 | **refuses BEFORE admit in the worst case** |
| K1 | - | 900 (one deadline) | - | - | - | B0-B7 caps | <= 900 | deadline reports the boundary's state |
| K2 | - | 2340 (one deadline) | - | - | - | work to WORK_END, stop reserve 360 | <= 2340 | stop always inside the reserve |

**No job claims that every ceiling fits** (keen-otter msg_dc650b95: preserve the limitation). What holds instead: a claim run or
closure query starts only if its WHOLE cap fits (full-cap-or-refuse); J2's admit and apply start only if NEED_ADMIT / NEED_APPLY
remain (a slow precheck phase refuses before admit, nothing mutated); J1R's renders and all bookkeeping are bounded by budget()
and OPDL and refuse when short. So a shortfall is a typed refusal BEFORE a run or mutation, never a run cut off before its roster
or a mutation started on a shortened cap. The real costs of need's hashes, the inventories and the self-tests are seconds locally;
srv1's real build duration and closure-query cost at this pin are NOT measured here (the 130-160 s closure figure is a citation,
calm-crab msg_a019e137, with no receipt in this packet).

J1 detail (rev13's itemization, + 90 s for the rev16 promisor/shallow guards, startup 280 instead of ~20): sum before the R leg
7610; the R leg starts only if <= 5340 s are used, so in the worst case J1 refuses R before starting it (fullcap-run). The
structural alternative (R in its own job) is still offered, not done.

## Section Y (APPLY-EFFECTS)

| # | Finding | Found by | Status | Fixture |
|---|---|---|---|---|
| Y1 | Section Y was not closed: placement tests #2-21 and #34-45 were classified by their constructors, not traced line by line, and the live-effect fields were checked by symbol search only | sleek-owl (rev10 review) | CONDITIONAL (keen-otter msg_aaaec02c): .dag-level trace done (rev14, source only, 26fe4b8f), pending an independent reviewer and the two residuals: every body followed through every executed callee; all 32 LOCAL-ONLY, no verdict changes; the only instance fields read are instance_root, dispatch_state_scope, worker, directory_owner, host_identity, dispatch_worker_memory_max and instance_id; the live-reaching functions (systemctl, /proc, real-instance resolution) are never entered. Two corrections to Y-SP (holder evidence reads the fixture ledger; unlisted `date +%s` spawns). Residuals stated in APPLY-EFFECTS Y-SP: the Rust service-verb realizations were not read; the empty-mktemp-path branch was not traced. Reviewer acceptance pending | none (source trace) |

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

rev16 FINAL bytes: same inversion (`all-pass` 0 -> 2, its reason field set) in a scratch copy; J1, J1B1, J1B2, J1B3, J1B4
each `startup exit=2; join fixture all-pass: exit 0, required 2` (reaching the join self-test, past the env, clock, CG, PATH and
helper checks). J1R and J2 do not run claim_batch, so they carry no join self-test; K1/K2 are standalone root scripts.
rev15 FINAL bytes (the published ones): same inversion; J1, J1B1, J1B2, J1B3, J1B4 each `startup exit=2; join fixture
all-pass: exit 0, required 2` (reaching the join self-test, i.e. past the new env, clock, CG and helper checks).
rev13 bytes: same inversion in a scratch copy; J1, J1B1, J1B2, J1B3, J1B4 each `startup exit=2;
join fixture all-pass: exit 0, required 2`.
rev12 bytes: same inversion in a scratch copy; J1, J1B1, J1B2, J1B3, J1B4 each `startup exit=2;
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
