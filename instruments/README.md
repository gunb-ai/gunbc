# #14152 measurement instruments (scratch; NOT for the PR)

These are the throwaway probes and dispatch scripts I used to measure the unify_generics fold on
the CI corpus: the rebind census (rebind_census.sh, its output, and the self-host log it parsed),
the substitution/producer probes, the seed-swap dispatch scripts, and the regen attempts.

They live here, cited by commit, rather than in the PR, for the reason given in review: a
hand-authored shell parser over CI logs landing in tools/ is the §6 tell -- raw shell implementing
semantics with no final consumer. The claims in PR #14152 cite this commit instead of shipping the
instrument. If the census is worth keeping as an instrument, it becomes a gunbc test label later,
as its own decision.

Reproduce: from a checkout of this branch, `bash instruments/<script>`; each script is self-contained
and dispatches on the remote runner via ctrl-build. The seed carriers they swap in are seedred/ and
seedgreen/ (recovered with `git show 29f17f288d9:seedgreen/<file>`).
