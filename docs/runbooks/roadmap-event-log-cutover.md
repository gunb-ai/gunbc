# Roadmap event-log cutover — export → stop → migrate → verify → start

> **Scaffold status of the shell in this file.** The nine numbered steps below are `cutover_step_rows` rendered —
> each heading names its row, and each row declares the modelled entry that step must go through when it changes a
> roadmap fact, what the step does to the host, and the predicate that admits the next change. The commands
> themselves are hand-written and marked scaffold against the orchestration-to-shell bash-emit target, as
> `gunbc.witness.roadmap_cutover::runbook_dissolve_trigger`; the two witnesses beside it carry
> `::runbook_witness_dissolve_trigger` and `::dispatch_witness_dissolve_trigger`. When the modelled host-effect and
> instrument routes (`gunbc.instrument_targets`, `gunbc.live_deploy`, `gunbc.fleet`) can carry the steps themselves,
> these commands are emitted from the rows and this file shrinks to what an operator actually reads: prerequisites,
> readback, dispositions.

**What this is.** The one authorized sequence that moves `retired` and `accepted` in the roadmap from their old
homes (an authored `SupersededLine` list, a JSONL history file, a fabric partition watched by an observation
module) to their single home: the roadmap event log. It is run by the operator (or their delegate) on srv1, not
by the code's author and not by any agent, because it stops and starts production readers and deploys.

**Why this order and no other.** `gunbc.design_argument::replacement_migration`: "the old authority ends in one
motion at its root: the old reader is *consumed*, its authority is *taken away*, and what replaces it answers
every case the old one answered." Concretely that forbids two orderings people reach for:

- **Deploying first** strands the event log behind a build that refuses it (a live page 503 for as long as it
  takes), and
- **exporting live** would let the observation modules write the fabric partitions *during* the export, so the
  export would be a snapshot of a moving thing — and after a reset it would be unfalsifiable, because the
  observation would re-materialize the pre-cutover content inside the export directory.

So: **export from the remote-authoritative branch tip with the observers stopped → stop the readers → migrate
into that same remote → verify through production's own readers → only then stage and activate the build that
reads the event log, in one motion.** The readers stay down for a bounded window; nothing publishes a partial
state, because every step that could publish is behind an assertion that refuses.

**The one authority.** Everything here is read back through `gunbc.roadmap.roadmap_event_log` (the log) and
`gunbc.roadmap.roadmap_event_cli`'s `roadmap_event_log_read_back_cli`, which composes the migration verifier, the
retired-set reader and the acceptance-history reader exactly as production's readers compose them, and refuses if
its own document cannot be written. A `gunbc run` CLI entry that returns data instead of a refusal is refused by
the runner, so these read-backs are `ProcessExit` doors, not bare loads.

**The order is modelled, and this file realizes it.** `gunbc.witness.roadmap_cutover::cutover_step_rows` declares
one row per step below — what it may invoke and which predicate admits the next state change — and
`::cutover_witness_case_rows` declares the cases the witnesses run, including the refusal text each must produce.
The two `tools/roadmap_cutover_*.sh` witnesses load their helpers from THIS file at run time and are marked
scaffold against those rows (`::runbook_witness_dissolve_trigger`, `::dispatch_witness_dissolve_trigger`): when the
rows can run as modelled test rows, the shell is deleted rather than emitted. A step whose predicate is dropped
here, or a witness case that would accept any nonzero exit, is a claim failure
(`test.claim.roadmap.roadmap_cutover_witness`), not a typo.

**Conventions.** `$REPO=/opt/gunbc/gunbc`, `$BIN=/opt/gunbc/bin/gunbc`, `$E=gunbc.roadmap.roadmap_event_cli`,
`$D=/opt/gunbc/exports/roadmap-event-log-cutover-<UTC timestamp>` (kept, never deleted), `$SHA` = the exact
revision being deployed. Every heavy command runs under `sudo systemd-run --wait --collect --pipe --uid=briansrls
-p MemoryMax=24G`. Nothing in this runbook touches `/var/lib/gunbc/printer-starts`, any fabric partition other
than the two named here, or any service outside the five `gunbc-roadmap*` units.

**Finding numbers.** The side chat reviewed this procedure four times and its findings were numbered from scratch
each time, so this file cites them by the row they are answered in, in the PR body's *Response to the reviews* table
(PR finding N), and by what was wrong rather than by a bare letter and digit.

**Failure policy.** Each step's assertions are *predicates*, not commentary: a failed one exits nonzero and the
script stops before any further state change, with the readers left as they were. Step 7 is the rollback; it
distinguishes **7A degraded restore** (no append landed, so the pre-cutover units may return — the page still
refuses, because three `retired` events already sit on the remote) from **7B roll forward** (an append landed, so
only the staged event-log-reading build may serve).

# Roadmap event-log cutover — export → stop → migrate → verify → start

All of it runs as `briansrls` on srv1, heavy work under `sudo systemd-run --wait --collect --pipe
--uid=briansrls -p MemoryMax=24G …`. The only writes are event-carrier commits made by
`issue_command_apply` beneath `roadmap_event_cli` — no storage file is ever edited, `/var/lib/gunbc/printer-starts`
is never touched, and `gunbc-roadmap-fabric-storage.service` and its socket are never touched.

Set once after the operator's staging step (checkout + binary side by side, unit file repointed, service
NOT started):

```bash
SHA='<main sha as staged>'
BIN=/opt/gunbc/bin/gunbc-$SHA
TREE=/opt/gunbc/releases/$SHA
OPERATOR='principal:google-oidc/accounts/116084671989231734979'   # the principal already seen on live events
RUN()  { sudo systemd-run --wait --collect --pipe --uid=briansrls -p MemoryMax=24G bash -lc "$1"; }
# RUNE: RUN with the DATA passed as environment (--setenv=), never interpolated into the command line.
# Call it as:  CMD='<the single-quoted gunbc command, referring to "$VAR">';  RUNE "VAR=$value"
RUNE() { local envs=() k; for k in "$@"; do envs+=(--setenv="$k"); done
         sudo systemd-run --wait --collect --pipe --uid=briansrls -p MemoryMax=24G "${envs[@]}" bash -lc "$CMD"; }
# RUNF: RUN a SCRIPT that travels INTACT on stdin, with its DATA as positional arguments. This exists because
# a double-quoted script handed to RUN is expanded by THIS shell before systemd-run ever sees it: the `$(git …)`
# would run here, and `"$URL"` — unset here, assigned only inside the inner script — would reach the inner
# `git clone` as the empty string, which fails or clones nothing while the caller still sees a tidy prompt.
# Same class, same cure, for every nested quote or here-string. With a QUOTED heredoc delimiter nothing is
# expanded locally at all, and paths arrive as data ($1, $2, …), never as code.
# Call it as:  RUNF "$D" <<'REMOTE' … REMOTE   and inside, `D=$1`.
# Verified locally, because the difference is exactly the class those two findings are about: `bash -s` keeps $0
# for itself and EVERY word after `--` becomes $1…, so a filler word there would silently shift the arguments
# and the script would read its export directory as the string "runbook". No filler word: $1 is $D.
RUNF() { sudo systemd-run --wait --collect --pipe --uid=briansrls -p MemoryMax=24G bash -s -- "$@"; }
```

## 1. Read-only export of everything touched, immediately before any write — row `export`

One script, one transient unit, one log. It travels through `RUNF` (above) so that every `$(…)`, every
`$D` and every nested quote is expanded **remotely, in order, in one shell** — the reviewer reproduced the
failure mode of the double-quoted form: `URL` was unset in the caller's shell, so the assignment arrived
after the `git clone` had already been handed `""`.

```bash
D=/opt/gunbc/exports/t6-$(date -u +%Y%m%dT%H%M%SZ)      # named ONCE, locally; below it is DATA, not code
mkdir -p /tmp/t6
# The heredoc is attached to RUNF (a `| tee` on the caller's line would attach it to tee instead), and the
# log is the artifact the PR quotes.
RUNF "$D" <<'REMOTE' > /tmp/t6/step1-export.log 2>&1
set -euo pipefail
D=$1
# ARGUMENT-DELIVERY GUARD. `bash -s` keeps $0 for itself and makes EVERY word after `--` a positional
# parameter, so a stray filler word in the helper would hand this script the literal string "runbook" as $1 and
# export into a relative directory. Rather than trust that, the script refuses anything that is not an absolute
# export path. Exercised in the Validation section, in both directions.
case "$D" in
  /*) : ;;
  *) echo "RUNF REFUSED: the export directory arrived as [$D] — argument delivery shifted"; exit 1 ;;
esac
install -d -m 0750 "$D"
UNITS=(gunbc-roadmap.service gunbc-roadmap-tailnet-door.service gunbc-roadmap-belt-discovery.service gunbc-roadmap-belt-exit.service gunbc-roadmap-belt-credential-recovery.service)
# EVERY affected unit's drop-in directory, not just the one that happens to have one today: the modeled deploy
# is entitled to add a drop-in to any of them, and a rollback that restores only one leaves the others diverged.
export_drop_ins() { # $1 = the systemd directory, $2 = the export directory
  install -d -m 0755 "$2/units/drop-ins"
  for U in "${UNITS[@]}"; do
    if [ -d "$1/$U.d" ]; then sudo cp -a "$1/$U.d" "$2/units/drop-ins/"; fi
  done
  find "$2/units/drop-ins" -type f 2>/dev/null | sort > "$2/units.drop-ins.inventory"
}
# The AUTHORITATIVE carrier is the `roadmap-events` BRANCH of origin, not srv1's checkout of it: every read
# and append owns a private snapshot that fetches origin and resets to the fetched commit, and unpushed
# local state is discarded on purpose (roadmap_event_carrier.dag:61-62, 201-215). srv1's holder is stale at
# 02c95a0 (2026-10-03), so the export takes the branch itself, at an exact tip. Read-only: writes under $D.
URL=$(git -C /opt/gunbc/roadmap-events remote get-url origin)
test -n "$URL"                                            # an empty remote URL is a refusal, not a clone of nothing
git clone --quiet --branch roadmap-events --single-branch "$URL" "$D/branch"
git -C "$D/branch" rev-parse HEAD > "$D/roadmap-events.tip.before"   # verified 2026-11-10: 10bf77aa512883ad…
git -C "$D/branch" ls-tree -r --name-only HEAD > "$D/roadmap-events.tree.before"
git -C "$D/branch" bundle create "$D/roadmap-events.bundle" --all
tar -czf "$D/roadmap-events.holder.tar.gz" -C /opt/gunbc roadmap-events   # the holder, corroborating only
# The unit files step 5b's deploy repoints are part of what is touched, so they are exported in the same
# breath, at the same instant: the five serving units, any drop-in the model added, and the EFFECTIVE
# definition (systemctl cat) — a drop-in is not the same thing as the file it overrides.
install -d "$D/units"
sudo cp /etc/systemd/system/gunbc-roadmap.service /etc/systemd/system/gunbc-roadmap-tailnet-door.service \
        /etc/systemd/system/gunbc-roadmap-belt-discovery.service \
        /etc/systemd/system/gunbc-roadmap-belt-exit.service \
        /etc/systemd/system/gunbc-roadmap-belt-credential-recovery.service \
        /etc/systemd/system/gunbc-roadmap-belt-credential-recovery.timer "$D/units/"
export_drop_ins /etc/systemd/system "$D"
for U in "${UNITS[@]}"; do systemctl cat "$U"; done > "$D/units.catalog"                                  # the EFFECTIVE definitions, drop-ins included
ls "$D/units" > "$D/units.inventory"
sudo find /etc/systemd/system -maxdepth 1 -name 'gunbc-roadmap*' -printf '%P\n' | sort > "$D/units.live.inventory"
# The fabric store, whole and content-addressed (objects/, heads/, the state/ slot), because objects are
# shared by digest — and which units are ACTIVE, so a rollback can tell "stopped by us" from "was already down".
install -d "$D/fabric"
sudo cp -a /opt/gunbc/fabric-storage "$D/fabric/storage"
sudo find /opt/gunbc/fabric-storage -type f | sort > "$D/fabric/inventory.before"
sudo find /opt/gunbc/fabric-storage -type f -print0 | sudo xargs -0 sha256sum | sort > "$D/fabric/SHA256SUMS.before"
sudo grep -rl 'roadmap-operator-acceptance' /opt/gunbc/fabric-storage | sort > "$D/fabric/acceptance.before" || true
wc -l < "$D/fabric/acceptance.before" | tr -d ' ' > "$D/fabric/acceptance.count"   # expect 0
for U in "${UNITS[@]}"; do printf '%s %s\n' "$U" "$(systemctl is-active "$U")"; done > "$D/units.active.before"
systemctl show gunbc-roadmap-fabric-storage.service -p ExecStart -p FragmentPath -p MemoryMax \
  -p MainPID -p ExecMainStartTimestamp > "$D/fabric-storage-unit.before"
# The 18 seed lines are read from GitHub at the pinned commit that still holds the file (its last
# revision before this PR deleted it), so srv1 needs no fetch and no checkout of this branch.
gh api 'repos/gunb-ai/gunbc/contents/dag/gunbc/roadmap/roadmap_acceptance_event_history.jsonl?ref=4aad92f8fdbcdaa029967ea0b174b1b91b81459b' --jq .content | base64 -d > "$D/seed.jsonl"
wc -l < "$D/seed.jsonl" | tr -d ' ' > "$D/seed.count"      # expect 18
# The digest is compared against the FILE, with no sed and no nested quoting to get wrong.
printf '%s  %s\n' e5c9aac33080519e982dbc47b5cc83516985522d6946fd0df657ef715d68b5cc "$D/seed.jsonl" | sha256sum -c -
find "$D" -type f -printf '%s\t%p\n' | sort -k2 > "$D/MANIFEST"
sha256sum "$D/roadmap-events.bundle" "$D/roadmap-events.holder.tar.gz" "$D/seed.jsonl" "$D/units.catalog" > "$D/SHA256SUMS"
echo "EXPORT_DIR=$D"
REMOTE
cat /tmp/t6/step1-export.log        # show this in the PR: it carries EXPORT_DIR and the sha256sum -c OK line
```

