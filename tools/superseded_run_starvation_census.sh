#!/usr/bin/env bash
set -euo pipefail

# Closed, job-grain census for gunbc.superseded_run_starvation_census. Arguments are repository,
# workflow, inclusive run-creation window, and observation boundary. The full row roster remains
# in the output: null standings are coverage obligations and are never dropped from the denominator.

REPO=${1:?repo}
WORKFLOW=${2:?workflow}
FROM=${3:?from}
THROUGH=${4:?through}
OBSERVED_AT=${5:?observed-at}
MAX_RUN_PAGES=${6:?max-run-pages}
export FROM THROUGH OBSERVED_AT

DISCOVERY_FROM=$(date -u -d "$FROM - 1 day" +%Y-%m-%dT%H:%M:%SZ)
RUN_QUERY="repos/$REPO/actions/workflows/$WORKFLOW/runs?event=pull_request&per_page=100&created=$DISCOVERY_FROM..$THROUGH"
FIRST_PAGE=$(gh api "$RUN_QUERY&page=1")
TOTAL_RUNS=$(jq '.total_count' <<<"$FIRST_PAGE")
PAGES=$(((TOTAL_RUNS + 99) / 100))
[ "$PAGES" -le "$MAX_RUN_PAGES" ] || { echo "refusing: $PAGES run pages exceed the caller-declared $MAX_RUN_PAGES-page input envelope" >&2; exit 1; }

RUNS=$(for PAGE in $(seq 1 "$PAGES"); do
  # Read a left lookback so a dead run created before FROM is still eligible; the summary names
  # any PR whose first observed run overlaps FROM as a boundary obligation instead of omitting it.
  if [ "$PAGE" -eq 1 ]; then PAGE_JSON=$FIRST_PAGE; else PAGE_JSON=$(gh api "$RUN_QUERY&page=$PAGE"); fi
  printf '%s\n' "$PAGE_JSON"
  [ "$(jq '.workflow_runs | length' <<<"$PAGE_JSON")" -eq 100 ] || break
done | jq -sc '[.[].workflow_runs[]]')

PAIRS=$(jq -c '
  map(select((.pull_requests | length) > 0))
  | sort_by(.created_at)
  | group_by(.pull_requests[0].number)
  | map(sort_by(.created_at) as $runs
      | [range(0; ($runs|length)-1) as $i
         | $runs[$i] as $dead
         | ([range($i+1; $runs|length) as $j | $runs[$j] | select(.head_sha != $dead.head_sha)] | first) as $live
         | select($live != null and $live.created_at >= $ENV.FROM and $live.created_at <= $ENV.THROUGH and $live.created_at < $dead.updated_at)
         | {pr:$dead.pull_requests[0].number, dead_run:$dead.id, dead_sha:$dead.head_sha,
            dead_status:$dead.status, dead_conclusion:$dead.conclusion, dead_updated_at:$dead.updated_at,
            live_run:$live.id, live_sha:$live.head_sha, live_created_at:$live.created_at}]
    ) | add | unique_by([.dead_run,.live_run])' <<<"$RUNS")

LEFT_BOUNDARY=$(jq -c '
  map(select((.pull_requests | length) > 0)) | sort_by(.created_at) | group_by(.pull_requests[0].number)
  | map(first | select(.created_at < $ENV.FROM and .updated_at > $ENV.FROM)
      | {standing:"LeftBoundaryLookbackExhausted",pull_request:.pull_requests[0].number,earliest_observed_run:.id})' <<<"$RUNS")
export LEFT_BOUNDARY

jq -r '.[] | [.dead_run,.live_run] | @tsv' <<<"$PAIRS" |
while IFS=$'\t' read -r DEAD LIVE; do
  DEAD_RUN=$(gh api "repos/$REPO/actions/runs/$DEAD")
  DEAD_JOBS=$(gh api "repos/$REPO/actions/runs/$DEAD/jobs?per_page=100&filter=latest")
  LIVE_JOBS=$(gh api "repos/$REPO/actions/runs/$LIVE/jobs?per_page=100&filter=latest")
  jq -cn --argjson pair "$(jq -c --argjson id "$DEAD" '.[]|select(.dead_run==$id)' <<<"$PAIRS")" \
    --argjson dead_run "$DEAD_RUN" --argjson dead "$DEAD_JOBS" --argjson live "$LIVE_JOBS" --arg observed "$OBSERVED_AT" '
      ($live.jobs | map(.created_at) | min // null) as $live_first
      | ($dead.jobs | map(select(.started_at != null) | .started_at) | min // null) as $dead_first_start
      | ($dead.jobs | map(select(.completed_at != null)) | sort_by(.completed_at) | last // null) as $last_job
      | (if $last_job == null or $last_job.started_at == null then null else
           ([$dead.jobs[] | select(.id != $last_job.id and .completed_at != null and .completed_at <= $last_job.started_at) | .completed_at] | max // null)
         end) as $prior_completion
      | $pair + {
          dead_status: $dead_run.status, dead_conclusion: $dead_run.conclusion,
          dead_updated_at: $dead_run.updated_at,
          dead_job_count: ($dead.jobs|length), live_job_count: ($live.jobs|length),
          dead_first_job_started_at: $dead_first_start, live_first_job_created_at: $live_first,
          start_standing: (
            if $dead_first_start != null and $live_first == null then "DeadHeadReachedInProgressFirst"
            elif $dead_first_start == null and $live_first != null then "LiveHeadRegisteredJobFirst"
            elif $dead_first_start == null and $live_first == null then "NeitherRunRegisteredAJob"
            elif $dead_first_start < $live_first then "DeadHeadReachedInProgressFirst"
            else "LiveHeadRegisteredJobFirst" end),
          cancelled_jobs: ([$dead.jobs[]|select(.conclusion=="cancelled")]|length),
          failed_jobs: ([$dead.jobs[]|select(.conclusion=="failure")]|length),
          queued_jobs: ([$dead.jobs[]|select(.status=="queued")]|length),
          terminal_laggard_started_at: ($last_job.started_at // null),
          prior_jobs_completed_at: $prior_completion,
          dead_time_under_hold_seconds: (if $prior_completion == null or $last_job.started_at == null then null
            else (($last_job.started_at|fromdateiso8601)-($prior_completion|fromdateiso8601)) end),
          group_hold_after_jobs_seconds: (if $last_job.completed_at == null or $dead_run.status != "completed" then null
            else (($dead_run.updated_at|fromdateiso8601)-($last_job.completed_at|fromdateiso8601)) end)
        }'
done | jq -s '
  . as $rows
  | {
      window: {from: $ENV.FROM, through: $ENV.THROUGH, observed_at: $ENV.OBSERVED_AT},
      left_boundary_obligations: ($ENV.LEFT_BOUNDARY | fromjson),
      pair_count: length,
      start_standings: (group_by(.start_standing) | map({standing: .[0].start_standing, count: length})),
      cancellation_rows: ([$rows[] | select(.cancelled_jobs > 0)] | map({pr,dead_run,cancelled_jobs,failed_jobs,queued_jobs,dead_time_under_hold_seconds,group_hold_after_jobs_seconds})),
      rows: $rows
    }'
