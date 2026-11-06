# r2 restoration packet, revision 12 (exact bytes for byte review). NOT a grant request; nothing staged or launched.

Packet-only branch: not code, never merged. Verify: cd r2-packet-rev12 && sha256sum -c SHA256SUMS
Local fixtures (own-PID containment only; no global process discovery): cd r2-packet-rev12/fixtures && bash run-all.sh
Harness sha256 (fixtures/run-all.sh): a124d28d421d89ff6afc6dbf4beeeca7d9b02ab7d0fee6bac0948a3fdd6b090f -- 51 cases, failures 0, exit 0 on this tree (GNU grep 3.11).
Findings ledger (finding -> finder -> fix -> fixture; budget table; open items; inversion proof): FINDINGS.md
