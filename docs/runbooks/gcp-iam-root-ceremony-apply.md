# GCP IAM root ceremony — `iam_bootstrap_apply` (operator procedure)

**Why you are here.** A fleet-converge run in `mode=gcp_iam_converge` died at job
`gcp-iam-bootstrap-request` with `ROOT CEREMONY CELLS MISSING` and the rest of the run was cancelled.
That refusal is the model refusing to mint an `iam-bootstrap` token while cells that a *trust-root
ceremony* owns are unestablished (`gunbc.auth.gcp_iam_bootstrap_delta_run` folds
`gunbc.auth.gcp_iam_bootstrap_delta_route bootstrap_delta_requestable`; the text is `root_cell_missing_text`,
and every missing cell names its repairing ceremony). For the cells named by run 40251866941 — the
`github-gcp-iam-converge` / `github-gcp-iam-bootstrap` pool roles (`gunbcIamObservePool`), the
`iam-observe` service account (`gunbcIamObserveAccount`), and the `gunbai-secrets-bootstrap-journal`
bucket roles (`gunbcIamBootstrapJournal`, `gunbcIamObserveBucket`) — the named repair is one entry:
`gunbc.auth.gcp_iam_bootstrap` `iam_bootstrap_apply`. This file is that entry's procedure.

Read it as a gate order, not a checklist to skim: **every** refusal in this route is a stop, and none of
them is fixed by a different command.

## What this ceremony is, and what it is not

`iam_bootstrap_apply` (`gunbc.auth.gcp_iam_bootstrap`) is the operator-authorized first-time creation of
the estate's WIF trust substrate: the two workload-identity pools, the `iam-bootstrap` and `iam-observe`
service accounts, the journal bucket, the custom roles, the deny boundary, the temporary authority cells,
and the federation/workflow trust — the twelve stages of `data iam_bootstrap_stages`, folded in order.

It is **not** the accessor grant you may actually be chasing. `approval-apns-auth-key` is not a trust root
(`gunbc.auth.gcp_iam_trust_roots` `ceremony_trust_root_secrets` lists the approval MAC keys, the store
receipt key and the gunbai-ci App key only), so `gunbc.auth.gcp_iam_bootstrap_model`
`iam_bootstrap_cell_owner` answers `DeltaOwned` for its accessor cell: no ceremony binds it. It is a
grant-set cell — `gunbc.auth.gcp_iam_converge` `fleet_roster_grants` folds every non-root roster row into
`gcp_iam_converge_grant_sets`, `plan_grant_set` derives the desired cells, and the converge's own
approved-effect pass writes them. You run this ceremony to unblock that converge, because in
`mode=gcp_iam_converge` the converge job `needs` `gcp-iam-bootstrap-request` and `gcp-iam-bootstrap-apply`;
while a root cell is missing the first refuses and the converge never runs. You never run this ceremony *to
bind* a roster accessor cell, and there is no per-key operator-token entry to run instead — the fold that
did that was deleted under the operator's 2026-10-18 ruling, "for ALL keys" (foot of
`gunbc.auth.fleet_secret_accessor_roster`). Hand-writing `gcloud` IAM commands is not a route either: an
ad-hoc binding has no journal entry, so the next converge reads it as drift and the refusal returns.

## Prerequisites

1. **Build the compiler binary at the commit you are standing on.** The `.dag` sources are the program;
   there is no `gunbc` release channel — you build it.
   ```bash
   cargo build --release -p v1-compiler --bin gunbc
   ```
2. **A token from your own `gcloud` sign-in, kept off the command line.** The entry reads exactly one
   variable, `GUNBC_GCP_ACCESS_TOKEN_FILE` (`gunbc.auth.access_token_source` `secret_access_token_path_env`,
   `read_supplied_access_token`), and refuses when it is absent, set-but-empty, or unreadable —
   `classify_supplied_token` never falls back to ambient ADC or to the interactive `GcloudPrintToken` arm
   (that arm is deliberately unreachable from the fold, so a run can never block on a prompt it cannot
   answer). It does **not** check the file's mode or the token's expiry, so the 0600 and the freshness are
   yours to keep — a stale token surfaces as a refused stage, not as a stop at the door. Never paste a
   token into a command line, a shell variable, or an Actions secret.
   ```bash
   gcloud auth application-default print-access-token > /tmp/gunbc-iam-ceremony-token
   chmod 0600 /tmp/gunbc-iam-ceremony-token
   ```
3. **A fresh run identity.** `iam_bootstrap_apply` refuses without `GUNBC_IAM_BOOTSTRAP_RECEIPT`, and it
   writes `target/gcp-iam-bootstrap-<receipt>-started.txt` with `WriteCreateNew` — the same receipt twice
   refuses ("bootstrap receipt already exists"). Make it unique per attempt.
