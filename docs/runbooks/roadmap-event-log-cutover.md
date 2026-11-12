# Roadmap event-log cutover — export → stop → migrate → verify → start

> **Scaffold status of the shell in this file.** The ten numbered steps below (step 0 first, then 1 through 9) are
> `cutover_step_rows` rendered — each heading names its row, and each row declares the modelled entry that step must
> go through when it changes a roadmap fact, what the step does to the host, and the predicate that admits the next
> change. The commands themselves are hand-written and marked scaffold against the orchestration-to-shell bash-emit
> target, as `gunbc.witness.roadmap_cutover::runbook_dissolve_trigger`. When the modelled host-effect and instrument
> routes (`gunbc.instrument_targets`, `gunbc.live_deploy`, `gunbc.fleet`) can carry the steps themselves, these
> commands are emitted from the rows and this file shrinks to what an operator actually reads: prerequisites,
> readback, dispositions. Nothing else on this branch is shell any more: the two witnesses that used to stand beside
> this file were cut, and `## Validating this runbook before the live partition is touched` says what checks their
> guarantees now.

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
one row per step below — what it may invoke and which predicate admits the next state change. A step whose predicate
is dropped here, or whose write is routed somewhere other than the entry its row names, is a claim failure
(`test.claim.roadmap.roadmap_cutover_witness`), not a typo; the migration's two write doors are additionally gated
in the route (`::roadmap_cutover_retire_cli`, `::roadmap_cutover_acceptance_record_cli`), so an operator cannot
reach the carrier with a fact the precheck never admitted even if this file's guards were edited away. What used to
be held by 2,309 lines of witness shell is accounted for, item by item, at the top of
`gunbc.witness.roadmap_cutover` and in the Validation section at the end.

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

## 0. Ask the log whether this one-time migration has already run — row `precheck`

Every write in this book appends a fact the log does not yet hold: the acceptances and the revocation of step 3,
the retirements of step 4. Appending one of them twice is not a no-op — a second `retired` event is a second
terminal event on that chain, and a receipt recorded twice is an audit trail that lies. The append route beneath
this book does refuse a duplicate by identity, but it refuses DURING the write, after earlier steps of the same
attempt have already landed, which is the worst moment to learn the migration ran before. So the book asks the
question FIRST, read-only, through the two readers production itself uses (`roadmap_retired_node_ids_load`,
`roadmap_acceptance_event_history_load`), and refuses the whole attempt if any fact it is about to add is already
live. The decision is the model's — `gunbc.roadmap.roadmap_event_cli::roadmap_cutover_precheck_cli` over
`roadmap_cutover_precheck_decide`: each reader's answer is matched by its own TYPE, a reader that cannot read is
a refusal and never an empty log, and the shell below only forwards the exit code. A refusal writes NOTHING —
neither report nor roster — so a second run cannot leave artifacts that look like an admitted attempt.

The same entry emits the roster the retirement step iterates, one node id per line, read from the model
(`roadmap_migration_retirement_ids`). Step 4 and step 6 read that file; no node id is typed into this book. A
roster restated in shell is a second home for one fact, and a hand-copied list is how the fifth id gets missed
when the sixth is added.

```bash
D=/opt/gunbc/exports/t6-$(date -u +%Y%m%dT%H%M%SZ)      # named ONCE, locally; below it is DATA, not code
install -d -m 0755 /tmp/t6
# The heredoc is attached to RUNF (a `| tee` on the caller's line would attach it to tee instead).
RUNF "$D" <<'REMOTE' > /tmp/t6/step0-precheck.log 2>&1
set -euo pipefail
D=$1
# ARGUMENT-DELIVERY GUARD, the same one step 1 carries: `bash -s` keeps $0 for itself and makes EVERY word after
# `--` a positional parameter, so a stray filler word would hand this script the literal "runbook" as $1.
case "$D" in
  /*) : ;;
  *) echo "RUNF REFUSED: the export directory arrived as [$D] — argument delivery shifted"; exit 1 ;;
esac
install -d -m 0750 "$D"
CMD="$BIN run --source-root $TREE/dag --source-root $TREE/src/v2 \
  --entry dag/gunbc/roadmap/roadmap_event_cli.dag --function roadmap_cutover_precheck_cli \
  --arg report_path=$D/cutover.precheck.out --arg roster_path=$D/cutover.retire.roster"
RUNE \
  || { echo "PRE-WRITE PRECHECK REFUSED: a fact of this one-time migration is already in the log, or the log could not be read — the entry named which, above; NOTHING was written and no later step of this book may run"; exit 1; }
[ -s "$D/cutover.precheck.out" ] \
  || { echo "PRE-WRITE PRECHECK REFUSED: it admitted the attempt but left no report at $D/cutover.precheck.out — a cutover with no quotable pre-write read is a cutover nobody can audit"; exit 1; }
[ -s "$D/cutover.retire.roster" ] \
  || { echo "PRE-WRITE PRECHECK REFUSED: it admitted the attempt but emitted no roster at $D/cutover.retire.roster — without that file there is nothing for step 4 to iterate"; exit 1; }
# WHAT THIS BLOCK NO LONGER DECIDES FROM PROSE (review 94487). It used to grep the report for the ADMITTED
# sentence. A status that a sentence decides is a status that changes whenever someone rewords a sentence, so the
# verdict is a machine line instead — the report's FIRST line, written and parsed by one function,
# roadmap_cutover_verdict_token — and it is consumed where the write happens: roadmap_cutover_retire_cli and
# roadmap_cutover_acceptance_record_cli refuse unless that line says admitted. This block's own duty is that the
# entry exited successfully (the precheck exits 1 when it refuses, and RUNF fails the block on it) and that both
# artefacts exist and are non-empty. The next line only ECHOES the verdict, for the operator and for the PR quote;
# nothing downstream reads it as an answer.
echo "PRECHECK VERDICT LINE: $(head -n 1 "$D/cutover.precheck.out")"
N=$(grep -c . "$D/cutover.retire.roster" || true)   # || true: grep exits 1 on zero matches, and under set -e that
                                                                                       # would abort with no message — the one
                                                                                       # refusal this line exists to say out loud
[ "$N" -ge 1 ] || { echo "PRE-WRITE PRECHECK REFUSED: the roster file holds no node id, so the migration would retire nothing and still report success"; exit 1; }
echo "PRE-WRITE PRECHECK ADMITTED: $N retirement(s) queued from the model's roster; the pre-write read is at $D/cutover.precheck.out"
REMOTE
```

