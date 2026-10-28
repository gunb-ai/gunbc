# Fleet keypair-mint lane — working notes

Task: one keypair-mint fleet-converge mode, parameterized by the secret row, so the boot medium reader key
and mtcollins1's SSH host key are both rows of it. Retires the human scaffold in
`docs/runbooks/mtcollins1-ssh-host-key-mint.md` (its
`anchored_host_key_mint_shell_bash_emit_dissolution_trigger` names exactly this).

Operator ruling (2026-10-28, relayed by merry-fox-82), recorded as a `DesignRuling` row:

> i don't see why a human needs to do this - this seems just like normal account creation

## The language constraint this design obeys

Verified locally with an existing interpreter binary (`gunbc run --entry <module> --function <fn>`, and
`--claim-run` for a witness module).

`type X = CoreType` **shares the core's arm names**. Naming an arm derived from the alias
(`ReaderMinted` over `type ReaderReceipt = CoreReceipt`) **typechecks and then fails at runtime** with
`PatternMatchFailure`. So coproduct types may not be delegated by alias: the alias is transparent to the
checker and opaque to the matcher, which is the worst combination available.

The design therefore shares **policy**, not vocabulary:

- `gunbc.host.fleet_keypair_mint` holds the verdicts as **data over shared types** (`HostCommandOutcome`,
  `String`, `Bool`) plus the pure validators, the tmpfs gate, the redaction, and the add-over-stdin.
- Each caller keeps its **own** coproduct arms and projects the core's verdicts into them, in one small
  function per coproduct. The strings move into the core verbatim, so a caller's witnesses assert what they
  always asserted.

That is also the honesty test for the generalization: `boot_medium_reader_key_mint_witness` passes against
the reader key's original names, so the second caller arrived by generalization rather than by replacement.

## Q1, preserved verbatim

The private half exists in exactly two places: a 0700 create-only directory under the mint job runner's
`/dev/shm` (tmpfs, so RAM-backed), and Secret Manager, added by `AddVersion` with the payload base64'd onto
request stdin. Never argv, never a log, never a durable path. The filesystem type is **read** from `findmnt`
and required to be `tmpfs` before anything generates — a path that merely begins `/dev/shm` could be a bind
mount onto disk, which a string test would pass and the rule would not.

The mint job runs on a **GitHub-hosted** VM, which is a safety property and not a convenience: the VM is
discarded with the job, so even a run killed before its cleanup leaves no copy on a fleet runner that other
workloads execute on. The directory is removed by the generation on every path out, and a run that created it
and could not remove it reports that as a refusal naming the leftover — the case a shell `trap` passed
silently, because a trap runs no code on `SIGKILL`.

## What the generalization did to the committed workflow

The mode `BootMediumReaderKeyMint` became `FleetKeypairMint`, and one job emits one mint step plus one receipt
upload **per row** of `fleet_keypair_mint_subject_rows()`, each gated on `mode == 'fleet_keypair_mint'` alone.
There is deliberately **no `keypair_subject` dispatch input**: this workflow's input roster sits exactly at
GitHub's hard limit of 25 (`workflow_dispatch_max_inputs`), and the emitter refuses to emit an over-cap roster
— which is how the input-gated first draft was caught, since the claim witnesses passed while
`expected_fleet_converge_yml` refused. The selection is instead the PIN: each row's mint asks its own secret
container whether the public key is pinned; a pinned row reports a holds-verdict and exits zero without
reaching an effect, so one dispatch cannot mint a key the fleet already has and cannot leave one un-minted.
The step's entry, function and receipt path are read off the row the step is built from, so a log line naming
one key cannot have executed the other's entry — which is why those three are subject fields and not a lookup
table in `gunbc.ci.ci_spec`.

## Regenerating the committed workflow

`.github/workflows/fleet-converge.yml` is a generated artifact (its header names the authority:
`gunbc.fleet.fleet_converge_workflow expected_fleet_converge_yml`, regen via
`gunbc.instruments.generated_artifact_gate main_wet`). Its bytes cannot be regenerated in this session: no
available interpreter binary is built at the current main, and building one is ruled out. No test compares
the committed file to the emission (only the scheduled `drift-watch` job on the default branch does, and it
reports rather than gate a PR), so the Dag change is CI-hermetic and the committed yml is a named follow-up.
