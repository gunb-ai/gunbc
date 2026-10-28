# mtcollins1 stable SSH host key: mint, deliver, anchor (operator procedure)

Goal: stand up the STABLE per-boot SSH host key of **mtcollins1** (operator ruling 2026-10-22,
option A — mtcollins1 is diskless, `gunbc.runner.runner_host_medium` RAM image; booting it is
ruled **not sensitive**) so a live `HostBoundTransport` through
`gunbc.host.host_bound_transport host_bound_transport_of` can bind with its trust anchored on that
key, via `gunbc.host.anchored_host_transport anchored_host_transport_of`.

This runbook is the **apply-ready** procedure the **operator or merry-fox-82** runs — minting the
real key and writing it into Secret Manager is a privileged effect, and no agent session runs it.
The `.dag` side is the authority and ships in the same PR as this runbook:

| Fact | Authority |
|------|-----------|
| The secret (SecretRef) | `dag/gunbc/fleet/fleet_ssh_access.dag` `mtcollins1_ssh_host_key_secret` (project `fleet_secrets_gcp_project` = `gunbai-secrets`, `dag/gunbc/fleet/fleet_secrets_config.dag`) |
| Per-boot delivery (Q1: in memory on the host only, never disk across boot, never argv, never logged) | `dag/gunbc/fleet/host_credential_custody_converge.dag` arm `Mtcollins1SshHostKey` → `/run/gunbc-ssh/ssh_host_ed25519_key`, `CustodyOnlyOnHost mtcollins1`, root-only 0400, `CustodyTmpfsNoSwap` |
| Accessor IAM | `dag/gunbc/auth/fleet_secret_accessor_roster.dag` `mtcollins1_ssh_host_key_accessor_row` (read, for custody) and `mtcollins1_host_key_version_adder_row` (`secretVersionAdder`, for the mint) — a converge grant-set cell (`gunbc.auth.gcp_iam_converge`). **Do not run `gcloud secrets add-iam-policy-binding`**: the roster is the single authority and converge binds it. |
| The trust anchor + its standing | `dag/gunbc/host/anchored_host_transport.dag` `mtcollins1_host_key_anchor`, the ONE mint `anchored_host_trusted_row` |
| The gated readback scan | `dag/gunbc/host/anchored_host_transport.dag` `anchored_host_key_scan_wet` (refuses without `GUNBC_HOST_KEY_READBACK_GO`; the go is a `HumanIntervention` — merry-fox-82, relayed by snappy-koi-83) |
| The mint | `dag/gunbc/host/mtcollins1_ssh_host_key_mint.dag` — **a workflow dispatch, not a shell step** (operator ruling 2026-10-28, `gunbc.design_ruling the_mint_of_a_fleet_ssh_keypair_is_account_creation`). One keypair-mint policy, `dag/gunbc/host/fleet_keypair_mint.dag`, parameterized by the secret row; this key and the boot medium reader key are its two rows. Dispatch it as fleet-converge mode `FleetKeypairMint`; one dispatch walks the roster and each row's own pin decides whether an effect is reached. Censused at `dag/gunbc/auth/privileged_effect_census.dag` as a federated grant. The scaffold it retired (`anchored_host_key_mint_shell` / `_intervention` / `_effect`, and the human Step 1 that used to sit below) is recorded at the foot of `dag/gunbc/host/anchored_host_transport.dag`. |
| Evidence | `dag/test/claim/host/anchored_host_transport_witness.dag` (the anchor, the retirement, the runbook RED) + `dag/test/claim/host/fleet_keypair_mint_witness.dag` (the mint policy: the Q1 argv RED, the tmpfs RED, the census row) |

## Rule Q1 (non-negotiable)

The PRIVATE half exists only: in a create-only `0700` directory under the mint runner's `/dev/shm`
between generation and upload, in Secret Manager, and in mtcollins1's memory on each boot. It never
goes on disk across boot, never into argv, never into a log, never into a commit. The mint passes it
to Secret Manager via **request stdin** only, never as an argument.

