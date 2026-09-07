# srv1 sudoers drift — observation receipt (2026-09-07)

Authority for the class: `gunbc.recurring_failure_mode.capability_arrives_outside_the_converged_path`,
Specimen C. This file is the detail behind that row's specimen entry.

**Why this file exists.** The observation was made on a live host while wiring the KVM applier that
row calls for. If the host is converged, the observed state is gone — so the record has to outlive
it. A chat message is not a record.

**Handling constraint (operator ruling).** Grants are described by SHAPE — binary, run-as, whether an
argument restriction is present — and never transcribed as copyable authorization lines. The point of
this document is the *shape of the drift*, and a verbatim line would add nothing to that while making
the file itself a liability.

## What was observed

On `srv1`, at the time of observation:

- The **modeled** grant artifact for this host, `provisioning/srv1/gunbc-ghrunner.sudoers`, is
  **absent from the host's sudoers directory**. It is not installed under any name.
- An **untracked, hand-maintained file** stands in its place. It is not in this repository: no
  authority row, no generator, no ignore rule, and no path that any modeled emission writes.
- Its mtime was **the same day it was found**, so it is maintained, not vestigial. A backup file
  beside it, dated some weeks earlier, lacks three of the grants present in the live file — so those
  three were **added** in the interval.

### The shape of the divergence

| | modeled artifact | installed file |
|---|---|---|
| run-as | `root` | `ALL` |
| argument restriction | **exact argv**, one line per permitted invocation | **none** — bare binary path |
| population | per-path, per-unit, enumerated | seven binaries, unrestricted |

Several of the seven binaries are ordinary root escalations when granted without argument
restriction — this follows from each tool's own documented behaviour (editing the privileged
configuration, altering account group membership, creating accounts, writing arbitrary
root-owned files, running arbitrary units). **No escalation was attempted or tested.** The property
is inherent to unrestricted-argument grants on those binaries and does not need demonstrating.

The grantee is the account that **executes CI jobs** on this host. That is what makes this a live
privilege boundary rather than only a provenance gap.

## Why this blocked the work in flight

The applier being wired runs one modeled argv — a supplementary-group grant for the KVM device.
While an unrestricted grant for that binary stands, **that argv is admitted by the hole, not by the
modeled line.** A success would therefore prove the over-broad grant works, not that convergence
does: the instrument and the subject share a path. That is a corrupted instrument, not a deferred
cleanup, which is why the work stopped here rather than proceeding and filing the finding.

## The discriminator, fixed in advance

**Stated in the authority row, not here.** The discriminating negative control, its positive
control, and why neither alone is sufficient are carried by
`gunbc.recurring_failure_mode.capability_arrives_outside_the_converged_path` — the row whose specimen
this document details. It is written there because that is the carrier the class lives on, and
restating it here would give one fact two independently editable homes.

What this document adds is only the *occasion*: the discriminator was fixed **before** the evidence
existed, because a post-convergence green is uninformative unless what would falsify it was named
first.

## A second, independent fact recorded at the same time

Applying the group grant does **not** make the device usable by anything already running. The live
runner listener process holds a fixed supplementary-group set that does not include the device group.
`usermod -aG` edits the account database; it cannot alter the credentials of a running process. The
executor acquires the group only in a **new session** — i.e. after the service restarts.

This is why the reconciler models `applied, pending new session` as an outcome distinct from both
`applied and now writable` and from failure. Collapsing them would report either a false success or a
false failure; which one occurs is determined by re-observation, never assumed.

## How to re-derive

Read-only, from a shell on the host:

- list the sudoers drop-in directory with a long listing (names, sizes, mtimes)
- grep that directory for `NOPASSWD` to see each grant's run-as and whether an argument follows the
  binary path
- search the same directory for the modeled artifact's filename to confirm presence or absence
- read the runner listener's `/proc/<pid>/status` `Groups:` line, and compare against the device
  group's gid from `getent group`

Nothing above modifies the host.
