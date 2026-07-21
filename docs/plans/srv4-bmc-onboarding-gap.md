# srv4 BMC onboarding — what the workflow is missing

Status: gap analysis for review (branch `session/srv4-onboarding-credential-rotation-gap`, 2026-07-21).
Trigger: srv4 hardware racked; its BMC is at `FactoryDefault` and the credential-rotation step
that the workflow *should* perform was skipped, leaving a manual gate. This documents why, so we
close the workflow gap rather than hand-rotate.

## Live srv4 facts (measured)

- BMC reachable at **`192.168.1.195`** and `192.168.1.199` (two NICs, one BMC).
  MACs `48:21:0b:87:2c:ad` / `…:df` (newer block than srv1–3's `…:81:*`).
- Redfish UUID `83819837-25c6-4881-9b0c-49de5e8c0c58` (distinct board, OpenBMC).
- Factory login `root/0penBmc` authenticates (session `201`) but **every resource returns
  `403 PasswordChangeRequired`** — OpenBMC's factory gate. Nothing (identity, virtual media,
  install) works until root's password is rotated.
- Onboarding stage per `gunbc.bmc_onboarding`: **`FactoryDefault`** (host OS not yet installed).

## What already exists (the machinery is real)

The rotation flow is fully modeled and fail-closed — this is *not* a from-scratch build:

- `gunbc.tools.bmc_onboard.bmc_converge_credential_idempotent` (`dag/gunbc/tools/bmc_onboard.dag`)
  probes phase → if `FactoryCredentialActive`, mints a 20-byte urandom credential,
  `gcp.SecretManager.AddVersion` + read-back durability check (refuses to rotate on mismatch, so a
  failed store can't lock us out), then `redfish.Http.SetAccountPassword` (PATCH
  `/redfish/v1/AccountService/Accounts/root`, which also clears `PasswordChangeRequired`) and
  verifies reauth. Idempotent: `RotatedCredentialActive` → `ExitSuccess`.
- `redfish.Http.SetAccountPassword` effect exists (`dag/extdeps/bmc/http.dag:95`).
- Lifecycle state machine `FactoryDefault → CredentialsRotated → OsInstalled → FabricJoined` with
  monotone credential-posture hardening (`dag/gunbc/bmc_onboarding.dag`).
- Wired into the standup spine: `host_standup_spine`'s first prefix step `BmcCredentialConverge`
  decl-refs `bmc_converge_credential_idempotent` (`dag/gunbc/host_standup.dag:193-199`).
- GCP IAM for the secret is modeled: assimilator SA with `secret_version_adder` + `secret_accessor`
  bindings (`dag/gunbc/assimilate/bmc_token_federation.dag`, `…/bmc_wif_delegation_chain.dag`).

## The gaps (why srv4 stalled)

### G1 — the whole chain is srv3-hardcoded, not per-host parameterized  *(dispositive)*
Everything bottoms out on one global `new_altra_onboarding_plan` (`bmc_onboarding.dag:213`):
`bmc: 192.168.1.192`, `rotated_credential: Stored { secret_name: "bmc-srv3-admin" }`.
- Every `bmc_onboard.dag` func reads that global + `srv3_gcp_project`; `onboarding_secret_id()`
  fallback is literally `"bmc-srv3-admin"`; error strings say "would lock out srv3".
- `host_standup.dag` imports `operator_host_srv3`, references `srv3_install_mechanism`, and
  `host_standup_spine` is a static `data` list, not a function of `HostIdentity`.
- There is **no way to point the converge at `.195` / `bmc-srv4-admin`** — no srv4 plan, no host
  parameter. This is the §2-horizontal / §3-single-authority violation: "BMC onboarding" is one
  concept that should span every host by breadth, forked to srv3 literals.
Fix shape: lift `new_altra_onboarding_plan` to `altra_onboarding_plan(host)` (or a per-host data
row keyed by `HostIdentity`); thread `plan` (or `host`) as a param through the `bmc_onboard.dag`
funcs and make `host_standup_spine` a function of `HostIdentity`.

### G2 — gcloud is assumed-present, not provisioned  *(the "upsert gcloud cli" ask)*
The credential step calls `shell.GCloud.AuthPrintAccessToken()` + `gcp.SecretManager.*`, which need
the `gcloud` CLI on the actuator host. But:
- `gcloud` is **not** modeled as a `CliTool` in `extdeps/tools/` (only curl/websocat/nbdkit/socat/
  sha256sum/xorriso).
- `ActuatorToolchainRequirement` (`extdeps/bmc/os_install_actuator_toolchain.dag`) ensures only the
  OS-install transport tools — gcloud is absent, and that requirement isn't scoped to the credential
  step anyway.
