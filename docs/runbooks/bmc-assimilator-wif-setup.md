# BMC assimilator: keyless GCP token via Workload Identity Federation

Goal: make the `.dag` effect `shell.GCloud.AuthPrintAccessToken()` (and the Secret
Manager ops in `dsl/extdeps/cloud/gcp/secret_manager.dag`) resolve on the **self-hosted
GitHub Actions runner with no human pasting a token** — the keyless path for unattended
BMC assimilate jobs.

The `.dag` does **not** change to get a token; token acquisition is host/CI config behind
that effect. This runbook is the **apply-ready** infrastructure the operator runs once with
GCP-admin credentials. The agent does **not** have admin creds and does **not** apply these.

## Single authority

The identity facts and the IAM bindings below are modeled in
`dsl/gunbc/assimilate/bmc_token_federation.dag` — that file is the authority, this runbook is
apply guidance derived from it. Verified by execution in
`dsl/test/claim/bmc_token_federation_witness_test.dag` (least-privilege + keyless emission).

| Fact | Value |
|------|-------|
| Project id | `gunbai-secrets` |
| Project number | `582015116396` |
| Secret (only one) | `bmc-srv3-admin` |
| Service account | `bmc-assimilator@gunbai-secrets.iam.gserviceaccount.com` |
| WIF pool | `github-actions` |
| WIF provider | `github-oidc` |
| Federated repo | `gunb-ai/gunbc` |
| OIDC issuer | `https://token.actions.githubusercontent.com` |

Blast radius is **one secret, two roles**. Nothing project-wide, no admin.

## One-command bootstrap (.dag-automated — preferred)

The entire bootstrap below (enable APIs → create SA → resource-level IAM binding →
create WIF pool → create WIF provider) is modeled as a single `.dag` orchestration over the
proven v1 REST executor — the same path Secret Manager runs on. The **only** manual input is
one initial **admin** access token; the orchestration acquires it via the operator's existing
`gcloud` login (`shell.GCloud.AuthPrintAccessToken()` — the "one button"), so with an admin
`gcloud` session active there is **nothing to paste**:

```bash
# one admin gcloud login (interactive, once) — the single human step
gcloud auth login          # as a project owner/admin of gunbai-secrets

# then the whole bootstrap, .dag-driven (idempotent; safe to re-run):
gunbc run --source-root dsl \
  --entry dsl/gunbc/assimilate/bmc_bootstrap_provision.dag \
  --function bmc_bootstrap_provision_srv3
```

The API set is **not** hand-typed: `bmc_required_gcp_apis` is *derived* from the
`GcpService` dependencies the assimilate path declares (`bmc_assimilate_service_deps`) via
`gcp_service_api_id` — enablement is a dependency of usage. Known first-run race: a freshly
created service account takes a few seconds to propagate before the IAM binding can reference
it, so a cold run can fail at the binding step with *"service account does not exist"* — just
**re-run** the command once and it converges (the SA already exists). Proven live against
`gunbai-secrets` on 2026-06-24 (all five ops dispatched; SA minted, least-priv bound, WIF
provider `ACTIVE`).

Authority: `dsl/gunbc/assimilate/bmc_bootstrap_provision.dag` (sequencing) +
`dsl/gunbc/assimilate/bmc_token_federation.dag` (identity facts). Least-privilege and the
closed API set are verified by execution in
`dsl/test/claim/bmc_bootstrap_provision_witness_test.dag`. The manual `gcloud` blocks in
§0–§3 below are the **documented equivalent / fallback** — run them only if you prefer to
apply by hand or are debugging a step.

## 0. Prerequisites (operator)

```bash
gcloud config set project gunbai-secrets
gcloud services enable iam.googleapis.com iamcredentials.googleapis.com \
  sts.googleapis.com secretmanager.googleapis.com
```

The self-hosted runner must have the `gcloud` CLI installed: the
`google-github-actions/auth` step calls `gcloud auth login --cred-file=...` automatically when
gcloud is on `PATH`, which is what makes `gcloud auth print-access-token` resolve keylessly.

## 1. Create the dedicated service account

```bash
gcloud iam service-accounts create bmc-assimilator \
  --project=gunbai-secrets \
  --display-name="BMC assimilator (keyless, srv3 secret only)"
```

## 2. Resource-level IAM on the single secret (least privilege)

Grant **only** on `bmc-srv3-admin`, **only** these two predefined roles. One command per
modeled binding row (`bmc_assimilator_bindings`):

```bash
# roles/secretmanager.secretAccessor — read the current rotated BMC admin secret
gcloud secrets add-iam-policy-binding bmc-srv3-admin \
  --project=gunbai-secrets \
  --member="serviceAccount:bmc-assimilator@gunbai-secrets.iam.gserviceaccount.com" \
  --role="roles/secretmanager.secretAccessor"

# roles/secretmanager.secretVersionAdder — write the post-rotation secret version
gcloud secrets add-iam-policy-binding bmc-srv3-admin \
  --project=gunbai-secrets \
  --member="serviceAccount:bmc-assimilator@gunbai-secrets.iam.gserviceaccount.com" \
  --role="roles/secretmanager.secretVersionAdder"
```

Do **not** grant `roles/secretmanager.admin`, project-level Secret Manager roles, or any
`owner`/`editor`. The witness test fails closed if a broad role appears in the model.

## 3. Create the Workload Identity Federation pool + provider