REFUSALS, each one exercised in the Validation section: the argument delivery shifted; the entry refused because a
retirement or an acceptance of this migration is already live; the entry refused because a reader could not read
the log; the report missing or empty; the roster missing or empty; the report present but its first line not the
precheck's machine verdict — in which case the write doors at steps 3 and 4 refuse, naming the line they were
given.

**What the PR must show:** the whole of `$D/cutover.precheck.out`, verbatim — it is the read-only answer of the
two production readers to "is any of this already here", taken immediately before the first write — and the
number of lines in `$D/cutover.retire.roster`, which is what step 4 iterates.

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

**The authoritative carrier, measured read-only on 2026-11-12 at branch tip `10bf77aa512883ad…` (unchanged
since 2026-11-08, the date of its newest event):** 126 objects, of which `events/` holds 67 across 23 node chains
— `claimed` 60, `retired` 3, `result_returned` 2, `comment_created` 2 — and `operations/` holds 59 operation
receipts (`{event, id}`, the idempotence receipts the append route writes beside each event). No event of kind
`acceptance_recorded` or `acceptance_revoked` exists anywhere on the branch, and none of the 17 acceptance nodes
step 3 replays has ANY chain, so each of those appends starts a fresh chain and none can collide with an existing
one. The retirement roster, node by node, with the compare-and-swap parent each retirement must name:

| node (from `roadmap_migration_retirement_ids`) | events on the branch | chain tip = the `parent` its retirement names |
| --- | --- | --- |
| `entry-graph-union-construction` | none | the genesis id (no chain) |
| `shell-dag-floor-peak-calibration-rows` | 1 `claimed` | `0cc95abae720582e`, `claimed`, 2026-11-05T16:14:22Z |
| `shell-dag-ci-fmt-gate-line` | 1 `claimed` | `4ebc95d0c29acebe`, `claimed`, 2026-11-05T16:18:48Z |
| `shell-dag-ci-deploy-invoke-prelude` | 4 (`result_returned` 1, `claimed` 3) | `0c24c5195eb6db0f`, `result_returned`, 2026-11-04T21:11:21Z |
| `shell-dag-ci-heal-regen-invoke` | none | the genesis id (no chain) |

None of the five carries a `retired` event, so on this measurement the migration has not run and step 0 admits it.
That table is a READING, not an authority: step 0 re-measures it through the modeled readers immediately before
the first write and refuses if any of it has changed, including if it has changed in the direction of "already
done". The three `retired` events on the branch, all reason `test`, sit on `harness-checkpoint-subject-planner`,
`harness-conversation-log-forking` and `v2-emitter-direct-rust-door` — they are what put the 503 on the live page
(step 2b), and they are exactly the kind step 4 writes.


(The unit files are exported inside the same script because they are part of what is touched: step 5b's
deploy repoints them, so a rollback needs both the files and the effective definitions from the same instant.)

## 2. Stop every reader of this carrier — timers first — row `stop`

The operator ruled (relayed 2026-11-10): all five roadmap units move together to the staged release, and
`gunbc-roadmap-fabric-storage.service` is not touched at all. Stop the timer before the units it drives, so
a tick cannot fire into the window:

