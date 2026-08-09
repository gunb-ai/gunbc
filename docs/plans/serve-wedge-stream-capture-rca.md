# Serve wedge stream-capture RCA (srv1 roadmap dashboard, 2026-08-09)

**Node:** `adhoc-02b6693e-e6a` · **Mitigation landed:** #8081 (evaluation budget kernel) · **This PR:** bounded shell stream capture + causal replay receipt

**Authority coordination:** `roadmap-serve-emitted-realization` in `gunbc.roadmap_authority` owns the broader serve wedge (emit-on-demand, interpreter concat cost-shape, request deadline, daily-page quadratic). This PR is an **additive** lane on the v1 shell exec seam only; it does not land that row's first_slice (concat fix, emit-on-demand wiring, GcpProjectId). Both findings share the **copied-accumulator family** — interpreted concat copies on every call; `wait_with_output` + `Value::Str(result.stderr.clone())` copies ~59.5 MiB the `.dag` ignores.

## Incident summary

On 2026-08-09 the srv1 `gunbc-roadmap` serve process wedged on a single core from 18:00:56 UTC with 11 TCP connections queued and none accepted. The process self-recovered without restart (`NRestarts=0`, MainPID 3364098, start 07:12:47). Deployed pin: `bb4048a5` (#7896).

This was a **cost-shape defect**, not unbounded recursion or non-termination:

| Signal | Measurement |
|--------|-------------|
| `perf` top frame | 98.5% in glibc aarch64 `memcpy` bulk loop |
| Call stack | `eval_expr_inner` → `eval_pure_named_call` → `eval_expr` |
| RSS | Flat over the wedge window |
| `VmStk` | 7.8 MB (shallow stack — not deep recursion) |
| `CALL_DEPTH_LIMIT` | 100_000 (unbounded recursion ruled out) |

Memcpy is proven; **quadratic concat is not yet measured** — that chars-walked curve at several page sizes is `roadmap-serve-emitted-realization` first_slice, not this PR.

## Wedged route — not proven

| Source | Claim |
|--------|-------|
| `roadmap-serve-emitted-realization` (2026-08-08) | Daily page render |
| Operator journal inference (2026-08-09) | `workflow.json` — attempt-state reads, then `belt-last-tick.json` |
| `gunbc.roadmap_serve` | Both routes consume the **same** attempt observation (`serve_daily_page_observed_note`) |

**Status:** route not established from primary logs in this session (srv1 SSH unavailable in agent environment). Both routes read attempt state and can invoke the jq projector over `provider-events.jsonl`; the stderr capture defect applies to **any** shell transport on the success path regardless of route.

## Falsified hypotheses (do not resurrect)

1. **Unbounded recursion** — stack shallow, depth limit 100k, evaluation finite (self-recovered).
2. **Nested `any` over dispatch refs** — only 17 refs in the hot closure.
3. **`parse_codex_jsonl` fed megabyte lines** — modeled jq projector runs first (`codex_provider_event_projection_filter` in `gunbc.roadmap_provider_events`); parser input measured 30,770 bytes / 523 lines / max line 61 chars.

## Measured causal chain (stderr capture)

### Artifact

`/opt/gunbc/attempt-state/v2-emitter-direct-rust-door-acc0923494fc8b9cb/provider-events.jsonl`

| Metric | Value |
|--------|-------|
| File size | 60,257,673 bytes |
| Lines | 831 |
| Non-JSON lines | 308 (plaintext tracing logs interleaved with protocol JSON) |

**Selection state:** this attempt is no longer selected on srv1 today (`workflow.json` ~0.38s); replay uses the measured artifact shape and a jq-like subprocess fixture.

### jq projection

| Stream | Bytes | Notes |
|--------|-------|-------|
| stdout | ~30 KB | Valid projected JSON |
| stderr | 59,522,411 | `fromjson` refuses bad lines, echoes them to stderr (longest line 193,620 chars) |
| jq wall / CPU | ~0.71s / ~0.37s per invocation | Exits 0 |

### Host capture bug (pre-fix)

`dispatch_shell` in `v1_interpreter.rs` used `Stdio::piped` + `wait_with_output`, materializing **both** streams fully. `map_shell_outputs` cloned stderr into `Value::Str` on success; discard guards require `exit_code != 0`. `Value::Str(String)` is the only non-`Rc` string carrier — later transport deep-copies during evaluation.

## Fix (this PR)

1. **Model** — `std.shell_stream_capture`: `StreamCapturePolicy` + `CapturedStreamObservation`.
2. **Realize** — `shell_stream_capture.rs`: concurrent drain **while** child runs; never `wait_with_output` then truncate.
3. **Wire** — `dispatch_shell` default policies: stdout `CaptureBounded` 8 MiB; stderr `CaptureDigestAndBoundedTail` 16 KiB + FNV digest.

## A/B replay (`bounded_shell_stream_capture_replay.rs`)

Three legs on one jq-like subprocess (small stdout, large stderr, exit 0):

| Leg | What |
|-----|------|
| A | Legacy `wait_with_output` full materialization |
| B | Production bounded concurrent drain |
| C | Protocol-only stdout fixture (separated diagnostics — production split is PR 2) |

Then **11 sequential** bounded requests vs 1 to falsify backlog multiplication of retained stderr strings.

**Honest CI-scale result:** at an 8 MiB stderr fixture, bounded wall time may exceed legacy (threaded drain vs single-buffer read). That does **not** falsify the production hypothesis — it redirects interpretation to retained-byte bounds and, if bounding does not move cost at incident scale, to the roadmap row's concat hypothesis.

| Outcome | Interpretation |
|---------|----------------|
| A slow, B fast, identical stdout | stderr materialization was principal amplifier |
| A and B both slow, C fast | mixed-stream projection / full-history work |
| All three slow | narrow declaration attribution; chars-walked curve per roadmap row |

## Acceptance mapping

| Criterion | Evidence |
|-----------|----------|
| stdout byte-identical A vs B | Replay harness `bounded_capture_preserves_stdout_bytes_against_legacy` |
| stderr total ~59.5 MB counted, retained ≤ 16 KiB | `StreamCaptureObservation.total_bytes` + witness |
| truncation explicit | `truncated` flag + digest on stderr policy |
| RSS bounded vs child stderr | `bounded_capture_rss_growth_not_proportional_to_child_stderr` |
| 11× sequential does not multiply retained stderr | `sequential_eleven_bounded_requests_do_not_multiply_retained_stderr` |
| RED on real shape | Witness `bounded_observation_refuses_oversized_retained_red_control` |

## Out of scope

| Item | Owner |
|------|-------|
| Interpreter concat fix, emit-on-demand, request deadline, GcpProjectId | `roadmap-serve-emitted-realization` |
| Channel separation | PR 2 |
| Incremental provider-state receipt | PR 3 |
| Production serve budgets + worker isolation | PR 4 |
| `Value::Str` → `Rc<str>` | PR 5 |