It used to say "in the operator's shell between mint and upload". That clause left with the human
mint (operator ruling 2026-10-28; `gunbc.design_ruling the_mint_of_a_fleet_ssh_keypair_is_account_creation`),
and the Q1 window got **smaller**, not larger, when the human left: the mint runner's `/dev/shm` is
checked on a machine the fleet controls, on a directory the run created itself and removes on every
path out of the generation, and the run that created it reports — as a refusal that fails its step —
if it could not.

Two corollaries, enforced mechanically — now by the mint's own model rather than by a shell script
somebody pastes, which is a stronger form of the same rule:

- **The mint directory MUST be verified RAM-backed (tmpfs) before the key is created there.**
  `shred -u` is NOT erasure on SSD or copy-on-write media: the block it "overwrote" may live
  elsewhere (FTL remapping, CoW snapshots), so shredding a file that ever touched flash leaves the
  bytes recoverable. On a verified tmpfs the key never touches flash at all, and plain removal
  plus page reclaim is the honest erasure (the remaining window is RAM content until reboot, which
  is the same window as any shell variable — acceptable under Q1 because the alternative, writing
  to the SSD and shredding, is strictly worse).
- **Every failure path the run can see cleans the temp key.** The generation removes its directory on
  the success path AND on every refusal, and — the part a `trap` did not have to model, because a shell
  trap silently does nothing on `SIGKILL` — a run that created the directory and could not remove it turns
  that into a refusal naming the leftover directory, rather than a receipt that stays silent about it. Its
  honest limits are the trap's old limits unchanged: an uncatchable kill and a host crash run no code at
  all. On the verified tmpfs above those cases still leave nothing on flash — the key existed only in RAM,
  and a reboot (or `umount`/recreation of the tmpfs) clears it. The residual window is RAM contents on a
  machine that has not rebooted since, which is why this runbook ends with a rotation note: if a minting
  run is ever killed hard, treat the key as exposed and rotate the secret.
- **Nothing here can be run by an agent session, and the difference matters.** The mint's *model* is
  ordinary code; the mint's *dispatch* is an operator go. Retiring the human's shell removed a step, not
  the operator's judgment about when a fleet credential should come into existence.

## Step 1 — dispatch the mint (operator go; no agent session runs it)

This step used to be a fenced block of commands that a human pasted into a single interactive session:
the generation, the store-side container creation, the upload over a pipe, and the cleanup `trap` that was
the only thing standing between a crash and a key left in a temp directory. It is now a **workflow
dispatch**, and the RED that keeps it that way is
`test.claim.host.anchored_host_transport_witness the_runbook_mints_nothing_and_still_installs_the_anchor` —
it reads this file off disk and fails if a minting command ever comes back, so this prose cannot drift into
a recipe. The
operator's ruling (2026-10-28, relayed by merry-fox-82) is recorded verbatim as a design ruling,
`gunbc.design_ruling the_mint_of_a_fleet_ssh_keypair_is_account_creation`: "i don't see why a human needs to
do this - this seems just like normal account creation". The step that retired is the one the retired
scaffold's own `anchored_host_key_mint_shell_bash_emit_dissolution_trigger` named.