**Receipt for the wrong-shell expansion (PR finding 13), taken locally before anyone touches srv1.** The same two
shapes, run on a scratch box with
`git`/`gh` stubbed — the old one dies on the shell boundary, the new one does the work:

```text
=== OLD FORM: the script is a double-quoted string, so $URL is expanded HERE, where it is unset ===
bash: line 2: URL: unbound variable

=== NEW FORM: RUNF — quoted heredoc, data as positional args, one remote shell ===
D arrived as data:        [/opt/gunbc/exports/t6-proof]
clone target (remote):    [https://example.invalid/roadmap-events.git]  into [/opt/gunbc/exports/t6-proof/branch]
/tmp/t6-proof/seed.jsonl: OK
```

The old shape cannot be made to work by escaping harder: the caller's shell and the transient unit's shell
cannot both own the same variables, which is why the fix moves the script wholesale rather than adjusting the
quotes. Every fenced `bash` block in this runbook is checked with `bash -n` (10 blocks, all clean), and the two
heredoc scripts — step 1's export and step 7's rollback — are additionally executed as intact units against
stubs, as above.


Record the printed `EXPORT_DIR`, `roadmap-events.tip.before`, `acceptance.count` and the
`MANIFEST`/`SHA256SUMS`/`fabric/SHA256SUMS.before` in the PR. The fabric copy is the WHOLE content-addressed store (`objects/`, `heads/`, the `state/` slot) rather than one
partition, because objects are shared by digest. Step 1 writes nothing outside `$D`: `cp -a`, `find`,
`sha256sum`, `grep`, `clone`.

**Why `acceptance.count` is expected to be 0** (measured read-only 2026-11-09): `/opt/gunbc/fabric-storage`
holds ~180 objects, 87 heads (`cas-slot-*`, `host-placement.*`, `pair-serving-authority-group-a.*`) and a
5-object `state/` slot, with NO head named `roadmap-operator-acceptance.<n>` and no object carrying an
acceptance record — the only object mentioning "acceptance" is an unrelated `roadmap-task-event/v1` record. So
the fabric acceptance partition is empty in production (the serve close route never advanced it, or the
2026-09-19 cutover reset it), deleting its writer loses no live fact, and the verifier still compares against
the manifest derived from this export, so a tail appearing before the run is carried, not lost.