```bash
set -euo pipefail
# THE SAME FIVE THIS CUTOVER OWNS — spelled identically to UNITS in step 1's export, plus the timer that drives two
# of the oneshots. A roster here that differs from the exported one means a reader was never measured. Nothing in the
# tree cross-checks the two spellings any more (the shell witness that used to refuse them apart was cut, and the
# note at the top of gunbc.witness.roadmap_cutover says why), so this list is compared against step 1's export by
# eye, on the record, before the stop is authorized.
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
: "${D:?MIGRATION REFUSED: EXPORT_DIR is not set — run step 1 and use the path it echoed}"
: "${OPERATOR:?MIGRATION REFUSED: the author is part of the fact; an empty author is a wrong fact}"
# THE ATTEMPT IS NAMED BEFORE ANYTHING CAN FAIL, and the completion receipt at the end of this step carries that
# name; step 4 compares them. Counts and digests cannot do this job: a receipt left by an EARLIER attempt — over a
# branch that may since have been restored — is byte-for-byte indistinguishable from a fresh one, because the
# inputs are re-prepared from the same export. Only a name mints per attempt and cannot be inherited.
ATTEMPT=$(printf '%s-%s-%s' "$(date -u +%FT%TZ)" "$$" "${RANDOM}${RANDOM}" | sha256sum | cut -c1-16)
# The name exists only in this shell until the pair below has been destroyed. Publishing it first would mean a
# failed publication leaves the PREVIOUS attempt's name on disk — and the previous receipt next to it, which is
# exactly what step 4 admits on. So: mint here, destroy there, publish after.
# ONE COMPLETION PER ATTEMPT. The receipt written at the end of this step is the ONLY thing step 4 demands before
# it retires anything, and this step writes it only after the LAST admitted append — so when a retry begins, the
# PREVIOUS attempt's receipt is still standing on disk. A replay that dies on its third append would leave step 4
# holding a receipt that is intact, self-consistent, and describes a history the failed replay never wrote: the
# inputs are re-prepared from the same export, so the counts, the seed digest and the input digest ALL match it.
# Nothing downstream can tell that receipt from a fresh one, so it is destroyed below — by a predicate, not a hope.
# THE DELETION IS AN ADMISSION PREDICATE, NOT A HOPE. Its status is checked and its EFFECT is re-checked after it:
# `rm -f` reports success for a file it never touched and failure for one it could not remove (a read-only mount, an
# immutable file, another user's ownership), and a deletion that fails silently leaves the previous attempt's
# receipt sitting on the exact path step 4 reads. It also comes BEFORE `cd "$TREE"`: a checkout that cannot be
# entered must not be able to fail after an old receipt has been left standing to be trusted.
if [ -e "$D/migrate.attempt" ] || [ -e "$D/migrate.completion" ]; then
  # ONE ACT, BOTH HALVES OF THE AUTHORIZATION. `rm` visits every operand even after one of them fails, so a partial
  # failure still breaks the pair: whichever of the two could be removed is gone, and step 4 needs BOTH the name and
  # the receipt naming it. Deleting only the receipt (the previous shape of this step) left a path to a usable
  # authorization: an unwritable $D/migrate.attempt keeps the previous attempt's name, and if this step then refused
  # before rewriting it, the previous receipt still matched it.
  rm -f -- "$D/migrate.attempt" "$D/migrate.completion" \
    || { echo "MIGRATION REFUSED: could not destroy $D/migrate.attempt and $D/migrate.completion in one act — a previous attempt's name and receipt must not survive together to authorize step 4"; exit 1; }
  for stale in "$D/migrate.attempt" "$D/migrate.completion"; do
    [ ! -e "$stale" ] \
      || { echo "MIGRATION REFUSED: $stale still exists although the delete reported success — treat it as live and refuse"; exit 1; }
  done
  echo "STALE INVALIDATED: $D/migrate.attempt $D/migrate.completion (this attempt is $ATTEMPT)"
fi
# PUBLISH THE NAME, THEN READ IT BACK. The file was destroyed above, so anything in it now was written by this
# attempt — and the read-back says so in words rather than by assumption: a truncated or racing write is refused
# here, not discovered by step 4 comparing two documents this run does not control.
printf '%s\n' "$ATTEMPT" > "$D/migrate.attempt" \
  || { echo "MIGRATION REFUSED: this attempt could not be recorded in $D/migrate.attempt — an unrecorded attempt cannot be told from an earlier one, which is the hole step 4 would then fall into"; exit 1; }
[ "$(cat "$D/migrate.attempt")" = "$ATTEMPT" ] \
  || { echo "MIGRATION REFUSED: $D/migrate.attempt does not hold the name this attempt minted (it holds $(head -c 24 "$D/migrate.attempt" | tr -d '\n')) — step 4 would be comparing against a name this run never wrote"; exit 1; }
cd "$TREE"

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
  # THE MIGRATED ENTRY, not the bare record route: roadmap_cutover_acceptance_record_cli refuses a receipt the
  # closed set does not name unless step 0's report is in hand carrying its admitted verdict. The guard above is a
  # courtesy to the operator; this is the enforcement.
  CMD="$BIN run --source-root dag --source-root src/v2 \
    --entry dag/gunbc/roadmap/roadmap_event_cli.dag --function roadmap_cutover_acceptance_record_cli \
    --arg repo_root=/opt/gunbc/gunbc --arg receipt_line=\"\$RECEIPT_LINE\" \
    --arg pr_url= --arg merged_at= --arg head_sha= \
    --arg author=\"\$OPERATOR\" --arg report_path=$D/cutover.precheck.out"
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
  printf 'attempt=%s\n' "$ATTEMPT"
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
`operation-conflict`, and it never appends a second event. The same holds for each of the roster's retirements, whose
operation key is the node plus the digest of its reason; a second retirement of an already-retired node
is refused by the admission law rather than appended. Witnessed by executing real appends and real retries
against a temp carrier:
`a_retry_of_a_committed_acceptance_after_the_head_moved_answers_the_first_recording` and
`a_second_retirement_of_a_retired_node_refuses_and_a_retry_of_the_first_answers_it`.

The CLI answers `ExitSuccess`/`ExitFailure` (a `ProcessExit`), so its exit is NOT the receipt. The receipts
are the identities printed by step 4, read out of the carrier through the production reader.

## 4. Then the roster's retirements (step 1(a)) — the retirement is the newest fact, so it goes last — row `migrate-retired`

```bash
set -euo pipefail
cd $TREE
: "${D:?RETIREMENT REFUSED: EXPORT_DIR is not set — the step-3 admission files are the precondition for this block}"
: "${OPERATOR:?RETIREMENT REFUSED: the author is part of the fact; an empty author is a wrong fact}"
# step 3's admission is a PRECONDITION, checked here rather than trusted. The LINE FILES are only half of it and
# are the weaker half: they are prepared before the first append, so a migration that died at its third receipt
# leaves both of them sitting here full. What proves the history arrived is step 3's COMPLETION RECEIPT — written
# after the last admitted append, and required here in full: present, exactly the five fields step 3 writes,
# counting the same number of receipts the inputs hold, digesting the same input bytes, recorded against the
# seed that is actually on disk, and NAMING THE ATTEMPT that is running now. A receipt copied from another run,
# truncated, rewritten to agree with itself, written against a seed since changed, or left over from a migration
# this process never completed cannot authorize a retirement.
[ -s "$D/migrate.acceptance.lines" ] \
  || { echo "RETIREMENT REFUSED: $D/migrate.acceptance.lines is missing or empty — step 3 did not admit its input"; exit 1; }
