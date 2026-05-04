# R3 CI Performance Profile 2026-05-04

## Scope

Profile/report only. No workflow behavior changes are proposed in this PR.

Inspected workflow definitions:

- `.github/workflows/ci.yml`

Inspected recent runs:

| PR | Run ID | Relevant result |
| --- | ---: | --- |
| #1663 | `25338908305` | `v3` passed in 8m17s; `self_host_ratchet` was still running past 8m; legacy `ci` failed in v2 compile gates. |
| #1664 | `25339309924` | `v3` passed in 7m49s; `self_host_ratchet` passed in 8m45s; legacy `ci` failed. |
| #1665 | `25341947478` | Fresh run was pending during the first profile; partial logs showed the same v3 shape, including a slower Stage 2d pass. |

## Per-Job Timing

| Job | Observed wall time | Dominant cost |
| --- | ---: | --- |
| `fmt` | 18-20s | Rust/toolchain setup and cache restore; `cargo fmt --all --check` itself is small. |
| legacy `ci` | About 9m11s before failure/pass boundary | v2 `run_ci_pipeline`, especially ignored `ci_*` tests and self-compile gates. |
| `v3` | 7m49s-8m17s | Full v3 integration suite, Stage 2d duplication, clippy/no-run compile costs, bootstrap verify. |
| `self_host_ratchet` | 8m45s-11m04s | Release compilation for DB-8 determinism and self-host fixed point; runtime is tiny. |

## Legacy `ci` Breakdown

Approximate timings from `25338908305` / `25339309924`:

| Step family | Observed time | Notes |
| --- | ---: | --- |
| Setup/cache | 9-30s | Repeated per job. Cache is mostly registry/git, not enough target reuse to avoid rebuild cost. |
| Script/doc ratchets | About 16s | Small relative to the job. |
| v3 bootstrap verify | 15-17s | Duplicated with the dedicated `v3` job. |
| v2 release build | 1m27s-1m34s | Compile-heavy setup before v2 pipeline gates. |
| v2 `run_ci_pipeline` | 6m22s-6m52s | Dominates legacy `ci`. |
| v2 ignored `ci_*` tests | 5m25s-5m49s inside `run_ci_pipeline` | Largest single sub-cost in the legacy lane. |
| v2 normal tests | 54-60s inside `run_ci_pipeline` | Material but not the main blocker. |
| v2 self-compile gates | About 77-90s plus about 66s | Expensive repeated compiler work. |

Conclusion: legacy `ci` cannot approach a one-minute target while v2 `run_ci_pipeline` remains required on PRs.

## `v3` Breakdown

Approximate timings from `25338908305` / `25339309924`, with partial `25341947478` confirmation:

| Step family | Observed time | Notes |
| --- | ---: | --- |
| Setup/cache/helper | 19-26s | Repeated Rust setup/build environment work. |
| Stage 2d focused test | 61-68s normally; partial `25341947478` showed 146s | Run as an explicit step, then overlapping coverage also appears in the full suite. |
| Full v3 suite | 226-247s | Primary v3 wall-clock cost. Integration tests run as one unsharded harness. |
| Per-test ratchet | Less than 1s | Not a bottleneck. |
| `cargo clippy` default | 30-34s | Compile/check cost. |
| `cargo clippy` with regen features | About 13s | Smaller but still repeated compilation/check work. |
| `cargo test --no-run --features bootstrap-regen-fresh` | 81-83s | Compile-only validation is a material cost. |
| Bootstrap regen verify | 15-18s | Also present in legacy `ci`. |
| L-7/L-8/compiler-std/banked ratchets | 0-1s each | Not bottlenecks. |

The v3 integration tests do not appear inherently serial from CI shape; they are currently unsharded because `tests/integration.rs` is one large harness. Safe shard boundaries look feasible by module family: substrate/projection/parser; emit roundtrip/matrix; runner/lens/thesis; and lib/unit tests as a separate job. The lowest-risk version is to split `tests/integration.rs` into multiple integration binaries by existing module family rather than filtering one binary many ways.

## `self_host_ratchet` Breakdown

Approximate timings:

| Step family | Observed time | Notes |
| --- | ---: | --- |
| Setup/cache | 16-27s | Repeated per job. |
| DB-8 determinism matrix, release | 4m19s-5m38s | Release compile dominates: about 4m14s compile time; test runtime about 3.6s. |
| self-host fixed point, release | 4m00s-4m50s | Release compile dominates: about 4m00s compile time; runtime about 0.06s. |

The self-host lane is compile-bound, not execution-bound. The expensive fixed-point and emit/determinism matrix work runs on every PR even when the changed files are not near self-host, DB-8, or emit determinism behavior.

## Redundancy Findings

- Rust setup and dependency/build preparation repeat across `fmt`, `ci`, `v3`, and `self_host_ratchet`.
- Cache strategy does not remove most target compilation cost; jobs repeatedly compile the same Rust crates in debug and release contexts.
- Bootstrap regeneration/verification is duplicated in legacy `ci` and `v3`.
- v3 Stage 2d runs as a focused step while overlapping coverage also runs in the full suite.
- DB-8 determinism appears in the v3 full suite and again in `self_host_ratchet` release mode.
- Legacy v2 self-compile work is independent of the v3 checks and dominates the old `ci` lane.

## Recommendations

### Quick Workflow-Only Wins

- Remove or gate the duplicate v3 bootstrap verify from legacy `ci` once ownership accepts the dedicated v3 job as the required verifier.
- Split required PR smoke from scheduled full `self_host_ratchet`. Keep a small required PR signal, and run the full release fixed-point/determinism matrix on schedule or path-relevant changes.
- Add path filters or conditional guards for self-host fixed point and DB-8 release checks when PRs only touch docs or unrelated substrate surfaces.
- Consider target-artifact caching only for stable, high-hit paths after measuring cache save/restore overhead; this is a quick config experiment but needs care.

### Requires Test Harness Work

- Split or shard `tests/integration.rs` by module family so v3 can run parallel required jobs instead of one large serial harness.
- Move DB-8 determinism out of the v3 full suite or skip the duplicate debug/release pairing once the dedicated self-host/DB-8 lane owns that signal.
- Avoid running Stage 2d both as a focused step and inside the full suite after coverage ownership signs off.

### Requires v2 Retirement

- Delete or de-require legacy v2 `ci` gates. The v2 `run_ci_pipeline` and self-compile gates are the only way legacy `ci` gets near the one-minute Director target.

### Risky / Defer

- Cross-job target caches can help but are easy to make slower or flaky because cache save/restore and invalidation costs are high. Measure before requiring.
- Dropping clippy, no-run, or bootstrap verify without a replacement owner would remove useful signals; defer until each signal has one required home.
- Rewriting workflow topology before v2 deletion risks spending effort around the dominant cost instead of removing it.

## Highest Leverage Next Actions

1. Land #1701 / v2 `ci` deletion. This is the only path for legacy `ci` to approach the target, and the current lane is actively blocking #1663/#1664.
2. Split required PR smoke from scheduled full `self_host_ratchet`; current release compile dominates and repeats DB-8 coverage.
3. Split/shard `tests/integration.rs`; v3 is unsharded rather than inherently serial.
4. Remove duplicated Stage 2d/full-suite execution and DB-8 debug+release duplication once ownership signs off.
