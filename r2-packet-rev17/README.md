# r2 restoration packet, revision 17 (exact bytes)

Fixes keen-otter's five K1/K2 findings (msg_029acc92) and sleek-owl's independent review items (msg_cfe1a18a, msg_c144f271);
details in FINDINGS.md (rev17 section) and GATES.md.
- Receipts: checked write + readback before success; failure after a mutation is UNRESOLVED, exit 2.
- Root writes only into a verified root-private dir (owner 0, mode 700, not a symlink); exports to OUT run as briansrls (runuser).
- Every root step is under `timeout -k` against a whole-act deadline; authority inputs are snapshotted, then hashed and parsed from the copy.
- J2->K1 inventory handoff; K2 stop scoped to its own invocation via InvocationID + NRestarts.

OPERATOR DECISION 2026-11-07 (chat, verbatim): "yes, i can run whatever you need now, or you can run it via my user on
briansrls" / "yes you have my approval for what you mentioned basically" -- in answer to: (A) approve J3 and run K1/K2 with
sudo, or let them run as briansrls; (B) prioritize the srv1 restore over finishing the full review cycle.

Verify: cd r2-packet-rev17 && sha256sum -c SHA256SUMS
Local fixtures (stubs and synthetic repos only): cd r2-packet-rev17/fixtures && bash run-all.sh
Harness sha256: 9726aeb1a8717049e6ceace2e5ae5a9996276d42eb977bd0a8aadb4393b1385b -- 139 cases, failures 0, exit 0 on this tree.
Inversions: fixtures/inversions-rev17.sh (one row per removed guard). Independently re-checked by hand on this tree:
k1-receipt-unwritable, k1-priv-symlink and k2-empty-invocation each go RED with their guard removed and GREEN with it present.
