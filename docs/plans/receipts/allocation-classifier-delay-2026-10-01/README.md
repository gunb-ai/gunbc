# Required-floor classification delay

Run 36919023056 at 1c19906c843 exceeded the existing 90-minute floor boundary.
Generated and emit-build succeeded. The fleet-file edit classification consumed
3,360,027 ms (56 minutes) for six identities. This occurred before test execution;
it was not a 56-minute execution of the six witnesses.

The classifier eagerly resolved base-file imports before checking whether the
edit was a conjunct removal. The repair keeps tokenization, declaration extraction,
and token comparison unchanged, and resolves references only after a nonempty
conjunct removal is established. Rejected edits retain their existing causes;
qualifying removals still check the same discovered test-function identities.

Focused controls and real-file timing are recorded separately. This does not
establish a passing integrated floor or authorize live commissioning by itself.
Memory and timeout limits are unchanged. No host or runner services were changed.

All 17 focused classifier controls passed with exit 0 over a byte-identical
117-module closure, using compiler SHA256
621b46ed699db2d4a90a11a688d2c030a04272fd76e37d463b5ae0ac9f6f5113
under 6 GiB/no swap.

The real-file benchmark timed out with exit 124 at its 25-minute bound under
6 GiB/no swap. Its log is /tmp/allocation-cost-debt-lazy-benchmark.log. The
lazy-resolution repair did not establish adequate performance or an integrated pass.

On October 2, main was reconciled through b76ac274c4bd2118b3457516c6a19b9ded020e9d
without conflicts. This includes upstream #12927 (8318e08374e), which avoids
constructing unused list/string tails in wildcard Cons patterns used by the lexer.
A fresh compiler build and same-input qualification are required before attributing
any performance improvement to that change.

The reconciled compiler built successfully in 7m04s under 12 GiB/no swap.
All 17 classifier controls passed (exit 0) under 6 GiB/no swap. Binary identities
are in reconciled-identity.json. The old-compiler stage probe exited 124 after
10 minutes before its post-tokenization marker was written: declaration scanning
was not reached. The reconciled real-file benchmark is separately bounded to
10 minutes; its terminal result must be recorded before claiming sufficient speed.

The reconciled same-input benchmark timed out after 10 minutes (exit 124).
This shorter bound does not establish whether it is faster than the earlier
25-minute timeout; neither produced a complete classification verdict. No
sufficient speed improvement or integrated qualification is established.
The input SHA256 is recorded in reconciled-identity.json. Further work must
attribute and reduce the modeled tokenizer/interpreter cost before another
full CI run; no limits, verdict requirements, or witness populations were relaxed.
