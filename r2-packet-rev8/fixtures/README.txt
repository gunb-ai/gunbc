Local fixture harnesses for the r2 packet. They run the REAL generated job scripts from ../ against stubbed host tools and a
fake filesystem, all inside a private temp dir. No sudo, no real systemctl, no srv1, no network.
Usage, run from this directory: bash run-all.sh
Each case prints "<case> PASS|FAIL (expected <outcome>)". The exit is 0 only when every case gives its expected outcome.
Edits applied to each COPY of a job script, never to the job file itself:
  - the scope gate is stubbed (true);
  - CG points at a fake cgroup;
  - the want lines are no-ops;
  - paths are rewritten into the temp root;
  - prior_scope_gone/need are stubbed for J2/J4, because the chain fixtures exercise those separately.
Cases:
  frame       the frame and join-law self-tests in all 8 jobs
  join-neg    the join self-test refuses when run_entry's two count checks are removed
  chain       a genuine 3-link chain verifies; tampering the oldest receipt, an unreviewed older script, an ignored path appearing,
              a marker write failure, and a printf-forged marker each exit 2
  contain     a setsid -f survivor after exit 0 is killed and verified gone
  j2-happy    J2 reaches J2_INSTALLED with exactly 1 reload
  j2-enable   an apply-created enable link pointing at DEST refuses at pre-reload, with 0 reloads
  j2-kill     SIGKILL during apply leaves J2_PARTIAL, and the rerun refuses
  j2-passwd   a sudo rule without NOPASSWD refuses before any mutation (nothing admitted)
  j2-ssh      ssh srv1 with BatchMode no and no known_hosts entry refuses before any mutation
  j4-happy    J4 reaches J4_UP
  j4-deadline no listener: deadline reached, stop verified, exit 2
  j4-subtree  a pid survives in a sub-cgroup after stop: UNRESOLVED
  j4-nopid    a listener line without a pid refuses
  j4-receipt  a stale-InvocationID receipt and an extra-line receipt both refuse
  j4-stall    the post-start property read hangs: bounded, explicit outcome inside RT
