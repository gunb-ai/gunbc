# Qualification reconciliation, 2026-10-02

Main was merged through `a5e08e4dff9` at allocation head `ba4c65cec0a30badccd7ed69eef936d44c7376a2`.
The prior head's local startup and floor receipts are retained under names explicitly marking their older subject. They do not qualify the new head.

The required floor at ba4c65 refused before witness evaluation: `github.AppInstallationAccessTokens.Create` was classified both as a service call and as a reference to an unrelated `fn github` from a checkout witness. The service census contains `github.AppInstallationAccessTokens`, not the namespace prefix `github`.

The repair uses resolved service prefixes for each dotted chain. An independent bare call or an unresolved member access still requires its own provider. The service provider itself still requires an import. No debt-roster exemption or witness rename is used.

No host installation, commissioning, readiness publication, reservation, guest boot, SSH, release, or reuse is established by these receipts.

The closure-admission group passes 17 tests, with the explicitly ignored live-corpus differential scheduled separately. The original group run exposed a stale two-pool fixture; the test now retains its cross-root bypass assertion, drops the first owned index, and resets the test-only cache before the clean counterpart. The production second-pool refusal remains tested and unchanged.

At 1296fb5f713, all 621 live-corpus sources compared successfully, the native fixture passed all ten cases, and the complete site reached loopback readiness in 236.074 seconds within 12 GiB/no swap. CI then correctly found a missing live-tree import after the cursor witnesses gained their first explicit imports. All three affected witness files now name the live-tree authority; their four assertions pass under 6 GiB/no swap. Full-repository batch admission subsequently passed with all four assertions and exit 0 at 01a43aa013a. Its OverAttributed cost diagnostic is preserved in the receipt and supports no cost conclusion. Exact-head required CI remains a separate gate.
