# Namecheap credential custody

The owner reports `projects/582015116396/secrets/namecheap-api-key` provisioned.
The project number matches `fleet_secrets_project_number`; the project ID comes
from `fleet_secrets_gcp_project` (`gunbai-secrets`). No new version is provisioned
by this lane, and no key is requested in chat or in a persistent local file.

The owner also supplied account `briansrls` and an API allowlist entry named `dev`
for `96.224.201.7`. These are typed operator reports in
`gunbc.namecheap.account`, not evidence of an authenticated API call or a
permanent DNS-controller placement.

`gunbc.namecheap.credential` binds the container once. Its `latest` reference is
for the container-scoped IAM binding only. Execution requires an observed
numeric version, so an approval over one DNS plan cannot silently switch to a
new key through an alias. `gunbc.namecheap.credential_read` uses the existing
`fetch_secret_ref_credential` with `WorkloadIdentityToken`; the shared reader
checks the response resource against the requested project, secret and version.
It does not print or persist the payload.

The dedicated `namecheap-dns` federation joins `gcp_iam_converge_targets`. Its
trust is pinned by the shared fleet job claim authority to this repository, the
fleet-converge workflow at main, a dispatch on main, and the `namecheap-dns`
environment. Its only secret grant is accessor on `namecheap-api-key`. The
existing IAM workflow plans as iam-observe, files a request in the approval app,
impersonates iam-converge only after approval, rechecks the approved plan, applies
and independently reads back. Enrollment is not evidence of an applied grant.
The new target also changes the resource-local bootstrap bindings needed by the
IAM controller; those must be observed before running its apply.
This preserves the repo's distinction between routine per-secret federated
reads and operation-specific human approvals. For DNS mutations, the consumer
must use the existing `approval_gate` and scoped authorization over the frozen
zone plan; secret possession alone never authorizes a zone replacement. This
increment does not yet connect that DNS mutation consumer.

Live status: secret creation and the allowlist are operator-reported. No enabled
version or applied IAM binding has been independently observed. This local shell
has no WIF token or gcloud installation. No provider request, IAM write, key
access, or DNS mutation has been executed. A read-only CI mode that supplies WIF
and records version metadata remains to be connected before live verification.
The earlier DNS model lives on `work/daily-end-to-end`; this lane is based on the
current repository so it can reuse the landed approval app rather than copying
its newer authorities into that older checkout. The previous local `api_key_file`
onboarding proposal is superseded by this Secret Manager reference.

Validation: all 21 `gcp_iam_converge_witness_test` claims passed with the full
`dag` and `src/v2` roots, including the new dedicated Namecheap target and the
existing approval, expiry, stale-policy and controller-trust controls. All five
`namecheap_credential_witness_test` claims passed. These are model checks, not
live GCP or Namecheap observations. The current repository binary was required;
the older stage0 experiment binary cannot parse this base's newer declarations.