- srv1 (candidate actuator) has no gcloud CLI installed.
Fix shape: model `gcloud` as an ensured `CliTool` and add it to a toolchain requirement covering the
credential/secret actuation (mirrors the existing os-install toolchain-ensure).

### G3 — token acquisition is hardwired to one handler  *(lets the operator token work)*
Getting the GCP access token is hardwired to `shell.GCloud.AuthPrintAccessToken()`. But the
SecretManager effects already take `access_token` as an explicit input
(`secret_manager.dag:114` AddVersion, `:71` AccessVersion). So "obtain access_token" is really one
interface with N handlers — {gcloud-CLI print-token | **operator-supplied token** | WIF federation}
— currently fused to the gcloud-CLI handler (a §3 transport fork). De-forking it lets the operator's
supplied token drive the rotation with no interactive gcloud auth (and G2's gcloud upsert becomes
the other handler, not a hard prerequisite).

### G4 — no srv4 network identity  *(mechanical)*
`fleet_intent_network.dag` has srv1/2/3 endpoints but no srv4. Add BMC `.195` (+`.199`), secret
`bmc-srv4-admin`, host endpoint TBD-post-install.

## Proposed sequence (for review — not yet implemented)

1. G4: add srv4 identity rows.
2. G1: parameterize the onboarding plan by host; thread through `bmc_onboard` + `host_standup`.
3. G3: de-fork token acquisition so an operator-supplied token is a first-class handler.
4. G2: model `gcloud` as an ensured tool for the workflow to self-provision (defense-in-depth /
   the other token handler).
5. Run `bmc_converge_credential_idempotent` against srv4 with the operator token → `CredentialsRotated`,
   secret in `bmc-srv4-admin` → then the existing OS-install spine.

Open question for the operator: scope now — do all of G1–G4, or minimally G1+G3 (parameterize +
operator-token handler) to unblock srv4 today and land G2 (gcloud self-provision) as follow-up?

## Outcome (2026-07-21) — srv4 rotated + two gaps found by execution

Full close G1–G4 landed and verified (850-module typecheck clean + 11 witnesses green).
srv4's BMC was then rotated **live** with an operator-supplied token — the first real
execution of this flow (srv3's rotation was never run live; its debt marker said so). Two
gaps surfaced only because we executed (§5 "done = green by execution"):

- **Credential format (FIXED).** `mint_bmc_credential` used `Urandom.ReadBytes(...).octets_b64`
  — base64, whose `=`/`/`/`+` OpenBMC rejects with `PropertyValueFormatError` (HTTP 400) on the
  password PATCH. The fail-closed read-back gate correctly aborted before rotating (no lockout).
  Then a *firmware-policy divergence* surfaced: srv4 (newer OpenBMC, UUID 83819837) accepted a
  16-char **alphanumeric** password, but srv3 (OpenBMC **2.07.00**, UUID 6df9b4b1) rejected it —
  its `pam_pwquality` needs a 4th character class (special). Policy read from srv3's AccountService:
  MinPasswordLength 9, MaxPasswordLength 20. Fixed universally: added `Urandom.ReadPassword`, a
  composition-guaranteed generator (≥1 upper/lower/digit/special from the shell/JSON/basic-auth-safe
  set `_.@#%-`), and `mint_bmc_credential` mints a **16-char** such password (within 9–20, satisfies
  both firmwares). Live-verified: srv4 rotated with a 16-char alnum credential (before the divergence
  was known); srv3 rotated with a 16-char 4-class credential.
- **Secret-container creation (FOLLOW-UP).** `bmc_store_and_rotate`'s `AddVersion` assumes the
  secret exists; `bmc-srv4-admin` did not (404) because the GCP bootstrap
  (`gunbc.assimilate.bmc_bootstrap_provision`, still srv3-hardcoded) never ran for srv4. Created
  it manually this session. Correct fix: parameterize `bmc_bootstrap_provision` per-host (same move
  as G1) so it creates `bmc-<host>-admin` + IAM before onboarding. A blind create-if-missing inside
  the rotation path is unsafe — the gcp effect consumption style aborts on non-200, so a 409
  (already-exists) on re-run would break idempotency; the bootstrap is the right home.

State now: **both srv3 and srv4 CredentialsRotated.** srv4 → Secret Manager `bmc-srv4-admin` v2
(v1, rejected base64, destroyed). srv3 → `bmc-srv3-admin` v6 (v1–v5, prior base64/alnum attempts
that populated the secret but never rotated the BMC — the "secret exists, server never rotated"
state — destroyed). Both: factory `0penBmc` → 401, stored credential → 200. Next lifecycle step is
OsInstalled (the existing seeded-autoinstall spine); the rotation persists across OS installs since
the BMC is out-of-band.
