# Roadmap event-log cutover — export → stop → migrate → verify → start

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

## 1. Read-only export of everything touched, immediately before any write

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

## 2. Stop every reader of this carrier — timers first

The operator ruled (relayed 2026-11-10): all five roadmap units move together to the staged release, and
`gunbc-roadmap-fabric-storage.service` is not touched at all. Stop the timer before the units it drives, so
a tick cannot fire into the window:

```bash
sudo systemctl stop gunbc-roadmap-belt-credential-recovery.timer
sudo systemctl stop gunbc-roadmap.service \
                    gunbc-roadmap-tailnet-door.service \
                    gunbc-roadmap-belt-discovery.service \
                    gunbc-roadmap-belt-exit.service \
                    gunbc-roadmap-belt-credential-recovery.service
# two of the belt oneshots are in `failed` state today; clear that so a later start is not refused
sudo systemctl reset-failed gunbc-roadmap-belt-exit.service \
                            gunbc-roadmap-belt-credential-recovery.service
systemctl is-active gunbc-roadmap.service gunbc-roadmap-tailnet-door.service   # expect: inactive, inactive
systemctl is-active gunbc-roadmap-belt-credential-recovery.timer               # expect: inactive
ps -ef | grep -c "[/]opt/gunbc/bin/gunbc"                                       # expect: 0 while stopped
```

Nothing else on srv1 reads this carrier; the enumeration, with each unit file, is under "Who else reads
this carrier" below, and the lab instance `gunbc-srv1-lab-codex-feedback-v0*` keeps its own carrier at
`/opt/gunbc-lab/codex-provider-feedback-v0/roadmap-events`.

## 2b. Stage only — the release is copied; the deploy is NOT run here, and not at this point

Staging is a copy and nothing else: a checkout at `/opt/gunbc/releases/<SHA>` and a binary at
`/opt/gunbc/bin/gunbc-<SHA>`, which the parent already put on srv1 (both still there after his rolled-back
attempt of 14:11/14:21). `BIN` below is that staged binary and `TREE` its checkout: every write in steps 3 to
5 goes through them, because the deployed Oct-15 build cannot read the kinds being appended. No unit file, no
fragment, no symlink moves — the five units are still the old ones, and they are stopped from step 2, so
staging activates nothing. **The deploy is step 5b, after `MIGRATION VERIFIED`.** It is not run in this PR:
it needs #16256 merged first, so that it leaves `gunbc-roadmap-fabric-storage.service` alone, and it needs the
operator's go-ahead.

## 3. Append the history first, in its recorded order — through the modeled route only

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
cd $TREE

# 17 acceptances, in their recorded order: fabric tail lines first (empty in practice today), then the
# seed's acceptance lines. The three pull-request arguments are empty BECAUSE THE FACT IS ABSENT — the
# seed recorded no pull request for any of them (see above); the route refuses a half-filled triple, an
# empty string where a value was recorded, and any absence dated on or after 2026-11-01.
while IFS= read -r LINE; do
  CMD="$BIN run --source-root dag --source-root src/v2 \
    --entry dag/gunbc/roadmap/roadmap_event_cli.dag --function roadmap_acceptance_record_cli \
    --arg repo_root=/opt/gunbc/gunbc --arg receipt_line=\"\$RECEIPT_LINE\" \
    --arg pr_url= --arg merged_at= --arg head_sha= \
    --arg author='$OPERATOR'"
  RUNE "RECEIPT_LINE=$LINE"
done < <( { sudo grep -rl 'roadmap-operator-acceptance' $D/fabric/storage 2>/dev/null | while read -r o; do
             sudo sed 's/^fabric_storage\.object;[0-9]*://' "$o"
           done | jq -r '.payload.line'; }
           grep acceptance_recorded $D/seed.jsonl )

