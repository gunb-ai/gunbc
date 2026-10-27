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
| Accessor IAM | `dag/gunbc/auth/fleet_secret_accessor_roster.dag` `mtcollins1_ssh_host_key_accessor_row` — a converge grant-set cell (`gunbc.auth.gcp_iam_converge`). **Do not run `gcloud secrets add-iam-policy-binding`**: the roster is the single authority and converge binds it. |
| The trust anchor + its standing | `dag/gunbc/host/anchored_host_transport.dag` `mtcollins1_host_key_anchor`, the ONE mint `anchored_host_trusted_row` |
| The gated readback scan | `dag/gunbc/host/anchored_host_transport.dag` `anchored_host_key_scan_wet` (refuses without `GUNBC_HOST_KEY_READBACK_GO`; the go is a `HumanIntervention` — merry-fox-82, relayed by snappy-koi-83) |
| The mint (review 85232) | `dag/gunbc/host/anchored_host_transport.dag` `anchored_host_key_mint_shell` (the Step 1 block below IS its render) + `anchored_host_key_mint_intervention` + `anchored_host_key_mint_effect`, censused in `dag/gunbc/auth/privileged_effect_census.dag`; scaffold trigger `anchored_host_key_mint_shell_bash_emit_dissolution_trigger` |
| Evidence | `dag/test/claim/host/anchored_host_transport_witness.dag` (16 claims) |

## Rule Q1 (non-negotiable)

The PRIVATE half exists only: in the operator's shell between mint and upload, in Secret Manager,
and in mtcollins1's memory on each boot. It never goes on disk across boot, never into argv, never
into a log, never into a commit. Every command below passes it via **stdin pipe**, never as an
argument.

Two corollaries this runbook enforces mechanically, not by trust:

- **The mint directory MUST be verified RAM-backed (tmpfs) before the key is created there.**
  `shred -u` is NOT erasure on SSD or copy-on-write media: the block it "overwrote" may live
  elsewhere (FTL remapping, CoW snapshots), so shredding a file that ever touched flash leaves the
  bytes recoverable. On a verified tmpfs the key never touches flash at all, and plain removal
  plus page reclaim is the honest erasure (the remaining window is RAM content until reboot, which
  is the same window as any shell variable — acceptable under Q1 because the alternative, writing
  to the SSD and shredding, is strictly worse).
- **Every failure path the shell can see cleans the temp key.** A single shell `trap ... EXIT` covers
  every exit the shell controls — error, Ctrl-C (SIGINT), termination (SIGTERM), and success. Its honest
  limits: a `SIGKILL` (or any uncatchable kill) and a host crash or power loss mid-session run no shell
  code at all, so no trap fires. On the verified tmpfs above, those cases still leave nothing on flash —
  the key existed only in RAM, and a reboot (or `umount`/recreation of the tmpfs) clears it. The residual
  window is RAM contents on a machine that has not rebooted since, which is why the runbook ends with a
  rotation note: if a minting session is ever killed hard, treat the key as exposed and rotate the secret.

## Step 1 — mint and upload (operator / merry-fox-82, one shell session)

**This block is the model, checked byte-for-byte: `test.claim.host.anchored_host_transport_witness`
`the_runbook_mint_block_is_the_model_render` reads this file off disk and pins the fenced block below
to a fresh `gunbc.host.anchored_host_transport` `anchored_host_key_mint_shell()` render — do not
hand-edit it, and do not run a remembered copy of it. Drift between this block and the model fails the
suite, not just review.** Every command line inside it is
produced by an admitted argv builder (`extdeps.ssh.openssh_client_commands.ssh_keygen_generate_command`,
`extdeps.tools.gcloud.gcloud_secrets_create_command`, `extdeps.tools.gcloud.gcloud_secrets_versions_add_stdin_command`,
`extdeps.tools.gnu_coreutils` `mkdir_create_only_with_mode_command` / `cat_file_command` /
`rm_force_command` / `rmdir_command`, `extdeps.tools.util_linux_findmnt.findmnt_fstype_for_path_command`);
the secret name and project come from `mtcollins1_ssh_host_key_secret` (`dag/gunbc/fleet/fleet_ssh_access.dag`)
and its `fleet_secrets_gcp_project` project field, never from strings written here. The model is the
authority — this block was produced by rendering it, and the claims in
`test.claim.host.anchored_host_transport_witness` (`the_mint_effect_is_censused_and_diverges_with_reason`,
`the_mint_render_carries_no_private_material`) pin the render's legs. The block's out-of-band human
actuation is a declared scaffold — see `anchored_host_key_mint_shell_bash_emit_dissolution_trigger`
(dissolves to a real dispatch when an execution route can carry it) — and its privileged effect is
censused: `anchored_host_key_mint_intervention` / `anchored_host_key_mint_effect`
(`dag/gunbc/auth/privileged_effect_census.dag`).