Dispatch the **fleet-converge** workflow with mode `FleetKeypairMint`. There is no per-key input to set: one
dispatch walks the keypair roster (`fleet_keypair_mint_subject_rows` in
`dag/gunbc/host/fleet_keypair_mint.dag`), and each row's step asks its own secret container whether the public
key is pinned — an unpinned row mints, a pinned row reports `host_key=already pinned` and exits zero without
reaching an effect. So this key's mint runs beside the boot medium reader key's (the other row), and neither is
disturbed by the other's state. One policy, two rows, is what the root ruling 6b requires — a second
implementation of "the private half never leaves tmpfs" is the copy that quietly drops the `findmnt` read.
(There is deliberately no `keypair_subject` dispatch input: the workflow's input roster already sits at
GitHub's hard limit of 25, and the pin decides what a dispatch does better than a free-text choice can.)

What the run does, and what it refuses to do:

- It reads this key's trust anchor (`mtcollins1_host_key_pinned_line`) **first**. If a key is already
  pinned, the run is a **no-op** and reports `host_key=already pinned` — which is what makes a re-dispatch
  after an unrelated failure safe rather than a way to accumulate versions.
- It creates its mint directory **create-only at 0700**, asks `findmnt` what filesystem it landed on, and
  **refuses before generating** unless the answer is `tmpfs`. Never assumed from the path string: a path
  beginning `/dev/shm` can be bind-mounted onto flash and would satisfy a name test while betraying Q1. An
  unanswered `findmnt` is a refusal too — a gate that passed on silence passes whenever the check is broken.
- It generates the pair into that directory, with an empty passphrase so nothing blocks on a prompt nobody
  can answer. **No argument carries key material**: the private half is what the generator *writes*, so the
  runner's process list shows a path and nothing else. The four commands a mint issues are enumerated word by
  word, for both key rows, by `dag/test/claim/host/fleet_keypair_mint_witness.dag` — which is the form this
  claim has to take, since the way it would actually rot is a later edit adding the material as an operand.
- It reads both halves and the `SHA256:` fingerprint, **removes the directory**, and only then judges. The
  removal is judged **first**: a pair generated into a directory that could not be removed is refused even
  though everything else succeeded, because reporting a usable keypair from a run that left it on disk would
  reward the one failure the mint cannot undo with a receipt saying `minted`.
- It adds the private half as a **version** of `mtcollins1-ssh-host-key` with the payload on **request
  stdin**, and establishes the secret's **container first** if it is absent — a precondition checked before
  the irreversible write, so a run never generates a key into tmpfs only to discover nobody can store it
  (`gunbc.recurring_failure_mode.a_precondition_check_sits_downstream_of_the_irreversible_effect`).
- It writes a receipt to `target/mtcollins1-ssh-host-key-mint.txt` naming **version, public key and
  fingerprint** — no arm of the receipt type has a field a private half could occupy, so "we never print the
  key" is a property of the type rather than a promise about its callers — and **exits non-zero unless the
  mint holds**. An outcome-UNKNOWN does **not** hold; see the note below.

Nothing in the run reads a stored key, so there is no code path from a private half to an output. The
grant the run needs (`secretVersionAdder` on this one row) arrives as
`mtcollins1_host_key_version_adder_row` through the accessor roster and the IAM converge; **nobody adds it
in the console**, and a hand-made grant would be unmodeled authority the census does not describe.

**If the step fails with `host_key=outcome unknown`.** The upload may have landed, and nothing in the run
can tell — the difference between `refused` and `unknown` is the difference between "retry" and "go look, a
credential may exist", which is why the mint keeps them as different arms. **A retry is safe and not
sufficient**: a retry mints a second version and nothing points at the first. The minting identity holds
`versions.access` on its own secret, so a human with that read lists the versions and compares each public
half against the intended pin. A match IS the pin to record — public row and fingerprint, in the same commit
as step 3.

**If the step fails with a container refusal.** The secret's container is established by the
operator-approved bootstrap/converge apply, not by this run's per-row grant: a minting identity with the
correct per-row `secretVersionAdder` and no project-level `secrets.create` reaches this arm. The remedy is
that approved apply. It is the approved apply, and never a hand-run store creation from a
terminal — and the run says so in its own refusal text, rather than leaving "the store refused" to send an
operator to look for a policy nobody wrote.

## Step 2 — pin the SecretRef version from the receipt

`mtcollins1_ssh_host_key_secret.version` ships as `"pending"` (HashPending) precisely because there is no
published version to pin until the mint lands. Update it, in a normal PR, to the version number the
**receipt** names (`version_name=...`) — following the pinning convention of the neighboring rows in
`dag/gunbc/auth/gcp_secret_accessor_roster.dag` / `gcp_secret_access` (a pinned version string is the guard
against silent rotation). Until this happens every custody read refuses at Secret Manager, which is the
honest state, not a failure to hide.

## Step 3 — transcribe the PUBLIC half into the anchor (this does NOT discharge the read obligation)

Take the two components (key type, base64 public blob) and the `SHA256:` fingerprint **from the mint's
receipt**, into the ONE mint `anchored_host_trusted_row` (admitted callers: `mtcollins1_host_key_anchor` and
the witness) — host, address, key type, public base64, fingerprint. **Both halves in the same commit as the
SecretRef pin above**: a minted key nobody records is a key that does not exist for any consumer, and a
minted private half with no recorded public is worse — a live credential in Secret Manager that nothing can
prove it holds, so no consumer would ever authenticate with it while its logs implied otherwise. This used to
be a human copying figures off a terminal before a `trap` deleted the file they came from; the receipt the
same run writes is where they come from now, and a hand-transcribed fingerprint is not a warning an operator
sees — it is a connection that refuses forever, or (worse, if the anchor is then "fixed" by copying what the
host presented) the very unanchoring the row exists to prevent.

Fabricating or guessing key material is forbidden (DESIGN §4d) — the components come from THIS receipt, and
the mint carries the standing (`TranscribedUncited`) and the read obligation as TYPED VALUES.

**WHAT THIS DOES NOT DO.** Transcribing the public half copies a figure across a boundary nothing in the DAG
read back; it lands the anchor with its standing and its obligation INTACT. It does not discharge the read
obligation — only step 4's live presented-key readback, matched line for line against this row, does that (as
the model says: `mtcollins1_host_key_read_obligation`, and the `TranscribedUncited` standing the mint
carries). A runbook reader who finished step 3 and believed the anchor verified would be the defect; the typed
standing is the guard, and it survives here.