# then the revocations, in recorded order, after the acceptance they name is live:
while IFS= read -r LINE; do
  CMD="$BIN run --source-root dag --source-root src/v2 \
    --entry dag/gunbc/roadmap/roadmap_event_cli.dag --function roadmap_acceptance_revoke_cli \
    --arg repo_root=/opt/gunbc/gunbc --arg revocation_line=\"\$REVOCATION_LINE\" \
    --arg author='$OPERATOR'"
  RUNE "REVOCATION_LINE=$LINE"
done < <(grep acceptance_revoked $D/seed.jsonl)
```

Each append prints its own refusal, verbatim and located, if the line is not spelled exactly as the
operator law spells one (`roadmap_acceptance_line_of` refuses, it never guesses a node or a receipt).
Stop and read the refusal before continuing — see the ordering note under this block.

What "an append succeeded" means: the CLIs return a `ProcessExit` (`ExitSuccess`, or `ExitFailure { code: 1,
reason: … }` whose reason the runtime prints, e.g. `append refused at <step>: <reason>`,
`roadmap_event_cli.dag:256`). Success carries **no event id** — that is why step 5's verifier output, not the
append output, is the per-line receipt.

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

## 4. Then the two retirements (step 1(a)) — the retirement is the newest fact, so it goes last

```bash
cd $TREE
for NODE in entry-graph-union-construction shell-dag-floor-peak-calibration-rows; do
  RUN "$BIN run --source-root dag --source-root src/v2 \
    --entry dag/gunbc/roadmap/roadmap_event_cli.dag --function roadmap_event_record_cli \
    --arg repo_root=/opt/gunbc/gunbc --arg node=$NODE --arg kind=retire \
    --arg text='The floor cut retired this work; its remaining cost lives on its successor.' \
    --arg author=$OPERATOR"
