BRANCH-VINTAGE BINARIES FOR integration/namespace-cut (gunbc#8282)
DELIVERED VIA GIT, DELIBERATELY. Binaries in git is ugly; it is also the only channel
demonstrated to cross session containers. THE FILESYSTEM IS PER-CONTAINER: /session-home
is session-local, and so is /home/briansrls -- it merely LOOKS shared because each
container has a .worktrees populated from the same origin. The discriminating test:
one session could not see another session's own worktree under /home/briansrls/.worktrees.
So no filesystem path either party can name is readable by the other, and writability was
never the binding constraint -- visibility was.

The usual objection to binaries in git does not apply here: THESE ARTIFACTS ARE
IRREPRODUCIBLE. The branch cannot build a claim_executor, which is the entire reason
they matter.

  git fetch origin receipt/preserved-namespace-cut-binaries
  git checkout FETCH_HEAD -- bin SHA256SUMS.txt README.txt
  gunzip -k bin/*.gz && sha256sum -c   # compare against SHA256SUMS.txt (raw lines)
  chmod +x bin/<name>

WHY THESE EXIST: the branch head does not build (broken since c98516772e7, 2026-08-22),
so these CANNOT be rebuilt from the branch as it stands. They were built from the cut
branch earlier, before the head, and preserved 2026-08-24 when the breakage was found.
linux/arm64.

  gunbc_branch_vintage_2026-08-23T09-04            sha256 f09f5d56f41de561...
      `gunbc`. RUNS, and RESOLVES the import-free corpus: 3875 modules indexed, 42 errors
      ALL in ONE file (dag/gunbc/bare_name_identity_consumer_census.dag,
      `unresolved type std.types.String`). A MAIN-vintage binary on the same corpus:
      520 errors, 205x undefined `extdeps`, 47x undefined `std`. That contrast is the
      measured proof the capability inversion is a fact about MAIN-vintage compilers only.

  claim_executor_branch_vintage_2026-08-23T06-28   sha256 bd2d6354010ce613...
  claim_executor_branch_vintage_2026-08-23T04-18   sha256 610ca855ba7ab7c7...
      `claim_executor` -- the binary that drives regen, and the reason this directory
      matters most. `--required-regen` on the branch REFUSES: unresolved type
      'std.induction.SubValueRelation', 'std.syntax.BinOp', 'std.types.SourceSpan',
      all in src/v1/00_core.dag -- the SEED's own .dag, not merely its mirrors.

WHAT IS ALREADY RULED OUT, so nobody re-runs it:
  `gunbc compile --source-root dag --source-root src/v2 --source-root src/v1 --target rust`
  driven by the gunbc above -> EXIT 1, ZERO FILES EMITTED. Refuses on the branch's own
  corpus: variant 'BuildGreen' not found in type 'BuildOutcome' (dag/gunbc/devboot/build.dag),
  non-exhaustive match missing Built/BuildRefused, function 'changed_surface_skip_authority'
  not found in scope. The cheap bootstrap path is CLOSED.

Full diagnosis: gunbc#8282 PR body (top section).
