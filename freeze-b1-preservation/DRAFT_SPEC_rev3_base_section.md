## The freeze-2 base, as a commit sha (adds to "The seed base is a commit sha, not a branch")

The base is frozen and accepted (parent msg_4103aeb4). It is named as a commit sha, with both parents and the
tree id, so any reviewer can reproduce it independently:

```text
base   57ce2e787aa5227fc1840961c607981f4abdde86
tree   5a14ac7ca50f43312760f092c86ebee7c679ce37
parent 1  a4a21da46284e64a31efff08585c42aaf589089a   #15999 head (repair/fatal-cause-authority; dashboard review 91633 = approve)
parent 2  7d062bda1b6dab67bc72e0c49d0eb635c45ca7f9   the head sha OF THE GREEN main v2-bootstrap RUN:
          witnesses run 42342945138, job v2-bootstrap completed success 2026-11-07T07:07:08Z
```

Reproduce: `git merge --no-ff a4a21da4 7d062bda` (automatic, zero conflicts) and compare the resulting tree to
`5a14ac7ca50f43312760f092c86ebee7c679ce37`. The base was never pushed; it is reproduced, not fetched. main had
already moved past the green run's sha before the cut, and the ruling is explicit that the main component is the
green run's head sha, not main's moving tip.

Pre-run condition at this base, measured before any packet run (receipt `bd31f058b06af1290509907fb695a1a42a002fc172d5b2af634df65685918689`,
GZB64 at comment 6703855069): `prerun_checks_freeze2.sh` (sha256 `b81822845549a3fbe096ef47d3cd0fc28ae5c5bfca2128dfbd79b7200c2c1261`)
reports **STATUS: PASS**, 10 OK / 0 FAIL, rc 0 - `git apply --check` with NO fuzz is CLEAN at this base, the applied
rtp file declares **21** `test fn` (17 registered witnesses + 4 probes), and the applied probe block digests to
`af6f17477ac0f39591abb5345840e0eac827e388072afddb39f002b7eb17bd22`.

DRAFT - not posted. Posting needs the parent's go; the patcher must assert the on-disk SPEC still hashes to
a2e6cf1c... and preserve every original line, superseding a2e6cf1c explicitly as rev3.
