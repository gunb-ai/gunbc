# Reconciled server and parser-census qualification

At source `5c06dac72feec902896f22e19c7b571cc89d47d5`, the rebuilt compiler reached full roadmap preview readiness in 244.446 seconds under 12 GiB/no swap, with zero OOM events. The attached receipt pins source and binary identities. This was loopback qualification, not deployment.

Integrated floor run 36614665788 refused a stale `rfc_8118.dag#Optional` debt entry after the full-parser census merge. The follow-up explicitly imports Optional/Present/Absent from their authority and retires the Optional debt as ImportsFixed. The citation CIT1 suite passed all 22 claims using a byte-identical 77-module closure under 6 GiB/no swap and the rebuilt compiler. Full integrated qualification remains required.

Credential custody is established on srv1 (36605269073) and srv2 (36613639455), both executing the qualified b43704581 revision. The srv2 run uses #12646's remote-runner placement. Metadata readback establishes root custody and equality to the pinned Secret Manager version, not negative effective-open tests.

No VM reservation, boot, sanitation or reuse is claimed.