**The authoritative carrier, measured read-only on 2026-11-10 at branch tip `10bf77aa512883ad…`:** 126
objects, of which `events/` holds 67 across 23 node chains — `claimed` 60, `retired` 3, `result_returned` 2,
`comment_created` 2 — and `operations/` holds 58 operation receipts (`{event, id}`, the idempotence receipts the
append route writes beside each event). None of the 18 migrated nodes has ANY event on the branch, so each of
the 18 appends starts a fresh chain and no append can collide with an existing one. Of the two retirements:
`entry-graph-union-construction` has no chain (its event's parent is the genesis id) and
`shell-dag-floor-peak-calibration-rows` holds one `claimed` event, `0cc95abae720582e…`, which is therefore the
compare-and-swap parent its retirement names. The three `retired` events, all reason `test`, sit on
`harness-checkpoint-subject-planner`, `harness-conversation-log-forking` and `v2-emitter-direct-rust-door` —
they are what put the 503 on the live page (step 2b), and they are exactly the kind step 3 writes.


(The unit files are exported inside the same script because they are part of what is touched: step 5b's
deploy repoints them, so a rollback needs both the files and the effective definitions from the same instant.)

## 2. Stop every reader of this carrier — timers first — row `stop`

The operator ruled (relayed 2026-11-10): all five roadmap units move together to the staged release, and
`gunbc-roadmap-fabric-storage.service` is not touched at all. Stop the timer before the units it drives, so
a tick cannot fire into the window:

```bash
set -euo pipefail
# THE SAME FIVE THIS CUTOVER OWNS — spelled identically to UNITS in step 1's export (tools/
# roadmap_cutover_runbook_witness.sh PART G refuses the two spellings apart), plus the timer that drives two
# of the oneshots. A roster here that differs from the exported one means a reader was never measured.
READERS=(gunbc-roadmap.service gunbc-roadmap-tailnet-door.service gunbc-roadmap-belt-discovery.service \
         gunbc-roadmap-belt-exit.service gunbc-roadmap-belt-credential-recovery.service)
TIMER=gunbc-roadmap-belt-credential-recovery.timer
# EVERY ACT IS CHECKED, AND THE STOP IS ADMITTED BY MEASUREMENT — the old block printed "expect: inactive"
# and moved on, which is how an unobserved stop leaves a reader serving the old code through the whole
# migration window. Timer first, so a tick cannot fire into the window.
sudo systemctl stop "$TIMER" || { echo "STOP REFUSED: the belt-credential-recovery timer is still loaded"; exit 1; }
for U in "${READERS[@]}"; do
  sudo systemctl stop "$U" || { echo "STOP REFUSED: $U did not stop"; exit 1; }
done
# two of the belt oneshots are in `failed` state today; clear that so a later start is not refused
sudo systemctl reset-failed gunbc-roadmap-belt-exit.service gunbc-roadmap-belt-credential-recovery.service \
  || { echo "STOP REFUSED: the failed state could not be cleared"; exit 1; }
for U in "${READERS[@]}" "$TIMER"; do
  ST=$(systemctl is-active "$U" 2>/dev/null || true)
  [ "$ST" = inactive ] || { echo "STOP REFUSED: $U reports [$ST], not inactive — no migration write may follow"; exit 1; }
done
LIVE=$(ps -ef | grep -c "[/]opt/gunbc/bin/gunbc" || true)
# This check is written to fail in the annoying direction, which is the safe one: the pattern is broad enough
# that an unrelated gunbc process — a printer, a serve, someone's session — makes LIVE non-zero and the stop
# refuses. A false refusal costs an operator a look at `ps`; a false clearance costs a migration racing a live
# reader. Do not "fix" this by narrowing the pattern to the units' own names.
[ "${LIVE:-0}" -eq 0 ] || { echo "STOP REFUSED: $LIVE gunbc process(es) still running"; exit 1; }
echo "STOP ADMITTED: ${#READERS[@]} units and the timer report inactive, no gunbc process remains"
```

Nothing else on srv1 reads this carrier; the enumeration, with each unit file, is under "Who else reads
this carrier" below, and the lab instance `gunbc-srv1-lab-codex-feedback-v0*` keeps its own carrier at
`/opt/gunbc-lab/codex-provider-feedback-v0/roadmap-events`.

## 2b. Stage only — the release is copied; the deploy is NOT run here, and not at this point — row `stage`

Staging is a copy and nothing else: a checkout at `/opt/gunbc/releases/<SHA>` and a binary at
`/opt/gunbc/bin/gunbc-<SHA>`, which the parent already put on srv1 (both still there after his rolled-back
attempt of 14:11/14:21). `BIN` below is that staged binary and `TREE` its checkout: every write in steps 3 to
5 goes through them, because the deployed Oct-15 build cannot read the kinds being appended. No unit file, no
fragment, no symlink moves — the five units are still the old ones, and they are stopped from step 2, so
staging activates nothing. **The deploy is step 5b, after `MIGRATION VERIFIED`.** It is not run in this PR:
it needs #16256 merged first, so that it leaves `gunbc-roadmap-fabric-storage.service` alone, and it needs the
operator's go-ahead.

## 3. Append the history first, in its recorded order — through the modeled route only — row `migrate-accepted`

The manifest is the exported text of the two old carriers. Build the list of lines to replay
(`$D/seed.jsonl` for the 18 committed seed events, then the fabric tail's own lines in their recorded
order), then append. `roadmap_acceptance_record_cli` takes the receipt LINE — the node is the receipt's own
field, and the route beneath refuses a receipt naming another node; `roadmap_acceptance_revoke_cli` takes
the revocation line alone.

**The three pull-request arguments are passed EMPTY, and that is the honest value, not a placeholder.**
The 17 acceptance rows the seed holds carry `node`, `criteria_digest`, `red_control`, `handback`,
`accepted_by` and `accepted_on` — and no pull request, merge time or head SHA: the hand-committed seed
never recorded those fields (checked with `jq` over all 18 rows; its latest `accepted_on` is 2026-10-11).
So this runbook has no historical source to resolve them from, and inventing one to satisfy a field would
be a fabricated fact in the log. The event therefore records them as an ABSENCE — the typed
variant `AcceptancePrEvidence.NoPullRequest`, written as no member and read back as no member — and the
route admits that absence only for a receipt whose `accepted_on` day is before
`roadmap_acceptance_pr_evidence_cutoff` (`2026-11-01`), the dated door whose entire case is this one-time
migration, because the live tail it replaces was read and verified empty. Half a pull request cannot be
written at all (it is not a value of the type) and is refused where the three arguments are read
(`roadmap_acceptance_pr_evidence_from_args`); an absence dated on or after the cutoff is refused at the
route (`issue_command_acceptance_evidence_law`; witnesses
`a_partial_pull_request_evidence_refuses_and_an_absence_is_admitted_only_before_the_cutoff` and
`a_migrated_event_with_swapped_evidence_disagrees_with_the_manifest`).

**No line of JSON is interpolated into a shell string anywhere in this runbook.** Each line is handed to
the modeled route through the **environment** (`RUNE` in the preamble, `systemd-run --setenv=`), so nothing
in a receipt line — quotes, `$`, backslashes, a stray newline — can be re-parsed by an intermediate shell;
the command each loop runs is single-quoted, so the only expansion inside it is the environment variable
itself. This is not a hypothetical: the one historical revocation carries the reason text

```
The floor cut retired this work's subject
```

— an apostrophe inside a JSON value, which under the older `--arg revocation_line='$LINE'` form terminated
the shell's single-quoted argument and made that one line impossible to migrate. Passing lines as environment
values is what makes it migratable; the verifier then requires its reason, revoker and day to match the export
byte for byte.

```bash
set -euo pipefail
cd $TREE
: "${D:?MIGRATION REFUSED: EXPORT_DIR is not set — run step 1 and use the path it echoed}"
: "${OPERATOR:?MIGRATION REFUSED: the author is part of the fact; an empty author is a wrong fact}"
# THE SEED IS RE-VERIFIED BY DIGEST, NOT BY COUNT, BEFORE ANYTHING IS APPENDED. Step 1 hashed the seed it
# exported into $D/SHA256SUMS. A line count cannot see a rewrite — an approver named differently, a receipt tail
# edited, one line swapped for another of the same length — and step 5 builds its expected manifest from the file
# as it is HERE. So the digest recorded at export is re-checked against the file on disk, and a mismatch stops
# the migration BEFORE the first append rather than after a whole history has been written to the carrier.
[ -s "$D/SHA256SUMS" ] || { echo "MIGRATION REFUSED: step 1 recorded no SHA256SUMS to re-verify the seed against"; exit 1; }
RECORDED_SEED_SHA=$(awk -v f="$D/seed.jsonl" '$2 == f { print $1 }' "$D/SHA256SUMS")
[ -n "$RECORDED_SEED_SHA" ] \
  || { echo "MIGRATION REFUSED: $D/SHA256SUMS carries no digest for $D/seed.jsonl — the export was never hashed"; exit 1; }
[ "${#RECORDED_SEED_SHA}" -eq 64 ] \
  || { echo "MIGRATION REFUSED: the recorded seed digest is ${#RECORDED_SEED_SHA} characters, expected a 64-character sha256"; exit 1; }
[ "$(sha256sum "$D/seed.jsonl" | cut -d" " -f1)" = "$RECORDED_SEED_SHA" ] \
  || { echo "MIGRATION REFUSED: $D/seed.jsonl no longer matches the digest step 1 recorded — same count, different content is exactly what a count cannot see"; exit 1; }
echo "SEED RE-VERIFIED against the digest step 1 recorded, before any append"

# INPUT PREPARATION IS ITS OWN CHECKED STEP, NOT A PROCESS SUBSTITUTION. `< <( … )` hides every failure from
# `set -e`: a dead `sudo`, a jq error, or a missing seed line would simply produce fewer lines, the loop would
# append what it got, and the block would report a migration that migrated LESS THAN THE EXPORT. So the inputs
# are written to files by checked commands and their counts are admitted against the export BEFORE any append.
sudo grep -rl 'roadmap-operator-acceptance' $D/fabric/storage > "$D/migrate.fabric.objects" 2>/dev/null || true
: > "$D/migrate.acceptance.lines"
while read -r o; do
  sudo sed 's/^fabric_storage\.object;[0-9]*://' "$o" | jq -er '.payload.line' >> "$D/migrate.acceptance.lines" \
    || { echo "MIGRATION REFUSED: $o does not carry a payload.line — the fabric tail is not what the export measured"; exit 1; }
done < "$D/migrate.fabric.objects"
grep acceptance_recorded "$D/seed.jsonl" > "$D/migrate.seed.acceptance"
cat "$D/migrate.seed.acceptance" >> "$D/migrate.acceptance.lines"
grep acceptance_revoked  "$D/seed.jsonl" > "$D/migrate.revocation.lines"

NFB=$(tr -dc '0-9' < "$D/fabric/acceptance.count" 2>/dev/null) \
  || { echo "MIGRATION REFUSED: the export carries no fabric/acceptance.count — step 1 never measured the fabric tail"; exit 1; }
NFB=${NFB:-0}
NSEED_EXPORT=$(tr -dc '0-9' < "$D/seed.count" 2>/dev/null) \
  || { echo "MIGRATION REFUSED: the export carries no seed.count — step 1 never counted the seed"; exit 1; }
NSEED=$(wc -l < "$D/seed.jsonl"); NSEED=${NSEED:-0}
# The seed the digest admitted in step 1 must still be the seed step 1 COUNTED. A file edited, truncated or
# half-written after the export is not the evidence the export describes, and the count below would otherwise
# quietly derive its own expectation from it.
[ "$NSEED" = "$NSEED_EXPORT" ] \
  || { echo "MIGRATION REFUSED: the seed holds $NSEED lines where the export measured $NSEED_EXPORT — the file changed after step 1"; exit 1; }
NREV=$(wc -l < "$D/migrate.revocation.lines"); NREV=${NREV:-0}
NFAB=$(wc -l < "$D/migrate.acceptance.lines"); NFAB=${NFAB:-0}
# The expected acceptance count is DERIVED from what step 1 measured (seed lines minus revocations) plus the
# fabric tail the export counted — never copied from this document, so it cannot drift out of date here.
EXPECT=$(( NSEED - NREV + NFB ))
[ "$NFAB" -eq "$EXPECT" ] \
  || { echo "MIGRATION REFUSED: $NFAB acceptance lines prepared, the export says $EXPECT — the input was cut short; nothing is appended"; exit 1; }
[ "$NREV" -eq 1 ] \
  || { echo "MIGRATION REFUSED: the seed must carry exactly one revocation, found $NREV; nothing is appended"; exit 1; }
echo "INPUT ADMITTED: $NFAB acceptance lines ($(( NSEED - NREV )) from the seed, $NFB from the fabric tail) and $NREV revocation"

# EACH APPEND IS AN ADMITTED STEP, NOT A LINE IN A LOOP: the first refusal aborts the block, so no later
# receipt is appended and step 4's retirements cannot run on top of a half-migrated history. Each append
# prints its own located refusal (roadmap_acceptance_line_of refuses; it never guesses a node or a receipt).
I=0
while IFS= read -r LINE; do
  I=$((I + 1))
  CMD="$BIN run --source-root dag --source-root src/v2 \
    --entry dag/gunbc/roadmap/roadmap_event_cli.dag --function roadmap_acceptance_record_cli \
    --arg repo_root=/opt/gunbc/gunbc --arg receipt_line=\"\$RECEIPT_LINE\" \
    --arg pr_url= --arg merged_at= --arg head_sha= \
    --arg author=\"\$OPERATOR\""
  RUNE "RECEIPT_LINE=$LINE" "OPERATOR=$OPERATOR" \
    || { echo "MIGRATION REFUSED: acceptance $I of $NFAB did not append — no further appends, and step 4 must not run"; exit 1; }
  echo "appended acceptance $I/$NFAB"
done < "$D/migrate.acceptance.lines"

# then the revocations, in recorded order, after the acceptance they name is live:
J=0
while IFS= read -r LINE; do
  J=$((J + 1))
  CMD="$BIN run --source-root dag --source-root src/v2 \
    --entry dag/gunbc/roadmap/roadmap_event_cli.dag --function roadmap_acceptance_revoke_cli \
    --arg repo_root=/opt/gunbc/gunbc --arg revocation_line=\"\$REVOCATION_LINE\" \
    --arg author=\"\$OPERATOR\""
  RUNE "REVOCATION_LINE=$LINE" "OPERATOR=$OPERATOR" \
    || { echo "MIGRATION REFUSED: revocation $J of $NREV did not append — no further appends, and step 4 must not run"; exit 1; }
  echo "appended revocation $J/$NREV"
done < "$D/migrate.revocation.lines"
# THE COMPLETION RECEIPT IS WRITTEN ONLY WHEN EVERY APPEND WAS ADMITTED. The two line files are INPUTS, prepared
# before the first append, so they survive an aborted migration sitting there full — their being non-empty is not
# evidence that the carrier ever took them. This file is the only thing that records that the whole history
# arrived: the counts, the seed digest the appends were made against, and a digest of the inputs themselves.
# Nothing below it runs unless the loops above completed, so the file cannot exist for a partial migration.
{
  printf 'acceptances=%s\n' "$NFAB"
  printf 'revocations=%s\n' "$NREV"
  printf 'seed_sha256=%s\n' "$RECORDED_SEED_SHA"
  printf 'inputs_sha256=%s\n' "$(cat "$D/migrate.acceptance.lines" "$D/migrate.revocation.lines" | sha256sum | cut -d" " -f1)"
} > "$D/migrate.completion"
echo "MIGRATION COMPLETE: $NFAB acceptance receipts (a seed of $NSEED lines + $NFB from the fabric tail) and $NREV revocation receipt admitted; completion recorded in $D/migrate.completion"
```

Each append prints its own refusal, verbatim and located, if the line is not spelled exactly as the
operator law spells one (`roadmap_acceptance_line_of` refuses, it never guesses a node or a receipt).
Stop and read the refusal before continuing — see the ordering note under this block.

What "an append succeeded" means: the CLIs return a `ProcessExit` (`ExitSuccess`, or `ExitFailure { code: 1,
reason: … }` whose reason the runtime prints, e.g. `append refused at <step>: <reason>`,
`roadmap_event_cli.dag:256`). Success carries **no event id** — that is why step 5's verifier output, not the
append output, is the per-line receipt.

And what "step 3 finished" means, which is a different claim: `MIGRATION COMPLETE` plus
`$D/migrate.completion`. It is written after the last admitted append and cannot exist for a partial migration,
which is why step 4 requires it. The line files cannot stand in for it — they are written BEFORE the first
append, precisely so that a refusal can be diagnosed against the input that was refused, and they therefore
survive an aborted migration full.

Order is not cosmetic: the seed holds an acceptance of `entry-graph-union-construction` followed by a
revocation of that same receipt, and a revocation whose receipt is not yet live is REFUSED by
`roadmap_revocation_names_live_acceptance`. That refusal is the migration's own fail-closed check, not a
failure to handle — the fix is always the order, never the law.

**An interrupted run resumes by replaying the same commands**, which is what makes the loop above safe to
re-enter after a crash or a killed `systemd-run`: the append route keeps an operation receipt per
(node, kind of fact, receipt digest) and answers the FIRST recording when the same operation returns, even
when the chain head has moved in between — so a retry is `EventAlreadyPresent`, not
`operation-conflict`, and it never appends a second event. The same holds for the two retirements, whose
operation key is the node plus the digest of their reason; a second retirement of an already-retired node
is refused by the admission law rather than appended. Witnessed by executing real appends and real retries
against a temp carrier:
`a_retry_of_a_committed_acceptance_after_the_head_moved_answers_the_first_recording` and
`a_second_retirement_of_a_retired_node_refuses_and_a_retry_of_the_first_answers_it`.

The CLI answers `ExitSuccess`/`ExitFailure` (a `ProcessExit`), so its exit is NOT the receipt. The receipts
are the identities printed by step 4, read out of the carrier through the production reader.

## 4. Then the two retirements (step 1(a)) — the retirement is the newest fact, so it goes last — row `migrate-retired`

```bash
set -euo pipefail
cd $TREE
: "${D:?RETIREMENT REFUSED: EXPORT_DIR is not set — the step-3 admission files are the precondition for this block}"
: "${OPERATOR:?RETIREMENT REFUSED: the author is part of the fact; an empty author is a wrong fact}"
# step 3's admission is a PRECONDITION, checked here rather than trusted. The LINE FILES are only half of it and
# are the weaker half: they are prepared before the first append, so a migration that died at its third receipt
# leaves both of them sitting here full. What proves the history arrived is step 3's COMPLETION RECEIPT — written
# after the last admitted append, and required here in full: present, exactly the four fields step 3 writes,
# counting the same number of receipts the inputs hold, and recorded against the seed that is actually on disk.
# A receipt copied from another run, truncated, or written against a seed since changed cannot authorize a
# retirement.
[ -s "$D/migrate.acceptance.lines" ] \
  || { echo "RETIREMENT REFUSED: $D/migrate.acceptance.lines is missing or empty — step 3 did not admit its input"; exit 1; }
[ -s "$D/migrate.revocation.lines" ] \
  || { echo "RETIREMENT REFUSED: $D/migrate.revocation.lines is missing or empty — step 3 did not admit its input"; exit 1; }
[ -s "$D/migrate.completion" ] \
  || { echo "RETIREMENT REFUSED: step 3 never wrote $D/migrate.completion — its appends did not all reach the carrier, so the acceptance history is incomplete"; exit 1; }
[ "$(wc -l < "$D/migrate.completion")" -eq 4 ] \
  || { echo "RETIREMENT REFUSED: $D/migrate.completion holds $(wc -l < "$D/migrate.completion") fields, expected 4 — that is not the receipt step 3 writes"; exit 1; }
grep -q "^acceptances=$(wc -l < "$D/migrate.acceptance.lines")$" "$D/migrate.completion" \
  || { echo "RETIREMENT REFUSED: the completion receipt counts a different number of acceptance receipts than the inputs hold"; exit 1; }
grep -q "^revocations=$(wc -l < "$D/migrate.revocation.lines")$" "$D/migrate.completion" \
  || { echo "RETIREMENT REFUSED: the completion receipt counts a different number of revocation receipts than the inputs hold"; exit 1; }
grep -q "^seed_sha256=$(sha256sum "$D/seed.jsonl" | cut -d" " -f1)$" "$D/migrate.completion" \
  || { echo "RETIREMENT REFUSED: the completion receipt was written against a different seed than the one on disk"; exit 1; }
echo "COMPLETION RECEIPT ADMITTED: $(tr "\n" " " < "$D/migrate.completion")"

# THE RETIREMENT TEXT AND AUTHOR TRAVEL AS ENVIRONMENT DATA, never spliced into the command string. The first
# refusal aborts the loop: a half-retired floor is not a retired floor, and step 5 must not be told otherwise.
RETIRED_TEXT='The floor cut retired this work; its remaining cost lives on its successor.'
for NODE in entry-graph-union-construction shell-dag-floor-peak-calibration-rows; do
  CMD="$BIN run --source-root dag --source-root src/v2 \
    --entry dag/gunbc/roadmap/roadmap_event_cli.dag --function roadmap_event_record_cli \
    --arg repo_root=/opt/gunbc/gunbc --arg node=\"\$NODE\" --arg kind=retire \
    --arg text=\"\$RETIRED_TEXT\" --arg author=\"\$OPERATOR\""
  RUNE "NODE=$NODE" "RETIRED_TEXT=$RETIRED_TEXT" "OPERATOR=$OPERATOR" \
    || { echo "RETIREMENT REFUSED: $NODE did not take its retirement — no further retirement, and step 5 must not run"; exit 1; }
  echo "retired $NODE"
done
```

An already-retired node answers idempotently; a duplicate of an acceptance already held is refused at the
chain (`EventAlreadyPresent`) or at the operation record (the operation identity is the fact, not the node
— see `roadmap_acceptance_operation_identity`), which is the retry-safe answer, not a lost write.

## 5. Verify — the activation gate, and the source of the receipts — row `verify`

`roadmap_acceptance_migration_verify` is the activation gate, and every one of its comparisons lives in ONE
pure function — `roadmap_acceptance_migration_problems` (`roadmap_operator_acceptance.dag`) — so each
refusal is witnessed directly over constructed lists, not only through a live carrier. It answers as seven separate claims (names at the receipts below), each constructed over lists rather than a
live carrier, so no facet can hide behind another:

1. **Complete facts, as a MULTISET.** Each side's count of each COMPLETE canonical fact must agree. A fact is
   spelled by the same member law that writes the event's own bytes (`roadmap_acceptance_receipt_members`):
   node, criteria digest, the red control and the handback each with its legs, the accepting principal and day
   — and for a revocation its disposition, reason, revoker and day. Node-and-digest membership would certify an event whose evidence had
   been swapped, and membership tested with `any` cannot see a count disagree: a manifest naming one acceptance
   twice against a log holding it once is a lost event. Both are refused, naming the fact and both counts.
2. **Closedness, BOTH WAYS.** The receipts the manifest's own sequence leaves live, against what the log's
   reader (`roadmap_operator_node_closed`) establishes — a lost acceptance refuses, and so does a node the log
   calls closed that the manifest never accepted.
3. **A manifest that does not replay REFUSES** (`the exported manifest does not replay as an acceptance
   history; nothing can be certified from it`): an unorderable history used to fold to an empty expected set,
   which certified whatever the log happened to hold.
4. **The retired roster**, against the fail-closed retired read — an unorderable retirement lane refuses rather
   than answering "nothing is retired", and a named node the log does not hold retired refuses.

It prints the committed event identities, and it prints `MIGRATION REFUSED` naming every disagreement
instead of certifying.

```bash
# THE GATE IS EXECUTABLE, IT IS ONE ENTRY, AND NOTHING DOWNSTREAM RUNS WITHOUT ITS EXIT CODE. The finding this
# answers (PR finding 14) was that the first *executable* `grep '^MIGRATION VERIFIED'` sat in step 6, after step 5b
# had already deployed. It sits here now, and it is not a grep on the operator's reading: `roadmap_event_log_read_back_cli`
# asks the three readers production itself asks — the migration verifier, the fail-closed retired read, and the
# acceptance-history reader — plus the standing sentence for every name in the retired roster, writes what they
# said to $D/read-back.out through the modeled Filesystem.Write, and exits 0 ONLY when all four parts were
# admitted (gunbc.roadmap.roadmap_event_cli::roadmap_read_back_is_admitted, claimed in
# dag/test/claim/roadmap/roadmap_receipt_continuity_acceptance.dag::the_read_back_refuses_when_any_single_part_refuses,
# which includes a document that could not be WRITTEN refusing the whole). `MIGRATION VERIFIED` in prose is not
# the gate; the exit code is, and it is derived from the readers' own verdicts. The roster's standing sentences
# are informational because the verifier itself refuses a roster name whose retirement is not in the log, so a
# missing retirement cannot ride through on a friendly standing line.
MANIFEST="$(cat "$D/seed.jsonl")"   # the fabric store contributes nothing today (verified at step 1: acceptance.count is 0);
                                    # if acceptance.before is not empty, append each object's payload.line as step 3 derives them
ROSTER="entry-graph-union-construction,shell-dag-floor-peak-calibration-rows"
# The manifest travels in the ENVIRONMENT (RUNE), not inside a quoted script: it carries real newlines and JSON
# quoting, and the verifier splits it on newlines and refuses a bad line by index — so it must arrive
# byte-for-byte, never re-parsed by the shell systemd-run spawns. A mangled paste fails loudly rather than
# certifying less than the old carriers held.
CMD="$BIN run --source-root $TREE/dag --source-root $TREE/src/v2 --entry dag/gunbc/roadmap/roadmap_event_cli.dag --function roadmap_event_log_read_back_cli --arg repo_root=/opt/gunbc/gunbc --arg expected_manifest=\"\$EXPECTED_MANIFEST\" --arg retired_roster=\"\$EXPECTED_ROSTER\" --arg report_path=$D/read-back.out"
RUNE "EXPECTED_MANIFEST=$MANIFEST" "EXPECTED_ROSTER=$ROSTER" > "$D/read-back.driver.log" 2>&1
READ_BACK_RC=$?
if [ "$READ_BACK_RC" -eq 0 ] && grep -q '^MIGRATION VERIFIED' "$D/read-back.out"; then
  { echo 'MIGRATION VERIFIED'
    echo "SHA=$SHA"; echo "TREE=$TREE"; echo "BIN=$BIN"
    echo "TIP_BEFORE=$(cat "$D/roadmap-events.tip.before")"
    echo "TIP_NOW=$(git -C "$D/branch" ls-remote origin roadmap-events | cut -f1)"
    echo "MANIFEST_SHA256=$(sha256sum "$D/seed.jsonl" | cut -d' ' -f1)"
    echo "READ_BACK_SHA256=$(sha256sum "$D/read-back.out" | cut -d' ' -f1)"
    echo "VERIFIED_AT=$(date -u +%FT%TZ)"
  } > "$D/migration.verified"
  cat "$D/migration.verified"; cat "$D/read-back.out"
  echo 'MIGRATION VERIFIED — the only state from which step 5b may be entered'
else
  cat "$D/read-back.driver.log"
  echo 'ACTIVATION BLOCKED: the read-back did not admit. The readers stay DOWN — that is the safe state, not a'
  echo 'serving-but-wrong one. Fix the migration, or go to step 7. Step 5b is unreachable from here: exit 1.'
  exit 1
fi
```

Three things this leaves behind, and they are the receipts the PR quotes: `$D/read-back.out` (what production's
own readers answered, in their own words), `$D/read-back.driver.log` (the refusal text when it refused), and
`$D/migration.verified` (the binding: revision, release tree, binary, the remote tip before and after, the
manifest digest, the read-back digest, and the moment). **Step 5b reads the last two of these as admission
predicates, not as prose.**

The listed lines are the acceptance events the log holds (`roadmap_operator_acceptance.dag:819-824`,
`roadmap_event_kind_word` naming each kind) — the per-line receipts, since the CLI returns only a
`ProcessExit`. The two retirements are not listed: they are checked against the roster, and a missing one is
the refusal `the log does not hold <node> retired` (`roadmap_event_log.dag:1861`) — the stronger arm.

The refusals it can print, each naming what failed:
`the log does not hold the manifest's fact in the same number: expected <n>, the log holds <m>, for
<canonical fact>` · `the manifest leaves <node> closed on a live receipt; the log does not say so` ·
`the log calls <node> closed on a live receipt the exported manifest never accepted` ·
`the exported manifest does not replay as an acceptance history; nothing can be certified from it` ·
`the log does not hold <node> retired` · and the pre-checks `MIGRATION REFUSED: the event log does not
replay: …`, `… manifest line <i> is not a valid acceptance event: …`, `… no dashboard instance is
registered at …`.

Any `MIGRATION REFUSED` means **do not start the service**: fix the append (the same commands resume,
step 3), or restore the carrier from `$D` (step 7) and report.

## 5b. The deploy — the FIRST activation of any new reader; NOT executed here; the operator's call — row `activate`

Why this step sits here and not at 2b (PR finding 7): the modeled deploy does not merely
repoint dormant unit files: `gunbc.live_deploy.apply_legs deploy_apply_unit_lifecycle_legs` stages the unit,
runs `daemon-reload`, and **enables and restarts** it — both serving units and every belt member — and
`systemctl restart` starts a stopped unit. A deploy at 2b would have activated the new readers before steps 3
to 5 moved any history: the exact window this runbook exists to keep empty. The order is therefore stage,
migrate, verify, **then** deploy, and the deploy's own restart legs are the activation. The first new reader
that ever serves reads a log that already holds the history, and the old binary never reads a kind it cannot
decode, because it is stopped before the first append and never started again.

**The decision, taken; the prerequisite; the command; and what it does to the store's unit.** The operator
ruled on 2026-11-10, recorded in-model as
`gunbc.design_ruling::t6_fabric_storage_excluded_from_live_deploy_2026_11_10`: the modeled deploy is the route,
and `gunbc-roadmap-fabric-storage` keeps its existing unit fragment and process untouched. Today's model does
not honour that -- `gunbc.live_deploy.spec` names the unit as a member -- so **gunb-ai/gunbc#16256** first takes
it out of the membership in-model (spec, emitter, apply leg, retract leg, role-singleton member), keeping what
the deployment legitimately owns of that endpoint: the tailscale serve mapping to the store's door socket, and
the store's ownership marker. **Step 5b may not be entered until #16256 is merged — it is: `fd458f7`**, because a modeled deploy
run before it would repoint the unit the operator ruled out.

Once it is merged, the deploy is one command, and it is the only route there is (the parent measured that no
hand-written unit edit works, and that plan mode has no live-deploy admission):



What that deploy now does to the store's unit, stated plainly: **nothing.** After #16256 no leg of the apply
plan names `gunbc-roadmap-fabric-storage.service` in its argv at all -- not a stage, an enable, a daemon-reload
or a restart -- which #16256's witness `no_apply_leg_touches_the_fabric_storage_unit` checks over every leg, and
a spec still naming the member refuses the plan rather than acting on a file it no longer installs. The one
fabric leg that remains is the deployment's own tailscale serve mapping, which was repointed by earlier deploys
too. The check below therefore reads the unit's start timestamp as well as its text, so "untouched" is a
measurement and not a promise.


The route, corrected at this head. An earlier draft of this runbook named `gunbc converge --mode deploy`, and
that command does not deploy: `live_deploy_apply_prod_role_operator_local_wet` ends in a typed
`OperatorLocalDeployDissolved` refusal — "the operator-local converge deploy route (gunbc converge --mode
deploy) is retired; the CI dashboard_deploy transaction is the ONE deploy path … Nothing was mutated." Two
admitted doors are named, rather than a third invented:

