# r2 restoration packet, revision 10 (exact bytes for byte review). NOT a grant request; nothing staged or launched.

Packet-only branch: not code, never merged. Verify: cd r2-packet-rev10 && sha256sum -c SHA256SUMS
Local fixtures (own-PID containment only; no global process discovery): cd r2-packet-rev10/fixtures && bash run-all.sh
Harness sha256 (fixtures/run-all.sh): 27c92789cad82d627bcc5ed15ea82a7c78404fc243bd9848562eb3b3e7bd8b75 -- 38 cases, failures 0, exit 0 on this tree.
Findings ledger (finding -> finder -> fix -> fixture, plus open items and the inversion proof): FINDINGS.md