```bash
set -e
'trap' 'rm -f /dev/shm/mc-hostkey/mc_host; rm -f /dev/shm/mc-hostkey/mc_host.pub; rmdir /dev/shm/mc-hostkey' 'EXIT' 'INT' 'TERM'
'mkdir' '-m' '0700' '--' '/dev/shm/mc-hostkey'
if '[' $('findmnt' '-n' '-o' 'FSTYPE' '-T' '/dev/shm/mc-hostkey') '!=' 'tmpfs' ']'; then 'echo' 'mint dir is not tmpfs-backed; refusing to mint' >&2; exit 1; fi
'ssh-keygen' '-q' '-t' 'ed25519' '-N' '' '-C' 'mtcollins1' '-f' '/dev/shm/mc-hostkey/mc_host'
'gcloud' 'secrets' 'create' 'mtcollins1-ssh-host-key' '--project' 'gunbai-secrets'
'cat' '--' '/dev/shm/mc-hostkey/mc_host' | 'gcloud' 'secrets' 'versions' 'add' 'mtcollins1-ssh-host-key' '--project' 'gunbai-secrets' '--data-file=-'
```

Reading the render, leg by leg (the model's `@agent_brief` on each builder is the authority; these
are pointer notes, not a restatement of the commands):

- The **trap is one line for every exit the shell can see** (error, Ctrl-C/SIGINT, SIGTERM, and
  success). Its honest limits — SIGKILL and host crash run no shell code — are in the honesty note
  above; on the verified tmpfs those cases still leave nothing on flash. `mkdir` is **create-only**:
  if `/dev/shm/mc-hostkey` already exists a previous minting session died mid-run; the trap removes
  the stale key and the run refuses. Treat that key as exposed and rotate (rotation note, bottom).
- The **tmpfs check must print `tmpfs`** or the run refuses before any key exists.
- `ssh-keygen` writes the pair into the 0700 dir; the `-N ''` is an empty passphrase, and no
  argument anywhere carries key material.
- The **create leg carries no `--data-file` at all**: creating with `--data-file=/dev/null` would
  publish an EMPTY version 1 that a reader resolving `latest` (or the pin in step 2) could pick up
  as key material. Without data the secret exists with zero versions; the real key becomes version 1.
  (Older gcloud may prompt for a replication policy here; answer `automatic` — it does not prompt
  on the current SDK.)
- The **upload leg is `--data-file=-`**: the private half reaches the server only over the pipe.
  The only word that names the key is `cat`'s path argument — a path, not contents.
- Record the printed version number for step 2 (expect 1).

## Step 2 — pin the SecretRef

`mtcollins1_ssh_host_key_secret.version` ships as `"pending"` (HashPending) precisely because
there is no published version to pin until the mint lands. Update it, in a normal PR, to the
version number step 1 printed — following the pinning convention of the neighboring rows in
`dag/gunbc/auth/gcp_secret_accessor_roster.dag` / `gcp_secret_access` (a pinned version string is
the guard against silent rotation). Until this happens every custody read refuses at Secret
Manager, which is the honest state, not a failure to hide.

## Step 3 — transcribe the PUBLIC half into the anchor (this does NOT discharge the read obligation)

Still in the same shell, while `/dev/shm/mc-hostkey` still exists — order matters: the public half is
transcribed BEFORE the trap removes it (removing first leaves step 3 nothing to read):

```bash
cat "/dev/shm/mc-hostkey/mc_host.pub"    # "ssh-ed25519 <base64> mtcollins1"
```

Take the two components (key type, base64 public blob) into the ONE mint,
`anchored_host_trusted_row` (admitted callers: `mtcollins1_host_key_anchor` and the witness) —
host, address, key type, public base64, and the `SHA256:` fingerprint computed from the real key
(`ssh-keygen -lf /dev/shm/mc-hostkey/mc_host.pub` prints it). The mint carries the standing
(`TranscribedUncited`) and the read obligation as TYPED VALUES. Fabricating or guessing key
material is forbidden (DESIGN §4d) — the components come from THIS transcription.

**WHAT THIS DOES NOT DO.** Transcribing the public half copies a figure across a boundary nothing
in the DAG read back; it lands the anchor with its standing and its obligation INTACT. It does not
discharge the read obligation — only step 4's live presented-key readback, matched line for line
against this row, does that (as the model says: `mtcollins1_host_key_read_obligation`, and the
`TranscribedUncited` standing the mint carries). A runbook reader who finished step 3 and believed
the anchor verified would be the defect; the typed standing is the guard, and it survives here.

The trap from step 1 removes both halves on every exit the shell can see (including an error here), so
there is no separate shred step, and on a verified tmpfs plain removal is the honest erasure (shred is
theater on SSD/CoW and pointless on RAM). If the session is killed with `SIGKILL`, or the host goes down
mid-session, no trap runs: the key was only ever in RAM, so reboot that machine (or recreate the tmpfs)
before continuing, and treat the key as exposed — rotate the secret — if you cannot account for how it
died.

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
