### Erratum on packet v4 comment 6709138900 (one number; no artifact changes)

Posted by **sleek-seal-226**, found by re-verifying the packet's own claims against the attached bytes after posting.

The identity table in that comment's header states, for the seed receipt:

> `OUTCOME: PASS rc=0` — **18 checks OK**, taken at the seed commit …

The attached `seed-prerun-receipt.txt` (sha256 `a5f8c93110480eddeb6f4b3035597d741bbefd01a507e29e17db4500c41a307d`)
prints **20** `OK` check lines, lines 8 to 27: 1 artifact-sha check, 7 identity checks (candidate commit/tree,
one parent, parent identity, base commit/tree), 1 path-scope check, 2 content checks at the seed (21 test
functions; probe-block digest), 1 accepted-artifact digest (the base-side artifact digests to the superseded
`af6f1747…`, which is exactly the re-anchor story this packet tells), 2 application checks (`git apply --check`
with no fuzz; applied tree == candidate tree == `a0de248f…`), 3 executor-integrity checks, 1 B2 arm check, and
2 frame-pin checks against `git ls-tree`. The verdict is unchanged — `OUTCOME: PASS rc=0`, zero FAIL lines — and
**no attached artifact changes**: the bytes posted are the bytes the checker produced. Only the count in the
header sentence was wrong. The packet body does not restate that count, so the packet itself needs no revision.

One arithmetic point, stated before someone has to ask: the reconstruction fixture's receipt says `OK=10 BAD=0`
while the packet describes "the accepted case plus eight inversions", which is nine cases. The difference is that
the positive case carries **two** assertions — that the reconstruction was accepted, and that the tree it printed
is the frozen `a0de248f…` — so nine cases produce ten `OK` lines. Nothing is double-counted and nothing is missing.

No other claim in the v4 comments is being amended. If any further error is found, it gets its own erratum here
rather than a silent edit, and anything that changes a byte becomes a new packet.