**If a minting run was ever killed hard** (`SIGKILL`, host crash, power loss mid-run), no cleanup code ran.
On the verified tmpfs the key was only ever in RAM, so reboot that runner (or recreate the tmpfs) before
continuing, and treat the key as exposed — **rotate the secret** — if you cannot account for how it died.

## Step 4 — live boot, readback, match (needs merry-fox-82's go, relayed by snappy-koi-83)

1. Boot mtcollins1 (ruled not sensitive). The custody route delivers the secret to
   `/run/gunbc-ssh/ssh_host_ed25519_key` on the tmpfs; the boot route must start sshd pointing at
   exactly that HostKey (today the RAM image has no sshd in this slice — see
   `runner_host_first_boot_receipt` — which is frontier row 3 of
   `anchored_host_transport_frontier_rows`; closing this step dissolves it through its trigger).
2. Run the gated scan: `anchored_host_key_scan_wet(operator_host_mtcollins1)` with
   `GUNBC_HOST_KEY_READBACK_GO` exported by the named human (the `HumanIntervention` row
   `anchored_host_key_readback_go_step`). Its argv is `ssh-keyscan` only — the private half is
   never in argv, never rendered (witness-pinned; class
   `dag/gunbc/recurring_failure_mode/a_secret_crosses_into_shell_stdin_as_a_plain_string.dag`).
3. Match the presented `known_hosts_line` line-for-line against the anchored row's. A mismatch
   must REFUSE (witness-pinned RED); a match is the read that discharges the obligation.
4. Record the minted evidence **through the real dispatch** — the pairing obligation #14738 left
   open (`lost_boot_sweep_effect_pairing_frontier`): the read-to-admission route, not a supplied-
   text shortcut. The lost-boot sweep mode stays GATED until then.

## What refuses until each step lands

- Secret absent → custody read refuses at Secret Manager, naming the secret.
- Anchor absent (`"pending"`, no transcription) → any transport ROUTED through
  `anchored_host_transport_of` refuses at `AnchoredBindingAnchorAbsent`, cause carrying the read
  obligation and the secret name. (Production does not route through it yet — frontier rows 1–3.)
- Wrong presented key → `AnchoredBindingPresentedKeyMismatch` with expected+observed; the observed
  never equals the anchored material.
- Empty presented line → refuses before any cast.
