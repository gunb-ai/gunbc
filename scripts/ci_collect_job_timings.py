#!/usr/bin/env python3
"""Collect per-job wall-clock for the CURRENT GitHub Actions run, for the affected-set CI receipt.

Wave-1 §11.7 follow-up aggregator (the one the `affected` job comment promised): the kill-criterion
receipt's `actual_run_minutes` / `wall_clock_by_job` were emitted empty in v1 because the `affected`
job runs *before* any job timing exists. This script runs in a late `affected_timings` job, reads the
run's own job timings via the Actions API, and writes them in the shape the existing
`emit-affected-set-ci-receipt` bin already accepts (`--job-timings` + `--actual-run-minutes`). The bin
stays the single authority for the `saved_minutes` formula (src/.../receipt.rs); this script only
*projects* observed wall-clock, it computes no affected-set facts (RR-K §2.4 transport-only).

FAIL-SAFE: timing enrichment is best-effort and must NEVER fail the run. Any error (missing token,
API failure, malformed response) warns and writes an empty timings map + `0` minutes, exactly the v1
behavior — the receipt then carries no timings rather than failing CI.

Pure stdlib (urllib/json); no `gh`/`jq`/`requests` runner dependency.

Usage:
  ci_collect_job_timings.py <out_job_timings_json> <out_actual_minutes_file>
Env (provided by GitHub Actions):
  GITHUB_TOKEN | GH_TOKEN   token with actions:read
  GITHUB_REPOSITORY         "owner/repo"
  GITHUB_RUN_ID             numeric run id
  GITHUB_API_URL            API base (default https://api.github.com)
"""

import json
import os
import sys
import urllib.error
import urllib.request
from datetime import datetime


def _iso(s: str) -> datetime:
    # GitHub timestamps are RFC3339 UTC, e.g. "2026-06-13T01:13:28Z".
    return datetime.fromisoformat(s.replace("Z", "+00:00"))


def _warn(msg: str) -> None:
    # GitHub Actions warning annotation; visible but non-failing.
    print(f"::warning::ci_collect_job_timings: {msg}", file=sys.stderr)


def _fetch_jobs() -> list:
    token = os.environ.get("GITHUB_TOKEN") or os.environ.get("GH_TOKEN")
    repo = os.environ.get("GITHUB_REPOSITORY")
    run_id = os.environ.get("GITHUB_RUN_ID")
    api = os.environ.get("GITHUB_API_URL", "https://api.github.com")
    if not (token and repo and run_id):
        raise RuntimeError("missing GITHUB_TOKEN/GITHUB_REPOSITORY/GITHUB_RUN_ID")
    # This workflow has ~12 jobs; per_page=100 captures all without pagination.
    url = f"{api}/repos/{repo}/actions/runs/{run_id}/jobs?per_page=100"
    req = urllib.request.Request(
        url,
        headers={
            "Authorization": f"Bearer {token}",
            "Accept": "application/vnd.github+json",
            "X-GitHub-Api-Version": "2022-11-28",
        },
    )
    with urllib.request.urlopen(req, timeout=30) as resp:
        payload = json.load(resp)
    return payload.get("jobs", [])


def main() -> int:
    if len(sys.argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2
    out_timings, out_actual = sys.argv[1], sys.argv[2]

    job_timings: dict[str, int] = {}
    actual_run_minutes = 0.0
    try:
        jobs = _fetch_jobs()
        starts, ends = [], []
        for j in jobs:
            name, started, completed = j.get("name"), j.get("started_at"), j.get("completed_at")
            # Jobs still running (this aggregator itself) have no completed_at -> skip.
            if not (name and started and completed):
                continue
            s, e = _iso(started), _iso(completed)
            dur = int((e - s).total_seconds())
            if dur < 0:
                continue
            # Last writer wins on duplicate names (matrix/rerun); fine for a wall-clock ledger.
            job_timings[name] = dur
            starts.append(s)
            ends.append(e)
        if starts:
            # Observed wall-clock for the run = span from first job start to last job end.
            actual_run_minutes = round((max(ends) - min(starts)).total_seconds() / 60.0, 2)
    except (urllib.error.URLError, ValueError, KeyError, RuntimeError, OSError) as exc:
        _warn(f"{exc}; emitting empty timings (receipt keeps v1 zeros, run not failed)")
        job_timings, actual_run_minutes = {}, 0.0

    with open(out_timings, "w") as f:
        json.dump(job_timings, f, sort_keys=True)
    with open(out_actual, "w") as f:
        f.write(f"{actual_run_minutes}")
    print(
        f"collected {len(job_timings)} job timings; actual_run_minutes={actual_run_minutes}",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
