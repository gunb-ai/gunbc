# r2 restoration packet, revision 13 (exact bytes for byte review). NOT a grant request; nothing staged or launched.

Packet-only branch: not code, never merged. Verify: cd r2-packet-rev13 && sha256sum -c SHA256SUMS
Local fixtures (own-PID containment only; no global process discovery): cd r2-packet-rev13/fixtures && bash run-all.sh
Harness sha256 (fixtures/run-all.sh): 738255de59f784b436092e8e72e9d547650dfb52e96b2b5bfbbcc67bca0718fa -- 62 cases, failures 0, exit 0 on this tree (GNU grep 3.11).
Findings ledger (finding -> finder -> fix -> fixture; full J1 budget; open items; inversion proof): FINDINGS.md
