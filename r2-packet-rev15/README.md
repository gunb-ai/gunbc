# r2 restoration packet, revision 15 (exact bytes for byte review). NOT a grant request; nothing staged or launched.

The ONE scoped remediation pass for sleek-owl msg_1b45cc1a / keen-otter msg_9669a3e4.
Verify: cd r2-packet-rev15 && sha256sum -c SHA256SUMS
Local fixtures (own-PID containment only; no global process discovery): cd r2-packet-rev15/fixtures && bash run-all.sh
Harness sha256 (fixtures/run-all.sh): 3ec31e9c9a42676316b2303c1dcf2a57bbc06489f5d917f307d3f4cda6f7cdf5 -- 85 cases, failures 0, exit 0 on this tree (GNU grep 3.11, mawk).
GATES.md: every capture/compare site (144 source sites + the frame excerpt), its status handling and its regression fixture.
FINDINGS.md: finding -> finder -> fix -> fixture; budget; open items; inversion proof.
