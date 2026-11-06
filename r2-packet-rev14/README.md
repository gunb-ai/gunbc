# r2 restoration packet, revision 14 (exact bytes for byte review). NOT a grant request; nothing staged or launched.

DOCUMENTATION-ONLY revision: the 8 job scripts, J3-PLAN.txt and the harness are byte-identical to rev13 (cmp), so rev13's
harness run (62 cases, 0 failures) and inversion proof apply unchanged. Changed: APPLY-EFFECTS.md (Y-SP line-by-line trace of
#2-#21 and #34-#45) and FINDINGS.md (Y1 traced; calm-crab's launcher-environment attestation in E1).
Verify: cd r2-packet-rev14 && sha256sum -c SHA256SUMS
