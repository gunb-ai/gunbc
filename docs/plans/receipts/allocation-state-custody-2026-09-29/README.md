# Protected-state credential custody prerequisite

This cut extracts only the storage credential row from allocation PR #12505 onto
main `889312b928ae19593bb78e918bd2687c86e3db11`. It does not install the protected
storage server, designate a workspace slot, or deploy the allocation stack.

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

The branch-local release build passed under 6 GiB/no swap (15m10s). The initial
older-binary test and both branch-local full-root test/emission runs exited 137.
Byte-identical dependency-only runs also exited 137: 1746 modules for custody
controls, 2110 for the standard registry emitter. The latter reached some
transitive typechecking but neither produced a qualification result or workflow
output. No memory limit was increased.

PR #12580 remains draft and unqualified for installation. The superseded PR run
36510800728 was cancelled and supplies no qualification. The explicitly dispatched
integrated run 36512302394 is pinned to 7768a9597efb03bd6729cda76beb85b8e9485544
and remains outstanding at this checkpoint. The generated workflow option
must be emitted and checked before landing. These failures are a significant
local qualification blocker, not proof that the custody behavior passed or a
claim that any specific source defect caused the process kills.

Further bounded checks used the existing typed-module cache cap of one and
`MALLOC_ARENA_MAX=2`, matching the CI allocator setting. Both custody controls
(1746 original modules) and a direct call to the canonical fleet workflow
generator (1822 original modules plus a harness) ended with systemd
`Result=oom-kill`, before results. These were still 6 GiB/no swap; no further
identical local retry is planned. The integrated CI run uses its existing budget.

CI run 36512302394 passed floor, compiler, clippy and emit-build; generated
projection drift remained. PR run 36513776136 regenerated and verified the
workflow successfully. Its repair artifact intentionally excludes workflow files.
The emitted YAML was therefore read from that exact unchanged CI source checkout
over the modeled srv3 access path, checking HEAD before and after and requiring
an empty dag/src/v2 diff. Only the credential dropdown changes. The emitted bytes
and their provenance are now retained; the updated branch still needs its checks.
No credential installation was performed.