done
```

An already-retired node answers idempotently; a duplicate of an acceptance already held is refused at the
chain (`EventAlreadyPresent`) or at the operation record (the operation identity is the fact, not the node
— see `roadmap_acceptance_operation_identity`), which is the retry-safe answer, not a lost write.

## 5. Verify — the activation gate, and the source of the receipts

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

## 5b. The deploy — the FIRST activation of any new reader; NOT executed here; the operator's call

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
# moved at 03:05 cannot be activated on the 03:00 receipt. (2) The reference: gh workflow run takes a REF, and
# a bare commit SHA is not what the CLI documents — so the transaction is dispatched against a TAG minted at
# that revision, and the tag's remote resolution is checked against $SHA before anything is dispatched.
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
TAG="t6-cutover-$SHA"
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
  local before run tries got_mode got_sha
  before=$(gh run list --repo "$REPO" --workflow fleet-converge.yml --limit 50 --json databaseId -q '.[].databaseId' | sort -u | tr '\n' ' ')
  gh workflow run fleet-converge.yml --repo "$REPO" --ref "$TAG" -f host=srv1 -f mode="$want" "$@"
  run=''; tries=0
  while [ -z "$run" ]; do
    tries=$((tries + 1))
    if [ "$tries" -gt 90 ]; then echo "NO DEPLOY: no run appeared for mode=$want within 15 minutes"; exit 1; fi
    sleep 10
    run=$(gh run list --repo "$REPO" --workflow fleet-converge.yml --limit 50 \
            --json databaseId,headSha,event \
            -q "[.[] | select(.headSha==\"$SHA\" and .event==\"workflow_dispatch\") | .databaseId] - [$(printf '%s,' $before | sed 's/,$//')] | .[0] // empty")
  done
  got_mode=$(gh api "repos/$REPO/actions/runs/$run" -q '.inputs.mode // ""')
  got_sha=$(gh api "repos/$REPO/actions/runs/$run" -q '.head_sha')
  [ "$got_mode" = "$want" ] || { echo "NO DEPLOY: run $run declares mode=$got_mode, expected $want"; exit 1; }
  [ "$got_sha" = "$SHA" ]   || { echo "NO DEPLOY: run $run is at $got_sha, expected $SHA"; exit 1; }
  gh run watch "$run" --repo "$REPO" --exit-status \
    || { echo "NO DEPLOY: the $want run ($run) did not complete successfully"; exit 1; }
  STAGE_RUN="$run"
  echo "STAGE $want run=$STAGE_RUN mode=$got_mode revision=$got_sha"
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
echo "DEPLOYED tag=$TAG plan=$PLAN_RUN apply=$APPLY_RUN deploy=$DEPLOY_RUN plan_artifact_hash=$PLAN_HASH"
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


## 6. Read back through the reader production uses — step 5b's deploy is what brought the group up

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
# exit 0 = verifier + both readers + the write all admitted, in one document (step 5's artifact, regenerated):
RUN "$BIN run --source-root $TREE/dag --source-root $TREE/src/v2 --entry dag/gunbc/roadmap/roadmap_event_cli.dag --function roadmap_event_log_read_back_cli --arg repo_root=/opt/gunbc/gunbc --arg expected_manifest=\"$MANIFEST\" --arg retired_roster=\"$ROSTER\" --arg report_path=$D/read-back.after.out"
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

## 7. Rollback, if verification refuses

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
# export rather than from a list in this runbook, so a unit that was already down is not started by us.
sudo systemctl reset-failed "${UNITS[@]}" || true
if [ "$DISPOSITION" = 7B ]; then
  echo 'ROLLBACK 7B: re-enter step 5b — the modeled deploy is the only route that emits the unit definitions it starts.'
  echo 'Nothing was started here.'
else
  while read -r U STATE; do
    [ "$STATE" = active ] || continue
    sudo systemctl start "$U"
  done < <(paste -d' ' <(printf '%s\n' gunbc-roadmap.service gunbc-roadmap-tailnet-door.service \
      gunbc-roadmap-belt-discovery.service gunbc-roadmap-belt-exit.service \
      gunbc-roadmap-belt-credential-recovery.service) "$D/units.active.before")
  for U in gunbc-roadmap.service gunbc-roadmap-tailnet-door.service; do
    need "$U came up" test "$(systemctl is-active "$U")" = active
  done
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

A runbook is executable text: a shifted argument, a variable expanded in the wrong shell, or a check that only
prints is a defect that CI cannot see, because no CI job runs this file. Two witnesses live beside it and were run
against this file as committed — they extract the functions and guard blocks FROM THIS FILE at run time (and assert
byte-identity with it), so they cannot drift into testing a paraphrase:

```bash
bash tools/roadmap_cutover_runbook_witness.sh  docs/runbooks/roadmap-event-log-cutover.md   # parts A0/B0, A, B
bash tools/roadmap_cutover_dispatch_witness.sh docs/runbooks/roadmap-event-log-cutover.md   # step 5b's run identity
```

Nothing in them touches a real system: `sudo` is replaced by a function that strips the privilege, and
`systemd-run`, `systemctl` and `gh` resolve to stubs on `PATH`. They are run before the migration is authorized,
not after it fails. Their output at this revision:

### A0/B0 — the exercised text is this file's text

OK   RUNF, the transient-unit helper — the harness exercises the runbook's own text, byte-identical
OK   the export helper — the harness exercises the runbook's own text, byte-identical
OK   the restore helper — the harness exercises the runbook's own text, byte-identical
OK   the rollback's need predicate — the harness exercises the runbook's own text, byte-identical

### A — argument delivery through the actual helper invocation

The defect this exists for: `bash -s` claims `$0` for itself and makes **every** word after `--` a positional
parameter, so a helper written `bash -s -- runbook "$@"` would hand `$1` the literal string `runbook` — step 1
would export into a *relative* directory named `runbook`, and step 7 would read the wrong export. The helper is
`bash -s -- "$@"`, and each script additionally refuses a first argument that is not what it expects.

================ PART A: argument delivery through the ACTUAL helper shape ================
--- A1: correct helper, one data argument (step 1 passes $D) ---
GUARD HELD: D=/opt/gunbc/exports/t6-20261110T230000Z
A1 rc=0 — 0 and the absolute path echoed back are required
--- A2: the SHIFTED helper (a filler word after --), same script ---
RUNF REFUSED: the export directory arrived as [runbook] — argument delivery shifted
A2 rc=1 — nonzero plus the located refusal is the required behaviour
--- A3: correct helper, two data arguments (step 7 passes $D then $SHA) ---
GUARD HELD: D=/opt/gunbc/exports/t6-20261110T230000Z SHA=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
A3 rc=0 — 0 with both values intact is required
--- A4: the same script through the SHIFTED helper — the guard must refuse ---
ROLLBACK REFUSED: the export directory arrived as [runbook] — argument delivery shifted
A4 rc=1 — nonzero is required
--- A5: a revision that is not 40 hex is refused by step 7's own guard ---
ROLLBACK REFUSED: the revision arrived as [not-a-sha], which is not 40 hex
A5 rc=1 — nonzero is required

================ PART B: the drop-in round trip, pre-existing AND deployment-added ================

### B — the drop-in round trip, with a pre-existing drop-in AND a deployment-added one

Step 1 exports **every** affected unit's `<unit>.d` directory and step 7 puts each back **at its own path**
(`cp -a` of the directory, not of its contents — copying contents lands `override.conf` directly under
`/etc/systemd/system/`, where systemd does not read it as that unit's drop-in). The fixture starts with a
pre-existing drop-in on `gunbc-roadmap.service` and one on `gunbc-roadmap-tailnet-door.service`; the simulated
deploy then adds a new drop-in under the first unit and rewrites its unit file.

================ PART B: the drop-in round trip, pre-existing AND deployment-added ================
--- B1: export (step 1's helper) ---
units/drop-ins/gunbc-roadmap.service.d/pre-existing.conf
units/drop-ins/gunbc-roadmap-tailnet-door.service.d/10-deploy.conf
--- B2: the modeled deploy rewrites a unit file and adds a drop-in to a different unit ---
--- B3: restore (step 7's helper) ---
--- B4: assertions ---
ROLLBACK REFUSED: an unsatisfied predicate — nothing is started
ROUND TRIP OK: every unit's drop-in directory returned to its own path, byte-identical; the
               deployment-added one is gone; the pre-cutover ones (on both units) are back.
where the restored .conf files live: ./gunbc-roadmap.service.d/pre-existing.conf ./gunbc-roadmap-tailnet-door.service.d/10-deploy.conf 

ALL ASSERTIONS HOLD

### C — step 5b: the activation cannot follow a run it did not dispatch

`dispatch_stage` snapshots the run ids first, then polls for the run that appeared *after* its own dispatch and is
at `$SHA` with `event == workflow_dispatch`, checks that run's declared `mode` and `head_sha` against what it
dispatched, and only then watches it to completion — each stage's dispatch also carries
`plan_artifact_hash="$PLAN_HASH"`, which `admit_dashboard_dispatch_inputs` and `admit_dashboard_deploy_dispatch_inputs`
both require non-empty. Against a stubbed `gh`:

OK   the dispatched run is identified and admitted — admitted: STAGE plan run=21 mode=plan revision=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
OK   the newest run declares another mode — refused (rc=1): NO DEPLOY: run 21 declares mode=sweep, expected plan
OK   the run resolves to another revision — refused (rc=1): NO DEPLOY: run 21 is at bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, expected aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
OK   no new run appeared after the dispatch — refused (rc=1): NO DEPLOY: no run appeared for mode=plan within 15 minutes
OK   a pre-existing run cannot impersonate the dispatch — refused (rc=1): NO DEPLOY: no run appeared for mode=plan within 15 minutes
OK   the dispatched run did not conclude success — refused (rc=1): NO DEPLOY: the plan run (21) did not complete successfully
ALL DISPATCH-IDENTIFICATION CASES HOLD

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
