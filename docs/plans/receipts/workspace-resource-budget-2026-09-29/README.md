# Resource-budget selection qualification

The operator stated on 2026-09-29 that cost measurements are not required now.
Selection source: `3a2cc27322e28480e7263493cc9aeca39f6726f3`.
Final source: `97cd02e5f` additionally preserves legacy acquisition identity.

The shared selector now has an explicit request-bound resource-budget policy.
Workspace preparation consumes it, retaining provider/use and capacity-class
admission, actual resource fit, observed lease availability, the exclusive slot
hold and existing account/retirement protocol. Paid offers are refused. Unknown
costs remain unknown; zero user tariff is not a physical-cost measurement.
The durable plan carries the selection policy fingerprint, including its authority,
revision, request and allowed offers. Legacy plans have an explicit legacy marker;
malformed policy fields refuse rather than becoming legacy plans.

Under 6 GiB with swap disabled, 52 selection controls and 7 final launch/plan controls
passed. This includes seven new resource-budget controls and three new durable-plan
controls. The final codec closure contains 647 byte-identical modules. Legacy
plans retain their original serialized bytes because acquisition actors use the
plan document hash as their identity; adding a field to legacy serialization
would incorrectly change that identity. The selection closure manifest records its exact 733-module snapshot. Two final
source changes after copying that snapshot changed explanatory comments only.
The initial wrong entry-path invocation and the initial negative fixture failure
are retained. That fixture accidentally supplied the same demand identity twice;
the corrected test supplies a distinct demand and passes.

No production deployment, commissioning, reservation, guest boot or external SSH
acceptance is established by these tests. Production request preparation,
commissioning and maintained access observations remain unfinished.

Full server startup on the final clean source also passed: all 851 sources
resolved and the preview listener became ready in 303.361 seconds.
The included receipt pins the source and branch-local binary and records the
unchanged 12 GiB/no-swap test limit, peak memory and zero OOM events. The helper
stopped its temporary listener; it did not send requests or deploy the site.