4. **The exclusive pool-window confirmation, when a ceremony-owned pool has no policy yet.** A pool whose
   policy read answers no etag (an empty policy) is initialized only under
   `GUNBC_IAM_BOOTSTRAP_POOL_WINDOW`, which must equal `data iam_bootstrap_pool_window` in
   `gunbc.auth.gcp_iam_bootstrap`. Without it the post-run proof reports the pool cell as *"pool policy
   publication prerequisite unmet: successful read has no concurrency token"*. On 2026-10-28
   `github-gcp-iam-bootstrap` was such a pool, so srv1's estate needs it; set it only while nothing else
   writes those pools' policies.
5. **The temporary deny-administration authority pair.** `GUNBC_IAM_BOOTSTRAP_AUTHORITY_ID` /
   `GUNBC_IAM_BOOTSTRAP_AUTHORITY_EXPIRES_AT` are required by the deny stage
   (`iam_bootstrap_authority_converge`), with the same shape rules as the probe pair below; without them
   the deny stage refuses *"dependency unsatisfied: temporary deny administration needs an authority
   operation ID and absolute expiry"*.
   `GUNBC_IAM_BOOTSTRAP_PROBE_ID` / `GUNBC_IAM_BOOTSTRAP_PROBE_EXPIRES_AT` **are required** — the pair,
   together, not optionally: `iam_bootstrap_probe_authority_converge` (granting arm) refuses without a
   pinned probe identity and expiry ("probe token authority requires a pinned operation identity and
   absolute expiry"), and the probe-authority stage is in the folded stage list, so the whole ceremony
   refuses. Set both: `GUNBC_IAM_BOOTSTRAP_PROBE_ID` is an operation tag safe as a path segment (≤ 48
   characters) and `GUNBC_IAM_BOOTSTRAP_PROBE_EXPIRES_AT` is a canonical UTC instant
   (`YYYY-MM-DDTHH:MM:SSZ`, exactly that shape — `bootstrap_authority_intent_valid` refuses anything
   else) strictly in the future and at most one hour ahead. An absent *or half-supplied* pair refuses on
   the granting arm; only the two withdrawal arms (`BootstrapProbeCleanup`, end-of-run) no-op without it.

## Step 1 — plan first (refuses, effects nothing)

`iam_bootstrap_plan` is a pure read that prints the stage order and the boundary/role/cell population, so
you can compare against the refusal before touching GCP. It takes no token and creates nothing.

```bash
target/release/gunbc run --source-root dag --source-root src/v2 \
  --entry dag/gunbc/auth/gcp_iam_bootstrap.dag --function iam_bootstrap_plan
```

Confirm the plan's cell list covers what run 40251866941 reported missing. If a name in that refusal is in
neither list, stop: you are being told two different models, and applying under one of them is guessing.

## Step 2 — apply the ceremony

```bash
GUNBC_IAM_BOOTSTRAP_RECEIPT=iam-ceremony-$(date -u +%Y%m%dT%H%M%SZ) \
GUNBC_GCP_ACCESS_TOKEN_FILE=/tmp/gunbc-iam-ceremony-token \
  target/release/gunbc run --source-root dag --source-root src/v2 \
    --entry dag/gunbc/auth/gcp_iam_bootstrap.dag --function iam_bootstrap_apply
```

Exit 0 means every stage admitted, applied and read back **and** the temporary authority cells withdrew.
The fold stops at the first stage that refuses; per-stage receipts land in
`target/gcp-iam-bootstrap-<receipt>-<stage>.txt`, so a refusal tells you which stage and the file tells you
why. Two refusals you should expect to see at least once and understand rather than work around:

- *"bootstrap requires a safe receipt identity and an admitted model"* — the receipt is not a safe path
  segment, or `iam_bootstrap_model_admitted` is false. The model is the estate's model; a false here is a
  repo state, not a cloud state. Do not proceed.
- *"stage receipt failed; inspect cloud state before another attempt"* — a stage's cloud effect may have
  landed without its local receipt. Re-running blindly can double-apply. Read the stage back in the cloud
  first.

## Step 3 — read back, then let the delta finish the job

Re-run the converge in `mode=gcp_iam_converge`. `gcp-iam-bootstrap-request` is the check: it stops
refusing once no root-ceremony-owned cell is missing, mints the `iam-bootstrap` token, and the
approval-gated delta binds the roster cells — including `approval-apns-auth-key`'s reader cell for
`serviceAccount:fleet-secret-accessor@gunbai-secrets.iam.gserviceaccount.com`, which is the grant srv1's
`gunbc.fleet.host_credential_custody_converge` (arm `ApprovalApnsAuthKey`) is waiting on. The APNs push
stage A (#15124) activates on srv1 at that read, with no srv1-side actuation in this procedure.

The delta still needs its own approval. If it now refuses on something other than root cells, that is a
different gate and a different node — stop and report it rather than widening anything to get past it.

## Cleanup

Shred the token file; it is a live credential until it expires.

```bash
rm -f /tmp/gunbc-iam-ceremony-token
```

`target/gcp-iam-bootstrap-<receipt>-*.txt` are your receipts — keep them with the run they belong to.
