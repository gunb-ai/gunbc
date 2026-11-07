# r2 restoration packet, revision 18 (exact bytes)

rev17 plus ONE fix, found on srv1's real cgroup: J1 attempt 1 (rev17) refused at setup because survivors() excluded only $$,
so contain(), called inside join_fixture's per-case subshell, TERMed that subshell (exit 143). rev18 excludes the checker's own
chain ($BASHPID up to $$, walked with builtins, fail-closed). Diff: rev18-body-common.diff; every job differs from rev17 by
that hunk plus regenerated prior-job shas. K1/K2 unchanged.

OPERATOR 2026-11-07: "A IS FINE" (direct briansrls user scopes in place of the absent launcher); ~16:20Z "please delete the
heavy review process around srv1".

Verify: cd r2-packet-rev18 && sha256sum -c SHA256SUMS
Harness sha256: 9726aeb1a8717049e6ceace2e5ae5a9996276d42eb977bd0a8aadb4393b1385b -- 139 cases, failures 0, exit 0 on this tree.
srv1: J1 attempt 2 (rev18) passed every in-job self-test on the real cgroup before it was stopped (16:05:27Z, supervised).