[ -s "$D/migrate.revocation.lines" ] \
  || { echo "RETIREMENT REFUSED: $D/migrate.revocation.lines is missing or empty — step 3 did not admit its input"; exit 1; }
[ -s "$D/migrate.completion" ] \
  || { echo "RETIREMENT REFUSED: step 3 never wrote $D/migrate.completion — its appends did not all reach the carrier, so the acceptance history is incomplete"; exit 1; }
[ "$(wc -l < "$D/migrate.completion")" -eq 5 ] \
  || { echo "RETIREMENT REFUSED: $D/migrate.completion holds $(wc -l < "$D/migrate.completion") fields, expected 5 — that is not the receipt step 3 writes"; exit 1; }
grep -q "^acceptances=$(wc -l < "$D/migrate.acceptance.lines")$" "$D/migrate.completion" \
  || { echo "RETIREMENT REFUSED: the completion receipt counts a different number of acceptance receipts than the inputs hold"; exit 1; }
grep -q "^revocations=$(wc -l < "$D/migrate.revocation.lines")$" "$D/migrate.completion" \
  || { echo "RETIREMENT REFUSED: the completion receipt counts a different number of revocation receipts than the inputs hold"; exit 1; }
grep -q "^seed_sha256=$(sha256sum "$D/seed.jsonl" | cut -d" " -f1)$" "$D/migrate.completion" \
  || { echo "RETIREMENT REFUSED: the completion receipt was written against a different seed than the one on disk"; exit 1; }
# AND THE INPUT THEMSELVES, BY CONTENT. Counting lines and re-checking the seed still admits a receipt whose
# input bytes were changed underneath it: one line rewritten, count untouched, seed untouched, and step 4 would
# retire nodes over an acceptance history that is not the one step 3 completed. The digest is recomputed over
# exactly what step 3 digested — both files, in that order — before a single retirement is attempted.
grep -q "^inputs_sha256=$(cat "$D/migrate.acceptance.lines" "$D/migrate.revocation.lines" | sha256sum | cut -d" " -f1)$" "$D/migrate.completion" \
  || { echo "RETIREMENT REFUSED: the completion receipt digests different input lines than the ones on disk"; exit 1; }
# AND IT MUST NAME THIS ATTEMPT. Field count, counts and digests can ALL be satisfied by a receipt an earlier
# attempt wrote — including one whose destruction this run could not complete, and one written before a carrier
# was restored from step 7. The attempt name is the only field that ties a receipt to the migration happening NOW,
# and it is the only one that cannot be inherited: step 3 mints a fresh one per run and records it in
# $D/migrate.attempt before anything can fail.
[ -s "$D/migrate.attempt" ] \
  || { echo "RETIREMENT REFUSED: $D/migrate.attempt is missing — no attempt was recorded, so no receipt can be shown to belong to this run while $D/migrate.completion sits beside it and the receipt names $(sed -n 's/^attempt=//p' "$D/migrate.completion" 2>/dev/null || echo '<unreadable>')"; exit 1; }
ATTEMPT_NOW=$(head -n1 "$D/migrate.attempt")
[ "${#ATTEMPT_NOW}" -eq 16 ] \
  || { echo "RETIREMENT REFUSED: the recorded attempt is [$ATTEMPT_NOW], which is not the 16-character name step 3 mints"; exit 1; }
RECEIPT_ATTEMPT=$(sed -n 's/^attempt=//p' "$D/migrate.completion")
[ "$RECEIPT_ATTEMPT" = "$ATTEMPT_NOW" ] \
  || { echo "RETIREMENT REFUSED: $D/migrate.completion does not name THIS attempt — the receipt names ${RECEIPT_ATTEMPT:-<nothing>} and this run recorded $ATTEMPT_NOW, so it is a receipt from an earlier migration and cannot authorize retiring over a history this run did not write"; exit 1; }
echo "COMPLETION RECEIPT ADMITTED: $(tr "\n" " " < "$D/migrate.completion")"

# THE RETIREMENT TEXT AND AUTHOR TRAVEL AS ENVIRONMENT DATA, never spliced into the command string. The first
# refusal aborts the loop: a half-retired floor is not a retired floor, and step 5 must not be told otherwise.
RETIRED_TEXT='The floor cut retired this work; its remaining cost lives on its successor.'
# THE NODES ARE THE MODEL'S, not this book's: step 0 wrote the roster straight from
# roadmap_migration_retirement_ids, so the set of nodes retired is the set the model says to retire. A list
# retyped here would be a second home for that fact and would go stale the day the roster grows.
ROSTER_FILE="$D/cutover.retire.roster"
[ -s "$ROSTER_FILE" ] \
  || { echo "RETIREMENT REFUSED: $ROSTER_FILE is missing or empty — the roster comes from step 0's precheck, and without it there is nothing authorised to retire"; exit 1; }