```bash
# ONE AUTHORIZED INVOCATION, BOUND TO THE EXACT REVISION, BEHIND AN EXECUTABLE GATE — and each stage's run is
# POSITIVELY IDENTIFIED before its receipts are consumed. (1) The gate: five predicates, each refusing before
# GitHub is contacted, and the verifier asked AGAIN on the spot, so a migration verified at 03:00 whose branch
# moved at 03:05 cannot be activated on the 03:00 receipt. # (2) The reference: gh workflow run takes a REF, and a bare commit SHA is not what the CLI documents — so each
# stage is dispatched against its OWN TAG, minted by THIS invocation at this revision with a per-invocation
# nonce, pushed, and verified to resolve to $SHA before anything is dispatched. The ref is what the poll then
# correlates a run TO: see the minting block inside dispatch_stage.
# (3) The runs: three sequential dispatches (plan -> apply -> dashboard_deploy); each waits for a run to
# APPEAR that was not there before, then asserts that run's own declared mode and head_sha through the
# Actions API, then watches it to a successful conclusion. Nothing consumes a receipt from a run that was not
# identified as its own.
test -s "$D/migration.verified"                     || { echo 'NO DEPLOY: step 5 never admitted the read-back'; exit 1; }
grep -q "^SHA=$SHA$" "$D/migration.verified"        || { echo 'NO DEPLOY: the receipt binds another revision'; exit 1; }
grep -q "^READ_BACK_SHA256=$(sha256sum "$D/read-back.out" | cut -d' ' -f1)$" "$D/migration.verified" \
                                                  || { echo 'NO DEPLOY: the read-back document is not the one verified'; exit 1; }
LIVE_TIP=$(git -C "$D/branch" ls-remote origin roadmap-events | cut -f1)
grep -q "^TIP_NOW=$LIVE_TIP$" "$D/migration.verified" || { echo 'NO DEPLOY: the branch moved since verification'; exit 1; }
RUNE "EXPECTED_MANIFEST=$MANIFEST" "EXPECTED_ROSTER=$ROSTER" >> "$D/read-back.driver.log" 2>&1 \
  || { echo 'NO DEPLOY: the live re-verification refused; nothing is activated'; exit 1; }

# THE PINNED REFERENCE. There is no other deploy route: live_deploy_apply_prod_role_operator_local_wet ends in
# a typed OperatorLocalDeployDissolved — "the operator-local converge deploy route (gunbc converge --mode
# deploy) is retired; the CI dashboard_deploy transaction is the ONE deploy path … Nothing was mutated."
REPO=gunb-ai/gunbc
# THE DISPATCH REFERENCE IS PER INVOCATION, NOT PER REVISION. A tag named after the revision (t6-cutover-$SHA)
# identifies WHERE a run is, not WHO dispatched it: two invocations at one revision produce the same name, so
# the name cannot carry authorship. This nonce — 64 bits from the kernel, minted once, before any stage — makes
# the three stage refs names nobody else can produce, and each stage's adopted run must have been dispatched
# from its own ref. The refs are kept as the audit trail (git tag -l 't6-cutover-*' lists them); the runs' own
# head_branch is what GitHub keeps as the identity.
T6_NONCE=$(head -c 8 /dev/urandom | od -An -tx1 | tr -d ' \n')
: "${T6_NONCE:?NO DEPLOY: no per-invocation dispatch reference could be minted; refusing to dispatch runs that could not be attributed}"
[ "${#T6_NONCE}" -eq 16 ] \
  || { echo "NO DEPLOY: the per-invocation dispatch reference is ${#T6_NONCE} hex chars, expected 16 — entropy that is not there is not a nonce"; exit 1; }
T6_REF_BASE=t6-cutover-$SHA-$T6_NONCE
# The exclusive-dispatch fence. A FIXED path, not $TMPDIR: two operator shells have to contend on the same file
# for the fence to be a fence. Held across check → dispatch → watch by dispatch_stage, released on success.
STAGE_LOCK=/tmp/gunbc-t6-cutover-dispatch.lock
git -C /opt/gunbc/gunbc rev-parse --verify "${SHA}^{commit}" >/dev/null
git -C /opt/gunbc/gunbc tag -f "$TAG" "$SHA"
git -C /opt/gunbc/gunbc push -f origin "refs/tags/$TAG"
RESOLVED=$(git -C /opt/gunbc/gunbc ls-remote origin "refs/tags/$TAG" | cut -f1)
[ "$RESOLVED" = "$SHA" ] || { echo "NO DEPLOY: refs/tags/$TAG resolves to $RESOLVED, not $SHA"; exit 1; }
echo "PINNED REF tag=$TAG resolves=$RESOLVED"

# THE TRANSACTION, three dispatches at one reference. plan_artifact_hash goes on BOTH apply and dashboard_deploy:
# gunbc.fleet.fleet_converge_plan_revision::admit_dashboard_deploy_dispatch_inputs refuses an empty
# RLM_PLAN_ARTIFACT_HASH (fleet_converge_plan_revision.dag:413-419), and the workflow fills that variable from
# inputs.plan_artifact_hash (.github/workflows/fleet-converge.yml:3370, 3484), so a deploy dispatch without it
# cannot pass its own admission. expected_revision is NEVER omitted: omitting it admits github.sha, which is
# exactly the unbound activation this step refuses. The input names below are the workflow's own
# workflow_dispatch inputs — there is no free-text `reason` input, so identity comes from the API, not a label.
dispatch_stage() {   # $1 = the mode this stage must declare; remaining args are extra -f inputs
  local want="$1"; shift
  local raw live before run tries got_mode got_sha got_ref new nnew ref resolved
  # AN EXCLUSIVE-DISPATCH BOUNDARY THAT IS ONE. The in-flight check and the dispatch are two GitHub calls, so a
  # second operator shell on this host can slip between them and both pass the check. The fence below is held
  # across the whole of check → dispatch → watch, so the second shell refuses AT THE FENCE instead of racing.
  # `flock -n` REFUSES rather than queueing — queueing would dispatch the instant the first concluded, which is
  # the second run being prevented. A dispatch from ANOTHER host cannot take this fence — that case is answered
  # by the ref minted below (a run is adopted only if GitHub says it ran from the ref THIS invocation created),
  # and secondarily by the poll's uniqueness refusal, where two new runs is a refusal and never a pick-one.
  : "${STAGE_LOCK:?NO DEPLOY: STAGE_LOCK is unset; the exclusive-dispatch fence would have no path}"
  # The fence is held on a FILE DESCRIPTOR, not by a lock file whose presence means "busy": `exec 9>` opens the
  # descriptor, `flock -n 9` claims it, and the kernel drops the claim if this shell dies. A dispatch that
  # crashes therefore leaves NO stale lock behind for the next operator to find and delete — there is nothing
  # here to delete, which is why there is no cleanup path to get wrong.
  exec 9>"$STAGE_LOCK"
  flock -n 9 || { echo "NO DEPLOY: another dispatch for this cutover holds $STAGE_LOCK on this host; refusing to dispatch a second run"; exit 1; }
  # THE IDENTITY IS CREATED HERE, BEFORE ANYTHING IS DISPATCHED, AND EVERY STEP OF ITS CREATION IS CHECKED.
  # Why a ref and not just "the one new run at this SHA in this mode": another host can dispatch the same
  # workflow at the same SHA and mode between this invocation's snapshot and its poll, and that run then agrees
  # on everything the poll checks — mode, head_sha, event — because it genuinely is that revision in that mode.
  # `flock` cannot answer it (it is a file on this host), and "only one new run appeared" cannot answer it (the
  # other host's run can be the only one that has appeared yet). Authorship therefore comes from a name only
  # this invocation can produce: it creates refs/tags/$ref itself, and adopts a run only if GitHub says the run
  # was dispatched from that ref. An existing ref of the same name is a REFUSAL, never a reuse.
  : "${T6_REF_BASE:?NO DEPLOY: the per-invocation dispatch reference was never minted; refusing to dispatch a run that could not be attributed}"
  [ "${T6_REF_BASE#t6-cutover-$SHA-}" != "$T6_REF_BASE" ] \
    || { echo "NO DEPLOY: the dispatch reference [$T6_REF_BASE] is not derived from this revision ($SHA); a ref that does not name the revision it claims cannot attribute a run"; exit 1; }
  ref="$T6_REF_BASE-$want"
  if [ -n "$(git -C /opt/gunbc/gunbc ls-remote origin "refs/tags/$ref")" ]; then
    echo "NO DEPLOY: refs/tags/$ref already exists; this invocation refuses to share a dispatch identity with anyone"; exit 1; fi
  git -C /opt/gunbc/gunbc rev-parse --verify "${SHA}^{commit}" >/dev/null \
    || { echo "NO DEPLOY: $SHA is not a commit in this checkout; nothing is dispatched"; exit 1; }
  git -C /opt/gunbc/gunbc tag "$ref" "$SHA" \
    || { echo "NO DEPLOY: the dispatch ref $ref could not be created locally; nothing was dispatched, so no run is adopted"; exit 1; }
  git -C /opt/gunbc/gunbc push origin "refs/tags/$ref" \
    || { echo "NO DEPLOY: the dispatch ref $ref could not be pushed; nothing was dispatched, so no run is adopted"; exit 1; }
  resolved=$(git -C /opt/gunbc/gunbc ls-remote origin "refs/tags/$ref" | cut -f1)
  [ "$resolved" = "$SHA" ] \
    || { echo "NO DEPLOY: refs/tags/$ref resolves to ${resolved:-nothing}, not $SHA — a run dispatched on it could not be attributed to this revision"; exit 1; }
  echo "PINNED DISPATCH REF stage=$want ref=$ref resolves=$resolved"
  # THE SNAPSHOT IS FAIL-CLOSED, AND IT IS NOT A PIPELINE. `before=$(gh … | sort -u | tr …)` reports the status of
  # TR, which is zero when the GitHub call failed: an empty "before" then makes EVERY pre-existing run at this
  # revision look new, and the poll below adopts somebody else's run — with mode and head_sha both agreeing,
  # because it really is at this revision in this mode. So the GitHub call stands alone, its status is checked,
  # and an empty snapshot is a refusal rather than an empty list. sort/tr cannot mask it any more.
  if ! raw=$(gh run list --repo "$REPO" --workflow fleet-converge.yml --limit 50 --json databaseId -q '.[].databaseId'); then
    echo "NO DEPLOY: the pre-dispatch run snapshot for mode=$want failed; refusing to dispatch blind"; exit 1; fi
  [ -n "$raw" ] || { echo "NO DEPLOY: the pre-dispatch run snapshot for mode=$want is empty; treating every existing run as new is not a correlation"; exit 1; }
  before=$(printf '%s\n' "$raw" | sort -u | tr '\n' ' ')
  # THE EXCLUSIVE BOUNDARY, ENFORCED BEFORE THE DISPATCH. A workflow_dispatch run at this revision that is still
  # queued or in progress is a dispatch this runbook did not make; from that moment the poll cannot tell its run
  # from ours, which is precisely the misidentification being refused. Two dispatches at one revision are refused
  # rather than untangled after the fact.
  # ONE QUERY, NO --status FLAGS. gh documents --status as a single string-valued filter, so passing it twice
  # does not establish that both populations were examined; and queued+in_progress alone leave out a run sitting
  # in requested, waiting, pending or action_required — every one of which is still coming, and would be adopted
  # by the poll below as though this invocation had made it. The population is therefore named IN the query.
  if ! live=$(gh run list --repo "$REPO" --workflow fleet-converge.yml --limit 50 \
                --json databaseId,headSha,event,status \
                -q "[.[] | select(.event==\"workflow_dispatch\" and .headSha==\"$SHA\"
                      and (.status==\"queued\" or .status==\"in_progress\" or .status==\"requested\"
                           or .status==\"waiting\" or .status==\"pending\" or .status==\"action_required\"))
                    | .databaseId] | length"); then
    echo "NO DEPLOY: the in-flight check for mode=$want failed; refusing to dispatch"; exit 1; fi
  [ "${live:-0}" = 0 ] || { echo "NO DEPLOY: $live workflow_dispatch run(s) at $SHA are in a nonterminal state (queued, requested, waiting, pending, in_progress or action_required); refusing to dispatch beside a run this runbook did not create"; exit 1; }
  # THE DISPATCH ITSELF IS CHECKED. A gh that fails here dispatched NOTHING; polling on would find somebody
  # else's run at this revision and mode — head_sha and mode both agreeing, because it really is that revision
  # and that mode — and record it as this stage's run.
  gh workflow run fleet-converge.yml --repo "$REPO" --ref "$ref" -f host=srv1 -f mode="$want" "$@" \
    || { echo "NO DEPLOY: gh workflow run failed for mode=$want; nothing was dispatched, so no run is adopted"; exit 1; }
  run=''; tries=0
  while true; do
    tries=$((tries + 1))
    if [ "$tries" -gt 90 ]; then echo "NO DEPLOY: no run appeared for mode=$want within 15 minutes"; exit 1; fi
    sleep 10
    # THE REF IS PART OF THE QUERY, not just a check afterwards: a run dispatched from another ref is not a
    # candidate at all, so a staggered foreign dispatch at the same SHA and mode never enters the set the
    # uniqueness rule counts. The identity check below then re-reads it from the API, so a stub or an API that
    # ignored the query's filter cannot hand this invocation somebody else's run.
    if ! new=$(gh run list --repo "$REPO" --workflow fleet-converge.yml --limit 50 \
                 --json databaseId,headSha,event,headBranch \
                 -q "[.[] | select(.headSha==\"$SHA\" and .event==\"workflow_dispatch\" and .headBranch==\"$ref\") | .databaseId] - [$(printf '%s,' $before | sed 's/,$//')] | .[]"); then
      echo "NO DEPLOY: the poll for mode=$want failed"; exit 1; fi
    nnew=$(printf '%s\n' "$new" | grep -c . || true)
    # UNIQUENESS, NOT OPTIMISM. One new run is a correlation; two is a race, and taking the first of them proves
    # nothing about which this invocation created — so the ambiguous case stops the transaction instead of
    # guessing, and the operator retries once the other dispatch has concluded.
    if [ "${nnew:-0}" -gt 1 ]; then
      echo "NO DEPLOY: $nnew runs appeared for mode=$want at $SHA; two dispatches are racing, and the first of them is not evidence of authorship"; exit 1; fi
    [ "${nnew:-0}" -eq 1 ] || continue
    run=$(printf '%s\n' "$new" | head -1)
    break
  done
  got_mode=$(gh api "repos/$REPO/actions/runs/$run" -q '.inputs.mode // ""')
  got_sha=$(gh api "repos/$REPO/actions/runs/$run" -q '.head_sha')
  got_ref=$(gh api "repos/$REPO/actions/runs/$run" -q '.head_branch // ""')
  [ "$got_mode" = "$want" ] || { echo "NO DEPLOY: run $run declares mode=$got_mode, expected $want"; exit 1; }
  [ "$got_sha" = "$SHA" ]   || { echo "NO DEPLOY: run $run is at $got_sha, expected $SHA"; exit 1; }
  # AUTHORSHIP, NOT COINCIDENCE. Mode and revision say a run is at the right place in the right mode; only the
  # ref says it was dispatched by THIS invocation. A run that arrived from any other ref — another host, another
  # operator shell, a re-run of somebody else's dispatch — is refused here even if everything else agrees.
  [ "$got_ref" = "$ref" ]   || { echo "NO DEPLOY: run $run ran from ref [$got_ref], not the ref this invocation created ($ref); it is not this dispatch's run"; exit 1; }
  gh run watch "$run" --repo "$REPO" --exit-status \
    || { echo "NO DEPLOY: the $want run ($run) did not complete successfully"; exit 1; }
  STAGE_RUN="$run"
  exec 9>&-   # the transaction is closed and positively identified; the next stage may take the fence
  echo "STAGE $want run=$STAGE_RUN mode=$got_mode revision=$got_sha ref=$ref"
}

dispatch_stage plan
PLAN_RUN="$STAGE_RUN"
# Safe to read the log only because dispatch_stage has ALREADY positively identified that run and watched it to a
# successful conclusion; before this the reviewer's finding was consuming logs of a run nobody had identified.
PLAN_HASH=$(gh run view "$PLAN_RUN" --repo "$REPO" --log | sed -n 's/.*plan_artifact_hash=\([0-9a-f]*\).*/\1/p' | tail -1)
test -n "$PLAN_HASH" || { echo 'NO DEPLOY: the identified plan run published no plan_artifact_hash'; exit 1; }

dispatch_stage apply -f plan_artifact_hash="$PLAN_HASH" -f plan_workflow_run_id="$PLAN_RUN"
APPLY_RUN="$STAGE_RUN"

dispatch_stage dashboard_deploy -f plan_artifact_hash="$PLAN_HASH" -f plan_workflow_run_id="$PLAN_RUN" \
                                 -f apply_workflow_run_id="$APPLY_RUN" -f expected_revision="$SHA"
DEPLOY_RUN="$STAGE_RUN"
echo "DEPLOYED ref_base=$T6_REF_BASE plan=$PLAN_RUN apply=$APPLY_RUN deploy=$DEPLOY_RUN plan_artifact_hash=$PLAN_HASH"
```

