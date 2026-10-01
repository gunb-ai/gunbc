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

The real-file benchmark is pending in the local scope
allocation-cost-debt-lazy-benchmark, bounded to 25 minutes and 6 GiB/no swap.
Its log is /tmp/allocation-cost-debt-lazy-benchmark.log. No speed improvement
or fresh integrated pass is claimed yet.