grep -q '[^[:space:]]' "$ROSTER_FILE" \
  || { echo "RETIREMENT REFUSED: $ROSTER_FILE holds no non-whitespace node id"; exit 1; }
NODES=0
while IFS= read -r NODE || [ -n "$NODE" ]; do
  [ -n "$(printf '%s' "$NODE" | tr -d '[:space:]')" ] || continue
  case "$NODE" in
    *[[:space:]]*) echo "RETIREMENT REFUSED: roster line [$NODE] in $ROSTER_FILE is not a single node id"; exit 1 ;;
  esac
  NODES=$((NODES + 1))
  # THE MIGRATED ENTRY. roadmap_cutover_retire_cli reads the report and the roster itself and refuses unless the
  # report carries the admitted verdict, the roster IS roadmap_migration_retirement_ids, and it names this node — so
  # the ordering between step 0 and this loop is enforced by the route and cannot be re-arranged by editing a guard
  # here. A node outside the roster is refused by name; a roster a caller edited is refused wholesale.
  CMD="$BIN run --source-root dag --source-root src/v2 \
    --entry dag/gunbc/roadmap/roadmap_event_cli.dag --function roadmap_cutover_retire_cli \
    --arg repo_root=/opt/gunbc/gunbc --arg node=\"\$NODE\" \
    --arg text=\"\$RETIRED_TEXT\" --arg author=\"\$OPERATOR\" \
    --arg report_path=$D/cutover.precheck.out --arg roster_path=$D/cutover.retire.roster"
  RUNE "NODE=$NODE" "RETIRED_TEXT=$RETIRED_TEXT" "OPERATOR=$OPERATOR" \
    || { echo "RETIREMENT REFUSED: $NODE did not take its retirement — no further retirement, and step 5 must not run"; exit 1; }
  echo "retired $NODE"
done < "$ROSTER_FILE"
[ "$NODES" -ge 1 ] \
  || { echo "RETIREMENT REFUSED: the roster named no node, so nothing was retired — success would have been a lie"; exit 1; }
echo "RETIRED $NODES node(s), every one of them from $ROSTER_FILE"
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
# THE ADMISSION IS REBUILT OR IT IS ABSENT. step 5b reads $D/read-back.out and $D/migration.verified as admission
# PREDICATES, and this fence writes both only on its success path. Without this line a re-run whose read-back fails
# leaves the PREVIOUS attempt's documents standing: the `grep '^MIGRATION VERIFIED'` below is satisfied by the stale
# document, the digest it records is the digest of that stale document, and activation is certified by a read-back
# that never happened in this attempt. Both go together — an admission missing half its evidence is no admission.
# THE VERIFICATION IS NAMED, AND THE PREVIOUS AUTHORIZATION IS DESTROYED IN ONE ACT BEFORE THAT NAME IS PUBLISHED.
# Step 5b compares the name in $D/verify.attempt with the name the admission carries, BEFORE it contacts GitHub.
# Two failure shapes had to die together, and neither is closed by the other:
#   * a checked deletion that stops at its first failure — step 5 refuses with BOTH of the previous attempt's
#     documents still on disk, and step 5b finds a `migration.verified` that exists, binds this revision, digests
#     the read-back that is still there, and matches the live tip;
#   * a publication that fails — the previous attempt's name survives in $D/verify.attempt, beside the previous
#     admission that already names it, and nothing downstream can tell the pair from a fresh one.
# The order below kills both: the name is minted in this shell only, the documents are destroyed by ONE `rm` (which
# visits every operand even after one of them fails, so a partial failure still breaks the pair), and only then is
# the name published and read back. Any failure after this point leaves step 5b missing one of the documents it
# needs, or holding an admission that names a value no file carries.
VERIFY_ATTEMPT=$(printf '%s-%s-%s' "$(date -u +%FT%TZ)" "$$" "${RANDOM}${RANDOM}" | sha256sum | cut -c1-16)
# THE DELETION IS AN ADMISSION PREDICATE. This fence does not run under `set -e` (it cannot: its job is to inspect
# a failing read-back and print its refusal), so a bare `rm -f` whose status nobody reads is not a gate — a
# deletion that fails (read-only mount, immutable file, another user's ownership) leaves the PREVIOUS attempt's
# document on the exact path the check below reads, and `grep '^MIGRATION VERIFIED'` is then satisfied by a
# read-back that never ran in this attempt. Status checked, absence re-checked, and it happens BEFORE the
# read-back is issued, so nothing can be measured against a document this run did not produce.
if [ -e "$D/verify.attempt" ] || [ -e "$D/migration.verified" ] || [ -e "$D/read-back.out" ]; then
  # ONE ACT, THREE HALVES OF THE AUTHORIZATION. `rm` visits every operand even after one of them fails, so a partial
  # failure still breaks the pair: whatever could be removed is gone, and step 5b needs the admission, the name it
  # carries, and the read-back whose digest the admission records.
  rm -f -- "$D/verify.attempt" "$D/migration.verified" "$D/read-back.out" \
    || { echo "MIGRATION REFUSED: could not destroy $D/verify.attempt, $D/migration.verified and $D/read-back.out in one act — a previous attempt's verification must not survive to authorize this one"; exit 1; }
  for stale in "$D/verify.attempt" "$D/migration.verified" "$D/read-back.out"; do
    [ ! -e "$stale" ] \
      || { echo "MIGRATION REFUSED: $stale still exists although the delete reported success — treat it as live and refuse"; exit 1; }
  done
  echo "STALE INVALIDATED: $D/verify.attempt $D/migration.verified $D/read-back.out (this verification is $VERIFY_ATTEMPT)"