**Which receipt admits what, precisely** — the two admissions are different and neither is a formality. The
CI transaction admits the *deployment's* own receipts: `gunbc.live_deploy.apply_receipt
assert_dashboard_deploy_receipts` reads the run's receipt directory and refuses a receipt-less success
(`AssertZeroReceipts`), refuses two receipts (`AssertMultipleReceipts`), and refuses a receipt whose
conclusion does not match the job's own. The *migration* receipt (`$D/migration.verified`) is what authorizes
a human to dispatch that transaction at all — the modeled route takes no input for it, which is why the gate
above is written as shell predicates rather than left to judgment, and why the file itself is kept and quoted
in the PR. `ci_deploy_srv1_access` binds the principal `ghrunner`, so an ordinary session that runs the same
fold refuses **before any mutation** with `deploy_access: principal mismatch expected=ghrunner actual=briansrls`
— the access model working, not an obstacle to route around: reaching the fold means a dispatched job on the
host-pinned runner, which is what `gh workflow run` does. Either way it is entered only from a step 5 that
exited 0, and it brings up both serving units, the three belt oneshots and the timer. **It is not run in this
PR** — the operator's call, recorded as such.

| measured fact | evidence |
| --- | --- |
| the five units cannot be moved onto a main binary by editing unit files | the parent's side-by-side attempt
at main `5785188` (14:11 UTC, rolled back 14:21, old service answering 14:32) failed **both** serving units with
`unrecognized subcommand 'serve'` at argv parse — `main` has no `serve`; the command lines belong to
`gunbc.live_deploy.emit`. Left behind: an unused checkout `/opt/gunbc/releases/57851881501f…` and binary
`/opt/gunbc/bin/gunbc-57851881501f…` |
| there is no preview of that deploy | `gunbc converge --host srv1 --mode plan --scope scope:live-deploy` refuses `RequestUnobserved` after 35 min — plan mode does not accept live-deploy. `--mode deploy` goes straight to `gunbc.live_deploy.apply live_deploy_apply_prod_role_operator_local_wet`, whose host-memory admission needs a capped cgroup (`MemoryMax=32G MemorySwapMax=0`) |
| a modeled deploy repoints `gunbc-roadmap-fabric-storage.service` too | main's deployment spec names it as a member: `gunbc.live_deploy.spec deployment_fabric_storage_unit_name` |
| the belt units will not start as things stand | they refuse `UncappedSystemSlice`: `gunbc-roadmap.slice` (`live_deploy_slice_memory_max`) is not among `gunbc.live_deploy.slice_bounds` `declared_system_slice_memory_maxes`. child `nimble-newt-297` is fixing that census; **until it lands no branch ends with the belts running** |
| the live page has returned 503 since 2026-11-08 | three `retired` events (reason `test`) the running Oct-15 build cannot decode. Checked by me at that revision: `roadmap_event_log.dag` there has **zero** mentions of `Retired`; the kind arrived in `698e5f5caece` (2026-11-05, #15856). The page's fix is a binary that understands these kinds — the class of event this runbook writes |
| `gunbc converge --mode deploy` is not a deploy route at all | `gunbc.live_deploy.apply operator_local_deploy_dissolved_refusal`: the operator-local converge deploy route is retired — "the CI dashboard_deploy transaction is the ONE deploy path" — and it mutates nothing. The two admitted doors are below |

Read the units after any deploy rather than trusting it — the only command lines this runbook verifies, and
none is ever written by it:

```bash
for U in gunbc-roadmap.service gunbc-roadmap-tailnet-door.service \
         gunbc-roadmap-belt-discovery.service gunbc-roadmap-belt-exit.service \
         gunbc-roadmap-belt-credential-recovery.service; do
  echo "== $U"; systemctl show $U -p WorkingDirectory -p ExecStart -p ExecMainStartTimestamp -p MainPID | head -5
