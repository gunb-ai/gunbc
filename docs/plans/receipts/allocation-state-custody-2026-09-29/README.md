# Protected-state credential custody prerequisite

This cut extracts only the storage credential row from allocation PR #12505 onto
main `889312b928ae19593bb78e918bd2687c86e3db11`. It does not install the protected
storage server, designate a workspace slot, or deploy the held allocation stack.

The existing host custody convergence resolves the pinned Secret Manager version,
checks the administrator SSH subject, writes through the existing typed remote
file authority, and reads back content, permissions, ancestors and staging cleanup.
The state credential uses root:operator 0640 inside root:operator 0750, outside job
and guest trees. Permission-bit readback is not an ACL or effective-open proof.
The cloud accessor remains a row in the existing secret-accessor roster.

After review and qualification, the bounded custody sequence is the existing
`fleet-converge` mode `host_credential_custody_converge`, `credential=fabric_state_writer_key`,
once for srv1 (storage/controller) and once for srv2 (the website). Use the merged
reviewed source revision and the workflow's own WIF identity. No operator token,
manual SCP, or fallback approval key is required. Read each run's custody receipt
before advancing to storage service commissioning. Those runs have not occurred.

Validation is pending. The initial test attempt with the older bootstrap binary
was killed with exit 137 under MemoryMax=6G / MemorySwapMax=0 before producing
results. A clean build of this branch's own binary is in progress. This is not a
passing witness or deployment receipt.