fi
# PUBLISH THE NAME, THEN READ IT BACK — the file was destroyed above, so anything in it now was written by this
# attempt, and the read-back says so instead of leaving step 5b to discover it by comparing two documents.
printf '%s\n' "$VERIFY_ATTEMPT" > "$D/verify.attempt" \
  || { echo "MIGRATION REFUSED: this verification could not be recorded in $D/verify.attempt — an unrecorded verification cannot be told from an earlier one, which is the hole step 5b would then fall into"; exit 1; }
[ "$(cat "$D/verify.attempt")" = "$VERIFY_ATTEMPT" ] \
  || { echo "MIGRATION REFUSED: $D/verify.attempt does not hold the name this verification minted (it holds $(head -c 24 "$D/verify.attempt" | tr -d '\n')) — step 5b would be comparing against a name this run never wrote"; exit 1; }

MANIFEST="$(cat "$D/seed.jsonl")"   # the fabric store contributes nothing today (verified at step 1: acceptance.count is 0);
                                    # if acceptance.before is not empty, append each object's payload.line as step 3 derives them
# The SAME file step 4 iterated, joined with commas for the reader's argument: the roster the read-back checks
# against is the roster the migration acted on, never a second transcription that could disagree with it.
[ -s "$D/cutover.retire.roster" ] \
  || { echo "MIGRATION REFUSED: $D/cutover.retire.roster is missing or empty — step 5 cannot check the read-back against a roster it cannot read"; exit 1; }
ROSTER=$(paste -sd, "$D/cutover.retire.roster")
[ -n "$ROSTER" ] || { echo "MIGRATION REFUSED: the roster joined to an empty list"; exit 1; }
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
    echo "ATTEMPT=$VERIFY_ATTEMPT"
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
`ProcessExit`. The roster's retirements are not listed: they are checked against the roster, and a missing one is
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
# POSITIVELY IDENTIFIED before its receipts are consumed. (1) The gate: eight predicates, each refusing before
# GitHub is contacted, and the verifier asked AGAIN on the spot, so a migration verified at 03:00 whose branch
# moved at 03:05 cannot be activated on the 03:00 receipt.
# (2) The reference: gh workflow run takes a REF, and a bare commit SHA is not what the CLI documents — so each
# stage is dispatched against its OWN TAG, minted by THIS invocation at this revision with a per-invocation
# nonce, pushed, and verified to resolve to $SHA before anything is dispatched. The ref is what the poll then
# correlates a run TO: see the minting block inside dispatch_stage.
# (3) The runs: three sequential dispatches (plan -> apply -> dashboard_deploy); each waits for a run to
# APPEAR that was not there before, then asserts that run's own declared mode and head_sha through the
# Actions API, then watches it to a successful conclusion. Nothing consumes a receipt from a run that was not
# identified as its own.
test -s "$D/migration.verified"                     || { echo 'NO DEPLOY: step 5 never admitted the read-back'; exit 1; }
# AND THE ADMISSION MUST NAME THIS VERIFICATION. The predicates below can all be satisfied by documents an
# earlier step 5 wrote — which is exactly the state a step 5 that REFUSED on a failed invalidation leaves behind,
# since its deletion loop stops at the failure and the previous attempt's pair survives together. step 5 rewrites
# $D/verify.attempt on entry, before anything fallible; the admission carries the name of the run that produced it.
# Both names are printed on refusal, because "step 5 refused and its old admission is still standing" and
# "someone copied an admission in from another export" are different incidents with different repairs.
{ [ -f "$D/verify.attempt" ] && [ -s "$D/verify.attempt" ]; } \
                                                    || { echo 'NO DEPLOY: no verification attempt is recorded — nothing shows the admission below came from a step 5 that ran to completion'; exit 1; }
VERIFY_ATTEMPT_NOW=$(head -n1 "$D/verify.attempt")
[ "${#VERIFY_ATTEMPT_NOW}" -eq 16 ]                 || { echo "NO DEPLOY: the recorded verification attempt is [$VERIFY_ATTEMPT_NOW], not the 16 characters step 5 mints"; exit 1; }
ADMIT_ATTEMPT=$(sed -n 's/^ATTEMPT=//p' "$D/migration.verified")
[ "$ADMIT_ATTEMPT" = "$VERIFY_ATTEMPT_NOW" ]        || { echo "NO DEPLOY: $D/migration.verified does not name THIS verification — the admission names ${ADMIT_ATTEMPT:-<nothing>} and step 5's latest run recorded $VERIFY_ATTEMPT_NOW, so a step 5 that refused is being spoken for by documents it never wrote"; exit 1; }
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
# THE REPORT IS REBUILT OR ABSENT. The standing count at the end of this fence reads $D/read-back.after.out, which
# the read-back entry writes through the modeled Filesystem.Write; a re-run whose read-back produced nothing would
# otherwise be measured against the PREVIOUS attempt's document and report a retirement that is not in the log.
# AS IN STEP 5, AND FOR THE SAME REASON, in a fence that explicitly cannot rely on `set -e`: the standing count
# below is a `grep -c` over this one document, so a deletion that fails silently measures the PREVIOUS attempt's
# retirements as this run's. Status checked, absence re-checked, refusal in this step's own vocabulary.
for stale in "$D/read-back.after.out"; do
  if [ -e "$stale" ]; then
    rm -f -- "$stale" \
      || { echo "OBSERVATION REFUSED: could not destroy $stale — a report from a previous attempt must not be measured as this run's standing"; exit 1; }
    [ ! -e "$stale" ] \
      || { echo "OBSERVATION REFUSED: $stale still exists although the delete reported success — treat it as live and refuse"; exit 1; }
    echo "STALE INVALIDATED: $stale"
  fi