done
# expect WorkingDirectory=/opt/gunbc/releases/$SHA, argv[0] = /opt/gunbc/bin/gunbc-$SHA — EMITTED BY THE
# MODEL, not copied out of this runbook
systemctl show gunbc-roadmap-fabric-storage.service -p ExecStart -p FragmentPath -p MemoryMax \
  -p MainPID -p ExecMainStartTimestamp | diff - $D/fabric-storage-unit.before
# expect NO output: the operator's ruling is that this unit is left alone, and the prerequisite PR is what
# makes the modeled deploy honour that. A difference here means the prerequisite is not in effect: stop.
```


What the runbook exists to keep is one invariant: **the migration and the retirements are written only once the
binary that decodes these event kinds is the one that will read them.** Hence steps 3-6 sit behind this section,
step 5 gates activation on `MIGRATION VERIFIED`, and step 7 never starts a service against an unmigrated log —
while the appends themselves need no service up at all (see "Does any of this need the fabric-storage unit to
run new code?").

The three test events are on origin's `roadmap-events` **branch**, which is what the reader decodes (step 1
exports it at an exact tip; srv1's `/opt/gunbc/roadmap-events` is only the holder and is stale at `02c95a0`).

**And nothing here is hand-repaired.** If a read-back differs from what the export recorded, the run stops:
the unit text, the fragment path and the ExecStart are the deployment model's output, so a difference is a
fact about the model or about #16256 not being in effect — it is reported, with the export path
(`srv1:/tmp/t6-live2.tar.gz` and `$D`), and never patched by writing a unit file by hand.


## 6. Read back through the reader production uses — step 5b's deploy is what brought the group up — row `observe`

The last thing a user stands on is the HTTP page, and its own read-back is the one the parent already knows
how to take: the live roadmap page answered **503 since 2026-11-08** because the deployed build cannot decode
the three `retired` events on origin — after step 5b it must answer 200 with the migrated accepted work
visible. Its URL and port belong to the deployment model (the dashboard serve mapping in
`gunbc.live_deploy.spec`), not to this runbook, so the observation is taken from there and recorded beside the
export rather than being hardcoded here.

Then the same three facts entered DIRECTLY, as exits — the doors this runbook used to name as
`roadmap_retired_node_ids_load`, `roadmap_acceptance_event_history_load` and
`roadmap_event_standing_report`, which the runner cannot enter at all: `gunbc run` requires a `ProcessExit`
(`src/v1/stage0/src/cli_run.rs:17584`, "function returned `X`, not ProcessExit") and those three return DATA —
their return types are exactly what `gunbc.roadmap.roadmap_page`, `roadmap_forecast`, `roadmap_runtime_plan`,
`roadmap_acceptance_inbox` and the instruments read. Narrowing a production authority to an exit shape to make
it runnable would bend the reader to the runner, so `gunbc.roadmap.roadmap_event_cli` stands three entries
beside them that decide nothing and only report:

```bash
# exit 0 = the fail-closed retired read ADMITTED the set (a log it cannot read exits 1 naming the reason, and
# is never served as an empty retired set); the reason the runtime prints IS the diagnostic.
RUN "$BIN run --source-root $TREE/dag --source-root $TREE/src/v2 --entry dag/gunbc/roadmap/roadmap_event_cli.dag --function roadmap_event_log_retired_report_cli"
# exit 0 = the one decoder of the one home admitted the acceptance history.
RUN "$BIN run --source-root $TREE/dag --source-root $TREE/src/v2 --entry dag/gunbc/roadmap/roadmap_event_cli.dag --function roadmap_event_log_acceptance_report_cli"
# exit 0 = verifier + both readers + the write all admitted, in one document (step 5's artifact, regenerated).
# The manifest and the roster are DATA and travel in the ENVIRONMENT through RUNE, exactly as in step 5. An
# earlier pass put them INSIDE the command string (`--arg expected_manifest=\"$MANIFEST\"`) that RUN hands to
# `bash -lc`, and the shell systemd-run spawns then parsed the JSON's own quotes as SHELL: the reviewer
# reproduced an object arriving as several arguments with its quotes stripped, so the read-back would have
# certified less than the old carriers held — and a receipt line carrying `$(…)` would have been INTERPRETED,
# which is the same class as the export finding. Same data, same route, one argument, byte-for-byte.
CMD="$BIN run --source-root $TREE/dag --source-root $TREE/src/v2 --entry dag/gunbc/roadmap/roadmap_event_cli.dag --function roadmap_event_log_read_back_cli --arg repo_root=/opt/gunbc/gunbc --arg expected_manifest=\"\$EXPECTED_MANIFEST\" --arg retired_roster=\"\$EXPECTED_ROSTER\" --arg report_path=$D/read-back.after.out"
# An empty manifest would certify nothing, so the variables step 5 set are required to still be set here; an
# environment that lost them is a refusal, not a read-back over zero expected lines.
: "${MANIFEST:?OBSERVATION REFUSED: EXPECTED_MANIFEST is unset — re-run step 5 in this shell}"
: "${ROSTER:?OBSERVATION REFUSED: EXPECTED_ROSTER is unset — re-run step 5 in this shell}"
RUNE "EXPECTED_MANIFEST=$MANIFEST" "EXPECTED_ROSTER=$ROSTER" > "$D/read-back.after.driver.log" 2>&1
READ_AFTER_RC=$?
[ "$READ_AFTER_RC" -eq 0 ] || { echo "OBSERVATION REFUSED: the post-deploy read-back exited $READ_AFTER_RC — the page is NOT accepted as migrated; do not close the cutover, and step 7 is the rollback"; exit 1; }
cat "$D/read-back.after.out"
# and the per-node standing, as the log's own standing renderer words it:
grep -c 'retired by the event log' "$D/read-back.after.out"   # expect 2 (both roster nodes)
```

One thing the doors deliberately do NOT need: a row in `gunbc.auth.privileged_effect_census`. That census
lists sites whose effect is a credential read, a fleet or host mutation, a deploy or a grant (its one
roadmap-adjacent row is `branch_run_grant_apply_wet`, the wet apply, not the file it writes). These entries read
one log through the readers the page already uses and write one report document to a path the operator names —
the same class as `extdeps.tools.sha256sum` writing its checksum file or the grant writing its output, neither of
which is censused. Nothing here mints, reads or moves authority.

The claim population that holds these doors honest, at this head:
`a_retired_verdict_maps_to_the_exit_the_log_earned`,
`an_acceptance_history_verdict_maps_to_the_exit_the_log_earned`,
`the_migration_verdict_admits_only_verified_unrefused_text`, and
`the_read_back_refuses_when_any_single_part_refuses` (each in
`dag/test/claim/roadmap/roadmap_receipt_continuity_acceptance.dag`). They are pure-verified here because no
throwaway fixture can be a dashboard instance (`dashboard_instance_for_repo_root` folds
`public_builtin_dashboard_instances`, and a fixture is not registered in it) — the positive path is what
steps 5 and 6 measure on the registered instance, and the receipts in this PR are that measurement.

## 7. Rollback, if verification refuses — row `rollback`

One script, and **every check is an admission predicate**: the first that fails exits 1 and leaves every reader
stopped. The first pass found the rollback starting services after a check that only printed (PR finding 12); the
next pass found the same class surviving in the revised step 7 — the tip comparison, the five unit-definition
comparisons, the release identity and the five `is-active` observations were all `echo` diagnostics, and
`systemctl start` ran unconditionally afterwards. That shape is gone: `need` refuses, and nothing past a
refusal executes. It travels through `RUNF` intact, with `$D` and `$SHA` as data.

**Two dispositions, named apart, because an unchanged tip still does not prove the old reader healthy.** The
exported remote tip already carries three `retired` events the deployed Oct-15 build cannot decode — that is
the live 503 this PR exists to clear. So:

| disposition | when it is the one | what it means, said plainly |
| --- | --- | --- |
| **7A — restore the old reader** | only when origin's tip is **unchanged** (no append ever landed) | a **DEGRADED** rollback, and it must be reported as degraded: the page will still 503 on those three events. It is a stopped-services-plus-known-defect state, not a healthy restoration, and it is never recorded as one |
| **7B — roll forward to the staged build** | the default as soon as **any** append landed | `$BIN`/`$TREE` from step 2b decode every event kind. If the migration itself is untrusted, point the append at the exported pre-append tip (`$D/branch`) and re-run steps 3-5 rather than resurrecting a binary that cannot read the log. Activation is the modeled deploy (step 5b), because that fold — not a hand edit — is what emits these unit definitions |

```bash
RUNF "$D" "$SHA" <<'REMOTE'
set -euo pipefail
D=$1; SHA=$2
# ARGUMENT-DELIVERY GUARDS, the same class step 1 carries: $1 must be the absolute export directory and $2 the
# 40-hex revision this rollback is about. A shifted delivery refuses rather than reading the wrong export.
case "$D" in
  /*) : ;;
  *) echo "ROLLBACK REFUSED: the export directory arrived as [$D] — argument delivery shifted"; exit 1 ;;
esac
case "$SHA" in
  ''|*[!0-9a-f]*) echo "ROLLBACK REFUSED: the revision arrived as [$SHA], which is not 40 hex"; exit 1 ;;
esac
[ "${#SHA}" -eq 40 ] || { echo "ROLLBACK REFUSED: the revision arrived as [$SHA], which is not 40 hex"; exit 1; }
UNITS=(gunbc-roadmap.service gunbc-roadmap-tailnet-door.service gunbc-roadmap-belt-discovery.service gunbc-roadmap-belt-exit.service gunbc-roadmap-belt-credential-recovery.service)
# The counterpart of step 1's export_drop_ins: each unit's drop-in directory is removed and put back AT ITS OWN
# PATH (cp -a of the directory, not of its contents — copying contents would land override.conf directly under
# /etc/systemd/system, where systemd would not read it as that unit's drop-in).
restore_drop_ins() { # $1 = the systemd directory, $2 = the export directory
  for U in "${UNITS[@]}"; do
    sudo rm -rf "$1/$U.d"
    if [ -d "$2/units/drop-ins/$U.d" ]; then sudo cp -a "$2/units/drop-ins/$U.d" "$1/"; fi
  done
}
need() { local what="$1"; shift; if ! "$@"; then echo "ROLLBACK REFUSED: $what — nothing is started"; exit 1; fi; }

# PREDICATE 1 — THE AUTHORITY IS ORIGIN'S BRANCH, MEASURED REMOTELY. srv1's /opt/gunbc/roadmap-events is only
# the holder, and the read/append route fetches origin and hard-resets the private snapshot to what it fetched
# (roadmap_event_carrier.dag:200-212), so restoring the holder rolls back NOTHING and can even reproduce the
# 503 by handing the old binary nodes it cannot decode. The tip that decides is the one GitHub holds.
TIP=$(git -C "$D/branch" ls-remote origin roadmap-events | cut -f1)
test -n "$TIP" || { echo 'ROLLBACK REFUSED: origin has no roadmap-events tip — the carrier is in no state to reason about'; exit 1; }
if [ "$TIP" = "$(cat "$D/roadmap-events.tip.before")" ]; then
  DISPOSITION=7A; echo 'tip unchanged: no append landed, so 7A (DEGRADED restore of the old reader) is admissible'
else
  DISPOSITION=7B; echo 'the branch moved: the old reader is NOT admissible against this log; 7B only'
fi

# PREDICATE 2 — THE EFFECTIVE DEFINITIONS ARE RESTORED AND COMPARED BEFORE ANYTHING IS STARTED. Deleting one
# named drop-in is not restoring a unit file a modeled deploy rewrote, so the exported files go back first,
# then the EFFECTIVE catalog (systemctl cat, drop-ins included) is compared byte for byte against the export,
# for all five units — and then again against the exported files themselves.
for f in "$D"/units/*; do
  [ -f "$f" ] || continue
  sudo install -m 0644 -o root -g root "$f" "/etc/systemd/system/$(basename "$f")"
done
restore_drop_ins /etc/systemd/system "$D"
sudo systemctl daemon-reload
for U in "${UNITS[@]}"; do systemctl cat "$U"; done > /tmp/t6-units.now.catalog
need 'the five effective unit definitions are byte-identical to the export' \
  diff -u "$D/units.catalog" /tmp/t6-units.now.catalog
need 'the live unit inventory is the exported one' \
  diff -u "$D/units.inventory" <(ls "$D/units")

# PREDICATE 3 — THE RELEASE IDENTITY THE CATALOG NAMES EXISTS. (Identity is read from the definitions being
# restored, never asserted from a path this runbook invented.)
if [ "$DISPOSITION" = 7B ]; then
  need 'the staged binary exists'   test -x "/opt/gunbc/bin/gunbc-$SHA"
  need 'the staged release tree exists' test -d "/opt/gunbc/releases/$SHA"
else
  OLD=$(sed -n 's/.*ExecStart=\([^ ]*\).*/\1/p' "$D/units.catalog" | sort -u | head -1)
  need "the old reader the exported definitions point at still exists ($OLD)" test -x "$OLD"
