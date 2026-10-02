# Qualification input reduction

No production code or memory ceiling changes. This checkpoint preserves the 87 fleet assertions while separating the workflow-specific entry from the fleet plan entry. The moved test and helper are byte-for-byte unchanged. Both entries remain ordinary claim witnesses with `SubstrateInputsOnly` standing.

Twenty-one large JSON evidence manifests were compacted by formatting only. Every decoded value equals its predecessor at `6f09038cc63`; `manifest-compaction.json` records before/after sizes. This removes 584,931 bytes without losing source identities, hashes or evidence. It addresses the observed CI unified-diff output refusal without truncating the diff, narrowing the baseline or raising the transport limit. The complete staged diff against the local merge base measured 8,079,999 bytes before adding this small receipt, below the 8,388,608-byte cap. CI must independently confirm its own resolved baseline.

Local tests use the existing compiler SHA-256 `717e9b4159dc7bc722e94a235325e6760f5acd246e63d8bed14b828dac970243`, unmodified source snapshots and 12 GiB/no swap. This is not an exact-head compiler build or deployment evidence.

The fleet entry passed all 86 assertions; demanded modules fell from 1,985 to 1,148, with reported pre-entry peak RSS 9,149,268 KiB. The remaining unchanged workflow assertion is qualified separately.

The separate workflow entry passed its unchanged assertion under the same 12 GiB/no-swap scope (1,985 demanded modules; reported pre-entry peak RSS 12,159,328 KiB). Both processes exited successfully. The complete original population is therefore **86 + 1 = 87 passing tests**, with no omitted names, changed assertion bodies or raised limits.
