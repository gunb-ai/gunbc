# BMC keyless GCP bootstrap

**Goal:** let the GitHub runner acquire the assimilator's token through OIDC. Identity and federation belong to `gunbc.assimilate.bmc_token_federation`; API requirements and provisioning order belong to `gunbc.assimilate.bmc_bootstrap_provision`.

**Method:** an authorized operator supplies the initial project-admin `gcloud` login, then runs the modeled bootstrap:

```sh
gcloud auth login
gunbc run --source-root dag \
  --entry dag/gunbc/assimilate/bmc_bootstrap_provision.dag \
  --function bmc_bootstrap_provision_srv3
```

The bootstrap creates identity/federation, not secret IAM. Only `bmc_wif_signed_grants` in `gunbc.assimilate.bmc_wif_delegation_chain` reach `gunbc.auth.gcp_iam_converge`, after its bootstrap delta and approval. Unsigned grants require a keep-or-retire decision; do not bind them with a separate manual secret-policy write.

The runner needs `gcloud`, `contents: read` and `id-token: write`. Generate its auth step from `gcp_wif_auth_step`; `emit_bmc_token_smoke_workflow_yaml` supplies the smoke workflow. Preserve both repository-scoped provider admission and impersonation membership. Off-runner access follows `gunbc.auth.access_token_source` selection rather than copying a key-creation recipe.

**Validation:** `test.claim.bmc.bmc_token_federation_witness` and `test.claim.bmc.bmc_bootstrap_provision_witness` cover modeled boundaries; independently read the resulting signed grants and verify keyless token acquisition on the intended runner. A provisioning refusal names its step and does not prove an attempted write failed to commit; inspect that state before retrying, including service-account propagation delays.