fi

# PREDICATE 4 — NOTHING IS RUNNING WHEN THE DECISION IS MADE. All five must be inactive; a unit that is active
# here was started by something other than this runbook, and the run stops rather than guessing which.
for U in "${UNITS[@]}"; do
  need "$U is not already active" test "$(systemctl is-active "$U")" = inactive
done

# ONLY NOW, WITH EVERY PREDICATE HELD, is anything started — and only what step 2 FOUND active, read from the
# export rather than from a list in this runbook, so a unit that was already down is not started by us. The export's
# lines are ALREADY `<unit> <state>` (step 1: `printf '%s %s\n' "$U" "$(systemctl is-active "$U")"`), so they are
# read DIRECTLY. The first version of this block also pasted a second unit list in front of them, which made each
# line `<unit> <unit> <state>`; `read -r U STATE` then put `<unit> active` in STATE, never equalled `active`, the
# loop started NOTHING, and the `came up` predicate below failed with the readers down — a rollback that reports a
# failure instead of performing it. Both directions are now witnessed (tools/roadmap_cutover_runbook_witness.sh
# PART D): the units recorded active really receive `systemctl start`, the ones recorded inactive really do not,
# and an export that names a unit off the roster or omits one on it refuses.
sudo systemctl reset-failed "${UNITS[@]}" || true
if [ "$DISPOSITION" = 7B ]; then
  echo 'ROLLBACK 7B: re-enter step 5b — the modeled deploy is the only route that emits the unit definitions it starts.'
  echo 'Nothing was started here.'
else
  # No file, or an empty one, is no measurement at all: restoring "nothing" from an export that was never
  # written is exactly how readers stay down while the runbook prints a completion line.
  [ -s "$D/units.active.before" ] \
    || { echo "ROLLBACK REFUSED: $D/units.active.before is missing or empty — no measurement of what was running"; exit 1; }
  # THE EXPORT IS ADMITTED AS AN INVENTORY BEFORE ANYTHING IS STARTED. Not merely "two nonempty words":
  # `read -r U STATE` accepts `gunbc-roadmap.service gunbc-roadmap.service active` and binds STATE to
  # "gunbc-roadmap.service active", which is not "active" — so the malformed inventory was silently read as
  # "nothing to restore", the same silent-nonrestoration failure the whole of 7A exists to prevent, arriving
  # through the input instead of the loop. So: exactly two fields per record, the second one a state systemd
  # actually reports, the first one an admitted unit, one record per unit, and every admitted unit present.
  # (PART D of tools/roadmap_cutover_runbook_witness.sh refuses this input and every other shape defect here.)
  awk -v roster="${UNITS[*]}" '
    BEGIN { n = split(roster, r, " "); for (i = 1; i <= n; i++) want[r[i]] = 0 }
    NF != 2 { print "  line " NR ": expected exactly two fields (unit state), found " NF; bad = 1; next }
    $2 !~ /^(active|inactive|failed|activating|deactivating|reloading|maintenance)$/ {
      print "  line " NR ": state [" $2 "] is not a state systemd reports"; bad = 1 }
    !($1 in want) { print "  line " NR ": unit [" $1 "] is not in the admitted five-unit roster"; bad = 1 }
    ($1 in want) && want[$1]++ { print "  line " NR ": unit [" $1 "] is recorded more than once"; bad = 1 }
    END { for (u in want) if (want[u] == 0) { print "  the export has no line for " u "; refusing to guess whether it was running"; bad = 1 }
          exit (bad ? 1 : 0) }' "$D/units.active.before" \
    || { echo "ROLLBACK REFUSED: $D/units.active.before is not a valid inventory — nothing is started"; exit 1; }
  RESTORED=0
  while read -r U STATE; do
    [ "$STATE" = active ] || continue
    sudo systemctl start "$U"
    RESTORED=$((RESTORED + 1))
  done < "$D/units.active.before"
  echo "RESTORED $RESTORED unit(s) that step 2 found active (of ${#UNITS[@]} admitted)"
  # The inverse of the measurement, unit for unit: everything found active must be up, and nothing found down is
  # required to be up — a unit the operator had already stopped is not started by us and not judged by us either.
  while read -r U STATE; do
    [ "$STATE" = active ] || continue
    need "$U came up after the restore" test "$(systemctl is-active "$U")" = active
  done < "$D/units.active.before"
  echo 'ROLLBACK 7A COMPLETE — DEGRADED: the old reader cannot decode the three retired events already on'
  echo 'origin, so the page still answers 503 until a build that decodes them is deployed. Report it as such.'
fi
REMOTE
```

What rollback may **never** do, in either disposition: hand-write a unit file, an ExecStart or a fragment (the
definitions come from `gunbc.live_deploy.emit` and the export, and a hand-written one diverges the model from
srv1 and would be deployed away by the next modeled run); edit anything under `/opt/gunbc/fabric-storage` or
any fabric partition; or restore the deleted JSONL carrier as a second home for the acceptance history — it is
deleted, its importer and builtin are deleted, and re-adding it would re-open the fact this PR closes. The
event log's own history is the rollback record: nodes are never deleted, so an unwanted event is answered by a
**revocation** through `roadmap_acceptance_revoke_cli`, which names a live receipt or refuses.


## Validation of this runbook's own shell

A runbook is executable text: a shifted argument, a variable expanded in the wrong shell, a check that only
prints, or a rollback that restarts nothing are defects CI cannot see, because no CI job runs this file. Two
witnesses live beside it, and they extract the helpers, guard blocks, the rollback's restore leg and
`dispatch_stage` FROM THIS FILE at run time (asserting byte-identity as they go), so they cannot drift into
testing a paraphrase. Parts A-E cover argument delivery, the drop-in round trip, the model/heading join, the
rollback and the read-back; PART F is the stop of the readers and PART G the migration loops:

```bash
bash tools/roadmap_cutover_runbook_witness.sh  docs/runbooks/roadmap-event-log-cutover.md
bash tools/roadmap_cutover_dispatch_witness.sh docs/runbooks/roadmap-event-log-cutover.md
```

Nothing in them touches a real system: `sudo` becomes a function that strips the privilege, and `systemd-run`,
`systemctl`, `gunbc` and `gh` resolve to stubs on `PATH`; the `systemctl` stub in PART D is STATEFUL, because an
`is-active` that always answered `active` would let a rollback that started nothing pass. Every case below is an
assertion — the exit status AND the diagnostic are required, and a case that disagrees with its own expectation
fails the witness. They are run before the migration is authorized, not after it fails. Their output at this
revision, verbatim:
### A — argument delivery through the actual helper invocation

```text
================ PART A: argument delivery through the ACTUAL helper shape ================
--- A1: correct helper, one data argument (step 1 passes $D) ---
    GUARD HELD: D=/opt/gunbc/exports/t6-20261110T230000Z
--- A2: the SHIFTED helper (a filler word after --), same script ---
    RUNF REFUSED: the export directory arrived as [runbook] — argument delivery shifted
--- A3: correct helper, two data arguments (step 7 passes $D then $SHA) ---
    GUARD HELD: D=/opt/gunbc/exports/t6-20261110T230000Z SHA=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
--- A4: the same script through the SHIFTED helper — the guard must refuse ---
    ROLLBACK REFUSED: the export directory arrived as [runbook] — argument delivery shifted
