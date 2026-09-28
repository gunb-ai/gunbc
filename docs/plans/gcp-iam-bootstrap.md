# Commission the approval-gated GCP IAM workflow

The existing `gcp_iam_converge` fleet workflow needs its own cloud identities
before it can request approval and configure other workflows. This bootstrap
commissions that foundation from the existing IAM declarations. It does not
depend on the workspace allocation branch.

## Entry points

`gunbc.auth.gcp_iam_bootstrap.iam_bootstrap_plan` prints the exact roles,
permissions, bindings, deny permissions, and workflow admission condition without
cloud effects. `iam_bootstrap_apply` uses an explicitly supplied operator token
file and a fresh receipt identity:

```bash
export GUNBC_GCP_ACCESS_TOKEN_FILE=/path/to/private/operator-token
export GUNBC_IAM_BOOTSTRAP_RECEIPT=operator-bootstrap-unique-attempt
systemd-run --user --scope -p MemoryMax=6G -p MemorySwapMax=0 --quiet \
  ./target/release/gunbc run --source-root target/iam-bootstrap-controls \
  --entry target/iam-bootstrap-controls/gunbc.auth.gcp_iam_bootstrap.dag \
  --function iam_bootstrap_apply
```

The token file must be private and short-lived. No token belongs in a command
argument, source file, receipt, or GitHub variable. The ongoing workflow never
falls back to this operator token. No service-account key is created.

The operator needs existing authority to create/read the declared identities,
custom roles, and deny policy, update the specified resource policies, and
impersonate the observer/apply accounts for readback. GCP's
[Deny Admin documentation](https://docs.cloud.google.com/iam/docs/deny-access)
names `roles/iam.denyAdmin` for managing deny policies. Bootstrap does not grant
administrative permissions to its operator to overcome a refusal.

## What bootstrap owns

1. Create or exactly read back the dedicated IAM workflow pool/provider and its
   observer/apply accounts. Create only bare pools and accounts for declared
   target federations, so resource-local administrative roles can be attached.
2. Create or exactly read back the existing four apply role definitions and
   three resource-specific observer roles. Existing drift refuses adoption.
3. Install and read back the declared project deny policy for the apply account.
4. Add declared capability bindings with policy etags, preserving foreign cells.
5. Check access using short-lived impersonated observer/apply credentials. The
   operator must already have impersonation authority; bootstrap does not grant
   that authority to itself. Observer reads must work, and the apply account must
   receive HTTP 403 reading a credential that the observer just successfully read.
6. Only after these checks, add workload-identity impersonation bindings for the
   existing, pinned IAM workflow.

Target providers, workload impersonation, and target secret-access grants remain
in the existing approval-gated convergence plan. Bare target containers do not
activate those workflows. The target roster comes from `gcp_iam_converge_targets`,
including its current private-publisher and Namecheap entries; it is not copied
from the older allocation branch.

Permission probes are diagnostics, not an authorization oracle. Configuration
readback and the real observer/apply secret-read control are separate checks.
Successful bootstrap still requires a real GitHub OIDC/approval/apply run before
the ongoing workflow can be called operational.

## Recovery and evidence

Each invocation creates `target/gcp-iam-bootstrap-<receipt>-started.txt` before
effects, then a separate receipt for every attempted stage. A refusal stops
later stages. A process loss or asynchronous creation may require a new attempt
identity; it rereads named resources and their current policies. A started
receipt is not completion evidence. Existing policy drift never authorizes
automatic privilege expansion.

The existing `fleet_converge_workflow.dag` job already performs observer WIF,
request/approval, apply WIF, live approval recheck, and independent readback.
Bootstrap supplies its missing cloud prerequisites rather than adding another
approval workflow or a pasted-token path to secret consumers.

## Validation

Build the branch's own release binary under the same 6 GiB/no-swap scope. The
closure helper copies sources byte for byte and records their hashes:

```bash
python3 tools/tests/dag_validation_closure.py --output target/iam-bootstrap-controls \
  test.claim.auth.gcp_iam_bootstrap_witness_test \
  gunbc.bmc_onboarding gunbc.host_converge gunbc.host_effect \
  gunbc.os_install_deduction gunbc.network_identity_subsumption \
  gunbc.srv3_os_install_diagnostic
systemd-run --user --scope -p MemoryMax=6G -p MemorySwapMax=0 --quiet \
  ./target/release/gunbc run --source-root target/iam-bootstrap-controls \
  --entry target/iam-bootstrap-controls/test.claim.auth.gcp_iam_bootstrap_witness_test.dag \
  --claim-run
```

The additional modules resolve existing admitted-caller references. This focused
closure is not the full repository landing suite. Qualification must also record
the binary hash, source revision, pure plan, missing-credential refusal, cloud
stage receipts, and a real approval-workflow run when available.

API contracts follow Google Cloud's official references for
[custom roles](https://docs.cloud.google.com/iam/docs/reference/rest/v1/projects.roles/create),
[project policy updates](https://docs.cloud.google.com/resource-manager/reference/rest/v1/projects/setIamPolicy),
[workload identity pools](https://docs.cloud.google.com/iam/docs/reference/rest/v1/projects.locations.workloadIdentityPools),
and [deny policy creation](https://docs.cloud.google.com/iam/docs/reference/rest/v2/policies/createPolicy).

## Current commissioning standing

The 2026-09-28 run completed identity and exact custom-role readback, then GCP
refused deny-policy creation with HTTP 403, `iam.denypolicies.create` missing.
Capability bindings, account-context probes, and workflow-trust grants were not
reached. See `receipts/gcp-iam-bootstrap-2026-09-28/README.md`. Resume requires an
authorized administrator credential with the missing authority; successful source
tests are not evidence that the approval workflow is operational.
