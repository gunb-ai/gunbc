# Fabric-state credential separation

At `71aae3c54`, both approval controls pass under 6 GiB/no swap using the
branch-local binary recorded in receipt.json. The 590-module source copy matches
repository bytes; sources.json records all identities. The production provisioning
and shared grant modules load/typecheck in this closure, but tests invoke only
request construction and approval gates. No cloud call, live approval, credential
read or host installation was performed.

The cross-operation control requires each decision to admit its own request and
refuse the other operation in both directions. The run-attempt control originally
failed because the material intent hash omitted its attempt. The preserved failure
log is the discriminating negative specimen; hashing the attempt makes it pass.
This is a gate-level control, not a reproduced live broker cross-run exploit.

The first two source-copy attempts failed on omitted legacy implicit dependencies.
The final copy adds the real modules gunbc.bmc_onboarding, gunbc.host_converge,
gunbc.host_effect, gunbc.os_install_deduction, gunbc.network_identity_subsumption
and gunbc.srv3_os_install_diagnostic. No dependency was stubbed or rewritten.

Reproduce the copy with tools/tests/dag_validation_closure.py, naming the witness
module test.claim.auth.fabric_state_provision_witness_test and those six modules,
then run its copied entry with --claim-run under MemoryMax=6G/MemorySwapMax=0.
This is not full landing-suite qualification or live credential commissioning.