--- A5: a revision that is not 40 hex is refused by step 7's own guard ---
    ROLLBACK REFUSED: the revision arrived as [not-a-sha], which is not 40 hex
    FAIL A2: the SHIFTED helper was NOT refused — a shifted delivery would export into a relative directory named 'runbook'
    FAIL A2: the refusal did not name the defect (expected the runbook's own 'argument delivery shifted')
    FAIL PART I: a hand-shell LOC row no longer describes its carrier — re-measure it and say so
    A6 OK: a neutralised guard makes the witness itself exit 3
```


A6 is the assertion about the assertions: the whole witness is re-run against a copy of this file whose step-1
argument guard has been neutralised, and the witness must fail. If it did not, the A-series would be narration.
### B — the drop-in round trip, pre-existing AND deployment-added

```text
================ PART B: the drop-in round trip, pre-existing AND deployment-added ================
--- B1: export (step 1's helper) ---
units/drop-ins/gunbc-roadmap.service.d/pre-existing.conf
units/drop-ins/gunbc-roadmap-tailnet-door.service.d/10-deploy.conf
--- B2: the modeled deploy rewrites a unit file and adds a drop-in under the first unit ---
--- B3: restore (step 7's helper) ---
--- B4: assertions ---
ROLLBACK REFUSED: an unsatisfied predicate — nothing is started
ROUND TRIP OK: every unit's drop-in directory returned to its own path, byte-identical; the
               deployment-added one is gone; the pre-cutover ones (on both units) are back.
```
### C — the modelled steps and this book’s headings are the same nine steps

```text
================ PART C: the runbook's headings and the model's rows join both ways ================
model rows: 9   runbook headings citing a row: 9   (model: /home/briansrls/.worktrees/gunbc/jolly-crab-297/dag/gunbc/witness/roadmap_cutover.dag)
C1 OK: every modelled step is cited by exactly one heading, and every citation names a modelled step
```
### D — the rollback starts exactly what the export measured

```text
================ PART D: the rollback starts exactly the units the export found active ================
--- D1: two of five recorded active, DISPOSITION 7A — those two and only those two must come up ---
    RESTORED 2 unit(s) that step 2 found active (of 5 admitted)
    ROLLBACK 7A COMPLETE — DEGRADED: the old reader cannot decode the three retired events already on
    origin, so the page still answers 503 until a build that decodes them is deployed. Report it as such.
--- D2: the export names a unit this cutover does not own — refuse, and start nothing ---
      line 2: unit [not-our-unit.service] is not in the admitted five-unit roster
      the export has no line for gunbc-roadmap-belt-discovery.service; refusing to guess whether it was running
      the export has no line for gunbc-roadmap-tailnet-door.service; refusing to guess whether it was running
      the export has no line for gunbc-roadmap-belt-credential-recovery.service; refusing to guess whether it was running
      the export has no line for gunbc-roadmap-belt-exit.service; refusing to guess whether it was running
    ROLLBACK REFUSED: $TMP/rollback-export/units.active.before is not a valid inventory — nothing is started
--- D3: a roster unit has no line in the export — refusing to guess that it was down ---
      the export has no line for gunbc-roadmap-belt-credential-recovery.service; refusing to guess whether it was running
    ROLLBACK REFUSED: $TMP/rollback-export/units.active.before is not a valid inventory — nothing is started
--- D4: an empty export — no measurement, no rollback start ---
    ROLLBACK REFUSED: $TMP/rollback-export/units.active.before is missing or empty — no measurement of what was running
--- D5: the fixture must be able to see the defect it exists for (the pasted-list shape) ---
      line 1: expected exactly two fields (unit state), found 3
      line 2: expected exactly two fields (unit state), found 3
      line 3: expected exactly two fields (unit state), found 3
      line 4: expected exactly two fields (unit state), found 3
      line 5: expected exactly two fields (unit state), found 3
      the export has no line for gunbc-roadmap-belt-discovery.service; refusing to guess whether it was running
      the export has no line for gunbc-roadmap-tailnet-door.service; refusing to guess whether it was running
      the export has no line for gunbc-roadmap-belt-credential-recovery.service; refusing to guess whether it was running
      the export has no line for gunbc-roadmap-belt-exit.service; refusing to guess whether it was running
      the export has no line for gunbc-roadmap.service; refusing to guess whether it was running
    ROLLBACK REFUSED: $TMP/rollback-export/units.active.before is not a valid inventory — nothing is started
    D5 OK: the pasted shape is REFUSED as an invalid inventory, before anything is started
--- D6: a unit recorded twice is refused ---
      line 6: unit [gunbc-roadmap.service] is recorded more than once
    ROLLBACK REFUSED: $TMP/rollback-export/units.active.before is not a valid inventory — nothing is started
--- D7: a word that is not a state systemd reports is refused ---
      line 1: state [running] is not a state systemd reports
    ROLLBACK REFUSED: $TMP/rollback-export/units.active.before is not a valid inventory — nothing is started
```


D5 is the case that keeps D1 honest: the same export in the shape the defective rollback READ it (a second unit
list pasted in front of the states, so `read -r U STATE` bound `<unit> active` to STATE and nothing ever equalled
`active`). Zero starts is NOT an acceptable answer there — `RESTORED 0 unit(s)` after a rollback that restored
nothing reads as a completed rollback — so D5 requires a REFUSAL naming the malformed record, and D6/D7 do the same
for a unit recorded twice and for a word that is not a state `systemctl is-active` can print.
### E — the expected manifest travels as data, never as shell

```text
================ PART E: the manifest arrives as DATA, not as shell ================
loaded from the runbook: RUNE, the environment-passing helper
loaded from the runbook: RUN, the plain transient-unit helper
--- E1: RUNE delivers the manifest byte-for-byte, and none of it is in argv ---
--- E2: the unsafe interpolated shape must be CAUGHT corrupting it (the fixture's own eyes) ---
    E2 OK: SHAPE REFUSED — the interpolated shape never even ran; the spliced JSON left the inner shell with a syntax error (rc=2). That is the strongest form of the finding: the old step 6 could have failed to certify anything at all
```


E2 is what makes E1 worth reading: it rebuilds the shape step 6 used to have — the manifest spliced INTO the
command string — and requires it to arrive altered. `$(cat …)` written inside the string would NOT reproduce the
defect (a substitution the inner shell performs stays data); it is the outer splice that turns data back into
syntax, which is why the fix is the environment (`RUNE`), not a rewording.
### F — step 2: the readers are stopped, and the stop is admitted by measurement

```text
================ PART F: stopping the readers is a measured admission, not a printed hope ================
  [every unit and the timer inactive, no process] rc=0
    STOP ADMITTED: 5 units and the timer report inactive, no gunbc process remains
  [one reader still active after the stop] rc=1
    STOP REFUSED: gunbc-roadmap-tailnet-door.service reports [active], not inactive — no migration write may follow
  [the timer still firing after the stop] rc=1
    STOP REFUSED: gunbc-roadmap-belt-credential-recovery.timer reports [active], not inactive — no migration write may follow
  [one stop command fails] rc=1
    stub: gunbc-roadmap-belt-exit.service refuses to stop
    STOP REFUSED: gunbc-roadmap-belt-exit.service did not stop
  [the timer refuses to be unloaded] rc=1
    stub: gunbc-roadmap-belt-credential-recovery.timer refuses to stop
    STOP REFUSED: the belt-credential-recovery timer is still loaded
  [a gunbc process is still running] rc=1
    STOP REFUSED: 1 gunbc process(es) still running
  [F7 the two rosters are the same five units] OK: gunbc-roadmap-belt-credential-recovery.service gunbc-roadmap-belt-discovery.service gunbc-roadmap-belt-exit.service gunbc-roadmap-tailnet-door.service
```


The stop used to end in three lines of `# expect: inactive`. A comment is not an admission: an unobserved stop
leaves a reader answering from the old carrier for the whole migration window. F1 requires the five readers AND the
timer to have been stopped (six acts, not five — the timer drives two oneshots), and F2-F6 require a refusal for
each way the world can disagree: a reader that resists the stop, a timer still firing, a failed stop command, a
timer that will not unload, and a gunbc process still running. F7 joins the five-unit roster this book measures
against the five-unit roster step 2 admits, so the two lists cannot drift apart silently.
### G — steps 3 and 4: one refused append ends the migration

```text
================ PART G: a refused append stops the migration; the input is counted before it ================
loaded from the runbook: RUNE
  [the export as measured: 17 seed acceptances, empty fabric tail] rc=0 appends=18
    INPUT ADMITTED: 17 acceptance lines (17 from the seed, 0 from the fabric tail) and 1 revocation
    appended acceptance 1/17
    appended acceptance 2/17
    appended acceptance 3/17
    appended acceptance 4/17
    appended acceptance 5/17
    appended acceptance 6/17
    appended acceptance 7/17
    appended acceptance 8/17
    appended acceptance 9/17
    appended acceptance 10/17
    appended acceptance 11/17
    appended acceptance 12/17
    appended acceptance 13/17
    appended acceptance 14/17
    appended acceptance 15/17
    appended acceptance 16/17
    appended acceptance 17/17
    appended revocation 1/1
  [the third append is refused by the modeled CLI] rc=1 appends=3
    INPUT ADMITTED: 17 acceptance lines (17 from the seed, 0 from the fabric tail) and 1 revocation
    appended acceptance 1/17
    appended acceptance 2/17
    stub gunbc: this receipt is refused by the modeled CLI (append 3)
    MIGRATION REFUSED: acceptance 3 of 17 did not append — no further appends, and step 4 must not run
  [the seed carries 16 acceptances where the export says 17] rc=1 appends=0
    MIGRATION REFUSED: the seed holds 17 lines where the export measured 18 — the file changed after step 1
  [a fabric object carries no payload.line] rc=1 appends=0
    MIGRATION REFUSED: $TMP/migrate-export/fabric/storage/obj-19 does not carry a payload.line — the fabric tail is not what the export measured
  [the seed carries two revocations] rc=1 appends=0
    MIGRATION REFUSED: the seed must carry exactly one revocation, found 2; nothing is appended
  [step 4 with step 3's admission file missing] rc=1
    RETIREMENT REFUSED: $TMP/migrate-export/migrate.acceptance.lines is missing or empty — step 3 did not admit its input
  [the seed was rewritten after step 1 hashed it, with its line count unchanged] rc=1 appends=0
    MIGRATION REFUSED: $TMP/migrate-export/seed.jsonl no longer matches the digest step 1 recorded — same count, different content is exactly what a count cannot see
  [step 4 attempted after the third append was refused] rc=1 retires=0
    MIGRATION REFUSED: acceptance 3 of 17 did not append — no further appends, and step 4 must not run
    RETIREMENT REFUSED: step 3 never wrote $TMP/migrate-export/migrate.completion — its appends did not all reach the carrier, so the acceptance history is incomplete
  [the migration completes] rc=0
    SEED RE-VERIFIED against the digest step 1 recorded, before any append
    INPUT ADMITTED: 17 acceptance lines (17 from the seed, 0 from the fabric tail) and 1 revocation
    MIGRATION COMPLETE: 17 acceptance receipts (a seed of 18 lines + 0 from the fabric tail) and 1 revocation receipt admitted
  [step 4 after a completed migration] rc=0 retires=2
    COMPLETION RECEIPT ADMITTED: acceptances=17 revocations=1 seed_sha256=<64 hex> inputs_sha256=<64 hex>
  [step 4 with a truncated completion receipt] rc=1 retires=0
    RETIREMENT REFUSED: $TMP/migrate-export/migrate.completion holds 3 fields, expected 4 — that is not the receipt step 3 writes
  [step 4 with a receipt counting fewer receipts than the inputs hold] rc=1 retires=0
    RETIREMENT REFUSED: the completion receipt counts a different number of acceptance receipts than the inputs hold
  [step 4 with a receipt written against a seed that has since changed] rc=1 retires=0
    RETIREMENT REFUSED: the completion receipt was written against a different seed than the one on disk

<!-- The witnesses print their OWN counts (how many model rows, how many cases this file declares, the hand-shell
line counts) and refuse on a mismatch. Those numbers are deliberately not copied here: a book of counts copied
from a tree is a change detector, and it goes stale exactly when the thing it describes changes. -->

================ PART I: the hand-shell LOC rows agree with the carriers they describe ================
  [the three LOC rows agree with their carriers] OK

================ PART H: the cases this file declares are the model's rows, joined both ways ================
both counts, and the refusal when they disagree in either direction, are printed by the witness as it runs

ALL ASSERTIONS HOLD
```


These are failure injection, not narration. G2 has the modeled CLI refuse the THIRD receipt and requires that
nothing after it was appended — a loop that kept going would migrate a history with a hole in it and still print a
completion line, and that it wrote NO completion receipt, because that file is what tells step 4 the history
arrived. G8 rewrites the seed after step 1 hashed it, keeping the line count, which is the one thing a count
cannot see. G9 runs step 4 after that aborted migration and requires that nothing is retired; G9b runs it after a
completed one and requires that BOTH retirements go through. G10, G11 and G12 hand step 4 a receipt that is
truncated, a receipt whose counts disagree with its inputs, and a receipt written against a seed that has since
changed — each must refuse. G3 removes a seed line after step 1 counted it: the expectation is grounded in what the export
measured, not in the shortened file, so a cut-short input cannot certify itself. G4 and G5 are the fabric object
whose payload lost its line, and a second revocation hidden at the measured line count. G6 and G7 run step 4
against step 3's missing admission files: a half-migrated history cannot be retired over.
### H — step 5b: the activation cannot follow a run it did not dispatch

```text
OK   the dispatched run is identified and admitted — admitted: STAGE plan run=21 mode=plan revision=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
OK   the newest run declares another mode — refused (rc=1): NO DEPLOY: run 21 declares mode=sweep, expected plan
OK   the run resolves to another revision — refused (rc=1): NO DEPLOY: run 21 is at bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, expected aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
OK   no new run appeared after the dispatch — refused (rc=1): NO DEPLOY: no run appeared for mode=plan within 15 minutes
OK   a pre-existing run cannot impersonate the dispatch — refused (rc=1): NO DEPLOY: no run appeared for mode=plan within 15 minutes
OK   the dispatched run did not conclude success — refused (rc=1): NO DEPLOY: the plan run (21) did not complete successfully
OK   a snapshot command that fails is refused — refused (rc=1): NO DEPLOY: the pre-dispatch run snapshot for mode=plan failed; refusing to dispatch blind
OK   a snapshot that comes back empty is refused — refused (rc=1): NO DEPLOY: the pre-dispatch run snapshot for mode=plan is empty; treating every existing run as new is not a correlation
OK   a dispatch already in flight at the revision is refused — refused (rc=1): NO DEPLOY: 1 workflow_dispatch run(s) at aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa are in a nonterminal state (queued, requested, waiting, pending, in_progress or action_required); refusing to dispatch beside a run this runbook did not create
OK   the in-flight check failing is refused — refused (rc=1): NO DEPLOY: the in-flight check for mode=plan failed; refusing to dispatch
OK   two new runs at one revision are refused (no guessing which is ours) — refused (rc=1): NO DEPLOY: 2 runs appeared for mode=plan at aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa; two dispatches are racing, and the first of them is not evidence of authorship
OK   a run waiting for a runner is found by the in-flight check — refused (rc=1): NO DEPLOY: 1 workflow_dispatch run(s) at aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa are in a nonterminal state (queued, requested, waiting, pending, in_progress or action_required); refusing to dispatch beside a run this runbook did not create
OK   a run awaiting a decision is found by the in-flight check — refused (rc=1): NO DEPLOY: 1 workflow_dispatch run(s) at aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa are in a nonterminal state (queued, requested, waiting, pending, in_progress or action_required); refusing to dispatch beside a run this runbook did not create
OK   a run requested but not started is found by the in-flight check — refused (rc=1): NO DEPLOY: 1 workflow_dispatch run(s) at aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa are in a nonterminal state (queued, requested, waiting, pending, in_progress or action_required); refusing to dispatch beside a run this runbook did not create
OK   a failed dispatch is refused and never polled — refused (rc=1): NO DEPLOY: gh workflow run failed for mode=plan; nothing was dispatched, so no run is adopted
OK   a fence held by another dispatch is refused — refused (rc=1): NO DEPLOY: another dispatch for this cutover holds $TMP/held.lock on this host; refusing to dispatch a second run
OK   no fence path at all is refused — refused (rc=1): $TMP/inner.sh: line 11: STAGE_LOCK: NO DEPLOY: STAGE_LOCK is unset; the exclusive-dispatch fence would have no path
OK   a poll that fails is refused — refused (rc=1): NO DEPLOY: the poll for mode=plan failed
ALL DISPATCH-IDENTIFICATION CASES HOLD
```


The four newest cases are about authorship rather than outcome. A run in `waiting`, `requested` or
`action_required` must stop the dispatch — `gh`'s `--status` is one string-valued filter, so passing it twice proved
nothing and those runs were simply adopted. The stub answers only for the states the QUERY NAMES, so a filter that
quietly dropped one fails its own case. A failed `gh workflow run` must be refused WITHOUT a poll, which is asserted
by counting the polls issued rather than by reading a sentence. And the fence is a real contended `flock` held by a
separate live process: stubbed, the case would prove only that a stub can say no.


### The model behind this file

`gunbc.witness.roadmap_cutover` holds the sequence as data — `cutover_step_rows`, nine rows, one per numbered step
above: its `section`, its `modeled_entry` (the route a roadmap fact must change through, spelled
`module::declaration`), its `host_effect` (named even when the answer is "none"), and the `admission` predicate that
admits the next change — and the witness cases as data
(`cutover_witness_case_rows`: what each feeds, whether it must admit or refuse, and the located diagnostic a
refusing case must print). Its claims, run on the runner:

```text
every_cutover_step_is_authorized_to_act
a_step_without_an_admission_is_reported_and_located
a_roadmap_write_without_a_modelled_entry_is_reported
an_unnamed_host_effect_is_reported
every_refusing_witness_case_names_the_diagnostic_it_expects
a_refusing_case_with_no_refusal_text_is_reported
the_scaffold_promises_name_the_emitter_that_would_retire_them
a_scaffold_discriminator_naming_an_undeclared_case_is_rejected
```

The claims are deliberately free of change-detectors: none of them asserts how many steps or cases exist. Each
states an obligation (a predicate, a named host effect, a modelled entry where a roadmap fact changes, a refusal
text a refusing case must print) and proves it can go red by carrying a row or case that meets it wrongly.

The two lists are joined to the executed procedure BOTH ways, by the witnesses rather than by this prose: PART C
here pairs every numbered heading with a step row and every step row with a heading, and PART H of
`tools/roadmap_cutover_runbook_witness.sh` (PART B of the dispatch witness) pairs every case row with a case the
carrier declares, and every declared case with a row — while additionally requiring that a refusing row's promised
diagnostic appears in text that actually runs. A row that promises a case nothing runs, a case that runs with no
row, and a promise no script keeps are each separately fatal.

### Also checked, at this revision

- every shell fence in this file passes `bash -n`;
- the three read-back doors (`roadmap_event_log_retired_report_cli`, `roadmap_event_log_acceptance_report_cli`,
  `roadmap_event_log_read_back_cli`) refuse off-instance, naming the located `wrote.error` or the unreachable
  remote rather than producing an empty verdict;
- the four verdict predicates (`roadmap_retired_verdict_is_admitted`, `roadmap_acceptance_history_verdict_is_loaded`,
  `roadmap_migration_verdict_is_admitted`, `roadmap_read_back_is_admitted`) each have a claim that admits the right
  verdict and refuses a wrong-shaped one;
- the six revocation claims and the acceptance wet driver were re-run after these files changed, because
  `roadmap_event_cli.dag` is where those claims' imports resolve — see the receipt table in the PR.
