Local fixture harness for the r2 packet (rev17). It runs the REAL generated job scripts (../ld449-r2-*.sh) and the REAL
standalone root scripts (../K1.sh, ../K2.sh) against stubbed host tools and a fake filesystem, all inside a private temp dir,
as a NON-root user. No sudo, no real systemctl, no srv1, no network (the ancestry cases use local synthetic git repos only).
Usage, run from this directory: bash run-all.sh   (about 17 minutes; GNU grep 3.11 and mawk were used)
Each case prints "<case> PASS|FAIL (expected <outcome>)". The exit is 0 only when every case gives its expected outcome.

Edits applied to each COPY of a script under test, never to the file itself:
  jobs:  the scope gate and the rev15 CG re-check are stubbed (the harness has no real job cgroup; cg-recheck tests the
         re-check itself on this container's real /proc/self/cgroup); CG points at a fake cgroup; the want lines are no-ops;
         path_guard is a no-op (the harness PATH carries its own shims; path-guard tests the guard itself on a clean PATH);
         paths are rewritten into the temp root; prior_scope_gone/need are stubbed for J2 (the chain cases exercise them).
  K1/K2: the EUID check is stubbed; PRIV_OWNER=0:0 becomes this user's uid:gid; PATH gets the temp bin dir first (stub
         systemctl/install/mv/rm/cp/cat/ss/mktemp/chmod/tee, and `runuser`: the harness is ONE user and cannot drop privilege,
         so the stand-in logs argv, which must be exactly `-u briansrls -- ...`, and runs the command, or in the
         no-user-writes cases a sandboxed user-side writer); /etc/systemd/system, /etc/systemd/user, the user unit dir,
         /run/systemd/system, /run/r2-k1, /run/r2-k2 and the work dir are rewritten into the temp root; K2's whole-act deadline is set
         per case (X seconds) and, for timing cases, the reviewed constants are reduced (STOP_RESERVE 90, START_CAP 10,
         OBS_ALLOW per case); the cgroup root is injected by patching `CGR=/sys/fs/cgroup`, never through CGROOT.

Case families (see run-all.sh for the exact assertion of each; every negative case asserts its specific reason):
  frame[*]          the frame and join self-tests in every job (startup)
  join-neg          the join self-test refuses a broken join (its five join-law checks removed)
  mutation[*]       each join-law / closure check deleted ON ITS OWN makes the join self-test fail (the canonical-count check
                    is implied and excluded, see FINDINGS L4)
  chain-*, anchor-*, mark-*, need-*  the marker chain, receipt anchoring, marker write/readback/deadline failures
  subtree-*, contain, cst-*          containment and its real-cgroup self-test (emulated listing)
  inj-*             real-caller error injections (grep rc 2, awk rc 1 with right output) at the real call sites, + controls
  fns-*, version-j1-*, cg-recheck, path-guard, tmp-list-*, join-bound, opdl-phase, fullcap-*
  anc-*             J1's #15175 ancestry gate (promisor/shallow guards) on local synthetic repos, incl. the shallow-hidden case
                    the rev15 line accepted
  env-*             any exported GUNBC_* name refuses before any CLI call (value never printed)
  j2-*              J2 (no root): happy (J2_READY + exact K1 request, nothing installed), budget guards before admit/apply,
                    after-apply inventory failure, enable link made during apply, live attempt units, served identity rc,
                    SIGKILL partial + rerun refusal, ssh BatchMode
  k1-*              K1: happy (J2's own inv_roots output vs K1's projection: equal) and every boundary B1-B6 with its
                    disposition, a moved unit text, an unbound request, J2_PARTIAL; rev17: receipt unwritable / read back
                    different (UNRESOLVED, no K1_OK), a TERM-ignoring step KILLed at cap+grace, a late boundary refused before
                    the step, the snapshot race (originals replaced after the copy), the independent unit-sha argument, a
                    file touched between J2 and K1; rev17 (msg_c144f271): [Install] contract refused at B0 (static,
                    empty [Install]), no root write in a user path (planted links, sandboxed export), OUT renamed / export
                    failure after the mutation (UNRESOLVED step=X), OUT unusable at B0, PRIV not 700 / a symlink, J2's real
                    request end to end
  k2-*              K2 (replaces J4): happy, deadline + verified stop, sub-cgroup survivor, cgroup absent before the stop,
                    pid-less listener, stale/extra J3 receipt, installed bytes moved, stalled property read, late listener,
                    slow cgroup reads on the listener and stop paths; rev17: too little time refuses before start, an
                    invocation change without an NRestarts increase (no stop), with one (stop + verify); rev17 (msg_c144f271):
                    fail_ours through the real start path (foreign / automatic restart at the stop, start rc, empty or
                    unreadable invocation), a restart between the last pre-stop read and the stop (UNRESOLVED), snapshot race,
                    no root write in a user path, export failure after the start, OUT unusable, PRIV not 700
Inversion checks: inversion-check.sh builds a reduced harness with ONLY the named `#case` blocks, applies ONE literal mutation to a
  scratch copy of K1.sh/K2.sh, and exits 0 iff every named case FAILS (--control: iff every case PASSES unmutated).
  inversions-rev17.sh is the rev17 set (20 rows, one removed guard each); run it from this directory.
  j1b1-emptymktemp, fabric-gate      J1B1's prechecks
