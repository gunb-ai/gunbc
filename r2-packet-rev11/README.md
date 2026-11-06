# r2 restoration packet, revision 11 (exact bytes for byte review). NOT a grant request; nothing staged or launched.

Packet-only branch: not code, never merged. Verify: cd r2-packet-rev11 && sha256sum -c SHA256SUMS
Local fixtures (own-PID containment only; no global process discovery): cd r2-packet-rev11/fixtures && bash run-all.sh
Harness sha256 (fixtures/run-all.sh): c504914224381b177f56317cde04d351c2f041f71ba6e87beb1cc1cceeb3e52a -- 44 cases, failures 0, exit 0 on this tree (GNU grep 3.11).
Findings ledger (finding -> finder -> fix -> fixture; open items; inversion proof): FINDINGS.md
