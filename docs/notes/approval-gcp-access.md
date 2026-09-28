# Approval app and GCP access

`gunbc.auth.approved_gcp_access` joins the existing signed approval broker protocol
with an approval-gated workload credential. It files the actual AuthorizationRequest
and, at apply time, re-reads the broker and checks the re-derived request revision,
run attempt, scope and validity window before reading the credential variable.
Pending, unreadable, denied, expired and different-intent decisions cannot yield
access. A missing workload credential remains a separate refusal after approval.
It does not turn an approval into Google IAM permission or use an operator token-file
fallback. The reviewed workload still must mint its scoped WIF credential after the
approval step; the utility rechecks approval immediately before consuming it.

The existing GCP IAM apply entry now consumes this shared join. Fabric state
provisioning uses the same utility, with a SHA-256-bound request describing the
exact secret, initial version, accessor principal and role. The apply entry derives
that request again from current policy and the actual run attempt. It reuses the
existing secret creation/initial-version/readback actuator and accessor grant ensure.
No token material is placed in the request, purpose, CLI output or receipt.

Prepared entries:
- `request_fabric_state_key`: file the bounded request through the existing app.
- `provision_fabric_state_key`: re-admit the decision and perform the approved ensure.

Neither entry has run. The state-provision workload's scoped credential producer
and permissions remain to be connected and verified; merely naming
`FABRIC_STATE_PROVISION_ACCESS_TOKEN` is not that producer. Host installation is a
separate existing credential-custody convergence effect. Do not invoke an actuator
with a broad manually supplied credential to bypass this remaining boundary.

The user reports broker #12444 is installed and running from its branch on srv1,
but still compiling at startup. Its owning agent is measuring readiness and fixing
startup allowance. Enrollment and live approval tests wait for the broker to answer.
No broker installation or restart was performed by this allocation slice.

## Shared secret-access rollout

The user authorized applying this flow to secret workflows generally, not only the
allocation credential. The shared `approved_secret_grants` adapter now offers
request/apply entries for the fleet accessor roster, Spark access, and the
organization GitHub App key access. It hashes the exact secret, beneficiary, role,
condition and run attempt, files through the same broker, and rechecks approval
before each grant. Apply uses the existing policy reconciler and readback. These
are source adapters, not a deployed credential producer. The explicit legacy
bootstrap entries remain available until the replacement workflow is operational.

The existing `gcp_iam_converge` workflow already has the right approval-before-WIF
shape and resource-local secret-policy authority. Its `iam-converge` identity is
explicitly denied secret payload reads and version addition. Do not reuse it to
mint or retrieve secret values, and do not widen that identity to make the adapter
work. Its admitted target population currently contains dedicated federations,
not the whole fleet accessor roster. Connecting these adapters requires enrolling
the actual secret-policy targets and binding the reviewed request/poll/apply steps;
simply exporting `SECRET_GRANT_ACCESS_TOKEN` is not that integration.

Remaining operator-shaped paths found in the source inventory include printer
credential migration/delivery, BMC onboarding/credential access, R2 secret custody,
Spark credential workflows, and approval-key bootstrap. Reads already served by
WIF should retain their existing admitted workload identity rather than acquiring
new approval prompts. Creating the approval broker's initial keys cannot depend
on that same broker already being operational. Non-secret cloud workflows are
outside this initial rollout. No claim that every path has migrated is made here.

Validation note: the shared grant adapter's first focused run caught run-attempt
omission from its intent digest; the digest now includes the attempt as well as
the exact grant batch. A broader inclusion of the 36 existing secret-policy
witnesses failed dependency resolution on implicit host-standup references before
running claims. That is not a regression-suite pass. The final focused rerun is
recorded separately in `/tmp/shared-secret-approval-focused.log` with copied-source
hashes in `target/shared-secret-approval-focused/sources.json`.

The common no-credential diagnostic now directs secret workflows to the exact
operation's approval/convergence path and explicitly avoids requesting a pasted
GCP login code or bearer token. This changes the recovery instruction, not the
available credential sources or the production rollout status.

Final focused result: 3/3 PASS, wrapper exit 0, 6 GiB memory bound and zero swap.
These cover exact grant/beneficiary/role/run binding, empty-plan refusal and retry
identity separation. The earlier shared approval utility's three admission tests
passed separately; they were not re-executed by this entry. Source adapters and
common diagnostic changes remain uncommitted in the isolated integration tree.


## Explicit operator bootstrap, 2026-09-28

The user subsequently supplied a temporary operator token and explicitly authorized
using it to complete initial setup. The dedicated
`fabric_storage_state_bootstrap` entries were added for this authorization; normal
app-gated entries do not fall back to that credential. The material and accessor
steps completed separately, with real version and policy readbacks. Receipts are
in `../plans/receipts/fabric-state-bootstrap-2026-09-28/`.

This supersedes the earlier statement that neither provisioning entry had run only
for the explicit bootstrap entries. The normal workload credential flow remains
uncommissioned. The material approval now describes only key material and hashes
the run attempt; access policy has its own request via the shared grant authority.

The permanent IAM bootstrap is isolated on `codex/gcp-iam-bootstrap`, based on main,
so it can be reviewed independently of allocation HOLD. Live metadata showed the
two modeled IAM accounts absent. Creating the storage key did not create them or
make the approval workflow operational.