```bash
gcloud iam workload-identity-pools create github-actions \
  --project=gunbai-secrets --location=global \
  --display-name="GitHub Actions OIDC"

gcloud iam workload-identity-pools providers create-oidc github-oidc \
  --project=gunbai-secrets --location=global \
  --workload-identity-pool=github-actions \
  --issuer-uri="https://token.actions.githubusercontent.com" \
  --attribute-mapping="google.subject=assertion.sub,attribute.repository=assertion.repository,attribute.ref=assertion.ref" \
  --attribute-condition="assertion.repository == 'gunb-ai/gunbc'"
```

The `attribute-condition` pins federation to **this repo only** — an OIDC token from any other
repo is rejected before it can impersonate the SA.

## 4. Let the federated repo identity impersonate the SA

```bash
gcloud iam service-accounts add-iam-policy-binding \
  bmc-assimilator@gunbai-secrets.iam.gserviceaccount.com \
  --project=gunbai-secrets \
  --role="roles/iam.workloadIdentityUser" \
  --member="principalSet://iam.googleapis.com/projects/582015116396/locations/global/workloadIdentityPools/github-actions/attribute.repository/gunb-ai/gunbc"
```

This is the **only** identity that may mint a token for the SA, and it is scoped to the repo.

## 5. The GitHub Actions auth step (keyless)

Modeled in `dsl/gunbc/assimilate/bmc_token_federation.dag` (`gcp_wif_auth_step`,
`bmc_token_smoke_workflow`). A consuming assimilate workflow needs, at the workflow or job
level:

```yaml
permissions:
  contents: read
  id-token: write        # lets the runner request the GitHub OIDC token
```

and the auth step before any GCP effect:

```yaml
- name: Authenticate to GCP (keyless, Workload Identity Federation)
  id: gcp_auth
  uses: google-github-actions/auth@v2
  with:
    workload_identity_provider: projects/582015116396/locations/global/workloadIdentityPools/github-actions/providers/github-oidc
    service_account: bmc-assimilator@gunbai-secrets.iam.gserviceaccount.com
```

No `credentials_json`, no key file, no pasted token. Regenerate the full smoke workflow YAML
from the authority instead of hand-copying:

```bash
gunbc run --source-root dsl \
  --entry dsl/gunbc/assimilate/bmc_token_federation.dag \
  --function emit_bmc_token_smoke_workflow_yaml
```

After this step, `gcloud auth print-access-token` (the `.dag` `AuthPrintAccessToken()` effect)
resolves on the runner. GitHub OIDC tokens are issued by GitHub, not the host, so this works on
**self-hosted** runners (srv1/srv2/srv3) exactly as on hosted ones.

## 6. Local / manual fallback (scoped SA key)

For local or operator-driven runs off a GitHub runner, WIF is unavailable. Use a **scoped**
key for the same least-privilege SA — never a project-wide or user credential:

```bash
# Operator mints a short-lived scoped key (store 0600, never commit, rotate/delete after use)
gcloud iam service-accounts keys create /tmp/bmc-assimilator.json \
  --iam-account=bmc-assimilator@gunbai-secrets.iam.gserviceaccount.com

chmod 0600 /tmp/bmc-assimilator.json
gcloud auth activate-service-account \
  bmc-assimilator@gunbai-secrets.iam.gserviceaccount.com \
  --key-file=/tmp/bmc-assimilator.json

# now gcloud auth print-access-token works locally; clean up when done
gcloud auth print-access-token >/dev/null && echo "scoped token OK"
shred -u /tmp/bmc-assimilator.json 2>/dev/null || rm -f /tmp/bmc-assimilator.json
```

Prefer WIF; the key fallback exists only because a laptop cannot present a GitHub OIDC token.
A leaked key still only touches one secret with two roles.

## 7. Verify least privilege (operator, by inspection)

```bash
# Exactly the two secretmanager roles on exactly the one secret:
gcloud secrets get-iam-policy bmc-srv3-admin --project=gunbai-secrets \
  --flatten="bindings[].members" \
  --filter="bindings.members:bmc-assimilator@gunbai-secrets.iam.gserviceaccount.com" \
  --format="value(bindings.role)"
# Expect ONLY:
#   roles/secretmanager.secretAccessor
#   roles/secretmanager.secretVersionAdder

# The SA holds NO project-level roles:
gcloud projects get-iam-policy gunbai-secrets \
  --flatten="bindings[].members" \
  --filter="bindings.members:bmc-assimilator@gunbai-secrets.iam.gserviceaccount.com" \
  --format="value(bindings.role)"
# Expect: (empty)
```

## 8. Security notes

- A token pasted into any transcript or log is **spent** — rotate/discard it. The whole point
  of WIF is that no long-lived token or key exists to leak.
- `google-github-actions/auth@v2` is pinned to the major tag here for consistency with the
  other actions in `extdeps/github/actions.dag`. For stricter supply-chain posture, pin to a
  full commit SHA (update `google_auth_action.ref`) and re-emit.
- The `attribute-condition` (step 3) and the `principalSet` repo scope (step 4) are the two
  walls that stop another repo's OIDC token from reaching this SA. Keep both.

## Related model files

- `dsl/gunbc/assimilate/bmc_token_federation.dag` — identity facts, IAM bindings, auth step (authority)
- `dsl/extdeps/cloud/gcp/iam.dag` — `GcpRole` / `common_roles` (secretAccessor + secretVersionAdder)
- `dsl/extdeps/cloud/gcp/secret_manager.dag` — Secret Manager ops behind the token
- `dsl/extdeps/cloud/gcp/gcp.dag` — `WifPool` / `WifProvider` / `AuthPrintAccessToken`
- `dsl/extdeps/github/actions.dag` — `WorkflowPermissions.id_token`, `google_auth_action`
- `dsl/test/claim/bmc_token_federation_witness_test.dag` — least-privilege + keyless witness
