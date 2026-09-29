# Broker and custody authority integration

Source `a84a925916d15f9dea366a36fd543294cc6b3aac` reconciles main at
`6571276ce43de56e414bcccfa9623129c7c4e2b4`, including the merged fixed CI
broker unit. Both brokers consume the shared slot-purpose sanction. Eleven real
broker controls and five direct-controller controls pass under 6 GiB/no swap.
The new control requires a purpose refusal before reservation; storage failure
cannot satisfy it. Existing CI recovery remains owed after purpose withdrawal.

The full 856-source preview server reaches readiness in 261.351 seconds, peaking
at 12,575,674,368 bytes within 12 GiB/no swap, with zero OOM events. The helper
stops its temporary loopback server. This is not a deployment.

CI 36606943712 found the missing `v2.std.algebra.any` import in the floor-demand
witness. All 29 controls pass after that explicit import, over 59 unchanged closure
modules. The follow-up also makes that provider explicit in four other touched
witnesses that use it. Those additional witnesses still require integrated CI.
The generated lane remains outstanding; no integrated green result is claimed.

The srv1 census establishes an absent CI broker unit and two controller receipts
refusing before reservation. It does not establish complete history or sanitation.
Live credential custody is recorded separately in the sibling receipt directory.
No VM was reserved or launched by this qualification.