done
# exit 0 = the fail-closed retired read ADMITTED the set (a log it cannot read exits 1 naming the reason, and
# is never served as an empty retired set); the reason the runtime prints IS the diagnostic. An admission nobody
# consumes is a comment: this fence has no `set -e`, so the status is CAPTURED and a refusal stops the step here,
# before anything can call the page migrated.
RUN "$BIN run --source-root $TREE/dag --source-root $TREE/src/v2 --entry dag/gunbc/roadmap/roadmap_event_cli.dag --function roadmap_event_log_retired_report_cli"; RETIRED_RC=$?
[ "$RETIRED_RC" -eq 0 ] || { echo "OBSERVATION REFUSED: the fail-closed retired read exited $RETIRED_RC — the log was not read the way production reads it, so the page is NOT accepted as migrated"; exit 1; }
# exit 0 = the one decoder of the one home admitted the acceptance history. Same rule, same reason.
RUN "$BIN run --source-root $TREE/dag --source-root $TREE/src/v2 --entry dag/gunbc/roadmap/roadmap_event_cli.dag --function roadmap_event_log_acceptance_report_cli"; ACCEPTANCE_RC=$?
[ "$ACCEPTANCE_RC" -eq 0 ] || { echo "OBSERVATION REFUSED: the acceptance decoder exited $ACCEPTANCE_RC — the history the page will serve is not the one the migration admitted"; exit 1; }
# exit 0 = verifier + both readers + the write all admitted, in one document (step 5's artifact, regenerated).
# The manifest and the roster are DATA and travel in the ENVIRONMENT through RUNE, exactly as in step 5. An
# earlier pass put them INSIDE the command string (`--arg expected_manifest=\"$MANIFEST\"`) that RUN hands to
# `bash -lc`, and the shell systemd-run spawns then parsed the JSON's own quotes as SHELL: the reviewer
# reproduced an object arriving as several arguments with its quotes stripped, so the read-back would have
# certified less than the old carriers held — and a receipt line carrying `$(…)` would have been INTERPRETED,
# which is the same class as the export finding. Same data, same route, one argument, byte-for-byte.
CMD="$BIN run --source-root $TREE/dag --source-root $TREE/src/v2 --entry dag/gunbc/roadmap/roadmap_event_cli.dag --function roadmap_event_log_read_back_cli --arg repo_root=/opt/gunbc/gunbc --arg expected_manifest=\"\$EXPECTED_MANIFEST\" --arg retired_roster=\"\$EXPECTED_ROSTER\" --arg report_path=$D/read-back.after.out"
# An empty manifest would certify nothing, so the variables step 5 set are required to still be set here; an
# environment that lost them is a refusal, not a read-back over zero expected lines. The roster's emptiness is
# checked BEFORE the ${VAR:?} forms, and by a refusal of its own: an empty roster makes every expected count zero,
# and a verification with no subject passes — and ${VAR:?} killing an interactive operator shell mid-step is not a
# gate either.
[ -n "$ROSTER" ] || { echo "OBSERVATION REFUSED: the roster is empty — a read-back that expects nothing admits nothing"; exit 1; }
: "${MANIFEST:?OBSERVATION REFUSED: EXPECTED_MANIFEST is unset — re-run step 5 in this shell}"
: "${ROSTER:?OBSERVATION REFUSED: EXPECTED_ROSTER is unset — re-run step 5 in this shell}"
RUNE "EXPECTED_MANIFEST=$MANIFEST" "EXPECTED_ROSTER=$ROSTER" > "$D/read-back.after.driver.log" 2>&1
READ_AFTER_RC=$?
[ "$READ_AFTER_RC" -eq 0 ] || { echo "OBSERVATION REFUSED: the post-deploy read-back exited $READ_AFTER_RC — the page is NOT accepted as migrated; do not close the cutover, and step 7 is the rollback"; exit 1; }
cat "$D/read-back.after.out"
# and the per-node standing, as the log's own standing renderer words it. The expected count comes from the roster
# the operator carried in from step 5 — not from a literal here — and the count is REQUIRED, not printed: a fence
# without `set -e` that merely displays "0" reports a successful observation while serving the old carriers. A
# roster that arrived empty would make the comparison pass at zero, so an empty roster is its own refusal.
EXPECTED_STANDING=$(printf '%s\n' "$ROSTER" | tr ',' '\n' | grep -c '[^[:space:]]')
STANDING=$(grep -c 'retired by the event log' "$D/read-back.after.out")
[ "$STANDING" = "$EXPECTED_STANDING" ] || { echo "OBSERVATION REFUSED: the log's own standing renderer shows $STANDING of $EXPECTED_STANDING roster nodes retired — the page is NOT accepted as migrated"; exit 1; }
echo "STANDING ADMITTED: $STANDING of $EXPECTED_STANDING roster nodes retired by the event log"
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
# failure instead of performing it. WHAT CHECKS THIS NOW IS THE OPERATOR, NOT A HARNESS: the shell witness that ran
# both directions against a STATEFUL systemctl stub — units recorded active really receiving `systemctl start`,
# units recorded inactive really not, and an off-roster unit refused — was cut, and restarting the real readers is
# not something a dry run may do. So the guard below is the check, it is executed on the scratch before any live
# rollback is trusted, and the first real rollback is itself an observed event: report its `came up` lines.
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
  # This guard is the check: it is the reason the malformed inventory above cannot silently restore nothing.
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


