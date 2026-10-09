# Issue command boundary — 2026-09-28

Base: `e286b29962c3b89904277c64d0e0c7cc25ddb2d0`, isolated branch
`codex/issue-cut1-toposort`. The live checkout and authentication handlers are unchanged.

The Git carrier now owns an isolated snapshot for each operation and publishes against the
exact fetched remote ref. It checks the supplied issue-parent expectation in that snapshot,
then publishes event and operation receipt in one commit. A remote movement refuses; it never
silently rebases a stale command. This is a Git realization of revision CAS, not a migration
to the held fabric-storage implementation.

Operation identity binds node, authenticated principal, payload and original precondition.
The committed receipt is checked before the current issue revision, so retry after another
comment returns the original event/ordinal. Clock time is committed result data. Another
operation with identical comment text is an intentional second comment. Native forms have
server-minted keys and explicit empty-revision fields. Enhanced assignment preserves those
keys; an inline lost-response retry retains the original command.

The shared URL-encoded decoder handles native CRLF, Unicode, plus/percent encoding and
field-looking multiline text without ambiguous newline framing. Duplicate decoded names,
malformed percent escapes and invalid UTF-8 refuse. Missing expected revision is distinct
from explicitly empty expected revision.

Detail reads refuse unreadable/forked/incomplete issue history and derive standing, assignment
and form revision from one event snapshot. The attempt observation is still not connected:
the page explicitly reports that gap instead of claiming no attempts. Recents are suggestions
and may degrade independently. This cut does not install a durable consumer for Fabric launch
obligations; the existing first-time inline dispatch path remains. Retrying a committed command
does not dispatch another worker.

## Evidence and limits

- Actual `.dag` command -> disposable Git remote -> readback: PASS, including same-key retry
  after an intervening event, identical-body different command, stale revision and payload reuse.
- Forced concurrent publication: PASS, exactly one event and one operation receipt committed.
  Fixture receipt: `/tmp/gunbc-issue-command-yzokh_mx/receipt.json`.
- Native Chromium POST fed into actual `.dag` decoder: PASS.
- Emitted assignment client in Chromium with a lost response: PASS for native-form and inline
  retry identity, including preventing a different inline payload from reusing a pending key.
- Focused codec claims: 2 PASS. Focused carrier claims: 3 PASS.
- Workspace administration claims are independent: 5 PASS.
- Full serve witness closure: FAILED (exit 137 at the existing 6 GiB limit after serve typecheck).
  This is not a passing integrated floor, merge approval or deployment receipt.

All DAG executions use `MemoryMax=6G`, `MemorySwapMax=0`. Focused source closures are byte-identical
copies with source hashes, not edited test substitutes. The checked-in wet/browser controls
operate on disposable local fixtures, never production state. No memory ceiling was raised.
