Local fixture harness for the r2 packet (rev16). It runs the REAL generated job scripts (../ld449-r2-*.sh) and the REAL
standalone root scripts (../K1.sh, ../K2.sh) against stubbed host tools and a fake filesystem, all inside a private temp dir,
as a NON-root user. No sudo, no real systemctl, no srv1, no network (the ancestry cases use local synthetic git repos only).
Usage, run from this directory: bash run-all.sh   (about 17 minutes; GNU grep 3.11 and mawk were used)
Each case prints "<case> PASS|FAIL (expected <outcome>)". The exit is 0 only when every case gives its expected outcome.

Edits applied to each COPY of a script under test, never to the file itself:
  jobs:  the scope gate and the rev15 CG re-check are stubbed (the harness has no real job cgroup; cg-recheck tests the
         re-check itself on this container's real /proc/self/cgroup); CG points at a fake cgroup; the want lines are no-ops;
         path_guard is a no-op (the harness PATH carries its own shims; path-guard tests the guard itself on a clean PATH);
         paths are rewritten into the temp root; prior_scope_gone/need are stubbed for J2 (the chain cases exercise them).
  K1/K2: the EUID check is stubbed; PATH gets the temp bin dir first (stub systemctl/install/mv/rm/ss); /etc/systemd/system,
         /run/systemd/system, /run/r2-k1 and the work dir are rewritten into the temp root; K2's whole-act deadline is set
         per case (X seconds; WORK_END = X - 360).

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
  k1-*              K1: happy and every boundary B1-B6 with its disposition, a moved unit text, an unbound request, J2_PARTIAL
  k2-*              K2 (replaces J4): happy, deadline + verified stop, sub-cgroup survivor, cgroup absent before the stop,
                    pid-less listener, stale/extra J3 receipt, installed bytes moved, stalled property read, late listener,
                    slow cgroup reads on the listener and stop paths
  j1b1-emptymktemp, fabric-gate      J1B1's prechecks