## Validating this runbook before the live partition is touched

A runbook is executable text, and CI does not run this file. What used to run it was two shell witnesses on this
branch — 2,309 lines that lifted the book's fenced blocks and re-ran them against stubbed `gunbc`, `gh`,
`systemd-run` and `git`, with their 98 cases declared as rows in `gunbc.witness.roadmap_cutover`. They are gone, cut
at review 94464 on a reading this book agrees with: a one-time migration has no future consumer, and the test
`gunbc.design_argument::how_to_work` puts to a carrier is whether it survives into the terminal architecture and is
consumed by it. The full reason, and the accounting for what each witness used to guarantee, is the header note of
`gunbc.witness.roadmap_cutover`. Two lines of it belong here too, because they are the part an operator acts on: the
witnesses read the model by regular expression over `.dag` text, which made them a second parser rather than a
consumer, and no job in the tree executed them, so their 98 green cases were a claim rather than a fact.

What carries the safety now is machine-checked, in two places you can read and run:

1. **The routes.** `roadmap_cutover_precheck_cli` answers, from the production readers, whether this one-time
   migration has already run, and writes nothing when it refuses. `roadmap_cutover_retire_cli` and
   `roadmap_cutover_acceptance_record_cli` are the migration's only write doors, and each refuses unless the
   precheck's report is readable, opens with its machine admitted verdict, and — for a retirement — unless the
   roster on disk is exactly the roster the model holds and names the node. You cannot get the write by forgetting
   the check, by typing a report, or by editing a guard in this file.
2. **The claims.** `dag/test/claim/roadmap/roadmap_cutover_witness.dag` runs in the witness gate. The eleven that
   decide this migration: `a_migration_write_without_a_precheck_report_is_refused_naming_its_node`,
   `a_report_that_refused_never_admits_a_migration_write`,
   `the_admitted_report_admits_every_node_the_model_rosters`,
   `a_retirement_the_admitted_roster_does_not_name_is_refused`,
   `a_roster_the_model_does_not_hold_is_refused_wholesale`,
   `a_roster_of_blank_or_multi_token_lines_is_refused_as_not_a_roster`,
   `an_acceptance_is_admitted_inside_the_closed_set_and_refused_outside_it`,
   `an_empty_closed_set_admits_no_acceptance`, `the_report_opens_with_the_verdict_the_doors_parse`,
   `prose_alone_never_admits_a_migration_write`,
   `a_document_that_is_not_a_precheck_report_is_refused_as_unreadable`. Their report fixtures are rendered by the
   model's own `roadmap_cutover_precheck_document`, never typed here, so the doors and the renderer can only both
   pass if they still agree on the machine verdict line.

### The dry run, and the receipt it leaves

Before the live branch is touched, run steps 0 through 6 once against a SCRATCH of your own making on srv1: a
disposable clone of `gunb-ai/gunbc` whose event-log branch is a scratch branch created from the live tip
(`roadmap-events-dryrun-<UTC>`), with its own export directory. The commands are the commands above, verbatim, with
`--arg repo_root` and the export path pointing at the scratch — that is the whole point: the dry run fails for the
same reasons the real run will, and passes for the same reasons.

The dry run is only a dry run if it exercises the refusals, so run these four on purpose and keep the output:

| # | do this | the route must answer |
|---|---|---|
| 1 | `mv "$D/cutover.precheck.out" "$D/away"` and re-run step 4 | refusal: `no pre-write precheck report is readable for the retirement of <node>` — and the node is named |
| 2 | append one line to `$D/cutover.retire.roster` and re-run step 4 | refusal: `the roster in hand is not the roster the model holds` |
| 3 | run the whole migration a second time on the scratch | refusal: `retirement is ALREADY LIVE for <node>` / `an acceptance is ALREADY LIVE for <identity>` — the precheck, not the append, is what stops a second run |
| 4 | `sed -i '1s/.*/ADMITTED — none of the migration's own facts is live yet/' "$D/cutover.precheck.out"` and re-run step 4 | refusal: `its first line is [ADMITTED — none of the migration's own facts is live yet], which is neither verdict token this module writes` — the human sentence is not a permit, which is the whole point of the machine line |

Then restore, let the admitted path complete, and keep the scratch's `$D` — export, report, roster, completion
receipts — as the dry-run receipt. Attach it to the PR that authorises the live run, together with the scratch
branch's name and the tip it was created from. Two checks are what make the word "scratch" mean something, and both
are run before and after rather than remembered:

```bash
git -C /opt/gunbc/gunbc rev-parse origin/roadmap-events              # identical before and after
ls -l --time-style=full-iso /opt/gunbc/fabric-storage | head        # untouched: no new mtime anywhere
```

If either moves during a dry run, the dry run has written to the live carrier: stop, say so in the PR, and do not
proceed to the live migration until the scratch is proved separate.
