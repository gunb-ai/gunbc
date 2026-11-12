# genimp.py - scratch helper for the import-less witness population (NOT a repo instrument)

`python3 tools/genimp.py dag/test/claim/<witness>.dag` prints candidate
`import <home-module> { <names> }` lines for symbols used but neither declared
locally nor already imported. It resolves each symbol to the module that
declares it (declaration grep over dag/ + src/v2), with a builtin allowlist.

EPISODIC scratch tool from session tidy-swift-586 (PR #16359), offered so the
remaining import-less witnesses (182 under dag/test/claim at 49bca34f) can be
dispatched as one lane instead of file-by-file. Not a .dag instrument, carries
no claim; treat output as a first draft.

Verified limitations from #16359 (each caused a wrong import or a miss):
1. Names used as bare VALUES (`target_host: srv1_host`) are not matched by the
   `name(` heuristic - scan struct fields too, or the effect checker later
   refuses 'identity established but names no registry row'.
2. Symbols with NO declaration in the tree must be dropped, not guessed.
3. Unresolvable rows (`import Sym { UNKNOWN }`) are garbage - every name needs
   a real declaring module.
4. Cross-brand casts (`x as String as DnsLabel`) are not generated; witnesses
   passing data into branded parameters need them by hand.
5. Only EXECUTION certifies: run each file under claim_batch (one entry per
   process, GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES=450). Effect-callee and
   stale-pin classes surface under execution, not under imports alone.
